//! A tour of the core `Stack` API.
//!
//! Run with:
//!
//! ```text
//! cargo run --example stack_basic_usage
//! ```

use rust_stl::collections::stack::Stack;

fn main() {
    // --- Construction -------------------------------------------------------
    //
    // `new` does not allocate; the stack starts empty.
    let mut stack: Stack<i32> = Stack::new();
    println!("fresh: len={} is_empty={}", stack.len(), stack.is_empty());

    // --- Pushing ------------------------------------------------------------
    stack.push(10);
    stack.push(20);
    stack.push(30);
    println!("after pushes: {stack:?} (top={:?})", stack.top());

    // --- Inspecting the top -------------------------------------------------
    //
    // `top` borrows the most recent element. `top_mut` borrows it mutably, so it
    // can be updated in place without popping and re-pushing.
    if let Some(top) = stack.top_mut() {
        *top += 1;
    }
    println!("after top_mut: {stack:?}");

    // --- Popping (LIFO) -----------------------------------------------------
    //
    // Elements come back out in the reverse of the order they went in.
    while let Some(value) = stack.pop() {
        println!("popped {value}");
    }
    assert!(stack.is_empty());

    // --- Empty-stack behaviour ---------------------------------------------
    //
    // Reading or popping an empty stack yields `None`; it never panics.
    println!("pop on empty: {:?}", stack.pop());
    println!("top on empty: {:?}", stack.top());

    // --- Building from an iterator -----------------------------------------
    //
    // `collect` pushes in order, so the last item ends up on top.
    let mut stack: Stack<String> = ["a", "b", "c"].into_iter().map(String::from).collect();
    println!("collected: {stack:?}");

    // --- Capacity and clearing ---------------------------------------------
    //
    // `clear` drops the elements but keeps the allocation for reuse.
    let capacity = stack.capacity();
    stack.clear();
    assert!(stack.is_empty());
    println!("after clear: {stack:?} (capacity retained: {capacity})");
}
