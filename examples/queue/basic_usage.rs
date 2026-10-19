use rust_stl::collections::queue::Queue;

fn main() {
    let mut queue: Queue<i32> = Queue::new();
    queue.enqueue(1);
    queue.enqueue(2);
    queue.enqueue(3);

    println!("front: {:?}", queue.front());
    println!("back: {:?}", queue.back());

    while let Some(value) = queue.dequeue() {
        println!("dequeued: {value}");
    }

    // Reuse after draining.
    queue.enqueue(42);
    println!("after reuse: {:?}", queue.dequeue());
}
