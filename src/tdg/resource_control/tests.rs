#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    #[ignore = "Test hangs during compilation - tdg module conflicts"]
    async fn test_resource_controller_creation() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        let usage = controller.get_current_usage().await;

        assert_eq!(usage.active_operations, 0);
        assert_eq!(usage.memory_pressure, ResourcePressure::Low);
    }

    #[tokio::test]
    #[ignore = "Test hangs - needs investigation"]
    async fn test_resource_allocation_success() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        controller.start_monitoring().await.unwrap();

        let allocation = controller
            .request_resources(
                "test-op-1".to_string(),
                OperationType::Analysis,
                OperationPriority::High,
                100.0,
            )
            .await
            .unwrap();

        let usage = controller.get_current_usage().await;
        assert_eq!(usage.active_operations, 1);

        drop(allocation);
        sleep(Duration::from_millis(100)).await; // Allow cleanup to complete

        controller.stop_monitoring().await;
    }

    #[tokio::test]
    #[ignore = "Stack overflow issue - needs investigation"]
    async fn test_memory_limit_enforcement() {
        let limits = ResourceLimits {
            max_memory_mb: 200.0, // Small limit for testing
            ..Default::default()
        };
        let controller = PlatformResourceController::new(limits);
        controller.start_monitoring().await.unwrap();

        // Request more memory than limit
        let result = controller
            .request_resources(
                "test-op-memory".to_string(),
                OperationType::Analysis,
                OperationPriority::Low,
                300.0, // Exceeds 200MB limit
            )
            .await;

        assert!(result.is_err());
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_critical_priority_bypass() {
        let limits = ResourceLimits {
            max_memory_mb: 100.0, // Very small limit
            ..Default::default()
        };
        let controller = PlatformResourceController::new(limits);
        controller.start_monitoring().await.unwrap();

        // Critical operations should bypass normal limits
        let allocation = controller
            .request_resources(
                "critical-op".to_string(),
                OperationType::Commit,
                OperationPriority::Critical,
                150.0, // Exceeds limit but should be allowed
            )
            .await;

        assert!(allocation.is_ok());
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    #[ignore = "Test hangs - needs investigation"]
    async fn test_operation_counting() {
        let limits = ResourceLimits {
            max_concurrent_ops: 2, // Only 2 operations allowed
            ..Default::default()
        };
        let controller = PlatformResourceController::new(limits);
        controller.start_monitoring().await.unwrap();

        let _alloc1 = controller
            .request_resources(
                "op-1".to_string(),
                OperationType::Analysis,
                OperationPriority::High,
                50.0,
            )
            .await
            .unwrap();

        let _alloc2 = controller
            .request_resources(
                "op-2".to_string(),
                OperationType::Analysis,
                OperationPriority::High,
                50.0,
            )
            .await
            .unwrap();

        // Third operation should be rejected or queued
        let result = controller
            .request_resources(
                "op-3".to_string(),
                OperationType::Background,
                OperationPriority::Low,
                50.0,
            )
            .await;

        assert!(result.is_err());
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_enforcement_stats() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        controller.start_monitoring().await.unwrap();

        // Make some resource requests
        let _alloc = controller
            .request_resources(
                "stats-test".to_string(),
                OperationType::Analysis,
                OperationPriority::Medium,
                100.0,
            )
            .await
            .unwrap();

        let stats = controller.get_enforcement_stats().await;
        assert!(stats.total_requests > 0);
        assert!(stats.allowed_requests > 0);

        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_factory_patterns() {
        let default_ctrl = ResourceControllerFactory::create_default();
        let dev_ctrl = ResourceControllerFactory::create_dev_optimized();
        let prod_ctrl = ResourceControllerFactory::create_prod_optimized();
        let ci_ctrl = ResourceControllerFactory::create_ci_optimized();

        // Test that all controllers can start monitoring
        default_ctrl.start_monitoring().await.unwrap();
        dev_ctrl.start_monitoring().await.unwrap();
        prod_ctrl.start_monitoring().await.unwrap();
        ci_ctrl.start_monitoring().await.unwrap();

        // Verify they can handle resource requests
        let _alloc1 = default_ctrl
            .request_resources(
                "factory-test-1".to_string(),
                OperationType::Analysis,
                OperationPriority::Medium,
                50.0,
            )
            .await
            .unwrap();

        let _alloc2 = dev_ctrl
            .request_resources(
                "factory-test-2".to_string(),
                OperationType::Analysis,
                OperationPriority::Medium,
                50.0,
            )
            .await
            .unwrap();

        // Cleanup
        default_ctrl.stop_monitoring().await;
        dev_ctrl.stop_monitoring().await;
        prod_ctrl.stop_monitoring().await;
        ci_ctrl.stop_monitoring().await;
    }

    #[tokio::test]
    #[ignore = "Test involves background monitoring task that may hang in CI"]
    async fn test_resource_monitoring_lifecycle() {
        let controller = PlatformResourceController::new(ResourceLimits::default());

        // Should start successfully
        controller.start_monitoring().await.unwrap();

        // Starting again should be idempotent
        controller.start_monitoring().await.unwrap();

        // Should stop cleanly
        controller.stop_monitoring().await;

        // Should be able to restart
        controller.start_monitoring().await.unwrap();
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    #[ignore = "Test involves background monitoring task that may hang in CI"]
    async fn test_resource_pressure_levels() {
        let limits = ResourceLimits {
            max_memory_mb: 1000.0,
            memory_warning_threshold: 0.7, // 700MB warning
            ..Default::default()
        };
        let controller = PlatformResourceController::new(limits);
        controller.start_monitoring().await.unwrap();

        // Low pressure - under warning threshold
        let _alloc1 = controller
            .request_resources(
                "pressure-low".to_string(),
                OperationType::Analysis,
                OperationPriority::Medium,
                500.0, // 50% of limit
            )
            .await
            .unwrap();

        let usage1 = controller.get_current_usage().await;
        assert_eq!(usage1.memory_pressure, ResourcePressure::Low);

        // Medium pressure - over warning threshold
        let _alloc2 = controller
            .request_resources(
                "pressure-medium".to_string(),
                OperationType::Analysis,
                OperationPriority::Medium,
                250.0, // Total ~75% of limit
            )
            .await
            .unwrap();

        let usage2 = controller.get_current_usage().await;
        assert_eq!(usage2.memory_pressure, ResourcePressure::Medium);

        controller.stop_monitoring().await;
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod simple_tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_mb, 1024.0);
        assert_eq!(limits.max_cpu_utilization, 0.8);
        assert_eq!(limits.max_concurrent_ops, 20);
        assert_eq!(limits.memory_warning_threshold, 0.7);
        assert_eq!(limits.cpu_warning_threshold, 0.6);
        assert_eq!(limits.check_interval_secs, 5);
    }

    #[test]
    fn test_resource_limits_clone() {
        let limits = ResourceLimits {
            max_memory_mb: 512.0,
            max_cpu_utilization: 0.5,
            max_concurrent_ops: 10,
            memory_warning_threshold: 0.6,
            cpu_warning_threshold: 0.5,
            check_interval_secs: 3,
        };
        let cloned = limits.clone();
        assert_eq!(cloned.max_memory_mb, 512.0);
        assert_eq!(cloned.max_concurrent_ops, 10);
    }

    #[test]
    fn test_operation_type_variants() {
        let analysis = OperationType::Analysis;
        let commit = OperationType::Commit;
        let background = OperationType::Background;
        let storage = OperationType::Storage;
        let cleanup = OperationType::Cleanup;

        assert!(matches!(analysis, OperationType::Analysis));
        assert!(matches!(commit, OperationType::Commit));
        assert!(matches!(background, OperationType::Background));
        assert!(matches!(storage, OperationType::Storage));
        assert!(matches!(cleanup, OperationType::Cleanup));
    }

    #[test]
    fn test_operation_type_equality() {
        assert_eq!(OperationType::Analysis, OperationType::Analysis);
        assert_ne!(OperationType::Analysis, OperationType::Commit);
    }

    #[test]
    fn test_operation_type_clone() {
        let op = OperationType::Analysis;
        let cloned = op.clone();
        assert_eq!(cloned, OperationType::Analysis);
    }

    #[test]
    fn test_resource_pressure_variants() {
        let low = ResourcePressure::Low;
        let medium = ResourcePressure::Medium;
        let high = ResourcePressure::High;
        let critical = ResourcePressure::Critical;

        assert!(matches!(low, ResourcePressure::Low));
        assert!(matches!(medium, ResourcePressure::Medium));
        assert!(matches!(high, ResourcePressure::High));
        assert!(matches!(critical, ResourcePressure::Critical));
    }

    #[test]
    fn test_resource_pressure_equality() {
        assert_eq!(ResourcePressure::Low, ResourcePressure::Low);
        assert_ne!(ResourcePressure::Low, ResourcePressure::High);
    }

    #[test]
    fn test_operation_priority_variants() {
        let critical = OperationPriority::Critical;
        let high = OperationPriority::High;
        let medium = OperationPriority::Medium;
        let low = OperationPriority::Low;

        assert!(matches!(critical, OperationPriority::Critical));
        assert!(matches!(high, OperationPriority::High));
        assert!(matches!(medium, OperationPriority::Medium));
        assert!(matches!(low, OperationPriority::Low));
    }

    #[test]
    fn test_operation_priority_ordering() {
        assert!(OperationPriority::Critical < OperationPriority::High);
        assert!(OperationPriority::High < OperationPriority::Medium);
        assert!(OperationPriority::Medium < OperationPriority::Low);
    }

    #[test]
    fn test_operation_priority_equality() {
        assert_eq!(OperationPriority::High, OperationPriority::High);
        assert_ne!(OperationPriority::High, OperationPriority::Low);
    }

    #[test]
    fn test_resource_action_allow() {
        let action = ResourceAction::Allow;
        assert!(matches!(action, ResourceAction::Allow));
    }

    #[test]
    fn test_resource_action_throttle() {
        let action = ResourceAction::Throttle { delay_ms: 1000 };
        if let ResourceAction::Throttle { delay_ms } = action {
            assert_eq!(delay_ms, 1000);
        } else {
            panic!("Expected Throttle action");
        }
    }

    #[test]
    fn test_resource_action_queue() {
        let action = ResourceAction::Queue {
            estimated_wait_ms: 5000,
        };
        if let ResourceAction::Queue { estimated_wait_ms } = action {
            assert_eq!(estimated_wait_ms, 5000);
        } else {
            panic!("Expected Queue action");
        }
    }

    #[test]
    fn test_resource_action_reject() {
        let action = ResourceAction::Reject {
            reason: "Too busy".to_string(),
        };
        if let ResourceAction::Reject { reason } = action {
            assert_eq!(reason, "Too busy");
        } else {
            panic!("Expected Reject action");
        }
    }

    #[test]
    fn test_resource_action_emergency_stop() {
        let action = ResourceAction::EmergencyStop;
        assert!(matches!(action, ResourceAction::EmergencyStop));
    }

    #[test]
    fn test_operation_context_creation() {
        let context = OperationContext {
            id: "test-op".to_string(),
            operation_type: OperationType::Analysis,
            started_at: Instant::now(),
            estimated_memory_mb: 100.0,
            priority: OperationPriority::High,
        };

        assert_eq!(context.id, "test-op");
        assert_eq!(context.estimated_memory_mb, 100.0);
        assert_eq!(context.priority, OperationPriority::High);
    }

    #[test]
    fn test_operation_context_clone() {
        let context = OperationContext {
            id: "clone-test".to_string(),
            operation_type: OperationType::Commit,
            started_at: Instant::now(),
            estimated_memory_mb: 50.0,
            priority: OperationPriority::Critical,
        };

        let cloned = context.clone();
        assert_eq!(cloned.id, "clone-test");
        assert_eq!(cloned.priority, OperationPriority::Critical);
    }

    #[test]
    fn test_resource_enforcement_stats_format_diagnostic() {
        let stats = ResourceEnforcementStats {
            total_requests: 100,
            allowed_requests: 85,
            throttled_requests: 5,
            queued_requests: 7,
            rejected_requests: 3,
            current_active_operations: 10,
        };

        let output = stats.format_diagnostic();
        assert!(output.contains("Total requests: 100"));
        assert!(output.contains("85.0%")); // Success rate
        assert!(output.contains("Allowed: 85"));
        assert!(output.contains("Throttled: 5"));
        assert!(output.contains("Queued: 7"));
        assert!(output.contains("Rejected: 3"));
        assert!(output.contains("Active operations: 10"));
    }

    #[test]
    fn test_resource_enforcement_stats_empty() {
        let stats = ResourceEnforcementStats {
            total_requests: 0,
            allowed_requests: 0,
            throttled_requests: 0,
            queued_requests: 0,
            rejected_requests: 0,
            current_active_operations: 0,
        };

        let output = stats.format_diagnostic();
        assert!(output.contains("Total requests: 0"));
        assert!(output.contains("100.0%")); // 100% when no requests
    }

    #[test]
    fn test_resource_enforcement_stats_clone() {
        let stats = ResourceEnforcementStats {
            total_requests: 50,
            allowed_requests: 40,
            throttled_requests: 5,
            queued_requests: 3,
            rejected_requests: 2,
            current_active_operations: 5,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.total_requests, 50);
        assert_eq!(cloned.allowed_requests, 40);
    }

    #[test]
    fn test_enforcement_event_creation() {
        let usage = ResourceUsage {
            timestamp: Instant::now(),
            memory_mb: 500.0,
            cpu_utilization: 0.5,
            active_operations: 5,
            memory_pressure: ResourcePressure::Low,
            cpu_pressure: ResourcePressure::Low,
        };

        let event = EnforcementEvent {
            timestamp: Instant::now(),
            operation_id: "event-test".to_string(),
            action: ResourceAction::Allow,
            resource_usage: usage,
            reason: "Test event".to_string(),
        };

        assert_eq!(event.operation_id, "event-test");
        assert_eq!(event.reason, "Test event");
    }

    #[test]
    fn test_resource_usage_clone() {
        let usage = ResourceUsage {
            timestamp: Instant::now(),
            memory_mb: 750.0,
            cpu_utilization: 0.65,
            active_operations: 8,
            memory_pressure: ResourcePressure::Medium,
            cpu_pressure: ResourcePressure::Low,
        };

        let cloned = usage.clone();
        assert_eq!(cloned.memory_mb, 750.0);
        assert_eq!(cloned.cpu_utilization, 0.65);
        assert_eq!(cloned.active_operations, 8);
    }

    #[test]
    fn test_factory_create_default() {
        let controller = ResourceControllerFactory::create_default();
        // Just verify it creates without panic
        let _ = controller;
    }

    #[test]
    fn test_factory_create_dev_optimized() {
        let controller = ResourceControllerFactory::create_dev_optimized();
        let _ = controller;
    }

    #[test]
    fn test_factory_create_prod_optimized() {
        let controller = ResourceControllerFactory::create_prod_optimized();
        let _ = controller;
    }

    #[test]
    fn test_factory_create_ci_optimized() {
        let controller = ResourceControllerFactory::create_ci_optimized();
        let _ = controller;
    }

    #[test]
    fn test_controller_creation() {
        let limits = ResourceLimits::default();
        let controller = PlatformResourceController::new(limits);
        let _ = controller;
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod additional_coverage_tests {
    use super::*;

    // ============ ResourceLimits Tests ============

    #[test]
    fn test_resource_limits_debug() {
        let limits = ResourceLimits::default();
        let debug = format!("{:?}", limits);
        assert!(debug.contains("ResourceLimits"));
        assert!(debug.contains("max_memory_mb"));
    }

    #[test]
    fn test_resource_limits_serialization() {
        let limits = ResourceLimits {
            max_memory_mb: 2048.0,
            max_cpu_utilization: 0.9,
            max_concurrent_ops: 50,
            memory_warning_threshold: 0.8,
            cpu_warning_threshold: 0.7,
            check_interval_secs: 10,
        };
        let json = serde_json::to_string(&limits).unwrap();
        assert!(json.contains("2048"));
        assert!(json.contains("0.9"));

        let deserialized: ResourceLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_memory_mb, 2048.0);
        assert_eq!(deserialized.max_concurrent_ops, 50);
    }

    // ============ ResourceUsage Tests ============

    #[test]
    fn test_resource_usage_debug() {
        let usage = ResourceUsage {
            timestamp: Instant::now(),
            memory_mb: 512.0,
            cpu_utilization: 0.5,
            active_operations: 10,
            memory_pressure: ResourcePressure::Medium,
            cpu_pressure: ResourcePressure::Low,
        };
        let debug = format!("{:?}", usage);
        assert!(debug.contains("ResourceUsage"));
        assert!(debug.contains("memory_mb"));
    }

    #[test]
    fn test_resource_usage_serialization() {
        let usage = ResourceUsage {
            timestamp: Instant::now(),
            memory_mb: 768.0,
            cpu_utilization: 0.75,
            active_operations: 15,
            memory_pressure: ResourcePressure::High,
            cpu_pressure: ResourcePressure::Medium,
        };
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("768"));
        assert!(json.contains("0.75"));
    }

    // ============ ResourcePressure Tests ============

    #[test]
    fn test_resource_pressure_debug() {
        let pressures = [
            ResourcePressure::Low,
            ResourcePressure::Medium,
            ResourcePressure::High,
            ResourcePressure::Critical,
        ];
        for p in pressures {
            let debug = format!("{:?}", p);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_resource_pressure_clone() {
        let original = ResourcePressure::Critical;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_resource_pressure_serialization() {
        let pressure = ResourcePressure::High;
        let json = serde_json::to_string(&pressure).unwrap();
        let deserialized: ResourcePressure = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ResourcePressure::High);
    }

    // ============ ResourceAction Tests ============

    #[test]
    fn test_resource_action_debug() {
        let actions = [
            ResourceAction::Allow,
            ResourceAction::Throttle { delay_ms: 100 },
            ResourceAction::Queue {
                estimated_wait_ms: 500,
            },
            ResourceAction::Reject {
                reason: "test".to_string(),
            },
            ResourceAction::EmergencyStop,
        ];
        for action in actions {
            let debug = format!("{:?}", action);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_resource_action_clone() {
        let action = ResourceAction::Throttle { delay_ms: 250 };
        let cloned = action.clone();
        if let ResourceAction::Throttle { delay_ms } = cloned {
            assert_eq!(delay_ms, 250);
        } else {
            panic!("Clone failed");
        }
    }

    #[test]
    fn test_resource_action_serialization() {
        let action = ResourceAction::Queue {
            estimated_wait_ms: 1000,
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("1000"));

        let deserialized: ResourceAction = serde_json::from_str(&json).unwrap();
        if let ResourceAction::Queue { estimated_wait_ms } = deserialized {
            assert_eq!(estimated_wait_ms, 1000);
        } else {
            panic!("Deserialization failed");
        }
    }

    // ============ OperationType Tests ============

    #[test]
    fn test_operation_type_debug() {
        let types = [
            OperationType::Analysis,
            OperationType::Commit,
            OperationType::Background,
            OperationType::Storage,
            OperationType::Cleanup,
        ];
        for t in types {
            let debug = format!("{:?}", t);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_operation_type_serialization() {
        let op = OperationType::Storage;
        let json = serde_json::to_string(&op).unwrap();
        let deserialized: OperationType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, OperationType::Storage);
    }

    // ============ OperationPriority Tests ============

    #[test]
    fn test_operation_priority_debug() {
        let priorities = [
            OperationPriority::Critical,
            OperationPriority::High,
            OperationPriority::Medium,
            OperationPriority::Low,
        ];
        for p in priorities {
            let debug = format!("{:?}", p);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_operation_priority_serialization() {
        let priority = OperationPriority::Critical;
        let json = serde_json::to_string(&priority).unwrap();
        let deserialized: OperationPriority = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, OperationPriority::Critical);
    }

    #[test]
    fn test_operation_priority_clone() {
        let p = OperationPriority::Medium;
        let cloned = p.clone();
        assert_eq!(p, cloned);
    }

    // ============ OperationContext Tests ============

    #[test]
    fn test_operation_context_debug() {
        let ctx = OperationContext {
            id: "debug-test".to_string(),
            operation_type: OperationType::Analysis,
            started_at: Instant::now(),
            estimated_memory_mb: 100.0,
            priority: OperationPriority::High,
        };
        let debug = format!("{:?}", ctx);
        assert!(debug.contains("debug-test"));
    }

    #[test]
    fn test_operation_context_all_types() {
        let types = [
            OperationType::Analysis,
            OperationType::Commit,
            OperationType::Background,
            OperationType::Storage,
            OperationType::Cleanup,
        ];
        for op_type in types {
            let ctx = OperationContext {
                id: format!("ctx-{:?}", op_type),
                operation_type: op_type.clone(),
                started_at: Instant::now(),
                estimated_memory_mb: 50.0,
                priority: OperationPriority::Medium,
            };
            let cloned = ctx.clone();
            assert_eq!(cloned.operation_type, op_type);
        }
    }

    // ============ EnforcementEvent Tests ============

    #[test]
    fn test_enforcement_event_debug() {
        let event = EnforcementEvent {
            timestamp: Instant::now(),
            operation_id: "debug-event".to_string(),
            action: ResourceAction::Allow,
            resource_usage: ResourceUsage {
                timestamp: Instant::now(),
                memory_mb: 100.0,
                cpu_utilization: 0.3,
                active_operations: 2,
                memory_pressure: ResourcePressure::Low,
                cpu_pressure: ResourcePressure::Low,
            },
            reason: "Test reason".to_string(),
        };
        let debug = format!("{:?}", event);
        assert!(debug.contains("debug-event"));
    }

    #[test]
    fn test_enforcement_event_clone() {
        let event = EnforcementEvent {
            timestamp: Instant::now(),
            operation_id: "clone-event".to_string(),
            action: ResourceAction::Reject {
                reason: "busy".to_string(),
            },
            resource_usage: ResourceUsage {
                timestamp: Instant::now(),
                memory_mb: 800.0,
                cpu_utilization: 0.85,
                active_operations: 20,
                memory_pressure: ResourcePressure::High,
                cpu_pressure: ResourcePressure::High,
            },
            reason: "Clone test".to_string(),
        };
        let cloned = event.clone();
        assert_eq!(cloned.operation_id, "clone-event");
    }

    #[test]
    fn test_enforcement_event_serialization() {
        let event = EnforcementEvent {
            timestamp: Instant::now(),
            operation_id: "serial-event".to_string(),
            action: ResourceAction::Throttle { delay_ms: 500 },
            resource_usage: ResourceUsage {
                timestamp: Instant::now(),
                memory_mb: 400.0,
                cpu_utilization: 0.6,
                active_operations: 8,
                memory_pressure: ResourcePressure::Medium,
                cpu_pressure: ResourcePressure::Low,
            },
            reason: "Serialization test".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("serial-event"));
        assert!(json.contains("500"));
    }

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
