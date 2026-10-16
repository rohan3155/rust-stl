use rust_stl::collections::vector::Vector;

fn main() {
    let mut numbers = Vector::new();

    numbers.push(10);
    numbers.push(20);
    numbers.push(30);
    numbers.push(20);

    println!("Length: {}", numbers.len());
    println!("Capacity: {}", numbers.capacity());
    println!("Is empty: {}", numbers.is_empty());

    println!("Element at index 1: {:?}", numbers.get(1));

    let index = numbers.find_index(|n| *n > 15);
    println!("First matching index: {:?}", index);

    let count = numbers.count_matches(|n| *n > 15);
    println!("Matching elements: {}", count);

    let indices = numbers.indices_of(|n| *n > 15);
    println!("Matching indices: {:?}", indices.as_slice());

    let removed = numbers.remove_first(&20);
    println!("Removed first 20: {}", removed);

    println!("After removal: {:?}", numbers.as_slice());

    numbers.clear();

    println!("After clear: {:?}", numbers.as_slice());
    println!("Is empty: {}", numbers.is_empty());
}