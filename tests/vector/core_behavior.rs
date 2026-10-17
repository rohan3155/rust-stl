//! Behavioural tests for the public `Vector` API.

use rust_stl::collections::vector::Vector;

#[test]
fn new_vector_is_empty() {
    let vec: Vector<i32> = Vector::new();

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 0);
    assert!(vec.is_empty());
    assert_eq!(vec.as_slice(), &[] as &[i32]);
    assert!(vec.first().is_none());
    assert!(vec.last().is_none());
    assert_eq!(vec.get(0), None);
}

#[test]
fn default_matches_new() {
    let default: Vector<i32> = Vector::default();
    let created: Vector<i32> = Vector::new();

    assert_eq!(default, created);
    assert!(default.is_empty());
    assert_eq!(default.capacity(), 0);
}

#[test]
fn push_and_pop_across_multiple_growths() {
    let mut vec = Vector::new();
    let mut previous_capacity = vec.capacity();

    for i in 0..1_000 {
        vec.push(i);
        // Capacity must never shrink during growth and must always be
        // sufficient, but the growth factor itself is not part of the contract.
        assert!(vec.capacity() >= vec.len());
        assert!(vec.capacity() >= previous_capacity);
        previous_capacity = vec.capacity();
    }

    assert_eq!(vec.len(), 1_000);
    assert!(
        vec.capacity() > 4,
        "expected several growth steps, capacity stayed at {}",
        vec.capacity()
    );

    for i in (0..1_000).rev() {
        assert_eq!(vec.pop(), Some(i));
    }

    assert!(vec.is_empty());
    assert_eq!(vec.pop(), None);
}

#[test]
fn push_preserves_element_order_and_values() {
    let mut vec = Vector::new();

    for i in 0..50 {
        vec.push(i * 2);
    }

    for i in 0..50 {
        assert_eq!(vec[i], i * 2);
    }
}

#[test]
fn indexing_reads_and_writes() {
    let mut vec: Vector<i32> = (0..5).collect();

    assert_eq!(vec[0], 0);
    assert_eq!(vec[4], 4);

    vec[2] = 42;
    assert_eq!(vec[2], 42);
    assert_eq!(vec.as_slice(), &[0, 1, 42, 3, 4]);
}

#[test]
fn get_returns_none_out_of_bounds() {
    let mut vec: Vector<i32> = (0..3).collect();

    assert_eq!(vec.get(2), Some(&2));
    assert_eq!(vec.get(3), None);
    assert_eq!(vec.get(usize::MAX), None);

    assert_eq!(vec.get_mut(0), Some(&mut 0));
    assert_eq!(vec.get_mut(3), None);

    let empty: Vector<i32> = Vector::new();
    assert_eq!(empty.get(0), None);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn index_read_panics_out_of_bounds() {
    let vec: Vector<i32> = (0..3).collect();
    let _ = vec[3];
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn index_write_panics_out_of_bounds() {
    let mut vec: Vector<i32> = (0..3).collect();
    vec[3] = 1;
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn index_on_empty_vector_panics() {
    let vec: Vector<i32> = Vector::new();
    let _ = vec[0];
}

#[test]
fn first_and_last() {
    let mut vec: Vector<i32> = Vector::new();
    assert!(vec.first().is_none());
    assert!(vec.first_mut().is_none());
    assert!(vec.last().is_none());
    assert!(vec.last_mut().is_none());

    vec.push(1);
    assert_eq!(vec.first(), Some(&1));
    assert_eq!(vec.last(), Some(&1));

    vec.push(2);
    vec.push(3);
    assert_eq!(vec.first(), Some(&1));
    assert_eq!(vec.last(), Some(&3));

    *vec.first_mut().unwrap() = 10;
    *vec.last_mut().unwrap() = 30;
    assert_eq!(vec.as_slice(), &[10, 2, 30]);
}

#[test]
fn insert_at_beginning_middle_and_end() {
    let mut vec: Vector<i32> = (0..3).collect();

    vec.insert(0, -1);
    assert_eq!(vec.as_slice(), &[-1, 0, 1, 2]);

    vec.insert(2, 99);
    assert_eq!(vec.as_slice(), &[-1, 0, 99, 1, 2]);

    vec.insert(vec.len(), 100);
    assert_eq!(vec.as_slice(), &[-1, 0, 99, 1, 2, 100]);

    assert_eq!(vec.len(), 6);
}

#[test]
fn insert_into_empty_vector() {
    let mut vec: Vector<i32> = Vector::new();
    vec.insert(0, 7);

    assert_eq!(vec.len(), 1);
    assert_eq!(vec[0], 7);
}

#[test]
fn insert_into_full_vector_forces_growth() {
    let mut vec: Vector<i32> = Vector::new();
    vec.reserve_exact(3);

    for i in 0..3 {
        vec.push(i);
    }

    let capacity_before = vec.capacity();
    vec.insert(1, 42);

    assert!(vec.capacity() > capacity_before);
    assert_eq!(vec.as_slice(), &[0, 42, 1, 2]);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn insert_past_end_panics() {
    let mut vec: Vector<i32> = (0..3).collect();
    vec.insert(4, 1);
}

#[test]
fn remove_shifts_tail_left() {
    let mut vec: Vector<i32> = (0..5).collect();

    assert_eq!(vec.remove(0), 0);
    assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);

    assert_eq!(vec.remove(1), 2);
    assert_eq!(vec.as_slice(), &[1, 3, 4]);

    assert_eq!(vec.remove(vec.len() - 1), 4);
    assert_eq!(vec.as_slice(), &[1, 3]);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn remove_out_of_bounds_panics() {
    let mut vec: Vector<i32> = (0..3).collect();
    vec.remove(3);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn remove_from_empty_vector_panics() {
    let mut vec: Vector<i32> = Vector::new();
    vec.remove(0);
}

#[test]
fn swap_exchanges_two_elements() {
    let mut vec: Vector<i32> = (0..4).collect();

    vec.swap(0, 3);
    assert_eq!(vec.as_slice(), &[3, 1, 2, 0]);

    // Swapping an element with itself is a no-op.
    vec.swap(1, 1);
    assert_eq!(vec.as_slice(), &[3, 1, 2, 0]);
}

#[test]
#[should_panic(expected = "index1 out of bounds")]
fn swap_panics_on_first_out_of_bounds() {
    let mut vec: Vector<i32> = (0..2).collect();
    vec.swap(2, 0);
}

#[test]
#[should_panic(expected = "index2 out of bounds")]
fn swap_panics_on_second_out_of_bounds() {
    let mut vec: Vector<i32> = (0..2).collect();
    vec.swap(0, 2);
}

#[test]
fn swap_remove_returns_the_indexed_element_and_fills_the_hole_with_the_last() {
    let mut vec: Vector<i32> = (0..4).collect();

    // The element *at* the index is returned; the last element takes its place.
    assert_eq!(vec.swap_remove(0), Some(0));
    assert_eq!(vec.as_slice(), &[3, 1, 2]);

    assert_eq!(vec.swap_remove(1), Some(1));
    assert_eq!(vec.as_slice(), &[3, 2]);

    assert_eq!(vec.swap_remove(1), Some(2));
    assert_eq!(vec.as_slice(), &[3]);

    assert_eq!(vec.swap_remove(0), Some(3));
    assert!(vec.is_empty());
    assert_eq!(vec.swap_remove(0), None);
}

#[test]
fn clear_empties_but_keeps_capacity() {
    let mut vec: Vector<i32> = (0..8).collect();
    let capacity = vec.capacity();
    assert!(capacity >= 8);

    vec.clear();

    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), capacity, "clear must not release capacity");
    assert_eq!(vec.as_slice(), &[] as &[i32]);

    // The buffer is reusable after clearing.
    vec.push(5);
    assert_eq!(vec.as_slice(), &[5]);
}

#[test]
fn truncate_shortens_and_is_a_no_op_when_lengthening() {
    let mut vec: Vector<i32> = (0..6).collect();

    vec.truncate(3);
    assert_eq!(vec.as_slice(), &[0, 1, 2]);

    // Truncating to a larger length does nothing at all.
    vec.truncate(10);
    assert_eq!(vec.as_slice(), &[0, 1, 2]);

    vec.truncate(0);
    assert!(vec.is_empty());

    vec.truncate(0);
    assert!(vec.is_empty());
}

#[test]
fn find_index_count_matches_and_indices_of() {
    let vec: Vector<i32> = vec![1, 5, 2, 5, 3].into_iter().collect();

    assert_eq!(vec.find_index(|n| *n > 3), Some(1));
    assert_eq!(vec.find_index(|n| *n > 100), None);
    assert_eq!(vec.count_matches(|n| *n == 5), 2);
    assert_eq!(vec.count_matches(|_| false), 0);
    assert_eq!(vec.indices_of(|n| *n == 5).as_slice(), &[1, 3]);
    assert!(vec.indices_of(|_| false).is_empty());
}

#[test]
fn remove_first_removes_only_the_first_match() {
    let mut vec: Vector<i32> = vec![1, 2, 2, 3].into_iter().collect();

    assert!(vec.remove_first(&2));
    assert_eq!(vec.as_slice(), &[1, 2, 3]);

    assert!(vec.remove_first(&1));
    assert_eq!(vec.as_slice(), &[2, 3]);

    assert!(!vec.remove_first(&99));
    assert_eq!(vec.as_slice(), &[2, 3]);
}

#[test]
fn retain_keeps_matching_elements_and_preserves_order() {
    let mut vec: Vector<i32> = (0..10).collect();

    vec.retain(|n| n % 2 == 0);

    assert_eq!(vec.as_slice(), &[0, 2, 4, 6, 8]);
}

#[test]
fn retain_everything_and_retain_nothing() {
    let mut all: Vector<i32> = (0..5).collect();
    all.retain(|_| true);
    assert_eq!(all.as_slice(), &[0, 1, 2, 3, 4]);

    let mut none: Vector<i32> = (0..5).collect();
    none.retain(|_| false);
    assert!(none.is_empty());
}

#[test]
fn retain_mut_can_mutate_kept_elements() {
    let mut vec: Vector<i32> = (0..6).collect();

    vec.retain_mut(|n| {
        if *n % 2 == 0 {
            *n *= 10;
            true
        } else {
            false
        }
    });

    assert_eq!(vec.as_slice(), &[0, 20, 40]);
}

#[test]
fn remove_where_removes_matching_elements() {
    let mut vec: Vector<i32> = (0..6).collect();

    vec.remove_where(|n| *n % 2 == 0);

    assert_eq!(vec.as_slice(), &[1, 3, 5]);
}

#[test]
fn remove_where_and_retain_use_opposite_polarities() {
    let mut retained: Vector<i32> = (0..6).collect();
    let mut removed: Vector<i32> = (0..6).collect();

    retained.retain(|n| *n % 2 == 0);
    removed.remove_where(|n| *n % 2 == 0);

    assert_eq!(retained.as_slice(), &[0, 2, 4]);
    assert_eq!(removed.as_slice(), &[1, 3, 5]);
}

#[test]
fn remove_where_mut_can_inspect_and_remove() {
    let mut vec: Vector<i32> = (0..6).collect();

    vec.remove_where_mut(|n| {
        *n += 100;
        true
    });

    assert!(vec.is_empty());
}

#[test]
fn remove_all_removes_every_value_equal_to_the_argument() {
    let mut vec: Vector<i32> = vec![1, 2, 1, 3, 1].into_iter().collect();

    vec.remove_all(1);

    assert_eq!(vec.as_slice(), &[2, 3]);
}

#[test]
fn remove_duplicates_keeps_first_occurrence_of_each_value() {
    let mut vec: Vector<i32> = vec![3, 1, 3, 2, 1, 3].into_iter().collect();

    vec.remove_duplicates();

    assert_eq!(vec.as_slice(), &[3, 1, 2]);
}

#[test]
fn dedup_removes_only_consecutive_duplicates() {
    let mut vec: Vector<i32> = vec![1, 1, 2, 2, 2, 3, 1, 1].into_iter().collect();

    vec.dedup();

    assert_eq!(vec.as_slice(), &[1, 2, 3, 1]);
}

#[test]
fn dedup_leaves_distinct_and_short_vectors_alone() {
    let mut distinct: Vector<i32> = (0..3).collect();
    distinct.dedup();
    assert_eq!(distinct.as_slice(), &[0, 1, 2]);

    let mut empty: Vector<i32> = Vector::new();
    empty.dedup();
    assert!(empty.is_empty());

    let mut single: Vector<i32> = Vector::new();
    single.push(7);
    single.dedup();
    assert_eq!(single.as_slice(), &[7]);
}

#[test]
fn reverse_mirrors_the_order() {
    let mut vec: Vector<i32> = (0..5).collect();
    vec.reverse();
    assert_eq!(vec.as_slice(), &[4, 3, 2, 1, 0]);

    // Idempotent for empty and single-element vectors.
    let mut empty: Vector<i32> = Vector::new();
    empty.reverse();
    assert!(empty.is_empty());

    let mut single: Vector<i32> = Vector::new();
    single.push(1);
    single.reverse();
    assert_eq!(single.as_slice(), &[1]);

    let mut two: Vector<i32> = (0..2).collect();
    two.reverse();
    assert_eq!(two.as_slice(), &[1, 0]);
}

#[test]
fn sort_sort_by_and_sort_range() {
    let mut vec: Vector<i32> = vec![5, 1, 4, 2, 3].into_iter().collect();
    vec.sort();
    assert_eq!(vec.as_slice(), &[1, 2, 3, 4, 5]);

    let mut desc: Vector<i32> = vec![1, 3, 2].into_iter().collect();
    desc.sort_by(|a, b| b.cmp(a));
    assert_eq!(desc.as_slice(), &[3, 2, 1]);

    // `sort_range` treats its argument as the half-open range `start..end`.
    let mut partial: Vector<i32> = vec![9, 8, 3, 1, 2, 0].into_iter().collect();
    partial.sort_range(2, 5);
    assert_eq!(partial.as_slice(), &[9, 8, 1, 2, 3, 0]);
    // The element at `end` is not touched.
    assert_eq!(partial[5], 0);
}

#[test]
fn sort_range_accepts_empty_and_full_ranges() {
    let mut vec: Vector<i32> = vec![3, 1, 2].into_iter().collect();

    vec.sort_range(1, 1);
    assert_eq!(vec.as_slice(), &[3, 1, 2]);

    vec.sort_range(0, 3);
    assert_eq!(vec.as_slice(), &[1, 2, 3]);
}

#[test]
#[should_panic(expected = "invalid range")]
fn sort_range_panics_when_start_exceeds_end() {
    let mut vec: Vector<i32> = (0..3).collect();
    vec.sort_range(2, 1);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn sort_range_panics_when_end_exceeds_length() {
    let mut vec: Vector<i32> = (0..3).collect();
    vec.sort_range(0, 4);
}

#[test]
fn binary_search_results() {
    let vec: Vector<i32> = (0..10).map(|n| n * 2).collect();

    assert_eq!(vec.binary_search(&0), Ok(0));
    assert_eq!(vec.binary_search(&8), Ok(4));
    // An absent value reports the index at which it would be inserted.
    assert_eq!(vec.binary_search(&9), Err(5));
    assert_eq!(vec.binary_search(&-1), Err(0));
    assert_eq!(vec.binary_search(&100), Err(10));

    assert_eq!(vec.binary_search_by(|probe| probe.cmp(&6)), Ok(3));
    assert_eq!(
        vec.binary_search_by_key(&6, |probe| *probe),
        Ok(3),
        "by-key search with an equal key"
    );
}

#[test]
fn binary_search_on_empty_vector() {
    let vec: Vector<i32> = Vector::new();
    assert_eq!(vec.binary_search(&1), Err(0));
    assert_eq!(vec.binary_search_by(|_| std::cmp::Ordering::Equal), Err(0));
}

#[test]
fn rotate_left_and_right() {
    let mut left: Vector<i32> = (0..5).collect();
    left.rotate_left(2);
    assert_eq!(left.as_slice(), &[2, 3, 4, 0, 1]);

    let mut right: Vector<i32> = (0..5).collect();
    right.rotate_right(2);
    assert_eq!(right.as_slice(), &[3, 4, 0, 1, 2]);
}

#[test]
fn rotate_by_zero_and_full_length_is_a_no_op() {
    let original: Vector<i32> = (0..4).collect();

    let mut zero: Vector<i32> = original.clone();
    zero.rotate_left(0);
    zero.rotate_right(0);
    assert_eq!(zero, original);

    let mut full: Vector<i32> = original.clone();
    full.rotate_left(full.len());
    full.rotate_right(full.len());
    assert_eq!(full, original);
}

#[test]
#[should_panic(expected = "mid out of bounds")]
fn rotate_left_panics_past_length() {
    let mut vec: Vector<i32> = (0..3).collect();
    vec.rotate_left(4);
}

#[test]
#[should_panic(expected = "mid out of bounds")]
fn rotate_right_panics_past_length() {
    let mut vec: Vector<i32> = (0..3).collect();
    vec.rotate_right(4);
}

#[test]
fn split_at_returns_two_non_overlapping_slices() {
    let vec: Vector<i32> = (0..6).collect();

    let (head, tail) = vec.split_at(2);
    assert_eq!(head, &[0, 1]);
    assert_eq!(tail, &[2, 3, 4, 5]);

    let (all, none) = vec.split_at(0);
    assert!(all.is_empty());
    assert_eq!(none.len(), 6);

    let (everything, empty) = vec.split_at(6);
    assert_eq!(everything.len(), 6);
    assert!(empty.is_empty());
}

#[test]
#[should_panic]
fn split_at_past_length_panics() {
    let vec: Vector<i32> = (0..3).collect();
    let _ = vec.split_at(4);
}

#[test]
fn windows_overlap_and_chunks_do_not() {
    let vec: Vector<i32> = (0..5).collect();

    let windows: Vec<&[i32]> = vec.windows(3).collect();
    assert_eq!(windows, vec![&[0, 1, 2], &[1, 2, 3], &[2, 3, 4]]);

    let chunks: Vec<&[i32]> = vec.chunks(2).collect();
    assert_eq!(chunks, vec![&[0, 1][..], &[2, 3][..], &[4][..]]);
}

#[test]
fn windows_and_chunks_on_empty_and_oversized_input() {
    let empty: Vector<i32> = Vector::new();
    assert_eq!(empty.windows(1).count(), 0);
    assert_eq!(empty.chunks(1).count(), 0);

    let vec: Vector<i32> = (0..3).collect();
    // A window wider than the vector yields nothing.
    assert_eq!(vec.windows(4).count(), 0);
    // A chunk wider than the vector yields the whole vector once.
    assert_eq!(vec.chunks(10).collect::<Vec<_>>(), vec![&[0, 1, 2][..]]);
}

#[test]
fn begin_and_end_expose_the_initialized_range() {
    let vec: Vector<i32> = (0..4).collect();

    let begin = vec.begin();
    assert!(!begin.is_null());
    assert_eq!(begin, vec.as_slice().as_ptr());
    assert_eq!(unsafe { vec.end().offset_from(begin) }, 4);

    // For an empty vector the two pointers are equal but still valid.
    let empty: Vector<i32> = Vector::new();
    assert!(!empty.begin().is_null());
    assert_eq!(empty.begin(), empty.end());
}
