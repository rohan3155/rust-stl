use crate::collections::vector::Vector;
use crate::collections::vector::VectorIntoIter;
use std::fmt::Debug;
use std::iter::Extend;
use std::iter::FromIterator;
use std::mem::ManuallyDrop;

pub struct Deque<T> {
    data: Vector<T>,
    head: usize,
}

impl<T> Deque<T> {
    pub fn new() -> Self {
        Deque {
            data: Vector::new(),
            head: 0,
        }
    }
}
