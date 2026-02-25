#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handlers for memory management operations
//!
//! This module provides command-line interface for memory optimization features:
//! - Memory usage monitoring and statistics
//! - Manual memory cleanup operations
//! - Memory configuration management
//! - Pool-specific memory analysis
//!
//! # Available Commands
//!
//! - `pmat memory stats` - Show current memory usage statistics
//! - `pmat memory cleanup` - Force memory cleanup operations
//! - `pmat memory configure` - Configure memory limits and policies
//! - `pmat memory pools` - Show detailed pool statistics
//! - `pmat memory pressure` - Check current memory pressure level

use crate::services::memory_manager::{global_memory_manager, init_global_memory_manager};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Subcommand)]
pub enum MemoryCommand {
    /// Show current memory usage statistics
    Stats {
        /// Show detailed pool-by-pool breakdown
        #[arg(long)]
        detailed: bool,
        /// Output format (table, json, csv)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Force memory cleanup operations
    Cleanup {
        /// Target memory pressure level (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        target_pressure: f64,
        /// Show cleanup progress
        #[arg(long)]
        verbose: bool,
    },
    /// Configure memory limits and policies
    Configure {
        /// Maximum total memory usage in MB
        #[arg(long)]
        max_memory_mb: Option<usize>,
        /// Pool-specific limits (format: `pool:size_mb`)
        #[arg(long, value_delimiter = ',')]
        pool_limits: Vec<String>,
        /// Enable memory tracking
        #[arg(long)]
        enable_tracking: Option<bool>,
    },
    /// Show detailed pool statistics
    Pools {
        /// Pool type to focus on
        #[arg(long)]
        pool: Option<String>,
        /// Show efficiency metrics
        #[arg(long)]
        efficiency: bool,
    },
    /// Check current memory pressure level
    Pressure {
        /// Set pressure threshold for warnings (0.0-1.0)
        #[arg(long, default_value = "0.8")]
        threshold: f64,
        /// Check continuously (interval in seconds)
        #[arg(long)]
        watch: Option<u64>,
    },
}

/// Memory statistics output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatsOutput {
    pub total_allocated: usize,
    pub peak_usage: usize,
    pub allocation_pressure: f64,
    pub string_intern_size: usize,
    pub pool_stats: HashMap<String, PoolStatsOutput>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatsOutput {
    pub buffer_count: usize,
    pub total_size: usize,
    pub allocation_count: u64,
    pub reuse_count: u64,
    pub reuse_ratio: f64,
    pub efficiency_rating: String,
}

/// Handle memory management commands
pub async fn handle_memory_command(command: &MemoryCommand) -> Result<()> {
    // Initialize memory manager if not already done
    if global_memory_manager().is_err() {
        info!("Initializing global memory manager");
        init_global_memory_manager()?;
    }

    match command {
        MemoryCommand::Stats { detailed, format } => handle_memory_stats(*detailed, format).await,
        MemoryCommand::Cleanup {
            target_pressure,
            verbose,
        } => handle_memory_cleanup(*target_pressure, *verbose).await,
        MemoryCommand::Configure {
            max_memory_mb,
            pool_limits,
            enable_tracking,
        } => handle_memory_configure(max_memory_mb, pool_limits, enable_tracking).await,
        MemoryCommand::Pools { pool, efficiency } => handle_memory_pools(pool, *efficiency).await,
        MemoryCommand::Pressure { threshold, watch } => {
            handle_memory_pressure(*threshold, watch).await
        }
    }
}

// --- Submodule includes ---
include!("memory_display.rs");
include!("memory_operations.rs");
include!("memory_tests.rs");
