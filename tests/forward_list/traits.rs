use rust_stl::collections::forward_list::ForwardList;
use std::fmt::Debug;

#[test]
fn debug() {
    let list: ForwardList<i32> = (0..4).collect();
    assert_eq!(format!("{:?}", list), "[0, 1, 2, 3]");
    let empty: ForwardList<i32> = ForwardList::new();
    assert_eq!(format!("{:?}", empty), "[]");
}

#[test]
fn clone_independent() {
    let original: ForwardList<i32> = (0..3).collect();
    let mut cloned = original.clone();
    assert_eq!(cloned, original);
    *cloned.front_mut().unwrap() = 99;
    assert_ne!(cloned, original);
}

#[test]
fn partial_eq() {
    let a: ForwardList<i32> = (0..3).collect();
    let b: ForwardList<i32> = (0..3).collect();
    assert_eq!(a, b);
    let c: ForwardList<i32> = (0..4).collect();
    assert_ne!(a, c);
}

#[test]
fn extend_from_iterator() {
    let mut list: ForwardList<i32> = ForwardList::new();
    list.extend(0..3);
    list.extend([10, 11]);
    assert_eq!(
        list.iter().copied().collect::<Vec<_>>(),
        vec![0, 1, 2, 10, 11]
    );
    let list2: ForwardList<i32> = (0..3).collect();
    assert_eq!(list2.front(), Some(&0));
}

#[test]
fn index_traits() {
    let mut list: ForwardList<i32> = (0..5).collect();
    assert_eq!(list[0], 0);
    assert_eq!(list[2], 2);
    list[1] = 9;
    assert_eq!(list[1], 9);
}

#[test]
fn into_iterator_selectable() {
    let mut list: ForwardList<i32> = (0..3).collect();
    fn takes_ref<'a>(v: impl IntoIterator<Item = &'a i32>) -> usize {
        v.into_iter().count()
    }
    assert_eq!(takes_ref(&list), 3);
    fn takes_mut<'a>(v: impl IntoIterator<Item = &'a mut i32>) -> i32 {
        v.into_iter()
            .map(|x| {
                *x += 1;
                *x
            })
            .sum()
    }
    assert_eq!(takes_mut(&mut list), 1 + 2 + 3);
    fn takes_owned(v: impl IntoIterator<Item = i32>) -> i32 {
        v.into_iter().sum()
    }
    assert_eq!(takes_owned(list), 6);
}

#[test]
fn generic_debug() {
    fn describe<T: Debug>(v: &ForwardList<T>) -> String {
        format!("{:?}", v)
    }
    let list: ForwardList<i32> = (0..3).collect();
    assert_eq!(describe(&list), "[0, 1, 2]");
}
