use super::*;

// Disk I/O throttle
pub struct IoThrottle {
    limits: DiskIoLimits,
}

impl IoThrottle {
    pub fn new(limits: DiskIoLimits) -> Result<Self, ResourceError> {
        Ok(Self { limits })
    }
}

impl ResourceController for IoThrottle {
    fn apply_limits(&self, _limits: &ResourceLimits) -> Result<(), ResourceError> {
        Ok(())
    }
    
    fn get_usage(&self) -> Result<ResourceUsage, ResourceError> {
        Ok(ResourceUsage {
            cpu_percent: 0.0,
            memory_bytes: 0,
            gpu_memory_bytes: None,
            gpu_compute_percent: None,
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