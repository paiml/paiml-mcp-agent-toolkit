//! WebAssembly memory pool management
//!
//! This module provides memory pool management for WebAssembly parsing.

/// Memory pool for WASM parsing
pub struct MemoryPool {
    max_size: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Get maximum pool size
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new(64 * 1024 * 1024) // 64MB default
    }
}
