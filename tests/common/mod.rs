//! Shared helpers for the integration tests.
//!
//! Kept deliberately small: a drop-counting element type that is safe to use
//! from tests running in parallel, plus a couple of trivial ZSTs.
#![allow(dead_code)]

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// An owned counter shared between a test and the elements it creates.
#[derive(Clone, Default)]
pub struct DropTracker(Arc<AtomicUsize>);

impl DropTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of [`DropCounter`] / [`PanicOnClone`] values dropped so far.
    pub fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }

    fn record(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// An element that increments a [`DropTracker`] exactly once when destroyed.
///
/// Holding the tracker in an `Arc` (rather than a `static`) means every test
/// gets its own counter and the suite stays correct under parallel execution.
pub struct DropCounter {
    pub id: i32,
    tracker: DropTracker,
}

impl DropCounter {
    pub fn new(id: i32, tracker: &DropTracker) -> Self {
        Self {
            id,
            tracker: tracker.clone(),
        }
    }

    /// Builds `len` elements with ids `0..len`.
    pub fn range(len: i32, tracker: &DropTracker) -> Vec<DropCounter> {
        (0..len).map(|id| DropCounter::new(id, tracker)).collect()
    }
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.tracker.record();
    }
}

impl Clone for DropCounter {
    /// Cloning must *not* count as a drop: it creates a second live value.
    fn clone(&self) -> Self {
        Self::new(self.id, &self.tracker)
    }
}

impl PartialEq for DropCounter {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DropCounter {}

impl fmt::Debug for DropCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DropCounter").field("id", &self.id).finish()
    }
}

/// A zero-sized type that records its drops in a shared counter.
///
/// A ZST cannot store the `Arc` inline, so — like `std`'s ZST tests — the
/// tracker has to live outside the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zst;

pub struct ZstTracker;

static ZST_DROPS: AtomicUsize = AtomicUsize::new(0);

impl ZstTracker {
    /// The number of [`DropZst`] values dropped since the process started.
    ///
    /// Reading and mutating this counter is only meaningful while no other test
    /// is dropping a `DropZst`, so exactly one test in the suite uses it.
    pub fn count() -> usize {
        ZST_DROPS.load(Ordering::SeqCst)
    }
}

impl Drop for ZstTracker {
    fn drop(&mut self) {
        ZST_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

/// The zero-sized element type used by the ZST destructor test.
pub type DropZst = ZstTracker;

/// A cloneable element whose `Clone` panics on the `nth` clone.
pub struct PanicOnClone {
    pub id: i32,
    pub panic_at: usize,
    pub clones: Arc<AtomicUsize>,
    tracker: DropTracker,
}

impl PanicOnClone {
    pub fn new(id: i32, panic_at: usize, tracker: &DropTracker) -> Self {
        Self {
            id,
            panic_at,
            clones: Arc::new(AtomicUsize::new(0)),
            tracker: tracker.clone(),
        }
    }
}

impl Clone for PanicOnClone {
    fn clone(&self) -> Self {
        let n = self.clones.fetch_add(1, Ordering::SeqCst);
        if n == self.panic_at {
            panic!("clone panic requested by test");
        }
        Self {
            id: self.id,
            panic_at: self.panic_at,
            clones: self.clones.clone(),
            tracker: self.tracker.clone(),
        }
    }
}

impl Drop for PanicOnClone {
    fn drop(&mut self) {
        self.tracker.record();
    }
}
