use rust_stl::collections::forward_list::ForwardList;

#[test]
fn empty_list_reads() {
    let mut list: ForwardList<i32> = ForwardList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert_eq!(list.front(), None);
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.pop_back(), None);
    assert_eq!(list.get(0), None);
    assert_eq!(format!("{:?}", list), "[]");
}

#[test]
fn single_element() {
    let mut list: ForwardList<i32> = ForwardList::new();
    list.push_front(42);
    assert_eq!(list.len(), 1);
    assert_eq!(list.front(), Some(&42));
    assert_eq!(list.pop_front(), Some(42));
    assert!(list.is_empty());
}

#[test]
fn non_copy_strings() {
    let mut list: ForwardList<String> = ForwardList::new();
    list.push_back("a".to_string());
    list.push_back("b".to_string());
    assert_eq!(list.pop_front(), Some("a".to_string()));
    assert_eq!(list.front(), Some(&"b".to_string()));
}

#[test]
fn invalid_indices() {
    let mut list: ForwardList<i32> = (0..3).collect();
    assert_eq!(list.remove(10), None);
    assert_eq!(list.remove_after(10), None);
    assert_eq!(list.get(10), None);
    assert!(list.get_mut(10).is_none());
}

#[test]
fn retain_all_none() {
    let mut all: ForwardList<i32> = (0..4).collect();
    all.retain(|_| true);
    assert_eq!(all.len(), 4);
    let mut none: ForwardList<i32> = (0..4).collect();
    none.retain(|_| false);
    assert!(none.is_empty());
}

#[test]
fn remove_all_none_match() {
    let mut list: ForwardList<i32> = (0..4).collect();
    list.remove_all(99);
    assert_eq!(list.len(), 4);
}
