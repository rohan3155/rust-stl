use crate::collections::vector::Vector;
use std::fmt::Debug;
pub struct Stack<T> {
    data: Vector<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            data: Vector::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn top(&self) -> Option<&T> {
        self.data.last()
    }

    pub fn top_mut(&mut self) -> Option<&mut T> {
        self.data.last_mut()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vector::with_capacity(capacity),
        }
    }
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity()
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

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for Stack<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.data.iter()).finish()
    }
}

impl<T> Clone for Stack<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}
