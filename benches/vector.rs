//! A small, dependency-free micro-benchmark for [`Vector`] against
//! [`std::vec::Vec`].
//!
//! Run with:
//!
//! ```text
//! cargo bench --bench vector
//! ```
//!
//! # What this is, and what it is not
//!
//! This is a *sanity* harness: it reports the best wall-clock time of a handful
//! of repetitions of a few representative operations, side by side with the
//! standard library. It is deliberately free of third-party benchmarking
//! dependencies so the crate stays dependency-free.
//!
//! It is **not** a rigorous benchmark. There is no statistical analysis, no
//! warm-up control beyond `REPEATS`, and no isolation from background load. Do
//! not read the numbers as guarantees, and do not put them in the README. They
//! exist to spot gross regressions while working on the implementation.

use std::hint::black_box;
use std::time::{Duration, Instant};

use rust_stl::collections::vector::Vector;

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
fn report(operation: &str, vector: Duration, std_vec: Duration) {
    let ratio = vector.as_secs_f64() / std_vec.as_secs_f64();
    println!("{operation:<26} vector {vector:>12.3?}   std::vec {std_vec:>12.3?}   x{ratio:.2}");
}

fn bench_push() {
    let vector = best_of(|| {
        let mut v: Vector<u32> = Vector::new();
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            v.push(i);
        }
        let elapsed = start.elapsed();
        black_box(v.len());
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

    report("push (u32)", vector, std_vec);
}

fn bench_push_with_reserve() {
    let vector = best_of(|| {
        let mut v: Vector<u32> = Vector::new();
        v.reserve(ELEMENTS);
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            v.push(i);
        }
        let elapsed = start.elapsed();
        black_box(v.len());
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

    report("push after reserve", vector, std_vec);
}

fn bench_iterate_sum() {
    let vector_data: Vector<u64> = (0..ELEMENTS as u64).collect();
    let std_vec_data: Vec<u64> = (0..ELEMENTS as u64).collect();

    let vector = best_of(|| {
        let start = Instant::now();
        let sum: u64 = vector_data.iter().copied().sum();
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

    report("iterate + sum (u64)", vector, std_vec);
}

fn bench_get() {
    let vector_data: Vector<u64> = (0..ELEMENTS as u64).collect();
    let std_vec_data: Vec<u64> = (0..ELEMENTS as u64).collect();

    let vector = best_of(|| {
        let start = Instant::now();
        let mut sum = 0u64;
        for i in 0..ELEMENTS {
            sum = sum.wrapping_add(*vector_data.get(i).expect("in range"));
        }
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    let std_vec = best_of(|| {
        let start = Instant::now();
        let mut sum = 0u64;
        for i in 0..ELEMENTS {
            sum = sum.wrapping_add(*std_vec_data.get(i).expect("in range"));
        }
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    report("sequential get (u64)", vector, std_vec);
}

fn bench_insert_front() {
    // Small enough that the quadratic shifting dominates without taking forever.
    const INSERTS: usize = 20_000;

    let vector = best_of(|| {
        let mut v: Vector<u32> = Vector::new();
        let start = Instant::now();
        for i in 0..INSERTS as u32 {
            v.insert(0, i);
        }
        let elapsed = start.elapsed();
        black_box(v.len());
        elapsed
    });

    let std_vec = best_of(|| {
        let mut v: Vec<u32> = Vec::new();
        let start = Instant::now();
        for i in 0..INSERTS as u32 {
            v.insert(0, i);
        }
        let elapsed = start.elapsed();
        black_box(v.len());
        elapsed
    });

    report("insert(0, ..) x20k", vector, std_vec);
}

fn bench_sort() {
    const SORTED: usize = 200_000;

    // The same pseudo-random (well, deterministic but shuffled) input each time.
    let make = || -> Vec<i32> {
        let mut values: Vec<i32> = (0..SORTED as i32).collect();
        // A cheap LCG keeps the ordering chaotic enough for a sort benchmark.
        let mut state = 0x1234_5678u32;
        for i in (1..values.len()).rev() {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let j = (state as usize) % (i + 1);
            values.swap(i, j);
        }
        values
    };

    let vector = best_of(|| {
        let mut v: Vector<i32> = make().into_iter().collect();
        let start = Instant::now();
        v.sort();
        let elapsed = start.elapsed();
        black_box(v.len());
        elapsed
    });

    let std_vec = best_of(|| {
        let mut v: Vec<i32> = make();
        let start = Instant::now();
        v.sort();
        let elapsed = start.elapsed();
        black_box(v.len());
        elapsed
    });

    report("sort (i32)", vector, std_vec);
}

fn bench_clone() {
    let vector_data: Vector<u32> = (0..ELEMENTS as u32).collect();
    let std_vec_data: Vec<u32> = (0..ELEMENTS as u32).collect();

    let vector = best_of(|| {
        let start = Instant::now();
        let cloned = vector_data.clone();
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

    report("clone (u32)", vector, std_vec);
}

fn main() {
    println!("rust-stl micro-benchmark: Vector<T> vs std::vec::Vec");
    println!("best of {REPEATS} repetitions; {ELEMENTS} elements unless stated");
    println!("(report-only: no statistical rigour, no guarantees)");
    println!();

    bench_push();
    bench_push_with_reserve();
    bench_iterate_sum();
    bench_get();
    bench_insert_front();
    bench_sort();
    bench_clone();
}
