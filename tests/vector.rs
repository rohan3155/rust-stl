use rust_stl::collections::vector::Vector;

#[test]
fn new_vector_is_empty() {
    let vec: Vector<i32> = Vector::new();

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 0);
}
