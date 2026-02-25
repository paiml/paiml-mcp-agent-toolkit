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
