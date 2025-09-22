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
pub struct ResourceLimits {
    pub cpu: CpuLimits,
    pub memory: MemoryLimits,
    pub gpu: Option<GpuLimits>,
    pub network: NetworkLimits,
    pub disk_io: DiskIoLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuLimits {
    pub cores: f32,               // Fractional cores (e.g., 1.5)
    pub max_percent: f32,         // Max CPU percentage
    pub scheduling_priority: i32, // Nice value (-20 to 19)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    pub max_bytes: usize,
    pub max_heap_bytes: Option<usize>,
    pub max_stack_bytes: Option<usize>,
    pub swap_limit_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuLimits {
    pub device_id: u32,
    pub memory_bytes: usize,
    pub compute_percent: f32,
    pub exclusive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLimits {
    pub ingress_bytes_per_sec: u64,
    pub egress_bytes_per_sec: u64,
    pub max_connections: usize,
    pub burst_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub trait ResourceController: Send + Sync {
    fn apply_limits(&self, limits: &ResourceLimits) -> Result<(), ResourceError>;
    fn get_usage(&self) -> Result<ResourceUsage, ResourceError>;
    fn release(&self) -> Result<(), ResourceError>;
}

// Resource manager coordinating all resource controllers
pub struct ResourceManager {
    limits: Arc<RwLock<ResourceLimits>>,
    cpu_controller: Arc<dyn ResourceController>,
    memory_controller: Arc<dyn ResourceController>,
    gpu_controller: Option<Arc<dyn ResourceController>>,
    network_controller: Arc<dyn ResourceController>,
    io_controller: Arc<dyn ResourceController>,
    usage_history: Arc<RwLock<Vec<ResourceUsage>>>,
}

impl ResourceManager {
    pub fn new(limits: ResourceLimits) -> Result<Self, ResourceError> {
        use cpu_limiter::CpuLimiter;
        use io_throttle::IoThrottle;
        use memory_limiter::MemoryLimiter;
        use network_throttle::NetworkThrottle;

        let cpu_controller = Arc::new(CpuLimiter::new(limits.cpu.clone())?);
        let memory_controller = Arc::new(MemoryLimiter::new(limits.memory.clone())?);
        let network_controller = Arc::new(NetworkThrottle::new(limits.network.clone())?);
        let io_controller = Arc::new(IoThrottle::new(limits.disk_io.clone())?);

        let gpu_controller = if let Some(gpu_limits) = &limits.gpu {
            use gpu_scheduler::GpuScheduler;
            Some(Arc::new(GpuScheduler::new(gpu_limits.clone())?) as Arc<dyn ResourceController>)
        } else {
            None
        };

        Ok(Self {
            limits: Arc::new(RwLock::new(limits)),
            cpu_controller,
            memory_controller,
            gpu_controller,
            network_controller,
            io_controller,
            usage_history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub fn update_limits(&self, new_limits: ResourceLimits) -> Result<(), ResourceError> {
        self.cpu_controller.apply_limits(&new_limits)?;
        self.memory_controller.apply_limits(&new_limits)?;
        self.network_controller.apply_limits(&new_limits)?;
        self.io_controller.apply_limits(&new_limits)?;

        if let Some(gpu) = &self.gpu_controller {
            gpu.apply_limits(&new_limits)?;
        }

        *self.limits.write() = new_limits;
        Ok(())
    }

    pub fn get_current_usage(&self) -> Result<ResourceUsage, ResourceError> {
        let cpu_usage = self.cpu_controller.get_usage()?;
        let memory_usage = self.memory_controller.get_usage()?;
        let network_usage = self.network_controller.get_usage()?;
        let io_usage = self.io_controller.get_usage()?;

        let gpu_usage = if let Some(gpu) = &self.gpu_controller {
            gpu.get_usage().ok()
        } else {
            None
        };

        let usage = ResourceUsage {
            cpu_percent: cpu_usage.cpu_percent,
            memory_bytes: memory_usage.memory_bytes,
            gpu_memory_bytes: gpu_usage.as_ref().and_then(|u| u.gpu_memory_bytes),
            gpu_compute_percent: gpu_usage.and_then(|u| u.gpu_compute_percent),
            network_ingress_bytes: network_usage.network_ingress_bytes,
            network_egress_bytes: network_usage.network_egress_bytes,
            disk_read_bytes: io_usage.disk_read_bytes,
            disk_write_bytes: io_usage.disk_write_bytes,
            timestamp: std::time::SystemTime::now(),
        };

        // Store in history
        let mut history = self.usage_history.write();
        history.push(usage.clone());

        // Keep only last 1000 samples
        let history_len = history.len();
        if history_len > 1000 {
            history.drain(0..history_len - 1000);
        }

        Ok(usage)
    }

    pub fn get_usage_history(&self, duration: Duration) -> Vec<ResourceUsage> {
        let history = self.usage_history.read();
        let cutoff = std::time::SystemTime::now() - duration;

        history
            .iter()
            .filter(|u| u.timestamp >= cutoff)
            .cloned()
            .collect()
    }

    pub fn release_all(&self) -> Result<(), ResourceError> {
        self.cpu_controller.release()?;
        self.memory_controller.release()?;
        self.network_controller.release()?;
        self.io_controller.release()?;

        if let Some(gpu) = &self.gpu_controller {
            gpu.release()?;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
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

// Resource pool for sharing resources across agents
pub struct ResourcePool {
    _total_limits: ResourceLimits,
    allocated: Arc<RwLock<Vec<(uuid::Uuid, ResourceLimits)>>>,
    available: Arc<RwLock<ResourceLimits>>,
}

impl ResourcePool {
    pub fn new(total_limits: ResourceLimits) -> Self {
        Self {
            available: Arc::new(RwLock::new(total_limits.clone())),
            _total_limits: total_limits,
            allocated: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn request(
        &self,
        agent_id: uuid::Uuid,
        requested: ResourceLimits,
    ) -> Result<ResourceLimits, ResourceError> {
        let mut available = self.available.write();

        // Check if resources are available
        if requested.cpu.cores > available.cpu.cores {
            return Err(ResourceError::NotAvailable(
                "Insufficient CPU cores".to_string(),
            ));
        }
        if requested.memory.max_bytes > available.memory.max_bytes {
            return Err(ResourceError::NotAvailable(
                "Insufficient memory".to_string(),
            ));
        }

        // Allocate resources
        available.cpu.cores -= requested.cpu.cores;
        available.memory.max_bytes -= requested.memory.max_bytes;
        available.network.ingress_bytes_per_sec -= requested
            .network
            .ingress_bytes_per_sec
            .min(available.network.ingress_bytes_per_sec);
        available.network.egress_bytes_per_sec -= requested
            .network
            .egress_bytes_per_sec
            .min(available.network.egress_bytes_per_sec);

        self.allocated.write().push((agent_id, requested.clone()));

        Ok(requested)
    }

    pub fn release(&self, agent_id: uuid::Uuid) -> Result<(), ResourceError> {
        let mut allocated = self.allocated.write();
        let mut available = self.available.write();

        if let Some(pos) = allocated.iter().position(|(id, _)| *id == agent_id) {
            let (_, limits) = allocated.remove(pos);

            // Return resources to pool
            available.cpu.cores += limits.cpu.cores;
            available.memory.max_bytes += limits.memory.max_bytes;
            available.network.ingress_bytes_per_sec += limits.network.ingress_bytes_per_sec;
            available.network.egress_bytes_per_sec += limits.network.egress_bytes_per_sec;
        }

        Ok(())
    }

    pub fn get_available(&self) -> ResourceLimits {
        self.available.read().clone()
    }

    pub fn get_allocated(&self) -> Vec<(uuid::Uuid, ResourceLimits)> {
        self.allocated.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.cpu.cores, 1.0);
        assert_eq!(limits.memory.max_bytes, 1024 * 1024 * 1024);
        assert!(limits.gpu.is_none());
    }

    #[test]
    fn test_resource_pool() {
        let total = ResourceLimits::default();
        let pool = ResourcePool::new(total);

        let agent1 = uuid::Uuid::new_v4();
        let mut requested = ResourceLimits::default();
        requested.cpu.cores = 0.5;
        requested.memory.max_bytes = 512 * 1024 * 1024;

        let allocated = pool.request(agent1, requested).unwrap();
        assert_eq!(allocated.cpu.cores, 0.5);

        let available = pool.get_available();
        assert_eq!(available.cpu.cores, 0.5);

        pool.release(agent1).unwrap();
        let available = pool.get_available();
        assert_eq!(available.cpu.cores, 1.0);
    }
}
