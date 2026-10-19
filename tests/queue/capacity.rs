//! Capacity management for `Queue`: growth, reservation and shrinking.

use rust_stl::collections::queue::Queue;

#[test]
fn enqueue_across_many_growths_preserves_values_and_fifo_order() {
    let mut queue: Queue<u32> = Queue::new();
    let mut previous_capacity = queue.capacity();

    for i in 0..1_000 {
        queue.enqueue(i);
        assert!(queue.capacity() >= queue.len());
        assert!(
            queue.capacity() >= previous_capacity,
            "capacity never shrinks"
        );
        previous_capacity = queue.capacity();
    }

    assert!(queue.capacity() > 4);

    for i in 0..1_000 {
        assert_eq!(queue.dequeue(), Some(i));
    }
    assert!(queue.is_empty());
}

#[test]
fn with_capacity_reserves_room_without_adding_elements() {
    let mut queue: Queue<i32> = Queue::with_capacity(16);

    assert!(queue.capacity() >= 16);
    assert_eq!(queue.len(), 0);
    assert!(queue.is_empty());

    queue.extend(0..16);
    assert_eq!(queue.len(), 16);
    assert_eq!(queue.front(), Some(&0));
    assert_eq!(queue.back(), Some(&15));
    assert!(queue.capacity() >= 16);
}

#[test]
fn reserve_provides_at_least_the_requested_additional_slots() {
    let mut queue: Queue<i32> = (0..4).collect();

    queue.reserve(10);

    assert!(queue.capacity() >= 14);
    assert_eq!(queue.len(), 4);
    assert_eq!(queue.front(), Some(&0));

    queue.reserve(0);
    assert_eq!(queue.len(), 4);
    assert_eq!(queue.front(), Some(&0));
}

#[test]
fn reserve_exact_fits_len_plus_additional() {
    let mut queue: Queue<i32> = Queue::new();

    queue.reserve_exact(8);
    assert!(queue.capacity() >= 8);

    queue.extend(0..8);
    assert_eq!(queue.len(), 8);
    assert_eq!(queue.dequeue(), Some(0));
}

#[test]
fn reserve_does_not_lose_elements() {
    let mut queue: Queue<String> = Queue::new();
    for i in 0..32 {
        queue.enqueue(format!("value-{i}"));
    }

    queue.reserve(1_024);
    assert!(queue.capacity() >= 1_056);

    for i in 0..32 {
        assert_eq!(queue.dequeue(), Some(format!("value-{i}")));
    }
}

#[test]
fn clear_retains_capacity_and_the_queue_stays_usable() {
    let mut queue: Queue<i32> = (0..16).collect();
    let capacity = queue.capacity();
    assert!(capacity >= 16);

    queue.clear();

    assert!(queue.is_empty());
    assert_eq!(queue.capacity(), capacity);

    queue.extend(0..3);
    assert_eq!(queue.capacity(), capacity);
    assert_eq!(queue.dequeue(), Some(0));
}

#[test]
fn shrink_to_fit_reduces_capacity_and_keeps_values() {
    let mut queue: Queue<i32> = Queue::with_capacity(128);
    queue.extend(0..10);

    queue.shrink_to_fit();

    assert_eq!(queue.capacity(), 10);
    assert_eq!(queue.len(), 10);
    assert_eq!(queue.front(), Some(&0));
    assert_eq!(queue.back(), Some(&9));
}

#[test]
fn shrink_to_fit_on_empty_queue_releases_storage_and_stays_usable() {
    let mut queue: Queue<i32> = Queue::with_capacity(64);

    queue.shrink_to_fit();

    assert_eq!(queue.capacity(), 0);
    assert!(queue.is_empty());

    queue.enqueue(1);
    assert_eq!(queue.dequeue(), Some(1));
}

#[test]
fn capacity_is_never_reported_smaller_than_length() {
    let mut queue: Queue<i32> = Queue::new();

    for i in 0..200 {
        queue.enqueue(i);
        assert!(queue.capacity() >= queue.len());
    }

    queue.shrink_to_fit();
    assert!(queue.capacity() >= queue.len());
}

#[test]
fn try_reserve_matches_underlying_contract() {
    let mut queue: Queue<i32> = Queue::new();
    assert!(queue.try_reserve(8).is_ok());
    assert!(queue.capacity() >= 8);

    // Overflow is reported rather than panicking.
    let mut q: Queue<i32> = Queue::new();
    q.enqueue(1);
    assert_eq!(q.try_reserve(usize::MAX), Err("capacity overflow"));
}
