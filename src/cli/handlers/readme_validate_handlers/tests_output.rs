#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for README validation output formatting: text, JSON, and JUnit

use super::types::{OutputFormat, ValidateReadmeCmd};
use crate::services::hallucination_detector::{
    Claim, ClaimType, Entity, Evidence, ValidationResult, ValidationStatus,
};
use std::path::PathBuf;

// ============================================================================
// Test helper
// ============================================================================

fn create_test_validation_result(
    text: &str,
    status: ValidationStatus,
    confidence: f32,
) -> ValidationResult {
    ValidationResult {
        claim: Claim {
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            text: text.to_string(),
            claim_type: ClaimType::Capability,
            entities: vec![Entity::Capability("analyze".to_string())],
            is_negative: false,
        },
        status,
        evidence: Some(Evidence {
            source: "test".to_string(),
            similarity: confidence,
            content: "test evidence".to_string(),
        }),
        error_message: None,
        confidence,
    }
}

// ============================================================================
// print_text_summary tests
// ============================================================================

#[test]
fn test_print_text_summary_all_verified() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![create_test_validation_result(
            "PMAT can analyze",
            ValidationStatus::Verified,
            0.95,
        )],
    )];

    // This just tests that it doesn't panic - actual output goes to stdout
    cmd.print_text_summary(&results, 1, 0, 0);
}

#[test]
fn test_print_text_summary_with_contradictions() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![create_test_validation_result(
            "PMAT can compile",
            ValidationStatus::Contradiction,
            0.2,
        )],
    )];

    cmd.print_text_summary(&results, 0, 1, 0);
}

#[test]
fn test_print_text_summary_with_unverified() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![create_test_validation_result(
            "PMAT can analyze",
            ValidationStatus::Unverified,
            0.5,
        )],
    )];

    cmd.print_text_summary(&results, 0, 0, 1);
}

#[test]
fn test_print_text_summary_failures_only() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: true,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![
            create_test_validation_result("PMAT can analyze", ValidationStatus::Verified, 0.95),
            create_test_validation_result(
                "PMAT can compile",
                ValidationStatus::Contradiction,
                0.2,
            ),
        ],
    )];

    // With failures_only, only contradictions should be printed
    cmd.print_text_summary(&results, 1, 1, 0);
}

#[test]
fn test_print_text_summary_verbose() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: true,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![create_test_validation_result(
            "PMAT can analyze",
            ValidationStatus::Verified,
            0.95,
        )],
    )];

    cmd.print_text_summary(&results, 1, 0, 0);
}

#[test]
fn test_print_text_summary_all_status_icons() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![
            create_test_validation_result("claim1", ValidationStatus::Verified, 0.95),
            create_test_validation_result("claim2", ValidationStatus::Contradiction, 0.2),
            create_test_validation_result("claim3", ValidationStatus::Unverified, 0.5),
            create_test_validation_result("claim4", ValidationStatus::NotFound, 0.3),
            create_test_validation_result("claim5", ValidationStatus::Outdated, 0.4),
            create_test_validation_result("claim6", ValidationStatus::Inconclusive, 0.5),
        ],
    )];

    cmd.print_text_summary(&results, 1, 1, 3);
}

// ============================================================================
// print_json_summary tests
// ============================================================================

#[test]
fn test_print_json_summary_empty_results() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Json,
        failures_only: false,
        verbose: false,
    };

    let results: Vec<(PathBuf, Vec<ValidationResult>)> = vec![];
    let result = cmd.print_json_summary(&results);
    assert!(result.is_ok());
}

#[test]
fn test_print_json_summary_with_results() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Json,
        failures_only: false,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![create_test_validation_result(
            "PMAT can analyze",
            ValidationStatus::Verified,
            0.95,
        )],
    )];

    let result = cmd.print_json_summary(&results);
    assert!(result.is_ok());
}

#[test]
fn test_print_json_summary_counts_statuses() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Json,
        failures_only: false,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![
            create_test_validation_result("claim1", ValidationStatus::Verified, 0.95),
            create_test_validation_result("claim2", ValidationStatus::Contradiction, 0.2),
            create_test_validation_result("claim3", ValidationStatus::Unverified, 0.5),
            create_test_validation_result("claim4", ValidationStatus::NotFound, 0.3),
            create_test_validation_result("claim5", ValidationStatus::Outdated, 0.4),
        ],
    )];

    let result = cmd.print_json_summary(&results);
    assert!(result.is_ok());
}

// ============================================================================
// print_junit_summary tests
// ============================================================================

#[test]
fn test_print_junit_summary_empty_results() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Junit,
        failures_only: false,
        verbose: false,
    };

    let results: Vec<(PathBuf, Vec<ValidationResult>)> = vec![];
    let result = cmd.print_junit_summary(&results);
    assert!(result.is_ok());
}

#[test]
fn test_print_junit_summary_with_passing_tests() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Junit,
        failures_only: false,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![create_test_validation_result(
            "PMAT can analyze",
            ValidationStatus::Verified,
            0.95,
        )],
    )];

    let result = cmd.print_junit_summary(&results);
    assert!(result.is_ok());
}

#[test]
fn test_print_junit_summary_with_failures() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Junit,
        failures_only: false,
        verbose: false,
    };

    let results = vec![(
        PathBuf::from("test.md"),
        vec![
            create_test_validation_result("claim1", ValidationStatus::Contradiction, 0.2),
            create_test_validation_result("claim2", ValidationStatus::Unverified, 0.5),
            create_test_validation_result("claim3", ValidationStatus::NotFound, 0.3),
            create_test_validation_result("claim4", ValidationStatus::Outdated, 0.4),
        ],
    )];

    let result = cmd.print_junit_summary(&results);
    assert!(result.is_ok());
}

#[test]
fn test_print_junit_summary_with_special_chars_in_claim() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Junit,
        failures_only: false,
        verbose: false,
    };

    // Create result with special XML characters in the claim text
    let mut result_with_special_chars = create_test_validation_result(
        "PMAT can analyze <Rust> & 'TypeScript' code",
        ValidationStatus::Contradiction,
        0.2,
    );
    result_with_special_chars.evidence = Some(Evidence {
        source: "test".to_string(),
        similarity: 0.2,
        content: "Evidence with <special> & \"chars\"".to_string(),
    });

    let results = vec![(PathBuf::from("test.md"), vec![result_with_special_chars])];

    let result = cmd.print_junit_summary(&results);
    assert!(result.is_ok());
}

#[test]
fn test_print_junit_summary_long_claim_text_truncation() {
    let cmd = ValidateReadmeCmd {
        targets: vec![PathBuf::from("test.md")],
        deep_context: PathBuf::from("dc.md"),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Junit,
        failures_only: false,
        verbose: false,
    };

    // Create result with very long claim text (> 50 chars)
    let long_claim = "PMAT can analyze very complex Rust code with many features and capabilities that span multiple lines";
    let result = create_test_validation_result(long_claim, ValidationStatus::Verified, 0.95);

    let results = vec![(PathBuf::from("test.md"), vec![result])];

    let result = cmd.print_junit_summary(&results);
    assert!(result.is_ok());
}
