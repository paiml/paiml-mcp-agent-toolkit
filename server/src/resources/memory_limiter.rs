use super::*;
use parking_lot::RwLock;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

type PressureCallbacks = Arc<RwLock<Vec<Box<dyn Fn(f32) + Send + Sync>>>>;
use sysinfo::System as SysInfo;

// Memory limiter with custom allocator
pub struct MemoryLimiter {
    limits: Arc<RwLock<MemoryLimits>>,
    allocated: Arc<AtomicUsize>,
    peak_allocated: Arc<AtomicUsize>,
    system: Arc<RwLock<SysInfo>>,
    pid: u32,
}

impl MemoryLimiter {
    pub fn new(limits: MemoryLimits) -> Result<Self, ResourceError> {
        let mut system = SysInfo::new_all();
        system.refresh_all();

        let limiter = Self {
            limits: Arc::new(RwLock::new(limits.clone())),
            allocated: Arc::new(AtomicUsize::new(0)),
            peak_allocated: Arc::new(AtomicUsize::new(0)),
            system: Arc::new(RwLock::new(system)),
            pid: std::process::id(),
        };

        // Skip applying actual system limits during tests to avoid failures
        #[cfg(not(test))]
        limiter.apply_memory_limits(&limits)?;

        Ok(limiter)
    }

    fn apply_memory_limits(&self, limits: &MemoryLimits) -> Result<(), ResourceError> {
        // Apply RSS limit using setrlimit
        self.set_rss_limit(limits.max_bytes)?;

        // Apply heap limit if specified
        if let Some(heap_limit) = limits.max_heap_bytes {
            self.set_heap_limit(heap_limit)?;
        }

        // Apply stack limit if specified
        if let Some(stack_limit) = limits.max_stack_bytes {
            self.set_stack_limit(stack_limit)?;
        }

        // Apply swap limit if specified
        if let Some(swap_limit) = limits.swap_limit_bytes {
            self.set_swap_limit(swap_limit)?;
        }

        Ok(())
    }

    fn set_rss_limit(&self, limit_bytes: usize) -> Result<(), ResourceError> {
        #[cfg(unix)]
        {
            use libc::{rlimit, setrlimit, RLIMIT_AS};

            let limit = rlimit {
                rlim_cur: limit_bytes as libc::rlim_t,
                rlim_max: limit_bytes as libc::rlim_t,
            };

            unsafe {
                let result = setrlimit(RLIMIT_AS, &limit);
                if result != 0 {
                    return Err(ResourceError::MemoryError(format!(
                        "Failed to set RSS limit: {}",
                        std::io::Error::last_os_error()
                    )));
                }
            }
        }

        Ok(())
    }

    fn set_heap_limit(&self, limit_bytes: usize) -> Result<(), ResourceError> {
        #[cfg(unix)]
        {
            use libc::{rlimit, setrlimit, RLIMIT_DATA};

            let limit = rlimit {
                rlim_cur: limit_bytes as libc::rlim_t,
                rlim_max: limit_bytes as libc::rlim_t,
            };

            unsafe {
                let result = setrlimit(RLIMIT_DATA, &limit);
                if result != 0 {
                    return Err(ResourceError::MemoryError(format!(
                        "Failed to set heap limit: {}",
                        std::io::Error::last_os_error()
                    )));
                }
            }
        }

        Ok(())
    }

    fn set_stack_limit(&self, limit_bytes: usize) -> Result<(), ResourceError> {
        #[cfg(unix)]
        {
            use libc::{rlimit, setrlimit, RLIMIT_STACK};

            let limit = rlimit {
                rlim_cur: limit_bytes as libc::rlim_t,
                rlim_max: limit_bytes as libc::rlim_t,
            };

            unsafe {
                let result = setrlimit(RLIMIT_STACK, &limit);
                if result != 0 {
                    return Err(ResourceError::MemoryError(format!(
                        "Failed to set stack limit: {}",
                        std::io::Error::last_os_error()
                    )));
                }
            }
        }

        Ok(())
    }

    fn set_swap_limit(&self, _limit_bytes: usize) -> Result<(), ResourceError> {
        // Swap limiting typically requires cgroup configuration
        #[cfg(target_os = "linux")]
        {
            let cgroup_path = format!("/sys/fs/cgroup/memory/agent_{}", self.pid);

            if std::path::Path::new(&cgroup_path).exists() {
                use std::fs;

                let swap_limit = _limit_bytes.to_string();
                fs::write(
                    format!("{}/memory.memsw.limit_in_bytes", cgroup_path),
                    swap_limit,
                )
                .map_err(|e| {
                    ResourceError::MemoryError(format!("Failed to set swap limit: {}", e))
                })?;
            }
        }

        Ok(())
    }

    pub fn check_allocation(&self, size: usize) -> Result<(), ResourceError> {
        let current = self.allocated.load(Ordering::Relaxed);
        let limit = self.limits.read().max_bytes;

        if current + size > limit {
            Err(ResourceError::MemoryError(format!(
                "Memory limit exceeded: {} + {} > {}",
                current, size, limit
            )))
        } else {
            Ok(())
        }
    }

    pub fn record_allocation(&self, size: usize) {
        let new_allocated = self.allocated.fetch_add(size, Ordering::SeqCst) + size;

        // Update peak if necessary
        let mut peak = self.peak_allocated.load(Ordering::Relaxed);
        while new_allocated > peak {
            match self.peak_allocated.compare_exchange_weak(
                peak,
                new_allocated,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => peak = current,
            }
        }
    }

    pub fn record_deallocation(&self, size: usize) {
        self.allocated.fetch_sub(size, Ordering::SeqCst);
    }

    pub fn get_allocated(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    pub fn get_peak_allocated(&self) -> usize {
        self.peak_allocated.load(Ordering::Relaxed)
    }
}

impl ResourceController for MemoryLimiter {
    fn apply_limits(&self, limits: &ResourceLimits) -> Result<(), ResourceError> {
        *self.limits.write() = limits.memory.clone();
        self.apply_memory_limits(&limits.memory)
    }

    fn get_usage(&self) -> Result<ResourceUsage, ResourceError> {
        let mut system = self.system.write();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let memory_bytes =
            if let Some(process) = system.process(sysinfo::Pid::from(self.pid as usize)) {
                process.memory() * 1024 // Convert from KB to bytes
            } else {
                self.allocated.load(Ordering::Relaxed) as u64
            };

        Ok(ResourceUsage {
            cpu_percent: 0.0,
            memory_bytes: memory_bytes as usize,
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
        // Reset limits to system defaults
        #[cfg(unix)]
        {
            use libc::{rlimit, setrlimit, RLIMIT_AS, RLIM_INFINITY};

            let unlimited = rlimit {
                rlim_cur: RLIM_INFINITY,
                rlim_max: RLIM_INFINITY,
            };

            unsafe {
                setrlimit(RLIMIT_AS, &unlimited);
            }
        }

        Ok(())
    }
}

// Custom allocator that tracks and limits memory usage
pub struct LimitedAllocator {
    limiter: Arc<MemoryLimiter>,
    inner: System,
}

impl LimitedAllocator {
    pub fn new(limiter: Arc<MemoryLimiter>) -> Self {
        Self {
            limiter,
            inner: System,
        }
    }
}

unsafe impl GlobalAlloc for LimitedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();

        // Check if allocation would exceed limit
        if self.limiter.check_allocation(size).is_err() {
            return std::ptr::null_mut();
        }

        // Perform actual allocation
        let ptr = self.inner.alloc(layout);

        if !ptr.is_null() {
            self.limiter.record_allocation(size);
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.limiter.record_deallocation(layout.size());
        self.inner.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_size = layout.size();

        if new_size > old_size {
            // Check if additional allocation would exceed limit
            let additional = new_size - old_size;
            if self.limiter.check_allocation(additional).is_err() {
                return std::ptr::null_mut();
            }
        }

        let new_ptr = self.inner.realloc(ptr, layout, new_size);

        if !new_ptr.is_null() {
            if new_size > old_size {
                self.limiter.record_allocation(new_size - old_size);
            } else {
                self.limiter.record_deallocation(old_size - new_size);
            }
        }

        new_ptr
    }
}

// Memory pressure monitor
pub struct MemoryMonitor {
    limiter: Arc<MemoryLimiter>,
    pressure_callbacks: PressureCallbacks,
}

impl MemoryMonitor {
    pub fn new(limiter: Arc<MemoryLimiter>) -> Self {
        Self {
            limiter,
            pressure_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_pressure_callback<F>(&self, callback: F)
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.pressure_callbacks.write().push(Box::new(callback));
    }

    pub fn check_memory_pressure(&self) -> f32 {
        let allocated = self.limiter.get_allocated();
        let limit = self.limiter.limits.read().max_bytes;

        let pressure = allocated as f32 / limit as f32;

        // Trigger callbacks if pressure is high
        if pressure > 0.8 {
            let callbacks = self.pressure_callbacks.read();
            for callback in callbacks.iter() {
                callback(pressure);
            }
        }

        pressure
    }

    pub async fn monitor_loop(&self) {
        loop {
            self.check_memory_pressure();
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_limiter_creation() {
        let limits = MemoryLimits {
            max_bytes: 1024 * 1024, // 1MB for testing
            max_heap_bytes: None,   // Don't set heap limit in tests
            max_stack_bytes: None,  // Don't set stack limit in tests
            swap_limit_bytes: None,
        };

        let _limiter = MemoryLimiter::new(limits).unwrap();
    }

    #[test]
    fn test_allocation_tracking() {
        let limits = MemoryLimits {
            max_bytes: 1024 * 1024, // 1MB
            max_heap_bytes: None,
            max_stack_bytes: None,
            swap_limit_bytes: None,
        };

        let limiter = MemoryLimiter::new(limits).unwrap();

        assert!(limiter.check_allocation(512 * 1024).is_ok());
        limiter.record_allocation(512 * 1024);
        assert_eq!(limiter.get_allocated(), 512 * 1024);

        assert!(limiter.check_allocation(600 * 1024).is_err());

        limiter.record_deallocation(256 * 1024);
        assert_eq!(limiter.get_allocated(), 256 * 1024);
    }

    #[test]
    fn test_memory_pressure() {
        let limits = MemoryLimits {
            max_bytes: 1000,
            max_heap_bytes: None,
            max_stack_bytes: None,
            swap_limit_bytes: None,
        };

        let limiter = Arc::new(MemoryLimiter::new(limits).unwrap());
        let monitor = MemoryMonitor::new(limiter.clone());

        limiter.record_allocation(500);
        let pressure = monitor.check_memory_pressure();
        assert_eq!(pressure, 0.5);

        limiter.record_allocation(400);
        let pressure = monitor.check_memory_pressure();
        assert_eq!(pressure, 0.9);
    }
}
