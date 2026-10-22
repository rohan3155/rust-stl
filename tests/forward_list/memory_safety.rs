use crate::common::{DropCounter, DropTracker};
use rust_stl::collections::forward_list::ForwardList;

#[test]
fn pop_front_transfers_ownership() {
    let tracker = DropTracker::new();
    let mut list: ForwardList<DropCounter> = ForwardList::new();
    for i in 0..4 {
        list.push_back(DropCounter::new(i, &tracker));
    }
    let v = list.pop_front().unwrap();
    assert_eq!(v.id, 0);
    assert_eq!(tracker.count(), 0);
    drop(v);
    assert_eq!(tracker.count(), 1);
    drop(list);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn owned_iter_drops_remaining() {
    let tracker = DropTracker::new();
    let mut list: ForwardList<DropCounter> = ForwardList::new();
    for i in 0..4 {
        list.push_back(DropCounter::new(i, &tracker));
    }
    let mut it = list.into_iter();
    let first = it.next().unwrap();
    assert_eq!(first.id, 0);
    drop(it);
    assert_eq!(tracker.count(), 3);
    drop(first);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn clear_drops_all() {
    let tracker = DropTracker::new();
    let mut list: ForwardList<DropCounter> = ForwardList::new();
    for i in 0..4 {
        list.push_back(DropCounter::new(i, &tracker));
    }
    list.clear();
    assert_eq!(tracker.count(), 4);
}
