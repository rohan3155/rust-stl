//! Standard trait implementations and the three iteration forms for `Queue`.

use rust_stl::collections::queue::Queue;
use std::fmt::Debug;

use crate::common::{DropCounter, DropTracker};

fn sample() -> Queue<i32> {
    (0..5).collect()
}

#[test]
fn debug_formats_front_to_back_like_a_list() {
    let queue = sample();
    assert_eq!(format!("{queue:?}"), "[0, 1, 2, 3, 4]");
    assert_eq!(
        format!("{queue:#?}"),
        "[\n    0,\n    1,\n    2,\n    3,\n    4,\n]"
    );

    let empty: Queue<i32> = Queue::new();
    assert_eq!(format!("{empty:?}"), "[]");
}

#[test]
fn clone_contains_equal_values_and_is_independent() {
    let original = sample();
    let mut cloned = original.clone();

    assert_eq!(cloned, original);

    *cloned.front_mut().unwrap() = 99;
    assert_eq!(cloned.front(), Some(&99));
    assert_eq!(original.front(), Some(&0), "the original is unaffected");
    assert_ne!(cloned, original);
}

#[test]
fn partial_eq_ignores_capacity_and_compares_contents_in_order() {
    let a: Queue<i32> = (0..3).collect();
    let b: Queue<i32> = (0..3).collect();
    assert_eq!(a, b);

    let mut roomy: Queue<i32> = Queue::with_capacity(1_000);
    roomy.extend(0..3);
    assert_eq!(a, roomy);

    let different_len: Queue<i32> = (0..4).collect();
    assert_ne!(a, different_len);

    let mut different_value = a.clone();
    *different_value.front_mut().unwrap() = 99;
    assert_ne!(a, different_value);
}

#[test]
fn eq_supports_generic_functions() {
    fn same<T: PartialEq>(a: &Queue<T>, b: &Queue<T>) -> bool {
        a == b
    }

    let a: Queue<i32> = (0..3).collect();
    let b: Queue<i32> = (0..3).collect();
    let c: Queue<i32> = (0..4).collect();

    assert!(same(&a, &b));
    assert!(!same(&a, &c));
}

#[test]
fn extend_appends_to_the_back_from_any_iterator() {
    let mut queue: Queue<i32> = Queue::new();
    queue.extend(0..3);
    queue.extend([10, 11]);
    queue.extend(std::iter::once(99));

    assert_eq!(queue.len(), 6);
    assert_eq!(queue.front(), Some(&0));
    assert_eq!(queue.back(), Some(&99));
    assert_eq!(format!("{queue:?}"), "[0, 1, 2, 10, 11, 99]");
}

#[test]
fn extend_via_the_extend_trait_is_equivalent() {
    fn extend_with_trait<T: Extend<i32>>(target: &mut T, source: &[i32]) {
        target.extend(source.iter().copied());
    }

    let mut queue: Queue<i32> = Queue::new();
    extend_with_trait(&mut queue, &[1, 2, 3]);

    assert_eq!(queue.len(), 3);
    assert_eq!(queue.front(), Some(&1));
    assert_eq!(queue.dequeue(), Some(1));
}

#[test]
fn from_iterator_builds_front_to_back() {
    let from_range: Queue<i32> = (0..5).collect();
    assert_eq!(from_range.len(), 5);
    assert_eq!(from_range.front(), Some(&0));
    assert_eq!(from_range.back(), Some(&4));
    assert_eq!(format!("{from_range:?}"), "[0, 1, 2, 3, 4]");

    let from_vec: Queue<&str> = vec!["a", "b"].into_iter().collect();
    assert_eq!(from_vec.front(), Some(&"a"));

    let empty: Queue<i32> = std::iter::empty().collect();
    assert!(empty.is_empty());
}

#[test]
fn owned_iteration_yields_front_to_back_and_consumes_the_queue() {
    let owned: Vec<i32> = sample().into_iter().collect();
    assert_eq!(owned, [0, 1, 2, 3, 4]);

    let reversed: Vec<i32> = sample().into_iter().rev().collect();
    assert_eq!(reversed, [4, 3, 2, 1, 0]);

    let rebuilt: Queue<i32> = owned.into_iter().collect();
    assert_eq!(rebuilt, sample());
}

#[test]
fn owned_iteration_transfers_ownership_without_double_drop() {
    let tracker = DropTracker::new();
    let mut queue: Queue<DropCounter> = Queue::new();
    for i in 0..4 {
        queue.enqueue(DropCounter::new(i, &tracker));
    }

    let mut iter = queue.into_iter();
    let first = iter.next().expect("the front element");
    assert_eq!(first.id, 0);
    assert_eq!(tracker.count(), 0);

    drop(iter);
    assert_eq!(tracker.count(), 3, "the three unyielded values drop once");
    drop(first);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn shared_iteration_yields_references_without_moving_elements() {
    let tracker = DropTracker::new();
    let mut queue: Queue<DropCounter> = Queue::new();
    for i in 0..3 {
        queue.enqueue(DropCounter::new(i, &tracker));
    }

    let ids: Vec<i32> = (&queue).into_iter().map(|element| element.id).collect();
    assert_eq!(ids, [0, 1, 2]);
    assert_eq!(tracker.count(), 0);

    let mut seen = Vec::new();
    for element in &queue {
        seen.push(element.id);
    }
    assert_eq!(seen, [0, 1, 2]);

    assert_eq!(queue.len(), 3, "iteration leaves the queue usable");
    drop(queue);
    assert_eq!(tracker.count(), 3);
}

#[test]
fn mutable_iteration_can_update_every_element_in_place() {
    let mut queue = sample();

    for value in &mut queue {
        *value *= 10;
    }

    assert_eq!(format!("{queue:?}"), "[0, 10, 20, 30, 40]");
    assert_eq!(queue.front(), Some(&0));
    assert_eq!(queue.back(), Some(&40));
}

#[test]
fn mutable_iteration_does_not_move_or_drop_elements() {
    let tracker = DropTracker::new();
    let mut queue: Queue<DropCounter> = Queue::new();
    for i in 0..3 {
        queue.enqueue(DropCounter::new(i, &tracker));
    }

    for element in &mut queue {
        element.id += 100;
    }

    assert_eq!(tracker.count(), 0);
    assert_eq!(queue.front().map(|element| element.id), Some(100));
    drop(queue);
    assert_eq!(tracker.count(), 3);
}

#[test]
fn into_iterator_implementations_are_selectable() {
    let mut queue: Queue<i32> = (0..3).collect();

    fn takes_ref<'a>(values: impl IntoIterator<Item = &'a i32>) -> usize {
        values.into_iter().count()
    }
    assert_eq!(takes_ref(&queue), 3);

    fn takes_mut<'a>(values: impl IntoIterator<Item = &'a mut i32>) -> i32 {
        values
            .into_iter()
            .map(|value| {
                *value += 1;
                *value
            })
            .sum()
    }
    assert_eq!(takes_mut(&mut queue), 1 + 2 + 3);
    assert_eq!(format!("{queue:?}"), "[1, 2, 3]");

    fn takes_owned(values: impl IntoIterator<Item = i32>) -> i32 {
        values.into_iter().sum()
    }
    assert_eq!(takes_owned(queue), 6);
}

#[test]
fn generic_functions_work_over_element_types() {
    fn describe<T: Debug>(values: &Queue<T>) -> String {
        format!("{values:?}")
    }

    let queue: Queue<i32> = (0..3).collect();
    assert_eq!(describe(&queue), "[0, 1, 2]");
}

#[test]
fn retain_mut_updates_and_filters() {
    let mut queue: Queue<i32> = (0..5).collect();
    queue.retain_mut(|value| {
        *value *= 10;
        *value < 30
    });
    assert_eq!(queue.as_slice(), &[0, 10, 20]);
}
