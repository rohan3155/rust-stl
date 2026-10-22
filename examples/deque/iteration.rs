use rust_stl::collections::deque::Deque;

fn main() {
    let mut deque: Deque<i32> = (0..5).collect();

    for value in &deque {
        print!("{value} ");
    }
    println!();

    for value in &mut deque {
        *value *= 10;
    }
    println!("{:?}", deque.as_slice());

    let owned: Vec<i32> = deque.into_iter().collect();
    println!("{owned:?}");
}
