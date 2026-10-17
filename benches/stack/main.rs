//! A small, dependency-free micro-benchmark for [`Stack`] against a
//! [`std::vec::Vec`] used as a stack.
//!
//! Run with:
//!
//! ```text
//! cargo bench --bench stack
//! ```
//!
//! # What this is, and what it is not
//!
//! This is a *sanity* harness: it reports the best wall-clock time of a handful
//! of repetitions of a few representative operations, side by side with the
//! standard library. It is deliberately free of third-party benchmarking
//! dependencies so the crate stays dependency-free.
//!
//! Rust has no stack type in the standard library, so the baseline is a
//! [`Vec`] driven as a stack (`push` / `pop` / `last`).
//!
//! It is **not** a rigorous benchmark. There is no statistical analysis, no
//! warm-up control beyond `REPEATS`, and no isolation from background load. Do
//! not read the numbers as guarantees, and do not put them in the README. They
//! exist to spot gross regressions while working on the implementation.

use std::hint::black_box;
use std::time::{Duration, Instant};

use rust_stl::collections::stack::Stack;

/// Number of operations per repetition for the linear work-loads.
const ELEMENTS: usize = 1_000_000;

/// Number of repetitions; the best of these is reported.
const REPEATS: u32 = 5;

/// Runs `work` `REPEATS` times and returns the fastest observation.
fn best_of(work: impl Fn() -> Duration) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..REPEATS {
        let elapsed = work();
        if elapsed < best {
            best = elapsed;
        }
    }
    best
}

/// Prints one comparison row.
fn report(operation: &str, stack: Duration, std_vec: Duration) {
    let ratio = stack.as_secs_f64() / std_vec.as_secs_f64();
    println!("{operation:<26} stack {stack:>12.3?}   std::vec {std_vec:>12.3?}   x{ratio:.2}");
}

fn bench_push() {
    let stack = best_of(|| {
        let mut s: Stack<u32> = Stack::new();
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            s.push(i);
        }
        let elapsed = start.elapsed();
        black_box(s.len());
        elapsed
    });

    let std_vec = best_of(|| {
        let mut v: Vec<u32> = Vec::new();
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            v.push(i);
        }
        let elapsed = start.elapsed();
        black_box(v.len());
        elapsed
    });

    report("push (u32)", stack, std_vec);
}

fn bench_push_with_capacity() {
    let stack = best_of(|| {
        let mut s: Stack<u32> = Stack::with_capacity(ELEMENTS);
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            s.push(i);
        }
        let elapsed = start.elapsed();
        black_box(s.len());
        elapsed
    });

    let std_vec = best_of(|| {
        let mut v: Vec<u32> = Vec::with_capacity(ELEMENTS);
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            v.push(i);
        }
        let elapsed = start.elapsed();
        black_box(v.len());
        elapsed
    });

    report("push after capacity", stack, std_vec);
}

fn bench_pop() {
    let stack = best_of(|| {
        let mut s: Stack<u32> = (0..ELEMENTS as u32).collect();
        let start = Instant::now();
        let mut sum = 0u64;
        while let Some(value) = s.pop() {
            sum = sum.wrapping_add(u64::from(value));
        }
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    let std_vec = best_of(|| {
        let mut v: Vec<u32> = (0..ELEMENTS as u32).collect();
        let start = Instant::now();
        let mut sum = 0u64;
        while let Some(value) = v.pop() {
            sum = sum.wrapping_add(u64::from(value));
        }
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    report("pop to empty (u32)", stack, std_vec);
}

fn bench_top() {
    let stack_data: Stack<u64> = (0..ELEMENTS as u64).collect();
    let std_vec_data: Vec<u64> = (0..ELEMENTS as u64).collect();

    let stack = best_of(|| {
        let start = Instant::now();
        let mut sum = 0u64;
        for _ in 0..ELEMENTS {
            if let Some(value) = black_box(stack_data.top()) {
                sum = sum.wrapping_add(*value);
            }
        }
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    let std_vec = best_of(|| {
        let start = Instant::now();
        let mut sum = 0u64;
        for _ in 0..ELEMENTS {
            if let Some(value) = black_box(std_vec_data.last()) {
                sum = sum.wrapping_add(*value);
            }
        }
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    report("top/peek (u64)", stack, std_vec);
}

fn bench_iterate_sum() {
    let stack_data: Stack<u64> = (0..ELEMENTS as u64).collect();
    let std_vec_data: Vec<u64> = (0..ELEMENTS as u64).collect();

    let stack = best_of(|| {
        let start = Instant::now();
        let sum: u64 = (&stack_data).into_iter().copied().sum();
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    let std_vec = best_of(|| {
        let start = Instant::now();
        let sum: u64 = std_vec_data.iter().copied().sum();
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    report("iterate + sum (u64)", stack, std_vec);
}

fn bench_clone() {
    let stack_data: Stack<u32> = (0..ELEMENTS as u32).collect();
    let std_vec_data: Vec<u32> = (0..ELEMENTS as u32).collect();

    let stack = best_of(|| {
        let start = Instant::now();
        let cloned = stack_data.clone();
        let elapsed = start.elapsed();
        black_box(cloned.len());
        elapsed
    });

    let std_vec = best_of(|| {
        let start = Instant::now();
        let cloned = std_vec_data.clone();
        let elapsed = start.elapsed();
        black_box(cloned.len());
        elapsed
    });

    report("clone (u32)", stack, std_vec);
}

fn main() {
    println!("rust-stl micro-benchmark: Stack<T> vs std::vec::Vec (as a stack)");
    println!("best of {REPEATS} repetitions; {ELEMENTS} elements unless stated");
    println!("(report-only: no statistical rigour, no guarantees)");
    println!();

    bench_push();
    bench_push_with_capacity();
    bench_pop();
    bench_top();
    bench_iterate_sum();
    bench_clone();
}
