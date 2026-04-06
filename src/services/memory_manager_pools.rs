// Pool infrastructure: PooledBuffer methods/Drop, StringInterner methods, MemoryPool methods
//
// This file is included by memory_manager.rs and shares its module scope.
// Do NOT add `use` imports or `#!` inner attributes here.

impl PooledBuffer {
    fn new(data: Vec<u8>, pool_type: PoolType, manager: Arc<MemoryManager>) -> Self {
        Self {
            data,
            pool_type,
            manager: Some(manager),
        }
    }

    /// Get the buffer data
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable access to buffer data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get buffer capacity
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Resize buffer (may trigger reallocation)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0);
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(manager) = self.manager.take() {
            manager.return_buffer(self.pool_type, std::mem::take(&mut self.data));
        }
    }
}

impl StringInterner {
    fn new(max_size: usize) -> Self {
        Self {
            strings: RwLock::new(FxHashSet::default()),
            total_size: Mutex::new(0),
            max_size,
        }
    }

    /// Intern a string, returning a shared reference
    fn intern(&self, s: &str) -> Result<Arc<str>> {
        // Check if string already exists
        {
            let strings = self.strings.read();
            if let Some(existing) = strings.get(s) {
                return Ok(Arc::clone(existing));
            }
        }

        // Check memory limits
        {
            let current_size = *self.total_size.lock();
            if current_size + s.len() > self.max_size {
                return Err(anyhow!("String interning pool exhausted"));
            }
        }

        // Add new string
        let mut strings = self.strings.write();
        let mut total_size = self.total_size.lock();

        // Double-check after acquiring write lock
        if let Some(existing) = strings.get(s) {
            return Ok(Arc::clone(existing));
        }

        let arc_str: Arc<str> = Arc::from(s);
        *total_size += s.len();
        strings.insert(Arc::clone(&arc_str));

        Ok(arc_str)
    }

    /// Get current memory usage
    fn memory_usage(&self) -> usize {
        *self.total_size.lock()
    }

    /// Clear all interned strings
    fn clear(&self) {
        self.strings.write().clear();
        *self.total_size.lock() = 0;
    }
}

impl MemoryPool {
    fn new(max_size: usize) -> Self {
        Self {
            buffers: Mutex::new(VecDeque::new()),
            total_size: Mutex::new(0),
            max_size,
            allocation_count: Mutex::new(0),
            reuse_count: Mutex::new(0),
        }
    }

    /// Get a buffer from the pool or allocate a new one
    fn get_buffer(&self, min_size: usize) -> Vec<u8> {
        let mut buffers = self.buffers.lock();
        let mut total_size = self.total_size.lock();

        // Try to reuse existing buffer
        if let Some(mut buffer) = buffers.pop_front() {
            if buffer.capacity() >= min_size {
                buffer.clear();
                buffer.resize(min_size, 0);
                *self.reuse_count.lock() += 1;
                return buffer;
            }
            // Buffer too small, account for its removal
            *total_size -= buffer.capacity();
        }

        // Allocate new buffer
        *self.allocation_count.lock() += 1;
        let mut buffer = Vec::with_capacity(min_size.max(4096)); // Minimum 4KB
        buffer.resize(min_size, 0);
        *total_size += buffer.capacity();

        buffer
    }

    /// Return a buffer to the pool
    fn return_buffer(&self, mut buffer: Vec<u8>) {
        let mut buffers = self.buffers.lock();
        let total_size = self.total_size.lock();

        // Don't store buffer if pool is full
        if *total_size + buffer.capacity() > self.max_size {
            return;
        }

        // Clear buffer and return to pool
        buffer.clear();
        buffers.push_back(buffer);
    }

    /// Get pool statistics
    fn stats(&self) -> PoolStats {
        let buffers = self.buffers.lock();
        let total_size = *self.total_size.lock();
        let allocation_count = *self.allocation_count.lock();
        let reuse_count = *self.reuse_count.lock();

        PoolStats {
            buffer_count: buffers.len(),
            total_size,
            allocation_count,
            reuse_count,
            reuse_ratio: if allocation_count > 0 {
                reuse_count as f64 / allocation_count as f64
            } else {
                0.0
            },
        }
    }

    /// Clear all buffers
    fn clear(&self) {
        self.buffers.lock().clear();
        *self.total_size.lock() = 0;
    }
}
