// Core MemoryManager implementation and global manager functions
//
// This file is included by memory_manager.rs and shares its module scope.
// Do NOT add `use` imports or `#!` inner attributes here.

impl MemoryManager {
    /// Create a new memory manager with default configuration
    pub fn new() -> Result<Arc<Self>> {
        Self::with_config(MemoryConfig::default())
    }

    /// Create a new memory manager with custom configuration
    pub fn with_config(config: MemoryConfig) -> Result<Arc<Self>> {
        let mut pools = FxHashMap::default();

        for (&pool_type, &max_size) in &config.pool_limits {
            pools.insert(pool_type, MemoryPool::new(max_size));
        }

        let string_interner = StringInterner::new(
            config
                .pool_limits
                .get(&PoolType::StringIntern)
                .copied()
                .unwrap_or(16 * 1024 * 1024),
        );

        Ok(Arc::new(Self {
            config,
            pools,
            string_interner,
            total_allocated: Mutex::new(0),
            peak_usage: Mutex::new(0),
            last_cleanup: Mutex::new(Instant::now()),
        }))
    }

    /// Configure a specific memory pool
    pub fn configure_pool(&self, pool_type: PoolType, _max_size: usize) -> Result<()> {
        if let Some(_pool) = self.pools.get(&pool_type) {
            // Note: Current implementation doesn't support runtime pool resizing
            // This would require a more complex design with pool reconstruction
            warn!("Pool reconfiguration not supported in current implementation");
        }
        Ok(())
    }

    /// Allocate a buffer using the appropriate strategy
    pub fn allocate_buffer(
        self: &Arc<Self>,
        pool_type: PoolType,
        size: usize,
    ) -> Result<PooledBuffer> {
        debug_assert!(size > 0, "size must be positive");
        let strategy = self.determine_strategy(size);

        match strategy {
            AllocationStrategy::Pooled => {
                if let Some(pool) = self.pools.get(&pool_type) {
                    let buffer = pool.get_buffer(size);
                    self.track_allocation(buffer.capacity());
                    Ok(PooledBuffer::new(buffer, pool_type, Arc::clone(self)))
                } else {
                    Err(anyhow!("Pool type {pool_type:?} not configured"))
                }
            }
            AllocationStrategy::Direct => {
                let buffer = vec![0; size];
                self.track_allocation(buffer.capacity());
                Ok(PooledBuffer::new(buffer, pool_type, Arc::clone(self)))
            }
            AllocationStrategy::MemoryMapped => {
                // For very large allocations, use direct allocation
                // Memory mapping would require file-backed storage
                let buffer = vec![0; size];
                self.track_allocation(buffer.capacity());
                Ok(PooledBuffer::new(buffer, pool_type, Arc::clone(self)))
            }
        }
    }

    /// Intern a string for memory efficiency
    pub fn intern_string(&self, s: &str) -> Result<Arc<str>> {
        debug_assert!(!s.is_empty(), "s must not be empty");
        self.string_interner.intern(s)
    }

    /// Get current memory statistics
    pub fn stats(&self) -> MemoryStats {
        let total_allocated = *self.total_allocated.lock();
        let peak_usage = *self.peak_usage.lock();

        let mut pool_stats = FxHashMap::default();
        for (&pool_type, pool) in &self.pools {
            pool_stats.insert(pool_type, pool.stats());
        }

        let string_intern_size = self.string_interner.memory_usage();
        let allocation_pressure = total_allocated as f64 / self.config.max_total_memory as f64;

        MemoryStats {
            total_allocated,
            pool_stats,
            string_intern_size,
            peak_usage,
            allocation_pressure,
        }
    }

    /// Force cleanup of unused memory
    pub fn cleanup(&self) -> Result<usize> {
        let mut cleaned = 0;

        // Check if cleanup is needed
        let now = Instant::now();
        let mut last_cleanup = self.last_cleanup.lock();
        if now.duration_since(*last_cleanup) < Duration::from_secs(30) {
            return Ok(0); // Too recent
        }
        *last_cleanup = now;

        // Check memory pressure
        let stats = self.stats();
        if stats.allocation_pressure < self.config.cache_pressure_threshold {
            return Ok(0); // No pressure
        }

        // Clear string interner if under pressure
        if stats.allocation_pressure > 0.9 {
            self.string_interner.clear();
            cleaned += stats.string_intern_size;
            info!(
                "Cleared string interner: {} bytes",
                stats.string_intern_size
            );
        }

        // Clear least recently used pool buffers
        for (pool_type, pool) in &self.pools {
            let pool_stats = pool.stats();
            if pool_stats.total_size > 0 {
                pool.clear();
                cleaned += pool_stats.total_size;
                debug!(
                    "Cleared pool {:?}: {} bytes",
                    pool_type, pool_stats.total_size
                );
            }
        }

        if cleaned > 0 {
            info!("Memory cleanup freed {} bytes", cleaned);
        }

        Ok(cleaned)
    }

    /// Determine allocation strategy based on size
    fn determine_strategy(&self, size: usize) -> AllocationStrategy {
        debug_assert!(size > 0, "size must be positive");
        if size < self.config.small_allocation_threshold {
            AllocationStrategy::Pooled
        } else if size > self.config.large_allocation_threshold {
            AllocationStrategy::MemoryMapped
        } else {
            AllocationStrategy::Direct
        }
    }

    /// Track memory allocation for statistics
    fn track_allocation(&self, size: usize) {
        debug_assert!(size > 0, "size must be positive");
        let mut total = self.total_allocated.lock();
        *total += size;

        let mut peak = self.peak_usage.lock();
        if *total > *peak {
            *peak = *total;
        }

        // Trigger cleanup if approaching limit
        if *total as f64 / self.config.max_total_memory as f64
            > self.config.cache_pressure_threshold
        {
            trace!(
                "Memory pressure detected: {:.1}%",
                *total as f64 / self.config.max_total_memory as f64 * 100.0
            );
        }
    }

    /// Return buffer to pool (internal use)
    fn return_buffer(&self, pool_type: PoolType, buffer: Vec<u8>) {
        debug_assert!(!buffer.is_empty(), "buffer must not be empty");
        if let Some(pool) = self.pools.get(&pool_type) {
            let capacity = buffer.capacity();
            pool.return_buffer(buffer);

            let mut total = self.total_allocated.lock();
            *total = total.saturating_sub(capacity);
        }
    }
}

/// Global memory manager instance
static GLOBAL_MEMORY_MANAGER: std::sync::OnceLock<Arc<MemoryManager>> = std::sync::OnceLock::new();

/// Get the global memory manager instance
pub fn global_memory_manager() -> Result<Arc<MemoryManager>> {
    GLOBAL_MEMORY_MANAGER
        .get()
        .cloned()
        .ok_or_else(|| anyhow!("Global memory manager not initialized"))
}

/// Initialize the global memory manager
pub fn init_global_memory_manager() -> Result<()> {
    let manager = MemoryManager::new()?;
    GLOBAL_MEMORY_MANAGER
        .set(manager)
        .map_err(|_| anyhow!("Global memory manager already initialized"))?;
    Ok(())
}

/// Initialize the global memory manager with custom config
pub fn init_global_memory_manager_with_config(config: MemoryConfig) -> Result<()> {
    let manager = MemoryManager::with_config(config)?;
    GLOBAL_MEMORY_MANAGER
        .set(manager)
        .map_err(|_| anyhow!("Global memory manager already initialized"))?;
    Ok(())
}
