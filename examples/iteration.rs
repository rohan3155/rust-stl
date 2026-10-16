//! The three iteration forms: `&Vector<T>`, `&mut Vector<T>` and `Vector<T>`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example iteration
//! ```

use rust_stl::collections::vector::Vector;

fn main() {
    let mut words: Vector<String> = Vector::new();
    words.push("alpha".to_string());
    words.push("beta".to_string());
    words.push("gamma".to_string());

    // --- By shared reference: `&Vector<T>` ---------------------------------
    //
    // Yields `&T`. The vector stays usable afterwards.
    print!("by reference:");
    for word in &words {
        print!(" {}", word.to_uppercase());
    }
    println!();

    // Iterator adapters work here too.
    let lengths: Vector<usize> = words.iter().map(|word| word.len()).collect();
    println!("lengths: {lengths:?}");

    // --- By mutable reference: `&mut Vector<T>` ----------------------------
    //
    // Yields `&mut T`, so elements can be modified in place.
    for word in &mut words {
        word.push('!');
    }
    println!("after mutation: {words:?}");

    // --- By value: `Vector<T>` ---------------------------------------------
    //
    // Yields `T` by value and consumes the vector. This is the cheapest form
    // for non-`Copy` types because nothing is cloned.
    let joined = words.into_iter().collect::<Vec<String>>().join(", ");
    println!("after consuming iteration: {joined}");

    // --- Owned iterators are double-ended ----------------------------------
    let numbers: Vector<i32> = (1..=5).collect();
    let mut iter = numbers.into_iter();

    // Alternate taking from the front and the back. Elements that are never
    // taken are still freed correctly when the iterator is dropped.
    let mut front = Vec::new();
    let mut back = Vec::new();
    while let Some(value) = iter.next() {
        front.push(value);
        match iter.next_back() {
            Some(value) => back.push(value),
            None => break,
        }
    }
    println!("from the front: {front:?}");
    println!("from the back:  {back:?}");

    // --- Zero-sized element types ------------------------------------------
    //
    // A vector of `()` occupies no heap memory but still tracks its length.
    let mut markers: Vector<()> = Vector::new();
    markers.resize(4, ());
    println!(
        "four ZSTs: len={} capacity={} (usize::MAX means 'no allocation')",
        markers.len(),
        markers.capacity()
    );
}
