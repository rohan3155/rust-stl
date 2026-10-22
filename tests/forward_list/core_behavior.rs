use rust_stl::collections::forward_list::ForwardList;

#[test]
fn new_list_is_empty() {
    let list: ForwardList<i32> = ForwardList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert_eq!(list.front(), None);
}

#[test]
fn push_pop_front() {
    let mut list = ForwardList::new();
    list.push_front(1);
    list.push_front(2);
    list.push_front(3);
    assert_eq!(list.pop_front(), Some(3));
    assert_eq!(list.pop_front(), Some(2));
    assert_eq!(list.pop_front(), Some(1));
    assert_eq!(list.pop_front(), None);
}

#[test]
fn push_back_pop_back() {
    let mut list = ForwardList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    assert_eq!(list.pop_back(), Some(3));
    assert_eq!(list.pop_back(), Some(2));
    assert_eq!(list.pop_back(), Some(1));
    assert_eq!(list.pop_back(), None);
}

#[test]
fn front_mut() {
    let mut list: ForwardList<i32> = (0..3).collect();
    *list.front_mut().unwrap() = 10;
    assert_eq!(list.front(), Some(&10));
}

#[test]
fn insert_and_remove_index() {
    let mut list: ForwardList<i32> = (0..3).collect();
    list.insert(1, 9);
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0, 9, 1, 2]);
    assert_eq!(list.remove(1), Some(9));
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0, 1, 2]);
}

#[test]
fn insert_after_cursor() {
    let mut list: ForwardList<i32> = (1..=3).collect();
    let cur = list.before_begin();
    let cur = list.insert_after(cur, 0);
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    list.insert_after(cur, 99);
    assert_eq!(
        list.iter().copied().collect::<Vec<_>>(),
        vec![0, 99, 1, 2, 3]
    );
}

#[test]
fn erase_after_cursor() {
    let mut list: ForwardList<i32> = (1..=4).collect();
    let cur = list.before_begin();
    assert_eq!(list.erase_after(cur), Some(1));
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
}

#[test]
fn remove_first_all() {
    let mut list: ForwardList<i32> = vec![1, 2, 1, 3, 1].into_iter().collect();
    assert!(list.remove_first(&1));
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![2, 1, 3, 1]);
    list.remove_all(1);
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![2, 3]);
}

#[test]
fn retain() {
    let mut list: ForwardList<i32> = (0..5).collect();
    list.retain(|x| x % 2 == 0);
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0, 2, 4]);
}

#[test]
fn reverse() {
    let mut list: ForwardList<i32> = (0..4).collect();
    list.reverse();
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![3, 2, 1, 0]);
    let mut empty: ForwardList<i32> = ForwardList::new();
    empty.reverse();
    assert!(empty.is_empty());
}

#[test]
fn splice_and_append() {
    let mut a: ForwardList<i32> = (1..=2).collect();
    let mut b: ForwardList<i32> = (3..=4).collect();
    let cur = a.before_begin();
    a.splice_after(cur, &mut b);
    assert_eq!(a.iter().copied().collect::<Vec<_>>(), vec![3, 4, 1, 2]);
    assert!(b.is_empty());

    let mut c: ForwardList<i32> = (5..=6).collect();
    a.append(&mut c);
    assert_eq!(a.len(), 6);
    assert!(c.is_empty());
}

#[test]
fn resize() {
    let mut list: ForwardList<i32> = (0..3).collect();
    list.resize(1, 0);
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0]);
    list.resize(3, 7);
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0, 7, 7]);
    list.resize(0, 0);
    assert!(list.is_empty());
}
