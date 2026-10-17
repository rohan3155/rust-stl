//! Edge cases: empty vectors, single elements, zero-sized types, non-`Copy`
//! values and nested vectors.

use rust_stl::collections::vector::Vector;

use crate::common::{self, DropTracker};

/// A distinct zero-sized type used to make sure the implementation does not
/// special-case `()` alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Unit;

#[derive(Clone, Debug, PartialEq, Eq)]
struct NonCopy(String);

#[test]
fn empty_vector_supports_every_read_only_operation() {
    let vec: Vector<i32> = Vector::new();

    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 0);
    assert_eq!(vec.as_slice(), &[] as &[i32]);
    assert_eq!(vec.get(0), None);
    assert_eq!(vec.first(), None);
    assert_eq!(vec.last(), None);
    assert_eq!(vec.iter().count(), 0);
    assert_eq!(vec.windows(1).count(), 0);
    assert_eq!(vec.chunks(1).count(), 0);
    assert_eq!(vec.split_at(0), (&[] as &[i32], &[] as &[i32]));
    assert_eq!(vec.binary_search(&1), Err(0));
    assert_eq!(format!("{vec:?}"), "[]");
}

#[test]
fn empty_vector_supports_every_mutating_operation() {
    let mut vec: Vector<i32> = Vector::new();

    vec.clear();
    vec.truncate(0);
    vec.truncate(5);
    vec.reverse();
    vec.sort();
    vec.dedup();
    vec.remove_duplicates();
    vec.retain(|_| true);
    vec.retain(|_| false);
    vec.remove_where(|_| true);
    vec.reserve(0);
    vec.reserve_exact(0);
    vec.shrink_to_fit();
    vec.rotate_left(0);
    vec.rotate_right(0);
    vec.extend(std::iter::empty::<i32>());

    assert!(vec.is_empty());
    assert_eq!(vec.pop(), None);
    assert_eq!(vec.swap_remove(0), None);
    assert!(!vec.remove_first(&1));

    let mut other: Vector<i32> = Vector::new();
    vec.append(&mut other);
    assert!(vec.is_empty());
    assert!(other.is_empty());

    let tail = vec.split_off(0);
    assert!(tail.is_empty());
    assert!(vec.is_empty());
}

#[test]
fn single_element_vector_behaviour() {
    let mut vec: Vector<i32> = Vector::new();
    vec.push(1);

    assert_eq!(vec.len(), 1);
    assert_eq!(vec.first(), Some(&1));
    assert_eq!(vec.last(), Some(&1));
    assert_eq!(vec.split_at(1).0, &[1]);

    vec.dedup();
    vec.remove_duplicates();
    vec.reverse();
    vec.retain(|_| true);
    assert_eq!(vec.as_slice(), &[1]);

    assert_eq!(vec.pop(), Some(1));
    assert!(vec.is_empty());
}

#[test]
fn empty_slices_and_iterators_do_not_allocate() {
    let vec: Vector<i32> = Vector::new();
    assert_eq!(vec.capacity(), 0);

    // Consuming an empty vector must not start allocating.
    let mut iter = vec.into_iter();
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn operations_that_empty_a_vector_retain_capacity() {
    let mut vec: Vector<i32> = Vector::new();
    vec.extend(0..8);
    let capacity = vec.capacity();

    vec.clear();
    assert_eq!(vec.capacity(), capacity);

    vec.extend(0..8);
    vec.truncate(0);
    assert_eq!(vec.capacity(), capacity);

    vec.extend(0..8);
    vec.retain(|_| false);
    assert_eq!(vec.capacity(), capacity);

    vec.extend(0..8);
    vec.remove_where(|_| true);
    assert_eq!(vec.capacity(), capacity);

    vec.extend(0..8);
    let tail = vec.split_off(0);
    assert_eq!(tail.len(), 8);
    assert!(vec.is_empty());
}

#[test]
fn unit_type_is_a_zero_sized_type() {
    let mut vec: Vector<()> = Vector::new();

    assert_eq!(vec.capacity(), 0);

    for _ in 0..1_000 {
        vec.push(());
    }

    assert_eq!(vec.len(), 1_000);
    // A ZST has no storage, so the capacity is reported as unbounded.
    assert_eq!(vec.capacity(), usize::MAX);
    assert_eq!(vec.as_slice().len(), 1_000);
    assert_eq!(vec.iter().count(), 1_000);
    assert_eq!(vec.first(), Some(&()));
    assert_eq!(vec.last(), Some(&()));

    let popped = vec.pop();
    assert_eq!(popped, Some(()));
    assert_eq!(vec.len(), 999);

    assert_eq!(vec.remove(0), ());
    assert_eq!(vec.len(), 998);
    vec.insert(0, ());
    assert_eq!(vec.len(), 999);
    assert_eq!(vec.swap_remove(0), Some(()));
    assert_eq!(vec.len(), 998);

    assert!(vec.remove_first(&()));

    vec.retain(|_| false);
    assert!(vec.is_empty());
}

#[test]
fn zero_sized_type_reserve_and_shrink_do_not_touch_the_allocator() {
    let mut vec: Vector<Unit> = Vector::new();

    // These would abort if a zero-sized layout were passed to `alloc`/`dealloc`.
    vec.reserve(10_000);
    vec.reserve_exact(10_000);
    assert!(vec.try_reserve(1_000_000).is_ok());
    assert!(vec.try_reserve_exact(1_000_000).is_ok());
    vec.shrink_to_fit();

    vec.push(Unit);
    vec.push(Unit);

    assert_eq!(vec.len(), 2);
    assert_eq!(vec.remove(0), Unit);
    assert_eq!(vec.pop(), Some(Unit));

    vec.shrink_to_fit();
    vec.clear();
    assert!(vec.is_empty());
}

#[test]
fn zero_sized_type_split_off_and_append() {
    let mut head: Vector<Unit> = Vector::new();
    for _ in 0..4 {
        head.push(Unit);
    }

    let tail = head.split_off(2);
    assert_eq!(head.len(), 2);
    assert_eq!(tail.len(), 2);

    let mut other: Vector<Unit> = Vector::new();
    for _ in 0..3 {
        other.push(Unit);
    }
    head.append(&mut other);
    assert_eq!(head.len(), 5);
    assert!(other.is_empty());
}

#[test]
fn zero_sized_type_converts_through_all_iteration_forms() {
    let mut vec: Vector<Unit> = Vector::new();
    for _ in 0..5 {
        vec.push(Unit);
    }

    assert_eq!((&vec).into_iter().count(), 5);
    vec.fill(Unit);
    assert_eq!(vec.into_iter().count(), 5);
}

#[test]
fn zero_sized_type_sort_and_search() {
    let mut vec: Vector<Unit> = Vector::new();
    for _ in 0..4 {
        vec.push(Unit);
    }

    vec.sort();
    vec.reverse();
    assert_eq!(vec.len(), 4);
    // All elements compare equal, and `binary_search` may return any match.
    assert!(vec.binary_search(&Unit).is_ok());
    vec.dedup();
    assert_eq!(vec.len(), 1);
    vec.remove_duplicates();
    assert_eq!(vec.len(), 1);
}

#[test]
fn non_copy_elements_can_be_moved_in_and_out() {
    let mut vec: Vector<NonCopy> = Vector::new();

    vec.push(NonCopy("a".to_string()));
    vec.push(NonCopy("b".to_string()));
    vec.insert(1, NonCopy("c".to_string()));
    vec.extend_from_slice(&[NonCopy("d".to_string())]);

    assert_eq!(vec.len(), 4);
    assert_eq!(vec[1], NonCopy("c".to_string()));

    // `remove` hands ownership back to the caller.
    let removed = vec.remove(1);
    assert_eq!(removed, NonCopy("c".to_string()));

    let popped = vec.pop();
    assert_eq!(popped, Some(NonCopy("d".to_string())));

    // Values survive a clone and can be compared.
    let cloned = vec.clone();
    assert_eq!(cloned, vec);

    let sorted: Vector<NonCopy> = vec.clone();
    assert_eq!(format!("{sorted:?}"), r#"[NonCopy("a"), NonCopy("b")]"#);
}

#[test]
fn nested_vectors_work() {
    let mut outer: Vector<Vector<i32>> = Vector::new();

    outer.push((0..3).collect());
    outer.push(Vector::new());
    outer.push((3..5).collect());

    assert_eq!(outer.len(), 3);
    assert_eq!(outer[0].as_slice(), &[0, 1, 2]);
    assert!(outer[1].is_empty());
    assert_eq!(outer[2].as_slice(), &[3, 4]);

    outer[1].push(99);
    assert_eq!(outer[1].as_slice(), &[99]);

    // Nested ownership survives removal and cloning.
    let removed = outer.remove(0);
    assert_eq!(removed.as_slice(), &[0, 1, 2]);
    assert_eq!(outer.len(), 2);

    let cloned = outer.clone();
    assert_eq!(cloned, outer);
}

#[test]
fn nested_vectors_release_inner_allocations_once() {
    let tracker = DropTracker::new();

    {
        let mut outer: Vector<Vector<common::DropCounter>> = Vector::new();
        let mut inner: Vector<common::DropCounter> = Vector::new();
        for i in 0..5 {
            inner.push(common::DropCounter::new(i, &tracker));
        }
        outer.push(inner);
        assert_eq!(tracker.count(), 0);

        // Clearing the outer vector must drop the inner vector, which drops its
        // elements — exactly once each.
        outer.clear();
        assert_eq!(tracker.count(), 5);
    }

    assert_eq!(tracker.count(), 5);
}

#[test]
fn append_handles_empty_operands() {
    let mut left: Vector<i32> = (0..3).collect();
    let mut right: Vector<i32> = Vector::new();

    left.append(&mut right);
    assert_eq!(left.as_slice(), &[0, 1, 2]);
    assert!(right.is_empty());

    let mut empty: Vector<i32> = Vector::new();
    let mut values: Vector<i32> = (0..3).collect();
    empty.append(&mut values);
    assert_eq!(empty.as_slice(), &[0, 1, 2]);
    assert!(values.is_empty());
}

#[test]
fn split_off_at_zero_length_and_middle() {
    let mut vec: Vector<i32> = (0..6).collect();

    let all = vec.split_off(0);
    assert_eq!(all.as_slice(), &[0, 1, 2, 3, 4, 5]);
    assert!(vec.is_empty());

    let mut vec: Vector<i32> = (0..6).collect();
    let none = vec.split_off(6);
    assert!(none.is_empty());
    assert_eq!(vec.len(), 6);

    let mut vec: Vector<i32> = (0..6).collect();
    let tail = vec.split_off(3);
    assert_eq!(vec.as_slice(), &[0, 1, 2]);
    assert_eq!(tail.as_slice(), &[3, 4, 5]);
}

#[test]
fn split_off_moves_ownership_without_dropping() {
    let tracker = DropTracker::new();

    {
        let mut vec: Vector<common::DropCounter> = Vector::new();
        for i in 0..6 {
            vec.push(common::DropCounter::new(i, &tracker));
        }

        let tail = vec.split_off(2);
        assert_eq!(tracker.count(), 0, "splitting must not drop anything");
        assert_eq!(vec.len(), 2);
        assert_eq!(tail.len(), 4);

        drop(vec);
        assert_eq!(tracker.count(), 2);
        drop(tail);
        assert_eq!(tracker.count(), 6);
    }

    assert_eq!(tracker.count(), 6);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn split_off_past_length_panics() {
    let mut vec: Vector<i32> = (0..3).collect();
    let _ = vec.split_off(4);
}
