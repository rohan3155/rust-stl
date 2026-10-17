//! Standard trait implementations, including generic (non-`i32`) usage.

use rust_stl::collections::vector::Vector;
use std::fmt::Debug;

use crate::common::{self, DropTracker};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn debug_formats_like_a_list() {
    let vec: Vector<i32> = (1..4).collect();
    assert_eq!(format!("{vec:?}"), "[1, 2, 3]");
    assert_eq!(format!("{vec:#?}"), "[\n    1,\n    2,\n    3,\n]");

    let empty: Vector<i32> = Vector::new();
    assert_eq!(format!("{empty:?}"), "[]");

    let nested: Vector<Vector<i32>> = vec![(0..2).collect()].into_iter().collect();
    assert_eq!(format!("{nested:?}"), "[[0, 1]]");
}

#[test]
fn clone_is_independent_and_equal() {
    let original: Vector<Point> = vec![
        Point { x: 0, y: 0 },
        Point { x: 1, y: 2 },
        Point { x: 3, y: 4 },
    ]
    .into_iter()
    .collect();

    let mut cloned = original.clone();
    assert_eq!(cloned, original);
    assert_ne!(cloned.begin(), original.begin());

    cloned[0].x = 99;
    assert_ne!(cloned, original);
    assert_eq!(original[0].x, 0, "the original must be unaffected");
}

#[test]
fn clone_of_empty_vector_allocates_nothing_observable() {
    let empty: Vector<Point> = Vector::new();
    let cloned = empty.clone();

    assert_eq!(cloned, empty);
    assert!(cloned.is_empty());
    assert_eq!(cloned.capacity(), 0);
}

#[test]
fn partial_eq_compares_length_then_elements() {
    let a: Vector<i32> = (0..3).collect();
    let b: Vector<i32> = (0..3).collect();
    let c: Vector<i32> = (0..4).collect();
    let d: Vector<i32> = vec![0, 1, 9].into_iter().collect();

    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(a, d);

    // Different capacities, same contents, still equal.
    let mut roomy: Vector<i32> = Vector::new();
    roomy.reserve(1_000);
    roomy.extend(0..3);
    assert_eq!(a, roomy);
}

#[test]
fn eq_is_usable_in_generic_code() {
    fn all_equal<T: Eq + Clone>(values: &Vector<T>) -> bool {
        values
            .iter()
            .all(|value| value.clone() == values[0].clone())
    }

    let same: Vector<Point> = vec![Point { x: 1, y: 1 }; 3].into_iter().collect();
    assert!(all_equal(&same));

    let mut mixed = same.clone();
    mixed[2].y = 5;
    assert!(!all_equal(&mixed));
}

#[test]
fn from_iterator_collects_in_order() {
    let from_range: Vector<i32> = (0..5).collect();
    assert_eq!(from_range.as_slice(), &[0, 1, 2, 3, 4]);

    let from_vec: Vector<&str> = vec!["a", "b"].into_iter().collect();
    assert_eq!(from_vec.as_slice(), &["a", "b"]);

    let empty: Vector<i32> = std::iter::empty().collect();
    assert!(empty.is_empty());
}

#[test]
fn extend_appends_from_any_iterator() {
    let mut vec: Vector<i32> = Vector::new();
    vec.extend(0..3);
    vec.extend([10, 11]);
    vec.extend(std::iter::once(99));

    assert_eq!(vec.as_slice(), &[0, 1, 2, 10, 11, 99]);
}

#[test]
fn extend_via_the_extend_trait_is_equivalent() {
    fn extend_with_trait<T: Extend<i32>>(target: &mut T, source: &[i32]) {
        target.extend(source.iter().copied());
    }

    let mut vec: Vector<i32> = Vector::new();
    extend_with_trait(&mut vec, &[1, 2, 3]);

    assert_eq!(vec.as_slice(), &[1, 2, 3]);
}

#[test]
fn extend_from_slice_clones_the_source() {
    let source = [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
    let mut vec: Vector<Point> = Vector::new();

    vec.extend_from_slice(&source);

    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0], source[0]);
    assert_eq!(vec[1], source[1]);
    assert_eq!(source.len(), 2, "the source slice is untouched");
}

#[test]
fn deref_exposes_slice_methods() {
    let mut vec: Vector<i32> = (0..5).collect();

    // These come from `[T]` through `Deref`.
    assert!(vec.contains(&3));
    assert!(vec.starts_with(&[0, 1]));
    assert!(vec.ends_with(&[3, 4]));
    assert_eq!(vec.first_chunk::<2>(), Some(&[0, 1]));

    // `DerefMut` exposes the mutating slice methods.
    vec.swap(0, 1);
    vec.sort_unstable();
    vec.fill(7);
    assert_eq!(vec.as_slice(), &[7, 7, 7, 7, 7]);
}

#[test]
fn deref_target_is_a_slice() {
    fn takes_slice(values: &[i32]) -> usize {
        values.len()
    }

    let vec: Vector<i32> = (0..4).collect();
    assert_eq!(takes_slice(&vec), 4);
    assert_eq!(takes_slice(&vec[1..]), 3);
}

#[test]
fn range_indexing_works_like_a_slice() {
    let mut vec: Vector<i32> = (0..6).collect();

    assert_eq!(&vec[1..3], &[1, 2]);
    assert_eq!(&vec[..2], &[0, 1]);
    assert_eq!(&vec[4..], &[4, 5]);
    assert_eq!(&vec[..], &[0, 1, 2, 3, 4, 5]);

    vec[1..3].sort_by(|a, b| b.cmp(a));
    assert_eq!(vec.as_slice(), &[0, 2, 1, 3, 4, 5]);
}

#[test]
#[should_panic(expected = "range end index 9 out of range")]
fn range_indexing_panics_out_of_range() {
    let vec: Vector<i32> = (0..3).collect();
    let _ = &vec[0..9];
}

#[test]
fn index_and_index_mut_agree_with_the_slice_view() {
    let mut vec: Vector<Point> = vec![Point { x: 1, y: 1 }, Point { x: 2, y: 2 }]
        .into_iter()
        .collect();

    assert_eq!(vec[0].x, 1);
    vec[0].x = 42;

    assert_eq!(vec[0].x, 42);
    assert_eq!(vec.as_slice()[0].x, 42);
}

#[test]
fn into_iterator_implementations_are_selectable() {
    let mut vec: Vector<i32> = (0..3).collect();

    // &Vector
    fn takes_ref<'a>(values: impl IntoIterator<Item = &'a i32>) -> usize {
        values.into_iter().count()
    }
    assert_eq!(takes_ref(&vec), 3);

    // &mut Vector
    fn takes_mut<'a>(values: impl IntoIterator<Item = &'a mut i32>) -> usize {
        values
            .into_iter()
            .map(|v| {
                *v += 1;
                *v
            })
            .count()
    }
    assert_eq!(takes_mut(&mut vec), 3);
    assert_eq!(vec.as_slice(), &[1, 2, 3]);

    // Vector (by value)
    fn takes_owned(values: impl IntoIterator<Item = i32>) -> i32 {
        values.into_iter().sum()
    }
    assert_eq!(takes_owned(vec), 6);
}

#[test]
fn generic_functions_work_over_element_types() {
    fn sum_by<T, F>(values: &Vector<T>, project: F) -> i32
    where
        F: Fn(&T) -> i32,
    {
        values.iter().map(project).sum()
    }

    fn describe<T: Debug>(values: &Vector<T>) -> String {
        format!("{values:?}")
    }

    let points: Vector<Point> = vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]
        .into_iter()
        .collect();

    assert_eq!(sum_by(&points, |p| p.x), 4);
    assert_eq!(sum_by(&points, |p| p.y), 6);
    assert_eq!(
        describe(&points),
        "[Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]"
    );
}

#[test]
fn vectors_of_strings_behave_as_owned_values() {
    let mut vec: Vector<String> = Vector::new();
    vec.push("alpha".to_string());
    vec.push("beta".to_string());

    assert_eq!(vec.join("-"), "alpha-beta");

    // `String` is not `Copy`, so this exercises by-value iteration.
    let joined: String = vec.into_iter().collect::<Vec<_>>().join(",");
    assert_eq!(joined, "alpha,beta");
}

#[test]
fn clone_of_a_vector_with_destructors_keeps_both_alive() {
    let tracker = DropTracker::new();
    let mut vec: Vector<common::DropCounter> = Vector::new();
    for i in 0..3 {
        vec.push(common::DropCounter::new(i, &tracker));
    }

    let cloned = vec.clone();
    assert_eq!(cloned.len(), 3);
    assert_eq!(tracker.count(), 0);

    drop(vec);
    assert_eq!(tracker.count(), 3);
    assert_eq!(cloned.len(), 3, "the clone is independent");

    drop(cloned);
    assert_eq!(tracker.count(), 6);
}
