//! A tour of the core `Vector` API.
//!
//! Run with:
//!
//! ```text
//! cargo run --example basic_usage
//! ```

use rust_stl::collections::vector::Vector;

fn main() {
    // --- Construction -------------------------------------------------------
    //
    // `new` does not allocate; the vector starts empty with zero capacity.
    let mut readings: Vector<i32> = Vector::new();
    println!(
        "fresh: len={} capacity={}",
        readings.len(),
        readings.capacity()
    );

    // --- Appending and pushing ---------------------------------------------
    readings.push(30);
    readings.push(10);
    readings.push(20);
    println!(
        "after pushes: {readings:?} (capacity={})",
        readings.capacity()
    );

    // --- Inserting ---------------------------------------------------------
    //
    // `insert` shifts the tail right, so it is O(len). `index == len` is the
    // same as pushing.
    readings.insert(1, 15);
    println!("after insert(1, 15): {readings:?}");

    // --- Reading -----------------------------------------------------------
    //
    // Indexing panics when out of range; `get` returns `Option` instead.
    let second = readings[1];
    println!("index 1 is {second}");
    println!("index 99 is {:?}", readings.get(99));

    // --- Updating ----------------------------------------------------------
    //
    // `get_mut` and `IndexMut` both hand out a mutable borrow.
    if let Some(first) = readings.first_mut() {
        *first += 1;
    }
    readings[0] = readings[0] * 2;
    println!("after updates: {readings:?}");

    // --- Iterating ---------------------------------------------------------
    let total: i32 = readings.iter().sum();
    println!("sum = {total}");

    // --- Removing ----------------------------------------------------------
    //
    // `remove` preserves order and is O(len); `swap_remove` is O(1) but does
    // not preserve order.
    let removed = readings.remove(1);
    println!("remove(1) returned {removed}: {readings:?}");

    let swapped = readings.swap_remove(0);
    println!("swap_remove(0) returned {swapped:?}: {readings:?}");

    // --- Sorting and searching --------------------------------------------
    readings.extend([7, 3, 9, 3]);
    readings.sort();
    println!("sorted: {readings:?}");

    // A successful search yields `Ok(index)`; a miss yields the insertion point.
    match readings.binary_search(&7) {
        Ok(index) => println!("found 7 at index {index}"),
        Err(index) => println!("7 would be inserted at index {index}"),
    }

    // --- Consuming ---------------------------------------------------------
    //
    // `into_iter` yields owned values; the vector is gone afterwards.
    let doubled: Vector<i32> = readings.into_iter().map(|n| n * 2).collect();
    println!("doubled and rebuilt: {doubled:?}");
}
