use crate::collections::vector::Vector;
use crate::collections::vector::VectorIntoIter;
use crate::collections::vector::VectorIter;
use crate::collections::vector::VectorIterMut;
use std::fmt::Debug;
use std::iter::Extend;
use std::iter::FromIterator;


pub struct Queue<T> {
    data: Vector<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { data: Vector::new() }
    }

}