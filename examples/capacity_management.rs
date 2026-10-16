//! Capacity management: reserving, shrinking and handling allocation failure.
//!
//! Run with:
//!
//! ```text
//! cargo run --example capacity_management
//! ```

use rust_stl::collections::vector::Vector;

fn main() {
    // --- Reserve before filling -------------------------------------------
    //
    // Reserving the final size up front avoids the intermediate reallocations
    // that repeated pushing would otherwise cause.
    let mut numbers: Vector<i32> = Vector::new();
    numbers.reserve(1_000);
    println!("after reserve(1000): capacity={}", numbers.capacity());

    for n in 0..1_000 {
        numbers.push(n);
    }
    println!(
        "after 1000 pushes:   len={} capacity={} (no reallocation)",
        numbers.len(),
        numbers.capacity()
    );

    // --- reserve vs. reserve_exact ----------------------------------------
    //
    // `reserve` may over-allocate to keep future pushes cheap;
    // `reserve_exact` asks for exactly the space needed when the final size is
    // already known.
    let mut exact: Vector<i32> = Vector::new();
    exact.reserve_exact(10);
    println!("reserve_exact(10) on empty: capacity={}", exact.capacity());

    // A request that already fits never reallocates, so `reserve(0)` is a
    // genuine no-op.
    let capacity_before = numbers.capacity();
    numbers.reserve(0);
    assert_eq!(numbers.capacity(), capacity_before);
    println!("reserve(0) left capacity unchanged: {capacity_before}");

    // --- Clear keeps capacity; shrink_to_fit releases it ------------------
    let mut reused: Vector<String> = Vector::new();
    reused.extend(["a", "b", "c"].map(String::from));
    let capacity = reused.capacity();

    reused.clear();
    println!(
        "after clear: len={} capacity={} (retained)",
        reused.len(),
        reused.capacity()
    );
    assert_eq!(reused.capacity(), capacity);

    reused.shrink_to_fit();
    println!("after shrink_to_fit: capacity={}", reused.capacity());

    // Shrinking an empty vector releases the allocation entirely — and leaves
    // the vector perfectly usable.
    reused.push("back".to_string());
    println!("still usable after shrinking to zero: {reused:?}");

    // Shrinking a non-empty vector keeps every element.
    numbers.truncate(5);
    numbers.shrink_to_fit();
    println!(
        "truncate(5) + shrink_to_fit: {numbers:?} capacity={}",
        numbers.capacity()
    );

    // --- Fallible reservation ---------------------------------------------
    //
    // `try_reserve` returns an error instead of panicking, which matters when
    // the requested size comes from untrusted input. On failure the vector is
    // left completely untouched.
    let mut guarded: Vector<u32> = Vector::new();
    guarded.push(1);

    match guarded.try_reserve_exact(usize::MAX) {
        Ok(()) => println!("reserved an impossible amount of memory?!"),
        Err(message) => println!("try_reserve_exact(usize::MAX) failed: {message}"),
    }

    // The contents and length are exactly as they were.
    assert_eq!(guarded.as_slice(), &[1]);
    println!("the vector survived the failed reservation: {guarded:?}");
}
