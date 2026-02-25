#[cfg_attr(coverage_nightly, coverage(off))]
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

    #[test]
    fn test_cpu_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.cpu.cores, 1.0);
        assert_eq!(limits.cpu.max_percent, 100.0);
        assert_eq!(limits.cpu.scheduling_priority, 0);
    }

    #[test]
    fn test_memory_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.memory.max_bytes, 1024 * 1024 * 1024);
        assert!(limits.memory.max_heap_bytes.is_none());
        assert!(limits.memory.max_stack_bytes.is_none());
        assert!(limits.memory.swap_limit_bytes.is_none());
    }

    #[test]
    fn test_network_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.network.ingress_bytes_per_sec, 10 * 1024 * 1024);
        assert_eq!(limits.network.egress_bytes_per_sec, 10 * 1024 * 1024);
        assert_eq!(limits.network.max_connections, 1000);
        assert!(limits.network.burst_size.is_none());
    }

    #[test]
    fn test_disk_io_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.disk_io.read_bytes_per_sec, 100 * 1024 * 1024);
        assert_eq!(limits.disk_io.write_bytes_per_sec, 100 * 1024 * 1024);
        assert_eq!(limits.disk_io.read_iops, 10000);
        assert_eq!(limits.disk_io.write_iops, 10000);
    }

    #[test]
    fn test_resource_pool_insufficient_cpu() {
        let total = ResourceLimits::default();
        let pool = ResourcePool::new(total);

        let agent1 = uuid::Uuid::new_v4();
        let mut requested = ResourceLimits::default();
        requested.cpu.cores = 2.0; // More than available

        let result = pool.request(agent1, requested);
        assert!(result.is_err());
        match result {
            Err(ResourceError::NotAvailable(msg)) => {
                assert!(msg.contains("CPU"));
            }
            _ => panic!("Expected NotAvailable error"),
        }
    }

    #[test]
    fn test_resource_pool_insufficient_memory() {
        let total = ResourceLimits::default();
        let pool = ResourcePool::new(total);

        let agent1 = uuid::Uuid::new_v4();
        let mut requested = ResourceLimits::default();
        requested.cpu.cores = 0.5;
        requested.memory.max_bytes = 2 * 1024 * 1024 * 1024; // More than available

        let result = pool.request(agent1, requested);
        assert!(result.is_err());
        match result {
            Err(ResourceError::NotAvailable(msg)) => {
                assert!(msg.contains("memory"));
            }
            _ => panic!("Expected NotAvailable error"),
        }
    }

    #[test]
    fn test_resource_pool_multiple_agents() {
        let total = ResourceLimits::default();
        let pool = ResourcePool::new(total);

        let agent1 = uuid::Uuid::new_v4();
        let agent2 = uuid::Uuid::new_v4();

        let mut requested1 = ResourceLimits::default();
        requested1.cpu.cores = 0.3;
        requested1.memory.max_bytes = 256 * 1024 * 1024;

        let mut requested2 = ResourceLimits::default();
        requested2.cpu.cores = 0.4;
        requested2.memory.max_bytes = 256 * 1024 * 1024;

        pool.request(agent1, requested1).unwrap();
        pool.request(agent2, requested2).unwrap();

        let available = pool.get_available();
        assert!((available.cpu.cores - 0.3).abs() < 0.001);

        let allocated = pool.get_allocated();
        assert_eq!(allocated.len(), 2);
    }

    #[test]
    fn test_resource_pool_release_unknown_agent() {
        let total = ResourceLimits::default();
        let pool = ResourcePool::new(total);

        let unknown_agent = uuid::Uuid::new_v4();
        let result = pool.release(unknown_agent);
        // Should succeed even if agent is not found
        assert!(result.is_ok());
    }

    #[test]
    fn test_resource_limits_clone() {
        let limits = ResourceLimits::default();
        let cloned = limits.clone();
        assert_eq!(limits.cpu.cores, cloned.cpu.cores);
        assert_eq!(limits.memory.max_bytes, cloned.memory.max_bytes);
    }

    #[test]
    fn test_gpu_limits_creation() {
        let gpu_limits = GpuLimits {
            device_id: 0,
            memory_bytes: 8 * 1024 * 1024 * 1024,
            compute_percent: 50.0,
            exclusive: false,
        };
        assert_eq!(gpu_limits.device_id, 0);
        assert_eq!(gpu_limits.compute_percent, 50.0);
    }

    #[test]
    fn test_resource_limits_with_gpu() {
        let mut limits = ResourceLimits::default();
        limits.gpu = Some(GpuLimits {
            device_id: 0,
            memory_bytes: 4 * 1024 * 1024 * 1024,
            compute_percent: 100.0,
            exclusive: true,
        });
        assert!(limits.gpu.is_some());
        let gpu = limits.gpu.unwrap();
        assert!(gpu.exclusive);
    }

    #[test]
    fn test_resource_error_display() {
        let err = ResourceError::CpuError("test cpu error".to_string());
        assert!(err.to_string().contains("CPU"));

        let err = ResourceError::MemoryError("test memory error".to_string());
        assert!(err.to_string().contains("Memory"));

        let err = ResourceError::GpuError("test gpu error".to_string());
        assert!(err.to_string().contains("GPU"));

        let err = ResourceError::NetworkError("test network error".to_string());
        assert!(err.to_string().contains("Network"));

        let err = ResourceError::IoError("test io error".to_string());
        assert!(err.to_string().contains("I/O"));

        let err = ResourceError::NotAvailable("test not available".to_string());
        assert!(err.to_string().contains("not available"));

        let err = ResourceError::PermissionDenied("test permission denied".to_string());
        assert!(err.to_string().contains("Permission"));
    }

    #[test]
    fn test_resource_usage_creation() {
        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_bytes: 512 * 1024 * 1024,
            gpu_memory_bytes: Some(1024 * 1024 * 1024),
            gpu_compute_percent: Some(25.0),
            network_ingress_bytes: 1000,
            network_egress_bytes: 2000,
            disk_read_bytes: 5000,
            disk_write_bytes: 3000,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(usage.cpu_percent, 50.0);
        assert_eq!(usage.memory_bytes, 512 * 1024 * 1024);
        assert!(usage.gpu_memory_bytes.is_some());
    }

    #[test]
    fn test_resource_limits_serialization() {
        let limits = ResourceLimits::default();
        let json = serde_json::to_string(&limits).unwrap();
        assert!(json.contains("cpu"));
        assert!(json.contains("memory"));
        assert!(json.contains("network"));
        assert!(json.contains("disk_io"));
    }

    #[test]
    fn test_resource_limits_deserialization() {
        let json = r#"{
            "cpu": {"cores": 2.0, "max_percent": 80.0, "scheduling_priority": -5},
            "memory": {"max_bytes": 2147483648},
            "gpu": null,
            "network": {"ingress_bytes_per_sec": 20971520, "egress_bytes_per_sec": 10485760, "max_connections": 500},
            "disk_io": {"read_bytes_per_sec": 52428800, "write_bytes_per_sec": 52428800, "read_iops": 5000, "write_iops": 5000}
        }"#;
        let limits: ResourceLimits = serde_json::from_str(json).unwrap();
        assert_eq!(limits.cpu.cores, 2.0);
        assert_eq!(limits.cpu.max_percent, 80.0);
        assert_eq!(limits.cpu.scheduling_priority, -5);
    }
}
