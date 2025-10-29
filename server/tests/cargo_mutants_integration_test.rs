//! Integration test for PMAT-070-001 + PMAT-070-002
//!
//! Tests the complete workflow: CargoMutantsWrapper → JSON parsing → PMAT format
//!
//! This integration test validates that Phases 1 and 2 work together correctly.

use pmat::services::mutation::cargo_mutants_wrapper::CargoMutantsWrapper;
use pmat::services::mutation::json_parser::{CargoMutantsReport, MutantOutcome};
use pmat::services::mutation::types::MutantStatus;

#[test]
#[ignore] // Only run when cargo-mutants is installed
fn test_cargo_mutants_end_to_end_workflow() {
    // This test validates the complete workflow:
    // 1. Detect cargo-mutants installation
    // 2. Validate version
    // 3. Parse sample JSON (simulating cargo-mutants output)
    // 4. Convert to PMAT format
    // 5. Calculate statistics

    // Step 1: Detect cargo-mutants installation
    let wrapper = CargoMutantsWrapper::new();

    if wrapper.is_err() {
        println!("⚠️  Skipping integration test: cargo-mutants not installed");
        println!("   Install: cargo install cargo-mutants");
        return;
    }

    let wrapper = wrapper.unwrap();

    // Step 2: Validate version
    println!("✅ cargo-mutants detected");
    let version = wrapper.version().expect("Failed to get version");
    println!("   Version: {}", version);

    let validation = wrapper.validate_version();
    assert!(
        validation.is_ok(),
        "cargo-mutants version validation failed: {:?}",
        validation.err()
    );
    println!("✅ Version validation passed");

    // Step 3: Parse sample JSON (simulating real cargo-mutants output)
    let sample_json = r#"{
  "mutants": [
    {
      "outcome": "caught",
      "file": "src/lib.rs",
      "function": "add",
      "line": 10,
      "replacement": "0"
    },
    {
      "outcome": "missed",
      "file": "src/lib.rs",
      "function": "subtract",
      "line": 15,
      "replacement": "1"
    },
    {
      "outcome": "timeout",
      "file": "src/lib.rs",
      "function": "multiply",
      "line": 20,
      "replacement": "panic!()"
    },
    {
      "outcome": "unviable",
      "file": "src/lib.rs",
      "function": "divide",
      "line": 25,
      "replacement": "compile_error!()"
    }
  ]
}"#;

    let report =
        CargoMutantsReport::from_json(sample_json).expect("Failed to parse cargo-mutants JSON");

    println!("✅ Parsed {} mutants from JSON", report.mutants.len());
    assert_eq!(report.mutants.len(), 4, "Should parse 4 mutants");

    // Step 4: Convert to PMAT format
    let pmat_report = report.to_pmat_report();
    println!(
        "✅ Converted to PMAT format ({} mutants)",
        pmat_report.len()
    );
    assert_eq!(
        pmat_report.len(),
        4,
        "PMAT report should have same number of mutants"
    );

    // Validate outcome mapping
    assert_eq!(
        pmat_report[0].status,
        MutantStatus::Killed,
        "caught should map to Killed"
    );
    assert_eq!(
        pmat_report[1].status,
        MutantStatus::Survived,
        "missed should map to Survived"
    );
    assert_eq!(
        pmat_report[2].status,
        MutantStatus::Timeout,
        "timeout should map to Timeout"
    );
    assert_eq!(
        pmat_report[3].status,
        MutantStatus::CompileError,
        "unviable should map to CompileError"
    );

    println!("✅ Outcome mapping verified");

    // Step 5: Calculate statistics using utility methods
    let mutation_score = report.mutation_score();
    println!("✅ Mutation score calculated: {:.1}%", mutation_score);
    assert_eq!(
        mutation_score, 25.0,
        "Mutation score should be 25% (1/4 caught)"
    );

    let caught_count = report.count_by_outcome(MutantOutcome::Caught);
    let missed_count = report.count_by_outcome(MutantOutcome::Missed);
    let timeout_count = report.count_by_outcome(MutantOutcome::Timeout);
    let unviable_count = report.count_by_outcome(MutantOutcome::Unviable);

    assert_eq!(caught_count, 1, "Should have 1 caught mutant");
    assert_eq!(missed_count, 1, "Should have 1 missed mutant");
    assert_eq!(timeout_count, 1, "Should have 1 timeout mutant");
    assert_eq!(unviable_count, 1, "Should have 1 unviable mutant");

    println!("✅ Statistics verified:");
    println!("   Caught: {}", caught_count);
    println!("   Missed: {}", missed_count);
    println!("   Timeout: {}", timeout_count);
    println!("   Unviable: {}", unviable_count);

    println!("\n🎉 Integration test passed! Phases 1 & 2 working together correctly.");
}

#[test]
fn test_integration_with_empty_project() {
    // Test integration with edge case: empty project (no mutants)

    use std::path::PathBuf;

    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/empty");
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Failed to parse empty fixture");

    assert_eq!(report.mutants.len(), 0, "Should parse 0 mutants");

    let pmat_report = report.to_pmat_report();
    assert_eq!(pmat_report.len(), 0, "PMAT report should be empty");

    let mutation_score = report.mutation_score();
    assert_eq!(
        mutation_score, 0.0,
        "Mutation score should be 0% for empty report"
    );

    println!("✅ Empty project integration test passed");
}

#[test]
fn test_integration_outcome_counts() {
    // Test that outcome counting works correctly in integration

    use std::path::PathBuf;

    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/some-missed");
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Failed to parse fixture");

    let caught_count = report.count_by_outcome(MutantOutcome::Caught);
    let missed_count = report.count_by_outcome(MutantOutcome::Missed);
    let mutation_score = report.mutation_score();

    assert_eq!(caught_count, 4, "Should have 4 caught mutants");
    assert_eq!(missed_count, 1, "Should have 1 missed mutant");
    assert_eq!(
        mutation_score, 80.0,
        "Mutation score should be 80% (4/5 caught)"
    );

    println!("✅ Outcome counting integration test passed");
    println!("   Caught: {}", caught_count);
    println!("   Missed: {}", missed_count);
    println!("   Score: {:.1}%", mutation_score);
}
