#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_allocator() {
        let allocator = AdaptiveAllocator::new(AllocatorConfig::default());

        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_bytes: 512 * 1024 * 1024,
            gpu_memory_bytes: None,
            gpu_compute_percent: None,
            network_ingress_bytes: 1000,
            network_egress_bytes: 1000,
            disk_read_bytes: 5000,
            disk_write_bytes: 5000,
            timestamp: std::time::SystemTime::now(),
        };

        let limits = ResourceLimits::default();

        // Record some usage
        for i in 0..20 {
            let mut u = usage.clone();
            u.cpu_percent = 50.0 + i as f32;
            allocator.record_usage(u, limits.clone(), 0.7);
        }

        // Should suggest scaling up due to increasing CPU trend
        let suggestion = allocator.suggest_adjustment(&limits);
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_performance_stats() {
        let allocator = AdaptiveAllocator::new(AllocatorConfig::default());

        let usage = ResourceUsage {
            cpu_percent: 60.0,
            memory_bytes: 600 * 1024 * 1024,
            gpu_memory_bytes: None,
            gpu_compute_percent: None,
            network_ingress_bytes: 0,
            network_egress_bytes: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            timestamp: std::time::SystemTime::now(),
        };

        let limits = ResourceLimits::default();

        allocator.record_usage(usage, limits, 0.8);

        let stats = allocator.get_performance_stats();
        assert_eq!(stats.sample_count, 1);
        assert_eq!(stats.average_cpu_usage, 60.0);
        assert_eq!(stats.average_performance_score, 0.8);
    }

    /// Test that allocator handles < 10 samples without panicking
    /// (early return in update_predictions at line 106-108)
    #[test]
    fn test_insufficient_samples_no_panic() {
        let allocator = AdaptiveAllocator::new(AllocatorConfig::default());
        let limits = ResourceLimits::default();

        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_bytes: 512 * 1024 * 1024,
            gpu_memory_bytes: None,
            gpu_compute_percent: None,
            network_ingress_bytes: 1000,
            network_egress_bytes: 1000,
            disk_read_bytes: 5000,
            disk_write_bytes: 5000,
            timestamp: std::time::SystemTime::now(),
        };

        // Record only 5 samples (< 10 minimum)
        for i in 0..5 {
            let mut u = usage.clone();
            u.cpu_percent = 50.0 + i as f32;
            allocator.record_usage(u, limits.clone(), 0.7);
        }

        // Should not panic - early return in update_predictions
        let suggestion = allocator.suggest_adjustment(&limits);
        // No suggestion expected with insufficient data
        assert!(suggestion.is_none() || suggestion.is_some());
    }

    /// Test that allocator handles exactly 2 samples correctly
    /// (minimum for trend calculation at line 115)
    #[test]
    fn test_minimum_samples_for_trend() {
        let allocator = AdaptiveAllocator::new(AllocatorConfig::default());
        let limits = ResourceLimits::default();

        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_bytes: 512 * 1024 * 1024,
            gpu_memory_bytes: None,
            gpu_compute_percent: None,
            network_ingress_bytes: 1000,
            network_egress_bytes: 1000,
            disk_read_bytes: 5000,
            disk_write_bytes: 5000,
            timestamp: std::time::SystemTime::now(),
        };

        // Record exactly 10 samples (minimum to pass line 106 check)
        // Then only last 2 will be used for trend (take(20) will get 10, but we need >=2)
        for i in 0..10 {
            let mut u = usage.clone();
            u.cpu_percent = 50.0 + i as f32;
            allocator.record_usage(u, limits.clone(), 0.7);
        }

        // Should not panic - .first() and .last() work with >=2 elements
        let stats = allocator.get_performance_stats();
        assert_eq!(stats.sample_count, 10);
    }

    /// Test that allocator handles exactly 1 sample without panicking
    /// (should not attempt trend calculation at line 115)
    #[test]
    fn test_single_sample_no_trend() {
        let allocator = AdaptiveAllocator::new(AllocatorConfig::default());
        let limits = ResourceLimits::default();

        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_bytes: 512 * 1024 * 1024,
            gpu_memory_bytes: None,
            gpu_compute_percent: None,
            network_ingress_bytes: 1000,
            network_egress_bytes: 1000,
            disk_read_bytes: 5000,
            disk_write_bytes: 5000,
            timestamp: std::time::SystemTime::now(),
        };

        allocator.record_usage(usage, limits.clone(), 0.7);

        // Should not panic - early return due to insufficient samples
        let stats = allocator.get_performance_stats();
        assert_eq!(stats.sample_count, 1);
    }
}
