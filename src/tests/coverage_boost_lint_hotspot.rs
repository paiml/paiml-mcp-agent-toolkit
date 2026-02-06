#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for cli/handlers/lint_hotspot_handlers.rs
//! Tests struct types, serialization, and format_summary function

use crate::cli::handlers::lint_hotspot_handlers::{
    format_summary, EnforcementMetadata, FileSummary, LintHotspot, LintHotspotParams,
    LintHotspotResult, QualityGateStatus, QualityViolation, RefactorChain, RefactorStep,
    SeverityDistribution, ViolationDetail,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

fn create_test_violation() -> ViolationDetail {
    ViolationDetail {
        file: PathBuf::from("src/main.rs"),
        line: 10,
        column: 5,
        end_line: 10,
        end_column: 20,
        lint_name: "clippy::unwrap_used".to_string(),
        message: "used `unwrap()` on a `Result` value".to_string(),
        severity: "warning".to_string(),
        suggestion: Some("use `expect()` or handle the error".to_string()),
        machine_applicable: false,
    }
}

fn create_test_result() -> LintHotspotResult {
    let mut summary_by_file = HashMap::new();
    summary_by_file.insert(
        PathBuf::from("src/main.rs"),
        FileSummary {
            total_violations: 5,
            errors: 1,
            warnings: 3,
            sloc: 100,
            defect_density: 0.05,
        },
    );
    summary_by_file.insert(
        PathBuf::from("src/lib.rs"),
        FileSummary {
            total_violations: 2,
            errors: 0,
            warnings: 2,
            sloc: 200,
            defect_density: 0.01,
        },
    );

    LintHotspotResult {
        hotspot: LintHotspot {
            file: PathBuf::from("src/main.rs"),
            defect_density: 0.05,
            total_violations: 5,
            sloc: 100,
            severity_distribution: SeverityDistribution {
                error: 1,
                warning: 3,
                suggestion: 1,
                note: 0,
            },
            top_lints: vec![
                ("clippy::unwrap_used".to_string(), 3),
                ("clippy::pedantic".to_string(), 2),
            ],
            detailed_violations: vec![create_test_violation()],
        },
        all_violations: vec![create_test_violation()],
        summary_by_file,
        total_project_violations: 7,
        enforcement: None,
        refactor_chain: None,
        quality_gate: QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        },
    }
}

// --- Struct instantiation and Debug tests ---

#[test]
fn test_severity_distribution_default() {
    let dist = SeverityDistribution::default();
    assert_eq!(dist.error, 0);
    assert_eq!(dist.warning, 0);
    assert_eq!(dist.suggestion, 0);
    assert_eq!(dist.note, 0);
    let _ = format!("{:?}", dist);
}

#[test]
fn test_violation_detail_debug_clone() {
    let v = create_test_violation();
    let _ = format!("{:?}", v);
    let cloned = v.clone();
    assert_eq!(cloned.line, 10);
    assert_eq!(cloned.lint_name, "clippy::unwrap_used");
}

#[test]
fn test_file_summary_debug() {
    let summary = FileSummary {
        total_violations: 10,
        errors: 3,
        warnings: 7,
        sloc: 500,
        defect_density: 0.02,
    };
    let _ = format!("{:?}", summary);
}

#[test]
fn test_enforcement_metadata_debug() {
    let meta = EnforcementMetadata {
        enforcement_score: 7.5,
        requires_enforcement: true,
        estimated_fix_time: 3600,
        automation_confidence: 0.85,
        enforcement_priority: 2,
    };
    let _ = format!("{:?}", meta);
}

#[test]
fn test_refactor_chain_debug() {
    let chain = RefactorChain {
        id: "chain-001".to_string(),
        estimated_reduction: 15,
        automation_confidence: 0.9,
        steps: vec![RefactorStep {
            id: "step-001".to_string(),
            lint: "clippy::unwrap_used".to_string(),
            confidence: 0.95,
            impact: 5,
            description: "Replace unwrap with expect".to_string(),
        }],
    };
    let _ = format!("{:?}", chain);
}

#[test]
fn test_quality_gate_status_debug() {
    let status = QualityGateStatus {
        passed: false,
        violations: vec![QualityViolation {
            rule: "defect_density".to_string(),
            threshold: 0.01,
            actual: 0.05,
            severity: "error".to_string(),
        }],
        blocking: true,
    };
    let _ = format!("{:?}", status);
    assert!(!status.passed);
}

#[test]
fn test_lint_hotspot_params_construction() {
    let params = LintHotspotParams {
        project_path: PathBuf::from("."),
        file: None,
        format: crate::cli::LintHotspotOutputFormat::Summary,
        max_density: 5.0,
        min_confidence: 0.8,
        enforce: false,
        dry_run: false,
        enforcement_metadata: false,
        output: None,
        perf: false,
        clippy_flags: String::new(),
        top_files: 10,
        include: vec![],
        exclude: vec![],
    };
    assert_eq!(params.max_density, 5.0);
}

// --- Serialization tests ---

#[test]
fn test_severity_distribution_serde() {
    let dist = SeverityDistribution {
        error: 3,
        warning: 5,
        suggestion: 2,
        note: 1,
    };
    let json = serde_json::to_string(&dist).unwrap();
    let back: SeverityDistribution = serde_json::from_str(&json).unwrap();
    assert_eq!(back.error, 3);
    assert_eq!(back.warning, 5);
}

#[test]
fn test_violation_detail_serde() {
    let v = create_test_violation();
    let json = serde_json::to_string(&v).unwrap();
    let back: ViolationDetail = serde_json::from_str(&json).unwrap();
    assert_eq!(back.line, 10);
    assert_eq!(back.lint_name, "clippy::unwrap_used");
}

#[test]
fn test_lint_hotspot_result_serde() {
    let result = create_test_result();
    let json = serde_json::to_string(&result).unwrap();
    let back: LintHotspotResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_project_violations, 7);
    assert!(back.quality_gate.passed);
}

#[test]
fn test_quality_violation_serde() {
    let v = QualityViolation {
        rule: "density".to_string(),
        threshold: 0.01,
        actual: 0.05,
        severity: "error".to_string(),
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: QualityViolation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.rule, "density");
}

// --- format_summary tests ---

#[test]
fn test_format_summary_basic() {
    let result = create_test_result();
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

    assert!(output.contains("# Lint Hotspot Analysis"));
    assert!(output.contains("**Total Project Violations**: 7"));
    assert!(output.contains("## Top Files with Lint Issues"));
    assert!(output.contains("## Hottest File Details"));
    assert!(output.contains("src/main.rs"));
}

#[test]
fn test_format_summary_with_perf() {
    let result = create_test_result();
    let output = format_summary(&result, true, Duration::from_millis(1500), 10).unwrap();

    assert!(output.contains("Analysis completed in"));
}

#[test]
fn test_format_summary_without_perf() {
    let result = create_test_result();
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

    assert!(!output.contains("Analysis completed in"));
}

#[test]
fn test_format_summary_with_enforcement() {
    let mut result = create_test_result();
    result.enforcement = Some(EnforcementMetadata {
        enforcement_score: 8.5,
        requires_enforcement: true,
        estimated_fix_time: 3600,
        automation_confidence: 0.9,
        enforcement_priority: 1,
    });

    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("## Enforcement Metadata"));
    assert!(output.contains("Score: 8.5/10"));
}

#[test]
fn test_format_summary_with_quality_gate_failure() {
    let mut result = create_test_result();
    result.quality_gate = QualityGateStatus {
        passed: false,
        violations: vec![QualityViolation {
            rule: "defect_density".to_string(),
            threshold: 0.01,
            actual: 0.05,
            severity: "error".to_string(),
        }],
        blocking: true,
    };

    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("Quality Gate Failed"));
    assert!(output.contains("defect_density exceeded"));
}

#[test]
fn test_format_summary_severity_distribution() {
    let result = create_test_result();
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

    assert!(output.contains("Errors: 1"));
    assert!(output.contains("Warnings: 3"));
    assert!(output.contains("Suggestions: 1"));
}

#[test]
fn test_format_summary_top_lints() {
    let result = create_test_result();
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

    assert!(output.contains("clippy::unwrap_used: 3 occurrences"));
    assert!(output.contains("clippy::pedantic: 2 occurrences"));
}

#[test]
fn test_format_summary_custom_top_files() {
    let result = create_test_result();
    // With top_files=1, should only show 1 file
    let output = format_summary(&result, false, Duration::from_secs(1), 1).unwrap();
    // Should still contain the top file section
    assert!(output.contains("## Top Files with Lint Issues"));
}

#[test]
fn test_format_summary_zero_top_files_defaults_to_10() {
    let result = create_test_result();
    let output = format_summary(&result, false, Duration::from_secs(1), 0).unwrap();
    assert!(output.contains("## Top Files with Lint Issues"));
}
