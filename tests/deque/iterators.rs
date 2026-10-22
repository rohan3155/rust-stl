use rust_stl::collections::deque::Deque;

#[test]
fn owned_iteration() {
    let d: Deque<i32> = (0..5).collect();
    let v: Vec<i32> = d.into_iter().collect();
    assert_eq!(v, vec![0, 1, 2, 3, 4]);
}

#[test]
fn owned_rev() {
    let d: Deque<i32> = (0..5).collect();
    let v: Vec<i32> = d.into_iter().rev().collect();
    assert_eq!(v, vec![4, 3, 2, 1, 0]);
}

#[test]
fn shared_iteration() {
    let d: Deque<i32> = (0..3).collect();
    let v: Vec<i32> = (&d).into_iter().copied().collect();
    assert_eq!(v, vec![0, 1, 2]);
    assert_eq!(d.len(), 3);
}

#[test]
fn mutable_iteration() {
    let mut d: Deque<i32> = (0..3).collect();
    for x in &mut d {
        *x *= 10;
    }
    assert_eq!(d.as_slice(), &[0, 10, 20]);
}

#[test]
fn size_hint() {
    let mut d: Deque<i32> = (0..5).collect();
    d.pop_front();
    let it = d.iter();
    assert_eq!(it.size_hint(), (4, Some(4)));
    let it = d.into_iter();
    assert_eq!(it.size_hint(), (4, Some(4)));
}

#[test]
fn iteration_after_wraparound() {
    let mut d: Deque<i32> = Deque::new();
    for i in 0..5 {
        d.push_back(i);
    }
    for _ in 0..3 {
        d.pop_front();
    }
    for i in 5..8 {
        d.push_back(i);
    }
    let v: Vec<i32> = d.iter().copied().collect();
    assert_eq!(v, vec![3, 4, 5, 6, 7]);
    let v2: Vec<i32> = d.into_iter().collect();
    assert_eq!(v2, vec![3, 4, 5, 6, 7]);
}

#[test]
fn into_iterator_selectable() {
    let mut d: Deque<i32> = (0..3).collect();
    fn takes_ref<'a>(v: impl IntoIterator<Item = &'a i32>) -> usize {
        v.into_iter().count()
    }
    assert_eq!(takes_ref(&d), 3);
    fn takes_mut<'a>(v: impl IntoIterator<Item = &'a mut i32>) -> i32 {
        v.into_iter()
            .map(|x| {
                *x += 1;
                *x
            })
            .sum()
    }
    assert_eq!(takes_mut(&mut d), 1 + 2 + 3);
    fn takes_owned(v: impl IntoIterator<Item = i32>) -> i32 {
        v.into_iter().sum()
    }
    // d now [1,2,3]
    assert_eq!(takes_owned(d), 6);
}
