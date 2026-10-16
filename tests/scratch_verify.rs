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
