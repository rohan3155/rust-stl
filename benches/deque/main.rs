use std::collections::VecDeque;
use std::hint::black_box;
use std::time::{Duration, Instant};

use rust_stl::collections::deque::Deque;

const ELEMENTS: usize = 1_000_000;
const FRONT_ELEMENTS: usize = 10_000;
const REPEATS: u32 = 5;

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

fn report(operation: &str, deque: Duration, vecdeque: Duration) {
    let ratio = deque.as_secs_f64() / vecdeque.as_secs_f64();
    println!("{operation:<28} deque {deque:>12.3?}   VecDeque {vecdeque:>12.3?}   x{ratio:.2}");
}

fn bench_push_back() {
    let deque = best_of(|| {
        let mut d: Deque<u32> = Deque::new();
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            d.push_back(i);
        }
        let elapsed = start.elapsed();
        black_box(d.len());
        elapsed
    });

    let vecdeque = best_of(|| {
        let mut d: VecDeque<u32> = VecDeque::new();
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            d.push_back(i);
        }
        let elapsed = start.elapsed();
        black_box(d.len());
        elapsed
    });

    report("push_back (u32)", deque, vecdeque);
}

fn bench_push_front() {
    let deque = best_of(|| {
        let mut d: Deque<u32> = Deque::new();
        let start = Instant::now();
        for i in 0..FRONT_ELEMENTS as u32 {
            d.push_front(i);
        }
        let elapsed = start.elapsed();
        black_box(d.len());
        elapsed
    });

    let vecdeque = best_of(|| {
        let mut d: VecDeque<u32> = VecDeque::new();
        let start = Instant::now();
        for i in 0..FRONT_ELEMENTS as u32 {
            d.push_front(i);
        }
        let elapsed = start.elapsed();
        black_box(d.len());
        elapsed
    });

    report("push_front (u32)", deque, vecdeque);
}

fn bench_pop_front() {
    let deque = best_of(|| {
        let mut d: Deque<u32> = (0..ELEMENTS as u32).collect();
        let start = Instant::now();
        while black_box(d.pop_front()).is_some() {}
        let elapsed = start.elapsed();
        black_box(d.len());
        elapsed
    });

    let vecdeque = best_of(|| {
        let mut d: VecDeque<u32> = (0..ELEMENTS as u32).collect();
        let start = Instant::now();
        while black_box(d.pop_front()).is_some() {}
        let elapsed = start.elapsed();
        black_box(d.len());
        elapsed
    });

    report("pop_front to empty (u32)", deque, vecdeque);
}

fn main() {
    println!("rust-stl micro-benchmark: Deque<T> vs std::collections::VecDeque");
    bench_push_back();
    bench_push_front();
    bench_pop_front();
}
