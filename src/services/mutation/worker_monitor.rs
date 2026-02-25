#![cfg_attr(coverage_nightly, coverage(off))]
//! Worker monitoring system for distributed mutation testing
//!
//! Provides a worker state tracking and monitoring system that ensures
//! distributed mutation testing is reliable, observable, and recoverable.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Current state of a worker
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerState {
    /// Worker is idle and available for work
    Idle,

    /// Worker is currently processing a task
    Processing,

    /// Worker has failed
    Failed,

    /// Worker has been terminated
    Terminated,
}

/// Metrics for a single worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMetrics {
    /// Worker ID
    pub id: usize,

    /// Current state of the worker
    pub state: WorkerState,

    /// Number of tasks processed successfully
    pub processed_count: usize,

    /// Number of tasks that failed
    pub failed_count: usize,

    /// When the worker was last active (sent heartbeat)
    #[serde(skip, default = "Instant::now")]
    pub last_heartbeat: Instant,

    /// Most recent error messages (up to 5)
    pub recent_errors: Vec<String>,

    /// Average processing time (ms)
    pub avg_processing_time_ms: f64,

    /// Number of heartbeats received
    pub heartbeat_count: usize,

    /// Custom metrics specific to this worker
    pub custom_metrics: HashMap<String, String>,
}

/// Monitor tracking all workers in the distributed system
pub struct WorkerMonitor {
    /// Collection of worker metrics
    workers: RwLock<HashMap<usize, WorkerMetrics>>,

    /// Timeout for considering a worker stalled
    stall_timeout: Duration,

    /// Total number of workers
    worker_count: usize,
}

// WorkerMetrics method implementations
include!("worker_monitor_tracking.rs");

// WorkerMonitor method implementations
include!("worker_monitor_operations.rs");

// Tests
include!("worker_monitor_tests.rs");
