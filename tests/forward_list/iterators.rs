use rust_stl::collections::forward_list::ForwardList;

#[test]
fn shared_iteration() {
    let list: ForwardList<i32> = (0..3).collect();
    let v: Vec<i32> = (&list).into_iter().copied().collect();
    assert_eq!(v, vec![0, 1, 2]);
    assert_eq!(list.len(), 3);
}

#[test]
fn mutable_iteration() {
    let mut list: ForwardList<i32> = (0..3).collect();
    for x in &mut list {
        *x *= 10;
    }
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0, 10, 20]);
}

#[test]
fn owned_iteration() {
    let list: ForwardList<i32> = (0..4).collect();
    let v: Vec<i32> = list.into_iter().collect();
    assert_eq!(v, vec![0, 1, 2, 3]);
}

#[test]
fn size_hint_exact() {
    let list: ForwardList<i32> = (0..5).collect();
    let mut it = list.iter();
    assert_eq!(it.size_hint(), (5, Some(5)));
    assert_eq!(it.len(), 5);
    it.next();
    assert_eq!(it.len(), 4);
}

#[test]
fn empty_iteration() {
    let list: ForwardList<i32> = ForwardList::new();
    assert_eq!(list.iter().count(), 0);
    assert_eq!(list.into_iter().count(), 0);
}
