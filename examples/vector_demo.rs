use rust_stl::collections::vector::VecExt;

fn main() {
    // let mut numbers = vec![10, 20, 30, 20];

    // let index = numbers.find_index(|n| *n > 15);
    // let count = numbers.count_matches(|n| *n == 20);
    // let removed = numbers.remove_first(&20);
    // let indices = numbers.indices_of(|n| *n > 15);

    // println!("{index:?} {count} {removed} {indices:?}");
    let numbers = vec![10, 20, 30, 20];

    let indices: Vec<usize> = numbers
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if *item > 15 { Some(index) } else { None }
        })
        .collect();

    println!("{indices:?}");
}
