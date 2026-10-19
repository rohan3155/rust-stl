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
    pub fn peek(&self) -> Option<&T> {
        self.data.get(0)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.data.get_mut(0)
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Queue { data: Vector::with_capacity(capacity) }
    }
    pub fn front(&self) -> Option<&T> {
        self.data.get(0)
    }
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.data.get_mut(0)
    }
    pub fn back(&self) -> Option<&T> {
        self.data.get(self.len() - 1)
    }
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.data.get_mut(self.len() - 1)
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
    }
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
}
