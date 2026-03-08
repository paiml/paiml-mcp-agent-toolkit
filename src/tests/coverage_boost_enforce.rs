#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for cli/handlers/enforce_handlers.rs
//! Tests: type construction, serde, pure functions, state machine

use crate::cli::handlers::enforce_handlers::{
    format_violations_output, handle_complete_state, handle_refactoring_state,
    handle_violating_state, EnforcementProgress, EnforcementResult, EnforcementState,
    QualityProfile, QualityViolation,
};
use crate::cli::EnforceOutputFormat;
use std::path::PathBuf;

/// Strip ANSI escape codes from a string for test assertions
fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

// --- EnforcementState tests ---

#[test]
fn test_enforcement_state_variants_serde() {
    let states = vec![
        EnforcementState::Analyzing,
        EnforcementState::Violating,
        EnforcementState::Refactoring,
        EnforcementState::Validating,
        EnforcementState::Complete,
    ];
    for state in &states {
        let json = serde_json::to_string(state).unwrap();
        let back: EnforcementState = serde_json::from_str(&json).unwrap();
        assert_eq!(*state, back);
        let _ = format!("{:?}", state);
    }
}

#[test]
fn test_enforcement_state_serde_screaming_snake() {
    let json = serde_json::to_string(&EnforcementState::Analyzing).unwrap();
    assert!(json.contains("ANALYZING"));
}

#[test]
fn test_enforcement_state_copy() {
    let state = EnforcementState::Complete;
    let copied = state;
    assert_eq!(copied, EnforcementState::Complete);
}

// --- QualityViolation tests ---

#[test]
fn test_quality_violation_serde() {
    let v = QualityViolation {
        violation_type: "complexity".to_string(),
        severity: "high".to_string(),
        location: "src/main.rs:42".to_string(),
        current: 25.0,
        target: 10.0,
        suggestion: "Extract method".to_string(),
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: QualityViolation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.violation_type, "complexity");
    assert_eq!(back.current, 25.0);
}

#[test]
fn test_quality_violation_clone_debug() {
    let v = QualityViolation {
        violation_type: "satd".to_string(),
        severity: "medium".to_string(),
        location: "src/lib.rs:10".to_string(),
        current: 5.0,
        target: 0.0,
        suggestion: "Remove TODO".to_string(),
    };
    let cloned = v.clone();
    assert_eq!(cloned.severity, "medium");
    let _ = format!("{:?}", v);
}

// --- EnforcementProgress tests ---

#[test]
fn test_enforcement_progress_serde() {
    let progress = EnforcementProgress {
        files_completed: 10,
        files_remaining: 90,
        estimated_iterations: 5,
    };
    let json = serde_json::to_string(&progress).unwrap();
    let back: EnforcementProgress = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files_completed, 10);
    assert_eq!(back.estimated_iterations, 5);
}

// --- EnforcementResult tests ---

#[test]
fn test_enforcement_result_serde() {
    let result = EnforcementResult {
        state: EnforcementState::Violating,
        score: 0.65,
        target: 1.0,
        current_file: Some("src/main.rs".to_string()),
        violations: vec![QualityViolation {
            violation_type: "complexity".to_string(),
            severity: "high".to_string(),
            location: "src/main.rs:42".to_string(),
            current: 30.0,
            target: 10.0,
            suggestion: "Refactor".to_string(),
        }],
        next_action: "manual_intervention_required".to_string(),
        progress: EnforcementProgress {
            files_completed: 0,
            files_remaining: 5,
            estimated_iterations: 3,
        },
    };
    let json = serde_json::to_string(&result).unwrap();
    let back: EnforcementResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.state, EnforcementState::Violating);
    assert_eq!(back.score, 0.65);
    assert_eq!(back.violations.len(), 1);
}

// --- QualityProfile tests ---

#[test]
fn test_quality_profile_default() {
    let profile = QualityProfile::default();
    assert!(profile.coverage_min >= 80.0);
    assert!(profile.complexity_max <= 20);
    assert_eq!(profile.satd_allowed, 0);
}

#[test]
fn test_quality_profile_serde() {
    let profile = QualityProfile::default();
    let json = serde_json::to_string(&profile).unwrap();
    let back: QualityProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.coverage_min, profile.coverage_min);
    assert_eq!(back.complexity_max, profile.complexity_max);
}

// --- handle_complete_state tests ---

#[test]
fn test_handle_complete_state() {
    let result = handle_complete_state().unwrap();
    assert_eq!(result.state, EnforcementState::Complete);
    assert_eq!(result.score, 1.0);
    assert!(result.violations.is_empty());
    assert_eq!(result.next_action, "none");
    assert_eq!(result.progress.files_remaining, 0);
}

// --- handle_violating_state tests ---

#[test]
fn test_handle_violating_state_with_apply() {
    let violations = vec![QualityViolation {
        violation_type: "complexity".to_string(),
        severity: "high".to_string(),
        location: "src/test.rs".to_string(),
        current: 20.0,
        target: 10.0,
        suggestion: "Split function".to_string(),
    }];
    let result = handle_violating_state(violations, 0.5, true, false, None).unwrap();
    assert_eq!(result.state, EnforcementState::Refactoring);
    assert_eq!(result.next_action, "apply_refactoring");
}

#[test]
fn test_handle_violating_state_no_apply() {
    let violations = vec![QualityViolation {
        violation_type: "satd".to_string(),
        severity: "medium".to_string(),
        location: "src/lib.rs".to_string(),
        current: 3.0,
        target: 0.0,
        suggestion: "Remove TODO comments".to_string(),
    }];
    let result = handle_violating_state(violations, 0.7, false, false, None).unwrap();
    assert_eq!(result.state, EnforcementState::Violating);
    assert_eq!(result.next_action, "manual_intervention_required");
}

#[test]
fn test_handle_violating_state_dry_run() {
    let violations = vec![];
    let result = handle_violating_state(violations, 0.8, true, true, None).unwrap();
    // dry_run=true means we don't apply, so state stays Violating
    assert_eq!(result.state, EnforcementState::Violating);
}

#[test]
fn test_handle_violating_state_with_file() {
    let file = PathBuf::from("src/specific.rs");
    let violations = vec![];
    let result = handle_violating_state(violations, 0.9, true, false, Some(&file)).unwrap();
    assert!(result.current_file.is_some());
    assert!(result.current_file.unwrap().contains("specific.rs"));
}

// --- handle_refactoring_state tests ---

#[test]
fn test_handle_refactoring_state() {
    let result = handle_refactoring_state(0.7, None).unwrap();
    assert_eq!(result.state, EnforcementState::Validating);
    assert!(result.score > 0.7); // Should have improvement
    assert!(result.violations.is_empty());
    assert_eq!(result.next_action, "validate_changes");
}

#[test]
fn test_handle_refactoring_state_with_file() {
    let file = PathBuf::from("src/module.rs");
    let result = handle_refactoring_state(0.5, Some(&file)).unwrap();
    assert!(result.current_file.is_some());
}

// --- format_violations_output tests ---

#[test]
fn test_format_violations_json() {
    let violations = vec![
        QualityViolation {
            violation_type: "complexity".to_string(),
            severity: "high".to_string(),
            location: "src/main.rs:10".to_string(),
            current: 25.0,
            target: 10.0,
            suggestion: "Extract method".to_string(),
        },
        QualityViolation {
            violation_type: "satd".to_string(),
            severity: "medium".to_string(),
            location: "src/lib.rs:20".to_string(),
            current: 3.0,
            target: 0.0,
            suggestion: "Remove TODO".to_string(),
        },
    ];
    let profile = QualityProfile::default();
    let output =
        format_violations_output(&violations, &profile, EnforceOutputFormat::Json).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["summary"]["total"], 2);
}

#[test]
fn test_format_violations_text() {
    let violations = vec![QualityViolation {
        violation_type: "complexity".to_string(),
        severity: "high".to_string(),
        location: "src/main.rs:10".to_string(),
        current: 25.0,
        target: 10.0,
        suggestion: "Reduce complexity".to_string(),
    }];
    let profile = QualityProfile::default();
    let output =
        format_violations_output(&violations, &profile, EnforceOutputFormat::Summary).unwrap();
    let plain = strip_ansi(&output);
    assert!(plain.contains("COMPLEXITY"));
    assert!(plain.contains("high"));
    assert!(plain.contains("25"));
}

#[test]
fn test_format_violations_empty() {
    let violations: Vec<QualityViolation> = vec![];
    let profile = QualityProfile::default();
    let output =
        format_violations_output(&violations, &profile, EnforceOutputFormat::Summary).unwrap();
    let plain = strip_ansi(&output);
    assert!(plain.contains("0 violations"));
}
