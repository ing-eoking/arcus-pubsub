use std::cell::RefCell;
use std::rc::{Rc, Weak};


pub type Link<T> = Option<Rc<RefCell<Node<T>>>>;
type WeakLink<T> = Option<Weak<RefCell<Node<T>>>>;

struct Node<T> {
    value: T,
    prev: WeakLink<T>,
    next: Link<T>,
}


#[derive(Default)]
pub struct DoublyLinkedList<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn push_back(&mut self, value: T) -> Link<T> {
        let new_node = Rc::new(RefCell::new(Node {
            value,
            prev: None,
            next: None,
        }));

        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(new_node.clone());
                new_node.borrow_mut().prev = Some(Rc::downgrade(&old_tail));
                self.tail = Some(new_node.clone());
            }
            None => {
                self.head = Some(new_node.clone());
                self.tail = Some(new_node.clone());
            }
        }

        self.len += 1;
        Some(new_node)
    }

    pub fn remove(&mut self, node: &Rc<RefCell<Node<T>>>) {
        let prev = node.borrow().prev.as_ref().and_then(|w| w.upgrade());
        let next = node.borrow().next.clone();

        if let Some(ref prev_node) = prev {
            prev_node.borrow_mut().next = next.clone();
        } else {
            self.head = next.clone();
        }

        if let Some(ref next_node) = next {
            next_node.borrow_mut().prev = prev.as_ref().map(Rc::downgrade);
        } else {
            self.tail = prev;
        }

        self.len -= 1;
    }

    pub fn print(&self)
    where
        T: std::fmt::Debug,
    {
        let mut cur = self.head.clone();
        while let Some(node) = cur {
            print!("{:?} ", node.borrow().value);
            cur = node.borrow().next.clone();
        }
        println!();
    }
}
