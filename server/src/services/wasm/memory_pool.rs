//! Memory pool for WebAssembly parser instances
//!
//! This module provides efficient pooling of parser instances to reduce
//! allocation overhead and improve performance for concurrent parsing.
use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::assemblyscript::AssemblyScriptParser;
use super::binary::WasmBinaryAnalyzer;
use super::error::WasmResult;
use super::traits::WasmAwareParser;
use super::wat::WatParser;

/// Strategy for allocating parsers
#[derive(Debug, Clone, Copy)]
pub enum AllocationStrategy {
    /// Create new parser when pool is empty
    Dynamic,

    /// Wait for parser availability  
    Blocking,

    /// Fail if no parser available
    FailFast,
}

/// Configuration for parser pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum parsers per type
    pub max_parsers_per_type: usize,

    /// Memory limit per parser in bytes
    pub memory_limit_per_parser: usize,

    /// Allocation strategy
    pub allocation_strategy: AllocationStrategy,

    /// Maximum wait time for blocking strategy
    pub max_wait_time: Duration,

    /// Enable parser health checks
    pub enable_health_checks: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_parsers_per_type: 8,
            memory_limit_per_parser: 100 * 1_024 * 1_024, // 100MB
            allocation_strategy: AllocationStrategy::Dynamic,
            max_wait_time: Duration::from_secs(5),
            enable_health_checks: true,
        }
    }
}

/// Pooled parser wrapper
pub struct PooledParser {
    parser: Box<dyn WasmAwareParser>,
    pool: Arc<ArrayQueue<Box<dyn WasmAwareParser>>>,
    created_at: Instant,
    usage_count: usize,
}

impl PooledParser {
    /// Create a new pooled parser
    fn new(
        parser: Box<dyn WasmAwareParser>,
        pool: Arc<ArrayQueue<Box<dyn WasmAwareParser>>>,
    ) -> Self {
        Self {
            parser,
            pool,
            created_at: Instant::now(),
            usage_count: 0,
        }
    }
    ///
    ///
    /// # Panics
    ///
    /// May panic on out-of-bounds array/slice access

    /// Get the underlying parser
//! Memory pool for WebAssembly parser instances
//!
//! This module provides efficient pooling of parser instances to reduce
//! allocation overhead and improve performance for concurrent parsing.
use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::assemblyscript::AssemblyScriptParser;
use super::binary::WasmBinaryAnalyzer;
use super::error::WasmResult;
use super::traits::WasmAwareParser;
use super::wat::WatParser;

/// Strategy for allocating parsers
#[derive(Debug, Clone, Copy)]
pub enum AllocationStrategy {
    /// Create new parser when pool is empty
    Dynamic,

    /// Wait for parser availability  
    Blocking,

    /// Fail if no parser available
    FailFast,
}

/// Configuration for parser pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum parsers per type
    pub max_parsers_per_type: usize,

    /// Memory limit per parser in bytes
    pub memory_limit_per_parser: usize,

    /// Allocation strategy
    pub allocation_strategy: AllocationStrategy,

    /// Maximum wait time for blocking strategy
    pub max_wait_time: Duration,

    /// Enable parser health checks
    pub enable_health_checks: bool,
}
