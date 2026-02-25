// ==================== SPRINT 30 TDG SYSTEM MCP TOOLS ====================

/// Get comprehensive TDG system diagnostics
pub async fn tdg_system_diagnostics(
    detailed: bool,
    components: Vec<String>, // ["storage", "scheduler", "adaptive", "resources"]
) -> Result<Value> {
    let base_path = PathBuf::from(".");

    // Create diagnostic command
    let show_all = components.contains(&"all".to_string()) || components.is_empty();
    let command = TdgCommand::Diagnostics {
        detailed,
        storage: show_all || components.contains(&"storage".to_string()),
        scheduler: show_all || components.contains(&"scheduler".to_string()),
        adaptive: show_all || components.contains(&"adaptive".to_string()),
        resources: show_all || components.contains(&"resources".to_string()),
        all: show_all,
        format: DiagnosticOutputFormat::Json,
    };

    // Execute diagnostics
    match tdg_diagnostic_handler::handle_tdg_diagnostics(&command, &base_path).await {
        Ok(()) => Ok(json!({
            "status": "completed",
            "message": "TDG system diagnostics completed",
            "result_type": "diagnostics",
            "components_checked": if show_all {
                vec!["storage", "scheduler", "adaptive", "resources"]
            } else {
                components.iter().map(std::string::String::as_str).collect::<Vec<&str>>()
            },
            "detailed": detailed
        })),
        Err(e) => Ok(json!({
            "status": "error",
            "message": format!("Diagnostics failed: {}", e),
            "error": e.to_string()
        })),
    }
}

/// Get TDG performance metrics and adaptive threshold status
pub async fn tdg_performance_metrics() -> Result<Value> {
    // Create adaptive threshold manager to get performance stats
    let adaptive = AdaptiveThresholdFactory::create_default();
    let thresholds = adaptive.get_current_thresholds().await;
    let performance = adaptive.get_performance_stats().await;

    // Create scheduler for scheduling stats
    let scheduler = SchedulerFactory::create_balanced();
    let scheduler_stats = scheduler.get_statistics().await;

    Ok(json!({
        "status": "completed",
        "message": "TDG performance metrics retrieved",
        "result_type": "performance_metrics",
        "adaptive_thresholds": {
            "hot_cache_size": thresholds.hot_cache_size,
            "compression_level": thresholds.compression_level,
            "high_priority_permits": thresholds.high_priority_permits,
            "low_priority_permits": thresholds.low_priority_permits,
        },
        "performance_stats": {
            "avg_analysis_duration_ms": performance.avg_analysis_duration_ms,
            "avg_cache_hit_ratio": performance.avg_cache_hit_ratio,
            "avg_memory_usage_mb": performance.avg_memory_usage_mb,
            "avg_cpu_utilization": performance.avg_cpu_utilization,
            "total_samples": performance.total_samples,
        },
        "scheduler_stats": {
            "high_permits_available": scheduler_stats.high_permits_available,
            "low_permits_available": scheduler_stats.low_permits_available,
            "active_commits": scheduler_stats.active_commits,
            "active_background": scheduler_stats.active_background,
            "avg_wait_time_ms": scheduler_stats.avg_wait_time_ms,
            "total_active_operations": scheduler_stats.total_active_operations,
        }
    }))
}

/// Get TDG system health status with recommendations
pub async fn tdg_health_check() -> Result<Value> {
    let mut health_issues = Vec::new();
    let mut recommendations = Vec::new();
    let mut overall_status = "healthy".to_string();

    // Check storage health
    match TieredStorageFactory::create_default() {
        Ok(storage) => {
            let stats = storage.get_statistics();
            if stats.hot_memory_kb > 100_000 {
                // > 100MB
                health_issues.push("High hot cache memory usage detected".to_string());
                recommendations.push(
                    "Consider cleaning up hot cache or increasing archival frequency".to_string(),
                );
            }
            if stats.compression_ratio > 0.9 {
                health_issues.push("Low compression ratio detected".to_string());
                recommendations
                    .push("Consider different compression settings or backend".to_string());
            }
        }
        Err(e) => {
            health_issues.push(format!("Storage system unavailable: {e}"));
            overall_status = "critical".to_string();
        }
    }

    // Check scheduler health
    let scheduler = SchedulerFactory::create_balanced();
    let scheduler_stats = scheduler.get_statistics().await;
    if scheduler_stats.avg_wait_time_ms > 1000 {
        health_issues.push("High scheduler wait times detected".to_string());
        recommendations
            .push("Consider increasing scheduler permits or optimizing workload".to_string());
    }

    // Check adaptive thresholds health
    let adaptive = AdaptiveThresholdFactory::create_default();
    let performance = adaptive.get_performance_stats().await;
    if performance.avg_cache_hit_ratio < 0.7 {
        health_issues.push("Low cache hit ratio detected".to_string());
        recommendations
            .push("Consider increasing cache size or reviewing access patterns".to_string());
    }

    if !health_issues.is_empty() && overall_status == "healthy" {
        overall_status = "warning".to_string();
    }

    Ok(json!({
        "status": "completed",
        "message": "TDG system health check completed",
        "result_type": "health_check",
        "overall_status": overall_status,
        "health_score": if overall_status == "healthy" { 100 } else if overall_status == "warning" { 75 } else { 25 },
        "issues": health_issues,
        "recommendations": recommendations,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "storage": if health_issues.iter().any(|i| i.contains("Storage")) { "warning" } else { "healthy" },
            "scheduler": if health_issues.iter().any(|i| i.contains("scheduler")) { "warning" } else { "healthy" },
            "adaptive": if health_issues.iter().any(|i| i.contains("Adaptive")) { "warning" } else { "healthy" }
        }
    }))
}
