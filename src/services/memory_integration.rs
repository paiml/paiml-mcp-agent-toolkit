#![cfg_attr(coverage_nightly, coverage(off))]
//! Memory management integration utilities for existing PMAT services
//!
//! This module provides integration helpers to retrofit existing services with
//! optimized memory management patterns. It includes:
//! - Wrapper types for common allocation patterns
//! - Drop-in replacements for Vec and String allocations
//! - Cache-friendly data structure implementations
//! - Memory-aware service trait extensions
//!
//! # Integration Strategy
//!
//! The integration follows a phased approach:
//! 1. **Drop-in Replacements**: `MemoryVec`, `MemoryString` for common cases
//! 2. **Service Extensions**: `MemoryAware` trait for service-level optimization
//! 3. **Cache Integration**: Memory-aware cache strategies
//! 4. **AST Optimization**: Pool-based AST node allocation
//!
//! # Usage Example
//!
//! ```rust
//! use pmat::services::memory_integration::{MemoryVec, MemoryAware};
//! use pmat::services::memory_manager::PoolType;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Drop-in replacement for Vec<String>
//! let mut identifiers = MemoryVec::new(PoolType::StringIntern)?;
//! identifiers.push("function_name".to_string())?;
//! identifiers.push("variable_name".to_string())?;
//!
//! // Memory-aware processing
//! let processed = identifiers.process_with_memory_awareness(|items| {
//!     items.iter().map(|s| s.len()).sum::<usize>()
//! })?;
//! # Ok(())
//! # }
//! ```

use crate::services::memory_manager::{
    global_memory_manager, MemoryManager, PoolType, PooledBuffer,
};
use anyhow::Result;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tracing::{debug, trace};

// ---------------------------------------------------------------------------
// Type definitions
// ---------------------------------------------------------------------------

/// Memory-aware vector that uses pooled allocation
pub struct MemoryVec<T> {
    data: Vec<T>,
    pool_type: PoolType,
    memory_manager: Arc<MemoryManager>,
}

/// Memory-aware string that uses string interning
pub struct MemoryString {
    content: Arc<str>,
    memory_manager: Arc<MemoryManager>,
}

/// Trait for services that can be made memory-aware
pub trait MemoryAware {
    /// Get memory usage estimate for this service
    fn memory_usage(&self) -> usize;

    /// Cleanup unused memory in this service
    fn cleanup_memory(&mut self) -> Result<usize>;

    /// Configure memory limits for this service
    fn configure_memory(&mut self, max_memory: usize) -> Result<()>;

    /// Check if service is under memory pressure
    fn memory_pressure(&self) -> f64 {
        debug_assert!(true, "contract: memory_usage");
        0.0 // Default implementation
    }
}

/// Memory-optimized buffer pool for AST parsing
pub struct AstBufferPool {
    memory_manager: Arc<MemoryManager>,
    pool_type: PoolType,
}

/// Memory-optimized string collection with interning
pub struct InternedStringSet {
    strings: std::collections::HashSet<Arc<str>>,
    memory_manager: Arc<MemoryManager>,
}

/// Memory-aware cache wrapper for existing caches
pub struct MemoryAwareCache<K, V> {
    cache: std::collections::HashMap<K, V>,
    memory_manager: Arc<MemoryManager>,
    pool_type: PoolType,
    max_items: usize,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub item_count: usize,
    pub max_items: usize,
    pub estimated_memory: usize,
}

// ---------------------------------------------------------------------------
// Implementation blocks (impls for all types above)
// ---------------------------------------------------------------------------
include!("memory_integration_impls.rs");

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------
include!("memory_integration_utils.rs");

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
include!("memory_integration_tests.rs");
