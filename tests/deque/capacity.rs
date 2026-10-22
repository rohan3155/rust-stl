use rust_stl::collections::deque::Deque;

#[test]
fn push_back_growth_preserves() {
    let mut d: Deque<u32> = Deque::new();
    for i in 0..1000 {
        d.push_back(i);
    }
    for i in 0..1000 {
        assert_eq!(d.pop_front(), Some(i));
    }
    assert!(d.is_empty());
}

#[test]
fn push_front_growth_preserves() {
    let mut d: Deque<u32> = Deque::new();
    for i in 0..1000 {
        d.push_front(i);
    }
    for i in (0..1000).rev() {
        assert_eq!(d.pop_front(), Some(i));
    }
    assert!(d.is_empty());
}

#[test]
fn alternating_ends_growth() {
    let mut d: Deque<i32> = Deque::new();
    for i in 0..500 {
        d.push_back(i);
        d.push_front(-i - 1);
    }
    assert_eq!(d.len(), 1000);
    // front should be -500
    assert_eq!(d.front(), Some(&-500));
}

#[test]
fn capacity_never_below_len() {
    let mut d: Deque<i32> = Deque::new();
    for i in 0..200 {
        d.push_back(i);
        assert!(d.capacity() >= d.len());
        if i % 3 == 0 {
            d.push_front(i);
            assert!(d.capacity() >= d.len());
        }
    }
}

#[test]
fn with_capacity() {
    let mut d: Deque<i32> = Deque::with_capacity(16);
    assert!(d.capacity() >= 16);
    d.extend(0..16);
    assert_eq!(d.len(), 16);
}

#[test]
fn wraparound_capacity_reuse() {
    let mut d: Deque<i32> = Deque::with_capacity(8);
    for i in 0..8 {
        d.push_back(i);
    }
    for _ in 0..4 {
        d.pop_front();
    }
    for i in 8..12 {
        d.push_back(i);
    }
    let v: Vec<i32> = d.iter().copied().collect();
    assert_eq!(v, vec![4, 5, 6, 7, 8, 9, 10, 11]);
}
