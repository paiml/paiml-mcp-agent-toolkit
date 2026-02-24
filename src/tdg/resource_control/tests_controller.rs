#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod controller_coverage_tests {
    use super::*;

    // ============ ResourceEnforcementStats Tests ============

    #[test]
    fn test_resource_enforcement_stats_debug() {
        let stats = ResourceEnforcementStats {
            total_requests: 200,
            allowed_requests: 180,
            throttled_requests: 10,
            queued_requests: 5,
            rejected_requests: 5,
            current_active_operations: 15,
        };
        let debug = format!("{:?}", stats);
        assert!(debug.contains("ResourceEnforcementStats"));
    }

    #[test]
    fn test_resource_enforcement_stats_serialization() {
        let stats = ResourceEnforcementStats {
            total_requests: 100,
            allowed_requests: 90,
            throttled_requests: 5,
            queued_requests: 3,
            rejected_requests: 2,
            current_active_operations: 12,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: ResourceEnforcementStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_requests, 100);
        assert_eq!(deserialized.allowed_requests, 90);
    }

    #[test]
    fn test_format_diagnostic_various_scenarios() {
        // Scenario 1: All allowed
        let stats1 = ResourceEnforcementStats {
            total_requests: 50,
            allowed_requests: 50,
            throttled_requests: 0,
            queued_requests: 0,
            rejected_requests: 0,
            current_active_operations: 5,
        };
        let output1 = stats1.format_diagnostic();
        assert!(output1.contains("100.0%"));

        // Scenario 2: Mixed results
        let stats2 = ResourceEnforcementStats {
            total_requests: 100,
            allowed_requests: 70,
            throttled_requests: 15,
            queued_requests: 10,
            rejected_requests: 5,
            current_active_operations: 20,
        };
        let output2 = stats2.format_diagnostic();
        assert!(output2.contains("70.0%"));
        assert!(output2.contains("Throttled: 15"));

        // Scenario 3: High rejection rate
        let stats3 = ResourceEnforcementStats {
            total_requests: 100,
            allowed_requests: 20,
            throttled_requests: 10,
            queued_requests: 20,
            rejected_requests: 50,
            current_active_operations: 2,
        };
        let output3 = stats3.format_diagnostic();
        assert!(output3.contains("20.0%"));
        assert!(output3.contains("Rejected: 50"));
    }

    // ============ Factory Tests ============

    #[test]
    fn test_factory_dev_limits() {
        let controller = ResourceControllerFactory::create_dev_optimized();
        // Verify dev limits are actually lower
        assert_eq!(controller.limits.max_memory_mb, 512.0);
        assert_eq!(controller.limits.max_concurrent_ops, 5);
    }

    #[test]
    fn test_factory_prod_limits() {
        let controller = ResourceControllerFactory::create_prod_optimized();
        // Verify prod limits are actually higher
        assert_eq!(controller.limits.max_memory_mb, 2048.0);
        assert_eq!(controller.limits.max_concurrent_ops, 50);
    }

    #[test]
    fn test_factory_ci_limits() {
        let controller = ResourceControllerFactory::create_ci_optimized();
        // Verify CI limits
        assert_eq!(controller.limits.max_memory_mb, 1024.0);
        assert_eq!(controller.limits.max_cpu_utilization, 0.9);
    }

    // ============ PlatformResourceController Tests ============

    #[test]
    fn test_controller_new_with_custom_limits() {
        let limits = ResourceLimits {
            max_memory_mb: 4096.0,
            max_cpu_utilization: 0.95,
            max_concurrent_ops: 100,
            memory_warning_threshold: 0.5,
            cpu_warning_threshold: 0.4,
            check_interval_secs: 1,
        };
        let controller = PlatformResourceController::new(limits);
        assert_eq!(controller.limits.max_memory_mb, 4096.0);
        assert_eq!(controller.limits.max_concurrent_ops, 100);
    }

    #[tokio::test]
    async fn test_controller_get_initial_usage() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        let usage = controller.get_current_usage().await;
        assert_eq!(usage.active_operations, 0);
        assert_eq!(usage.memory_pressure, ResourcePressure::Low);
        assert_eq!(usage.cpu_pressure, ResourcePressure::Low);
    }

    #[tokio::test]
    async fn test_controller_get_enforcement_stats_empty() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        let stats = controller.get_enforcement_stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.allowed_requests, 0);
        assert_eq!(stats.current_active_operations, 0);
    }

    #[tokio::test]
    async fn test_measure_current_usage_empty_ops() {
        let limits = ResourceLimits::default();
        let active_ops = Arc::new(RwLock::new(HashMap::new()));

        let usage = PlatformResourceController::measure_current_usage(&limits, &active_ops).await;

        assert_eq!(usage.active_operations, 0);
        assert_eq!(usage.memory_mb, 100.0); // Base memory
        assert_eq!(usage.cpu_utilization, 0.0);
        assert_eq!(usage.memory_pressure, ResourcePressure::Low);
    }

    #[tokio::test]
    async fn test_measure_current_usage_with_ops() {
        let limits = ResourceLimits {
            max_memory_mb: 1000.0,
            memory_warning_threshold: 0.7,
            ..Default::default()
        };
        let active_ops = Arc::new(RwLock::new(HashMap::new()));

        // Add some operations
        {
            let mut ops = active_ops.write().await;
            ops.insert(
                "op1".to_string(),
                OperationContext {
                    id: "op1".to_string(),
                    operation_type: OperationType::Analysis,
                    started_at: Instant::now(),
                    estimated_memory_mb: 300.0,
                    priority: OperationPriority::High,
                },
            );
            ops.insert(
                "op2".to_string(),
                OperationContext {
                    id: "op2".to_string(),
                    operation_type: OperationType::Storage,
                    started_at: Instant::now(),
                    estimated_memory_mb: 200.0,
                    priority: OperationPriority::Medium,
                },
            );
        }

        let usage = PlatformResourceController::measure_current_usage(&limits, &active_ops).await;

        assert_eq!(usage.active_operations, 2);
        assert!(usage.memory_mb >= 500.0); // At least 300 + 200 + base
    }

    #[tokio::test]
    async fn test_measure_current_usage_high_memory_pressure() {
        let limits = ResourceLimits {
            max_memory_mb: 500.0,
            memory_warning_threshold: 0.7,
            ..Default::default()
        };
        let active_ops = Arc::new(RwLock::new(HashMap::new()));

        // Add operation that causes high memory
        {
            let mut ops = active_ops.write().await;
            ops.insert(
                "heavy-op".to_string(),
                OperationContext {
                    id: "heavy-op".to_string(),
                    operation_type: OperationType::Analysis,
                    started_at: Instant::now(),
                    estimated_memory_mb: 450.0, // 450 + 100 base > 500 limit
                    priority: OperationPriority::High,
                },
            );
        }

        let usage = PlatformResourceController::measure_current_usage(&limits, &active_ops).await;

        assert_eq!(usage.memory_pressure, ResourcePressure::Critical);
    }

    #[tokio::test]
    async fn test_estimate_resource_wait_time_empty() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        let wait_time = controller.estimate_resource_wait_time().await;
        assert_eq!(wait_time, 100); // Minimum wait
    }

    #[tokio::test]
    async fn test_estimate_operation_wait_time_available_permits() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        let wait_time = controller.estimate_operation_wait_time().await;
        assert_eq!(wait_time, 100); // Has available permits
    }

    #[tokio::test]
    async fn test_stop_monitoring_when_not_started() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        // Should not panic when stopping monitoring that was never started
        controller.stop_monitoring().await;
    }

    // ============ Pressure Level Calculation Tests ============

    #[tokio::test]
    async fn test_pressure_levels_low() {
        let limits = ResourceLimits {
            max_memory_mb: 1000.0,
            max_cpu_utilization: 0.8,
            memory_warning_threshold: 0.7,
            cpu_warning_threshold: 0.6,
            ..Default::default()
        };
        let active_ops = Arc::new(RwLock::new(HashMap::new()));

        // Add light operation
        {
            let mut ops = active_ops.write().await;
            ops.insert(
                "light".to_string(),
                OperationContext {
                    id: "light".to_string(),
                    operation_type: OperationType::Cleanup,
                    started_at: Instant::now(),
                    estimated_memory_mb: 50.0,
                    priority: OperationPriority::Low,
                },
            );
        }

        let usage = PlatformResourceController::measure_current_usage(&limits, &active_ops).await;
        assert_eq!(usage.memory_pressure, ResourcePressure::Low);
    }

    #[tokio::test]
    async fn test_pressure_levels_medium() {
        let limits = ResourceLimits {
            max_memory_mb: 500.0,
            memory_warning_threshold: 0.7, // 350MB warning
            ..Default::default()
        };
        let active_ops = Arc::new(RwLock::new(HashMap::new()));

        // Add operation that exceeds warning threshold
        {
            let mut ops = active_ops.write().await;
            ops.insert(
                "medium".to_string(),
                OperationContext {
                    id: "medium".to_string(),
                    operation_type: OperationType::Analysis,
                    started_at: Instant::now(),
                    estimated_memory_mb: 300.0, // 300 + 100 base = 400 > 350 warning
                    priority: OperationPriority::Medium,
                },
            );
        }

        let usage = PlatformResourceController::measure_current_usage(&limits, &active_ops).await;
        assert_eq!(usage.memory_pressure, ResourcePressure::Medium);
    }

    #[tokio::test]
    async fn test_pressure_levels_high() {
        let limits = ResourceLimits {
            max_memory_mb: 500.0,
            memory_warning_threshold: 0.7,
            ..Default::default()
        };
        let active_ops = Arc::new(RwLock::new(HashMap::new()));

        // Add operation that causes >90% of limit
        {
            let mut ops = active_ops.write().await;
            ops.insert(
                "high".to_string(),
                OperationContext {
                    id: "high".to_string(),
                    operation_type: OperationType::Analysis,
                    started_at: Instant::now(),
                    estimated_memory_mb: 370.0, // 370 + 100 base = 470 > 450 (90%)
                    priority: OperationPriority::High,
                },
            );
        }

        let usage = PlatformResourceController::measure_current_usage(&limits, &active_ops).await;
        assert_eq!(usage.memory_pressure, ResourcePressure::High);
    }
}
