use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct Queue<T> {
    inner: Mutex<VecDeque<T>>,
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
        Ok(())
    }

    /// ## Description
    /// Remove an item from the front of the queue. Returns `None`
    /// immediately if the queue is empty rather than blocking — an empty
    /// queue is a valid state, not an error condition to wait out.
    ///
    /// ## Errors
    /// - An error occurred connecting to the queue (the lock was poisoned).
    pub fn dequeue(&self) -> Result<Option<T>, &str> {
        let mut queue = self
            .inner
            .lock()
            .map_err(|_| "An error occurred connecting to the queue.")?;

        let item = queue.pop_front();
        if item.is_some() {
            self.full.notify_one();
        }
        Ok(item)
    }
}
