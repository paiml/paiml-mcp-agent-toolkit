#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{interval, sleep};

/// Operation types for resource planning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Analysis,
    Commit,
    Background,
    Storage,
    Cleanup,
}

/// Resource limits for platform control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: f64,
    /// Maximum CPU utilization (0.0 - 1.0)
    pub max_cpu_utilization: f64,
    /// Maximum concurrent operations
    pub max_concurrent_ops: usize,
    /// Memory warning threshold (0.0 - 1.0)
    pub memory_warning_threshold: f64,
    /// CPU warning threshold (0.0 - 1.0)
    pub cpu_warning_threshold: f64,
    /// Resource check interval in seconds
    pub check_interval_secs: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024.0,         // 1GB limit
            max_cpu_utilization: 0.8,      // 80% CPU max
            max_concurrent_ops: 20,        // 20 concurrent operations
            memory_warning_threshold: 0.7, // 70% warning
            cpu_warning_threshold: 0.6,    // 60% warning
            check_interval_secs: 5,        // Check every 5 seconds
        }
    }
}

/// Current resource usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub memory_mb: f64,
    pub cpu_utilization: f64,
    pub active_operations: usize,
    pub memory_pressure: ResourcePressure,
    pub cpu_pressure: ResourcePressure,
}

/// Resource pressure levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourcePressure {
    Low,      // Below warning threshold
    Medium,   // Above warning, below limit
    High,     // At or above limit
    Critical, // Emergency conditions
}

/// Resource enforcement actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAction {
    /// Allow operation to proceed
    Allow,
    /// Throttle operation (add delay)
    Throttle { delay_ms: u64 },
    /// Queue operation for later
    Queue { estimated_wait_ms: u64 },
    /// Reject operation due to resource constraints
    Reject { reason: String },
    /// Emergency shutdown of non-critical operations
    EmergencyStop,
}

/// Platform resource controller
pub struct PlatformResourceController {
    limits: ResourceLimits,
    current_usage: Arc<RwLock<ResourceUsage>>,
    operation_semaphore: Arc<Semaphore>,
    active_operations: Arc<RwLock<HashMap<String, OperationContext>>>,
    enforcement_history: Arc<RwLock<Vec<EnforcementEvent>>>,
    monitoring_active: Arc<RwLock<bool>>,
}

/// Context for active operations
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub id: String,
    pub operation_type: OperationType,
    pub started_at: Instant,
    pub estimated_memory_mb: f64,
    pub priority: OperationPriority,
}

/// Operation priority for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationPriority {
    Critical, // User commits, must succeed
    High,     // Interactive analysis
    Medium,   // Background analysis
    Low,      // Cleanup, maintenance
}

/// Resource enforcement event for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementEvent {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub operation_id: String,
    pub action: ResourceAction,
    pub resource_usage: ResourceUsage,
    pub reason: String,
}

/// Resource allocation guard - automatically releases resources when dropped
pub struct ResourceAllocation {
    operation_id: String,

    permit: tokio::sync::OwnedSemaphorePermit,
    active_operations: Arc<RwLock<HashMap<String, OperationContext>>>,
}

/// Resource enforcement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEnforcementStats {
    pub total_requests: usize,
    pub allowed_requests: usize,
    pub throttled_requests: usize,
    pub queued_requests: usize,
    pub rejected_requests: usize,
    pub current_active_operations: usize,
}

/// Factory for creating resource controllers
pub struct ResourceControllerFactory;

// --- Implementation includes ---
include!("controller_core.rs");
include!("controller_evaluation.rs");
include!("allocation_and_stats.rs");

// --- Test includes ---
include!("tests_async.rs");
include!("tests_simple.rs");
include!("tests_coverage.rs");
include!("tests_controller.rs");
