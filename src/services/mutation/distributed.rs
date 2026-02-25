#![cfg_attr(coverage_nightly, coverage(off))]
//! Distributed mutation testing execution
//!
//! Provides parallel mutant execution with work queue distribution,
//! progress tracking, and result aggregation for production-scale
//! mutation testing workloads.

use super::language::LanguageAdapter;
use super::types::*;
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};

/// Distributed mutation executor configuration
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    /// Number of parallel workers
    pub worker_count: usize,

    /// Maximum concurrent executions
    pub max_concurrent: usize,

    /// Work queue buffer size
    pub queue_size: usize,

    /// Enable progress tracking
    pub track_progress: bool,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        let cpus = num_cpus::get();
        Self {
            worker_count: cpus,
            max_concurrent: cpus * 2,
            queue_size: 1000,
            track_progress: true,
        }
    }
}

/// Progress tracking for mutation execution
#[derive(Debug, Clone)]
pub struct MutationProgress {
    /// Total mutants to execute
    pub total: usize,

    /// Mutants completed
    pub completed: usize,

    /// Mutants currently executing
    pub in_progress: usize,

    /// Killed mutants
    pub killed: usize,

    /// Survived mutants
    pub survived: usize,

    /// Failed/errored mutants
    pub failed: usize,
}

impl MutationProgress {
    fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            in_progress: 0,
            killed: 0,
            survived: 0,
            failed: 0,
        }
    }

    /// Calculate completion percentage
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }

    /// Calculate mutation score (killed / total non-equivalent)
    pub fn mutation_score(&self) -> f64 {
        let total_tested = self.killed + self.survived;
        if total_tested == 0 {
            return 0.0;
        }
        (self.killed as f64 / total_tested as f64) * 100.0
    }
}

/// Distributed mutation executor
pub struct DistributedExecutor {
    /// Language adapter for test execution
    adapter: Arc<dyn LanguageAdapter>,

    /// Configuration for distributed execution
    config: DistributedConfig,

    /// Progress tracking for mutation execution
    progress: Arc<RwLock<MutationProgress>>,

    /// Worker monitoring system
    worker_monitor: Option<Arc<super::worker_monitor::WorkerMonitor>>,
}

// Executor construction, parallel execution, worker pool, and mutant execution
include!("distributed_executor.rs");

// Unit tests for DistributedConfig, MutationProgress, and DistributedExecutor
include!("distributed_tests.rs");
