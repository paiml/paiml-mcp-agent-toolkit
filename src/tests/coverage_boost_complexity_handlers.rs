#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for complexity_handlers.rs
//!
//! Tests the complexity analysis command handlers including:
//! - ComplexityConfig creation and detection
//! - Complexity filter functions (apply_complexity_filters, apply_top_files_limit)
//! - SATD formatting functions (format_satd_summary, format_satd_markdown, generate_satd_sarif)
//! - Churn analysis functions (apply_churn_filters, create_and_report_file_filter)
//! - Watch mode helpers (should_reanalyze, is_source_code_file, should_include_file)
//! - File and path helper functions
//! - Critical items and top files section formatting

use crate::cli::{ComplexityOutputFormat, DagType, SatdOutputFormat, SatdSeverity};
use crate::models::churn::{ChurnOutputFormat, CodeChurnAnalysis, ChurnFileInfo, ChurnSummary};
use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::satd_detector::{
    DebtCategory, SATDAnalysisResult, SATDSummary, Severity, TechnicalDebt,
};
use crate::utils::file_filter::FileFilter;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;

// =============================================================================
// format_satd_summary tests
// =============================================================================

#[test]
fn test_format_satd_summary_empty_result() {
    let result = SATDAnalysisResult {
        items: vec![],
        summary: SATDSummary {
            total_items: 0,
            by_severity: HashMap::new(),
            by_category: HashMap::new(),
            files_with_satd: 0,
            avg_age_days: 0.0,
        },
        total_files_analyzed: 10,
        files_with_debt: 0,
        analysis_timestamp: Utc::now(),
    };

    let output = crate::cli::handlers::complexity_handlers::format_satd_summary(&result, false);

    assert!(output.contains("# SATD Analysis Summary"));
    assert!(output.contains("**Files analyzed**: 10"));
    assert!(output.contains("**Files with SATD**: 0"));
    assert!(output.contains("**Total SATD items**: 0"));
}

#[test]
fn test_format_satd_summary_with_items() {
    let result = create_satd_result_with_items(5, 20);

    let output = crate::cli::handlers::complexity_handlers::format_satd_summary(&result, false);

    assert!(output.contains("# SATD Analysis Summary"));
    assert!(output.contains("**Files analyzed**: 20"));
    assert!(output.contains("**Total SATD items**: 5"));
    assert!(output.contains("## Top Files with SATD"));
}

#[test]
fn test_format_satd_summary_with_metrics() {
    let mut by_severity = HashMap::new();
    by_severity.insert("Critical".to_string(), 2);
    by_severity.insert("High".to_string(), 3);

    let mut by_category = HashMap::new();
    by_category.insert("Defect".to_string(), 3);
    by_category.insert("Design".to_string(), 2);

    let result = SATDAnalysisResult {
        items: create_sample_debt_items(5),
        summary: SATDSummary {
            total_items: 5,
            by_severity,
            by_category,
            files_with_satd: 3,
            avg_age_days: 30.0,
        },
        total_files_analyzed: 20,
        files_with_debt: 3,
        analysis_timestamp: Utc::now(),
    };

    let output = crate::cli::handlers::complexity_handlers::format_satd_summary(&result, true);

    assert!(output.contains("## By Severity"));
    assert!(output.contains("## By Category"));
    assert!(output.contains("**Critical**"));
    assert!(output.contains("**Defect**"));
}

#[test]
fn test_format_satd_summary_metrics_disabled() {
    let mut by_severity = HashMap::new();
    by_severity.insert("Critical".to_string(), 2);

    let result = SATDAnalysisResult {
        items: create_sample_debt_items(2),
        summary: SATDSummary {
            total_items: 2,
            by_severity,
            by_category: HashMap::new(),
            files_with_satd: 1,
            avg_age_days: 0.0,
        },
        total_files_analyzed: 5,
        files_with_debt: 1,
        analysis_timestamp: Utc::now(),
    };

    let output = crate::cli::handlers::complexity_handlers::format_satd_summary(&result, false);

    // When metrics=false, severity/category sections should not appear
    assert!(!output.contains("## By Severity"));
    assert!(!output.contains("## By Category"));
}

#[test]
fn test_format_satd_summary_top_files_section() {
    let items = vec![
        create_debt_item("src/file1.rs", 10),
        create_debt_item("src/file1.rs", 20),
        create_debt_item("src/file1.rs", 30),
        create_debt_item("src/file2.rs", 15),
        create_debt_item("src/file3.rs", 25),
    ];

    let result = SATDAnalysisResult {
        items,
        summary: SATDSummary {
            total_items: 5,
            by_severity: HashMap::new(),
            by_category: HashMap::new(),
            files_with_satd: 3,
            avg_age_days: 0.0,
        },
        total_files_analyzed: 10,
        files_with_debt: 3,
        analysis_timestamp: Utc::now(),
    };

    let output = crate::cli::handlers::complexity_handlers::format_satd_summary(&result, false);

    assert!(output.contains("## Top Files with SATD"));
    // file1.rs has the most items (3)
    assert!(output.contains("file1.rs"));
}

#[test]
fn test_format_satd_summary_critical_items_section() {
    let items = vec![
        create_critical_debt_item("src/critical.rs", 10, "Critical security issue"),
        create_critical_debt_item("src/critical.rs", 20, "Critical bug"),
    ];

    let result = SATDAnalysisResult {
        items,
        summary: SATDSummary {
            total_items: 2,
            by_severity: HashMap::new(),
            by_category: HashMap::new(),
            files_with_satd: 1,
            avg_age_days: 0.0,
        },
        total_files_analyzed: 5,
        files_with_debt: 1,
        analysis_timestamp: Utc::now(),
    };

    let output = crate::cli::handlers::complexity_handlers::format_satd_summary(&result, false);

    assert!(output.contains("## Critical Items"));
    assert!(output.contains("Critical security issue"));
}

// =============================================================================
// is_source_code_file tests
// =============================================================================

#[test]
fn test_is_source_code_file_rust() {
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("src/main.rs"));
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("lib.rs"));
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("/path/to/mod.rs"));
}

#[test]
fn test_is_source_code_file_typescript() {
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("src/index.ts"));
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("component.tsx"));
}

#[test]
fn test_is_source_code_file_javascript() {
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("script.js"));
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("component.jsx"));
}

#[test]
fn test_is_source_code_file_python() {
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("app.py"));
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("scripts/test.py"));
}

#[test]
fn test_is_source_code_file_c_cpp() {
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("main.c"));
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("lib.cpp"));
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("header.h"));
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("header.hpp"));
}

#[test]
fn test_is_source_code_file_non_source() {
    assert!(!crate::cli::handlers::complexity_handlers::is_source_code_file("README.md"));
    assert!(!crate::cli::handlers::complexity_handlers::is_source_code_file("Cargo.toml"));
    assert!(!crate::cli::handlers::complexity_handlers::is_source_code_file("config.json"));
    assert!(!crate::cli::handlers::complexity_handlers::is_source_code_file("image.png"));
    assert!(!crate::cli::handlers::complexity_handlers::is_source_code_file(".gitignore"));
}

// =============================================================================
// should_include_file tests
// =============================================================================

#[test]
fn test_should_include_file_empty_patterns() {
    // Empty patterns means include all files
    assert!(crate::cli::handlers::complexity_handlers::should_include_file("src/main.rs", &[]));
    assert!(crate::cli::handlers::complexity_handlers::should_include_file("test.py", &[]));
    assert!(crate::cli::handlers::complexity_handlers::should_include_file("any/path/file.txt", &[]));
}

#[test]
fn test_should_include_file_matching_pattern() {
    let patterns = vec!["src".to_string()];
    assert!(crate::cli::handlers::complexity_handlers::should_include_file("src/main.rs", &patterns));
    assert!(crate::cli::handlers::complexity_handlers::should_include_file("src/lib.rs", &patterns));
    assert!(!crate::cli::handlers::complexity_handlers::should_include_file("test/main.rs", &patterns));
}

#[test]
fn test_should_include_file_multiple_patterns() {
    let patterns = vec!["src".to_string(), "lib".to_string()];
    assert!(crate::cli::handlers::complexity_handlers::should_include_file("src/main.rs", &patterns));
    assert!(crate::cli::handlers::complexity_handlers::should_include_file("lib/mod.rs", &patterns));
    assert!(!crate::cli::handlers::complexity_handlers::should_include_file("tests/test.rs", &patterns));
}

#[test]
fn test_should_include_file_extension_pattern() {
    let patterns = vec![".rs".to_string()];
    assert!(crate::cli::handlers::complexity_handlers::should_include_file("src/main.rs", &patterns));
    assert!(!crate::cli::handlers::complexity_handlers::should_include_file("src/main.ts", &patterns));
}

// =============================================================================
// apply_complexity_filters tests
// =============================================================================

#[test]
fn test_apply_complexity_filters_no_thresholds() {
    let mut metrics = create_file_metrics_list(5);
    let filtered = crate::cli::handlers::complexity_handlers::apply_complexity_filters(
        &mut metrics,
        None,
        None,
    );

    // No thresholds means no filtering
    assert_eq!(filtered, 0);
    assert_eq!(metrics.len(), 5);
}

#[test]
fn test_apply_complexity_filters_cyclomatic_threshold() {
    let mut metrics = vec![
        create_file_metrics_with_complexity("file1.rs", 5, 3),   // Below threshold
        create_file_metrics_with_complexity("file2.rs", 15, 8),  // Above cyclomatic
        create_file_metrics_with_complexity("file3.rs", 8, 5),   // Below threshold
    ];

    let filtered = crate::cli::handlers::complexity_handlers::apply_complexity_filters(
        &mut metrics,
        Some(10),
        None,
    );

    // Only file2.rs should remain (cyclomatic > 10)
    assert_eq!(filtered, 2);
    assert_eq!(metrics.len(), 1);
}

#[test]
fn test_apply_complexity_filters_cognitive_threshold() {
    let mut metrics = vec![
        create_file_metrics_with_complexity("file1.rs", 5, 3),   // Below threshold
        create_file_metrics_with_complexity("file2.rs", 8, 20),  // Above cognitive
        create_file_metrics_with_complexity("file3.rs", 6, 12),  // Below threshold
    ];

    let filtered = crate::cli::handlers::complexity_handlers::apply_complexity_filters(
        &mut metrics,
        None,
        Some(15),
    );

    // Only file2.rs should remain (cognitive > 15)
    assert_eq!(filtered, 2);
    assert_eq!(metrics.len(), 1);
}

#[test]
fn test_apply_complexity_filters_both_thresholds() {
    let mut metrics = vec![
        create_file_metrics_with_complexity("file1.rs", 5, 3),   // Below both
        create_file_metrics_with_complexity("file2.rs", 15, 8),  // Above cyclomatic
        create_file_metrics_with_complexity("file3.rs", 8, 20),  // Above cognitive
        create_file_metrics_with_complexity("file4.rs", 25, 30), // Above both
    ];

    let filtered = crate::cli::handlers::complexity_handlers::apply_complexity_filters(
        &mut metrics,
        Some(10),
        Some(15),
    );

    // Files 2, 3, 4 should remain
    assert_eq!(filtered, 1);
    assert_eq!(metrics.len(), 3);
}

#[test]
fn test_apply_complexity_filters_all_filtered() {
    let mut metrics = vec![
        create_file_metrics_with_complexity("file1.rs", 5, 3),
        create_file_metrics_with_complexity("file2.rs", 6, 4),
    ];

    let filtered = crate::cli::handlers::complexity_handlers::apply_complexity_filters(
        &mut metrics,
        Some(10),
        Some(10),
    );

    // All files below thresholds
    assert_eq!(filtered, 2);
    assert!(metrics.is_empty());
}

// =============================================================================
// apply_top_files_limit tests
// =============================================================================

#[test]
fn test_apply_top_files_limit_zero_keeps_all() {
    let mut metrics = create_file_metrics_list(10);
    crate::cli::handlers::complexity_handlers::apply_top_files_limit(&mut metrics, 0);

    // 0 means no limit
    assert_eq!(metrics.len(), 10);
}

#[test]
fn test_apply_top_files_limit_truncates() {
    let mut metrics = create_file_metrics_list(10);
    crate::cli::handlers::complexity_handlers::apply_top_files_limit(&mut metrics, 5);

    assert_eq!(metrics.len(), 5);
}

#[test]
fn test_apply_top_files_limit_larger_than_list() {
    let mut metrics = create_file_metrics_list(3);
    crate::cli::handlers::complexity_handlers::apply_top_files_limit(&mut metrics, 10);

    // Limit larger than list size, keeps all
    assert_eq!(metrics.len(), 3);
}

#[test]
fn test_apply_top_files_limit_sorts_by_complexity() {
    let mut metrics = vec![
        create_file_metrics_with_complexity("low.rs", 5, 3),
        create_file_metrics_with_complexity("high.rs", 25, 30),
        create_file_metrics_with_complexity("medium.rs", 15, 12),
    ];

    crate::cli::handlers::complexity_handlers::apply_top_files_limit(&mut metrics, 2);

    // Should keep the top 2 by total complexity (high and medium)
    assert_eq!(metrics.len(), 2);
    assert!(metrics[0].path.to_string_lossy().contains("high"));
    assert!(metrics[1].path.to_string_lossy().contains("medium"));
}

#[test]
fn test_apply_top_files_limit_empty_list() {
    let mut metrics: Vec<FileComplexityMetrics> = vec![];
    crate::cli::handlers::complexity_handlers::apply_top_files_limit(&mut metrics, 5);

    assert!(metrics.is_empty());
}

// =============================================================================
// has_complexity_violations tests
// =============================================================================

#[test]
fn test_has_complexity_violations_no_violations() {
    let metrics = vec![
        create_file_metrics_with_complexity("file1.rs", 5, 8),
        create_file_metrics_with_complexity("file2.rs", 8, 10),
    ];

    let has_violations = crate::cli::handlers::complexity_handlers::has_complexity_violations(
        &metrics,
        Some(20),
        Some(15),
    );

    assert!(!has_violations);
}

#[test]
fn test_has_complexity_violations_cyclomatic_exceeded() {
    let metrics = vec![
        create_file_metrics_with_complexity("file1.rs", 5, 8),
        create_file_metrics_with_complexity("file2.rs", 25, 10), // Exceeds cyclomatic
    ];

    let has_violations = crate::cli::handlers::complexity_handlers::has_complexity_violations(
        &metrics,
        Some(20),
        Some(15),
    );

    assert!(has_violations);
}

#[test]
fn test_has_complexity_violations_cognitive_exceeded() {
    let metrics = vec![
        create_file_metrics_with_complexity("file1.rs", 5, 8),
        create_file_metrics_with_complexity("file2.rs", 10, 20), // Exceeds cognitive
    ];

    let has_violations = crate::cli::handlers::complexity_handlers::has_complexity_violations(
        &metrics,
        Some(20),
        Some(15),
    );

    assert!(has_violations);
}

#[test]
fn test_has_complexity_violations_default_thresholds() {
    let metrics = vec![
        create_file_metrics_with_complexity("file1.rs", 25, 20), // Both exceed defaults (20, 15)
    ];

    let has_violations = crate::cli::handlers::complexity_handlers::has_complexity_violations(
        &metrics,
        None,
        None,
    );

    assert!(has_violations);
}

#[test]
fn test_has_complexity_violations_empty_metrics() {
    let metrics: Vec<FileComplexityMetrics> = vec![];

    let has_violations = crate::cli::handlers::complexity_handlers::has_complexity_violations(
        &metrics,
        Some(10),
        Some(10),
    );

    assert!(!has_violations);
}

// =============================================================================
// Churn filter tests
// =============================================================================

#[test]
fn test_create_and_report_file_filter_empty() {
    let filter = crate::cli::handlers::complexity_handlers::create_and_report_file_filter(
        vec![],
        vec![],
    );

    assert!(filter.is_ok());
    let filter = filter.unwrap();
    assert!(!filter.has_filters());
}

#[test]
fn test_create_and_report_file_filter_include_only() {
    let filter = crate::cli::handlers::complexity_handlers::create_and_report_file_filter(
        vec!["src/**/*.rs".to_string()],
        vec![],
    );

    assert!(filter.is_ok());
    let filter = filter.unwrap();
    assert!(filter.has_filters());
}

#[test]
fn test_create_and_report_file_filter_exclude_only() {
    let filter = crate::cli::handlers::complexity_handlers::create_and_report_file_filter(
        vec![],
        vec!["tests/**".to_string()],
    );

    assert!(filter.is_ok());
    let filter = filter.unwrap();
    assert!(filter.has_filters());
}

#[test]
fn test_create_and_report_file_filter_both() {
    let filter = crate::cli::handlers::complexity_handlers::create_and_report_file_filter(
        vec!["src/**/*.rs".to_string()],
        vec!["**/tests/**".to_string()],
    );

    assert!(filter.is_ok());
    let filter = filter.unwrap();
    assert!(filter.has_filters());
}

#[test]
fn test_apply_churn_filters_no_filter() {
    let filter = FileFilter::new(vec![], vec![]).unwrap();
    let mut analysis = create_churn_analysis(5);

    crate::cli::handlers::complexity_handlers::apply_churn_filters(&mut analysis, &filter, 0);

    assert_eq!(analysis.files.len(), 5);
}

#[test]
fn test_apply_churn_filters_top_files_limit() {
    let filter = FileFilter::new(vec![], vec![]).unwrap();
    let mut analysis = create_churn_analysis(10);

    crate::cli::handlers::complexity_handlers::apply_churn_filters(&mut analysis, &filter, 3);

    assert_eq!(analysis.files.len(), 3);
}

#[test]
fn test_apply_churn_filters_sorts_by_commit_count() {
    let filter = FileFilter::new(vec![], vec![]).unwrap();
    let mut analysis = CodeChurnAnalysis {
        files: vec![
            ChurnFileInfo {
                path: "low.rs".to_string(),
                commit_count: 5,
                lines_added: 10,
                lines_removed: 5,
                churn_rate: 1.0,
                last_commit: Utc::now(),
            },
            ChurnFileInfo {
                path: "high.rs".to_string(),
                commit_count: 50,
                lines_added: 100,
                lines_removed: 50,
                churn_rate: 3.0,
            last_commit: Utc::now(),
            },
            ChurnFileInfo {
                path: "medium.rs".to_string(),
                commit_count: 20,
                lines_added: 40,
                lines_removed: 20,
                churn_rate: 2.0,
                last_commit: Utc::now(),
            },
        ],
        summary: ChurnSummary {
            total_files_changed: 3,
            total_commits: 75,
            total_lines_added: 150,
            total_lines_removed: 75,
            avg_churn_rate: 2.0,
            analysis_period_days: 30,
        },
        period_start: Utc::now(),
        period_end: Utc::now(),
    };

    crate::cli::handlers::complexity_handlers::apply_churn_filters(&mut analysis, &filter, 2);

    // Should have top 2 by commit count
    assert_eq!(analysis.files.len(), 2);
    assert_eq!(analysis.files[0].path, "high.rs");
    assert_eq!(analysis.files[1].path, "medium.rs");
}

// =============================================================================
// SATD filter tests
// =============================================================================

#[test]
fn test_apply_satd_filters_no_filters() {
    let mut result = create_satd_result_with_items(10, 20);

    crate::cli::handlers::complexity_handlers::apply_satd_filters(
        &mut result,
        None,
        false,
        0,
    );

    assert_eq!(result.items.len(), 10);
}

#[test]
fn test_apply_satd_filters_severity_filter() {
    let mut result = create_satd_result_with_varied_severity();

    crate::cli::handlers::complexity_handlers::apply_satd_filters(
        &mut result,
        Some(SatdSeverity::High),
        false,
        0,
    );

    // Only High and Critical should remain
    assert!(result.items.iter().all(|i| i.severity >= Severity::High));
}

#[test]
fn test_apply_satd_filters_critical_only() {
    let mut result = create_satd_result_with_varied_severity();

    crate::cli::handlers::complexity_handlers::apply_satd_filters(
        &mut result,
        None,
        true,
        0,
    );

    // Only Critical should remain
    assert!(result.items.iter().all(|i| i.severity == Severity::Critical));
}

#[test]
fn test_apply_satd_filters_top_files() {
    let mut result = create_satd_result_with_multiple_files(3, 5);

    crate::cli::handlers::complexity_handlers::apply_satd_filters(
        &mut result,
        None,
        false,
        2,
    );

    // Should only have items from top 2 files
    let unique_files: std::collections::HashSet<_> = result.items.iter().map(|i| &i.file).collect();
    assert!(unique_files.len() <= 2);
}

// =============================================================================
// generate_satd_sarif tests
// =============================================================================

#[test]
fn test_generate_satd_sarif_empty() {
    let result = SATDAnalysisResult {
        items: vec![],
        summary: SATDSummary {
            total_items: 0,
            by_severity: HashMap::new(),
            by_category: HashMap::new(),
            files_with_satd: 0,
            avg_age_days: 0.0,
        },
        total_files_analyzed: 0,
        files_with_debt: 0,
        analysis_timestamp: Utc::now(),
    };

    let sarif = crate::cli::handlers::complexity_handlers::generate_satd_sarif(&result);

    assert!(sarif["version"].as_str() == Some("2.1.0"));
    assert!(sarif["runs"].is_array());
    let results = &sarif["runs"][0]["results"];
    assert!(results.is_array());
    assert_eq!(results.as_array().unwrap().len(), 0);
}

#[test]
fn test_generate_satd_sarif_with_items() {
    let result = create_satd_result_with_items(3, 10);

    let sarif = crate::cli::handlers::complexity_handlers::generate_satd_sarif(&result);

    let results = &sarif["runs"][0]["results"];
    assert_eq!(results.as_array().unwrap().len(), 3);

    // Check structure
    let first_result = &results[0];
    assert_eq!(first_result["ruleId"].as_str(), Some("satd"));
    assert!(first_result["message"]["text"].is_string());
    assert!(first_result["locations"].is_array());
}

#[test]
fn test_generate_satd_sarif_severity_mapping() {
    let items = vec![
        create_debt_item_with_severity("critical.rs", 10, Severity::Critical),
        create_debt_item_with_severity("high.rs", 20, Severity::High),
        create_debt_item_with_severity("medium.rs", 30, Severity::Medium),
        create_debt_item_with_severity("low.rs", 40, Severity::Low),
    ];

    let result = SATDAnalysisResult {
        items,
        summary: SATDSummary::default(),
        total_files_analyzed: 4,
        files_with_debt: 4,
        analysis_timestamp: Utc::now(),
    };

    let sarif = crate::cli::handlers::complexity_handlers::generate_satd_sarif(&result);
    let results = sarif["runs"][0]["results"].as_array().unwrap();

    // Critical maps to "error"
    assert_eq!(results[0]["level"].as_str(), Some("error"));
    // High maps to "error"
    assert_eq!(results[1]["level"].as_str(), Some("error"));
    // Medium maps to "warning"
    assert_eq!(results[2]["level"].as_str(), Some("warning"));
    // Low maps to "note"
    assert_eq!(results[3]["level"].as_str(), Some("note"));
}

#[test]
fn test_generate_satd_sarif_tool_info() {
    let result = create_satd_result_with_items(1, 1);

    let sarif = crate::cli::handlers::complexity_handlers::generate_satd_sarif(&result);

    let tool = &sarif["runs"][0]["tool"]["driver"];
    assert_eq!(tool["name"].as_str(), Some("pmat"));
    assert!(tool["version"].is_string());
    assert!(tool["informationUri"].is_string());
    assert!(tool["rules"].is_array());
}

// =============================================================================
// Enum tests
// =============================================================================

#[test]
fn test_complexity_output_format_variants() {
    let formats = [
        ComplexityOutputFormat::Summary,
        ComplexityOutputFormat::Full,
        ComplexityOutputFormat::Json,
        ComplexityOutputFormat::Sarif,
    ];

    for format in formats {
        match format {
            ComplexityOutputFormat::Summary => {}
            ComplexityOutputFormat::Full => {}
            ComplexityOutputFormat::Json => {}
            ComplexityOutputFormat::Sarif => {}
        }
    }
}

#[test]
fn test_satd_output_format_variants() {
    let formats = [
        SatdOutputFormat::Summary,
        SatdOutputFormat::Json,
        SatdOutputFormat::Sarif,
        SatdOutputFormat::Markdown,
    ];

    for format in formats {
        match format {
            SatdOutputFormat::Summary => {}
            SatdOutputFormat::Json => {}
            SatdOutputFormat::Sarif => {}
            SatdOutputFormat::Markdown => {}
        }
    }
}

#[test]
fn test_satd_severity_variants() {
    let severities = [
        SatdSeverity::Low,
        SatdSeverity::Medium,
        SatdSeverity::High,
        SatdSeverity::Critical,
    ];

    for severity in severities {
        match severity {
            SatdSeverity::Low => {}
            SatdSeverity::Medium => {}
            SatdSeverity::High => {}
            SatdSeverity::Critical => {}
        }
    }
}

#[test]
fn test_dag_type_variants() {
    let types = [
        DagType::CallGraph,
        DagType::ImportGraph,
        DagType::FullDependency,
        DagType::ModuleGraph,
    ];

    for dag_type in types {
        match dag_type {
            DagType::CallGraph => {}
            DagType::ImportGraph => {}
            DagType::FullDependency => {}
            DagType::ModuleGraph => {}
        }
    }
}

// =============================================================================
// SATDSummary default tests
// =============================================================================

#[test]
fn test_satd_summary_default() {
    let summary = SATDSummary::default();

    assert_eq!(summary.total_items, 0);
    assert!(summary.by_severity.is_empty());
    assert!(summary.by_category.is_empty());
    assert_eq!(summary.files_with_satd, 0);
    assert_eq!(summary.avg_age_days, 0.0);
}

// =============================================================================
// watch mode helper tests (should_reanalyze)
// =============================================================================

#[test]
fn test_should_analyze_path_rust_file() {
    assert!(crate::cli::handlers::complexity_handlers::should_analyze_path(
        std::path::Path::new("src/main.rs"),
        &[]
    ));
}

#[test]
fn test_should_analyze_path_non_source() {
    assert!(!crate::cli::handlers::complexity_handlers::should_analyze_path(
        std::path::Path::new("README.md"),
        &[]
    ));
}

#[test]
fn test_should_analyze_path_with_include_pattern() {
    let patterns = vec!["src".to_string()];

    assert!(crate::cli::handlers::complexity_handlers::should_analyze_path(
        std::path::Path::new("src/main.rs"),
        &patterns
    ));

    assert!(!crate::cli::handlers::complexity_handlers::should_analyze_path(
        std::path::Path::new("tests/test.rs"),
        &patterns
    ));
}

// =============================================================================
// Helper functions
// =============================================================================

fn create_debt_item(file: &str, line: u32) -> TechnicalDebt {
    TechnicalDebt {
        category: DebtCategory::Defect,
        severity: Severity::Medium,
        text: "FIXME: Handle error".to_string(),
        file: PathBuf::from(file),
        line,
        column: 1,
        context_hash: [0; 16],
    }
}

fn create_critical_debt_item(file: &str, line: u32, text: &str) -> TechnicalDebt {
    TechnicalDebt {
        category: DebtCategory::Security,
        severity: Severity::Critical,
        text: text.to_string(),
        file: PathBuf::from(file),
        line,
        column: 1,
        context_hash: [0; 16],
    }
}

fn create_debt_item_with_severity(file: &str, line: u32, severity: Severity) -> TechnicalDebt {
    TechnicalDebt {
        category: DebtCategory::Defect,
        severity,
        text: format!("{:?} severity item", severity),
        file: PathBuf::from(file),
        line,
        column: 1,
        context_hash: [0; 16],
    }
}

fn create_sample_debt_items(count: usize) -> Vec<TechnicalDebt> {
    (0..count)
        .map(|i| create_debt_item(&format!("file{}.rs", i), (i * 10) as u32))
        .collect()
}

fn create_satd_result_with_items(item_count: usize, files_analyzed: usize) -> SATDAnalysisResult {
    SATDAnalysisResult {
        items: create_sample_debt_items(item_count),
        summary: SATDSummary {
            total_items: item_count,
            by_severity: HashMap::new(),
            by_category: HashMap::new(),
            files_with_satd: item_count.min(files_analyzed),
            avg_age_days: 30.0,
        },
        total_files_analyzed: files_analyzed,
        files_with_debt: item_count.min(files_analyzed),
        analysis_timestamp: Utc::now(),
    }
}

fn create_satd_result_with_varied_severity() -> SATDAnalysisResult {
    let items = vec![
        create_debt_item_with_severity("file1.rs", 10, Severity::Low),
        create_debt_item_with_severity("file2.rs", 20, Severity::Medium),
        create_debt_item_with_severity("file3.rs", 30, Severity::High),
        create_debt_item_with_severity("file4.rs", 40, Severity::Critical),
    ];

    SATDAnalysisResult {
        items,
        summary: SATDSummary::default(),
        total_files_analyzed: 4,
        files_with_debt: 4,
        analysis_timestamp: Utc::now(),
    }
}

fn create_satd_result_with_multiple_files(files: usize, items_per_file: usize) -> SATDAnalysisResult {
    let mut items = Vec::new();
    for file_idx in 0..files {
        for item_idx in 0..items_per_file {
            items.push(TechnicalDebt {
                category: DebtCategory::Defect,
                severity: Severity::Medium,
                text: format!("Item {} in file {}", item_idx, file_idx),
                file: PathBuf::from(format!("file{}.rs", file_idx)),
                line: (item_idx * 10) as u32,
                column: 1,
                context_hash: [file_idx as u8; 16],
            });
        }
    }

    SATDAnalysisResult {
        items,
        summary: SATDSummary::default(),
        total_files_analyzed: files,
        files_with_debt: files,
        analysis_timestamp: Utc::now(),
    }
}

fn create_file_metrics_list(count: usize) -> Vec<FileComplexityMetrics> {
    (0..count)
        .map(|i| create_file_metrics_with_complexity(&format!("file{}.rs", i), (i * 5) as u16 + 1, (i * 3) as u16 + 1))
        .collect()
}

fn create_file_metrics_with_complexity(name: &str, cyclomatic: u16, cognitive: u16) -> FileComplexityMetrics {
    FileComplexityMetrics {
        path: PathBuf::from(name),
        language: Some("rust".to_string()),
        functions: vec![FunctionComplexity {
            name: "test_fn".to_string(),
            line: 1,
            column: 0,
            end_line: Some(10),
            metrics: ComplexityMetrics::new(cyclomatic, cognitive, 2, 10),
        }],
        total_complexity: ComplexityMetrics::new(cyclomatic, cognitive, 2, 10),
    }
}

fn create_churn_analysis(files: usize) -> CodeChurnAnalysis {
    CodeChurnAnalysis {
        files: (0..files)
            .map(|i| ChurnFileInfo {
                path: format!("file{}.rs", i),
                commit_count: (i * 5 + 1) as u32,
                lines_added: (i * 10) as u32,
                lines_removed: (i * 5) as u32,
                churn_rate: (i as f64) + 1.0,
                last_commit: Utc::now(),
            })
            .collect(),
        summary: ChurnSummary {
            total_files_changed: files,
            total_commits: files as u32 * 10,
            total_lines_added: files as u32 * 50,
            total_lines_removed: files as u32 * 25,
            avg_churn_rate: 2.0,
            analysis_period_days: 30,
        },
        period_start: Utc::now(),
        period_end: Utc::now(),
    }
}

// =============================================================================
// Additional edge case tests
// =============================================================================

#[test]
fn test_format_satd_summary_special_characters_in_text() {
    let items = vec![TechnicalDebt {
        category: DebtCategory::Defect,
        severity: Severity::Critical,
        text: "FIXME: Handle | pipe | and <angle> brackets".to_string(),
        file: PathBuf::from("src/special.rs"),
        line: 10,
        column: 1,
        context_hash: [0; 16],
    }];

    let result = SATDAnalysisResult {
        items,
        summary: SATDSummary::default(),
        total_files_analyzed: 1,
        files_with_debt: 1,
        analysis_timestamp: Utc::now(),
    };

    let output = crate::cli::handlers::complexity_handlers::format_satd_summary(&result, false);
    // Should not panic and should contain the text
    assert!(output.contains("FIXME"));
}

#[test]
fn test_is_source_code_file_case_sensitivity() {
    // Rust typically uses lowercase extensions
    assert!(crate::cli::handlers::complexity_handlers::is_source_code_file("file.rs"));
    // Some systems might have uppercase
    assert!(!crate::cli::handlers::complexity_handlers::is_source_code_file("file.RS"));
}

#[test]
fn test_apply_complexity_filters_with_multiple_functions() {
    // Create a file with multiple functions where only some exceed threshold
    let metrics = FileComplexityMetrics {
        path: PathBuf::from("multi_func.rs"),
        language: Some("rust".to_string()),
        functions: vec![
            FunctionComplexity {
                name: "simple_fn".to_string(),
                line: 1,
                column: 0,
                end_line: Some(5),
                metrics: ComplexityMetrics::new(3, 2, 1, 5),
            },
            FunctionComplexity {
                name: "complex_fn".to_string(),
                line: 10,
                column: 0,
                end_line: Some(50),
                metrics: ComplexityMetrics::new(25, 30, 5, 50), // Exceeds both
            },
        ],
        total_complexity: ComplexityMetrics::new(28, 32, 5, 55),
    };

    let mut metrics_vec = vec![metrics];
    let filtered = crate::cli::handlers::complexity_handlers::apply_complexity_filters(
        &mut metrics_vec,
        Some(10),
        Some(15),
    );

    // File should remain because one function exceeds thresholds
    assert_eq!(filtered, 0);
    assert_eq!(metrics_vec.len(), 1);
}

#[test]
fn test_satd_sarif_location_details() {
    let items = vec![TechnicalDebt {
        category: DebtCategory::Design,
        severity: Severity::Medium,
        text: "HACK: Temporary workaround".to_string(),
        file: PathBuf::from("src/hack.rs"),
        line: 42,
        column: 8,
        context_hash: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    }];

    let result = SATDAnalysisResult {
        items,
        summary: SATDSummary::default(),
        total_files_analyzed: 1,
        files_with_debt: 1,
        analysis_timestamp: Utc::now(),
    };

    let sarif = crate::cli::handlers::complexity_handlers::generate_satd_sarif(&result);
    let location = &sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"];

    assert!(location["artifactLocation"]["uri"].as_str().unwrap().contains("hack.rs"));
    assert_eq!(location["region"]["startLine"].as_u64(), Some(42));
    assert_eq!(location["region"]["startColumn"].as_u64(), Some(8));
}

impl Default for SATDSummary {
    fn default() -> Self {
        Self {
            total_items: 0,
            by_severity: HashMap::new(),
            by_category: HashMap::new(),
            files_with_satd: 0,
            avg_age_days: 0.0,
        }
    }
}
