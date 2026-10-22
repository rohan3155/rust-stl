use crate::common::{DropCounter, DropTracker};
use rust_stl::collections::deque::Deque;

#[test]
fn pop_front_transfers_ownership() {
    let tracker = DropTracker::new();
    let mut d: Deque<DropCounter> = Deque::new();
    for i in 0..4 {
        d.push_back(DropCounter::new(i, &tracker));
    }
    let first = d.pop_front().expect("front");
    assert_eq!(first.id, 0);
    assert_eq!(tracker.count(), 0);
    drop(first);
    assert_eq!(tracker.count(), 1);
    drop(d);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn pop_back_transfers_ownership() {
    let tracker = DropTracker::new();
    let mut d: Deque<DropCounter> = Deque::new();
    for i in 0..4 {
        d.push_back(DropCounter::new(i, &tracker));
    }
    let last = d.pop_back().expect("back");
    assert_eq!(last.id, 3);
    assert_eq!(tracker.count(), 0);
    drop(last);
    assert_eq!(tracker.count(), 1);
    drop(d);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn owned_iter_drops_unyielded() {
    let tracker = DropTracker::new();
    let mut d: Deque<DropCounter> = Deque::new();
    for i in 0..4 {
        d.push_back(DropCounter::new(i, &tracker));
    }
    let mut it = d.into_iter();
    let first = it.next().unwrap();
    assert_eq!(first.id, 0);
    assert_eq!(tracker.count(), 0);
    drop(it);
    assert_eq!(tracker.count(), 3);
    drop(first);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn clear_drops_all() {
    let tracker = DropTracker::new();
    let mut d: Deque<DropCounter> = Deque::new();
    for i in 0..4 {
        d.push_back(DropCounter::new(i, &tracker));
    }
    d.clear();
    assert_eq!(tracker.count(), 4);
}

#[test]
fn push_front_after_pop_reuses_without_double_drop() {
    let tracker = DropTracker::new();
    let mut d: Deque<DropCounter> = Deque::new();
    for i in 0..4 {
        d.push_back(DropCounter::new(i, &tracker));
    }
    d.pop_front();
    d.pop_front();
    d.push_front(DropCounter::new(99, &tracker));
    assert_eq!(d.len(), 3);
    drop(d);
    assert_eq!(tracker.count(), 5);
}
