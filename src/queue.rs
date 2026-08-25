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
    pub fn new(capacity: usize) -> Self {
        Queue {
            inner: Mutex::new(VecDeque::with_capacity(capacity)),
            empty: Default::default(),
            full: Default::default(),
            capacity,
        }
    }

    /// ## Description
    /// Add an item to the back of the queue. If the queue is full, this
    /// will block the calling thread until space becomes available.
    ///
    /// ### Args
    /// - `item` (`T`): the item to add to the queue.
    ///
    /// ## Errors
    /// - An error occurred connecting to the queue (the lock was poisoned).
    pub fn enqueue(&self, item: T) -> Result<(), &str> {
        let mut queue = self
            .inner
            .lock()
            .map_err(|_| "An error occurred connecting to the queue.")?;

        while queue.len() == self.capacity {
            queue = self
                .full
                .wait(queue)
                .map_err(|_| "An error occurred connecting to the queue.")?;
        }

        queue.push_back(item);
        self.empty.notify_one();
        Ok(())
    }

    /// ## Description
    /// Remove an item from the front of the queue. If the queue is empty,
    /// this will block the calling thread until an item becomes available.
    ///
    /// ## Errors
    /// - An error occurred connecting to the queue (the lock was poisoned).
    pub fn dequeue(&self) -> Result<Option<T>, &str> {
        let mut queue = self
            .inner
            .lock()
            .map_err(|_| "An error occurred connecting to the queue.")?;

        while queue.is_empty() {
            queue = self
                .empty
                .wait(queue)
                .map_err(|_| "An error occurred connecting to the queue.")?;
        }

        let item = queue.pop_front(); // safe: while loop guarantees non-empty
        self.full.notify_one();
        Ok(item)
    }
}
