use super::*;

// GPU resource scheduler
pub struct GpuScheduler {
    limits: GpuLimits,
}

impl GpuScheduler {
    pub fn new(limits: GpuLimits) -> Result<Self, ResourceError> {
        Ok(Self { limits })
    }
}

impl ResourceController for GpuScheduler {
    fn apply_limits(&self, _limits: &ResourceLimits) -> Result<(), ResourceError> {
        Ok(())
    }

    fn get_usage(&self) -> Result<ResourceUsage, ResourceError> {
        Ok(ResourceUsage {
            cpu_percent: 0.0,
            memory_bytes: 0,
            gpu_memory_bytes: Some(0),
            gpu_compute_percent: Some(0.0),
            network_ingress_bytes: 0,
            network_egress_bytes: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            timestamp: std::time::SystemTime::now(),
        })
    }

    fn release(&self) -> Result<(), ResourceError> {
        Ok(())
    }
}
