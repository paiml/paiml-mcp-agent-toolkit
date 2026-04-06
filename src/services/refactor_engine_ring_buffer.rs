impl<T> RingBuffer<T> {
    /// Creates a new ring buffer with the specified capacity.
    ///
    /// The ring buffer maintains a fixed-size circular buffer that automatically
    /// evicts the oldest items when capacity is exceeded, following FIFO semantics.
    ///
    /// # Performance
    ///
    /// - Time: O(1) for initialization
    /// - Space: O(capacity) for initial allocation
    /// - Push: O(1) amortized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::services::refactor_engine::RingBuffer;
    ///
    /// let buffer: RingBuffer<i32> = RingBuffer::new(3);
    ///
    /// assert_eq!(buffer.len(), 0);
    /// assert!(buffer.is_empty());
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Pushes an item to the back of the ring buffer.
    ///
    /// If the buffer is at capacity, the oldest item (front) is automatically
    /// removed to make space for the new item.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::services::refactor_engine::RingBuffer;
    ///
    /// let mut buffer = RingBuffer::new(2);
    ///
    /// buffer.push(1);
    /// buffer.push(2);
    /// assert_eq!(buffer.len(), 2);
    ///
    /// // Adding third item evicts the first
    /// buffer.push(3);
    /// assert_eq!(buffer.len(), 2);
    ///
    /// let items = buffer.drain();
    /// assert_eq!(items, vec![2, 3]); // First item (1) was evicted
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn push(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    /// Drains all items from the buffer and returns them as a vector.
    ///
    /// After this operation, the buffer will be empty. Items are returned
    /// in the order they were added (oldest first).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::services::refactor_engine::RingBuffer;
    ///
    /// let mut buffer = RingBuffer::new(5);
    /// buffer.push("first");
    /// buffer.push("second");
    /// buffer.push("third");
    ///
    /// let items = buffer.drain();
    /// assert_eq!(items, vec!["first", "second", "third"]);
    /// assert!(buffer.is_empty());
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn drain(&mut self) -> Vec<T> {
        self.buffer.drain(..).collect()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
