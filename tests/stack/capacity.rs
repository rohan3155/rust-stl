//! Capacity management for `Stack`: growth, reservation and shrinking.
//!
//! These tests rely only on the guarantees the API documents — sufficient
//! capacity, no data loss and no shrinkage during growth — never on an exact
//! growth factor, which is an implementation detail of the backing `Vector`.

use rust_stl::collections::stack::Stack;

#[test]
fn push_across_many_growths_preserves_values_and_lifo_order() {
    let mut stack: Stack<u32> = Stack::new();
    let mut previous_capacity = stack.capacity();

    for i in 0..1_000 {
        stack.push(i);
        assert!(stack.capacity() >= stack.len());
        assert!(
            stack.capacity() >= previous_capacity,
            "capacity never shrinks"
        );
        previous_capacity = stack.capacity();
    }

    assert!(
        stack.capacity() > 4,
        "expected several growth steps, capacity stayed at {}",
        stack.capacity()
    );

    for i in (0..1_000).rev() {
        assert_eq!(stack.pop(), Some(i));
    }
    assert!(stack.is_empty());
}

#[test]
fn with_capacity_reserves_room_without_adding_elements() {
    let mut stack: Stack<i32> = Stack::with_capacity(16);

    assert!(stack.capacity() >= 16);
    assert_eq!(stack.len(), 0);
    assert!(stack.is_empty());

    stack.extend(0..16);
    assert_eq!(stack.len(), 16);
    assert_eq!(stack.top(), Some(&15));
    assert!(stack.capacity() >= 16);
}

#[test]
fn reserve_provides_at_least_the_requested_additional_slots() {
    let mut stack: Stack<i32> = (0..4).collect();

    stack.reserve(10);

    assert!(stack.capacity() >= 14);
    assert_eq!(stack.len(), 4);
    assert_eq!(stack.top(), Some(&3));

    // A request that already fits must not disturb the contents.
    stack.reserve(0);
    assert_eq!(stack.len(), 4);
    assert_eq!(stack.top(), Some(&3));
}

#[test]
fn reserve_exact_fits_len_plus_additional() {
    let mut stack: Stack<i32> = Stack::new();

    stack.reserve_exact(8);
    assert!(stack.capacity() >= 8);

    stack.extend(0..8);
    assert_eq!(stack.len(), 8);
    assert_eq!(stack.pop(), Some(7));
}

#[test]
fn reserve_does_not_lose_elements() {
    let mut stack: Stack<String> = Stack::new();
    for i in 0..32 {
        stack.push(format!("value-{i}"));
    }

    stack.reserve(1_024);
    assert!(stack.capacity() >= 1_056);

    for i in (0..32).rev() {
        assert_eq!(stack.pop(), Some(format!("value-{i}")));
    }
}

#[test]
fn clear_retains_capacity_and_the_stack_stays_usable() {
    let mut stack: Stack<i32> = (0..16).collect();
    let capacity = stack.capacity();
    assert!(capacity >= 16);

    stack.clear();

    assert!(stack.is_empty());
    assert_eq!(
        stack.capacity(),
        capacity,
        "clear must not release capacity"
    );

    stack.extend(0..3);
    assert_eq!(stack.capacity(), capacity);
    assert_eq!(stack.pop(), Some(2));
}

#[test]
fn shrink_to_fit_reduces_capacity_and_keeps_values() {
    let mut stack: Stack<i32> = Stack::with_capacity(128);
    stack.extend(0..10);

    stack.shrink_to_fit();

    assert_eq!(stack.capacity(), 10);
    assert_eq!(stack.len(), 10);
    assert_eq!(stack.top(), Some(&9));
}

#[test]
fn shrink_to_fit_on_empty_stack_releases_storage_and_stays_usable() {
    let mut stack: Stack<i32> = Stack::with_capacity(64);

    stack.shrink_to_fit();

    assert_eq!(stack.capacity(), 0);
    assert!(stack.is_empty());

    stack.push(1);
    assert_eq!(stack.pop(), Some(1));
}

#[test]
fn capacity_is_never_reported_smaller_than_length() {
    let mut stack: Stack<i32> = Stack::new();

    for i in 0..200 {
        stack.push(i);
        assert!(stack.capacity() >= stack.len());
    }

    stack.shrink_to_fit();
    assert!(stack.capacity() >= stack.len());
}
