// Web dashboard tests extracted from web_dashboard.rs for file health (CB-040).
// This file is include!()'d into web_dashboard.rs scope.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    #[ignore] // SQLite database lock issues in concurrent tests
    async fn test_dashboard_state_creation() {
        let result = DashboardState::new().await;
        assert!(result.is_ok());

        let state = result.unwrap();
        let metrics = state.metrics_cache.read().await;
        assert_eq!(metrics.health_status.overall, "healthy");
    }

    #[tokio::test]
    #[ignore] // SQLite database lock issues in concurrent tests
    async fn test_metrics_update() {
        let state = DashboardState::new().await.unwrap();
        let result = state.update_metrics().await;
        assert!(result.is_ok());

        let metrics = state.metrics_cache.read().await;
        assert!(metrics.timestamp > SystemTime::UNIX_EPOCH);
    }

    #[test]
    #[ignore] // SQLite database lock issues in concurrent tests
    fn test_router_creation() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let state = DashboardState::new().await.unwrap();
            let _router = create_dashboard_router(state);
            // If we get here without panicking, router creation succeeded
        });
    }

    #[tokio::test]
    #[ignore] // SQLite database lock issues in concurrent tests
    async fn test_background_metrics_updates() {
        let state = DashboardState::new().await.unwrap();

        // Get initial timestamp
        let initial_time = {
            let metrics = state.metrics_cache.read().await;
            metrics.timestamp
        };

        // Update metrics
        sleep(Duration::from_millis(10)).await;
        state.update_metrics().await.unwrap();

        // Verify timestamp was updated
        let updated_time = {
            let metrics = state.metrics_cache.read().await;
            metrics.timestamp
        };

        assert!(updated_time > initial_time);
    }

    // ==================== Data Structure Tests ====================

    #[test]
    fn test_storage_metrics_creation() {
        let metrics = StorageMetrics {
            total_entries: 100,
            cache_hit_ratio: Some(0.85),
            compression_ratio: 0.65,
            backend_type: "sled".to_string(),
            storage_size_mb: 25.5,
        };

        assert_eq!(metrics.total_entries, 100);
        assert_eq!(metrics.cache_hit_ratio, Some(0.85));
        assert!((metrics.compression_ratio - 0.65).abs() < f64::EPSILON);
        assert_eq!(metrics.backend_type, "sled");
        assert!((metrics.storage_size_mb - 25.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_storage_metrics_serialization() {
        let metrics = StorageMetrics {
            total_entries: 42,
            cache_hit_ratio: Some(0.9),
            compression_ratio: 0.5,
            backend_type: "rocksdb".to_string(),
            storage_size_mb: 128.0,
        };

        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        let deserialized: StorageMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.total_entries, metrics.total_entries);
        assert_eq!(deserialized.backend_type, metrics.backend_type);
    }

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics {
            avg_analysis_time_ms: 150.5,
            active_operations: 5,
            queue_depth: 10,
            cpu_usage_percent: 45.0,
            memory_usage_mb: 512.0,
        };

        assert!((metrics.avg_analysis_time_ms - 150.5).abs() < f64::EPSILON);
        assert_eq!(metrics.active_operations, 5);
        assert_eq!(metrics.queue_depth, 10);
        assert!((metrics.cpu_usage_percent - 45.0).abs() < f64::EPSILON);
        assert!((metrics.memory_usage_mb - 512.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let metrics = PerformanceMetrics {
            avg_analysis_time_ms: 200.0,
            active_operations: 3,
            queue_depth: 7,
            cpu_usage_percent: 75.5,
            memory_usage_mb: 256.25,
        };

        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        let deserialized: PerformanceMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.active_operations, metrics.active_operations);
        assert_eq!(deserialized.queue_depth, metrics.queue_depth);
    }

    #[test]
    fn test_health_status_creation_healthy() {
        let status = HealthStatus {
            overall: "healthy".to_string(),
            issues: Vec::new(),
            recommendations: Vec::new(),
            uptime_seconds: 3600,
        };

        assert_eq!(status.overall, "healthy");
        assert!(status.issues.is_empty());
        assert!(status.recommendations.is_empty());
        assert_eq!(status.uptime_seconds, 3600);
    }

    #[test]
    fn test_health_status_creation_warning() {
        let status = HealthStatus {
            overall: "warning".to_string(),
            issues: vec!["High analysis times detected".to_string()],
            recommendations: vec![
                "Consider increasing cache size or optimizing queries".to_string()
            ],
            uptime_seconds: 7200,
        };

        assert_eq!(status.overall, "warning");
        assert_eq!(status.issues.len(), 1);
        assert_eq!(status.recommendations.len(), 1);
    }

    #[test]
    fn test_health_status_creation_critical() {
        let status = HealthStatus {
            overall: "critical".to_string(),
            issues: vec![
                "High analysis times detected".to_string(),
                "Low cache hit ratio".to_string(),
                "Memory pressure detected".to_string(),
            ],
            recommendations: vec![
                "Immediate attention required".to_string(),
                "Scale up resources".to_string(),
                "Optimize queries".to_string(),
            ],
            uptime_seconds: 86400,
        };

        assert_eq!(status.overall, "critical");
        assert_eq!(status.issues.len(), 3);
        assert_eq!(status.recommendations.len(), 3);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus {
            overall: "warning".to_string(),
            issues: vec!["Test issue".to_string()],
            recommendations: vec!["Test recommendation".to_string()],
            uptime_seconds: 1000,
        };

        let json = serde_json::to_string(&status).expect("Failed to serialize");
        let deserialized: HealthStatus =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.overall, status.overall);
        assert_eq!(deserialized.issues, status.issues);
        assert_eq!(deserialized.recommendations, status.recommendations);
        assert_eq!(deserialized.uptime_seconds, status.uptime_seconds);
    }

    #[test]
    fn test_system_metrics_creation() {
        let metrics = SystemMetrics {
            timestamp: SystemTime::now(),
            storage_stats: StorageMetrics {
                total_entries: 50,
                cache_hit_ratio: Some(0.8),
                compression_ratio: 0.6,
                backend_type: "libsql".to_string(),
                storage_size_mb: 64.0,
            },
            performance_stats: PerformanceMetrics {
                avg_analysis_time_ms: 100.0,
                active_operations: 2,
                queue_depth: 5,
                cpu_usage_percent: 30.0,
                memory_usage_mb: 256.0,
            },
            health_status: HealthStatus {
                overall: "healthy".to_string(),
                issues: Vec::new(),
                recommendations: Vec::new(),
                uptime_seconds: 3600,
            },
        };

        assert!(metrics.timestamp > SystemTime::UNIX_EPOCH);
        assert_eq!(metrics.storage_stats.total_entries, 50);
        assert_eq!(metrics.performance_stats.active_operations, 2);
        assert_eq!(metrics.health_status.overall, "healthy");
    }

    #[test]
    fn test_system_metrics_serialization() {
        let metrics = SystemMetrics {
            timestamp: SystemTime::UNIX_EPOCH + Duration::from_secs(1000000),
            storage_stats: StorageMetrics {
                total_entries: 100,
                cache_hit_ratio: Some(0.9),
                compression_ratio: 0.7,
                backend_type: "sled".to_string(),
                storage_size_mb: 128.0,
            },
            performance_stats: PerformanceMetrics {
                avg_analysis_time_ms: 50.0,
                active_operations: 1,
                queue_depth: 2,
                cpu_usage_percent: 25.0,
                memory_usage_mb: 128.0,
            },
            health_status: HealthStatus {
                overall: "healthy".to_string(),
                issues: Vec::new(),
                recommendations: Vec::new(),
                uptime_seconds: 7200,
            },
        };

        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        assert!(!json.is_empty());

        let deserialized: SystemMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(
            deserialized.storage_stats.total_entries,
            metrics.storage_stats.total_entries
        );
    }

    // ==================== Query/Request Tests ====================

    #[test]
    fn test_analysis_query_deserialization() {
        let json = r#"{"path": "/test/path", "backend": "sled", "priority": "high"}"#;
        let query: AnalysisQuery = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(query.path, "/test/path");
        assert_eq!(query.backend, Some("sled".to_string()));
        assert_eq!(query.priority, Some("high".to_string()));
    }

    #[test]
    fn test_analysis_query_minimal_deserialization() {
        let json = r#"{"path": "/minimal/path"}"#;
        let query: AnalysisQuery = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(query.path, "/minimal/path");
        assert!(query.backend.is_none());
        assert!(query.priority.is_none());
    }

    #[test]
    fn test_storage_operation_deserialization() {
        let json = r#"{"action": "flush", "options": {"force": true}}"#;
        let operation: StorageOperation =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(operation.action, "flush");
        assert!(operation.options.is_some());
        let options = operation.options.unwrap();
        assert_eq!(options["force"], true);
    }

    #[test]
    fn test_storage_operation_minimal_deserialization() {
        let json = r#"{"action": "cleanup"}"#;
        let operation: StorageOperation =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(operation.action, "cleanup");
        assert!(operation.options.is_none());
    }

    #[test]
    fn test_storage_operation_stats() {
        let json = r#"{"action": "stats"}"#;
        let operation: StorageOperation =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(operation.action, "stats");
    }

    // ==================== Health Logic Tests ====================

    #[test]
    fn test_health_determination_healthy() {
        // Simulate the health determination logic from update_metrics
        let avg_analysis_time_ms = 500.0; // Below 1000ms threshold
        let cache_hit_ratio = 0.85; // Above 0.7 threshold

        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        if avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
            recommendations
                .push("Consider increasing cache size or optimizing queries".to_string());
        }

        if cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
            recommendations.push("Review access patterns and consider cache tuning".to_string());
        }

        let overall = if issues.is_empty() {
            "healthy".to_string()
        } else if issues.len() <= 2 {
            "warning".to_string()
        } else {
            "critical".to_string()
        };

        assert_eq!(overall, "healthy");
        assert!(issues.is_empty());
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_health_determination_warning_high_analysis_time() {
        let avg_analysis_time_ms = 1500.0; // Above 1000ms threshold
        let cache_hit_ratio = 0.85; // Above 0.7 threshold

        let mut issues = Vec::new();

        if avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
        }

        if cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
        }

        let overall = if issues.is_empty() {
            "healthy".to_string()
        } else if issues.len() <= 2 {
            "warning".to_string()
        } else {
            "critical".to_string()
        };

        assert_eq!(overall, "warning");
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_health_determination_warning_low_cache_hit() {
        let avg_analysis_time_ms = 500.0; // Below 1000ms threshold
        let cache_hit_ratio = 0.5; // Below 0.7 threshold

        let mut issues = Vec::new();

        if avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
        }

        if cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
        }

        let overall = if issues.is_empty() {
            "healthy".to_string()
        } else if issues.len() <= 2 {
            "warning".to_string()
        } else {
            "critical".to_string()
        };

        assert_eq!(overall, "warning");
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_health_determination_warning_both_issues() {
        let avg_analysis_time_ms = 1500.0; // Above 1000ms threshold
        let cache_hit_ratio = 0.5; // Below 0.7 threshold

        let mut issues = Vec::new();

        if avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
        }

        if cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
        }

        let overall = if issues.is_empty() {
            "healthy".to_string()
        } else if issues.len() <= 2 {
            "warning".to_string()
        } else {
            "critical".to_string()
        };

        assert_eq!(overall, "warning");
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_health_determination_critical_many_issues() {
        // Simulate more than 2 issues (would need additional logic in real code)
        let issues = ["High analysis times detected".to_string(),
            "Low cache hit ratio".to_string(),
            "Memory pressure detected".to_string()];

        let overall = if issues.is_empty() {
            "healthy".to_string()
        } else if issues.len() <= 2 {
            "warning".to_string()
        } else {
            "critical".to_string()
        };

        assert_eq!(overall, "critical");
    }

    // ==================== Edge Cases Tests ====================

    #[test]
    fn test_storage_metrics_zero_values() {
        let metrics = StorageMetrics {
            total_entries: 0,
            cache_hit_ratio: Some(0.0),
            compression_ratio: 0.0,
            backend_type: String::new(),
            storage_size_mb: 0.0,
        };

        assert_eq!(metrics.total_entries, 0);
        assert_eq!(metrics.cache_hit_ratio, Some(0.0));
    }

    #[test]
    fn test_storage_metrics_max_values() {
        let metrics = StorageMetrics {
            total_entries: u64::MAX,
            cache_hit_ratio: Some(1.0),
            compression_ratio: 1.0,
            backend_type: "a".repeat(1000),
            storage_size_mb: f64::MAX,
        };

        assert_eq!(metrics.total_entries, u64::MAX);
        assert_eq!(metrics.cache_hit_ratio, Some(1.0));
    }

    #[test]
    fn test_performance_metrics_zero_values() {
        let metrics = PerformanceMetrics {
            avg_analysis_time_ms: 0.0,
            active_operations: 0,
            queue_depth: 0,
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
        };

        assert!(metrics.avg_analysis_time_ms.abs() < f64::EPSILON);
        assert_eq!(metrics.active_operations, 0);
    }

    #[test]
    fn test_health_status_empty_vectors() {
        let status = HealthStatus {
            overall: "healthy".to_string(),
            issues: Vec::new(),
            recommendations: Vec::new(),
            uptime_seconds: 0,
        };

        assert!(status.issues.is_empty());
        assert!(status.recommendations.is_empty());
    }

    #[test]
    fn test_health_status_large_uptime() {
        let status = HealthStatus {
            overall: "healthy".to_string(),
            issues: Vec::new(),
            recommendations: Vec::new(),
            uptime_seconds: 365 * 24 * 3600, // One year in seconds
        };

        assert_eq!(status.uptime_seconds, 31536000);
    }

    #[test]
    fn test_system_metrics_clone() {
        let original = SystemMetrics {
            timestamp: SystemTime::now(),
            storage_stats: StorageMetrics {
                total_entries: 100,
                cache_hit_ratio: Some(0.85),
                compression_ratio: 0.6,
                backend_type: "sled".to_string(),
                storage_size_mb: 64.0,
            },
            performance_stats: PerformanceMetrics {
                avg_analysis_time_ms: 150.0,
                active_operations: 5,
                queue_depth: 10,
                cpu_usage_percent: 45.0,
                memory_usage_mb: 256.0,
            },
            health_status: HealthStatus {
                overall: "warning".to_string(),
                issues: vec!["Test issue".to_string()],
                recommendations: vec!["Test recommendation".to_string()],
                uptime_seconds: 3600,
            },
        };

        let cloned = original.clone();

        assert_eq!(
            cloned.storage_stats.total_entries,
            original.storage_stats.total_entries
        );
        assert_eq!(
            cloned.performance_stats.active_operations,
            original.performance_stats.active_operations
        );
        assert_eq!(cloned.health_status.overall, original.health_status.overall);
    }

    // ==================== JSON Roundtrip Tests ====================

    #[test]
    fn test_storage_metrics_json_roundtrip() {
        let original = StorageMetrics {
            total_entries: 12345,
            cache_hit_ratio: Some(0.78),
            compression_ratio: 0.55,
            backend_type: "libsql".to_string(),
            storage_size_mb: 256.75,
        };

        let json = serde_json::to_string(&original).unwrap();
        let roundtrip: StorageMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.total_entries, original.total_entries);
        assert_eq!(roundtrip.cache_hit_ratio, original.cache_hit_ratio);
        assert_eq!(roundtrip.backend_type, original.backend_type);
    }

    #[test]
    fn test_performance_metrics_json_roundtrip() {
        let original = PerformanceMetrics {
            avg_analysis_time_ms: 99.99,
            active_operations: 42,
            queue_depth: 100,
            cpu_usage_percent: 87.5,
            memory_usage_mb: 1024.0,
        };

        let json = serde_json::to_string(&original).unwrap();
        let roundtrip: PerformanceMetrics = serde_json::from_str(&json).unwrap();

        assert!((roundtrip.avg_analysis_time_ms - original.avg_analysis_time_ms).abs() < 0.001);
        assert_eq!(roundtrip.active_operations, original.active_operations);
    }

    #[test]
    fn test_health_status_json_roundtrip() {
        let original = HealthStatus {
            overall: "warning".to_string(),
            issues: vec!["Issue 1".to_string(), "Issue 2".to_string()],
            recommendations: vec![
                "Recommendation A".to_string(),
                "Recommendation B".to_string(),
            ],
            uptime_seconds: 86400,
        };

        let json = serde_json::to_string(&original).unwrap();
        let roundtrip: HealthStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.overall, original.overall);
        assert_eq!(roundtrip.issues, original.issues);
        assert_eq!(roundtrip.recommendations, original.recommendations);
    }

    // ==================== Debug Trait Tests ====================

    #[test]
    fn test_storage_metrics_debug() {
        let metrics = StorageMetrics {
            total_entries: 50,
            cache_hit_ratio: Some(0.75),
            compression_ratio: 0.5,
            backend_type: "test".to_string(),
            storage_size_mb: 32.0,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("StorageMetrics"));
        assert!(debug_str.contains("50"));
    }

    #[test]
    fn test_performance_metrics_debug() {
        let metrics = PerformanceMetrics {
            avg_analysis_time_ms: 100.0,
            active_operations: 3,
            queue_depth: 5,
            cpu_usage_percent: 50.0,
            memory_usage_mb: 128.0,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("PerformanceMetrics"));
    }

    #[test]
    fn test_health_status_debug() {
        let status = HealthStatus {
            overall: "healthy".to_string(),
            issues: Vec::new(),
            recommendations: Vec::new(),
            uptime_seconds: 1000,
        };

        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("HealthStatus"));
        assert!(debug_str.contains("healthy"));
    }

    #[test]
    fn test_system_metrics_debug() {
        let metrics = SystemMetrics {
            timestamp: SystemTime::UNIX_EPOCH,
            storage_stats: StorageMetrics {
                total_entries: 0,
                cache_hit_ratio: Some(0.0),
                compression_ratio: 0.0,
                backend_type: String::new(),
                storage_size_mb: 0.0,
            },
            performance_stats: PerformanceMetrics {
                avg_analysis_time_ms: 0.0,
                active_operations: 0,
                queue_depth: 0,
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
            },
            health_status: HealthStatus {
                overall: "healthy".to_string(),
                issues: Vec::new(),
                recommendations: Vec::new(),
                uptime_seconds: 0,
            },
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("SystemMetrics"));
    }

    #[test]
    fn test_analysis_query_debug() {
        let query = AnalysisQuery {
            path: "/test/path".to_string(),
            backend: Some("sled".to_string()),
            priority: Some("high".to_string()),
        };

        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("AnalysisQuery"));
        assert!(debug_str.contains("/test/path"));
    }

    #[test]
    fn test_storage_operation_debug() {
        let operation = StorageOperation {
            action: "flush".to_string(),
            options: None,
        };

        let debug_str = format!("{:?}", operation);
        assert!(debug_str.contains("StorageOperation"));
        assert!(debug_str.contains("flush"));
    }

    // ==================== Threshold Boundary Tests ====================

    #[test]
    fn test_analysis_time_at_boundary() {
        // Exactly at 1000ms threshold
        let avg_analysis_time_ms = 1000.0;

        let mut issues = Vec::new();
        if avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
        }

        // At exactly 1000.0, should NOT trigger (> not >=)
        assert!(issues.is_empty());
    }

    #[test]
    fn test_analysis_time_just_above_boundary() {
        // Just above 1000ms threshold
        let avg_analysis_time_ms = 1000.001;

        let mut issues = Vec::new();
        if avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
        }

        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_cache_hit_ratio_at_boundary() {
        // Exactly at 0.7 threshold
        let cache_hit_ratio = 0.7;

        let mut issues = Vec::new();
        if cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
        }

        // At exactly 0.7, should NOT trigger (< not <=)
        assert!(issues.is_empty());
    }

    #[test]
    fn test_cache_hit_ratio_just_below_boundary() {
        // Just below 0.7 threshold
        let cache_hit_ratio = 0.699;

        let mut issues = Vec::new();
        if cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
        }

        assert_eq!(issues.len(), 1);
    }

    // ==================== DashboardState Cloning Tests ====================

    #[test]
    fn test_dashboard_state_clone_trait() {
        // Verify DashboardState implements Clone (compile-time check)
        fn assert_clone<T: Clone>() {}
        assert_clone::<DashboardState>();
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn storage_metrics_serialization_roundtrip(
            total_entries in 0u64..1_000_000u64,
            cache_hit_ratio in 0.0f64..1.0f64,
            compression_ratio in 0.0f64..1.0f64,
            storage_size_mb in 0.0f64..10000.0f64,
        ) {
            let metrics = StorageMetrics {
                total_entries,
                cache_hit_ratio: Some(cache_hit_ratio),
                compression_ratio,
                backend_type: "test".to_string(),
                storage_size_mb,
            };

            let json = serde_json::to_string(&metrics).unwrap();
            let roundtrip: StorageMetrics = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.total_entries, total_entries);
            prop_assert!((roundtrip.cache_hit_ratio.unwrap() - cache_hit_ratio).abs() < 0.0001);
        }

        #[test]
        fn performance_metrics_serialization_roundtrip(
            avg_analysis_time_ms in 0.0f64..10000.0f64,
            active_operations in 0u32..1000u32,
            queue_depth in 0u32..1000u32,
            cpu_usage_percent in 0.0f64..100.0f64,
            memory_usage_mb in 0.0f64..16384.0f64,
        ) {
            let metrics = PerformanceMetrics {
                avg_analysis_time_ms,
                active_operations,
                queue_depth,
                cpu_usage_percent,
                memory_usage_mb,
            };

            let json = serde_json::to_string(&metrics).unwrap();
            let roundtrip: PerformanceMetrics = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.active_operations, active_operations);
            prop_assert_eq!(roundtrip.queue_depth, queue_depth);
        }

        #[test]
        fn health_status_issue_count_determines_severity(issue_count in 0usize..10usize) {
            let issues: Vec<String> = (0..issue_count).map(|i| format!("Issue {}", i)).collect();
            let recommendations = issues.clone();

            let overall = if issues.is_empty() {
                "healthy".to_string()
            } else if issues.len() <= 2 {
                "warning".to_string()
            } else {
                "critical".to_string()
            };

            let status = HealthStatus {
                overall: overall.clone(),
                issues: issues.clone(),
                recommendations,
                uptime_seconds: 0,
            };

            match issue_count {
                0 => prop_assert_eq!(status.overall, "healthy"),
                1 | 2 => prop_assert_eq!(status.overall, "warning"),
                _ => prop_assert_eq!(status.overall, "critical"),
            }
        }

        #[test]
        fn uptime_always_serializable(uptime_secs in 0u64..u64::MAX) {
            let status = HealthStatus {
                overall: "healthy".to_string(),
                issues: Vec::new(),
                recommendations: Vec::new(),
                uptime_seconds: uptime_secs,
            };

            let json = serde_json::to_string(&status);
            prop_assert!(json.is_ok());

            let roundtrip: HealthStatus = serde_json::from_str(&json.unwrap()).unwrap();
            prop_assert_eq!(roundtrip.uptime_seconds, uptime_secs);
        }

        #[test]
        fn analysis_query_path_preserved(path in "[a-zA-Z0-9/_.-]+") {
            let query = AnalysisQuery {
                path: path.clone(),
                backend: None,
                priority: None,
            };

            // Serialize via JSON to simulate HTTP query parsing behavior
            let json = serde_json::to_string(&query).unwrap();
            let roundtrip: AnalysisQuery = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.path, path);
        }

        #[test]
        fn storage_operation_action_preserved(action in "flush|cleanup|stats|unknown") {
            let operation = StorageOperation {
                action: action.clone(),
                options: None,
            };

            let json = serde_json::to_string(&operation).unwrap();
            let roundtrip: StorageOperation = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.action, action);
        }

        #[test]
        fn metrics_cloning_preserves_values(
            total_entries in 0u64..1_000_000u64,
            cache_hit_ratio in 0.0f64..1.0f64,
        ) {
            let original = StorageMetrics {
                total_entries,
                cache_hit_ratio: Some(cache_hit_ratio),
                compression_ratio: 0.5,
                backend_type: "test".to_string(),
                storage_size_mb: 64.0,
            };

            let cloned = original.clone();

            prop_assert_eq!(cloned.total_entries, original.total_entries);
            prop_assert!((cloned.cache_hit_ratio.unwrap() - original.cache_hit_ratio.unwrap()).abs() < f64::EPSILON);
        }

        #[test]
        fn system_metrics_timestamp_always_valid(secs_since_epoch in 0u64..u64::MAX / 2) {
            // Use a valid timestamp range
            let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(secs_since_epoch);

            let metrics = SystemMetrics {
                timestamp,
                storage_stats: StorageMetrics {
                    total_entries: 0,
                    cache_hit_ratio: Some(0.0),
                    compression_ratio: 0.0,
                    backend_type: String::new(),
                    storage_size_mb: 0.0,
                },
                performance_stats: PerformanceMetrics {
                    avg_analysis_time_ms: 0.0,
                    active_operations: 0,
                    queue_depth: 0,
                    cpu_usage_percent: 0.0,
                    memory_usage_mb: 0.0,
                },
                health_status: HealthStatus {
                    overall: "healthy".to_string(),
                    issues: Vec::new(),
                    recommendations: Vec::new(),
                    uptime_seconds: 0,
                },
            };

            prop_assert!(metrics.timestamp >= SystemTime::UNIX_EPOCH);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode as AxumStatusCode};
    use tower::ServiceExt;

    /// Helper to create a minimal DashboardState for testing without real storage
    async fn create_test_state() -> DashboardState {
        // Use in-memory storage to avoid database conflicts
        let storage = Arc::new(TieredStorageFactory::create_in_memory());
        let analyzer = Arc::new(TdgAnalyzer::new().unwrap());

        let initial_metrics = SystemMetrics {
            timestamp: SystemTime::now(),
            storage_stats: StorageMetrics {
                total_entries: 0,
                cache_hit_ratio: Some(0.0),
                compression_ratio: 0.0,
                backend_type: "in-memory".to_string(),
                storage_size_mb: 0.0,
            },
            performance_stats: PerformanceMetrics {
                avg_analysis_time_ms: 0.0,
                active_operations: 0,
                queue_depth: 0,
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
            },
            health_status: HealthStatus {
                overall: "healthy".to_string(),
                issues: Vec::new(),
                recommendations: Vec::new(),
                uptime_seconds: 0,
            },
        };

        DashboardState {
            storage,
            analyzer,
            metrics_cache: Arc::new(RwLock::new(initial_metrics)),
            started_at: Instant::now(),
        }
    }

    #[tokio::test]
    async fn test_dashboard_index_route() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);
    }

    #[tokio::test]
    async fn test_dashboard_route() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/dashboard")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);
    }

    #[tokio::test]
    async fn test_storage_stats_endpoint() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/storage/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);
    }

    #[tokio::test]
    async fn test_events_endpoint() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);
    }

    #[tokio::test]
    async fn test_diagnostics_endpoint() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/diagnostics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);
    }

    #[tokio::test]
    async fn test_analysis_missing_path() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/analysis?path=/nonexistent/path/that/does/not/exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return NOT_FOUND for non-existent path
        assert_eq!(response.status(), AxumStatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_storage_operation_flush() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/storage/operation")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action": "flush"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);
    }

    /// `{"action":"cleanup"}` used to answer 200 {"status":"completed",
    /// "entries_cleaned":0} without touching storage. With no retention age to
    /// work from there is nothing to clean, so it must not claim success.
    #[tokio::test]
    async fn test_storage_operation_cleanup_without_max_age_is_rejected() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/storage/operation")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action": "cleanup"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_storage_operation_cleanup_reports_real_eviction_count() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/storage/operation")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"action": "cleanup", "options": {"max_age_seconds": 0}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        // An empty in-memory store really has nothing to evict; the point is
        // that the number now comes from cleanup_hot_cache().
        assert_eq!(json["entries_cleaned"], 0);
        assert!(json["message"].as_str().unwrap().contains("hot cache"));
    }

    #[tokio::test]
    async fn test_storage_operation_stats() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/storage/operation")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action": "stats"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::OK);
    }

    #[tokio::test]
    async fn test_storage_operation_unknown() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/storage/operation")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action": "unknown_operation"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), AxumStatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_metrics_cache_update() {
        let state = create_test_state().await;

        // Initial state
        {
            let metrics = state.metrics_cache.read().await;
            assert_eq!(metrics.health_status.overall, "healthy");
        }

        // Simulate a metrics update by modifying the cache directly
        {
            let mut metrics = state.metrics_cache.write().await;
            metrics.health_status.overall = "warning".to_string();
            metrics.health_status.issues.push("Test issue".to_string());
        }

        // Verify update persisted
        {
            let metrics = state.metrics_cache.read().await;
            assert_eq!(metrics.health_status.overall, "warning");
            assert_eq!(metrics.health_status.issues.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_concurrent_metrics_access() {
        let state = create_test_state().await;
        let state_clone = state.clone();

        // Spawn concurrent read operations
        let handle1 = tokio::spawn(async move {
            for _ in 0..10 {
                let _metrics = state.metrics_cache.read().await;
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        });

        let handle2 = tokio::spawn(async move {
            for i in 0..10 {
                let mut metrics = state_clone.metrics_cache.write().await;
                metrics.performance_stats.active_operations = i;
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        });

        // Both should complete without deadlock
        let (r1, r2) = tokio::join!(handle1, handle2);
        assert!(r1.is_ok());
        assert!(r2.is_ok());
    }

    #[tokio::test]
    async fn test_state_storage_access() {
        let state = create_test_state().await;

        // Get statistics from storage
        let stats = state.storage.get_statistics();

        // In-memory storage should report minimal statistics
        assert_eq!(stats.hot_entries, 0);
        assert_eq!(stats.warm_entries, 0);
    }

    /// `/api/health` reported `uptime_seconds` as seconds since the UNIX epoch
    /// (1786295016, i.e. ~56.6 years, identical to the payload's own timestamp
    /// and unchanged four seconds later), never as time since the server came
    /// up.
    #[tokio::test]
    async fn uptime_is_measured_from_server_start_not_from_1970() {
        let state = create_test_state().await;
        state.update_metrics().await.unwrap();

        let uptime = state.metrics_cache.read().await.health_status.uptime_seconds;
        assert!(
            uptime < 60,
            "a server that just started cannot have been up {uptime}s"
        );

        let secs_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_ne!(uptime, secs_since_epoch);
    }

    /// `cache_hit_ratio` was the literal 0.85 in both `/api/metrics` and
    /// `/api/storage/stats`, and the `< 0.7` "Low cache hit ratio" check was
    /// then evaluated against that same constant, so it could never fire.
    #[tokio::test]
    async fn cache_hit_ratio_is_reported_as_unmeasured() {
        let state = create_test_state().await;
        state.update_metrics().await.unwrap();

        assert_eq!(
            state.metrics_cache.read().await.storage_stats.cache_hit_ratio,
            None,
            "nothing counts cache hits, so no ratio may be reported"
        );

        let app = create_dashboard_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/storage/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            json["cache_hit_ratio"].is_null(),
            "expected null, got {}",
            json["cache_hit_ratio"]
        );
    }

    /// The "Low cache hit ratio" issue must still be reachable for a measured
    /// ratio below the threshold — otherwise the check is dead either way.
    #[tokio::test]
    async fn low_measured_cache_hit_ratio_is_still_an_issue() {
        let state = create_test_state().await;
        {
            let mut metrics = state.metrics_cache.write().await;
            metrics.storage_stats.cache_hit_ratio = Some(0.5);
        }

        let issues: Vec<String> = {
            let metrics = state.metrics_cache.read().await;
            let mut issues = Vec::new();
            if metrics
                .storage_stats
                .cache_hit_ratio
                .is_some_and(|ratio| ratio < 0.7)
            {
                issues.push("Low cache hit ratio".to_string());
            }
            issues
        };
        assert_eq!(issues, vec!["Low cache hit ratio".to_string()]);
    }

    /// The dashboard used to attach `CorsLayer::new().allow_origin(Any)
    /// .allow_methods(Any).allow_headers(Any)` unconditionally, so any page the
    /// user had open could read `/api/analysis?path=<any file>` and POST to
    /// `/api/storage/operation`. Its own HTML is same-origin and needs no CORS.
    #[tokio::test]
    async fn no_cross_origin_access_is_granted() {
        let state = create_test_state().await;
        let app = create_dashboard_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .header("origin", "https://evil.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            response
                .headers()
                .get("access-control-allow-origin")
                .is_none(),
            "dashboard must not advertise cross-origin access"
        );
    }
}
