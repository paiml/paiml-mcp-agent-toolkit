//! TDG Web Dashboard - Unit Tests

#[cfg(test)]
mod tests {
    use super::super::web_dashboard_types::{
        AnalysisQuery, HealthStatus, PerformanceMetrics, StorageMetrics, StorageOperation,
        SystemMetrics,
    };
    use std::time::{Duration, SystemTime};

    // ==================== Data Structure Tests ====================

    #[test]
    fn test_storage_metrics_creation() {
        let metrics = StorageMetrics {
            total_entries: 100,
            cache_hit_ratio: 0.85,
            compression_ratio: 0.65,
            backend_type: "sled".to_string(),
            storage_size_mb: 25.5,
        };

        assert_eq!(metrics.total_entries, 100);
        assert!((metrics.cache_hit_ratio - 0.85).abs() < f64::EPSILON);
        assert!((metrics.compression_ratio - 0.65).abs() < f64::EPSILON);
        assert_eq!(metrics.backend_type, "sled");
        assert!((metrics.storage_size_mb - 25.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_storage_metrics_serialization() {
        let metrics = StorageMetrics {
            total_entries: 42,
            cache_hit_ratio: 0.9,
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
                "Consider increasing cache size or optimizing queries".to_string(),
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
                cache_hit_ratio: 0.8,
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
                cache_hit_ratio: 0.9,
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
        let avg_analysis_time_ms = 500.0;
        let cache_hit_ratio = 0.85;

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
        let avg_analysis_time_ms = 1500.0;
        let cache_hit_ratio = 0.85;

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
        let avg_analysis_time_ms = 500.0;
        let cache_hit_ratio = 0.5;

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
        let avg_analysis_time_ms = 1500.0;
        let cache_hit_ratio = 0.5;

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
        let issues = vec![
            "High analysis times detected".to_string(),
            "Low cache hit ratio".to_string(),
            "Memory pressure detected".to_string(),
        ];

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
            cache_hit_ratio: 0.0,
            compression_ratio: 0.0,
            backend_type: String::new(),
            storage_size_mb: 0.0,
        };

        assert_eq!(metrics.total_entries, 0);
        assert!(metrics.cache_hit_ratio.abs() < f64::EPSILON);
    }

    #[test]
    fn test_storage_metrics_max_values() {
        let metrics = StorageMetrics {
            total_entries: u64::MAX,
            cache_hit_ratio: 1.0,
            compression_ratio: 1.0,
            backend_type: "a".repeat(1000),
            storage_size_mb: f64::MAX,
        };

        assert_eq!(metrics.total_entries, u64::MAX);
        assert!((metrics.cache_hit_ratio - 1.0).abs() < f64::EPSILON);
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
            uptime_seconds: 365 * 24 * 3600,
        };

        assert_eq!(status.uptime_seconds, 31536000);
    }

    #[test]
    fn test_system_metrics_clone() {
        let original = SystemMetrics {
            timestamp: SystemTime::now(),
            storage_stats: StorageMetrics {
                total_entries: 100,
                cache_hit_ratio: 0.85,
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
        assert_eq!(
            cloned.health_status.overall,
            original.health_status.overall
        );
    }

    // ==================== JSON Roundtrip Tests ====================

    #[test]
    fn test_storage_metrics_json_roundtrip() {
        let original = StorageMetrics {
            total_entries: 12345,
            cache_hit_ratio: 0.78,
            compression_ratio: 0.55,
            backend_type: "libsql".to_string(),
            storage_size_mb: 256.75,
        };

        let json = serde_json::to_string(&original).unwrap();
        let roundtrip: StorageMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.total_entries, original.total_entries);
        assert!((roundtrip.cache_hit_ratio - original.cache_hit_ratio).abs() < 0.001);
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
            cache_hit_ratio: 0.75,
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
                cache_hit_ratio: 0.0,
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
        let avg_analysis_time_ms = 1000.0;

        let mut issues = Vec::new();
        if avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
        }

        assert!(issues.is_empty());
    }

    #[test]
    fn test_analysis_time_just_above_boundary() {
        let avg_analysis_time_ms = 1000.001;

        let mut issues = Vec::new();
        if avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
        }

        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_cache_hit_ratio_at_boundary() {
        let cache_hit_ratio = 0.7;

        let mut issues = Vec::new();
        if cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
        }

        assert!(issues.is_empty());
    }

    #[test]
    fn test_cache_hit_ratio_just_below_boundary() {
        let cache_hit_ratio = 0.699;

        let mut issues = Vec::new();
        if cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
        }

        assert_eq!(issues.len(), 1);
    }
}
