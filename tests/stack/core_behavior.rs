//! Core last-in, first-out behaviour of the public `Stack` API.

use rust_stl::collections::stack::Stack;

/// A non-`Copy` element used to exercise ownership, not just `i32`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    id: u32,
    label: String,
}

#[test]
fn new_stack_is_empty() {
    let mut stack: Stack<i32> = Stack::new();

    assert_eq!(stack.len(), 0);
    assert!(stack.is_empty());
    assert_eq!(stack.top(), None);
    assert_eq!(stack.top_mut(), None);
    assert_eq!(stack.capacity(), 0);
}

#[test]
fn default_matches_new() {
    let default: Stack<i32> = Stack::default();
    let created: Stack<i32> = Stack::new();

    assert_eq!(default, created);
    assert!(default.is_empty());
    assert_eq!(default.capacity(), 0);
}

#[test]
fn popping_an_empty_stack_keeps_returning_none() {
    let mut stack: Stack<i32> = Stack::new();

    for _ in 0..3 {
        assert_eq!(stack.pop(), None);
    }

    assert!(stack.is_empty());
    assert_eq!(stack.top(), None);
}

#[test]
fn push_and_pop_are_lifo() {
    let mut stack = Stack::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);

    assert_eq!(stack.len(), 3);
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), None);
}

#[test]
fn repeated_pushes_then_repeated_pops_across_growth() {
    let mut stack = Stack::new();

    for i in 0..1_000 {
        stack.push(i);
    }
    assert_eq!(stack.len(), 1_000);

    for i in (0..1_000).rev() {
        assert_eq!(stack.pop(), Some(i));
    }

    assert!(stack.is_empty());
    assert_eq!(stack.pop(), None);
}

#[test]
fn alternating_push_and_pop_preserves_lifo() {
    let mut stack = Stack::new();

    stack.push(1);
    stack.push(2);
    assert_eq!(stack.pop(), Some(2));

    stack.push(3);
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), None);

    // The stack is drained but still usable.
    stack.push(4);
    assert_eq!(stack.pop(), Some(4));
}

#[test]
fn top_observes_the_most_recent_push() {
    let mut stack = Stack::new();
    assert_eq!(stack.top(), None);

    stack.push(10);
    assert_eq!(stack.top(), Some(&10));

    stack.push(20);
    assert_eq!(stack.top(), Some(&20));

    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.top(), Some(&10));
}

#[test]
fn top_mut_updates_the_top_in_place() {
    let mut stack: Stack<String> = Stack::new();
    stack.push("hello".to_string());

    stack.top_mut().unwrap().push_str(" world");

    assert_eq!(stack.top(), Some(&"hello world".to_string()));
    assert_eq!(
        stack.len(),
        1,
        "mutating the top must not change the length"
    );
    assert_eq!(stack.pop(), Some("hello world".to_string()));
}

#[test]
fn top_mut_after_pop_reflects_the_new_top() {
    let mut stack: Stack<i32> = (0..3).collect();

    *stack.top_mut().unwrap() = 30;
    assert_eq!(stack.top(), Some(&30));

    assert_eq!(stack.pop(), Some(30));
    *stack.top_mut().unwrap() = 10;
    assert_eq!(stack.top(), Some(&10));

    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.top(), Some(&0));
    assert_eq!(stack.top_mut(), Some(&mut 0));
}

#[test]
fn clear_empties_and_the_stack_stays_reusable() {
    let mut stack: Stack<i32> = (0..5).collect();
    assert!(!stack.is_empty());

    stack.clear();

    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
    assert_eq!(stack.top(), None);
    assert_eq!(stack.pop(), None);

    stack.push(1);
    stack.push(2);
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));

    // Clearing twice in a row is harmless.
    stack.clear();
    stack.clear();
    assert!(stack.is_empty());
}

#[test]
fn single_element_stack_supports_every_operation() {
    let mut stack: Stack<i32> = Stack::new();
    stack.push(42);

    assert_eq!(stack.len(), 1);
    assert!(!stack.is_empty());
    assert_eq!(stack.top(), Some(&42));
    assert_eq!(stack.top_mut(), Some(&mut 42));
    assert_eq!(stack.get(0), Some(&42));

    *stack.top_mut().unwrap() = 43;
    assert_eq!(stack.top(), Some(&43));

    assert_eq!(stack.pop(), Some(43));
    assert!(stack.is_empty());
    assert_eq!(stack.top(), None);
    assert_eq!(stack.top_mut(), None);

    // Clearing an already-empty stack is a no-op.
    stack.clear();
    assert!(stack.is_empty());
}

#[test]
fn works_with_non_copy_custom_values() {
    let mut stack: Stack<Token> = Stack::new();
    stack.push(Token {
        id: 1,
        label: "one".to_string(),
    });
    stack.push(Token {
        id: 2,
        label: "two".to_string(),
    });

    assert_eq!(stack.top().map(|token| token.id), Some(2));

    let popped = stack.pop().expect("an element");
    assert_eq!(
        popped,
        Token {
            id: 2,
            label: "two".to_string(),
        }
    );
    assert_eq!(stack.top().map(|token| token.id), Some(1));
}

#[test]
fn strings_round_trip_through_the_stack() {
    let mut stack: Stack<String> = Stack::new();
    stack.push("alpha".to_string());
    stack.push("beta".to_string());

    let joined = format!(
        "{}-{}",
        stack.pop().expect("top"),
        stack.pop().expect("bottom")
    );
    assert_eq!(joined, "beta-alpha");
}

#[test]
fn get_and_get_mut_are_bounded_and_bottom_up() {
    let mut stack: Stack<i32> = (0..3).collect();

    assert_eq!(stack.get(0), Some(&0), "index 0 is the bottom");
    assert_eq!(stack.get(2), Some(&2), "the last index is the top");
    assert_eq!(stack.get(3), None);
    assert_eq!(stack.get(usize::MAX), None);

    *stack.get_mut(0).unwrap() = 100;
    assert_eq!(stack.get(0), Some(&100));
    assert_eq!(stack.get_mut(3), None);
}
