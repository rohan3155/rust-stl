use rust_stl::collections::forward_list::ForwardList;

fn main() {
    let mut list: ForwardList<i32> = (0..5).collect();

    for value in &list {
        print!("{value} ");
    }
    println!();

    for value in &mut list {
        *value *= 10;
    }
    println!("{:?}", list.iter().copied().collect::<Vec<i32>>());

    let owned: Vec<i32> = list.into_iter().collect();
    println!("{owned:?}");
}
