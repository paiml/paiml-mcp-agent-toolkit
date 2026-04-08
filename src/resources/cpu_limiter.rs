#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

// CPU resource limiter using cgroups v2 and affinity
/// Limiter for controlling cpu usage.
pub struct CpuLimiter {
    limits: Arc<RwLock<CpuLimits>>,
    system: Arc<RwLock<System>>,
    pid: u32,
    original_affinity: Option<Vec<usize>>,
    _monitor_handle: Option<thread::JoinHandle<()>>,
    shutdown: Arc<RwLock<bool>>,
}

impl CpuLimiter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(limits: CpuLimits) -> Result<Self, ResourceError> {
        let mut system = System::new_all();
        system.refresh_all();

        let pid = std::process::id();

        let limiter = Self {
            limits: Arc::new(RwLock::new(limits.clone())),
            system: Arc::new(RwLock::new(system)),
            pid,
            original_affinity: Self::get_current_affinity()?,
            _monitor_handle: None,
            shutdown: Arc::new(RwLock::new(false)),
        };

        limiter.apply_cpu_limits(&limits)?;

        Ok(limiter)
    }

    fn apply_cpu_limits(&self, limits: &CpuLimits) -> Result<(), ResourceError> {
        // Apply CPU affinity based on core count
        self.set_cpu_affinity(limits.cores)?;

        // Set scheduling priority (nice value)
        self.set_scheduling_priority(limits.scheduling_priority)?;

        // Apply cgroup CPU limits if available
        if self.is_cgroup_available() {
            self.apply_cgroup_limits(limits)?;
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn set_cpu_affinity(&self, cores: f32) -> Result<(), ResourceError> {
        #[cfg(target_os = "linux")]
        {
            use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
            use std::mem;

            let num_cores = (cores.ceil() as usize).min(num_cpus::get());

            // SAFETY: Setting CPU affinity via libc system calls.
            // This is safe because:
            // 1. mem::zeroed() creates a valid zero-initialized cpu_set_t struct
            // 2. CPU_ZERO and CPU_SET are well-defined libc macros for cpu_set_t manipulation
            // 3. sched_setaffinity is a standard POSIX system call with proper error handling
            // 4. We validate the return code and propagate errors appropriately
            unsafe {
                let mut set: cpu_set_t = mem::zeroed();
                CPU_ZERO(&mut set);

                for i in 0..num_cores {
                    CPU_SET(i, &mut set);
                }

                let result = sched_setaffinity(self.pid as i32, mem::size_of::<cpu_set_t>(), &set);

                if result != 0 {
                    return Err(ResourceError::CpuError(format!(
                        "Failed to set CPU affinity: {}",
                        std::io::Error::last_os_error()
                    )));
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // CPU affinity not supported on this platform
        }

        Ok(())
    }

    fn get_current_affinity() -> Result<Option<Vec<usize>>, ResourceError> {
        #[cfg(target_os = "linux")]
        {
            use libc::{cpu_set_t, sched_getaffinity, CPU_ISSET};
            use std::mem;

            // SAFETY: Getting current CPU affinity via libc system calls.
            // This is safe because:
            // 1. mem::zeroed() creates a valid zero-initialized cpu_set_t struct
            // 2. sched_getaffinity is a standard POSIX system call that fills the cpu_set_t
            // 3. CPU_ISSET is a well-defined libc macro for reading cpu_set_t
            // 4. We validate the return code and handle errors by returning None
            unsafe {
                let mut set: cpu_set_t = mem::zeroed();
                let result = sched_getaffinity(0, mem::size_of::<cpu_set_t>(), &mut set);

                if result != 0 {
                    return Ok(None);
                }

                let mut cores = Vec::new();
                for i in 0..CPU_SETSIZE {
                    if CPU_ISSET(i, &set) {
                        cores.push(i);
                    }
                }

                Ok(Some(cores))
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(None)
        }
    }

    fn set_scheduling_priority(&self, priority: i32) -> Result<(), ResourceError> {
        #[cfg(unix)]
        {
            use libc::setpriority;

            let nice = priority.max(-20).min(19);

            // SAFETY: Setting process priority via libc system call.
            // This is safe because:
            // 1. setpriority is a standard POSIX system call
            // 2. The nice value is clamped to valid range [-20, 19]
            // 3. We validate the return code and propagate errors appropriately
            // 4. PRIO_PROCESS and self.pid are valid parameters
            unsafe {
                let result = setpriority(libc::PRIO_PROCESS, self.pid, nice);
                if result != 0 {
                    return Err(ResourceError::CpuError(format!(
                        "Failed to set priority: {}",
                        std::io::Error::last_os_error()
                    )));
                }
            }
        }

        Ok(())
    }

    fn is_cgroup_available(&self) -> bool {
        batuta_common::sys::is_cgroup_available()
    }

    fn apply_cgroup_limits(&self, _limits: &CpuLimits) -> Result<(), ResourceError> {
        #[cfg(target_os = "linux")]
        {
            // Try cgroups v2 first
            if std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists() {
                self.apply_cgroup_v2_limits(_limits)?;
            }
            // Fall back to cgroups v1
            else if std::path::Path::new("/sys/fs/cgroup/cpu").exists() {
                self.apply_cgroup_v1_limits(_limits)?;
            }
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn apply_cgroup_v2_limits(&self, limits: &CpuLimits) -> Result<(), ResourceError> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;

            let cgroup_path = format!("/sys/fs/cgroup/agent_{}", self.pid);

            // Create cgroup if it doesn't exist
            if !std::path::Path::new(&cgroup_path).exists() {
                fs::create_dir(&cgroup_path).map_err(|e| {
                    ResourceError::CpuError(format!("Failed to create cgroup: {}", e))
                })?;
            }

            // Set CPU max (quota and period in microseconds)
            let period_us = 100000; // 100ms
            let quota_us = ((limits.max_percent / 100.0) * period_us as f32) as u64;

            let cpu_max = format!("{} {}", quota_us, period_us);
            fs::write(format!("{}/cpu.max", cgroup_path), cpu_max)
                .map_err(|e| ResourceError::CpuError(format!("Failed to set cpu.max: {}", e)))?;

            // Add current process to cgroup
            fs::write(
                format!("{}/cgroup.procs", cgroup_path),
                self.pid.to_string(),
            )
            .map_err(|e| {
                ResourceError::CpuError(format!("Failed to add process to cgroup: {}", e))
            })?;
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn apply_cgroup_v1_limits(&self, limits: &CpuLimits) -> Result<(), ResourceError> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;

            let cgroup_path = format!("/sys/fs/cgroup/cpu/agent_{}", self.pid);

            // Create cgroup if it doesn't exist
            if !std::path::Path::new(&cgroup_path).exists() {
                fs::create_dir(&cgroup_path).map_err(|e| {
                    ResourceError::CpuError(format!("Failed to create cgroup: {}", e))
                })?;
            }

            // Set CPU quota
            let period_us = 100000; // 100ms
            let quota_us = ((limits.max_percent / 100.0) * period_us as f32) as i64;

            fs::write(
                format!("{}/cpu.cfs_period_us", cgroup_path),
                period_us.to_string(),
            )
            .map_err(|e| ResourceError::CpuError(format!("Failed to set period: {}", e)))?;

            fs::write(
                format!("{}/cpu.cfs_quota_us", cgroup_path),
                quota_us.to_string(),
            )
            .map_err(|e| ResourceError::CpuError(format!("Failed to set quota: {}", e)))?;

            // Add current process to cgroup
            fs::write(format!("{}/tasks", cgroup_path), self.pid.to_string()).map_err(|e| {
                ResourceError::CpuError(format!("Failed to add task to cgroup: {}", e))
            })?;
        }

        Ok(())
    }

    fn start_monitor(&mut self) {
        let limits = self.limits.clone();
        let system = self.system.clone();
        let pid = self.pid;
        let shutdown = self.shutdown.clone();

        self._monitor_handle = Some(thread::spawn(move || {
            let mut last_check = Instant::now();

            loop {
                if *shutdown.read() {
                    break;
                }

                thread::sleep(Duration::from_millis(100));

                // Refresh system info periodically
                if last_check.elapsed() > Duration::from_secs(1) {
                    let mut sys = system.write();
                    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                    sys.refresh_cpu_all();
                    last_check = Instant::now();

                    // Check if we're exceeding limits
                    if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
                        let cpu_usage = process.cpu_usage();
                        let max_cpu = limits.read().max_percent;

                        if cpu_usage > max_cpu {
                            // Throttle by sleeping
                            let sleep_ratio = (cpu_usage - max_cpu) / 100.0;
                            let sleep_duration =
                                Duration::from_millis((sleep_ratio * 100.0) as u64);
                            thread::sleep(sleep_duration);
                        }
                    }
                }
            }
        }));
    }
}

impl ResourceController for CpuLimiter {
    fn apply_limits(&self, limits: &ResourceLimits) -> Result<(), ResourceError> {
        *self.limits.write() = limits.cpu.clone();
        self.apply_cpu_limits(&limits.cpu)
    }

    fn get_usage(&self) -> Result<ResourceUsage, ResourceError> {
        let mut system = self.system.write();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        system.refresh_cpu_all();

        let cpu_usage = if let Some(process) = system.process(sysinfo::Pid::from(self.pid as usize))
        {
            process.cpu_usage()
        } else {
            0.0
        };

        Ok(ResourceUsage {
            cpu_percent: cpu_usage,
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
        *self.shutdown.write() = true;

        // Restore original CPU affinity
        #[cfg(target_os = "linux")]
        {
            if let Some(cores) = &self.original_affinity {
                use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
                use std::mem;

                // SAFETY: Restoring original CPU affinity via libc system calls.
                // This is safe because:
                // 1. mem::zeroed() creates a valid zero-initialized cpu_set_t struct
                // 2. CPU_ZERO and CPU_SET are well-defined libc macros
                // 3. sched_setaffinity is a standard POSIX system call
                // 4. Cores are from original_affinity which was saved during initialization
                // 5. Error handling is intentionally omitted in Drop (best-effort cleanup)
                unsafe {
                    let mut set: cpu_set_t = mem::zeroed();
                    CPU_ZERO(&mut set);

                    for &core in cores {
                        CPU_SET(core, &mut set);
                    }

                    sched_setaffinity(self.pid as i32, mem::size_of::<cpu_set_t>(), &set);
                }
            }
        }

        // Remove from cgroup
        #[cfg(target_os = "linux")]
        {
            let cgroup_path = format!("/sys/fs/cgroup/agent_{}", self.pid);
            let _ = std::fs::remove_dir(&cgroup_path);

            let cgroup_v1_path = format!("/sys/fs/cgroup/cpu/agent_{}", self.pid);
            let _ = std::fs::remove_dir(&cgroup_v1_path);
        }

        Ok(())
    }
}

impl Drop for CpuLimiter {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

// CPU SETSIZE constant for Linux
#[cfg(target_os = "linux")]
const CPU_SETSIZE: usize = 1024;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_limiter_creation() {
        let limits = CpuLimits {
            cores: 1.0,
            max_percent: 50.0,
            scheduling_priority: 0,
        };

        // This may fail in test environment without proper permissions
        let _limiter = CpuLimiter::new(limits);
    }

    #[test]
    fn test_cpu_usage_monitoring() {
        let limits = CpuLimits {
            cores: 1.0,
            max_percent: 100.0,
            scheduling_priority: 0,
        };

        if let Ok(limiter) = CpuLimiter::new(limits) {
            let usage = limiter.get_usage().unwrap();
            assert!(usage.cpu_percent >= 0.0);
            assert!(usage.cpu_percent <= 100.0 * num_cpus::get() as f32);
        }
    }
}
