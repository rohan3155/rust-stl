//! Core first-in, first-out behaviour of the public `Queue` API.

use rust_stl::collections::queue::Queue;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    id: u32,
    label: String,
}

#[test]
fn new_queue_is_empty() {
    let mut queue: Queue<i32> = Queue::new();

    assert_eq!(queue.len(), 0);
    assert!(queue.is_empty());
    assert_eq!(queue.front(), None);
    assert_eq!(queue.front_mut(), None);
    assert_eq!(queue.back(), None);
    assert_eq!(queue.back_mut(), None);
    assert_eq!(queue.capacity(), 0);
}

#[test]
fn default_matches_new() {
    let default: Queue<i32> = Queue::default();
    let created: Queue<i32> = Queue::new();

    assert_eq!(default, created);
    assert!(default.is_empty());
    assert_eq!(default.capacity(), 0);
}

#[test]
fn dequeueing_an_empty_queue_keeps_returning_none() {
    let mut queue: Queue<i32> = Queue::new();

    for _ in 0..3 {
        assert_eq!(queue.dequeue(), None);
    }

    assert!(queue.is_empty());
    assert_eq!(queue.front(), None);
}

#[test]
fn enqueue_and_dequeue_are_fifo() {
    let mut queue = Queue::new();
    queue.enqueue(1);
    queue.enqueue(2);
    queue.enqueue(3);

    assert_eq!(queue.len(), 3);
    assert_eq!(queue.dequeue(), Some(1));
    assert_eq!(queue.dequeue(), Some(2));
    assert_eq!(queue.dequeue(), Some(3));
    assert_eq!(queue.dequeue(), None);
}

#[test]
fn repeated_enqueues_then_repeated_dequeues_across_growth() {
    let mut queue = Queue::new();

    for i in 0..1_000 {
        queue.enqueue(i);
    }
    assert_eq!(queue.len(), 1_000);

    for i in 0..1_000 {
        assert_eq!(queue.dequeue(), Some(i));
    }

    assert!(queue.is_empty());
    assert_eq!(queue.dequeue(), None);
}

#[test]
fn alternating_enqueue_and_dequeue_preserves_fifo() {
    let mut queue = Queue::new();

    queue.enqueue(1);
    queue.enqueue(2);
    assert_eq!(queue.dequeue(), Some(1));

    queue.enqueue(3);
    assert_eq!(queue.dequeue(), Some(2));
    assert_eq!(queue.dequeue(), Some(3));
    assert_eq!(queue.dequeue(), None);

    queue.enqueue(4);
    assert_eq!(queue.dequeue(), Some(4));
}

#[test]
fn front_observes_the_oldest_element() {
    let mut queue = Queue::new();
    assert_eq!(queue.front(), None);

    queue.enqueue(10);
    assert_eq!(queue.front(), Some(&10));
    assert_eq!(queue.back(), Some(&10));

    queue.enqueue(20);
    assert_eq!(queue.front(), Some(&10));
    assert_eq!(queue.back(), Some(&20));

    assert_eq!(queue.dequeue(), Some(10));
    assert_eq!(queue.front(), Some(&20));
}

#[test]
fn front_mut_updates_the_front_in_place() {
    let mut queue: Queue<String> = Queue::new();
    queue.enqueue("hello".to_string());

    queue.front_mut().unwrap().push_str(" world");

    assert_eq!(queue.front(), Some(&"hello world".to_string()));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.dequeue(), Some("hello world".to_string()));
}

#[test]
fn back_mut_updates_the_back_in_place() {
    let mut queue: Queue<i32> = (0..3).collect();
    *queue.back_mut().unwrap() = 30;
    assert_eq!(queue.back(), Some(&30));
    assert_eq!(queue.as_slice(), &[0, 1, 30]);
}

#[test]
fn clear_empties_and_the_queue_stays_reusable() {
    let mut queue: Queue<i32> = (0..5).collect();
    assert!(!queue.is_empty());

    queue.clear();

    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.front(), None);
    assert_eq!(queue.dequeue(), None);

    queue.enqueue(1);
    queue.enqueue(2);
    assert_eq!(queue.dequeue(), Some(1));
    assert_eq!(queue.dequeue(), Some(2));

    queue.clear();
    queue.clear();
    assert!(queue.is_empty());
}

#[test]
fn single_element_queue_supports_every_operation() {
    let mut queue: Queue<i32> = Queue::new();
    queue.enqueue(42);

    assert_eq!(queue.len(), 1);
    assert!(!queue.is_empty());
    assert_eq!(queue.front(), Some(&42));
    assert_eq!(queue.front_mut(), Some(&mut 42));
    assert_eq!(queue.back(), Some(&42));
    assert_eq!(queue.back_mut(), Some(&mut 42));
    assert_eq!(queue.get(0), Some(&42));
    assert_eq!(queue.peek(), Some(&42));

    *queue.front_mut().unwrap() = 43;
    assert_eq!(queue.front(), Some(&43));

    assert_eq!(queue.dequeue(), Some(43));
    assert!(queue.is_empty());
    assert_eq!(queue.front(), None);
    assert_eq!(queue.front_mut(), None);

    queue.clear();
    assert!(queue.is_empty());
}

#[test]
fn works_with_non_copy_custom_values() {
    let mut queue: Queue<Token> = Queue::new();
    queue.enqueue(Token {
        id: 1,
        label: "one".to_string(),
    });
    queue.enqueue(Token {
        id: 2,
        label: "two".to_string(),
    });

    assert_eq!(queue.front().map(|token| token.id), Some(1));
    assert_eq!(queue.back().map(|token| token.id), Some(2));

    let dequeued = queue.dequeue().expect("an element");
    assert_eq!(
        dequeued,
        Token {
            id: 1,
            label: "one".to_string(),
        }
    );
    assert_eq!(queue.front().map(|token| token.id), Some(2));
}

#[test]
fn strings_round_trip_through_the_queue() {
    let mut queue: Queue<String> = Queue::new();
    queue.enqueue("alpha".to_string());
    queue.enqueue("beta".to_string());

    let joined = format!(
        "{}-{}",
        queue.dequeue().expect("front"),
        queue.dequeue().expect("back")
    );
    assert_eq!(joined, "alpha-beta");
}

#[test]
fn get_and_get_mut_are_bounded_and_front_to_back() {
    let mut queue: Queue<i32> = (0..3).collect();

    assert_eq!(queue.get(0), Some(&0));
    assert_eq!(queue.get(2), Some(&2));
    assert_eq!(queue.get(3), None);
    assert_eq!(queue.get(usize::MAX), None);

    *queue.get_mut(0).unwrap() = 100;
    assert_eq!(queue.get(0), Some(&100));
    assert_eq!(queue.get_mut(3), None);
}

#[test]
fn append_moves_elements_in_fifo_order() {
    let mut head: Queue<i32> = (0..2).collect();
    let mut tail: Queue<i32> = (2..5).collect();
    head.append(&mut tail);
    assert_eq!(head.as_slice(), &[0, 1, 2, 3, 4]);
    assert!(tail.is_empty());

    assert_eq!(head.dequeue(), Some(0));
    assert_eq!(head.dequeue(), Some(1));
    assert_eq!(head.as_slice(), &[2, 3, 4]);
}
