//! Edge cases for `Stack`: empty, single-element, zero-sized and nested values.

use rust_stl::collections::stack::Stack;

/// A distinct zero-sized type, so the tests do not special-case `()` alone.
#[derive(Debug, PartialEq, Eq)]
struct Marker;

#[test]
fn empty_stack_supports_every_read_only_operation() {
    let stack: Stack<i32> = Stack::new();

    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
    assert_eq!(stack.capacity(), 0);
    assert_eq!(stack.top(), None);
    assert_eq!(stack.get(0), None);
    assert_eq!((&stack).into_iter().count(), 0);
    assert_eq!(format!("{stack:?}"), "[]");
}

#[test]
fn empty_stack_supports_every_mutating_operation() {
    let mut stack: Stack<i32> = Stack::new();

    stack.clear();
    stack.reserve(0);
    stack.reserve_exact(0);
    stack.shrink_to_fit();
    stack.extend(std::iter::empty::<i32>());

    assert!(stack.is_empty());
    assert_eq!(stack.pop(), None);

    let mut other: Stack<i32> = Stack::new();
    stack.swap(&mut other);
    assert!(stack.is_empty());
    assert!(other.is_empty());
}

#[test]
fn swap_exchanges_contents_and_capacity() {
    let mut a: Stack<i32> = (0..3).collect();
    let mut b: Stack<i32> = (10..12).collect();
    let a_capacity = a.capacity();
    let b_capacity = b.capacity();

    a.swap(&mut b);

    assert_eq!(a.len(), 2);
    assert_eq!(a.top(), Some(&11));
    assert_eq!(b.len(), 3);
    assert_eq!(b.top(), Some(&2));
    assert_eq!(a.capacity(), b_capacity);
    assert_eq!(b.capacity(), a_capacity);

    // Popping still works after the swap.
    assert_eq!(a.pop(), Some(11));
    assert_eq!(b.pop(), Some(2));
}

#[test]
fn unit_type_is_a_zero_sized_type() {
    let mut stack: Stack<()> = Stack::new();
    assert_eq!(stack.capacity(), 0);

    for _ in 0..1_000 {
        stack.push(());
    }

    assert_eq!(stack.len(), 1_000);
    // A ZST has no storage, so the capacity is reported as unbounded.
    assert_eq!(stack.capacity(), usize::MAX);
    assert_eq!(stack.top(), Some(&()));
    assert_eq!(stack.pop(), Some(()));
    assert_eq!(stack.len(), 999);

    stack.clear();
    assert!(stack.is_empty());
}

#[test]
fn zero_sized_type_reserve_and_shrink_do_not_touch_the_allocator() {
    let mut stack: Stack<Marker> = Stack::new();

    // These would abort if a zero-sized layout were passed to `alloc`/`dealloc`.
    stack.reserve(10_000);
    stack.reserve_exact(10_000);
    stack.shrink_to_fit();

    stack.push(Marker);
    stack.push(Marker);

    assert_eq!(stack.len(), 2);
    assert_eq!(stack.top(), Some(&Marker));
    assert_eq!(stack.pop(), Some(Marker));
    assert_eq!(stack.pop(), Some(Marker));
    assert_eq!(stack.pop(), None);
}

#[test]
fn nested_stacks_work() {
    let mut outer: Stack<Stack<i32>> = Stack::new();
    outer.push((0..2).collect());
    outer.push((2..4).collect());

    assert_eq!(outer.top().unwrap().top(), Some(&3));

    outer.top_mut().unwrap().push(4);
    assert_eq!(outer.top().unwrap().len(), 3);

    let popped = outer.pop().expect("the top stack");
    assert_eq!(popped.top(), Some(&4));
    assert_eq!(outer.len(), 1);
    assert_eq!(outer.top().unwrap().top(), Some(&1));
}

#[test]
fn a_large_stack_round_trips() {
    let mut stack: Stack<usize> = Stack::new();

    for i in 0..10_000 {
        stack.push(i);
    }
    assert_eq!(stack.len(), 10_000);

    for i in (0..10_000).rev() {
        assert_eq!(stack.pop(), Some(i));
    }
    assert!(stack.is_empty());
}
