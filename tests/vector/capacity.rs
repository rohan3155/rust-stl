//! Capacity management: reservation, growth, shrinking and overflow.
//!
//! These tests never assert an exact capacity produced by the growth policy;
//! they only assert the guarantees the API documents (sufficient capacity, no
//! data loss, no shrinkage during growth).

use rust_stl::collections::vector::Vector;

use crate::common::{self, DropTracker};

#[test]
fn reserve_provides_at_least_the_requested_additional_capacity() {
    let mut vec: Vector<i32> = Vector::new();

    vec.reserve(10);
    assert!(vec.capacity() >= 10);
    assert_eq!(vec.len(), 0);

    vec.extend(0..10);
    assert!(vec.capacity() >= 10);

    // Reserving on top of existing elements counts *additional* slots.
    let before = vec.capacity();
    vec.reserve(5);
    assert!(vec.capacity() >= 15);
    assert!(vec.capacity() >= before);
    assert_eq!(vec.as_slice(), &(0..10).collect::<Vec<_>>()[..]);
}

#[test]
fn reserve_exact_capacity_is_at_least_len_plus_additional() {
    let mut vec: Vector<i32> = (0..4).collect();

    vec.reserve_exact(6);

    assert!(vec.capacity() >= 10);
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &[0, 1, 2, 3]);
}

#[test]
fn reserve_below_current_capacity_does_not_reallocate() {
    let mut vec: Vector<i32> = Vector::new();
    vec.reserve(100);
    let capacity = vec.capacity();

    vec.push(1);
    vec.reserve(0);
    vec.reserve_exact(1);

    assert_eq!(vec.capacity(), capacity);
    assert_eq!(vec.as_slice(), &[1]);
}

#[test]
fn reserve_preserves_all_elements_across_reallocation() {
    let mut vec: Vector<i32> = Vector::new();

    for i in 0..64 {
        vec.push(i);
        // Force a reallocation on every second push.
        if i % 2 == 0 {
            vec.reserve(1);
        }
    }

    assert_eq!(vec.len(), 64);
    assert_eq!(vec.as_slice(), &(0..64).collect::<Vec<_>>()[..]);
}

#[test]
fn reserve_preserves_non_copy_elements() {
    let mut vec: Vector<String> = Vector::new();

    for i in 0..32 {
        vec.push(format!("value-{i}"));
    }

    vec.reserve(1_024);
    assert!(vec.capacity() >= 1_056);

    for i in 0..32 {
        assert_eq!(vec[i], format!("value-{i}"));
    }
}

#[test]
fn try_reserve_succeeds_for_reasonable_requests() {
    let mut vec: Vector<i32> = Vector::new();

    assert!(vec.try_reserve(64).is_ok());
    assert!(vec.capacity() >= 64);

    vec.extend(0..10);
    assert!(vec.try_reserve_exact(100).is_ok());
    assert!(vec.capacity() >= 110);
    assert_eq!(vec.as_slice(), &(0..10).collect::<Vec<_>>()[..]);
}

#[test]
fn try_reserve_reports_capacity_overflow_instead_of_panicking() {
    let mut vec: Vector<u32> = (0..4).collect();

    // `4 + usize::MAX` cannot be represented, so this is a capacity overflow.
    let err = vec.try_reserve_exact(usize::MAX);
    assert!(err.is_err(), "expected an error, got {err:?}");

    // A failed reservation must leave the vector completely untouched.
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &[0, 1, 2, 3]);

    assert!(vec.try_reserve(usize::MAX).is_err());
    assert_eq!(vec.as_slice(), &[0, 1, 2, 3]);
}

#[test]
fn try_reserve_reports_layout_overflow_for_huge_element_types() {
    // `usize::MAX / 2` elements of `u64` cannot exist, so the layout request
    // overflows even though `len + additional` does not.
    let mut vec: Vector<u64> = Vector::new();

    let err = vec.try_reserve_exact(usize::MAX / 2);
    assert!(err.is_err(), "expected a layout overflow error");
    assert!(vec.is_empty());

    // The vector is still usable afterwards.
    vec.push(1);
    assert_eq!(vec.as_slice(), &[1]);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn reserve_reports_capacity_overflow_by_panicking() {
    let mut vec: Vector<u32> = Vector::new();
    vec.reserve(usize::MAX);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn reserve_exact_reports_capacity_overflow_by_panicking() {
    let mut vec: Vector<u32> = Vector::new();
    vec.reserve_exact(usize::MAX);
}

#[test]
fn shrink_to_fit_on_empty_vector_releases_storage() {
    let mut vec: Vector<i32> = Vector::new();
    vec.reserve(64);
    assert!(vec.capacity() >= 64);

    vec.shrink_to_fit();

    assert_eq!(vec.capacity(), 0);
    assert!(vec.is_empty());
    assert_eq!(vec.as_slice(), &[] as &[i32]);

    // Still usable, and a fresh push must reallocate safely.
    vec.push(1);
    assert_eq!(vec.as_slice(), &[1]);
}

#[test]
fn shrink_to_fit_reduces_capacity_and_keeps_values() {
    let mut vec: Vector<i32> = Vector::new();
    vec.reserve(128);
    vec.extend(0..10);

    vec.shrink_to_fit();

    assert_eq!(vec.capacity(), 10);
    assert_eq!(vec.len(), 10);
    assert_eq!(vec.as_slice(), &(0..10).collect::<Vec<_>>()[..]);
}

#[test]
fn shrink_to_fit_is_a_no_op_when_len_equals_capacity() {
    let mut vec: Vector<i32> = Vector::new();
    vec.reserve_exact(4);
    vec.extend(0..4);
    let capacity = vec.capacity();

    vec.shrink_to_fit();

    assert_eq!(vec.capacity(), capacity);
    assert_eq!(vec.as_slice(), &[0, 1, 2, 3]);
}

#[test]
fn shrink_to_fit_preserves_ownership_of_non_copy_elements() {
    let tracker = DropTracker::new();

    {
        let mut vec: Vector<common::DropCounter> = Vector::new();
        vec.reserve(64);
        for i in 0..8 {
            vec.push(common::DropCounter::new(i, &tracker));
        }

        vec.shrink_to_fit();

        assert_eq!(vec.capacity(), 8);
        assert_eq!(vec.len(), 8);
        assert_eq!(
            tracker.count(),
            0,
            "shrinking must move elements, not drop them"
        );
    }

    assert_eq!(tracker.count(), 8, "each element dropped exactly once");
}

#[test]
fn clearing_retains_capacity_and_stays_reusable() {
    let mut vec: Vector<i32> = Vector::new();
    vec.extend(0..16);
    let capacity = vec.capacity();

    vec.clear();
    assert_eq!(vec.capacity(), capacity);

    vec.extend(0..3);
    assert_eq!(vec.capacity(), capacity);
    assert_eq!(vec.as_slice(), &[0, 1, 2]);
}

#[test]
fn shrink_to_fit_after_clear_releases_everything() {
    let mut vec: Vector<i32> = Vector::new();
    vec.extend(0..16);

    vec.clear();
    vec.shrink_to_fit();

    assert_eq!(vec.capacity(), 0);
    assert!(vec.is_empty());
}

#[test]
fn capacity_is_never_reported_smaller_than_length() {
    let mut vec: Vector<i32> = Vector::new();

    for i in 0..200 {
        vec.push(i);
        assert!(vec.capacity() >= vec.len());
    }

    vec.truncate(5);
    assert!(vec.capacity() >= vec.len());

    vec.shrink_to_fit();
    assert!(vec.capacity() >= vec.len());
}
