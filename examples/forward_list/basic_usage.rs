use rust_stl::collections::forward_list::ForwardList;

fn main() {
    let mut list: ForwardList<i32> = ForwardList::new();
    list.push_front(3);
    list.push_front(2);
    list.push_front(1);

    println!("front: {:?}", list.front());

    while let Some(value) = list.pop_front() {
        println!("popped: {value}");
    }

    let mut list: ForwardList<i32> = (0..4).collect();
    list.insert(2, 9);
    list.remove_first(&9);
    list.retain(|x| x % 2 == 0);
    list.reverse();
    println!("{:?}", list.iter().copied().collect::<Vec<i32>>());

    let mut other: ForwardList<i32> = (10..12).collect();
    let cur = list.before_begin();
    list.splice_after(cur, &mut other);
    println!("{:?}", list.iter().copied().collect::<Vec<i32>>());

    list.resize(6, 7);
    println!("{:?}", list.iter().copied().collect::<Vec<i32>>());
}
