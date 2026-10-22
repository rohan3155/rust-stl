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

    pub fn push_front(&mut self, value: T) {
        self.data.push_front(value);
    }
    pub fn push_back(&mut self, value: T) {
        self.data.push_back(value);
    }
}
