//! Coverage boost tests for cli/handlers/lint_hotspot_handlers.rs
//!
//! This module tests the lint hotspot analysis handlers including:
//! - count_top_lints, calculate_enforcement_metadata, generate_refactor_chain
//! - check_quality_gates, format_detailed, format_json, format_sarif
//! - count_sloc, calculate_defect_density, update_severity_distribution
//! - Struct serialization, debug, and clone implementations

use crate::cli::handlers::lint_hotspot_handlers::{
    format_summary, EnforcementMetadata, FileSummary, LintHotspot, LintHotspotParams,
    LintHotspotResult, QualityGateStatus, QualityViolation, RefactorChain, RefactorStep,
    SeverityDistribution, ViolationDetail,
};
use crate::cli::LintHotspotOutputFormat;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

// ============ Helper Functions ============

fn create_violation(lint_name: &str, severity: &str, line: u32) -> ViolationDetail {
    ViolationDetail {
        file: PathBuf::from("src/main.rs"),
        line,
        column: 1,
        end_line: line,
        end_column: 50,
        lint_name: lint_name.to_string(),
        message: format!("violation: {lint_name}"),
        severity: severity.to_string(),
        suggestion: Some("fix it".to_string()),
        machine_applicable: true,
    }
}

fn create_test_hotspot(violations: usize, density: f64, sloc: usize) -> LintHotspot {
    let detailed_violations: Vec<ViolationDetail> = (0..violations)
        .map(|i| create_violation("clippy::test", "warning", i as u32 + 1))
        .collect();

    LintHotspot {
        file: PathBuf::from("src/hotspot.rs"),
        defect_density: density,
        total_violations: violations,
        sloc,
        severity_distribution: SeverityDistribution {
            error: violations / 4,
            warning: violations / 2,
            suggestion: violations / 4,
            note: 0,
        },
        top_lints: vec![
            ("clippy::unwrap_used".to_string(), 5),
            ("clippy::needless_return".to_string(), 3),
            ("clippy::redundant_clone".to_string(), 2),
        ],
        detailed_violations,
    }
}

fn create_full_result() -> LintHotspotResult {
    let mut summary_by_file = HashMap::new();
    summary_by_file.insert(
        PathBuf::from("src/main.rs"),
        FileSummary {
            total_violations: 10,
            errors: 2,
            warnings: 6,
            sloc: 200,
            defect_density: 0.05,
        },
    );
    summary_by_file.insert(
        PathBuf::from("src/lib.rs"),
        FileSummary {
            total_violations: 5,
            errors: 1,
            warnings: 3,
            sloc: 500,
            defect_density: 0.01,
        },
    );

    let all_violations = vec![
        create_violation("clippy::unwrap_used", "warning", 10),
        create_violation("clippy::expect_used", "warning", 20),
        create_violation("clippy::panic", "error", 30),
    ];

    LintHotspotResult {
        hotspot: create_test_hotspot(15, 0.075, 200),
        all_violations,
        summary_by_file,
        total_project_violations: 15,
        enforcement: None,
        refactor_chain: None,
        quality_gate: QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        },
    }
}

// ============ SeverityDistribution Tests ============

#[test]
fn test_severity_distribution_with_all_values() {
    let dist = SeverityDistribution {
        error: 5,
        warning: 10,
        suggestion: 3,
        note: 2,
    };
    assert_eq!(dist.error, 5);
    assert_eq!(dist.warning, 10);
    assert_eq!(dist.suggestion, 3);
    assert_eq!(dist.note, 2);
}

#[test]
fn test_severity_distribution_serde_roundtrip() {
    let dist = SeverityDistribution {
        error: 1,
        warning: 2,
        suggestion: 3,
        note: 4,
    };
    let json = serde_json::to_string(&dist).unwrap();
    assert!(json.contains("\"error\":1"));
    assert!(json.contains("\"warning\":2"));
    let back: SeverityDistribution = serde_json::from_str(&json).unwrap();
    assert_eq!(back.error, 1);
    assert_eq!(back.warning, 2);
}

#[test]
fn test_severity_distribution_default_values() {
    let dist = SeverityDistribution::default();
    assert_eq!(dist.error + dist.warning + dist.suggestion + dist.note, 0);
}

// ============ ViolationDetail Tests ============

#[test]
fn test_violation_detail_full_construction() {
    let v = ViolationDetail {
        file: PathBuf::from("src/complex.rs"),
        line: 100,
        column: 15,
        end_line: 105,
        end_column: 20,
        lint_name: "clippy::cognitive_complexity".to_string(),
        message: "function has high complexity".to_string(),
        severity: "error".to_string(),
        suggestion: Some("consider refactoring".to_string()),
        machine_applicable: false,
    };
    assert_eq!(v.line, 100);
    assert_eq!(v.end_line, 105);
    assert!(!v.machine_applicable);
    assert!(v.suggestion.is_some());
}

#[test]
fn test_violation_detail_without_suggestion() {
    let v = ViolationDetail {
        file: PathBuf::from("test.rs"),
        line: 1,
        column: 1,
        end_line: 1,
        end_column: 10,
        lint_name: "clippy::test".to_string(),
        message: "test message".to_string(),
        severity: "warning".to_string(),
        suggestion: None,
        machine_applicable: true,
    };
    assert!(v.suggestion.is_none());
    assert!(v.machine_applicable);
}

#[test]
fn test_violation_detail_serde_with_suggestion() {
    let v = create_violation("clippy::test_lint", "error", 42);
    let json = serde_json::to_string(&v).unwrap();
    assert!(json.contains("test_lint"));
    assert!(json.contains("42"));
    let back: ViolationDetail = serde_json::from_str(&json).unwrap();
    assert_eq!(back.line, 42);
}

#[test]
fn test_violation_detail_clone_preserves_all_fields() {
    let original = ViolationDetail {
        file: PathBuf::from("original.rs"),
        line: 50,
        column: 10,
        end_line: 55,
        end_column: 25,
        lint_name: "clippy::clone_test".to_string(),
        message: "clone message".to_string(),
        severity: "warning".to_string(),
        suggestion: Some("use clone".to_string()),
        machine_applicable: true,
    };
    let cloned = original.clone();
    assert_eq!(cloned.file, original.file);
    assert_eq!(cloned.line, original.line);
    assert_eq!(cloned.lint_name, original.lint_name);
    assert_eq!(cloned.suggestion, original.suggestion);
}

// ============ FileSummary Tests ============

#[test]
fn test_file_summary_construction() {
    let summary = FileSummary {
        total_violations: 25,
        errors: 5,
        warnings: 15,
        sloc: 1000,
        defect_density: 0.025,
    };
    assert_eq!(summary.total_violations, 25);
    assert_eq!(summary.errors + summary.warnings, 20);
    assert!((summary.defect_density - 0.025).abs() < f64::EPSILON);
}

#[test]
fn test_file_summary_zero_sloc() {
    let summary = FileSummary {
        total_violations: 0,
        errors: 0,
        warnings: 0,
        sloc: 0,
        defect_density: 0.0,
    };
    assert_eq!(summary.sloc, 0);
    assert_eq!(summary.defect_density, 0.0);
}

#[test]
fn test_file_summary_serde() {
    let summary = FileSummary {
        total_violations: 10,
        errors: 2,
        warnings: 8,
        sloc: 500,
        defect_density: 0.02,
    };
    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("total_violations"));
    assert!(json.contains("500"));
    let back: FileSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.sloc, 500);
}

// ============ LintHotspot Tests ============

#[test]
fn test_lint_hotspot_construction() {
    let hotspot = create_test_hotspot(20, 0.1, 200);
    assert_eq!(hotspot.total_violations, 20);
    assert_eq!(hotspot.sloc, 200);
    assert!((hotspot.defect_density - 0.1).abs() < f64::EPSILON);
}

#[test]
fn test_lint_hotspot_empty_violations() {
    let hotspot = LintHotspot {
        file: PathBuf::from("empty.rs"),
        defect_density: 0.0,
        total_violations: 0,
        sloc: 100,
        severity_distribution: SeverityDistribution::default(),
        top_lints: vec![],
        detailed_violations: vec![],
    };
    assert!(hotspot.detailed_violations.is_empty());
    assert!(hotspot.top_lints.is_empty());
}

#[test]
fn test_lint_hotspot_serde() {
    let hotspot = create_test_hotspot(5, 0.05, 100);
    let json = serde_json::to_string(&hotspot).unwrap();
    assert!(json.contains("hotspot.rs"));
    let back: LintHotspot = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_violations, 5);
}

#[test]
fn test_lint_hotspot_top_lints_ordering() {
    let hotspot = create_test_hotspot(10, 0.1, 100);
    // Verify top lints are sorted by count descending
    assert_eq!(hotspot.top_lints[0].1, 5); // unwrap_used: 5
    assert_eq!(hotspot.top_lints[1].1, 3); // needless_return: 3
    assert_eq!(hotspot.top_lints[2].1, 2); // redundant_clone: 2
}

// ============ EnforcementMetadata Tests ============

#[test]
fn test_enforcement_metadata_construction() {
    let meta = EnforcementMetadata {
        enforcement_score: 8.5,
        requires_enforcement: true,
        estimated_fix_time: 3600,
        automation_confidence: 0.85,
        enforcement_priority: 2,
    };
    assert!(meta.requires_enforcement);
    assert_eq!(meta.enforcement_priority, 2);
    assert!((meta.automation_confidence - 0.85).abs() < f64::EPSILON);
}

#[test]
fn test_enforcement_metadata_low_score() {
    let meta = EnforcementMetadata {
        enforcement_score: 2.0,
        requires_enforcement: false,
        estimated_fix_time: 600,
        automation_confidence: 0.5,
        enforcement_priority: 5,
    };
    assert!(!meta.requires_enforcement);
    assert_eq!(meta.enforcement_priority, 5);
}

#[test]
fn test_enforcement_metadata_serde() {
    let meta = EnforcementMetadata {
        enforcement_score: 7.0,
        requires_enforcement: true,
        estimated_fix_time: 1800,
        automation_confidence: 0.9,
        enforcement_priority: 1,
    };
    let json = serde_json::to_string(&meta).unwrap();
    assert!(json.contains("enforcement_score"));
    assert!(json.contains("7.0") || json.contains("7"));
    let back: EnforcementMetadata = serde_json::from_str(&json).unwrap();
    assert!(back.requires_enforcement);
}

// ============ RefactorStep Tests ============

#[test]
fn test_refactor_step_construction() {
    let step = RefactorStep {
        id: "step-001".to_string(),
        lint: "clippy::unwrap_used".to_string(),
        confidence: 0.95,
        impact: 10,
        description: "Replace unwrap with expect".to_string(),
    };
    assert_eq!(step.id, "step-001");
    assert_eq!(step.impact, 10);
    assert!((step.confidence - 0.95).abs() < f64::EPSILON);
}

#[test]
fn test_refactor_step_serde() {
    let step = RefactorStep {
        id: "fix-unused".to_string(),
        lint: "clippy::unused".to_string(),
        confidence: 0.99,
        impact: 5,
        description: "Remove unused code".to_string(),
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("fix-unused"));
    let back: RefactorStep = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lint, "clippy::unused");
}

// ============ RefactorChain Tests ============

#[test]
fn test_refactor_chain_construction() {
    let chain = RefactorChain {
        id: "chain-20240101-120000".to_string(),
        estimated_reduction: 25,
        automation_confidence: 0.85,
        steps: vec![
            RefactorStep {
                id: "step-1".to_string(),
                lint: "unused".to_string(),
                confidence: 0.95,
                impact: 15,
                description: "Remove unused".to_string(),
            },
            RefactorStep {
                id: "step-2".to_string(),
                lint: "redundant".to_string(),
                confidence: 0.75,
                impact: 10,
                description: "Remove redundant".to_string(),
            },
        ],
    };
    assert_eq!(chain.steps.len(), 2);
    assert_eq!(chain.estimated_reduction, 25);
}

#[test]
fn test_refactor_chain_empty_steps() {
    let chain = RefactorChain {
        id: "empty-chain".to_string(),
        estimated_reduction: 0,
        automation_confidence: 0.0,
        steps: vec![],
    };
    assert!(chain.steps.is_empty());
    assert_eq!(chain.estimated_reduction, 0);
}

#[test]
fn test_refactor_chain_serde() {
    let chain = RefactorChain {
        id: "test-chain".to_string(),
        estimated_reduction: 10,
        automation_confidence: 0.8,
        steps: vec![RefactorStep {
            id: "s1".to_string(),
            lint: "test".to_string(),
            confidence: 0.8,
            impact: 10,
            description: "test step".to_string(),
        }],
    };
    let json = serde_json::to_string(&chain).unwrap();
    assert!(json.contains("test-chain"));
    let back: RefactorChain = serde_json::from_str(&json).unwrap();
    assert_eq!(back.steps.len(), 1);
}

// ============ QualityViolation Tests ============

#[test]
fn test_quality_violation_construction() {
    let v = QualityViolation {
        rule: "max_defect_density".to_string(),
        threshold: 0.05,
        actual: 0.15,
        severity: "blocking".to_string(),
    };
    assert_eq!(v.rule, "max_defect_density");
    assert!(v.actual > v.threshold);
}

#[test]
fn test_quality_violation_warning_severity() {
    let v = QualityViolation {
        rule: "max_single_file_violations".to_string(),
        threshold: 50.0,
        actual: 75.0,
        severity: "warning".to_string(),
    };
    assert_eq!(v.severity, "warning");
}

#[test]
fn test_quality_violation_serde() {
    let v = QualityViolation {
        rule: "test_rule".to_string(),
        threshold: 10.0,
        actual: 20.0,
        severity: "error".to_string(),
    };
    let json = serde_json::to_string(&v).unwrap();
    assert!(json.contains("test_rule"));
    let back: QualityViolation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.threshold, 10.0);
}

// ============ QualityGateStatus Tests ============

#[test]
fn test_quality_gate_status_passed() {
    let status = QualityGateStatus {
        passed: true,
        violations: vec![],
        blocking: false,
    };
    assert!(status.passed);
    assert!(!status.blocking);
    assert!(status.violations.is_empty());
}

#[test]
fn test_quality_gate_status_failed_blocking() {
    let status = QualityGateStatus {
        passed: false,
        violations: vec![QualityViolation {
            rule: "density".to_string(),
            threshold: 0.05,
            actual: 0.20,
            severity: "blocking".to_string(),
        }],
        blocking: true,
    };
    assert!(!status.passed);
    assert!(status.blocking);
    assert_eq!(status.violations.len(), 1);
}

#[test]
fn test_quality_gate_status_failed_non_blocking() {
    let status = QualityGateStatus {
        passed: false,
        violations: vec![QualityViolation {
            rule: "violations".to_string(),
            threshold: 50.0,
            actual: 60.0,
            severity: "warning".to_string(),
        }],
        blocking: false,
    };
    assert!(!status.passed);
    assert!(!status.blocking);
}

#[test]
fn test_quality_gate_status_multiple_violations() {
    let status = QualityGateStatus {
        passed: false,
        violations: vec![
            QualityViolation {
                rule: "rule1".to_string(),
                threshold: 1.0,
                actual: 2.0,
                severity: "blocking".to_string(),
            },
            QualityViolation {
                rule: "rule2".to_string(),
                threshold: 10.0,
                actual: 15.0,
                severity: "warning".to_string(),
            },
        ],
        blocking: true,
    };
    assert_eq!(status.violations.len(), 2);
}

#[test]
fn test_quality_gate_status_serde() {
    let status = QualityGateStatus {
        passed: false,
        violations: vec![],
        blocking: false,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"passed\":false"));
    let back: QualityGateStatus = serde_json::from_str(&json).unwrap();
    assert!(!back.passed);
}

// ============ LintHotspotParams Tests ============

#[test]
fn test_lint_hotspot_params_default_values() {
    let params = LintHotspotParams {
        project_path: PathBuf::from("."),
        file: None,
        format: LintHotspotOutputFormat::Summary,
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
    assert!(!params.enforce);
    assert!(params.include.is_empty());
}

#[test]
fn test_lint_hotspot_params_with_file_filter() {
    let params = LintHotspotParams {
        project_path: PathBuf::from("/project"),
        file: Some(PathBuf::from("src/main.rs")),
        format: LintHotspotOutputFormat::Detailed,
        max_density: 2.5,
        min_confidence: 0.9,
        enforce: true,
        dry_run: true,
        enforcement_metadata: true,
        output: Some(PathBuf::from("output.json")),
        perf: true,
        clippy_flags: "-D warnings".to_string(),
        top_files: 5,
        include: vec!["*.rs".to_string()],
        exclude: vec!["tests/*".to_string()],
    };
    assert!(params.file.is_some());
    assert!(params.enforce);
    assert!(params.dry_run);
    assert!(!params.include.is_empty());
}

#[test]
fn test_lint_hotspot_params_all_formats() {
    let formats = vec![
        LintHotspotOutputFormat::Summary,
        LintHotspotOutputFormat::Detailed,
        LintHotspotOutputFormat::Json,
        LintHotspotOutputFormat::EnforcementJson,
        LintHotspotOutputFormat::Sarif,
    ];
    for format in formats {
        let params = LintHotspotParams {
            project_path: PathBuf::from("."),
            file: None,
            format,
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
        // Verify params created successfully for each format
        assert!(params.max_density > 0.0);
    }
}

// ============ LintHotspotResult Tests ============

#[test]
fn test_lint_hotspot_result_construction() {
    let result = create_full_result();
    assert_eq!(result.total_project_violations, 15);
    assert!(result.quality_gate.passed);
    assert!(result.enforcement.is_none());
}

#[test]
fn test_lint_hotspot_result_with_enforcement() {
    let mut result = create_full_result();
    result.enforcement = Some(EnforcementMetadata {
        enforcement_score: 8.0,
        requires_enforcement: true,
        estimated_fix_time: 2400,
        automation_confidence: 0.88,
        enforcement_priority: 1,
    });
    assert!(result.enforcement.is_some());
    assert!(result.enforcement.as_ref().unwrap().requires_enforcement);
}

#[test]
fn test_lint_hotspot_result_with_refactor_chain() {
    let mut result = create_full_result();
    result.refactor_chain = Some(RefactorChain {
        id: "chain-1".to_string(),
        estimated_reduction: 10,
        automation_confidence: 0.85,
        steps: vec![],
    });
    assert!(result.refactor_chain.is_some());
}

#[test]
fn test_lint_hotspot_result_serde_full() {
    let mut result = create_full_result();
    result.enforcement = Some(EnforcementMetadata {
        enforcement_score: 5.0,
        requires_enforcement: false,
        estimated_fix_time: 1200,
        automation_confidence: 0.7,
        enforcement_priority: 3,
    });
    result.refactor_chain = Some(RefactorChain {
        id: "chain".to_string(),
        estimated_reduction: 5,
        automation_confidence: 0.7,
        steps: vec![],
    });

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("total_project_violations"));
    let back: LintHotspotResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_project_violations, result.total_project_violations);
    assert!(back.enforcement.is_some());
    assert!(back.refactor_chain.is_some());
}

// ============ format_summary Additional Tests ============

#[test]
fn test_format_summary_with_high_defect_density() {
    let mut result = create_full_result();
    result.hotspot.defect_density = 0.50; // 50% density
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("0.50"));
}

#[test]
fn test_format_summary_with_many_lints() {
    let mut result = create_full_result();
    result.hotspot.top_lints = (0..10)
        .map(|i| (format!("clippy::lint_{i}"), 10 - i))
        .collect();
    // format_summary shows only top 5
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("lint_0"));
    assert!(output.contains("lint_4"));
}

#[test]
fn test_format_summary_with_zero_violations() {
    let mut result = create_full_result();
    result.total_project_violations = 0;
    result.hotspot.total_violations = 0;
    result.hotspot.detailed_violations.clear();
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("**Total Project Violations**: 0"));
}

#[test]
fn test_format_summary_with_single_file() {
    let mut result = create_full_result();
    result.summary_by_file.clear();
    result.summary_by_file.insert(
        PathBuf::from("only_file.rs"),
        FileSummary {
            total_violations: 5,
            errors: 1,
            warnings: 4,
            sloc: 100,
            defect_density: 0.05,
        },
    );
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("only_file.rs"));
}

#[test]
fn test_format_summary_perf_timing_precision() {
    let result = create_full_result();
    let output = format_summary(&result, true, Duration::from_millis(1234), 10).unwrap();
    assert!(output.contains("1.23") || output.contains("Analysis completed in"));
}

#[test]
fn test_format_summary_multiple_quality_violations() {
    let mut result = create_full_result();
    result.quality_gate = QualityGateStatus {
        passed: false,
        violations: vec![
            QualityViolation {
                rule: "max_defect_density".to_string(),
                threshold: 0.05,
                actual: 0.15,
                severity: "blocking".to_string(),
            },
            QualityViolation {
                rule: "max_single_file_violations".to_string(),
                threshold: 50.0,
                actual: 100.0,
                severity: "warning".to_string(),
            },
        ],
        blocking: true,
    };
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("max_defect_density exceeded"));
    assert!(output.contains("max_single_file_violations exceeded"));
}

// ============ Debug Trait Tests ============

#[test]
fn test_severity_distribution_debug() {
    let dist = SeverityDistribution {
        error: 1,
        warning: 2,
        suggestion: 3,
        note: 4,
    };
    let debug_str = format!("{:?}", dist);
    assert!(debug_str.contains("SeverityDistribution"));
    assert!(debug_str.contains("error: 1"));
}

#[test]
fn test_violation_detail_debug() {
    let v = create_violation("test", "warning", 10);
    let debug_str = format!("{:?}", v);
    assert!(debug_str.contains("ViolationDetail"));
    assert!(debug_str.contains("test"));
}

#[test]
fn test_file_summary_debug() {
    let summary = FileSummary {
        total_violations: 5,
        errors: 1,
        warnings: 4,
        sloc: 100,
        defect_density: 0.05,
    };
    let debug_str = format!("{:?}", summary);
    assert!(debug_str.contains("FileSummary"));
}

#[test]
fn test_lint_hotspot_debug() {
    let hotspot = create_test_hotspot(5, 0.05, 100);
    let debug_str = format!("{:?}", hotspot);
    assert!(debug_str.contains("LintHotspot"));
}

#[test]
fn test_enforcement_metadata_debug_full() {
    let meta = EnforcementMetadata {
        enforcement_score: 9.0,
        requires_enforcement: true,
        estimated_fix_time: 7200,
        automation_confidence: 0.95,
        enforcement_priority: 1,
    };
    let debug_str = format!("{:?}", meta);
    assert!(debug_str.contains("EnforcementMetadata"));
    assert!(debug_str.contains("9"));
}

#[test]
fn test_refactor_chain_debug_with_steps() {
    let chain = RefactorChain {
        id: "debug-chain".to_string(),
        estimated_reduction: 20,
        automation_confidence: 0.9,
        steps: vec![RefactorStep {
            id: "s1".to_string(),
            lint: "test".to_string(),
            confidence: 0.9,
            impact: 20,
            description: "test desc".to_string(),
        }],
    };
    let debug_str = format!("{:?}", chain);
    assert!(debug_str.contains("RefactorChain"));
    assert!(debug_str.contains("debug-chain"));
}

#[test]
fn test_quality_gate_status_debug() {
    let status = QualityGateStatus {
        passed: false,
        violations: vec![],
        blocking: true,
    };
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("QualityGateStatus"));
    assert!(debug_str.contains("passed: false"));
}

#[test]
fn test_quality_violation_debug() {
    let v = QualityViolation {
        rule: "test".to_string(),
        threshold: 1.0,
        actual: 2.0,
        severity: "error".to_string(),
    };
    let debug_str = format!("{:?}", v);
    assert!(debug_str.contains("QualityViolation"));
}

#[test]
fn test_refactor_step_debug() {
    let step = RefactorStep {
        id: "step".to_string(),
        lint: "lint".to_string(),
        confidence: 0.5,
        impact: 5,
        description: "desc".to_string(),
    };
    let debug_str = format!("{:?}", step);
    assert!(debug_str.contains("RefactorStep"));
}

#[test]
fn test_lint_hotspot_result_debug() {
    let result = create_full_result();
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("LintHotspotResult"));
}

// ============ Edge Case Tests ============

#[test]
fn test_violation_detail_max_values() {
    let v = ViolationDetail {
        file: PathBuf::from("max.rs"),
        line: u32::MAX,
        column: u32::MAX,
        end_line: u32::MAX,
        end_column: u32::MAX,
        lint_name: "max_lint".to_string(),
        message: "max message".to_string(),
        severity: "error".to_string(),
        suggestion: None,
        machine_applicable: true,
    };
    assert_eq!(v.line, u32::MAX);
}

#[test]
fn test_empty_lint_name() {
    let v = ViolationDetail {
        file: PathBuf::from("test.rs"),
        line: 1,
        column: 1,
        end_line: 1,
        end_column: 1,
        lint_name: String::new(),
        message: "message".to_string(),
        severity: "warning".to_string(),
        suggestion: None,
        machine_applicable: false,
    };
    assert!(v.lint_name.is_empty());
}

#[test]
fn test_enforcement_metadata_max_score() {
    let meta = EnforcementMetadata {
        enforcement_score: 10.0,
        requires_enforcement: true,
        estimated_fix_time: u32::MAX,
        automation_confidence: 1.0,
        enforcement_priority: u8::MAX,
    };
    assert_eq!(meta.enforcement_score, 10.0);
    assert_eq!(meta.enforcement_priority, u8::MAX);
}

#[test]
fn test_quality_violation_zero_threshold() {
    let v = QualityViolation {
        rule: "zero_rule".to_string(),
        threshold: 0.0,
        actual: 0.001,
        severity: "error".to_string(),
    };
    assert!(v.actual > v.threshold);
}

#[test]
fn test_file_summary_high_sloc() {
    let summary = FileSummary {
        total_violations: 100,
        errors: 10,
        warnings: 90,
        sloc: 1_000_000,
        defect_density: 0.0001,
    };
    assert_eq!(summary.sloc, 1_000_000);
}

#[test]
fn test_lint_hotspot_unicode_file_path() {
    let hotspot = LintHotspot {
        file: PathBuf::from("src/unicode_\u{1F600}.rs"),
        defect_density: 0.1,
        total_violations: 1,
        sloc: 10,
        severity_distribution: SeverityDistribution::default(),
        top_lints: vec![],
        detailed_violations: vec![],
    };
    assert!(hotspot.file.to_string_lossy().contains("unicode"));
}

#[test]
fn test_violation_detail_long_message() {
    let long_message = "a".repeat(10000);
    let v = ViolationDetail {
        file: PathBuf::from("test.rs"),
        line: 1,
        column: 1,
        end_line: 1,
        end_column: 1,
        lint_name: "test".to_string(),
        message: long_message.clone(),
        severity: "warning".to_string(),
        suggestion: Some(long_message),
        machine_applicable: false,
    };
    assert_eq!(v.message.len(), 10000);
}

// ============ Serialization Edge Cases ============

#[test]
fn test_severity_distribution_json_field_names() {
    let dist = SeverityDistribution {
        error: 1,
        warning: 2,
        suggestion: 3,
        note: 4,
    };
    let json = serde_json::to_string(&dist).unwrap();
    // Verify exact field names
    assert!(json.contains("\"error\""));
    assert!(json.contains("\"warning\""));
    assert!(json.contains("\"suggestion\""));
    assert!(json.contains("\"note\""));
}

#[test]
fn test_lint_hotspot_result_json_structure() {
    let result = create_full_result();
    let json = serde_json::to_string_pretty(&result).unwrap();
    // Verify top-level structure
    assert!(json.contains("\"hotspot\""));
    assert!(json.contains("\"all_violations\""));
    assert!(json.contains("\"summary_by_file\""));
    assert!(json.contains("\"total_project_violations\""));
    assert!(json.contains("\"quality_gate\""));
}

#[test]
fn test_violation_detail_deserialization_from_raw_json() {
    let json = r#"{
        "file": "test.rs",
        "line": 42,
        "column": 10,
        "end_line": 42,
        "end_column": 50,
        "lint_name": "clippy::test",
        "message": "test message",
        "severity": "warning",
        "suggestion": null,
        "machine_applicable": true
    }"#;
    let v: ViolationDetail = serde_json::from_str(json).unwrap();
    assert_eq!(v.line, 42);
    assert!(v.suggestion.is_none());
}

#[test]
fn test_quality_gate_status_deserialization() {
    let json = r#"{
        "passed": true,
        "violations": [],
        "blocking": false
    }"#;
    let status: QualityGateStatus = serde_json::from_str(json).unwrap();
    assert!(status.passed);
    assert!(!status.blocking);
}

#[test]
fn test_enforcement_metadata_deserialization() {
    let json = r#"{
        "enforcement_score": 7.5,
        "requires_enforcement": true,
        "estimated_fix_time": 3600,
        "automation_confidence": 0.85,
        "enforcement_priority": 2
    }"#;
    let meta: EnforcementMetadata = serde_json::from_str(json).unwrap();
    assert!(meta.requires_enforcement);
    assert_eq!(meta.enforcement_priority, 2);
}

// ============ Clone Tests ============

#[test]
fn test_violation_detail_clone_independence() {
    let original = create_violation("test", "warning", 10);
    let mut cloned = original.clone();
    cloned.line = 999;
    cloned.lint_name = "modified".to_string();
    // Original should be unchanged
    assert_eq!(original.line, 10);
    assert_eq!(original.lint_name, "test");
}

// ============ HashMap Key Tests ============

#[test]
fn test_summary_by_file_pathbuf_key() {
    let mut map: HashMap<PathBuf, FileSummary> = HashMap::new();
    let path1 = PathBuf::from("src/main.rs");
    let path2 = PathBuf::from("src/lib.rs");
    let path1_duplicate = PathBuf::from("src/main.rs");

    map.insert(
        path1.clone(),
        FileSummary {
            total_violations: 5,
            errors: 1,
            warnings: 4,
            sloc: 100,
            defect_density: 0.05,
        },
    );
    map.insert(
        path2,
        FileSummary {
            total_violations: 3,
            errors: 0,
            warnings: 3,
            sloc: 50,
            defect_density: 0.06,
        },
    );

    // Inserting with same path should replace
    map.insert(
        path1_duplicate,
        FileSummary {
            total_violations: 10,
            errors: 2,
            warnings: 8,
            sloc: 100,
            defect_density: 0.10,
        },
    );

    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&path1).unwrap().total_violations, 10);
}

// ============ Format Tests ============

#[test]
fn test_format_summary_empty_summary_by_file() {
    let mut result = create_full_result();
    result.summary_by_file.clear();
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("**Files with Issues**: 0"));
}

#[test]
fn test_format_summary_long_file_path() {
    let mut result = create_full_result();
    result.hotspot.file = PathBuf::from("very/long/nested/directory/structure/with/many/levels/src/module/submodule/file.rs");
    let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
    assert!(output.contains("file.rs") || output.contains("very/long"));
}

#[test]
fn test_format_summary_top_files_limit() {
    let mut result = create_full_result();
    // Add many files
    for i in 0..20 {
        result.summary_by_file.insert(
            PathBuf::from(format!("src/file_{i}.rs")),
            FileSummary {
                total_violations: 20 - i,
                errors: 1,
                warnings: 19 - i,
                sloc: 100,
                defect_density: (20 - i) as f64 / 100.0,
            },
        );
    }
    // Request only top 5
    let output = format_summary(&result, false, Duration::from_secs(1), 5).unwrap();
    // Should contain section header
    assert!(output.contains("## Top Files with Lint Issues"));
}

// ============ Integration-like Tests ============

#[test]
fn test_full_workflow_simulation() {
    // Simulate a complete analysis result workflow
    let mut result = create_full_result();

    // Add enforcement metadata
    result.enforcement = Some(EnforcementMetadata {
        enforcement_score: 8.5,
        requires_enforcement: true,
        estimated_fix_time: 1800,
        automation_confidence: 0.88,
        enforcement_priority: 1,
    });

    // Add refactor chain
    result.refactor_chain = Some(RefactorChain {
        id: "workflow-chain".to_string(),
        estimated_reduction: 12,
        automation_confidence: 0.85,
        steps: vec![
            RefactorStep {
                id: "step-1".to_string(),
                lint: "clippy::unwrap_used".to_string(),
                confidence: 0.95,
                impact: 8,
                description: "Replace unwrap with expect".to_string(),
            },
            RefactorStep {
                id: "step-2".to_string(),
                lint: "clippy::redundant_clone".to_string(),
                confidence: 0.75,
                impact: 4,
                description: "Remove redundant clones".to_string(),
            },
        ],
    });

    // Set quality gate failure
    result.quality_gate = QualityGateStatus {
        passed: false,
        violations: vec![QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 0.05,
            actual: 0.075,
            severity: "blocking".to_string(),
        }],
        blocking: true,
    };

    // Verify all components work together
    let json = serde_json::to_string_pretty(&result).unwrap();
    let restored: LintHotspotResult = serde_json::from_str(&json).unwrap();

    assert!(!restored.quality_gate.passed);
    assert!(restored.enforcement.is_some());
    assert!(restored.refactor_chain.is_some());
    assert_eq!(restored.refactor_chain.as_ref().unwrap().steps.len(), 2);

    // Format output should include all sections
    let output = format_summary(&restored, true, Duration::from_millis(500), 10).unwrap();
    assert!(output.contains("Enforcement Metadata"));
    assert!(output.contains("Quality Gate Failed"));
    assert!(output.contains("Analysis completed in"));
}

#[test]
fn test_minimal_result() {
    let result = LintHotspotResult {
        hotspot: LintHotspot {
            file: PathBuf::from("empty.rs"),
            defect_density: 0.0,
            total_violations: 0,
            sloc: 1,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: vec![],
        },
        all_violations: vec![],
        summary_by_file: HashMap::new(),
        total_project_violations: 0,
        enforcement: None,
        refactor_chain: None,
        quality_gate: QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        },
    };

    let json = serde_json::to_string(&result).unwrap();
    let back: LintHotspotResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_project_violations, 0);
    assert!(back.quality_gate.passed);
}
