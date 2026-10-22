use rust_stl::collections::deque::Deque;

#[test]
fn empty_deque_reads() {
    let d: Deque<i32> = Deque::new();
    assert!(d.is_empty());
    assert_eq!(d.len(), 0);
    assert_eq!(d.capacity(), 0);
    assert_eq!(d.front(), None);
    assert_eq!(d.back(), None);
    assert_eq!(d.get(0), None);
    assert_eq!((&d).into_iter().count(), 0);
    assert_eq!(format!("{d:?}"), "[]");
}

#[test]
fn empty_deque_mutations() {
    let mut d: Deque<i32> = Deque::new();
    d.clear();
    d.reserve(0);
    d.shrink_to_fit();
    d.extend(std::iter::empty::<i32>());
    assert!(d.is_empty());
    assert_eq!(d.pop_front(), None);
    assert_eq!(d.pop_back(), None);
}

#[test]
fn single_element() {
    let mut d: Deque<i32> = Deque::new();
    d.push_back(42);
    assert_eq!(d.len(), 1);
    assert!(!d.is_empty());
    assert_eq!(d.front(), Some(&42));
    assert_eq!(d.back(), Some(&42));
    assert_eq!(d.pop_front(), Some(42));
    assert!(d.is_empty());
}

#[test]
fn zst() {
    let mut d: Deque<()> = Deque::new();
    for _ in 0..1000 {
        d.push_back(());
    }
    assert_eq!(d.len(), 1000);
    assert_eq!(d.capacity(), usize::MAX);
    assert_eq!(d.pop_front(), Some(()));
    d.clear();
    assert!(d.is_empty());
}

#[test]
fn nested() {
    let mut outer: Deque<Deque<i32>> = Deque::new();
    outer.push_back((0..2).collect());
    outer.push_back((2..4).collect());
    assert_eq!(outer.front().unwrap().front(), Some(&0));
    outer.front_mut().unwrap().push_back(9);
    assert_eq!(outer.front().unwrap().len(), 3);
    let dq = outer.pop_front().unwrap();
    assert_eq!(dq.as_slice(), &[0, 1, 9]);
}
