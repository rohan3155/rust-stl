use rust_stl::collections::deque::Deque;

#[test]
fn new_deque_is_empty() {
    let mut deque: Deque<i32> = Deque::new();
    assert_eq!(deque.len(), 0);
    assert!(deque.is_empty());
    assert_eq!(deque.front(), None);
    assert_eq!(deque.back(), None);
    assert_eq!(deque.capacity(), 0);
}

#[test]
fn push_back_and_pop_front_are_fifo() {
    let mut deque = Deque::new();
    deque.push_back(1);
    deque.push_back(2);
    deque.push_back(3);

    assert_eq!(deque.len(), 3);
    assert_eq!(deque.pop_front(), Some(1));
    assert_eq!(deque.pop_front(), Some(2));
    assert_eq!(deque.pop_front(), Some(3));
    assert_eq!(deque.pop_front(), None);
}

#[test]
fn push_front_and_pop_back_are_lifoish() {
    let mut deque = Deque::new();
    deque.push_front(1);
    deque.push_front(2);
    deque.push_front(3);

    assert_eq!(deque.pop_back(), Some(1));
    assert_eq!(deque.pop_back(), Some(2));
    assert_eq!(deque.pop_back(), Some(3));
    assert_eq!(deque.pop_back(), None);
}

#[test]
fn push_pop_both_ends() {
    let mut d = Deque::new();
    d.push_back(2);
    d.push_front(1);
    d.push_back(3);
    d.push_front(0);
    assert_eq!(d.as_slice(), &[0, 1, 2, 3]);
    assert_eq!(d.pop_front(), Some(0));
    assert_eq!(d.pop_back(), Some(3));
    assert_eq!(d.as_slice(), &[1, 2]);
}

#[test]
fn front_back_mut() {
    let mut d: Deque<i32> = (0..3).collect();
    *d.front_mut().unwrap() = 10;
    *d.back_mut().unwrap() = 30;
    assert_eq!(d.as_slice(), &[10, 1, 30]);
}

#[test]
fn get_bounds() {
    let mut d: Deque<i32> = (0..3).collect();
    assert_eq!(d.get(0), Some(&0));
    assert_eq!(d.get(2), Some(&2));
    assert_eq!(d.get(3), None);
    *d.get_mut(1).unwrap() = 5;
    assert_eq!(d.get(1), Some(&5));
}

#[test]
fn clear_reuse() {
    let mut d: Deque<i32> = (0..5).collect();
    d.clear();
    assert!(d.is_empty());
    d.push_back(1);
    d.push_front(0);
    assert_eq!(d.as_slice(), &[0, 1]);
}

#[test]
fn truncate_works() {
    let mut d: Deque<i32> = (0..5).collect();
    d.truncate(2);
    assert_eq!(d.as_slice(), &[0, 1]);
    d.truncate(10);
    assert_eq!(d.as_slice(), &[0, 1]);
}

#[test]
fn wraparound_pattern() {
    let mut d: Deque<i32> = Deque::new();
    for i in 0..5 {
        d.push_back(i);
    }
    for _ in 0..3 {
        assert!(d.pop_front().is_some());
    }
    for i in 5..8 {
        d.push_back(i);
    }
    let collected: Vec<i32> = d.iter().copied().collect();
    assert_eq!(collected, vec![3, 4, 5, 6, 7]);
}
