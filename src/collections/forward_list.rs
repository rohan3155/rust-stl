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

    pub fn push_back(&mut self, value: T) {
        let mut current = &mut self.head;

        while let Some(node) = current {
            current = &mut node.next;
        }

        *current = Some(Box::new(Node::new(value)));
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}
