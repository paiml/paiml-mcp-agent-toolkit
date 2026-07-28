#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_part2 {
    use super::super::tool::DiagnosticTool;
    use super::super::types::{
        AdaptiveDiagnostics, EnforcementStats, HealthStatus, PerformanceDiagnostics,
        ResourceDiagnostics, SchedulerDiagnostics, StorageDiagnostics, SystemDiagnostics,
    };
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_adaptive_diagnostics_clone() {
        let diag = AdaptiveDiagnostics {
            current_cache_size: 1000,
            current_compression_level: 5,
            high_priority_permits: 4,
            low_priority_permits: 8,
            performance_trend: "Stable".to_string(),
            adjustments_made: 10,
            avg_analysis_time_ms: 50.0,
            optimization_effectiveness: 0.9,
        };
        let cloned = diag.clone();
        assert_eq!(cloned.current_cache_size, diag.current_cache_size);
    }

    #[test]
    fn test_adaptive_diagnostics_debug() {
        let diag = AdaptiveDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("AdaptiveDiagnostics"));
    }

    #[test]
    fn test_adaptive_diagnostics_serialization() {
        let diag = AdaptiveDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("current_cache_size"));

        let deserialized: AdaptiveDiagnostics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.performance_trend, "Unknown");
    }

    #[test]
    fn test_adaptive_diagnostics_all_fields() {
        let diag = AdaptiveDiagnostics {
            current_cache_size: 5000,
            current_compression_level: 3,
            high_priority_permits: 6,
            low_priority_permits: 12,
            performance_trend: "Improving".to_string(),
            adjustments_made: 50,
            avg_analysis_time_ms: 75.0,
            optimization_effectiveness: 0.85,
        };
        assert_eq!(diag.current_compression_level, 3);
        assert_eq!(diag.adjustments_made, 50);
    }

    #[test]
    fn test_resource_diagnostics_clone() {
        let diag = ResourceDiagnostics {
            memory_usage_mb: 512.0,
            memory_limit_mb: 2048.0,
            memory_pressure: "Low".to_string(),
            cpu_utilization: 0.4,
            cpu_limit: 0.9,
            cpu_pressure: "Low".to_string(),
            enforcement_actions: EnforcementStats::default(),
        };
        let cloned = diag.clone();
        assert!((cloned.memory_usage_mb - diag.memory_usage_mb).abs() < f64::EPSILON);
    }

    #[test]
    fn test_resource_diagnostics_debug() {
        let diag = ResourceDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("ResourceDiagnostics"));
    }

    #[test]
    fn test_resource_diagnostics_serialization() {
        let diag = ResourceDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("memory_usage_mb"));

        let deserialized: ResourceDiagnostics = serde_json::from_str(&json).unwrap();
        assert!((deserialized.memory_limit_mb - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_resource_diagnostics_all_fields() {
        let diag = ResourceDiagnostics {
            memory_usage_mb: 768.0,
            memory_limit_mb: 1024.0,
            memory_pressure: "Medium".to_string(),
            cpu_utilization: 0.75,
            cpu_limit: 0.85,
            cpu_pressure: "High".to_string(),
            enforcement_actions: EnforcementStats {
                total_requests: 1000,
                allowed: 900,
                throttled: 50,
                queued: 30,
                rejected: 20,
                emergency_stops: 0,
            },
        };
        assert_eq!(diag.memory_pressure, "Medium");
        assert!((diag.cpu_utilization - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_enforcement_stats_clone() {
        let stats = EnforcementStats {
            total_requests: 500,
            allowed: 400,
            throttled: 50,
            queued: 30,
            rejected: 20,
            emergency_stops: 0,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_requests, stats.total_requests);
    }

    #[test]
    fn test_enforcement_stats_debug() {
        let stats = EnforcementStats::default();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("EnforcementStats"));
    }

    #[test]
    fn test_enforcement_stats_serialization() {
        let stats = EnforcementStats {
            total_requests: 100,
            allowed: 90,
            throttled: 5,
            queued: 3,
            rejected: 2,
            emergency_stops: 0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("total_requests"));

        let deserialized: EnforcementStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_requests, 100);
    }

    #[test]
    fn test_enforcement_stats_all_fields() {
        let stats = EnforcementStats {
            total_requests: 10000,
            allowed: 9000,
            throttled: 500,
            queued: 300,
            rejected: 200,
            emergency_stops: 5,
        };
        assert_eq!(stats.allowed, 9000);
        assert_eq!(stats.emergency_stops, 5);
    }

    #[test]
    fn test_performance_diagnostics_clone() {
        let diag = PerformanceDiagnostics {
            analyses_per_hour: 1000.0,
            avg_response_time_ms: 50.0,
            p50_response_time_ms: 40.0,
            p95_response_time_ms: 100.0,
            p99_response_time_ms: 200.0,
            throughput_mbps: 10.0,
            error_rate: 0.01,
        };
        let cloned = diag.clone();
        assert!((cloned.analyses_per_hour - diag.analyses_per_hour).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_diagnostics_debug() {
        let diag = PerformanceDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("PerformanceDiagnostics"));
    }

    #[test]
    fn test_performance_diagnostics_serialization() {
        let diag = PerformanceDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("analyses_per_hour"));

        let deserialized: PerformanceDiagnostics = serde_json::from_str(&json).unwrap();
        assert!((deserialized.error_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_diagnostics_all_fields() {
        let diag = PerformanceDiagnostics {
            analyses_per_hour: 5000.0,
            avg_response_time_ms: 25.0,
            p50_response_time_ms: 20.0,
            p95_response_time_ms: 80.0,
            p99_response_time_ms: 150.0,
            throughput_mbps: 50.0,
            error_rate: 0.005,
        };
        assert!((diag.p50_response_time_ms - 20.0).abs() < f64::EPSILON);
        assert!((diag.throughput_mbps - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_health_status_clone() {
        let healthy = HealthStatus::Healthy;
        let cloned = healthy.clone();
        assert_eq!(cloned, HealthStatus::Healthy);

        let degraded = HealthStatus::Degraded {
            reasons: vec!["Test".to_string()],
        };
        let cloned_degraded = degraded.clone();
        assert!(matches!(cloned_degraded, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_status_serialization() {
        let healthy = HealthStatus::Healthy;
        let json = serde_json::to_string(&healthy).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, HealthStatus::Healthy);

        let degraded = HealthStatus::Degraded {
            reasons: vec!["Reason 1".to_string()],
        };
        let json = serde_json::to_string(&degraded).unwrap();
        assert!(json.contains("Reason 1"));

        let critical = HealthStatus::Critical {
            reasons: vec!["Critical reason".to_string()],
        };
        let json = serde_json::to_string(&critical).unwrap();
        assert!(json.contains("Critical reason"));
    }

    #[test]
    fn test_health_status_eq() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(
            HealthStatus::Healthy,
            HealthStatus::Degraded { reasons: vec![] }
        );
        assert_ne!(
            HealthStatus::Degraded {
                reasons: vec!["a".to_string()]
            },
            HealthStatus::Critical {
                reasons: vec!["a".to_string()]
            }
        );
    }

    #[test]
    fn test_format_diagnostics_with_values() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(7200),
            storage: StorageDiagnostics {
                backend_type: "sled".to_string(),
                total_entries: 1000,
                hot_cache_entries: 500,
                warm_entries: 300,
                cold_entries: 200,
                cache_hit_ratio: 0.85,
                compression_ratio: 0.7,
                storage_size_mb: 50.0,
                last_archival: None,
                deduplication_savings: 0.1,
            },
            scheduler: SchedulerDiagnostics {
                active_operations: 5,
                queued_operations: 2,
                high_priority_available: 3,
                low_priority_available: 7,
                preemptions_count: 10,
                avg_wait_time_ms: 12.5,
                max_wait_time_ms: 50.0,
                operations_per_second: 200.0,
            },
            adaptive: AdaptiveDiagnostics {
                current_cache_size: 2000,
                current_compression_level: 3,
                high_priority_permits: 4,
                low_priority_permits: 8,
                performance_trend: "Stable".to_string(),
                adjustments_made: 25,
                avg_analysis_time_ms: 45.0,
                optimization_effectiveness: 0.9,
            },
            resources: ResourceDiagnostics {
                memory_usage_mb: 512.0,
                memory_limit_mb: 1024.0,
                memory_pressure: "Low".to_string(),
                cpu_utilization: 0.4,
                cpu_limit: 0.8,
                cpu_pressure: "Low".to_string(),
                enforcement_actions: EnforcementStats {
                    total_requests: 1000,
                    allowed: 950,
                    throttled: 30,
                    queued: 15,
                    rejected: 5,
                    emergency_stops: 0,
                },
            },
            performance: PerformanceDiagnostics {
                analyses_per_hour: 1200.0,
                avg_response_time_ms: 35.0,
                p50_response_time_ms: 30.0,
                p95_response_time_ms: 70.0,
                p99_response_time_ms: 120.0,
                throughput_mbps: 15.0,
                error_rate: 0.02,
            },
            health: HealthStatus::Healthy,
        };

        let formatted = DiagnosticTool::format_diagnostics(&diag);
        assert!(formatted.contains("sled"));
        assert!(formatted.contains("1000"));
        assert!(formatted.contains("Stable"));
        assert!(formatted.contains("HEALTHY"));
    }

    #[test]
    fn test_format_diagnostics_with_degraded_health() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Degraded {
                reasons: vec!["Memory high".to_string()],
            },
        };

        let formatted = DiagnosticTool::format_diagnostics(&diag);
        assert!(formatted.contains("DEGRADED"));
        assert!(formatted.contains("Memory high"));
    }

    #[test]
    fn test_format_diagnostics_with_critical_health() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Critical {
                reasons: vec!["System failure".to_string()],
            },
        };

        let formatted = DiagnosticTool::format_diagnostics(&diag);
        assert!(formatted.contains("CRITICAL"));
        assert!(formatted.contains("System failure"));
    }

    #[tokio::test]
    async fn test_collect_diagnostics_no_components() {
        let tool = DiagnosticTool::new();
        let diag = tool
            .collect_diagnostics(None, None, None, None)
            .await
            .unwrap();
        assert_eq!(diag.storage.backend_type, "none");
        assert_eq!(diag.scheduler.active_operations, 0);
        assert_eq!(diag.adaptive.performance_trend, "Unknown");
    }

    #[test]
    fn test_health_status_debug() {
        let healthy = HealthStatus::Healthy;
        let debug_str = format!("{:?}", healthy);
        assert!(debug_str.contains("Healthy"));

        let degraded = HealthStatus::Degraded {
            reasons: vec!["test".to_string()],
        };
        let debug_str = format!("{:?}", degraded);
        assert!(debug_str.contains("Degraded"));

        let critical = HealthStatus::Critical {
            reasons: vec!["test".to_string()],
        };
        let debug_str = format!("{:?}", critical);
        assert!(debug_str.contains("Critical"));
    }

    #[test]
    fn test_multiple_degraded_reasons() {
        let status = HealthStatus::Degraded {
            reasons: vec![
                "High memory".to_string(),
                "High CPU".to_string(),
                "Slow response".to_string(),
            ],
        };
        let display = format!("{}", status);
        assert!(display.contains("High memory"));
        assert!(display.contains("High CPU"));
        assert!(display.contains("Slow response"));
    }

    #[test]
    fn test_multiple_critical_reasons() {
        let status = HealthStatus::Critical {
            reasons: vec!["Memory critical".to_string(), "High error rate".to_string()],
        };
        let display = format!("{}", status);
        assert!(display.contains("Memory critical"));
        assert!(display.contains("High error rate"));
    }
}
