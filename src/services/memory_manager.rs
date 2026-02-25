#![cfg_attr(coverage_nightly, coverage(off))]
//! Advanced memory management optimization for PMAT
//!
//! This module provides comprehensive memory management optimizations including:
//! - Memory pools for AST parsing and analysis operations
//! - Smart allocation strategies based on file size and complexity
//! - Memory-mapped caching for large datasets
//! - String interning for repeated identifiers
//! - Buffer reuse patterns for reduced allocation pressure
//!
//! # Design Principles
//!
//! - **Zero-Copy Where Possible**: Use references and borrowing instead of cloning
//! - **Pool-Based Allocation**: Pre-allocate memory pools for common operations
//! - **Size-Aware Strategies**: Different approaches for small vs large files
//! - **Cache-Friendly Layouts**: Optimize data structures for CPU cache efficiency
//! - **Deterministic Cleanup**: Explicit memory lifecycle management
//!
//! # Example Usage
//!
//! ```rust
//! use pmat::services::memory_manager::{MemoryManager, PoolType};
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut manager = MemoryManager::new()?;
//!
//! // Configure pools based on expected workload
//! manager.configure_pool(PoolType::AstParsing, 32 * 1024 * 1024)?; // 32MB
//! manager.configure_pool(PoolType::StringIntern, 8 * 1024 * 1024)?;  // 8MB
//!
//! // Use pooled allocation for analysis
//! let buffer = manager.allocate_buffer(PoolType::AstParsing, 4096)?;
//! let interned_string = manager.intern_string("common_identifier")?;
//!
//! // Automatic cleanup when manager is dropped
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Result};
use parking_lot::{Mutex, RwLock};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, trace, warn};

/// Memory pool types for different allocation patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolType {
    /// AST parsing buffers (frequent allocation/deallocation)
    AstParsing,
    /// String interning pool for identifiers and tokens
    StringIntern,
    /// Analysis result caching (medium-lived allocations)
    AnalysisCache,
    /// Temporary file content buffers
    FileContent,
    /// Graph construction (connected components, DAGs)
    GraphConstruction,
}

/// Memory allocation strategy based on size and usage patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Use memory pool for small, frequent allocations
    Pooled,
    /// Direct allocation for large or infrequent allocations
    Direct,
    /// Memory-mapped allocation for very large data
    MemoryMapped,
}

/// Configuration for memory management behavior
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum total memory usage (bytes)
    pub max_total_memory: usize,
    /// Pool size limits per type
    pub pool_limits: FxHashMap<PoolType, usize>,
    /// Allocation strategy thresholds
    pub small_allocation_threshold: usize, // < 4KB
    pub large_allocation_threshold: usize, // > 1MB
    /// Cache eviction policy parameters
    pub max_cache_age: Duration,
    pub cache_pressure_threshold: f64, // 0.0-1.0
    /// Enable memory tracking and debugging
    pub enable_tracking: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        let mut pool_limits = FxHashMap::default();
        pool_limits.insert(PoolType::AstParsing, 64 * 1024 * 1024); // 64MB
        pool_limits.insert(PoolType::StringIntern, 16 * 1024 * 1024); // 16MB
        pool_limits.insert(PoolType::AnalysisCache, 128 * 1024 * 1024); // 128MB
        pool_limits.insert(PoolType::FileContent, 32 * 1024 * 1024); // 32MB
        pool_limits.insert(PoolType::GraphConstruction, 32 * 1024 * 1024); // 32MB

        Self {
            max_total_memory: 512 * 1024 * 1024, // 512MB
            pool_limits,
            small_allocation_threshold: 4 * 1024,    // 4KB
            large_allocation_threshold: 1024 * 1024, // 1MB
            max_cache_age: Duration::from_secs(300), // 5 minutes
            cache_pressure_threshold: 0.85,
            enable_tracking: true,
        }
    }
}

/// Memory buffer with automatic pool return
pub struct PooledBuffer {
    data: Vec<u8>,
    pool_type: PoolType,
    manager: Option<Arc<MemoryManager>>,
}

/// String interning for memory-efficient identifier storage
#[derive(Debug)]
struct StringInterner {
    strings: RwLock<FxHashSet<Arc<str>>>,
    total_size: Mutex<usize>,
    max_size: usize,
}

/// Memory pool for efficient buffer reuse
#[derive(Debug)]
struct MemoryPool {
    buffers: Mutex<VecDeque<Vec<u8>>>,
    total_size: Mutex<usize>,
    max_size: usize,
    allocation_count: Mutex<u64>,
    reuse_count: Mutex<u64>,
}

/// Memory pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub buffer_count: usize,
    pub total_size: usize,
    pub allocation_count: u64,
    pub reuse_count: u64,
    pub reuse_ratio: f64,
}

/// Memory usage tracking
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub pool_stats: FxHashMap<PoolType, PoolStats>,
    pub string_intern_size: usize,
    pub peak_usage: usize,
    pub allocation_pressure: f64,
}

/// Main memory manager for PMAT
pub struct MemoryManager {
    config: MemoryConfig,
    pools: FxHashMap<PoolType, MemoryPool>,
    string_interner: StringInterner,
    total_allocated: Mutex<usize>,
    peak_usage: Mutex<usize>,
    last_cleanup: Mutex<Instant>,
}

// Pool infrastructure: PooledBuffer methods/Drop, StringInterner, MemoryPool
include!("memory_manager_pools.rs");

// Core MemoryManager implementation and global manager functions
include!("memory_manager_core.rs");

// Unit tests and property tests
include!("memory_manager_tests.rs");
