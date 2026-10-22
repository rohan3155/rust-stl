use std::collections::LinkedList;
use std::hint::black_box;
use std::time::{Duration, Instant};

use rust_stl::collections::forward_list::ForwardList;

const SIZES: [usize; 3] = [100, 1_000, 10_000];

fn best_of(work: impl Fn() -> Duration) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..5 {
        let elapsed = work();
        if elapsed < best {
            best = elapsed;
        }
    }
    best
}

fn bench_push_front(n: usize) {
    let elapsed = best_of(|| {
        let mut list: ForwardList<u32> = ForwardList::new();
        let start = Instant::now();
        for i in 0..n as u32 {
            list.push_front(i);
        }
        let e = start.elapsed();
        black_box(list.len());
        e
    });
    println!("push_front n={n}: {elapsed:?}");
}

fn main() {
    println!("rust-stl ForwardList benchmark");
    for &n in &SIZES {
        bench_push_front(n);
        let _ = black_box(LinkedList::<u32>::new());
    }
}
