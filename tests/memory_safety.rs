//! Destructor correctness, ownership transfer, zero-sized types and panic
//! safety. Every test here asserts *exact* drop counts, so a double drop or a
//! leak fails the test rather than passing silently.

use rust_stl::collections::vector::Vector;

mod common;
use common::{DropCounter, DropTracker, DropZst, PanicOnClone, zst_drops};

/// Builds a vector of `n` `DropCounter`s with ids `0..n`.
fn counted(n: i32, tracker: &DropTracker) -> Vector<DropCounter> {
    let mut vec = Vector::new();
    for i in 0..n {
        vec.push(DropCounter::new(i, tracker));
    }
    vec
}

#[test]
fn drop_of_vector_drops_every_element_once() {
    let tracker = DropTracker::new();
    {
        let vec = counted(5, &tracker);
        assert_eq!(vec.len(), 5);
        assert_eq!(tracker.count(), 0);
    }
    assert_eq!(tracker.count(), 5);
}

#[test]
fn drop_of_empty_vector_drops_nothing() {
    let tracker = DropTracker::new();
    {
        let vec: Vector<DropCounter> = Vector::new();
        drop(vec);
    }
    assert_eq!(tracker.count(), 0);
}

#[test]
fn clear_drops_every_element_once() {
    let tracker = DropTracker::new();
    let mut vec = counted(6, &tracker);

    vec.clear();

    assert_eq!(tracker.count(), 6);
    assert!(vec.is_empty());

    // A second clear must not drop anything again.
    vec.clear();
    assert_eq!(tracker.count(), 6);

    drop(vec);
    assert_eq!(tracker.count(), 6);
}

#[test]
fn truncate_drops_only_the_removed_tail_once() {
    let tracker = DropTracker::new();
    let mut vec = counted(6, &tracker);

    vec.truncate(2);

    assert_eq!(tracker.count(), 4);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0].id, 0);
    assert_eq!(vec[1].id, 1);

    // Truncating to a longer length drops nothing.
    vec.truncate(9);
    assert_eq!(tracker.count(), 4);

    drop(vec);
    assert_eq!(tracker.count(), 6);
}

#[test]
fn pop_returns_ownership_and_the_vector_does_not_drop_it_again() {
    let tracker = DropTracker::new();
    let mut vec = counted(3, &tracker);

    let popped = vec.pop().expect("an element");

    assert_eq!(popped.id, 2);
    assert_eq!(tracker.count(), 0, "a returned element must stay alive");

    drop(popped);
    assert_eq!(tracker.count(), 1);

    drop(vec);
    assert_eq!(tracker.count(), 3);
}

#[test]
fn remove_returns_ownership_and_drops_the_rest_once() {
    let tracker = DropTracker::new();
    let mut vec = counted(4, &tracker);

    let removed = vec.remove(1);

    assert_eq!(removed.id, 1);
    assert_eq!(tracker.count(), 0);
    assert_eq!(vec.len(), 3);

    drop(removed);
    assert_eq!(tracker.count(), 1);

    drop(vec);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn swap_remove_returns_ownership_and_drops_the_rest_once() {
    let tracker = DropTracker::new();
    let mut vec = counted(4, &tracker);

    let removed = vec.swap_remove(0).expect("an element");
    assert_eq!(removed.id, 0);
    assert_eq!(tracker.count(), 0);
    assert_eq!(vec.len(), 3);

    drop(removed);
    assert_eq!(tracker.count(), 1);

    drop(vec);
    assert_eq!(tracker.count(), 4);
}

#[test]
fn retain_drops_only_rejected_elements_once() {
    let tracker = DropTracker::new();
    let mut vec = counted(10, &tracker);

    vec.retain(|e| e.id % 2 == 0);

    assert_eq!(tracker.count(), 5, "the five odd ids are dropped");
    assert_eq!(vec.len(), 5);
    assert_eq!(
        vec.iter().map(|e| e.id).collect::<Vec<_>>(),
        [0, 2, 4, 6, 8]
    );

    drop(vec);
    assert_eq!(tracker.count(), 10);
}

#[test]
fn remove_where_and_remove_all_drop_elements_once() {
    let tracker = DropTracker::new();
    let mut vec = counted(6, &tracker);

    vec.remove_where(|e| e.id < 2);
    assert_eq!(tracker.count(), 2);

    vec.remove_all(DropCounter::new(3, &tracker));
    // The probe value itself is dropped as well.
    assert_eq!(tracker.count(), 4);
    assert_eq!(vec.len(), 3);

    drop(vec);
    assert_eq!(tracker.count(), 7);
}

#[test]
fn dedup_drops_removed_duplicates_once() {
    let tracker = DropTracker::new();
    let mut vec: Vector<DropCounter> = Vector::new();

    // ids: 0, 0, 1, 1, 1, 0
    for id in [0, 0, 1, 1, 1, 0] {
        vec.push(DropCounter::new(id, &tracker));
    }

    vec.dedup();

    assert_eq!(tracker.count(), 3, "three consecutive duplicates removed");
    assert_eq!(vec.iter().map(|e| e.id).collect::<Vec<_>>(), [0, 1, 0]);

    drop(vec);
    assert_eq!(tracker.count(), 6);
}

#[test]
fn remove_duplicates_drops_repeats_once() {
    let tracker = DropTracker::new();
    let mut vec: Vector<DropCounter> = Vector::new();

    for id in [0, 1, 0, 1, 2] {
        vec.push(DropCounter::new(id, &tracker));
    }

    vec.remove_duplicates();

    assert_eq!(tracker.count(), 2);
    assert_eq!(vec.iter().map(|e| e.id).collect::<Vec<_>>(), [0, 1, 2]);

    drop(vec);
    assert_eq!(tracker.count(), 5);
}

#[test]
fn clone_produces_independent_values_that_are_each_dropped_once() {
    let tracker = DropTracker::new();

    {
        let original = counted(4, &tracker);
        let cloned = original.clone();

        assert_eq!(tracker.count(), 0);
        assert_eq!(cloned.len(), 4);
        assert_eq!(original, cloned);

        drop(original);
        assert_eq!(tracker.count(), 4, "the original's own elements only");

        drop(cloned);
        assert_eq!(tracker.count(), 8);
    }

    assert_eq!(tracker.count(), 8);
}

#[test]
fn consuming_the_owned_iterator_fully_drops_each_yielded_value_once() {
    let tracker = DropTracker::new();
    let vec = counted(5, &tracker);

    let mut ids = Vec::new();
    for element in vec {
        ids.push(element.id);
    }

    assert_eq!(ids, [0, 1, 2, 3, 4]);
    assert_eq!(tracker.count(), 5, "each value dropped exactly once");
}

#[test]
fn dropping_a_fully_consumed_owned_iterator_drops_nothing_more() {
    let tracker = DropTracker::new();
    let vec = counted(5, &tracker);

    let mut iter = vec.into_iter();
    let mut ids = Vec::new();
    for element in iter.by_ref() {
        ids.push(element.id);
    }
    assert_eq!(ids, [0, 1, 2, 3, 4]);
    assert_eq!(tracker.count(), 5);

    // An exhausted iterator stays exhausted and reports nothing remaining.
    assert!(iter.next().is_none());
    assert!(iter.next_back().is_none());
    assert_eq!(iter.size_hint(), (0, Some(0)));

    drop(iter);
    assert_eq!(tracker.count(), 5, "no double drop from the iterator");
}

#[test]
fn dropping_a_partially_consumed_owned_iterator_drops_remaining_once() {
    let tracker = DropTracker::new();
    let vec = counted(5, &tracker);

    let mut iter = vec.into_iter();
    let first = iter.next().expect("first");
    let second = iter.next().expect("second");
    assert_eq!((first.id, second.id), (0, 1));
    assert_eq!(tracker.count(), 0);

    drop(iter);
    assert_eq!(tracker.count(), 3, "elements 2, 3 and 4 only");

    drop(first);
    drop(second);
    assert_eq!(tracker.count(), 5);
}

#[test]
fn dropping_an_unconsumed_owned_iterator_drops_everything_once() {
    let tracker = DropTracker::new();
    let vec = counted(4, &tracker);

    let iter = vec.into_iter();
    drop(iter);

    assert_eq!(tracker.count(), 4);
}

#[test]
fn owned_iterator_drop_count_is_exact_with_double_ended_consumption() {
    let tracker = DropTracker::new();
    let vec = counted(6, &tracker);

    let mut iter = vec.into_iter();
    let a = iter.next().expect("front");
    let b = iter.next_back().expect("back");
    let c = iter.next().expect("front");
    assert_eq!((a.id, b.id, c.id), (0, 5, 1));
    assert_eq!(tracker.count(), 0);

    drop(iter);
    assert_eq!(tracker.count(), 3, "elements 2, 3 and 4 remain");

    drop((a, b, c));
    assert_eq!(tracker.count(), 6);
}

#[test]
fn append_transfers_ownership_without_double_drop() {
    let tracker = DropTracker::new();

    {
        let mut head = counted(3, &tracker);
        let mut tail = counted(4, &tracker);

        head.append(&mut tail);

        assert_eq!(tracker.count(), 0, "append moves, it does not copy");
        assert_eq!(head.len(), 7);
        assert!(tail.is_empty());

        drop(head);
        assert_eq!(tracker.count(), 7);
    }

    assert_eq!(tracker.count(), 7);
}

#[test]
fn append_of_an_empty_vector_drops_nothing() {
    let tracker = DropTracker::new();
    let mut head = counted(2, &tracker);
    let mut tail: Vector<DropCounter> = Vector::new();

    head.append(&mut tail);

    assert_eq!(tracker.count(), 0);
    assert_eq!(head.len(), 2);
}

#[test]
fn extend_and_extend_from_slice_do_not_drop_the_source() {
    let tracker = DropTracker::new();
    let values: Vec<DropCounter> = DropCounter::range(3, &tracker);
    let mut vec: Vector<DropCounter> = Vector::new();

    vec.extend_from_slice(&values);

    assert_eq!(tracker.count(), 0, "cloning creates new live values");
    assert_eq!(vec.len(), 3);
    assert_eq!(values.len(), 3);

    vec.extend(DropCounter::range(2, &tracker));
    assert_eq!(vec.len(), 5);
    assert_eq!(tracker.count(), 0);
}

#[test]
fn resize_and_resize_with_drop_the_removed_tail_once() {
    let tracker = DropTracker::new();
    let mut vec = counted(5, &tracker);

    vec.resize(2, DropCounter::new(99, &tracker));

    // Four elements removed (ids 2..5) plus the unused filler value.
    assert_eq!(tracker.count(), 4);
    assert_eq!(vec.len(), 2);

    vec.resize_with(4, || DropCounter::new(7, &tracker));
    assert_eq!(vec.len(), 4);
    assert_eq!(tracker.count(), 4);

    drop(vec);
    assert_eq!(tracker.count(), 8, "4 filler + 4 final elements");
}

#[test]
fn zero_sized_type_with_drop_has_exact_destructor_counts() {
    let before = zst_drops();

    {
        let mut vec: Vector<DropZst> = Vector::new();
        for _ in 0..100 {
            vec.push(DropZst);
        }
        assert_eq!(vec.len(), 100);
        assert_eq!(zst_drops(), before, "pushing ZSTs must not drop anything");

        vec.truncate(60);
        assert_eq!(zst_drops(), before + 40);

        let popped = vec.pop();
        assert_eq!(zst_drops(), before + 40, "pop returns the value");
        drop(popped);
        assert_eq!(zst_drops(), before + 41);

        // Fully consuming the remaining vector drops each element once.
        let remaining = vec.len();
        let consumed = vec.into_iter().count();
        assert_eq!(consumed, remaining);
        assert_eq!(zst_drops(), before + 41 + remaining);
    }

    assert_eq!(
        zst_drops(),
        before + 100,
        "100 ZSTs dropped, exactly once each"
    );

    // Partially consuming a zero-sized-value iterator must also drop each
    // remaining element exactly once.
    let mut vec: Vector<DropZst> = Vector::new();
    for _ in 0..5 {
        vec.push(DropZst);
    }
    {
        let mut iter = vec.into_iter();
        drop(iter.next());
        assert_eq!(zst_drops(), before + 101);
        drop(iter.next_back());
        assert_eq!(zst_drops(), before + 102);
        drop(iter);
        assert_eq!(zst_drops(), before + 105, "the three remaining ZSTs");
    }

    // The test is the only user of this counter, so nothing else may add to it.
    assert_eq!(zst_drops(), before + 105);
}

#[test]
fn panicking_predicate_leaves_the_vector_valid_and_drops_each_element_once() {
    let tracker = DropTracker::new();
    let mut vec = counted(5, &tracker);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.retain(|e| {
            if e.id == 3 {
                panic!("predicate panic requested by test");
            }
            e.id != 1
        });
    }));

    assert!(result.is_err(), "the predicate panic must propagate");

    // Element 1 was already rejected and dropped before the panic.
    assert_eq!(tracker.count(), 1);
    // Element 3 was never processed, so it is still alive.
    assert_eq!(vec.len(), 4);
    assert!(vec.iter().any(|e| e.id == 3));

    drop(vec);
    assert_eq!(tracker.count(), 5, "every element dropped exactly once");
}

#[test]
fn panicking_clone_does_not_corrupt_or_leak_the_source_vector() {
    let tracker = DropTracker::new();

    let clones = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut original: Vector<PanicOnClone> = Vector::new();
    for i in 0..5 {
        original.push(PanicOnClone::new(i, 2, &clones, &tracker));
    }
    assert_eq!(tracker.count(), 0);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| original.clone()));
    assert!(result.is_err(), "the clone panic must propagate");

    // Two clones were created before the panic; the partially built vector is
    // dropped during unwinding, so those two are released exactly once.
    assert_eq!(tracker.count(), 2);

    // The source is untouched and still owns all five elements.
    assert_eq!(original.len(), 5);
    assert_eq!(
        original.iter().map(|e| e.id).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4]
    );

    drop(original);
    assert_eq!(tracker.count(), 7);
}

#[test]
fn panicking_partial_eq_does_not_corrupt_either_vector() {
    let tracker = DropTracker::new();
    let left = counted(3, &tracker);
    let right = counted(3, &tracker);

    struct PanickingEq;
    impl PartialEq for PanickingEq {
        fn eq(&self, _: &Self) -> bool {
            panic!("equality panic requested by test");
        }
    }

    let a: Vector<PanickingEq> = vec![PanickingEq, PanickingEq].into_iter().collect();
    let b: Vector<PanickingEq> = vec![PanickingEq, PanickingEq].into_iter().collect();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a == b));

    assert!(result.is_err());
    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 2);
    // Comparison never moves or drops elements.
    assert_eq!(tracker.count(), 0);
    assert_eq!(left, right);
}

#[test]
fn clone_of_a_non_copy_type_allocates_independently() {
    let tracker = DropTracker::new();
    let original = counted(8, &tracker);
    let cloned = original.clone();

    // Capacity is an implementation detail, but the data must be separate and
    // independent: mutating the clone must not affect the original.
    let mut cloned = cloned;
    cloned[0].id = 1_000;

    assert_eq!(original[0].id, 0);
    assert_eq!(cloned[0].id, 1_000);
    assert_eq!(tracker.count(), 0);
}

#[test]
fn many_growth_cycles_do_not_leak_or_double_drop() {
    let tracker = DropTracker::new();

    {
        let mut vec: Vector<DropCounter> = Vector::new();
        for i in 0..500 {
            vec.push(DropCounter::new(i, &tracker));
        }
        // Reallocation moves elements without dropping them.
        assert_eq!(tracker.count(), 0);

        vec.truncate(200);
        assert_eq!(tracker.count(), 300);

        drop(vec);
        assert_eq!(tracker.count(), 500);
    }

    assert_eq!(tracker.count(), 500);
}
