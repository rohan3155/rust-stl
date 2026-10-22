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
    pub fn insert_after(&mut self, index: usize, value: T) {
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
        let mut current = self.head.take();
        let mut next = None;

        while let Some(mut node) = current {
            next = node.next.take();
            node.next = next.take();
            current = next;
        }

        self.head = current;
    }
    pub fn retain<F>(&mut self, mut predicate: F) where F: FnMut(&T) -> bool {
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

  }

impl<T> Default for ForwardList<T> {
    fn default() -> Self {
        Self::new()
    }
}
