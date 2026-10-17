//! The three iteration forms over a `Stack`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example stack_iteration
//! ```

use rust_stl::collections::stack::Stack;

fn main() {
    // Iteration walks from the BOTTOM to the TOP of the stack, which is the
    // reverse of the pop order.
    let stack: Stack<i32> = (1..=5).collect();
    println!("stack (bottom -> top): {stack:?}");

    // --- By shared reference: `&Stack<T>` ----------------------------------
    //
    // Yields `&T` and leaves the stack usable afterwards.
    print!("by reference:");
    for value in &stack {
        print!(" {value}");
    }
    println!();

    // --- By mutable reference: `&mut Stack<T>` -----------------------------
    //
    // Yields `&mut T`, so every element can be updated in place.
    let mut mutable = stack.clone();
    for value in &mut mutable {
        *value *= 10;
    }
    println!("after in-place mutation: {mutable:?}");

    // --- By value: `Stack<T>` ----------------------------------------------
    //
    // Consumes the stack and yields owned values.
    let drained: Vec<i32> = stack.clone().into_iter().collect();
    println!("owned iteration: {drained:?}");

    // The owned iterator is double-ended, so `rev` visits elements in pop order.
    let pop_order: Vec<i32> = stack.into_iter().rev().collect();
    println!("reversed (pop order): {pop_order:?}");

    // --- Zero-sized element types ------------------------------------------
    //
    // A stack of `()` occupies no heap memory but still tracks its length.
    let mut markers: Stack<()> = Stack::new();
    for _ in 0..4 {
        markers.push(());
    }
    println!(
        "four ZSTs: len={} capacity={} (usize::MAX means 'no allocation')",
        markers.len(),
        markers.capacity()
    );
}
