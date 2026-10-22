use rust_stl::collections::deque::Deque;
use std::fmt::Debug;

#[test]
fn debug_formats_front_to_back() {
    let d: Deque<i32> = (0..5).collect();
    assert_eq!(format!("{d:?}"), "[0, 1, 2, 3, 4]");
    let empty: Deque<i32> = Deque::new();
    assert_eq!(format!("{empty:?}"), "[]");
}

#[test]
fn clone_independent() {
    let original: Deque<i32> = (0..3).collect();
    let mut cloned = original.clone();
    assert_eq!(cloned, original);
    *cloned.front_mut().unwrap() = 99;
    assert_ne!(cloned, original);
}

#[test]
fn partial_eq_ignores_capacity() {
    let a: Deque<i32> = (0..3).collect();
    let b: Deque<i32> = (0..3).collect();
    assert_eq!(a, b);
    let mut roomy: Deque<i32> = Deque::with_capacity(1000);
    roomy.extend(0..3);
    assert_eq!(a, roomy);
    let diff: Deque<i32> = (0..4).collect();
    assert_ne!(a, diff);
}

#[test]
fn extend_and_from_iterator() {
    let mut d: Deque<i32> = Deque::new();
    d.extend(0..3);
    d.extend([10, 11]);
    assert_eq!(d.as_slice(), &[0, 1, 2, 10, 11]);
    let d2: Deque<i32> = (0..3).collect();
    assert_eq!(d2.front(), Some(&0));
}

#[test]
fn index_traits() {
    let mut d: Deque<i32> = (0..5).collect();
    assert_eq!(d[0], 0);
    assert_eq!(d[2], 2);
    d[1] = 9;
    assert_eq!(d[1], 9);
    assert_eq!(&d[1..3], &[9, 2]);
}

#[test]
fn into_iterator_trait_selectable() {
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
    assert_eq!(takes_owned(d), 6);
}

#[test]
fn generic_debug() {
    fn describe<T: Debug>(v: &Deque<T>) -> String {
        format!("{v:?}")
    }
    let d: Deque<i32> = (0..3).collect();
    assert_eq!(describe(&d), "[0, 1, 2]");
}
