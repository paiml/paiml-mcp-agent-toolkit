#![cfg_attr(coverage_nightly, coverage(off))]
//! Edge case tests for README validation: no evidence, negative claims, mixed results, formatting

use super::types::{OutputFormat, ValidateReadmeCmd};
#[allow(unused_imports)]
use crate::services::hallucination_detector::{
    Claim, ClaimType, Entity, Evidence, ValidationResult, ValidationStatus,
};
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use tempfile::NamedTempFile;

// ============================================================================
// Edge case tests
// ============================================================================

#[test]
fn test_validation_result_without_evidence() {
    let result = ValidationResult {
        claim: Claim {
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            text: "PMAT can analyze".to_string(),
            claim_type: ClaimType::Capability,
            entities: vec![],
            is_negative: false,
        },
        status: ValidationStatus::Inconclusive,
        evidence: None,
        error_message: Some("No evidence available".to_string()),
        confidence: 0.5,
    };

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

    let results = vec![(PathBuf::from("test.md"), vec![result])];

    // Test all output formats with no evidence
    cmd.print_text_summary(&results, 0, 0, 0);
    assert!(cmd.print_json_summary(&results).is_ok());
    assert!(cmd.print_junit_summary(&results).is_ok());
}

#[test]
fn test_negative_claim_handling() {
    let mut deep_context_file = NamedTempFile::new().unwrap();
    writeln!(
        deep_context_file,
        r#"
Supported languages:
- Rust
        "#
    )
    .unwrap();

    // Negative claim (PMAT cannot compile)
    let mut readme_file = NamedTempFile::new().unwrap();
    writeln!(readme_file, "PMAT cannot compile Rust code.").unwrap();

    let cmd = ValidateReadmeCmd {
        targets: vec![readme_file.path().to_path_buf()],
        deep_context: deep_context_file.path().to_path_buf(),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let result = cmd.execute();
    assert!(result.is_ok());
    // Negative claims about compile should be fine (it's correct that PMAT cannot compile)
}

#[test]
fn test_supports_pattern() {
    let mut deep_context_file = NamedTempFile::new().unwrap();
    writeln!(
        deep_context_file,
        r#"
Supported languages:
- Rust
- TypeScript
        "#
    )
    .unwrap();

    // Using "supports" pattern
    let mut readme_file = NamedTempFile::new().unwrap();
    writeln!(readme_file, "PMAT supports Rust and TypeScript analysis.").unwrap();

    let cmd = ValidateReadmeCmd {
        targets: vec![readme_file.path().to_path_buf()],
        deep_context: deep_context_file.path().to_path_buf(),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let result = cmd.execute();
    assert!(result.is_ok());
}

#[test]
fn test_empty_deep_context() {
    let mut deep_context_file = NamedTempFile::new().unwrap();
    writeln!(deep_context_file, "# Empty context").unwrap();

    let mut readme_file = NamedTempFile::new().unwrap();
    writeln!(readme_file, "PMAT can analyze Rust code.").unwrap();

    let cmd = ValidateReadmeCmd {
        targets: vec![readme_file.path().to_path_buf()],
        deep_context: deep_context_file.path().to_path_buf(),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let result = cmd.execute();
    assert!(result.is_ok());
}

#[test]
fn test_multiple_claims_mixed_results() {
    let mut deep_context_file = NamedTempFile::new().unwrap();
    writeln!(
        deep_context_file,
        r#"
Functions:
- analyze()
- parse()

Supported languages:
- Rust
- TypeScript
        "#
    )
    .unwrap();

    let mut readme_file = NamedTempFile::new().unwrap();
    writeln!(readme_file, "PMAT can analyze Rust code.").unwrap();
    writeln!(readme_file, "PMAT can compile TypeScript.").unwrap();
    writeln!(readme_file, "PMAT supports JavaScript analysis.").unwrap();

    let cmd = ValidateReadmeCmd {
        targets: vec![readme_file.path().to_path_buf()],
        deep_context: deep_context_file.path().to_path_buf(),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let result = cmd.execute();
    assert!(result.is_ok());
    // Should fail due to "compile" contradiction
    assert_eq!(result.unwrap(), ExitCode::FAILURE);
}

#[test]
fn test_claim_type_debug_formatting() {
    let claim = Claim {
        source_file: PathBuf::from("test.md"),
        line_number: 1,
        text: "test".to_string(),
        claim_type: ClaimType::Capability,
        entities: vec![Entity::Language("Rust".to_string())],
        is_negative: false,
    };

    // Test debug formatting works
    let debug_output = format!("{:?}", claim.claim_type);
    assert_eq!(debug_output, "Capability");
}

#[test]
fn test_validation_status_debug_formatting() {
    assert_eq!(format!("{:?}", ValidationStatus::Verified), "Verified");
    assert_eq!(
        format!("{:?}", ValidationStatus::Contradiction),
        "Contradiction"
    );
    assert_eq!(format!("{:?}", ValidationStatus::Unverified), "Unverified");
    assert_eq!(format!("{:?}", ValidationStatus::NotFound), "NotFound");
    assert_eq!(format!("{:?}", ValidationStatus::Outdated), "Outdated");
    assert_eq!(
        format!("{:?}", ValidationStatus::Inconclusive),
        "Inconclusive"
    );
}

#[test]
fn test_entity_debug_formatting() {
    let entities = vec![
        Entity::Language("Rust".to_string()),
        Entity::Function("main".to_string()),
        Entity::File("test.rs".to_string()),
        Entity::Module("cli".to_string()),
        Entity::Capability("analyze".to_string()),
    ];

    for entity in &entities {
        // Just verify debug formatting doesn't panic
        let _ = format!("{:?}", entity);
    }
}

// ============================================================================
// Execute edge case tests
// ============================================================================

#[test]
fn test_execute_with_multiple_targets() {
    let mut deep_context_file = NamedTempFile::new().unwrap();
    writeln!(
        deep_context_file,
        r#"
Supported languages:
- Rust
- TypeScript
        "#
    )
    .unwrap();

    let mut readme_file = NamedTempFile::new().unwrap();
    writeln!(readme_file, "PMAT can analyze Rust code.").unwrap();

    let mut claude_file = NamedTempFile::new().unwrap();
    writeln!(claude_file, "PMAT supports TypeScript analysis.").unwrap();

    let cmd = ValidateReadmeCmd {
        targets: vec![
            readme_file.path().to_path_buf(),
            claude_file.path().to_path_buf(),
        ],
        deep_context: deep_context_file.path().to_path_buf(),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let result = cmd.execute();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitCode::SUCCESS);
}

#[test]
fn test_execute_with_no_claims() {
    let mut deep_context_file = NamedTempFile::new().unwrap();
    writeln!(
        deep_context_file,
        r#"
Supported languages:
- Rust
        "#
    )
    .unwrap();

    // README with no PMAT claims
    let mut readme_file = NamedTempFile::new().unwrap();
    writeln!(
        readme_file,
        "This is a simple README without any PMAT claims."
    )
    .unwrap();
    writeln!(readme_file, "It just contains some text.").unwrap();

    let cmd = ValidateReadmeCmd {
        targets: vec![readme_file.path().to_path_buf()],
        deep_context: deep_context_file.path().to_path_buf(),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let result = cmd.execute();
    assert!(result.is_ok());
    // REGRESSION: this used to assert SUCCESS. A document from which no
    // verifiable claim could be extracted was never checked, so reporting it
    // as a pass let fabricated documentation through the gate. Nothing
    // checked is now an explicit failure, not a green tick.
    assert_eq!(result.unwrap(), ExitCode::FAILURE);
}

/// REGRESSION (blocker: `pmat validate-readme` certified fabricated docs).
///
/// A document citing a file that does not exist must fail the gate. Before the
/// fix the extractor recognised only three "PMAT can/cannot/supports" phrases,
/// so this document yielded zero claims and printed
/// "All documentation claims are verified!" with exit 0.
#[test]
fn test_execute_fails_on_fabricated_file_reference() {
    let mut deep_context_file = NamedTempFile::new().unwrap();
    writeln!(
        deep_context_file,
        r#"
Functions:
- main()

Supported languages:
- Rust
        "#
    )
    .unwrap();

    let mut readme_file = NamedTempFile::new().unwrap();
    writeln!(
        readme_file,
        "The entry point lives in `src/totally/made/up/nonexistent_module.rs` today."
    )
    .unwrap();

    let cmd = ValidateReadmeCmd {
        targets: vec![readme_file.path().to_path_buf()],
        deep_context: deep_context_file.path().to_path_buf(),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let result = cmd.execute();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitCode::FAILURE);
}

#[test]
fn test_execute_with_code_blocks_ignored() {
    let mut deep_context_file = NamedTempFile::new().unwrap();
    writeln!(
        deep_context_file,
        r#"
Supported languages:
- Rust
        "#
    )
    .unwrap();

    // README with a false claim inside a code block (must be ignored) plus a
    // real claim outside it, so the run has something to actually verify —
    // otherwise "nothing was checked" would be indistinguishable from a pass.
    let mut readme_file = NamedTempFile::new().unwrap();
    writeln!(readme_file, "# Usage").unwrap();
    writeln!(readme_file).unwrap();
    writeln!(readme_file, "PMAT can analyze Rust code.").unwrap();
    writeln!(readme_file, "```bash").unwrap();
    writeln!(readme_file, "# PMAT can compile code inside code block").unwrap();
    writeln!(readme_file, "pmat analyze").unwrap();
    writeln!(readme_file, "```").unwrap();

    let cmd = ValidateReadmeCmd {
        targets: vec![readme_file.path().to_path_buf()],
        deep_context: deep_context_file.path().to_path_buf(),
        verified_threshold: 0.9,
        contradiction_threshold: 0.3,
        fail_on_contradiction: true,
        fail_on_unverified: false,
        output: OutputFormat::Text,
        failures_only: false,
        verbose: false,
    };

    let result = cmd.execute();
    assert!(result.is_ok());
    // Should succeed because the claim is inside a code block and ignored
    assert_eq!(result.unwrap(), ExitCode::SUCCESS);
}
