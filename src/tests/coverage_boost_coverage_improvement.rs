//! Coverage boost tests for services/coverage_improvement module
//! Tests: config defaults, type construction, serialization

use crate::services::coverage_improvement::{
    CoverageImprovementConfig, CoverageImprovementReport, CoverageImprovementService,
    IterationReport,
};
use std::path::PathBuf;

// --- CoverageImprovementConfig tests ---

#[test]
fn test_config_default() {
    let config = CoverageImprovementConfig::default();
    assert_eq!(config.project_path, PathBuf::from("."));
    assert_eq!(config.target_coverage, 95.0);
    assert_eq!(config.max_iterations, 10);
    assert!(!config.fast_mode);
    assert_eq!(config.mutation_threshold, 80.0);
    assert!(config.focus_patterns.is_empty());
    assert!(config.exclude_patterns.is_empty());
}

#[test]
fn test_config_custom() {
    let config = CoverageImprovementConfig {
        project_path: PathBuf::from("/tmp/test"),
        target_coverage: 80.0,
        max_iterations: 5,
        fast_mode: true,
        mutation_threshold: 60.0,
        focus_patterns: vec!["src/services/**".to_string()],
        exclude_patterns: vec!["src/tests/**".to_string()],
    };
    assert!(config.fast_mode);
    assert_eq!(config.focus_patterns.len(), 1);
    assert_eq!(config.exclude_patterns.len(), 1);
}

#[test]
fn test_config_debug() {
    let config = CoverageImprovementConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("CoverageImprovementConfig"));
    assert!(debug.contains("95"));
}

#[test]
fn test_config_clone() {
    let config = CoverageImprovementConfig {
        project_path: PathBuf::from("/test"),
        target_coverage: 90.0,
        max_iterations: 3,
        fast_mode: true,
        mutation_threshold: 70.0,
        focus_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
    };
    let cloned = config.clone();
    assert_eq!(cloned.target_coverage, 90.0);
    assert_eq!(cloned.max_iterations, 3);
    assert!(cloned.fast_mode);
}

// --- IterationReport tests ---

#[test]
fn test_iteration_report_serde() {
    let report = IterationReport {
        iteration: 1,
        files_targeted: vec![PathBuf::from("src/main.rs")],
        tests_generated: 5,
        coverage_gain: 3.5,
        mutation_score: 85.0,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: IterationReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.iteration, 1);
    assert_eq!(back.tests_generated, 5);
    assert_eq!(back.coverage_gain, 3.5);
}

#[test]
fn test_iteration_report_debug_clone() {
    let report = IterationReport {
        iteration: 3,
        files_targeted: vec![],
        tests_generated: 0,
        coverage_gain: 0.0,
        mutation_score: 0.0,
    };
    let _ = format!("{:?}", report);
    let cloned = report.clone();
    assert_eq!(cloned.iteration, 3);
}

// --- CoverageImprovementReport tests ---

#[test]
fn test_improvement_report_serde() {
    let report = CoverageImprovementReport {
        baseline_coverage: 74.0,
        target_coverage: 95.0,
        final_coverage: 88.0,
        iterations: vec![
            IterationReport {
                iteration: 1,
                files_targeted: vec![PathBuf::from("src/a.rs")],
                tests_generated: 10,
                coverage_gain: 7.0,
                mutation_score: 82.0,
            },
            IterationReport {
                iteration: 2,
                files_targeted: vec![PathBuf::from("src/b.rs")],
                tests_generated: 8,
                coverage_gain: 7.0,
                mutation_score: 90.0,
            },
        ],
        success: false,
        stop_reason: "Max iterations reached".to_string(),
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: CoverageImprovementReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.baseline_coverage, 74.0);
    assert_eq!(back.final_coverage, 88.0);
    assert_eq!(back.iterations.len(), 2);
    assert!(!back.success);
}

#[test]
fn test_improvement_report_success() {
    let report = CoverageImprovementReport {
        baseline_coverage: 90.0,
        target_coverage: 95.0,
        final_coverage: 96.0,
        iterations: vec![],
        success: true,
        stop_reason: "Target reached".to_string(),
    };
    assert!(report.success);
    assert!(report.final_coverage >= report.target_coverage);
}

#[test]
fn test_improvement_report_debug_clone() {
    let report = CoverageImprovementReport {
        baseline_coverage: 50.0,
        target_coverage: 80.0,
        final_coverage: 75.0,
        iterations: vec![],
        success: false,
        stop_reason: "stalled".to_string(),
    };
    let _ = format!("{:?}", report);
    let cloned = report.clone();
    assert_eq!(cloned.baseline_coverage, 50.0);
}

// --- CoverageImprovementService tests ---

#[test]
fn test_service_construction() {
    let config = CoverageImprovementConfig::default();
    let _service = CoverageImprovementService::new(config);
}

#[test]
fn test_service_with_custom_config() {
    let config = CoverageImprovementConfig {
        project_path: PathBuf::from("/home/test"),
        target_coverage: 99.0,
        max_iterations: 20,
        fast_mode: true,
        mutation_threshold: 50.0,
        focus_patterns: vec!["src/**/*.rs".to_string()],
        exclude_patterns: vec!["src/tests/**".to_string()],
    };
    let _service = CoverageImprovementService::new(config);
}
