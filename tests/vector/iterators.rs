//! Tests for the three iteration forms and the iterator trait contracts.

use rust_stl::collections::vector::Vector;

use crate::common::{self, DropTracker};

fn sample() -> Vector<i32> {
    (0..5).collect()
}

#[test]
fn shared_iteration_yields_references_in_order() {
    let vec = sample();

    let collected: Vec<&i32> = vec.iter().collect();
    assert_eq!(collected, vec![&0, &1, &2, &3, &4]);

    let mut seen = Vec::new();
    for item in &vec {
        seen.push(*item);
    }
    assert_eq!(seen, vec![0, 1, 2, 3, 4]);
}

#[test]
fn shared_iteration_does_not_move_or_drop_elements() {
    let tracker = DropTracker::new();
    let mut vec: Vector<common::DropCounter> = Vector::new();
    for i in 0..4 {
        vec.push(common::DropCounter::new(i, &tracker));
    }

    let ids: Vec<i32> = vec.iter().map(|e| e.id).collect();
    assert_eq!(ids, vec![0, 1, 2, 3]);
    assert_eq!(tracker.count(), 0);

    // The vector is still fully usable after iteration.
    assert_eq!(vec.len(), 4);
    drop(vec);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn mutable_iteration_allows_in_place_mutation() {
    let mut vec = sample();

    for item in &mut vec {
        *item *= 2;
    }

    assert_eq!(vec.as_slice(), &[0, 2, 4, 6, 8]);
}

#[test]
fn mutable_iteration_does_not_move_elements() {
    let tracker = DropTracker::new();
    let mut vec: Vector<common::DropCounter> = Vector::new();
    for i in 0..4 {
        vec.push(common::DropCounter::new(i, &tracker));
    }

    for item in &mut vec {
        item.id += 100;
    }

    assert_eq!(tracker.count(), 0);
    assert_eq!(
        vec.iter().map(|e| e.id).collect::<Vec<_>>(),
        [100, 101, 102, 103]
    );

    drop(vec);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn consuming_iteration_yields_values_by_value() {
    let vec = sample();

    let owned: Vec<i32> = vec.into_iter().collect();
    assert_eq!(owned, vec![0, 1, 2, 3, 4]);

    let rebuilt: Vector<i32> = owned.into_iter().collect();
    assert_eq!(rebuilt, sample());
}

#[test]
fn all_three_forms_agree_on_values_and_order() {
    let vec = sample();

    let by_ref: Vec<i32> = vec.iter().copied().collect();
    let by_mut_ref: Vec<i32> = vec.clone().into_iter().collect();

    let mut mutable = vec.clone();
    let by_mut: Vec<i32> = (&mut mutable).into_iter().map(|n| *n).collect();

    let by_value: Vec<i32> = vec.into_iter().collect();

    assert_eq!(by_ref, by_mut_ref);
    assert_eq!(by_ref, by_mut);
    assert_eq!(by_ref, by_value);
}

#[test]
fn size_hint_is_exact_for_every_form() {
    let mut vec = sample();

    let mut shared = vec.iter();
    assert_eq!(shared.size_hint(), (5, Some(5)));
    shared.next();
    assert_eq!(shared.size_hint(), (4, Some(4)));

    let mut exclusive = vec.iter_mut();
    assert_eq!(exclusive.size_hint(), (5, Some(5)));
    exclusive.next();
    assert_eq!(exclusive.size_hint(), (4, Some(4)));

    let mut owned = vec.into_iter();
    assert_eq!(owned.size_hint(), (5, Some(5)));
    owned.next();
    assert_eq!(owned.size_hint(), (4, Some(4)));
    owned.next_back();
    assert_eq!(owned.size_hint(), (3, Some(3)));
}

#[test]
fn exact_size_iterator_len_tracks_consumption() {
    let vec = sample();

    let mut iter = vec.into_iter();
    assert_eq!(iter.len(), 5);
    iter.next();
    assert_eq!(iter.len(), 4);
    iter.next_back();
    assert_eq!(iter.len(), 3);
    iter.by_ref().for_each(drop);
    assert_eq!(iter.len(), 0);
}

#[test]
fn iterators_are_fused() {
    let vec = sample();

    let mut shared = vec.iter();
    for _ in 0..5 {
        shared.next();
    }
    assert!(shared.next().is_none());
    assert!(shared.next().is_none());

    let mut owned = vec.into_iter();
    for _ in 0..5 {
        owned.next();
    }
    assert!(owned.next().is_none());
    assert!(owned.next().is_none());
    assert!(owned.next_back().is_none());
}

#[test]
fn owned_iterator_can_be_reversed() {
    let vec = sample();

    let reversed: Vec<i32> = vec.into_iter().rev().collect();
    assert_eq!(reversed, vec![4, 3, 2, 1, 0]);
}

#[test]
fn owned_iterator_can_be_driven_from_both_ends() {
    let mut vec = sample();

    let mut front: Vec<i32> = Vec::new();
    let mut back: Vec<i32> = Vec::new();

    // Take one from each end via `iter_mut` first to exercise both accessors.
    assert_eq!(vec.iter_mut().next(), Some(&mut 0));

    let mut iter = vec.into_iter();
    front.push(iter.next().unwrap());
    back.push(iter.next_back().unwrap());
    front.push(iter.next().unwrap());
    back.push(iter.next_back().unwrap());
    front.push(iter.next().unwrap());

    assert_eq!(front, vec![0, 1, 2]);
    assert_eq!(back, vec![4, 3]);
    assert!(iter.next().is_none());
    assert!(iter.next_back().is_none());
}

#[test]
fn iterator_over_empty_vector_is_empty() {
    let vec: Vector<i32> = Vector::new();

    assert_eq!(vec.iter().next(), None);
    assert_eq!(vec.iter().size_hint(), (0, Some(0)));

    let mut owned = vec.into_iter();
    assert_eq!(owned.next(), None);
    assert_eq!(owned.next_back(), None);
    assert_eq!(owned.size_hint(), (0, Some(0)));
}

#[test]
fn iterator_over_single_element_works_from_both_ends() {
    let mut vec: Vector<i32> = Vector::new();
    vec.push(9);

    let mut owned = vec.into_iter();
    assert_eq!(owned.size_hint(), (1, Some(1)));
    assert_eq!(owned.next_back(), Some(9));
    assert_eq!(owned.next(), None);
}

#[test]
fn early_drop_of_owned_iterator_releases_remaining_elements_once() {
    let tracker = DropTracker::new();

    let mut vec: Vector<common::DropCounter> = Vector::new();
    for i in 0..6 {
        vec.push(common::DropCounter::new(i, &tracker));
    }

    let mut iter = vec.into_iter();
    let first = iter.next().expect("first element");
    assert_eq!(first.id, 0);
    assert_eq!(tracker.count(), 0, "nothing dropped while values are alive");

    drop(first);
    assert_eq!(tracker.count(), 1);

    drop(iter);
    assert_eq!(
        tracker.count(),
        6,
        "the 5 remaining elements are dropped once"
    );
}

#[test]
fn iterator_adapter_chains_work() {
    let vec = sample();

    let evens: Vec<i32> = vec.iter().copied().filter(|n| n % 2 == 0).collect();
    assert_eq!(evens, vec![0, 2, 4]);

    let sum: i32 = sample().into_iter().sum();
    assert_eq!(sum, 10);

    let doubled: Vector<i32> = sample().into_iter().map(|n| n * 2).collect();
    assert_eq!(doubled.as_slice(), &[0, 2, 4, 6, 8]);

    let zipped: Vec<(i32, i32)> = sample().into_iter().zip(sample()).collect();
    assert_eq!(zipped, vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
}

#[test]
fn iter_mut_can_be_used_with_partial_consumption() {
    let mut vec = sample();

    {
        let mut iter = vec.iter_mut();
        if let Some(first) = iter.next() {
            *first = 100;
        }
        if let Some(second) = iter.next() {
            *second = 200;
        }
    }

    assert_eq!(vec.as_slice(), &[100, 200, 2, 3, 4]);
}

#[test]
fn mutable_iteration_over_zero_sized_values() {
    let mut vec: Vector<()> = Vector::new();
    for _ in 0..4 {
        vec.push(());
    }

    let mut seen = 0;
    for _unit in &mut vec {
        seen += 1;
    }

    assert_eq!(seen, 4);
    assert_eq!(vec.len(), 4);
    assert_eq!((&vec).into_iter().count(), 4);
}
