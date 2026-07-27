#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_service_creation() {
        let telemetry = TelemetryService::new();
        let system_data = telemetry.get_system_telemetry().await.unwrap();

        assert_eq!(system_data.system_metrics.total_operations, 0);
        assert_eq!(system_data.services.len(), 0);
    }

    #[tokio::test]
    async fn test_record_successful_operation() {
        let telemetry = TelemetryService::new();

        let input = TelemetryInput {
            event_type: "test".to_string(),
            service_name: "complexity_analyzer".to_string(),
            operation: "analyze_file".to_string(),
            metrics: OperationMetrics {
                duration_ms: 150,
                items_processed: 5,
                memory_bytes: Some(1024),
                cpu_time_ms: Some(100),
                success: true,
                error_message: None,
            },
            tags: HashMap::new(),
            properties: HashMap::new(),
        };

        let output = telemetry.record_operation(input).await.unwrap();
        assert!(output.success);
        assert!(!output.event_id.is_empty());

        let service_data = telemetry
            .get_service_telemetry("complexity_analyzer")
            .await
            .unwrap();
        assert_eq!(service_data.total_operations, 1);
        assert_eq!(service_data.successful_operations, 1);
        assert_eq!(service_data.total_items_processed, 5);
        assert_eq!(service_data.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_record_failed_operation() {
        let telemetry = TelemetryService::new();

        let input = TelemetryInput {
            event_type: "test".to_string(),
            service_name: "refactor_engine".to_string(),
            operation: "refactor_function".to_string(),
            metrics: OperationMetrics {
                duration_ms: 500,
                items_processed: 0,
                memory_bytes: None,
                cpu_time_ms: None,
                success: false,
                error_message: Some("Parse error".to_string()),
            },
            tags: HashMap::new(),
            properties: HashMap::new(),
        };

        let output = telemetry.record_operation(input).await.unwrap();
        assert!(output.success); // Telemetry recording succeeded

        let service_data = telemetry
            .get_service_telemetry("refactor_engine")
            .await
            .unwrap();
        assert_eq!(service_data.total_operations, 1);
        assert_eq!(service_data.failed_operations, 1);
        assert_eq!(service_data.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_system_telemetry_aggregation() {
        let telemetry = TelemetryService::new();

        let services = ["complexity_analyzer", "refactor_engine", "satd_detector"];

        for service in &services {
            let input = TelemetryInput {
                event_type: "test".to_string(),
                service_name: service.to_string(),
                operation: "test_operation".to_string(),
                metrics: OperationMetrics {
                    duration_ms: 100,
                    items_processed: 2,
                    memory_bytes: None,
                    cpu_time_ms: None,
                    success: true,
                    error_message: None,
                },
                tags: HashMap::new(),
                properties: HashMap::new(),
            };

            telemetry.record_operation(input).await.unwrap();
        }

        let system_data = telemetry.get_system_telemetry().await.unwrap();
        assert_eq!(system_data.system_metrics.total_operations, 3);
        assert_eq!(system_data.services.len(), 3);
        assert_eq!(system_data.system_metrics.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_validation() {
        let telemetry = TelemetryService::new();

        let invalid_input = TelemetryInput {
            event_type: "".to_string(), // Invalid - empty
            service_name: "test_service".to_string(),
            operation: "test_op".to_string(),
            metrics: OperationMetrics {
                duration_ms: 100,
                items_processed: 1,
                memory_bytes: None,
                cpu_time_ms: None,
                success: true,
                error_message: None,
            },
            tags: HashMap::new(),
            properties: HashMap::new(),
        };

        let result = telemetry.validate_input(&invalid_input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("event_type"));
    }

    #[tokio::test]
    async fn test_global_telemetry_service() {
        let telemetry1 = telemetry();
        let telemetry2 = telemetry();

        // Should be the same instance (singleton)
        assert!(Arc::ptr_eq(&telemetry1, &telemetry2));
    }

    #[test]
    fn test_telemetry_service_default() {
        let telemetry = TelemetryService::default();
        assert_eq!(telemetry.event_counter.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_reset() {
        let telemetry = TelemetryService::new();

        let input = TelemetryInput {
            event_type: "test".to_string(),
            service_name: "test_service".to_string(),
            operation: "test_op".to_string(),
            metrics: OperationMetrics {
                duration_ms: 100,
                items_processed: 1,
                memory_bytes: None,
                cpu_time_ms: None,
                success: true,
                error_message: None,
            },
            tags: HashMap::new(),
            properties: HashMap::new(),
        };

        telemetry.record_operation(input).await.unwrap();

        assert!(telemetry
            .get_service_telemetry("test_service")
            .await
            .is_some());

        telemetry.reset();

        assert!(telemetry
            .get_service_telemetry("test_service")
            .await
            .is_none());
        assert_eq!(telemetry.event_counter.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_metrics() {
        let telemetry = TelemetryService::new();
        let metrics = telemetry.metrics();
        assert_eq!(metrics.request_count, 0);
    }

    #[tokio::test]
    async fn test_process_via_service_trait() {
        use crate::services::service_base::Service;

        let telemetry = TelemetryService::new();
        let input = TelemetryInput {
            event_type: "test".to_string(),
            service_name: "service_trait_test".to_string(),
            operation: "process".to_string(),
            metrics: OperationMetrics {
                duration_ms: 50,
                items_processed: 1,
                memory_bytes: None,
                cpu_time_ms: None,
                success: true,
                error_message: None,
            },
            tags: HashMap::new(),
            properties: HashMap::new(),
        };

        let output = telemetry.process(input).await.unwrap();
        assert!(output.success);
    }

    #[tokio::test]
    async fn test_validation_missing_service_name() {
        let telemetry = TelemetryService::new();

        let invalid_input = TelemetryInput {
            event_type: "test".to_string(),
            service_name: "".to_string(), // Invalid - empty
            operation: "test_op".to_string(),
            metrics: OperationMetrics {
                duration_ms: 100,
                items_processed: 1,
                memory_bytes: None,
                cpu_time_ms: None,
                success: true,
                error_message: None,
            },
            tags: HashMap::new(),
            properties: HashMap::new(),
        };

        let result = telemetry.validate_input(&invalid_input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("service_name"));
    }

    #[tokio::test]
    async fn test_validation_missing_operation() {
        let telemetry = TelemetryService::new();

        let invalid_input = TelemetryInput {
            event_type: "test".to_string(),
            service_name: "test".to_string(),
            operation: "".to_string(), // Invalid - empty
            metrics: OperationMetrics {
                duration_ms: 100,
                items_processed: 1,
                memory_bytes: None,
                cpu_time_ms: None,
                success: true,
                error_message: None,
            },
            tags: HashMap::new(),
            properties: HashMap::new(),
        };

        let result = telemetry.validate_input(&invalid_input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("operation"));
    }

    #[tokio::test]
    async fn test_operation_counts_tracking() {
        let telemetry = TelemetryService::new();

        for _ in 0..3 {
            let input = TelemetryInput {
                event_type: "test".to_string(),
                service_name: "test_service".to_string(),
                operation: "analyze".to_string(),
                metrics: OperationMetrics {
                    duration_ms: 50,
                    items_processed: 1,
                    memory_bytes: None,
                    cpu_time_ms: None,
                    success: true,
                    error_message: None,
                },
                tags: HashMap::new(),
                properties: HashMap::new(),
            };
            telemetry.record_operation(input).await.unwrap();
        }

        let input = TelemetryInput {
            event_type: "test".to_string(),
            service_name: "test_service".to_string(),
            operation: "refactor".to_string(),
            metrics: OperationMetrics {
                duration_ms: 100,
                items_processed: 1,
                memory_bytes: None,
                cpu_time_ms: None,
                success: true,
                error_message: None,
            },
            tags: HashMap::new(),
            properties: HashMap::new(),
        };
        telemetry.record_operation(input).await.unwrap();

        let service_data = telemetry
            .get_service_telemetry("test_service")
            .await
            .unwrap();
        assert_eq!(service_data.total_operations, 4);
        assert_eq!(service_data.operation_counts.get("analyze"), Some(&3));
        assert_eq!(service_data.operation_counts.get("refactor"), Some(&1));
    }

    #[tokio::test]
    async fn test_peak_memory_tracking() {
        let telemetry = TelemetryService::new();

        let memories = [1000u64, 5000u64, 2000u64];
        let ops = ["op1", "op2", "op3"];
        for (op, mem) in ops.iter().zip(memories.iter()) {
            let input = TelemetryInput {
                event_type: "test".to_string(),
                service_name: "memory_test".to_string(),
                operation: op.to_string(),
                metrics: OperationMetrics {
                    duration_ms: 50,
                    items_processed: 1,
                    memory_bytes: Some(*mem),
                    cpu_time_ms: None,
                    success: true,
                    error_message: None,
                },
                tags: HashMap::new(),
                properties: HashMap::new(),
            };
            telemetry.record_operation(input).await.unwrap();
        }

        let service_data = telemetry
            .get_service_telemetry("memory_test")
            .await
            .unwrap();
        assert_eq!(service_data.peak_memory_bytes, 5000);
    }

    #[tokio::test]
    async fn test_average_duration_calculation() {
        let telemetry = TelemetryService::new();

        let durations = [100, 200, 300];
        for duration in durations {
            let input = TelemetryInput {
                event_type: "test".to_string(),
                service_name: "duration_test".to_string(),
                operation: "test".to_string(),
                metrics: OperationMetrics {
                    duration_ms: duration,
                    items_processed: 1,
                    memory_bytes: None,
                    cpu_time_ms: None,
                    success: true,
                    error_message: None,
                },
                tags: HashMap::new(),
                properties: HashMap::new(),
            };
            telemetry.record_operation(input).await.unwrap();
        }

        let service_data = telemetry
            .get_service_telemetry("duration_test")
            .await
            .unwrap();
        assert_eq!(service_data.total_duration_ms, 600);
        assert_eq!(service_data.avg_duration_ms, 200); // 600 / 3
    }

    #[tokio::test]
    async fn test_get_service_telemetry_nonexistent() {
        let telemetry = TelemetryService::new();
        let result = telemetry.get_service_telemetry("nonexistent").await;
        assert!(result.is_none());
    }

    #[test]
    fn test_service_telemetry_data_default() {
        let data = ServiceTelemetryData::default();
        assert_eq!(data.total_operations, 0);
        assert_eq!(data.success_rate, 0.0);
        assert!(data.operation_counts.is_empty());
    }

    #[test]
    fn test_system_telemetry_data_default() {
        let data = SystemTelemetryData::default();
        assert_eq!(data.startup_time, 0);
        assert_eq!(data.uptime_seconds, 0);
        assert!(data.services.is_empty());
    }

    #[test]
    fn test_operation_metrics_serialize() {
        let metrics = OperationMetrics {
            duration_ms: 100,
            items_processed: 5,
            memory_bytes: Some(1024),
            cpu_time_ms: Some(50),
            success: true,
            error_message: None,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("1024"));
    }

    #[test]
    fn test_telemetry_output_serialize() {
        let output = TelemetryOutput {
            event_id: "test-id".to_string(),
            recorded_at: 1234567890,
            success: true,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("1234567890"));
    }
}

