#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for configuration_service module
//! Target: Test pure functions and configuration defaults

use crate::services::configuration_service::{
    AnalysisConfig, ConfigurationService, McpConfig, PerformanceConfig, PmatConfig, QualityConfig,
    SystemConfig, TelemetryConfig,
};
use std::path::PathBuf;

// ============================================================================
// default_config tests
// ============================================================================

#[test]
fn test_default_config_creates_valid_config() {
    let config = ConfigurationService::default_config();
    assert!(!config.system.project_name.is_empty());
}

#[test]
fn test_default_config_system_defaults() {
    let config = ConfigurationService::default_config();

    assert_eq!(config.system.project_name, "pmat");
    assert_eq!(config.system.output_dir, PathBuf::from("target/pmat"));
    assert!(config.system.max_concurrent_operations > 0);
    assert!(!config.system.verbose);
    assert!(!config.system.debug);
    assert_eq!(config.system.default_toolchain, "rust");
}

#[test]
fn test_default_config_quality_defaults() {
    let config = ConfigurationService::default_config();

    assert_eq!(config.quality.max_complexity, 30);
    assert_eq!(config.quality.max_cognitive_complexity, 25);
    assert!((config.quality.min_coverage - 80.0).abs() < f64::EPSILON);
    assert!(!config.quality.allow_satd);
    assert!(config.quality.require_docs);
    assert!(config.quality.lint_compliance);
    assert!(config.quality.fail_on_violation);
}

#[test]
fn test_default_config_analysis_defaults() {
    let config = ConfigurationService::default_config();

    assert!(!config.analysis.include_patterns.is_empty());
    assert!(config.analysis.include_patterns.contains(&"**/*.rs".to_string()));
    assert!(config.analysis.include_patterns.contains(&"**/*.ts".to_string()));
    assert!(!config.analysis.exclude_patterns.is_empty());
    assert!(config.analysis.exclude_patterns.contains(&"**/target/**".to_string()));
    assert!(config.analysis.exclude_patterns.contains(&"**/node_modules/**".to_string()));
    assert_eq!(config.analysis.max_file_size, 1024 * 1024); // 1MB
    assert_eq!(config.analysis.max_line_length, 100);
    assert!(config.analysis.skip_vendor);
    assert!(config.analysis.parallel);
    assert_eq!(config.analysis.thread_count, 0); // Auto
    assert_eq!(config.analysis.timeout_seconds, 300); // 5 minutes
}

#[test]
fn test_default_config_performance_defaults() {
    let config = ConfigurationService::default_config();

    assert!(config.performance.enable_regression_tests);
    assert!(config.performance.enable_memory_tests);
    assert!(config.performance.enable_throughput_tests);
    assert_eq!(config.performance.test_iterations, 10);
    assert_eq!(config.performance.timeout_ms, 30000);
    assert_eq!(config.performance.target_startup_latency_ms, 127);
    assert_eq!(config.performance.target_throughput_loc_per_sec, 487000);
    assert_eq!(config.performance.target_memory_mb, 47);
}

#[test]
fn test_default_config_mcp_defaults() {
    let config = ConfigurationService::default_config();

    assert_eq!(config.mcp.server_name, "pmat-mcp-server");
    assert!(!config.mcp.server_version.is_empty());
    assert!(config.mcp.enable_compression);
    assert_eq!(config.mcp.request_timeout_seconds, 30);
    assert_eq!(config.mcp.max_request_size, 10 * 1024 * 1024); // 10MB
    assert!(!config.mcp.log_requests);
    assert!(!config.mcp.enabled_tools.is_empty());
    assert!(config.mcp.enabled_tools.contains(&"analyze_complexity".to_string()));
    assert!(config.mcp.enabled_tools.contains(&"analyze_dead_code".to_string()));
    assert!(config.mcp.enabled_tools.contains(&"quality_gate".to_string()));
    assert!(config.mcp.enabled_tools.contains(&"refactor_start".to_string()));
}

#[test]
fn test_default_config_roadmap_defaults() {
    let config = ConfigurationService::default_config();

    assert_eq!(
        config.roadmap.roadmap_path,
        PathBuf::from("docs/execution/roadmap.md")
    );
    assert!(config.roadmap.auto_generate_todos);
    assert!(config.roadmap.enforce_quality_gates);
    assert!(config.roadmap.require_task_ids);
    assert_eq!(config.roadmap.task_id_pattern, "PMAT-[0-9]{4}");
    assert!(config.roadmap.velocity_tracking);
    assert!(config.roadmap.burndown_charts);
}

#[test]
fn test_default_config_git_defaults() {
    let config = ConfigurationService::default_config();

    // CRITICAL: Verify zero-branching policy is enforced
    assert!(!config.roadmap.git.create_branches, "Zero-branching policy must be enforced");
    assert_eq!(config.roadmap.git.branch_pattern, "feature/{task_id}");
    assert_eq!(config.roadmap.git.commit_pattern, "{task_id}: {message}");
    assert!(config.roadmap.git.require_quality_check);
}

#[test]
fn test_default_config_telemetry_defaults() {
    let config = ConfigurationService::default_config();

    assert!(config.telemetry.enabled);
    assert_eq!(config.telemetry.collection_interval_seconds, 60);
    assert_eq!(config.telemetry.max_data_age_days, 30);
    assert!(config.telemetry.enable_aggregation);
    assert!(!config.telemetry.enable_export);
    assert_eq!(config.telemetry.export_format, "json");
}

#[test]
fn test_default_config_semantic_defaults() {
    let config = ConfigurationService::default_config();

    // Semantic search is disabled by default (uses local embeddings, no API key needed)
    assert!(!config.semantic.enabled);
    assert!(config.semantic.vector_db_path.is_none());
    assert!(config.semantic.workspace_path.is_none());
    assert_eq!(config.semantic.embedding_model, "aprender-tfidf-local");
    assert_eq!(config.semantic.embedding_dimensions, 256);
    assert_eq!(config.semantic.default_search_mode, "hybrid");
    assert_eq!(config.semantic.default_limit, 10);
    assert!(!config.semantic.auto_sync);
    assert_eq!(config.semantic.sync_interval_seconds, 300);
    assert_eq!(config.semantic.max_chunk_tokens, 500);
    assert!(!config.semantic.supported_languages.is_empty());
}

#[test]
fn test_default_config_custom_is_empty() {
    let config = ConfigurationService::default_config();
    assert!(config.custom.is_empty());
}

// ============================================================================
// PmatConfig serialization tests
// ============================================================================

#[test]
fn test_pmat_config_serialization_roundtrip() {
    let config = ConfigurationService::default_config();

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: PmatConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.system.project_name, config.system.project_name);
    assert_eq!(deserialized.quality.max_complexity, config.quality.max_complexity);
    assert_eq!(deserialized.analysis.max_file_size, config.analysis.max_file_size);
}

#[test]
fn test_quality_config_serialization() {
    let config = QualityConfig {
        max_complexity: 25,
        max_cognitive_complexity: 20,
        min_coverage: 90.0,
        allow_satd: true,
        require_docs: false,
        lint_compliance: true,
        fail_on_violation: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: QualityConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.max_complexity, 25);
    assert_eq!(deserialized.max_cognitive_complexity, 20);
    assert!((deserialized.min_coverage - 90.0).abs() < f64::EPSILON);
    assert!(deserialized.allow_satd);
    assert!(!deserialized.require_docs);
}

#[test]
fn test_analysis_config_serialization() {
    let config = AnalysisConfig {
        include_patterns: vec!["**/*.py".to_string()],
        exclude_patterns: vec!["**/__pycache__/**".to_string()],
        max_file_size: 500_000,
        max_line_length: 120,
        skip_vendor: false,
        parallel: false,
        thread_count: 4,
        timeout_seconds: 60,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AnalysisConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.include_patterns, vec!["**/*.py".to_string()]);
    assert_eq!(deserialized.max_file_size, 500_000);
    assert_eq!(deserialized.thread_count, 4);
}

#[test]
fn test_performance_config_serialization() {
    let config = PerformanceConfig {
        enable_regression_tests: false,
        enable_memory_tests: false,
        enable_throughput_tests: true,
        test_iterations: 5,
        timeout_ms: 15000,
        target_startup_latency_ms: 100,
        target_throughput_loc_per_sec: 100000,
        target_memory_mb: 64,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: PerformanceConfig = serde_json::from_str(&json).unwrap();

    assert!(!deserialized.enable_regression_tests);
    assert!(deserialized.enable_throughput_tests);
    assert_eq!(deserialized.target_memory_mb, 64);
}

#[test]
fn test_mcp_config_serialization() {
    let config = McpConfig {
        server_name: "test-server".to_string(),
        server_version: "1.0.0".to_string(),
        enable_compression: false,
        request_timeout_seconds: 60,
        max_request_size: 5_000_000,
        log_requests: true,
        enabled_tools: vec!["tool1".to_string(), "tool2".to_string()],
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: McpConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.server_name, "test-server");
    assert!(!deserialized.enable_compression);
    assert!(deserialized.log_requests);
    assert_eq!(deserialized.enabled_tools.len(), 2);
}

#[test]
fn test_telemetry_config_serialization() {
    let config = TelemetryConfig {
        enabled: false,
        collection_interval_seconds: 120,
        max_data_age_days: 7,
        enable_aggregation: false,
        enable_export: true,
        export_format: "csv".to_string(),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: TelemetryConfig = serde_json::from_str(&json).unwrap();

    assert!(!deserialized.enabled);
    assert_eq!(deserialized.collection_interval_seconds, 120);
    assert!(deserialized.enable_export);
    assert_eq!(deserialized.export_format, "csv");
}

// ============================================================================
// Config validation tests
// ============================================================================

#[test]
fn test_quality_config_threshold_boundaries() {
    // Test that defaults are reasonable
    let config = ConfigurationService::default_config();

    // Complexity thresholds should be positive
    assert!(config.quality.max_complexity > 0);
    assert!(config.quality.max_cognitive_complexity > 0);

    // Coverage should be between 0 and 100
    assert!(config.quality.min_coverage >= 0.0);
    assert!(config.quality.min_coverage <= 100.0);
}

#[test]
fn test_analysis_config_reasonable_defaults() {
    let config = ConfigurationService::default_config();

    // Max file size should be reasonable (at least 1KB, at most 100MB)
    assert!(config.analysis.max_file_size >= 1024);
    assert!(config.analysis.max_file_size <= 100 * 1024 * 1024);

    // Line length should be reasonable (at least 40, at most 500)
    assert!(config.analysis.max_line_length >= 40);
    assert!(config.analysis.max_line_length <= 500);

    // Timeout should be reasonable (at least 1 second, at most 1 hour)
    assert!(config.analysis.timeout_seconds >= 1);
    assert!(config.analysis.timeout_seconds <= 3600);
}

#[test]
fn test_performance_config_reasonable_defaults() {
    let config = ConfigurationService::default_config();

    // Iterations should be at least 1
    assert!(config.performance.test_iterations >= 1);

    // Timeout should be reasonable
    assert!(config.performance.timeout_ms >= 1000);
    assert!(config.performance.timeout_ms <= 600_000);

    // Targets should be positive
    assert!(config.performance.target_startup_latency_ms > 0);
    assert!(config.performance.target_throughput_loc_per_sec > 0);
    assert!(config.performance.target_memory_mb > 0);
}

#[test]
fn test_mcp_config_reasonable_defaults() {
    let config = ConfigurationService::default_config();

    // Request size should be reasonable
    assert!(config.mcp.max_request_size >= 1024); // At least 1KB
    assert!(config.mcp.max_request_size <= 100 * 1024 * 1024); // At most 100MB

    // Timeout should be reasonable
    assert!(config.mcp.request_timeout_seconds >= 1);
    assert!(config.mcp.request_timeout_seconds <= 300);
}

#[test]
fn test_semantic_config_reasonable_defaults() {
    let config = ConfigurationService::default_config();

    // Embedding dimensions should be positive and reasonable
    assert!(config.semantic.embedding_dimensions > 0);
    assert!(config.semantic.embedding_dimensions <= 4096);

    // Default limit should be reasonable
    assert!(config.semantic.default_limit >= 1);
    assert!(config.semantic.default_limit <= 100);

    // Sync interval should be reasonable
    assert!(config.semantic.sync_interval_seconds >= 10);

    // Max chunk tokens should be reasonable
    assert!(config.semantic.max_chunk_tokens >= 100);
    assert!(config.semantic.max_chunk_tokens <= 100_000);
}

// ============================================================================
// SystemConfig tests
// ============================================================================

#[test]
fn test_system_config_clone() {
    let config = SystemConfig {
        project_name: "test".to_string(),
        project_path: PathBuf::from("/test"),
        output_dir: PathBuf::from("/output"),
        max_concurrent_operations: 8,
        verbose: true,
        debug: true,
        default_toolchain: "python".to_string(),
    };

    let cloned = config.clone();
    assert_eq!(cloned.project_name, config.project_name);
    assert_eq!(cloned.max_concurrent_operations, config.max_concurrent_operations);
}

#[test]
fn test_system_config_debug_format() {
    let config = SystemConfig {
        project_name: "test".to_string(),
        project_path: PathBuf::from("/test"),
        output_dir: PathBuf::from("/output"),
        max_concurrent_operations: 8,
        verbose: false,
        debug: false,
        default_toolchain: "rust".to_string(),
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("test"));
    assert!(debug_str.contains("rust"));
}

// ============================================================================
// RoadmapConfig tests
// ============================================================================

#[test]
fn test_roadmap_config_task_id_pattern_is_valid_regex() {
    let config = ConfigurationService::default_config();

    // The task ID pattern should be a valid regex
    let result = regex::Regex::new(&config.roadmap.task_id_pattern);
    assert!(result.is_ok(), "Task ID pattern should be valid regex");

    // Test that the pattern matches expected task IDs
    let regex = result.unwrap();
    assert!(regex.is_match("PMAT-1234"));
    assert!(regex.is_match("PMAT-0001"));
    assert!(regex.is_match("PMAT-9999"));
    // Note: regex matches substrings, so PMAT-12345 contains PMAT-1234 which matches
    // This is expected behavior - the pattern is for extraction, not full validation
    assert!(regex.is_match("PMAT-12345")); // Contains PMAT-1234 (matches)
    assert!(!regex.is_match("PMAT-123")); // Only 3 digits, doesn't match
    assert!(!regex.is_match("INVALID")); // No pattern
}

#[test]
fn test_roadmap_git_config_patterns() {
    let config = ConfigurationService::default_config();

    // Branch pattern should contain {task_id} placeholder
    assert!(config.roadmap.git.branch_pattern.contains("{task_id}"));

    // Commit pattern should contain {task_id} and {message} placeholders
    assert!(config.roadmap.git.commit_pattern.contains("{task_id}"));
    assert!(config.roadmap.git.commit_pattern.contains("{message}"));
}
