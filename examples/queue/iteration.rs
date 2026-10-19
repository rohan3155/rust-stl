use rust_stl::collections::queue::Queue;

fn main() {
    let mut queue: Queue<i32> = (0..5).collect();

    // Inspect without consuming.
    let values: Vec<i32> = queue.iter().copied().collect();
    println!("remaining: {values:?}");

    // Mutate in place.
    for value in &mut queue {
        *value *= 10;
    }
    println!("after mutation: {:?}", queue.as_slice());

    // Consume.
    let owned: Vec<i32> = queue.into_iter().collect();
    println!("owned: {owned:?}");
}
