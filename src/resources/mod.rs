#![cfg_attr(coverage_nightly, coverage(off))]
// Resource control and scheduling system
pub mod adaptive_allocator;
pub mod cpu_limiter;
pub mod gpu_scheduler;
pub mod io_throttle;
pub mod memory_limiter;
pub mod network_throttle;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Resource limits.
pub struct ResourceLimits {
    pub cpu: CpuLimits,
    pub memory: MemoryLimits,
    pub gpu: Option<GpuLimits>,
    pub network: NetworkLimits,
    pub disk_io: DiskIoLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Cpu limits.
pub struct CpuLimits {
    pub cores: f32,               // Fractional cores (e.g., 1.5)
    pub max_percent: f32,         // Max CPU percentage
    pub scheduling_priority: i32, // Nice value (-20 to 19)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Memory limits.
pub struct MemoryLimits {
    pub max_bytes: usize,
    pub max_heap_bytes: Option<usize>,
    pub max_stack_bytes: Option<usize>,
    pub swap_limit_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Gpu limits.
pub struct GpuLimits {
    pub device_id: u32,
    pub memory_bytes: usize,
    pub compute_percent: f32,
    pub exclusive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Network limits.
pub struct NetworkLimits {
    pub ingress_bytes_per_sec: u64,
    pub egress_bytes_per_sec: u64,
    pub max_connections: usize,
    pub burst_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Disk io limits.
pub struct DiskIoLimits {
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub read_iops: u32,
    pub write_iops: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu: CpuLimits {
                cores: 1.0,
                max_percent: 100.0,
                scheduling_priority: 0,
            },
            memory: MemoryLimits {
                max_bytes: 1024 * 1024 * 1024, // 1GB
                max_heap_bytes: None,
                max_stack_bytes: None,
                swap_limit_bytes: None,
            },
            gpu: None,
            network: NetworkLimits {
                ingress_bytes_per_sec: 10 * 1024 * 1024, // 10MB/s
                egress_bytes_per_sec: 10 * 1024 * 1024,
                max_connections: 1000,
                burst_size: None,
            },
            disk_io: DiskIoLimits {
                read_bytes_per_sec: 100 * 1024 * 1024, // 100MB/s
                write_bytes_per_sec: 100 * 1024 * 1024,
                read_iops: 10000,
                write_iops: 10000,
            },
        }
    }
}

// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Resource usage.
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_bytes: usize,
    pub gpu_memory_bytes: Option<usize>,
    pub gpu_compute_percent: Option<f32>,
    pub network_ingress_bytes: u64,
    pub network_egress_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub timestamp: std::time::SystemTime,
}

// Resource controller trait
/// Controller trait for resource management.
pub trait ResourceController: Send + Sync {
    fn apply_limits(&self, limits: &ResourceLimits) -> Result<(), ResourceError>;
    fn get_usage(&self) -> Result<ResourceUsage, ResourceError>;
    fn release(&self) -> Result<(), ResourceError>;
}

#[derive(Debug, thiserror::Error)]
/// Error variants for resource operations.
pub enum ResourceError {
    #[error("CPU limit error: {0}")]
    CpuError(String),
    #[error("Memory limit error: {0}")]
    MemoryError(String),
    #[error("GPU resource error: {0}")]
    GpuError(String),
    #[error("Network throttle error: {0}")]
    NetworkError(String),
    #[error("I/O throttle error: {0}")]
    IoError(String),
    #[error("Resource not available: {0}")]
    NotAvailable(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// Split implementations into include files
include!("resource_manager.rs");
include!("resource_pool.rs");
include!("resource_tests.rs");
