use rust_stl::collections::vector::MyVec;
use std::sync::atomic::{AtomicUsize, Ordering};

static DROPS: AtomicUsize = AtomicUsize::new(0);

struct Zst;

impl Drop for Zst {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

struct Noisy(#[allow(dead_code)] i32);

impl Drop for Noisy {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn zst_push_iter_drop() {
    DROPS.store(0, Ordering::SeqCst);
    let mut v: MyVec<Zst> = MyVec::new();
    for _ in 0..1000 {
        v.push(Zst);
    }
    assert_eq!(v.len(), 1000);
    assert_eq!(v.as_slice().len(), 1000);
    assert!(!v.as_slice().is_empty());
    assert_eq!(v.iter().count(), 1000);
    assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    drop(v);
    assert_eq!(DROPS.load(Ordering::SeqCst), 1000);
}

#[test]
fn unit_zst_no_alloc() {
    let mut v: MyVec<()> = MyVec::new();
    v.push(());
    v.push(());
    assert_eq!(v.len(), 2);
    assert_eq!(v.capacity(), usize::MAX);
    v.clear();
    assert_eq!(v.len(), 0);
    v.shrink_to_fit();
}

#[test]
fn empty_shrink_to_fit_is_safe() {
    let mut v: MyVec<i32> = MyVec::new();
    v.reserve(16);
    assert!(v.capacity() >= 16);
    v.shrink_to_fit();
    assert_eq!(v.len(), 0);
    assert_eq!(v.capacity(), 0);
    assert!(v.as_slice().is_empty());
    v.push(1);
    assert_eq!(v[0], 1);
}

#[test]
fn append_does_not_double_drop() {
    DROPS.store(0, Ordering::SeqCst);
    let mut a: MyVec<Noisy> = MyVec::new();
    let mut b: MyVec<Noisy> = MyVec::new();
    a.push(Noisy(1));
    a.push(Noisy(2));
    b.push(Noisy(3));
    a.append(&mut b);
    assert_eq!(a.len(), 3);
    assert_eq!(b.len(), 0);
    drop(a);
    assert_eq!(DROPS.load(Ordering::SeqCst), 3);
}

#[test]
fn zst_reverse_and_insert_remove() {
    let mut v: MyVec<u32> = MyVec::new();
    v.reverse();
    v.insert(0, 7);
    assert_eq!(v.remove(0), 7);
    assert_eq!(v.len(), 0);
}

#[test]
fn split_off_boundaries() {
    let mut v: MyVec<i32> = MyVec::new();
    for i in 0..5 {
        v.push(i);
    }
    let tail = v.split_off(0);
    assert_eq!(tail.len(), 5);
    assert_eq!(v.len(), 0);
    let mut v2: MyVec<i32> = MyVec::new();
    for i in 0..5 {
        v2.push(i);
    }
    let empty = v2.split_off(5);
    assert_eq!(empty.len(), 0);
    assert_eq!(v2.len(), 5);
    let mid = v2.split_off(2);
    assert_eq!(mid.as_slice(), &[2, 3, 4]);
    assert_eq!(v2.as_slice(), &[0, 1]);
}

#[test]
fn try_reserve_overflow_is_error_not_panic() {
    let mut v: MyVec<u32> = MyVec::new();
    v.push(1);
    v.push(2);

    let err = v.try_reserve_exact(usize::MAX);
    assert!(err.is_err(), "expected capacity overflow error, got {err:?}");

    // The vector must be untouched after a failed reservation.
    assert_eq!(v.len(), 2);
    assert_eq!(v.as_slice(), &[1, 2]);

    let mut w: MyVec<u64> = MyVec::new();
    w.push(1);
    assert!(w.try_reserve(usize::MAX).is_err());
    assert_eq!(w.as_slice(), &[1]);
}

#[test]
fn reserve_growth_preserves_values() {
    let mut v: MyVec<String> = MyVec::new();
    for i in 0..100 {
        v.push(format!("item-{i}"));
    }
    v.reserve(1000);
    assert!(v.capacity() >= 1100);
    for i in 0..100 {
        assert_eq!(v[i], format!("item-{i}"));
    }
    v.reserve_exact(0);
    v.shrink_to_fit();
    assert_eq!(v.capacity(), 100);
    for i in 0..100 {
        assert_eq!(v[i], format!("item-{i}"));
    }
}

#[test]
fn into_iter_partial_drop_drops_each_remaining_once() {
    DROPS.store(0, Ordering::SeqCst);
    let mut v: MyVec<Noisy> = MyVec::new();
    for i in 0..5 {
        v.push(Noisy(i));
    }

    {
        let mut it = v.into_iter();
        let a = it.next().expect("first");
        let b = it.next().expect("second");
        assert_eq!((a.0, b.0), (0, 1));
        // `a` and `b` are live locals; nothing dropped yet.
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
        // `it` still owns elements 2,3,4 and must drop exactly those.
        drop(it);
        assert_eq!(DROPS.load(Ordering::SeqCst), 3);
        drop(a);
        drop(b);
        assert_eq!(DROPS.load(Ordering::SeqCst), 5);
    }
}

#[test]
fn into_iter_unconsumed_drop_drops_all_once() {
    DROPS.store(0, Ordering::SeqCst);
    let mut v: MyVec<Noisy> = MyVec::new();
    for i in 0..4 {
        v.push(Noisy(i));
    }
    let it = v.into_iter();
    drop(it);
    assert_eq!(DROPS.load(Ordering::SeqCst), 4);
}

#[test]
fn into_iter_fully_consumed_does_not_double_drop() {
    DROPS.store(0, Ordering::SeqCst);
    let mut v: MyVec<Noisy> = MyVec::new();
    for i in 0..4 {
        v.push(Noisy(i));
    }
    let mut it = v.into_iter();
    let mut seen = Vec::new();
    while let Some(n) = it.next() {
        // `n` is dropped at the end of this iteration, so the count rises as we
        // consume; the iterator itself must not add any further drops.
        seen.push(n.0);
    }
    assert_eq!(seen, vec![0, 1, 2, 3]);
    assert_eq!(DROPS.load(Ordering::SeqCst), 4);
    assert_eq!(it.size_hint(), (0, Some(0)));
    assert!(it.next().is_none());
    drop(it);
    assert_eq!(DROPS.load(Ordering::SeqCst), 4);
}

#[test]
fn into_iter_double_ended() {
    let v: MyVec<i32> = (0..6).collect();
    let mut it = v.into_iter();
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next_back(), Some(5));
    assert_eq!(it.next_back(), Some(4));
    assert_eq!(it.size_hint(), (3, Some(3)));
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next_back(), Some(3));
    assert_eq!(it.next_back(), None);
    assert_eq!(it.next(), None);
}

#[test]
fn into_iter_zst_with_drop() {
    DROPS.store(0, Ordering::SeqCst);
    let mut v: MyVec<Zst> = MyVec::new();
    for _ in 0..7 {
        v.push(Zst);
    }
    let mut it = v.into_iter();
    assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    // Yielding a ZST transfers it to the caller, which drops it here.
    it.next();
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
    // The remaining six are owned by the iterator and dropped by its `Drop`.
    drop(it);
    assert_eq!(DROPS.load(Ordering::SeqCst), 7);
}

#[test]
fn into_iter_empty_and_zst_fully_consumed() {
    let v: MyVec<i32> = MyVec::new();
    let mut it = v.into_iter();
    assert_eq!(it.size_hint(), (0, Some(0)));
    assert_eq!(it.next(), None);
    drop(it);

    let mut z: MyVec<()> = MyVec::new();
    z.push(());
    z.push(());
    assert_eq!(z.into_iter().count(), 2);
}
