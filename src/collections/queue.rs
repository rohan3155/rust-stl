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
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn enqueue(&mut self, item: T) {
        self.data.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() { None } else { Some(self.data.remove(0)) }
    }
}
