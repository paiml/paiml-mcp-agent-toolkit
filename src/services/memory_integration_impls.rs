// Implementation blocks for memory integration types
// Included from memory_integration.rs - shares parent module scope

impl<T> MemoryVec<T> {
    /// Create a new memory-aware vector
    pub fn new(pool_type: PoolType) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            data: Vec::new(),
            pool_type,
            memory_manager,
        })
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(pool_type: PoolType, capacity: usize) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            data: Vec::with_capacity(capacity),
            pool_type,
            memory_manager,
        })
    }

    /// Push an item with memory tracking
    pub fn push(&mut self, item: T) -> Result<()> {
        let old_capacity = self.data.capacity();
        self.data.push(item);

        // Track memory growth
        let new_capacity = self.data.capacity();
        if new_capacity > old_capacity {
            let growth = (new_capacity - old_capacity) * std::mem::size_of::<T>();
            trace!(
                "MemoryVec grew by {} bytes for pool {:?}",
                growth,
                self.pool_type
            );
        }

        Ok(())
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) -> Result<()> {
        let old_capacity = self.data.capacity();
        self.data.reserve(additional);

        let new_capacity = self.data.capacity();
        if new_capacity > old_capacity {
            let growth = (new_capacity - old_capacity) * std::mem::size_of::<T>();
            trace!(
                "MemoryVec reserved {} bytes for pool {:?}",
                growth,
                self.pool_type
            );
        }

        Ok(())
    }

    /// Get memory usage estimate
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        self.data.capacity() * std::mem::size_of::<T>()
    }

    /// Process data with memory awareness
    pub fn process_with_memory_awareness<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&[T]) -> R,
    {
        // Check memory pressure before processing
        let stats = self.memory_manager.stats();
        if stats.allocation_pressure > 0.9 {
            debug!(
                "High memory pressure detected: {:.1}%",
                stats.allocation_pressure * 100.0
            );
            self.memory_manager.cleanup()?;
        }

        Ok(f(&self.data))
    }
}

impl<T> Deref for MemoryVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for MemoryVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl MemoryString {
    /// Create a new memory-aware string with interning
    pub fn new(content: &str) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        let interned = memory_manager.intern_string(content)?;
        Ok(Self {
            content: interned,
            memory_manager,
        })
    }

    /// Get the string content
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Check if this string shares memory with another
    #[must_use]
    pub fn shares_memory_with(&self, other: &MemoryString) -> bool {
        Arc::ptr_eq(&self.content, &other.content)
    }

    /// Get memory statistics for this string
    #[must_use]
    pub fn memory_stats(&self) -> super::memory_manager::MemoryStats {
        self.memory_manager.stats()
    }
}

impl Deref for MemoryString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl std::fmt::Display for MemoryString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl std::fmt::Debug for MemoryString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemoryString({})", self.content)
    }
}

impl AstBufferPool {
    /// Create a new AST buffer pool
    pub fn new(pool_type: PoolType) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            memory_manager,
            pool_type,
        })
    }

    /// Get a buffer for AST parsing
    pub fn get_buffer(&self, min_size: usize) -> Result<PooledBuffer> {
        self.memory_manager
            .allocate_buffer(self.pool_type, min_size)
    }

    /// Get buffer sized for specific file content
    pub fn get_buffer_for_content(&self, content: &str) -> Result<PooledBuffer> {
        // Size buffer based on content with some overhead for parsing structures
        let size = content.len() * 2; // 2x overhead for AST structures
        self.memory_manager.allocate_buffer(self.pool_type, size)
    }
}

impl InternedStringSet {
    /// Create a new interned string set
    pub fn new() -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            strings: std::collections::HashSet::new(),
            memory_manager,
        })
    }

    /// Insert a string with interning
    pub fn insert(&mut self, s: &str) -> Result<bool> {
        let interned = self.memory_manager.intern_string(s)?;
        Ok(self.strings.insert(interned))
    }

    /// Check if string exists
    #[must_use]
    pub fn contains(&self, s: &str) -> bool {
        // Note: This requires interning the string to check, which is not ideal
        // In practice, you'd store the interned version when inserting
        if let Ok(interned) = self.memory_manager.intern_string(s) {
            self.strings.contains(&interned)
        } else {
            false
        }
    }

    /// Get all strings
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.strings.iter().map(std::convert::AsRef::as_ref)
    }

    /// Get memory usage
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        // Rough estimate - doesn't account for interning savings
        self.strings.len() * std::mem::size_of::<Arc<str>>()
    }
}

impl<K, V> MemoryAwareCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new memory-aware cache
    pub fn new(pool_type: PoolType, max_items: usize) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            cache: std::collections::HashMap::new(),
            memory_manager,
            pool_type,
            max_items,
        })
    }

    /// Insert with memory pressure checking
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        // Check memory pressure
        let stats = self.memory_manager.stats();
        if stats.allocation_pressure > 0.85 && self.cache.len() >= self.max_items {
            // Evict oldest entries (simplified LRU)
            let keys_to_remove: Vec<_> = self
                .cache
                .keys()
                .take(self.cache.len() / 4)
                .cloned()
                .collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
            debug!("Evicted cache entries due to memory pressure");
        }

        Ok(self.cache.insert(key, value))
    }

    /// Get cached value
    pub fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    /// Get cache statistics
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            item_count: self.cache.len(),
            max_items: self.max_items,
            estimated_memory: self.cache.len()
                * (std::mem::size_of::<K>() + std::mem::size_of::<V>()),
        }
    }

    /// Get the memory pool type used by this cache
    #[must_use]
    pub fn pool_type(&self) -> PoolType {
        self.pool_type
    }
}
