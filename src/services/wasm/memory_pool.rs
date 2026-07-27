//! WebAssembly memory pool management
//!
//! This module provides memory pool management for WebAssembly parsing.

/// Memory pool for WASM parsing
pub struct MemoryPool {
    max_size: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Get maximum pool size
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new(64 * 1024 * 1024) // 64MB default
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_new() {
        let pool = MemoryPool::new(1024);
        assert_eq!(pool.max_size(), 1024);
    }

    #[test]
    fn test_memory_pool_default() {
        let pool = MemoryPool::default();
        assert_eq!(pool.max_size(), 64 * 1024 * 1024); // 64MB
    }

    #[test]
    fn test_memory_pool_zero_size() {
        let pool = MemoryPool::new(0);
        assert_eq!(pool.max_size(), 0);
    }

    #[test]
    fn test_memory_pool_large_size() {
        let pool = MemoryPool::new(usize::MAX);
        assert_eq!(pool.max_size(), usize::MAX);
    }
}
