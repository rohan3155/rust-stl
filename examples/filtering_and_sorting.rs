//! Filtering, de-duplication, sorting and searching.
//!
//! Run with:
//!
//! ```text
//! cargo run --example filtering_and_sorting
//! ```

use rust_stl::collections::vector::Vector;

fn main() {
    let mut values: Vector<i32> = vec![5, 3, 5, 1, 3, 9, 1, 7].into_iter().collect();
    println!("start:            {values:?}");

    // `retain` KEEPS matching elements; `remove_where` REMOVES them. Both
    // preserve the relative order of what survives.
    let mut kept = values.clone();
    kept.retain(|n| n % 2 == 1);
    println!("retain odd:       {kept:?}");

    let mut removed = values.clone();
    removed.remove_where(|n| n % 2 == 1);
    println!("remove odd:       {removed:?}");

    // `retain_mut` lets the predicate change the elements it keeps.
    let mut doubled_evens = values.clone();
    doubled_evens.retain_mut(|n| {
        if *n % 2 == 0 {
            *n *= 10;
            true
        } else {
            false
        }
    });
    println!("retain_mut evens: {doubled_evens:?}");

    // `dedup` removes only CONSECUTIVE duplicates...
    let mut consecutive: Vector<i32> = vec![1, 1, 2, 2, 3, 1].into_iter().collect();
    consecutive.dedup();
    println!("dedup:            {consecutive:?}  (consecutive duplicates)");

    // ...whereas `remove_duplicates` keeps only the first occurrence of each
    // distinct value, wherever it appears.
    let mut distinct: Vector<i32> = vec![1, 1, 2, 2, 3, 1].into_iter().collect();
    distinct.remove_duplicates();
    println!("remove_duplicates:{distinct:?}  (first occurrence only)");

    // Sorting, then searching. `binary_search` requires a sorted vector and
    // distinguishes `Ok(index)` from `Err(insertion_index)`.
    values.sort();
    println!("sorted:           {values:?}");
    println!("search 7 -> {:?}", values.binary_search(&7));
    println!(
        "search 4 -> {:?} (insertion point)",
        values.binary_search(&4)
    );

    // `sort_range` sorts the half-open range `start..end` only.
    let mut partially: Vector<i32> = vec![9, 8, 3, 1, 2, 0].into_iter().collect();
    partially.sort_range(2, 5);
    println!("sort_range(2,5):  {partially:?}  (index 5 untouched)");

    // Rotations, windows and chunks.
    let mut rotated: Vector<i32> = (1..=5).collect();
    rotated.rotate_left(2);
    println!("rotate_left(2):   {rotated:?}");

    let data: Vector<i32> = (1..=5).collect();
    let windows: Vec<&[i32]> = data.windows(3).collect();
    let chunks: Vec<&[i32]> = data.chunks(2).collect();
    println!("windows(3):       {windows:?}  (overlapping)");
    println!("chunks(2):        {chunks:?}  (non-overlapping)");

    // A quick word count, to show the container in a slightly larger example.
    let text = "the quick brown fox jumps over the lazy dog the fox";
    let mut words: Vector<&str> = text.split_whitespace().collect();
    words.sort();
    let mut counts: Vector<(&str, usize)> = Vector::new();
    for word in &words {
        match counts.last_mut() {
            Some((last, count)) if *last == *word => *count += 1,
            _ => counts.push((word, 1)),
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("most common words: {counts:?}");
}
