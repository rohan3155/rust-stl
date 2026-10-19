//! Edge cases for `Queue`: empty, single-element, zero-sized and nested values.

use rust_stl::collections::queue::Queue;

#[derive(Debug, PartialEq, Eq)]
struct Marker;

#[test]
fn empty_queue_supports_every_read_only_operation() {
    let queue: Queue<i32> = Queue::new();

    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.capacity(), 0);
    assert_eq!(queue.front(), None);
    assert_eq!(queue.back(), None);
    assert_eq!(queue.get(0), None);
    assert_eq!((&queue).into_iter().count(), 0);
    assert_eq!(format!("{queue:?}"), "[]");
}

#[test]
fn empty_queue_supports_every_mutating_operation() {
    let mut queue: Queue<i32> = Queue::new();

    queue.clear();
    queue.reserve(0);
    queue.reserve_exact(0);
    queue.shrink_to_fit();
    queue.extend(std::iter::empty::<i32>());

    assert!(queue.is_empty());
    assert_eq!(queue.dequeue(), None);

    let mut other: Queue<i32> = Queue::new();
    queue.swap(&mut other);
    assert!(queue.is_empty());
    assert!(other.is_empty());
}

#[test]
fn swap_exchanges_contents_and_capacity() {
    let mut a: Queue<i32> = (0..3).collect();
    let mut b: Queue<i32> = (10..12).collect();
    let a_capacity = a.capacity();
    let b_capacity = b.capacity();

    a.swap(&mut b);

    assert_eq!(a.len(), 2);
    assert_eq!(a.front(), Some(&10));
    assert_eq!(b.len(), 3);
    assert_eq!(b.front(), Some(&0));
    assert_eq!(a.capacity(), b_capacity);
    assert_eq!(b.capacity(), a_capacity);

    assert_eq!(a.dequeue(), Some(10));
    assert_eq!(b.dequeue(), Some(0));
}

#[test]
fn unit_type_is_a_zero_sized_type() {
    let mut queue: Queue<()> = Queue::new();
    assert_eq!(queue.capacity(), 0);

    for _ in 0..1_000 {
        queue.enqueue(());
    }

    assert_eq!(queue.len(), 1_000);
    assert_eq!(queue.capacity(), usize::MAX);
    assert_eq!(queue.front(), Some(&()));
    assert_eq!(queue.dequeue(), Some(()));
    assert_eq!(queue.len(), 999);

    queue.clear();
    assert!(queue.is_empty());
}

#[test]
fn zero_sized_type_reserve_and_shrink_do_not_touch_the_allocator() {
    let mut queue: Queue<Marker> = Queue::new();

    queue.reserve(10_000);
    queue.reserve_exact(10_000);
    queue.shrink_to_fit();

    queue.enqueue(Marker);
    queue.enqueue(Marker);

    assert_eq!(queue.len(), 2);
    assert_eq!(queue.front(), Some(&Marker));
    assert_eq!(queue.dequeue(), Some(Marker));
    assert_eq!(queue.dequeue(), Some(Marker));
    assert_eq!(queue.dequeue(), None);
}

#[test]
fn nested_queues_work() {
    let mut outer: Queue<Queue<i32>> = Queue::new();
    outer.enqueue((0..2).collect());
    outer.enqueue((2..4).collect());

    assert_eq!(outer.front().unwrap().front(), Some(&0));

    outer.front_mut().unwrap().enqueue(4);
    assert_eq!(outer.front().unwrap().len(), 3);

    let dequeued = outer.dequeue().expect("the front queue");
    assert_eq!(dequeued.front(), Some(&0));
    assert_eq!(outer.len(), 1);
    assert_eq!(outer.front().unwrap().front(), Some(&2));
}

#[test]
fn a_large_queue_round_trips() {
    let mut queue: Queue<usize> = Queue::new();

    for i in 0..10_000 {
        queue.enqueue(i);
    }
    assert_eq!(queue.len(), 10_000);

    for i in 0..10_000 {
        assert_eq!(queue.dequeue(), Some(i));
    }
    assert!(queue.is_empty());
}

#[test]
fn duplicate_values_are_preserved() {
    let mut queue: Queue<i32> = Queue::new();
    queue.enqueue(7);
    queue.enqueue(7);
    queue.enqueue(7);

    assert_eq!(queue.len(), 3);
    assert_eq!(queue.dequeue(), Some(7));
    assert_eq!(queue.dequeue(), Some(7));
    assert_eq!(queue.dequeue(), Some(7));
}

#[test]
fn enqueue_after_dequeue_and_clear() {
    let mut queue: Queue<i32> = (0..3).collect();
    assert_eq!(queue.dequeue(), Some(0));
    queue.enqueue(9);
    assert_eq!(queue.as_slice(), &[1, 2, 9]);

    queue.clear();
    queue.enqueue(5);
    assert_eq!(queue.as_slice(), &[5]);
}
