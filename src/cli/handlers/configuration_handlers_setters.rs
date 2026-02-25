// Configuration value setters for each config section.
// Includes: set_config_value and all per-section set_*_value functions.

/// Set a specific configuration value
fn set_config_value(config: &mut PmatConfig, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Key must be in format 'section.field', got '{key}'"
        ));
    }

    let section = parts[0];
    let field = parts[1];

    match section {
        "system" => set_system_value(&mut config.system, field, value),
        "quality" => set_quality_value(&mut config.quality, field, value),
        "analysis" => set_analysis_value(&mut config.analysis, field, value),
        "performance" => set_performance_value(&mut config.performance, field, value),
        "mcp" => set_mcp_value(&mut config.mcp, field, value),
        "roadmap" => set_roadmap_value(&mut config.roadmap, field, value),
        "telemetry" => set_telemetry_value(&mut config.telemetry, field, value),
        "semantic" => set_semantic_value(&mut config.semantic, field, value),
        _ => Err(anyhow::anyhow!("Unknown section '{section}'")),
    }
}

/// Set system configuration value
fn set_system_value(
    system: &mut crate::services::configuration_service::SystemConfig,
    field: &str,
    value: &str,
) -> Result<()> {
    match field {
        "project_name" => system.project_name = value.to_string(),
        "verbose" => system.verbose = value.parse()?,
        "debug" => system.debug = value.parse()?,
        "default_toolchain" => system.default_toolchain = value.to_string(),
        "max_concurrent_operations" => system.max_concurrent_operations = value.parse()?,
        _ => return Err(anyhow::anyhow!("Unknown system field '{field}'")),
    }
    Ok(())
}

/// Set quality configuration value
fn set_quality_value(
    quality: &mut crate::services::configuration_service::QualityConfig,
    field: &str,
    value: &str,
) -> Result<()> {
    match field {
        "max_complexity" => quality.max_complexity = value.parse()?,
        "max_cognitive_complexity" => quality.max_cognitive_complexity = value.parse()?,
        "min_coverage" => quality.min_coverage = value.parse()?,
        "allow_satd" => quality.allow_satd = value.parse()?,
        "require_docs" => quality.require_docs = value.parse()?,
        "lint_compliance" => quality.lint_compliance = value.parse()?,
        "fail_on_violation" => quality.fail_on_violation = value.parse()?,
        _ => return Err(anyhow::anyhow!("Unknown quality field '{field}'")),
    }
    Ok(())
}

/// Set analysis configuration value
fn set_analysis_value(
    analysis: &mut crate::services::configuration_service::AnalysisConfig,
    field: &str,
    value: &str,
) -> Result<()> {
    match field {
        "max_file_size" => analysis.max_file_size = value.parse()?,
        "max_line_length" => analysis.max_line_length = value.parse()?,
        "skip_vendor" => analysis.skip_vendor = value.parse()?,
        "parallel" => analysis.parallel = value.parse()?,
        "thread_count" => analysis.thread_count = value.parse()?,
        "timeout_seconds" => analysis.timeout_seconds = value.parse()?,
        _ => return Err(anyhow::anyhow!("Unknown analysis field '{field}'")),
    }
    Ok(())
}

/// Set performance configuration value
fn set_performance_value(
    performance: &mut crate::services::configuration_service::PerformanceConfig,
    field: &str,
    value: &str,
) -> Result<()> {
    match field {
        "enable_regression_tests" => performance.enable_regression_tests = value.parse()?,
        "enable_memory_tests" => performance.enable_memory_tests = value.parse()?,
        "enable_throughput_tests" => performance.enable_throughput_tests = value.parse()?,
        "test_iterations" => performance.test_iterations = value.parse()?,
        "timeout_ms" => performance.timeout_ms = value.parse()?,
        "target_startup_latency_ms" => performance.target_startup_latency_ms = value.parse()?,
        "target_throughput_loc_per_sec" => {
            performance.target_throughput_loc_per_sec = value.parse()?;
        }
        "target_memory_mb" => performance.target_memory_mb = value.parse()?,
        _ => return Err(anyhow::anyhow!("Unknown performance field '{field}'")),
    }
    Ok(())
}

/// Set MCP configuration value
fn set_mcp_value(
    mcp: &mut crate::services::configuration_service::McpConfig,
    field: &str,
    value: &str,
) -> Result<()> {
    match field {
        "server_name" => mcp.server_name = value.to_string(),
        "server_version" => mcp.server_version = value.to_string(),
        "enable_compression" => mcp.enable_compression = value.parse()?,
        "request_timeout_seconds" => mcp.request_timeout_seconds = value.parse()?,
        "max_request_size" => mcp.max_request_size = value.parse()?,
        "log_requests" => mcp.log_requests = value.parse()?,
        _ => return Err(anyhow::anyhow!("Unknown mcp field '{field}'")),
    }
    Ok(())
}

/// Set roadmap configuration value
fn set_roadmap_value(
    roadmap: &mut crate::services::configuration_service::RoadmapConfig,
    field: &str,
    value: &str,
) -> Result<()> {
    match field {
        "auto_generate_todos" => roadmap.auto_generate_todos = value.parse()?,
        "enforce_quality_gates" => roadmap.enforce_quality_gates = value.parse()?,
        "require_task_ids" => roadmap.require_task_ids = value.parse()?,
        "task_id_pattern" => roadmap.task_id_pattern = value.to_string(),
        "velocity_tracking" => roadmap.velocity_tracking = value.parse()?,
        "burndown_charts" => roadmap.burndown_charts = value.parse()?,
        _ => return Err(anyhow::anyhow!("Unknown roadmap field '{field}'")),
    }
    Ok(())
}

/// Set telemetry configuration value
fn set_telemetry_value(
    telemetry: &mut crate::services::configuration_service::TelemetryConfig,
    field: &str,
    value: &str,
) -> Result<()> {
    match field {
        "enabled" => telemetry.enabled = value.parse()?,
        "collection_interval_seconds" => telemetry.collection_interval_seconds = value.parse()?,
        "max_data_age_days" => telemetry.max_data_age_days = value.parse()?,
        "enable_aggregation" => telemetry.enable_aggregation = value.parse()?,
        "enable_export" => telemetry.enable_export = value.parse()?,
        "export_format" => telemetry.export_format = value.to_string(),
        _ => return Err(anyhow::anyhow!("Unknown telemetry field '{field}'")),
    }
    Ok(())
}

/// Set semantic configuration value
fn set_semantic_value(
    semantic: &mut crate::services::configuration_service::SemanticConfig,
    field: &str,
    value: &str,
) -> Result<()> {
    match field {
        "enabled" => semantic.enabled = value.parse()?,
        "vector_db_path" => semantic.vector_db_path = Some(value.to_string()),
        "workspace_path" => semantic.workspace_path = Some(std::path::PathBuf::from(value)),
        "embedding_model" => semantic.embedding_model = value.to_string(),
        "embedding_dimensions" => semantic.embedding_dimensions = value.parse()?,
        "default_search_mode" => semantic.default_search_mode = value.to_string(),
        "default_limit" => semantic.default_limit = value.parse()?,
        "auto_sync" => semantic.auto_sync = value.parse()?,
        "sync_interval_seconds" => semantic.sync_interval_seconds = value.parse()?,
        "max_chunk_tokens" => semantic.max_chunk_tokens = value.parse()?,
        "enable_mcp_tools" => semantic.enable_mcp_tools = value.parse()?,
        "enable_cache" => semantic.enable_cache = value.parse()?,
        "cache_expiration_days" => semantic.cache_expiration_days = value.parse()?,
        _ => return Err(anyhow::anyhow!("Unknown semantic field '{field}'")),
    }
    Ok(())
}
