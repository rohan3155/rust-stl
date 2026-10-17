//! Destructor correctness and ownership transfer for `Stack`.
//!
//! Every test here asserts *exact* drop counts, so a double drop or a leak fails
//! the test rather than passing silently.

use rust_stl::collections::stack::Stack;

use crate::common::{DropCounter, DropTracker, DropZst, zst_drops};

/// Builds a stack of `n` `DropCounter`s with ids `0..n`, id 0 at the bottom.
fn counted(n: i32, tracker: &DropTracker) -> Stack<DropCounter> {
    let mut stack = Stack::new();
    for i in 0..n {
        stack.push(DropCounter::new(i, tracker));
    }
    stack
}

#[test]
fn dropping_a_stack_drops_every_element_once() {
    let tracker = DropTracker::new();
    {
        let stack = counted(5, &tracker);
        assert_eq!(stack.len(), 5);
        assert_eq!(tracker.count(), 0);
    }
    assert_eq!(tracker.count(), 5);
}

#[test]
fn dropping_an_empty_stack_drops_nothing() {
    let tracker = DropTracker::new();
    {
        let stack: Stack<DropCounter> = Stack::new();
        drop(stack);
    }
    assert_eq!(tracker.count(), 0);
}

#[test]
fn pop_transfers_ownership_and_the_stack_does_not_drop_it_again() {
    let tracker = DropTracker::new();
    let mut stack = counted(3, &tracker);

    let popped = stack.pop().expect("an element");
    assert_eq!(popped.id, 2);
    assert_eq!(tracker.count(), 0, "a returned element must stay alive");

    drop(popped);
    assert_eq!(tracker.count(), 1);

    drop(stack);
    assert_eq!(tracker.count(), 3);
}

#[test]
fn clear_drops_every_element_once_and_is_idempotent() {
    let tracker = DropTracker::new();
    let mut stack = counted(6, &tracker);

    stack.clear();
    assert_eq!(tracker.count(), 6);
    assert!(stack.is_empty());

    // A second clear must not drop anything again.
    stack.clear();
    assert_eq!(tracker.count(), 6);

    drop(stack);
    assert_eq!(tracker.count(), 6);
}

#[test]
fn clear_after_a_partial_pop_drops_only_the_remaining() {
    let tracker = DropTracker::new();
    let mut stack = counted(5, &tracker);

    let popped = stack.pop().expect("the top");
    assert_eq!(popped.id, 4);
    assert_eq!(tracker.count(), 0);

    drop(popped);
    assert_eq!(tracker.count(), 1);

    stack.clear();
    assert_eq!(tracker.count(), 5, "the four remaining plus the popped one");
}

#[test]
fn draining_the_stack_leaves_nothing_for_the_stack_to_drop() {
    let tracker = DropTracker::new();
    let mut stack = counted(4, &tracker);

    let drained: Vec<DropCounter> = std::iter::from_fn(|| stack.pop()).collect();
    assert_eq!(
        drained.iter().map(|element| element.id).collect::<Vec<_>>(),
        [3, 2, 1, 0]
    );
    assert_eq!(tracker.count(), 0);

    drop(stack);
    assert_eq!(tracker.count(), 0, "the drained stack owns nothing");

    drop(drained);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn zero_sized_type_with_drop_has_exact_destructor_counts() {
    let before = zst_drops();

    {
        let mut stack: Stack<DropZst> = Stack::new();
        for _ in 0..100 {
            stack.push(DropZst);
        }
        assert_eq!(stack.len(), 100);
        assert_eq!(zst_drops(), before, "pushing ZSTs must not drop anything");

        let popped = stack.pop();
        assert_eq!(
            zst_drops(),
            before,
            "pop returns the value, dropping nothing"
        );
        drop(popped);
        assert_eq!(zst_drops(), before + 1);

        stack.clear();
        assert_eq!(zst_drops(), before + 100, "the other 99 drop exactly once");
    }

    // Fully consuming a stack drops each element exactly once.
    let mut stack: Stack<DropZst> = Stack::new();
    for _ in 0..5 {
        stack.push(DropZst);
    }
    let consumed = stack.into_iter().count();
    assert_eq!(consumed, 5);
    assert_eq!(zst_drops(), before + 105);

    // This is the only test that uses the process-wide ZST counter, so nothing
    // else may have added to it.
    assert_eq!(zst_drops(), before + 105);
}

#[test]
fn partially_consuming_the_owned_iterator_drops_the_rest_once() {
    let tracker = DropTracker::new();
    let stack = counted(5, &tracker);

    let mut iter = stack.into_iter();
    let first = iter.next().expect("the bottom element");
    assert_eq!(first.id, 0);
    assert_eq!(tracker.count(), 0);

    drop(iter);
    assert_eq!(tracker.count(), 4, "the four remaining drop exactly once");

    drop(first);
    assert_eq!(tracker.count(), 5);
}
