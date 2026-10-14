use rust_stl::collections::vector::MyVec;

#[test]
fn new_vector_is_empty() {
    let vec: MyVec<i32> = MyVec::new();

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 0);
}
