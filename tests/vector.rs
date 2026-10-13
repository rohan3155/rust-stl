use rust_stl::collections::vector::VecExt;

#[test]
fn new_vector_is_empty() {
    let vec: Vec<i32> = Vec::new();

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 0);
}
