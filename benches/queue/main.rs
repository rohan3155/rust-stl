use std::collections::VecDeque;
use std::hint::black_box;
use std::time::{Duration, Instant};

use rust_stl::collections::queue::Queue;

const ELEMENTS: usize = 1_000_000;
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

fn report(operation: &str, queue: Duration, vecdeque: Duration) {
    let ratio = queue.as_secs_f64() / vecdeque.as_secs_f64();
    println!("{operation:<28} queue {queue:>12.3?}   VecDeque {vecdeque:>12.3?}   x{ratio:.2}");
}

fn bench_enqueue() {
    let queue = best_of(|| {
        let mut q: Queue<u32> = Queue::new();
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            q.enqueue(i);
        }
        let elapsed = start.elapsed();
        black_box(q.len());
        elapsed
    });

    let vecdeque = best_of(|| {
        let mut q: VecDeque<u32> = VecDeque::new();
        let start = Instant::now();
        for i in 0..ELEMENTS as u32 {
            q.push_back(i);
        }
        let elapsed = start.elapsed();
        black_box(q.len());
        elapsed
    });

    report("enqueue (u32)", queue, vecdeque);
}

fn bench_dequeue() {
    let queue = best_of(|| {
        let mut q: Queue<u32> = (0..ELEMENTS as u32).collect();
        let start = Instant::now();
        let mut sum = 0u64;
        while let Some(value) = q.dequeue() {
            sum = sum.wrapping_add(u64::from(value));
        }
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    let vecdeque = best_of(|| {
        let mut q: VecDeque<u32> = (0..ELEMENTS as u32).collect();
        let start = Instant::now();
        let mut sum = 0u64;
        while let Some(value) = q.pop_front() {
            sum = sum.wrapping_add(u64::from(value));
        }
        let elapsed = start.elapsed();
        black_box(sum);
        elapsed
    });

    report("dequeue to empty (u32)", queue, vecdeque);
}

fn main() {
    println!("rust-stl micro-benchmark: Queue<T> vs std::collections::VecDeque");
    println!("best of {REPEATS} repetitions; {ELEMENTS} elements unless stated");
    println!();

    bench_enqueue();
    bench_dequeue();
}
