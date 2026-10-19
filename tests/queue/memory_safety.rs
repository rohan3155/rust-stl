//! Destructor correctness and ownership transfer for `Queue`.

use rust_stl::collections::queue::Queue;

use crate::common::{DropCounter, DropTracker, DropZst, zst_drops};

fn counted(n: i32, tracker: &DropTracker) -> Queue<DropCounter> {
    let mut queue = Queue::new();
    for i in 0..n {
        queue.enqueue(DropCounter::new(i, tracker));
    }
    queue
}

#[test]
fn dropping_a_queue_drops_every_element_once() {
    let tracker = DropTracker::new();
    {
        let queue = counted(5, &tracker);
        assert_eq!(queue.len(), 5);
        assert_eq!(tracker.count(), 0);
    }
    assert_eq!(tracker.count(), 5);
}

#[test]
fn dropping_an_empty_queue_drops_nothing() {
    let tracker = DropTracker::new();
    {
        let queue: Queue<DropCounter> = Queue::new();
        drop(queue);
    }
    assert_eq!(tracker.count(), 0);
}

#[test]
fn dequeue_transfers_ownership_and_the_queue_does_not_drop_it_again() {
    let tracker = DropTracker::new();
    let mut queue = counted(3, &tracker);

    let dequeued = queue.dequeue().expect("an element");
    assert_eq!(dequeued.id, 0);
    assert_eq!(tracker.count(), 0, "a returned element must stay alive");

    drop(dequeued);
    assert_eq!(tracker.count(), 1);

    drop(queue);
    assert_eq!(tracker.count(), 3);
}

#[test]
fn clear_drops_every_element_once_and_is_idempotent() {
    let tracker = DropTracker::new();
    let mut queue = counted(6, &tracker);

    queue.clear();
    assert_eq!(tracker.count(), 6);
    assert!(queue.is_empty());

    queue.clear();
    assert_eq!(tracker.count(), 6);

    drop(queue);
    assert_eq!(tracker.count(), 6);
}

#[test]
fn clear_after_a_partial_dequeue_drops_only_the_remaining() {
    let tracker = DropTracker::new();
    let mut queue = counted(5, &tracker);

    let dequeued = queue.dequeue().expect("the front");
    assert_eq!(dequeued.id, 0);
    assert_eq!(tracker.count(), 0);

    drop(dequeued);
    assert_eq!(tracker.count(), 1);

    queue.clear();
    assert_eq!(
        tracker.count(),
        5,
        "the four remaining plus the dequeued one"
    );
}

#[test]
fn draining_the_queue_leaves_nothing_for_the_queue_to_drop() {
    let tracker = DropTracker::new();
    let mut queue = counted(4, &tracker);

    let drained: Vec<DropCounter> = std::iter::from_fn(|| queue.dequeue()).collect();
    assert_eq!(
        drained.iter().map(|element| element.id).collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert_eq!(tracker.count(), 0);

    drop(queue);
    assert_eq!(tracker.count(), 0, "the drained queue owns nothing");

    drop(drained);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn zero_sized_type_with_drop_has_exact_destructor_counts() {
    let before = zst_drops();

    {
        let mut queue: Queue<DropZst> = Queue::new();
        for _ in 0..100 {
            queue.enqueue(DropZst);
        }
        assert_eq!(queue.len(), 100);
        assert_eq!(
            zst_drops(),
            before,
            "enqueueing ZSTs must not drop anything"
        );

        let dequeued = queue.dequeue();
        assert_eq!(
            zst_drops(),
            before,
            "dequeue returns the value, dropping nothing"
        );
        drop(dequeued);
        assert_eq!(zst_drops(), before + 1);

        queue.clear();
        assert_eq!(zst_drops(), before + 100, "the other 99 drop exactly once");
    }

    let mut queue: Queue<DropZst> = Queue::new();
    for _ in 0..5 {
        queue.enqueue(DropZst);
    }
    let consumed = queue.into_iter().count();
    assert_eq!(consumed, 5);
    assert_eq!(zst_drops(), before + 105);

    assert_eq!(zst_drops(), before + 105);
}

#[test]
fn partially_consuming_the_owned_iterator_drops_the_rest_once() {
    let tracker = DropTracker::new();
    let queue = counted(5, &tracker);

    let mut iter = queue.into_iter();
    let first = iter.next().expect("the front element");
    assert_eq!(first.id, 0);
    assert_eq!(tracker.count(), 0);

    drop(iter);
    assert_eq!(tracker.count(), 4, "the four unyielded values drop once");

    drop(first);
    assert_eq!(tracker.count(), 5);
}

#[test]
fn retain_drops_rejected_elements_once() {
    let tracker = DropTracker::new();
    let mut queue = counted(5, &tracker);

    queue.retain(|element| element.id % 2 == 0);
    assert_eq!(queue.len(), 3);
    assert_eq!(tracker.count(), 2, "two rejected elements dropped");

    drop(queue);
    assert_eq!(tracker.count(), 5);
}

#[test]
fn append_does_not_lose_or_duplicate_ownership() {
    let tracker = DropTracker::new();
    let mut head = counted(2, &tracker);
    let mut tail = counted(3, &tracker);

    head.append(&mut tail);
    assert_eq!(head.len(), 5);
    assert!(tail.is_empty());
    assert_eq!(tracker.count(), 0);

    drop(head);
    assert_eq!(tracker.count(), 5);
}
