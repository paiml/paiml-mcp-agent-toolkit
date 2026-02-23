#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::super::tool::DiagnosticTool;
    use super::super::types::{
        AdaptiveDiagnostics, EnforcementStats, HealthStatus, PerformanceDiagnostics,
        ResourceDiagnostics, SchedulerDiagnostics, StorageDiagnostics, SystemDiagnostics,
    };
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_diagnostic_tool_creation() {
        let tool = DiagnosticTool::new();
        assert_eq!(tool.analysis_count, 0);
        assert_eq!(tool.error_count, 0);
    }

    #[test]
    fn test_diagnostic_tool_default() {
        let tool = DiagnosticTool::default();
        assert_eq!(tool.analysis_count, 0);
    }

    #[test]
    fn test_health_assessment() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics {
            cache_hit_ratio: 0.9,
            ..Default::default()
        };

        let resources = ResourceDiagnostics {
            memory_usage_mb: 500.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.5,
            ..Default::default()
        };

        let performance = PerformanceDiagnostics {
            error_rate: 0.01,
            p99_response_time_ms: 1000.0,
            ..Default::default()
        };

        let health = tool.assess_health(&storage, &resources, &performance);
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_assessment_critical_memory() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics {
            memory_usage_mb: 980.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.5,
            ..Default::default()
        };
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Critical { .. }));
    }

    #[test]
    fn test_health_assessment_critical_cpu() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics {
            memory_usage_mb: 500.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.98,
            ..Default::default()
        };
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Critical { .. }));
    }

    #[test]
    fn test_health_assessment_degraded_memory() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics {
            memory_usage_mb: 850.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.5,
            ..Default::default()
        };
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_assessment_degraded_cpu() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics {
            memory_usage_mb: 500.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.85,
            ..Default::default()
        };
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_assessment_low_cache_hit() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics {
            cache_hit_ratio: 0.3,
            ..Default::default()
        };
        let resources = ResourceDiagnostics::default();
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_assessment_critical_error_rate() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics::default();
        let performance = PerformanceDiagnostics {
            error_rate: 0.15,
            ..Default::default()
        };

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Critical { .. }));
    }

    #[test]
    fn test_health_assessment_degraded_error_rate() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics::default();
        let performance = PerformanceDiagnostics {
            error_rate: 0.07,
            ..Default::default()
        };

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_assessment_slow_response() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics::default();
        let performance = PerformanceDiagnostics {
            p99_response_time_ms: 6000.0,
            ..Default::default()
        };

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_performance_sampling() {
        let mut tool = DiagnosticTool::new();

        // Record some samples
        tool.record_sample(100.0, true);
        tool.record_sample(200.0, true);
        tool.record_sample(150.0, false);

        assert_eq!(tool.analysis_count, 2);
        assert_eq!(tool.error_count, 1);
        assert_eq!(tool.performance_samples.len(), 3);
    }

    #[test]
    fn test_performance_sampling_cleanup() {
        let mut tool = DiagnosticTool::new();

        // Add more than 1000 samples to trigger cleanup
        for i in 0..1100 {
            tool.record_sample(i as f64, true);
        }

        // Should keep only 600 (drained 500, kept last 500 + 100 new)
        assert!(tool.performance_samples.len() <= 600);
    }

    #[test]
    fn test_calculate_performance_metrics_empty() {
        let tool = DiagnosticTool::new();
        let metrics = tool.calculate_performance_metrics();
        assert_eq!(metrics.avg_response_time_ms, 0.0);
    }

    #[test]
    fn test_calculate_performance_metrics_with_samples() {
        let mut tool = DiagnosticTool::new();
        for i in 1..=100 {
            tool.record_sample(i as f64, true);
        }
        let metrics = tool.calculate_performance_metrics();
        assert!(metrics.avg_response_time_ms > 0.0);
        assert!(metrics.p50_response_time_ms > 0.0);
    }

    #[test]
    fn test_health_status_display_healthy() {
        let status = HealthStatus::Healthy;
        let display = format!("{}", status);
        assert!(display.contains("HEALTHY"));
    }

    #[test]
    fn test_health_status_display_degraded() {
        let status = HealthStatus::Degraded {
            reasons: vec!["High memory".to_string()],
        };
        let display = format!("{}", status);
        assert!(display.contains("DEGRADED"));
        assert!(display.contains("High memory"));
    }

    #[test]
    fn test_health_status_display_critical() {
        let status = HealthStatus::Critical {
            reasons: vec!["Memory critical".to_string()],
        };
        let display = format!("{}", status);
        assert!(display.contains("CRITICAL"));
        assert!(display.contains("Memory critical"));
    }

    #[test]
    fn test_storage_diagnostics_default() {
        let diag = StorageDiagnostics::default();
        assert_eq!(diag.backend_type, "none");
        assert_eq!(diag.total_entries, 0);
        assert!(diag.last_archival.is_none());
    }

    #[test]
    fn test_scheduler_diagnostics_default() {
        let diag = SchedulerDiagnostics::default();
        assert_eq!(diag.active_operations, 0);
        assert_eq!(diag.preemptions_count, 0);
    }

    #[test]
    fn test_adaptive_diagnostics_default() {
        let diag = AdaptiveDiagnostics::default();
        assert_eq!(diag.current_cache_size, 0);
        assert_eq!(diag.performance_trend, "Unknown");
    }

    #[test]
    fn test_resource_diagnostics_default() {
        let diag = ResourceDiagnostics::default();
        assert_eq!(diag.memory_limit_mb, 1024.0);
        assert_eq!(diag.cpu_limit, 0.8);
    }

    #[test]
    fn test_performance_diagnostics_default() {
        let diag = PerformanceDiagnostics::default();
        assert_eq!(diag.analyses_per_hour, 0.0);
        assert_eq!(diag.error_rate, 0.0);
    }

    #[test]
    fn test_enforcement_stats_default() {
        let stats = EnforcementStats::default();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.emergency_stops, 0);
    }

    #[test]
    fn test_format_diagnostics() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(3600),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Healthy,
        };

        let formatted = DiagnosticTool::format_diagnostics(&diag);
        assert!(formatted.contains("DIAGNOSTICS"));
        assert!(formatted.contains("STORAGE"));
        assert!(formatted.contains("SCHEDULER"));
    }

    #[test]
    fn test_system_diagnostics_clone() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Healthy,
        };
        let cloned = diag.clone();
        assert_eq!(cloned.uptime, diag.uptime);
    }

    #[test]
    fn test_system_diagnostics_debug() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Healthy,
        };
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("SystemDiagnostics"));
    }

    #[test]
    fn test_system_diagnostics_serialization() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Healthy,
        };
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("uptime"));
        assert!(json.contains("storage"));
    }

    #[test]
    fn test_storage_diagnostics_clone() {
        let diag = StorageDiagnostics {
            backend_type: "sled".to_string(),
            total_entries: 100,
            hot_cache_entries: 50,
            warm_entries: 30,
            cold_entries: 20,
            cache_hit_ratio: 0.9,
            compression_ratio: 0.5,
            storage_size_mb: 10.0,
            last_archival: Some(SystemTime::now()),
            deduplication_savings: 0.1,
        };
        let cloned = diag.clone();
        assert_eq!(cloned.total_entries, diag.total_entries);
        assert_eq!(cloned.backend_type, diag.backend_type);
    }

    #[test]
    fn test_storage_diagnostics_debug() {
        let diag = StorageDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("StorageDiagnostics"));
    }

    #[test]
    fn test_storage_diagnostics_serialization() {
        let diag = StorageDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("backend_type"));

        let deserialized: StorageDiagnostics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.backend_type, "none");
    }

    #[test]
    fn test_storage_diagnostics_all_fields() {
        let diag = StorageDiagnostics {
            backend_type: "custom".to_string(),
            total_entries: 1000,
            hot_cache_entries: 100,
            warm_entries: 200,
            cold_entries: 700,
            cache_hit_ratio: 0.85,
            compression_ratio: 0.7,
            storage_size_mb: 256.5,
            last_archival: Some(SystemTime::now()),
            deduplication_savings: 0.15,
        };
        assert_eq!(diag.total_entries, 1000);
        assert!((diag.cache_hit_ratio - 0.85).abs() < f64::EPSILON);
        assert!(diag.last_archival.is_some());
    }

    #[test]
    fn test_scheduler_diagnostics_clone() {
        let diag = SchedulerDiagnostics {
            active_operations: 10,
            queued_operations: 5,
            high_priority_available: 8,
            low_priority_available: 12,
            preemptions_count: 3,
            avg_wait_time_ms: 15.5,
            max_wait_time_ms: 50.0,
            operations_per_second: 100.0,
        };
        let cloned = diag.clone();
        assert_eq!(cloned.active_operations, diag.active_operations);
    }

    #[test]
    fn test_scheduler_diagnostics_debug() {
        let diag = SchedulerDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("SchedulerDiagnostics"));
    }

    #[test]
    fn test_scheduler_diagnostics_serialization() {
        let diag = SchedulerDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("active_operations"));

        let deserialized: SchedulerDiagnostics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.active_operations, 0);
    }

    #[test]
    fn test_scheduler_diagnostics_all_fields() {
        let diag = SchedulerDiagnostics {
            active_operations: 25,
            queued_operations: 10,
            high_priority_available: 4,
            low_priority_available: 8,
            preemptions_count: 100,
            avg_wait_time_ms: 25.0,
            max_wait_time_ms: 100.0,
            operations_per_second: 500.0,
        };
        assert_eq!(diag.queued_operations, 10);
        assert_eq!(diag.preemptions_count, 100);
        assert!((diag.operations_per_second - 500.0).abs() < f64::EPSILON);
    }
}
