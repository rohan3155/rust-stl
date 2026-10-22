struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self { value, next: None }
    }
}

pub struct ForwardList<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> ForwardList<T> {
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    pub fn push_front(&mut self, value: T) {
        let mut node = Box::new(Node::new(value));
        node.next = self.head.take();
        self.head = Some(node);
        self.len += 1;
    }
    pub fn pop_front(&mut self) -> Option<T> {
        let mut node = self.head.take()?;
        self.head = node.next.take();
        self.len -= 1;
        Some(node.value)
    }
    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.value)
    }
    pub fn push_back(&mut self, value: T) {
        let mut current = &mut self.head;

        while let Some(node) = current {
            current = &mut node.next;
        }

        *current = Some(Box::new(Node::new(value)));
        self.len += 1;
    }
    pub fn pop_back(&mut self) -> Option<T> {
        if self.head.is_none() {
            return None;
        }

        if self.head.as_ref().unwrap().next.is_none() {
            self.len = 0;
            return self.head.take().map(|node| node.value);
        }

        let mut current = self.head.as_mut().unwrap();

        while current.next.as_ref().unwrap().next.is_some() {
            current = current.next.as_mut().unwrap();
        }

        let last = current.next.take().unwrap();
        self.len -= 1;

        Some(last.value)
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
    pub fn clear(&mut self) {
        self.head = None;
        self.len = 0;
    }
    pub fn insert(&mut self, index: usize, value: T) {
        if index > self.len {
            panic!("index out of bounds");
        }

        if index == 0 {
            self.push_front(value);
            return;
        }

        if index == self.len {
            self.push_back(value);
            return;
        }

        let mut current = self.head.as_mut().unwrap();

        for _ in 0..index - 1 {
            current = current.next.as_mut().unwrap();
        }

        let mut new_node = Box::new(Node::new(value));
        new_node.next = current.next.take();
        current.next = Some(new_node);
        self.len += 1;
    }
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        if index == 0 {
            return self.pop_front();
        }

        if index == self.len - 1 {
            return self.pop_back();
        }

        let mut current = self.head.as_mut()?;

        for _ in 0..index - 1 {
            current = current.next.as_mut()?;
        }

        let mut removed = current.next.take()?;
        current.next = removed.next.take();
        self.len -= 1;

        Some(removed.value)
    }
    pub fn remove_after(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        if index == 0 {
            return self.pop_front();
        }

        if index == self.len - 1 {
            return self.pop_back();
        }

        let mut current = self.head.as_mut()?;

        for _ in 0..index - 1 {
            current = current.next.as_mut()?;
        }

        let mut removed = current.next.take()?;
        current.next = removed.next.take();
        self.len -= 1;

        Some(removed.value)
    }
    pub fn remove_first(&mut self) -> Option<T> {
        self.pop_front()
    }
    pub fn remove_last(&mut self) -> Option<T> {
        self.pop_back()
    }
    pub fn remove_all(&mut self) {
        self.clear();
    }
    pub fn reverse(&mut self) {
        let mut prev = None;
        let mut current = self.head.take();

        while let Some(mut node) = current {
            let next = node.next.take();
            node.next = prev;
            prev = Some(node);
            current = next;
        }

        self.head = prev;
    }
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
        let mut current = &mut self.head;

        loop {
            let keep = match current.as_mut() {
                Some(node) => predicate(&node.value),
                None => {
                    break;
                }
            };

            if keep {
                current = &mut current.as_mut().unwrap().next;
            } else {
                let mut removed = current.take().unwrap();
                *current = removed.next.take();
                self.len -= 1;
            }
        }
    }

    /// Returns a cursor to the sentinel position before the first element.
    ///
    /// This mirrors `std::forward_list::before_begin`: inserting after this
    /// cursor places the new element at the front of the list.
    pub fn before_begin(&self) -> Cursor {
        Cursor { pos: 0 }
    }

    /// Inserts `value` after the element referenced by `pos`.
    ///
    /// `pos` may be the sentinel returned by [`before_begin`](Self::before_begin),
    /// in which case the value becomes the new first element. Returns a cursor
    /// to the newly inserted element.
    pub fn insert_after(&mut self, pos: Cursor, value: T) -> Cursor {
        if pos.pos == 0 {
            self.push_front(value);
            return Cursor { pos: 1 };
        }

        if pos.pos >= self.len {
            self.push_back(value);
            return Cursor { pos: self.len };
        }

        let mut current = self.head.as_mut().unwrap();
        for _ in 0..pos.pos - 1 {
            current = current.next.as_mut().unwrap();
        }

        let mut new_node = Box::new(Node::new(value));
        new_node.next = current.next.take();
        current.next = Some(new_node);
        self.len += 1;

        Cursor { pos: pos.pos + 1 }
    }

    /// Removes the element after the one referenced by `pos`.
    ///
    /// Passing the sentinel from [`before_begin`](Self::before_begin) removes the
    /// first element. Returns the removed value, or `None` if there is no
    /// element after `pos`.
    pub fn erase_after(&mut self, pos: Cursor) -> Option<T> {
        if pos.pos == 0 {
            return self.pop_front();
        }

        if pos.pos >= self.len {
            return None;
        }

        let mut current = self.head.as_mut()?;
        for _ in 0..pos.pos - 1 {
            current = current.next.as_mut()?;
        }

        let mut removed = current.next.take()?;
        current.next = removed.next.take();
        self.len -= 1;

        Some(removed.value)
    }

    /// Moves every element of `other` so that they follow the element referenced
    /// by `pos` in `self`.
    pub fn splice_after(&mut self, pos: Cursor, other: &mut Self) {
        if other.is_empty() {
            return;
        }

        if pos.pos == 0 {
            if self.is_empty() {
                self.head = other.head.take();
            } else {
                let mut tail = other.head.as_mut().unwrap();
                while tail.next.is_some() {
                    tail = tail.next.as_mut().unwrap();
                }
                tail.next = self.head.take();
                self.head = other.head.take();
            }
        } else {
            if pos.pos >= self.len {
                // Append at the end.
                self.append(other);
                return;
            }

            let mut current = self.head.as_mut().unwrap();
            for _ in 0..pos.pos - 1 {
                current = current.next.as_mut().unwrap();
            }

            let mut tail = other.head.as_mut().unwrap();
            while tail.next.is_some() {
                tail = tail.next.as_mut().unwrap();
            }
            tail.next = current.next.take();
            current.next = other.head.take();
        }

        self.len += other.len;
        other.len = 0;
    }

    /// Moves every element of `other` to the back of `self`.
    pub fn append(&mut self, other: &mut Self) {
        if other.is_empty() {
            return;
        }

        if self.is_empty() {
            self.head = other.head.take();
        } else {
            let mut tail = self.head.as_mut().unwrap();
            while tail.next.is_some() {
                tail = tail.next.as_mut().unwrap();
            }
            tail.next = other.head.take();
        }

        self.len += other.len;
        other.len = 0;
    }

    /// Shrinks the list to `new_len` elements.
    fn shrink(&mut self, new_len: usize) {
        if new_len == 0 {
            self.clear();
            return;
        }

        let mut current = self.head.as_mut().unwrap();

        for _ in 1..new_len {
            current = current.next.as_mut().unwrap();
        }

        current.next = None;
        self.len = new_len;
    }

    /// Grows the list to `new_len` elements by cloning `value`.
    fn grow(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        for _ in self.len..new_len {
            self.push_back(value.clone());
        }
    }

    /// Resizes the list to `new_len` elements, filling new slots with clones of
    /// `value` when growing.
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        if new_len < self.len {
            self.shrink(new_len);
        } else if new_len > self.len {
            self.grow(new_len, value);
        }
    }
}

/// A cursor into a [`ForwardList`].
///
/// Position `0` is the sentinel before the first element; position `n`
/// refers to the `n`-th element (1-based). The cursor is a lightweight,
/// copyable handle rather than a borrow of the list.
#[derive(Clone, Copy)]
pub struct Cursor {
    pos: usize,
}

impl<T> Default for ForwardList<T> {
    fn default() -> Self {
        Self::new()
    }
}
