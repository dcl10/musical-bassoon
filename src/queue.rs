use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct Queue<T> {
    inner: Mutex<VecDeque<T>>,
    empty: Condvar,
    full: Condvar,
    capacity: usize,
}

impl<T> Queue<T> {
    /// ## Description
    /// Create a new empty queue with the specified capacity.
    ///
    /// ### Args
    /// - `capacity` (`usize`): the maximum number of messages that can be in the queue.
    fn new(capacity: usize) -> Self {
        Queue {
            inner: Mutex::new(VecDeque::with_capacity(capacity)),
            empty: Default::default(),
            full: Default::default(),
            capacity,
        }
    }

    /// ## Description
    /// Add an item to the back of the queue. If the queue is full, this will raise an error.
    ///
    /// ### Args
    /// - `item` (`T`): the item to add to the queue.
    ///
    /// ## Errors
    /// - The queue is full, and you try to add a new item.
    /// - An error occurred connecting to the queue.
    pub fn enqueue(&self, item: T) -> Result<(), &str> {
        let inner = self.inner.lock();
        match inner {
            Ok(mut queue) => {
                if queue.len() == self.capacity {
                    return Err("Queue is full");
                }
                queue.push_back(item);
                self.empty.notify_one();
                Ok(())
            }
            Err(_) => Err("An error occurred connecting to the queue."),
        }
    }

    /// ## Description
    /// Remove an item from the front of the queue.
    ///
    /// ## Errors
    /// - An error occurred connecting to the queue.
    pub fn dequeue(&self) -> Result<Option<T>, &str> {
        let inner = self.inner.lock();
        match inner {
            Ok(mut queue) => {
                let item = queue.pop_front();
                if item.is_some() {
                    self.full.notify_one();
                }
                Ok(item)
            }
            Err(_) => Err("An error occurred connecting to the queue."),
        }
    }
}
