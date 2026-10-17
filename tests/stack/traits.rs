//! Standard trait implementations and the three iteration forms for `Stack`.
//!
//! Note the documented iteration order: iterating a stack walks from the bottom
//! to the top, which is the **reverse** of the pop order.

use rust_stl::collections::stack::Stack;
use std::fmt::Debug;

use crate::common::{DropCounter, DropTracker};

fn sample() -> Stack<i32> {
    (0..5).collect()
}

#[test]
fn debug_formats_bottom_to_top_like_a_list() {
    let stack = sample();
    assert_eq!(format!("{stack:?}"), "[0, 1, 2, 3, 4]");
    assert_eq!(
        format!("{stack:#?}"),
        "[\n    0,\n    1,\n    2,\n    3,\n    4,\n]"
    );

    let empty: Stack<i32> = Stack::new();
    assert_eq!(format!("{empty:?}"), "[]");
}

#[test]
fn clone_contains_equal_values_and_is_independent() {
    let original = sample();
    let mut cloned = original.clone();

    assert_eq!(cloned, original);

    *cloned.top_mut().unwrap() = 99;
    assert_eq!(cloned.top(), Some(&99));
    assert_eq!(original.top(), Some(&4), "the original is unaffected");
    assert_ne!(cloned, original);
}

#[test]
fn partial_eq_ignores_capacity_and_compares_contents_in_order() {
    let a: Stack<i32> = (0..3).collect();
    let b: Stack<i32> = (0..3).collect();
    assert_eq!(a, b);

    // Different capacities, same contents: still equal.
    let mut roomy: Stack<i32> = Stack::with_capacity(1_000);
    roomy.extend(0..3);
    assert_eq!(a, roomy);

    let different_len: Stack<i32> = (0..4).collect();
    assert_ne!(a, different_len);

    let mut different_value = a.clone();
    *different_value.top_mut().unwrap() = 99;
    assert_ne!(a, different_value);

    // Order matters: the same values pushed in a different order are unequal.
    let ascending: Stack<i32> = vec![1, 2].into_iter().collect();
    let descending: Stack<i32> = vec![2, 1].into_iter().collect();
    assert_ne!(ascending, descending);
}

#[test]
fn eq_supports_generic_functions() {
    fn same<T: PartialEq>(a: &Stack<T>, b: &Stack<T>) -> bool {
        a == b
    }

    let a: Stack<i32> = (0..3).collect();
    let b: Stack<i32> = (0..3).collect();
    let c: Stack<i32> = (0..4).collect();

    assert!(same(&a, &b));
    assert!(!same(&a, &c));
}

#[test]
fn extend_appends_on_top_from_any_iterator() {
    let mut stack: Stack<i32> = Stack::new();
    stack.extend(0..3);
    stack.extend([10, 11]);
    stack.extend(std::iter::once(99));

    assert_eq!(stack.len(), 6);
    assert_eq!(stack.top(), Some(&99));
    assert_eq!(format!("{stack:?}"), "[0, 1, 2, 10, 11, 99]");
}

#[test]
fn extend_via_the_extend_trait_is_equivalent() {
    fn extend_with_trait<T: Extend<i32>>(target: &mut T, source: &[i32]) {
        target.extend(source.iter().copied());
    }

    let mut stack: Stack<i32> = Stack::new();
    extend_with_trait(&mut stack, &[1, 2, 3]);

    assert_eq!(stack.len(), 3);
    assert_eq!(stack.top(), Some(&3));
    assert_eq!(stack.pop(), Some(3));
}

#[test]
fn from_iterator_builds_bottom_to_top() {
    let from_range: Stack<i32> = (0..5).collect();
    assert_eq!(from_range.len(), 5);
    assert_eq!(from_range.top(), Some(&4));
    assert_eq!(format!("{from_range:?}"), "[0, 1, 2, 3, 4]");

    let from_vec: Stack<&str> = vec!["a", "b"].into_iter().collect();
    assert_eq!(from_vec.top(), Some(&"b"));

    let empty: Stack<i32> = std::iter::empty().collect();
    assert!(empty.is_empty());
}

#[test]
fn owned_iteration_yields_bottom_to_top_and_consumes_the_stack() {
    let owned: Vec<i32> = sample().into_iter().collect();
    assert_eq!(owned, [0, 1, 2, 3, 4]);

    // The owned iterator is double-ended, so `rev` yields the pop order.
    let reversed: Vec<i32> = sample().into_iter().rev().collect();
    assert_eq!(reversed, [4, 3, 2, 1, 0]);

    // A stack can be rebuilt from an iterator.
    let rebuilt: Stack<i32> = owned.into_iter().collect();
    assert_eq!(rebuilt, sample());
}

#[test]
fn owned_iteration_transfers_ownership_without_double_drop() {
    let tracker = DropTracker::new();
    let mut stack: Stack<DropCounter> = Stack::new();
    for i in 0..4 {
        stack.push(DropCounter::new(i, &tracker));
    }

    let mut iter = stack.into_iter();
    let first = iter.next().expect("the bottom element");
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
    let mut stack: Stack<DropCounter> = Stack::new();
    for i in 0..3 {
        stack.push(DropCounter::new(i, &tracker));
    }

    let ids: Vec<i32> = (&stack).into_iter().map(|element| element.id).collect();
    assert_eq!(ids, [0, 1, 2]);
    assert_eq!(tracker.count(), 0);

    let mut seen = Vec::new();
    for element in &stack {
        seen.push(element.id);
    }
    assert_eq!(seen, [0, 1, 2]);

    assert_eq!(stack.len(), 3, "iteration leaves the stack usable");
    drop(stack);
    assert_eq!(tracker.count(), 3);
}

#[test]
fn mutable_iteration_can_update_every_element_in_place() {
    let mut stack = sample();

    for value in &mut stack {
        *value *= 10;
    }

    assert_eq!(format!("{stack:?}"), "[0, 10, 20, 30, 40]");
    assert_eq!(stack.top(), Some(&40));
}

#[test]
fn mutable_iteration_does_not_move_or_drop_elements() {
    let tracker = DropTracker::new();
    let mut stack: Stack<DropCounter> = Stack::new();
    for i in 0..3 {
        stack.push(DropCounter::new(i, &tracker));
    }

    for element in &mut stack {
        element.id += 100;
    }

    assert_eq!(tracker.count(), 0);
    assert_eq!(stack.top().map(|element| element.id), Some(102));
    drop(stack);
    assert_eq!(tracker.count(), 3);
}

#[test]
fn into_iterator_implementations_are_selectable() {
    let mut stack: Stack<i32> = (0..3).collect();

    // &Stack
    fn takes_ref<'a>(values: impl IntoIterator<Item = &'a i32>) -> usize {
        values.into_iter().count()
    }
    assert_eq!(takes_ref(&stack), 3);

    // &mut Stack
    fn takes_mut<'a>(values: impl IntoIterator<Item = &'a mut i32>) -> i32 {
        values
            .into_iter()
            .map(|value| {
                *value += 1;
                *value
            })
            .sum()
    }
    assert_eq!(takes_mut(&mut stack), 1 + 2 + 3);
    assert_eq!(format!("{stack:?}"), "[1, 2, 3]");

    // Stack (by value)
    fn takes_owned(values: impl IntoIterator<Item = i32>) -> i32 {
        values.into_iter().sum()
    }
    assert_eq!(takes_owned(stack), 6);
}

#[test]
fn generic_functions_work_over_element_types() {
    fn describe<T: Debug>(values: &Stack<T>) -> String {
        format!("{values:?}")
    }

    let stack: Stack<i32> = (0..3).collect();
    assert_eq!(describe(&stack), "[0, 1, 2]");
}
