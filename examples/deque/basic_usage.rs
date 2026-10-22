use rust_stl::collections::deque::Deque;

fn main() {
    let mut deque: Deque<i32> = Deque::new();
    deque.push_back(2);
    deque.push_front(1);
    deque.push_back(3);
    deque.push_front(0);

    println!("front: {:?}", deque.front());
    println!("back: {:?}", deque.back());
    println!("contents: {:?}", deque.as_slice());

    while let Some(value) = deque.pop_front() {
        println!("popped front: {value}");
    }

    // Reuse after draining.
    deque.push_back(42);
    println!("after reuse: {:?}", deque.pop_back());
}
