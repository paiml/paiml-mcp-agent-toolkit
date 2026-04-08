#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

// Disk I/O throttle
/// Io throttle.
pub struct IoThrottle {
    _limits: DiskIoLimits,
}

impl IoThrottle {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(limits: DiskIoLimits) -> Result<Self, ResourceError> {
        Ok(Self { _limits: limits })
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn create_test_disk_io_limits() -> DiskIoLimits {
        DiskIoLimits {
            read_bytes_per_sec: 100 * 1024 * 1024, // 100MB/s
            write_bytes_per_sec: 50 * 1024 * 1024, // 50MB/s
            read_iops: 10000,
            write_iops: 5000,
        }
    }

    fn create_test_resource_limits() -> ResourceLimits {
        ResourceLimits::default()
    }

    #[test]
    fn test_io_throttle_new_success() {
        let limits = create_test_disk_io_limits();
        let result = IoThrottle::new(limits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_io_throttle_new_with_zero_limits() {
        let limits = DiskIoLimits {
            read_bytes_per_sec: 0,
            write_bytes_per_sec: 0,
            read_iops: 0,
            write_iops: 0,
        };
        let result = IoThrottle::new(limits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_io_throttle_new_with_max_limits() {
        let limits = DiskIoLimits {
            read_bytes_per_sec: u64::MAX,
            write_bytes_per_sec: u64::MAX,
            read_iops: u32::MAX,
            write_iops: u32::MAX,
        };
        let result = IoThrottle::new(limits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_io_throttle_apply_limits() {
        let limits = create_test_disk_io_limits();
        let throttle = IoThrottle::new(limits).unwrap();
        let resource_limits = create_test_resource_limits();

        let result = throttle.apply_limits(&resource_limits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_io_throttle_get_usage() {
        let limits = create_test_disk_io_limits();
        let throttle = IoThrottle::new(limits).unwrap();

        let result = throttle.get_usage();
        assert!(result.is_ok());

        let usage = result.unwrap();
        assert_eq!(usage.cpu_percent, 0.0);
        assert_eq!(usage.memory_bytes, 0);
        assert!(usage.gpu_memory_bytes.is_none());
        assert!(usage.gpu_compute_percent.is_none());
        assert_eq!(usage.network_ingress_bytes, 0);
        assert_eq!(usage.network_egress_bytes, 0);
        assert_eq!(usage.disk_read_bytes, 0);
        assert_eq!(usage.disk_write_bytes, 0);
    }

    #[test]
    fn test_io_throttle_get_usage_has_valid_timestamp() {
        let limits = create_test_disk_io_limits();
        let throttle = IoThrottle::new(limits).unwrap();

        let before = std::time::SystemTime::now();
        let usage = throttle.get_usage().unwrap();
        let after = std::time::SystemTime::now();

        assert!(usage.timestamp >= before);
        assert!(usage.timestamp <= after);
    }

    #[test]
    fn test_io_throttle_release() {
        let limits = create_test_disk_io_limits();
        let throttle = IoThrottle::new(limits).unwrap();

        let result = throttle.release();
        assert!(result.is_ok());
    }

    #[test]
    fn test_io_throttle_multiple_operations() {
        let limits = create_test_disk_io_limits();
        let throttle = IoThrottle::new(limits).unwrap();

        // Should be able to call operations multiple times
        for _ in 0..10 {
            assert!(throttle.get_usage().is_ok());
        }
    }

    #[test]
    fn test_io_throttle_apply_different_limits() {
        let limits = create_test_disk_io_limits();
        let throttle = IoThrottle::new(limits).unwrap();

        // Apply with different resource limits configurations
        let mut resource_limits = create_test_resource_limits();
        resource_limits.disk_io.read_bytes_per_sec = 200 * 1024 * 1024;

        let result = throttle.apply_limits(&resource_limits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_io_throttle_release_then_get_usage() {
        let limits = create_test_disk_io_limits();
        let throttle = IoThrottle::new(limits).unwrap();

        // Release first, then get usage (should still work)
        assert!(throttle.release().is_ok());
        assert!(throttle.get_usage().is_ok());
    }
}
