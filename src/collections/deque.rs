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
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        Some(self.data.remove(self.head))
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.data.pop_back()
        }
    }
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.data.get(self.head)
        }
    }
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            self.data.get_mut(self.head)
        }
    }
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let len = self.data.len();
            self.data.get(len - 1)
        }
    }
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            let len = self.data.len();
            self.data.get_mut(len - 1)
        }
    }
}
