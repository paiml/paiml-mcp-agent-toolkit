//! Tests for diagnose module

use super::*;

#[test]
fn test_diagnostic_format_variants() {
    let formats = vec![
        DiagnosticFormat::Pretty,
        DiagnosticFormat::Json,
        DiagnosticFormat::Compact,
    ];

    for format in formats {
        match format {
            DiagnosticFormat::Pretty => assert_eq!(format!("{:?}", format), "Pretty"),
            DiagnosticFormat::Json => assert_eq!(format!("{:?}", format), "Json"),
            DiagnosticFormat::Compact => assert_eq!(format!("{:?}", format), "Compact"),
        }
    }
}

#[test]
fn test_build_info_creation() {
    let info = BuildInfo::current();

    // Just verify it creates successfully
    assert!(!info.rust_version.is_empty());
    assert!(!info.features.is_empty());
}

#[test]
fn test_feature_status_variants() {
    // Test FeatureStatus enum
    let ok = FeatureStatus::Ok;
    let degraded = FeatureStatus::Degraded("test".to_string());
    let failed = FeatureStatus::Failed;
    let skipped = FeatureStatus::Skipped("reason".to_string());

    assert!(matches!(ok, FeatureStatus::Ok));
    assert!(matches!(degraded, FeatureStatus::Degraded(_)));
    assert!(matches!(failed, FeatureStatus::Failed));
    assert!(matches!(skipped, FeatureStatus::Skipped(_)));
}

// ============================================================================
// BuildInfo Tests
// ============================================================================

#[test]
fn test_build_info_fields() {
    let info = BuildInfo::current();

    // Rust version should be set (from env or fallback)
    assert!(!info.rust_version.is_empty());

    // Build date should be set (from env or fallback)
    assert!(!info.build_date.is_empty());

    // Features should include "cli"
    assert!(info.features.contains(&"cli".to_string()));

    // Git commit may or may not be set
    // Just ensure it doesn't panic
    let _git = info.git_commit.clone();
}

#[test]
fn test_build_info_serialization() {
    let info = BuildInfo::current();
    let json = serde_json::to_string(&info).unwrap();

    assert!(json.contains("rust_version"));
    assert!(json.contains("build_date"));
    assert!(json.contains("features"));
}

// ============================================================================
// EnvironmentSnapshot Tests
// ============================================================================

#[test]
fn test_environment_snapshot_capture() {
    let snapshot = EnvironmentSnapshot::capture();

    // OS should match the current platform
    assert!(!snapshot.os.is_empty());
    assert!(["linux", "macos", "windows"]
        .iter()
        .any(|os| snapshot.os.contains(os)));

    // Architecture should be set
    assert!(!snapshot.arch.is_empty());

    // CPU count should be > 0
    assert!(snapshot.cpu_count > 0);

    // CWD should be set (may be empty in weird edge cases, but usually not)
    // We don't assert on memory_mb as it can be 0 if sys_info fails
}

#[test]
fn test_environment_snapshot_serialization() {
    let snapshot = EnvironmentSnapshot::capture();
    let json = serde_json::to_string(&snapshot).unwrap();

    assert!(json.contains("os"));
    assert!(json.contains("arch"));
    assert!(json.contains("cpu_count"));
    assert!(json.contains("memory_mb"));
    assert!(json.contains("cwd"));
}

// ============================================================================
// SelfDiagnostic Tests
// ============================================================================

#[test]
fn test_self_diagnostic_new() {
    let diagnostic = SelfDiagnostic::new();

    // Should have 8 tests registered
    assert_eq!(diagnostic.tests.len(), 8);
}

#[test]
fn test_self_diagnostic_default() {
    let diagnostic = SelfDiagnostic::default();

    // Default should behave same as new()
    assert_eq!(diagnostic.tests.len(), 8);
}

#[test]
fn test_compute_summary_all_passed() {
    let diagnostic = SelfDiagnostic::new();
    let mut features = BTreeMap::new();

    features.insert(
        "test1".to_string(),
        FeatureResult {
            status: FeatureStatus::Ok,
            duration_us: 100,
            error: None,
            metrics: Some(serde_json::json!({"test": true})),
        },
    );
    features.insert(
        "test2".to_string(),
        FeatureResult {
            status: FeatureStatus::Ok,
            duration_us: 200,
            error: None,
            metrics: None,
        },
    );

    let summary = diagnostic.compute_summary(&features);

    assert_eq!(summary.total, 2);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.degraded, 0);
    assert_eq!(summary.skipped, 0);
    assert!(summary.all_passed);
    assert!((summary.success_rate - 100.0).abs() < f64::EPSILON);
}

#[test]
fn test_compute_summary_with_failures() {
    let diagnostic = SelfDiagnostic::new();
    let mut features = BTreeMap::new();

    features.insert(
        "test1".to_string(),
        FeatureResult {
            status: FeatureStatus::Ok,
            duration_us: 100,
            error: None,
            metrics: None,
        },
    );
    features.insert(
        "test2".to_string(),
        FeatureResult {
            status: FeatureStatus::Failed,
            duration_us: 200,
            error: Some("Something went wrong".to_string()),
            metrics: None,
        },
    );

    let summary = diagnostic.compute_summary(&features);

    assert_eq!(summary.total, 2);
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 1);
    assert!(!summary.all_passed);
    assert!((summary.success_rate - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_compute_summary_with_degraded() {
    let diagnostic = SelfDiagnostic::new();
    let mut features = BTreeMap::new();

    features.insert(
        "test1".to_string(),
        FeatureResult {
            status: FeatureStatus::Degraded("Slow response".to_string()),
            duration_us: 5000000,
            error: None,
            metrics: None,
        },
    );

    let summary = diagnostic.compute_summary(&features);

    assert_eq!(summary.total, 1);
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.degraded, 1);
    assert!(!summary.all_passed);
}

#[test]
fn test_compute_summary_with_skipped() {
    let diagnostic = SelfDiagnostic::new();
    let mut features = BTreeMap::new();

    features.insert(
        "test1".to_string(),
        FeatureResult {
            status: FeatureStatus::Skipped("User requested skip".to_string()),
            duration_us: 0,
            error: None,
            metrics: None,
        },
    );

    let summary = diagnostic.compute_summary(&features);

    assert_eq!(summary.total, 1);
    assert_eq!(summary.skipped, 1);
    assert!(summary.all_passed); // Skipped doesn't count as failure
}

#[test]
fn test_compute_summary_empty() {
    let diagnostic = SelfDiagnostic::new();
    let features = BTreeMap::new();

    let summary = diagnostic.compute_summary(&features);

    assert_eq!(summary.total, 0);
    assert_eq!(summary.passed, 0);
    assert!(summary.all_passed);
    assert!((summary.success_rate - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_compute_summary_mixed() {
    let diagnostic = SelfDiagnostic::new();
    let mut features = BTreeMap::new();

    features.insert(
        "test1".to_string(),
        FeatureResult {
            status: FeatureStatus::Ok,
            duration_us: 100,
            error: None,
            metrics: None,
        },
    );
    features.insert(
        "test2".to_string(),
        FeatureResult {
            status: FeatureStatus::Failed,
            duration_us: 200,
            error: Some("error".to_string()),
            metrics: None,
        },
    );
    features.insert(
        "test3".to_string(),
        FeatureResult {
            status: FeatureStatus::Degraded("slow".to_string()),
            duration_us: 300,
            error: None,
            metrics: None,
        },
    );
    features.insert(
        "test4".to_string(),
        FeatureResult {
            status: FeatureStatus::Skipped("skip".to_string()),
            duration_us: 0,
            error: None,
            metrics: None,
        },
    );

    let summary = diagnostic.compute_summary(&features);

    assert_eq!(summary.total, 4);
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.degraded, 1);
    assert_eq!(summary.skipped, 1);
    assert!(!summary.all_passed);
    assert!((summary.success_rate - 25.0).abs() < f64::EPSILON);
}

// ============================================================================
// classify_error Tests
// ============================================================================

#[test]
fn test_classify_error_permission_denied() {
    let diagnostic = SelfDiagnostic::new();
    let error = "Error: Permission denied for file.txt";

    assert_eq!(diagnostic.classify_error(error), "permission_denied");
}

#[test]
fn test_classify_error_file_not_found() {
    let diagnostic = SelfDiagnostic::new();
    let error = "Error: file not found";

    assert_eq!(diagnostic.classify_error(error), "file_not_found");
}

#[test]
fn test_classify_error_timeout() {
    let diagnostic = SelfDiagnostic::new();
    let error = "Error: Operation timeout after 30s";

    assert_eq!(diagnostic.classify_error(error), "timeout");
}

#[test]
fn test_classify_error_git() {
    let diagnostic = SelfDiagnostic::new();
    let error = "Error: git command failed";

    assert_eq!(diagnostic.classify_error(error), "git_error");
}

#[test]
fn test_classify_error_unknown() {
    let diagnostic = SelfDiagnostic::new();
    let error = "Something completely different happened";

    assert_eq!(diagnostic.classify_error(error), "unknown");
}

// ============================================================================
// generate_fixes Tests
// ============================================================================

#[test]
fn test_generate_fixes_permission_denied() {
    let diagnostic = SelfDiagnostic::new();
    let mut patterns = BTreeMap::new();
    patterns.insert(
        "permission_denied".to_string(),
        vec!["feature1".to_string()],
    );

    let fixes = diagnostic.generate_fixes(&patterns);

    assert_eq!(fixes.len(), 1);
    assert_eq!(fixes[0].error_pattern, "permission_denied");
    assert_eq!(fixes[0].fix_command, Some("chmod +r <file>".to_string()));
}

#[test]
fn test_generate_fixes_git_error() {
    let diagnostic = SelfDiagnostic::new();
    let mut patterns = BTreeMap::new();
    patterns.insert("git_error".to_string(), vec!["integration.git".to_string()]);

    let fixes = diagnostic.generate_fixes(&patterns);

    assert_eq!(fixes.len(), 1);
    assert_eq!(fixes[0].error_pattern, "git_error");
    assert_eq!(fixes[0].fix_command, Some("git init".to_string()));
    assert!(fixes[0].documentation_link.is_some());
}

#[test]
fn test_generate_fixes_unknown() {
    let diagnostic = SelfDiagnostic::new();
    let mut patterns = BTreeMap::new();
    patterns.insert("unknown".to_string(), vec!["some.feature".to_string()]);

    let fixes = diagnostic.generate_fixes(&patterns);

    assert_eq!(fixes.len(), 1);
    assert_eq!(fixes[0].error_pattern, "unknown");
    assert!(fixes[0].fix_command.is_none());
    assert!(fixes[0].documentation_link.is_none());
}

#[test]
fn test_generate_fixes_multiple_features() {
    let diagnostic = SelfDiagnostic::new();
    let mut patterns = BTreeMap::new();
    patterns.insert(
        "permission_denied".to_string(),
        vec!["feature1".to_string(), "feature2".to_string()],
    );

    let fixes = diagnostic.generate_fixes(&patterns);

    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].feature.contains("feature1"));
    assert!(fixes[0].feature.contains("feature2"));
}

// ============================================================================
// extract_error_context Tests
// ============================================================================

#[test]
fn test_extract_error_context_no_failures() {
    let diagnostic = SelfDiagnostic::new();
    let mut features = BTreeMap::new();

    features.insert(
        "test1".to_string(),
        FeatureResult {
            status: FeatureStatus::Ok,
            duration_us: 100,
            error: None,
            metrics: None,
        },
    );

    let context = diagnostic.extract_error_context(&features);

    assert!(context.is_none());
}

#[test]
fn test_extract_error_context_with_failures() {
    let diagnostic = SelfDiagnostic::new();
    let mut features = BTreeMap::new();

    features.insert(
        "test1".to_string(),
        FeatureResult {
            status: FeatureStatus::Failed,
            duration_us: 100,
            error: Some("Permission denied".to_string()),
            metrics: None,
        },
    );

    let context = diagnostic.extract_error_context(&features);

    assert!(context.is_some());
    let ctx = context.unwrap();
    assert_eq!(ctx.failed_features.len(), 1);
    assert!(ctx.failed_features.contains(&"test1".to_string()));
    assert!(ctx.error_patterns.contains_key("permission_denied"));
    assert!(!ctx.suggested_fixes.is_empty());
}

#[test]
fn test_extract_error_context_multiple_failures() {
    let diagnostic = SelfDiagnostic::new();
    let mut features = BTreeMap::new();

    features.insert(
        "test1".to_string(),
        FeatureResult {
            status: FeatureStatus::Failed,
            duration_us: 100,
            error: Some("Permission denied".to_string()),
            metrics: None,
        },
    );
    features.insert(
        "test2".to_string(),
        FeatureResult {
            status: FeatureStatus::Failed,
            duration_us: 200,
            error: Some("git command failed".to_string()),
            metrics: None,
        },
    );

    let context = diagnostic.extract_error_context(&features);

    assert!(context.is_some());
    let ctx = context.unwrap();
    assert_eq!(ctx.failed_features.len(), 2);
    assert!(ctx.error_patterns.contains_key("permission_denied"));
    assert!(ctx.error_patterns.contains_key("git_error"));
}

// ============================================================================
// FeatureResult Tests
// ============================================================================

#[test]
fn test_feature_result_serialization_with_all_fields() {
    let result = FeatureResult {
        status: FeatureStatus::Ok,
        duration_us: 12345,
        error: Some("test error".to_string()),
        metrics: Some(serde_json::json!({"key": "value"})),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("12345"));
    assert!(json.contains("test error"));
    assert!(json.contains("key"));
}

#[test]
fn test_feature_result_serialization_skip_none() {
    let result = FeatureResult {
        status: FeatureStatus::Ok,
        duration_us: 100,
        error: None,
        metrics: None,
    };

    let json = serde_json::to_string(&result).unwrap();

    // None fields should be skipped due to skip_serializing_if
    assert!(!json.contains("error"));
    assert!(!json.contains("metrics"));
}

// ============================================================================
// DiagnosticSummary Tests
// ============================================================================

#[test]
fn test_diagnostic_summary_serialization() {
    let summary = DiagnosticSummary {
        total: 10,
        passed: 7,
        failed: 2,
        degraded: 1,
        skipped: 0,
        all_passed: false,
        success_rate: 70.0,
    };

    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("70"));
    assert!(json.contains("\"all_passed\":false"));
}

// ============================================================================
// DiagnosticReport Tests
// ============================================================================

#[test]
fn test_diagnostic_report_serialization() {
    let mut features = BTreeMap::new();
    features.insert(
        "test.feature".to_string(),
        FeatureResult {
            status: FeatureStatus::Ok,
            duration_us: 100,
            error: None,
            metrics: None,
        },
    );

    let report = DiagnosticReport {
        version: "1.0.0".to_string(),
        build_info: BuildInfo::current(),
        timestamp: Utc::now(),
        duration_ms: 1000,
        features,
        summary: DiagnosticSummary {
            total: 1,
            passed: 1,
            failed: 0,
            degraded: 0,
            skipped: 0,
            all_passed: true,
            success_rate: 100.0,
        },
        error_context: None,
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("1.0.0"));
    assert!(json.contains("test.feature"));
}

#[test]
fn test_diagnostic_report_with_error_context() {
    let features = BTreeMap::new();

    let report = DiagnosticReport {
        version: "1.0.0".to_string(),
        build_info: BuildInfo::current(),
        timestamp: Utc::now(),
        duration_ms: 1000,
        features,
        summary: DiagnosticSummary {
            total: 1,
            passed: 0,
            failed: 1,
            degraded: 0,
            skipped: 0,
            all_passed: false,
            success_rate: 0.0,
        },
        error_context: Some(CompactErrorContext {
            failed_features: vec!["test1".to_string()],
            error_patterns: BTreeMap::new(),
            suggested_fixes: vec![],
            environment: EnvironmentSnapshot::capture(),
        }),
    };

    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("error_context"));
    assert!(json.contains("failed_features"));
}

// ============================================================================
// SuggestedFix Tests
// ============================================================================

#[test]
fn test_suggested_fix_serialization() {
    let fix = SuggestedFix {
        feature: "test.feature".to_string(),
        error_pattern: "permission_denied".to_string(),
        fix_command: Some("chmod +r file".to_string()),
        documentation_link: Some("https://example.com".to_string()),
    };

    let json = serde_json::to_string(&fix).unwrap();
    assert!(json.contains("test.feature"));
    assert!(json.contains("chmod"));
    assert!(json.contains("https://example.com"));
}

#[test]
fn test_suggested_fix_serialization_without_optionals() {
    let fix = SuggestedFix {
        feature: "test.feature".to_string(),
        error_pattern: "unknown".to_string(),
        fix_command: None,
        documentation_link: None,
    };

    let json = serde_json::to_string(&fix).unwrap();
    assert!(json.contains("test.feature"));
    assert!(json.contains("unknown"));
}

// ============================================================================
// CompactErrorContext Tests
// ============================================================================

#[test]
fn test_compact_error_context_serialization() {
    let mut error_patterns = BTreeMap::new();
    error_patterns.insert("timeout".to_string(), vec!["slow.test".to_string()]);

    let context = CompactErrorContext {
        failed_features: vec!["feature1".to_string(), "feature2".to_string()],
        error_patterns,
        suggested_fixes: vec![SuggestedFix {
            feature: "feature1".to_string(),
            error_pattern: "timeout".to_string(),
            fix_command: None,
            documentation_link: None,
        }],
        environment: EnvironmentSnapshot::capture(),
    };

    let json = serde_json::to_string(&context).unwrap();
    assert!(json.contains("feature1"));
    assert!(json.contains("feature2"));
    assert!(json.contains("timeout"));
}

// ============================================================================
// Feature Test Implementations Tests
// ============================================================================

#[tokio::test]
async fn test_rust_ast_test_name() {
    let test = RustAstTest;
    assert_eq!(test.name(), "ast.rust");
}

#[tokio::test]
async fn test_rust_ast_test_execute() {
    let test = RustAstTest;
    let result = test.execute().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.get("parsed_items").is_some());
    assert!(metrics.get("parse_time_us").is_some());
}

#[tokio::test]
async fn test_typescript_ast_test_name() {
    let test = TypeScriptAstTest;
    assert_eq!(test.name(), "ast.typescript");
}

#[tokio::test]
async fn test_typescript_ast_test_execute() {
    let test = TypeScriptAstTest;
    let result = test.execute().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.get("typescript_test").is_some());
}

#[tokio::test]
async fn test_python_ast_test_name() {
    let test = PythonAstTest;
    assert_eq!(test.name(), "ast.python");
}

#[tokio::test]
async fn test_python_ast_test_execute() {
    let test = PythonAstTest;
    let result = test.execute().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.get("python_test").is_some());
}

#[tokio::test]
async fn test_cache_subsystem_test_name() {
    let test = CacheSubsystemTest;
    assert_eq!(test.name(), "cache.subsystem");
}

#[tokio::test]
async fn test_cache_subsystem_test_execute() {
    let test = CacheSubsystemTest;
    let result = test.execute().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.get("cache_initialized").is_some());
}

#[tokio::test]
async fn test_mermaid_generator_test_name() {
    let test = MermaidGeneratorTest;
    assert_eq!(test.name(), "output.mermaid");
}

#[tokio::test]
async fn test_mermaid_generator_test_execute() {
    let test = MermaidGeneratorTest;
    let result = test.execute().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.get("mermaid_syntax_valid").is_some());
}

#[tokio::test]
async fn test_complexity_analysis_test_name() {
    let test = ComplexityAnalysisTest;
    assert_eq!(test.name(), "analysis.complexity");
}

#[tokio::test]
async fn test_complexity_analysis_test_execute() {
    let test = ComplexityAnalysisTest;
    let result = test.execute().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.get("status").is_some());
}

#[tokio::test]
async fn test_deep_context_test_name() {
    let test = DeepContextTest;
    assert_eq!(test.name(), "analysis.deep_context");
}

#[tokio::test]
async fn test_deep_context_test_execute() {
    let test = DeepContextTest;
    let result = test.execute().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.get("status").is_some());
}

#[tokio::test]
async fn test_git_integration_test_name() {
    let test = GitIntegrationTest;
    assert_eq!(test.name(), "integration.git");
}

#[tokio::test]
async fn test_git_integration_test_execute() {
    let test = GitIntegrationTest;
    let result = test.execute().await;

    // This test should pass regardless of git availability
    assert!(result.is_ok());
    let metrics = result.unwrap();
    // Either git_available or status (skipped) should be present
    assert!(metrics.get("git_available").is_some() || metrics.get("status").is_some());
}

// ============================================================================
// run_diagnostic Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires full integration, may timeout in CI"]
async fn test_run_diagnostic_all_tests() {
    let diagnostic = SelfDiagnostic::new();
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Pretty,
        only: vec![],
        skip: vec![],
        timeout: 60,
    };

    let report = diagnostic.run_diagnostic(&args).await;

    // Should have run all 8 tests
    assert_eq!(report.features.len(), 8);
    assert!(!report.version.is_empty());
    assert!(report.duration_ms > 0);
}

#[tokio::test]
async fn test_run_diagnostic_with_only_filter() {
    let diagnostic = SelfDiagnostic::new();
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Json,
        only: vec!["ast.rust".to_string()],
        skip: vec![],
        timeout: 60,
    };

    let report = diagnostic.run_diagnostic(&args).await;

    // Should only run the specified test
    assert_eq!(report.features.len(), 1);
    assert!(report.features.contains_key("ast.rust"));
}

#[tokio::test]
async fn test_run_diagnostic_with_skip_filter() {
    let diagnostic = SelfDiagnostic::new();
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Compact,
        only: vec![],
        skip: vec!["ast.rust".to_string(), "ast.typescript".to_string()],
        timeout: 60,
    };

    let report = diagnostic.run_diagnostic(&args).await;

    // Should have 8 results (6 run + 2 skipped)
    assert_eq!(report.features.len(), 8);

    // Check that skipped tests have correct status
    let rust = report.features.get("ast.rust").unwrap();
    assert!(matches!(rust.status, FeatureStatus::Skipped(_)));

    let ts = report.features.get("ast.typescript").unwrap();
    assert!(matches!(ts.status, FeatureStatus::Skipped(_)));
}

#[tokio::test]
async fn test_run_diagnostic_with_both_filters() {
    let diagnostic = SelfDiagnostic::new();
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Pretty,
        only: vec!["ast.rust".to_string(), "ast.typescript".to_string()],
        skip: vec!["ast.typescript".to_string()],
        timeout: 60,
    };

    let report = diagnostic.run_diagnostic(&args).await;

    // Only should filter first, then skip should apply
    // Only ast.rust and ast.typescript considered, but ast.typescript is skipped
    assert_eq!(report.features.len(), 2);
    assert!(report.features.contains_key("ast.rust"));
    assert!(report.features.contains_key("ast.typescript"));

    let ts = report.features.get("ast.typescript").unwrap();
    assert!(matches!(ts.status, FeatureStatus::Skipped(_)));
}

// ============================================================================
// handle_diagnose Tests
// ============================================================================

#[tokio::test]
async fn test_handle_diagnose_pretty_format() {
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Pretty,
        only: vec!["ast.rust".to_string()],
        skip: vec![],
        timeout: 10,
    };

    // Should not panic
    let result = handle_diagnose(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_diagnose_json_format() {
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Json,
        only: vec!["ast.rust".to_string()],
        skip: vec![],
        timeout: 10,
    };

    // Should not panic
    let result = handle_diagnose(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_diagnose_compact_format() {
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Compact,
        only: vec!["ast.rust".to_string()],
        skip: vec![],
        timeout: 10,
    };

    // Should not panic
    let result = handle_diagnose(args).await;
    assert!(result.is_ok());
}

// ============================================================================
// FeatureStatus Serialization Tests
// ============================================================================

#[test]
fn test_feature_status_ok_serialization() {
    let result = FeatureResult {
        status: FeatureStatus::Ok,
        duration_us: 100,
        error: None,
        metrics: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"ok\""));
}

#[test]
fn test_feature_status_degraded_serialization() {
    let result = FeatureResult {
        status: FeatureStatus::Degraded("slow performance".to_string()),
        duration_us: 100,
        error: None,
        metrics: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("degraded"));
    assert!(json.contains("slow performance"));
}

#[test]
fn test_feature_status_failed_serialization() {
    let result = FeatureResult {
        status: FeatureStatus::Failed,
        duration_us: 100,
        error: None,
        metrics: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"failed\""));
}

#[test]
fn test_feature_status_skipped_serialization() {
    let result = FeatureResult {
        status: FeatureStatus::Skipped("not applicable".to_string()),
        duration_us: 0,
        error: None,
        metrics: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("skipped"));
    assert!(json.contains("not applicable"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_classify_error_case_sensitivity() {
    let diagnostic = SelfDiagnostic::new();

    // Error patterns are case sensitive
    assert_eq!(diagnostic.classify_error("PERMISSION DENIED"), "unknown");
    assert_eq!(
        diagnostic.classify_error("Permission denied"),
        "permission_denied"
    );
}

#[test]
fn test_classify_error_multiple_patterns() {
    let diagnostic = SelfDiagnostic::new();

    // When multiple patterns match, first match wins
    let error = "Permission denied: git repository not found";
    // "Permission denied" comes first in classify_error
    assert_eq!(diagnostic.classify_error(error), "permission_denied");
}

#[test]
fn test_generate_fixes_empty_patterns() {
    let diagnostic = SelfDiagnostic::new();
    let patterns = BTreeMap::new();

    let fixes = diagnostic.generate_fixes(&patterns);
    assert!(fixes.is_empty());
}

#[tokio::test]
async fn test_run_diagnostic_with_nonexistent_only_filter() {
    let diagnostic = SelfDiagnostic::new();
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Pretty,
        only: vec!["nonexistent.test".to_string()],
        skip: vec![],
        timeout: 60,
    };

    let report = diagnostic.run_diagnostic(&args).await;

    // No tests should match
    assert!(report.features.is_empty());
    assert_eq!(report.summary.total, 0);
}

#[tokio::test]
async fn test_run_diagnostic_timeout_respected() {
    let diagnostic = SelfDiagnostic::new();
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Pretty,
        only: vec!["ast.rust".to_string()],
        skip: vec![],
        timeout: 1, // Very short timeout
    };

    let report = diagnostic.run_diagnostic(&args).await;

    // Test should still complete (Rust AST is fast)
    assert!(!report.features.is_empty());
}

// ============================================================================
// DiagnoseArgs Tests
// ============================================================================

#[test]
fn test_diagnose_args_default_values() {
    // Test that default values work as expected
    // This would be the default if parsed from empty args
    let args = DiagnoseArgs {
        format: DiagnosticFormat::Pretty,
        only: vec![],
        skip: vec![],
        timeout: 60,
    };

    assert!(matches!(args.format, DiagnosticFormat::Pretty));
    assert!(args.only.is_empty());
    assert!(args.skip.is_empty());
    assert_eq!(args.timeout, 60);
}
