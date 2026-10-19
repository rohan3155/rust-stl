//! Iterator specifics for `Queue`.

use rust_stl::collections::queue::Queue;

#[test]
fn size_hint_and_exact_size_after_partial_dequeue() {
    let mut queue: Queue<i32> = (0..5).collect();
    assert_eq!(queue.dequeue(), Some(0));

    let iter = queue.iter();
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.count(), 4);

    let mut_iter = queue.iter_mut();
    assert_eq!(mut_iter.size_hint(), (4, Some(4)));
    assert_eq!(mut_iter.count(), 4);
}

#[test]
fn owned_iterator_size_hint() {
    let queue: Queue<i32> = (0..6).collect();
    let mut iter = queue.into_iter();
    assert_eq!(iter.size_hint(), (6, Some(6)));
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.size_hint(), (5, Some(5)));
}

#[test]
fn iteration_after_empty_and_growth() {
    let mut queue: Queue<i32> = Queue::new();
    assert_eq!((&queue).into_iter().count(), 0);

    for i in 0..100 {
        queue.enqueue(i);
    }
    let collected: Vec<i32> = queue.iter().copied().collect();
    assert_eq!(collected.len(), 100);
    assert_eq!(collected[0], 0);
    assert_eq!(collected[99], 99);
}

#[test]
fn mutable_iteration_updates_after_dequeue() {
    let mut queue: Queue<i32> = (0..4).collect();
    assert_eq!(queue.dequeue(), Some(0));

    for value in &mut queue {
        *value += 1;
    }
    assert_eq!(queue.as_slice(), &[2, 3, 4]);
}
