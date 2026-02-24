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
}
