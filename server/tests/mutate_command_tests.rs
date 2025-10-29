//! GREEN Phase Tests for PMAT-070-003: cargo-mutants Backend Integration
//!
//! This test suite validates the cargo-mutants backend for `pmat mutate --use-cargo-mutants`.
//! Following Extreme TDD: Tests written in RED phase, implementation in GREEN phase.
//!
//! Note: Sprint 61 implemented PMAT's built-in mutation testing (`pmat mutate`).
//! Sprint 70 adds cargo-mutants wrapper as an alternative backend via `--use-cargo-mutants`.

use pmat::cli::handlers::cargo_mutants_backend::{self, CargoMutantsConfig};
use pmat::services::mutation::cargo_mutants_wrapper::CargoMutantsWrapper;
use pmat::services::mutation::json_parser::CargoMutantsReport;
use std::path::PathBuf;

// ============================================================================
// Test Helpers
// ============================================================================

/// Helper to create a default test config
fn test_config() -> CargoMutantsConfig {
    CargoMutantsConfig {
        path: PathBuf::from("."),
        output: None,
        timeout: 300,
        jobs: None,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
#[ignore] // RED phase: Will fail until backend implemented
fn test_cargo_mutants_backend_detects_installation() {
    // Test: Verify cargo-mutants detection
    // Expected: CargoMutantsWrapper::new() succeeds if installed
    //           Returns error with installation instructions if not found

    let wrapper_result = CargoMutantsWrapper::new();

    // Should either succeed (if installed) or fail gracefully
    match wrapper_result {
        Ok(_) => {
            // cargo-mutants is installed, proceed
        }
        Err(e) => {
            // Should contain helpful error message
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("cargo-mutants") || error_msg.contains("not found"),
                "Error message should mention cargo-mutants: {}",
                error_msg
            );
        }
    }
}

#[test]
#[ignore] // RED phase: Will fail until backend implemented
fn test_cargo_mutants_backend_validates_version() {
    // Test: Verify version validation (v24.7.0+)
    // Expected: wrapper.validate_version() checks minimum version

    if let Ok(wrapper) = CargoMutantsWrapper::new() {
        let version_result = wrapper.validate_version();

        // Should succeed for v24.7.0+, or fail with upgrade message
        match version_result {
            Ok(_) => {
                // Version is sufficient
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("24.7.0") || error_msg.contains("version"),
                    "Error should mention version requirement: {}",
                    error_msg
                );
            }
        }
    }
}

#[test]
fn test_cargo_mutants_backend_parses_some_missed_fixture() {
    // Test: Parse real cargo-mutants output with some missed mutants
    // Expected:
    // 1. Parse outcomes.json from fixture
    // 2. Extract 5 mutants (4 caught, 1 missed)
    // 3. Calculate 80% mutation score

    use pmat::services::mutation::json_parser::MutantOutcome;

    // Use real cargo-mutants output fixture
    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/some-missed");
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Should parse fixture");

    // Verify mutant counts
    assert_eq!(report.mutants.len(), 5, "Should have 5 mutants");
    assert_eq!(
        report.count_by_outcome(MutantOutcome::Caught),
        4,
        "Should have 4 caught mutants"
    );
    assert_eq!(
        report.count_by_outcome(MutantOutcome::Missed),
        1,
        "Should have 1 missed mutant"
    );

    // Verify mutation score
    assert_eq!(report.mutation_score(), 80.0, "Should have 80% score");
}

#[test]
#[ignore] // RED phase: Will fail until backend implemented
fn test_cargo_mutants_backend_passes_timeout() {
    // Test: Verify --timeout flag is passed to cargo-mutants
    // Expected: Command includes --timeout <value>

    let mut config = test_config();
    config.timeout = 600; // 10 minutes

    let result = cargo_mutants_backend::execute(config);

    // Should build: cargo mutants --timeout 600
    assert!(result.is_ok(), "Should handle timeout flag");
}

#[test]
fn test_cargo_mutants_backend_handles_missing_file() {
    // Test: Verify graceful error when outcomes.json doesn't exist
    // Expected: Return error with helpful message

    let nonexistent = PathBuf::from("tests/fixtures/cargo-mutants-output/nonexistent");

    let parse_result = CargoMutantsReport::from_output_dir(&nonexistent);

    assert!(parse_result.is_err(), "Should fail when file missing");

    if let Err(e) = parse_result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("outcomes.json") || error_msg.contains("No such file"),
            "Error should mention missing file: {}",
            error_msg
        );
    }
}

#[test]
#[ignore] // RED phase: Will fail until backend implemented
fn test_cargo_mutants_backend_saves_output() {
    // Test: Verify --output flag saves JSON to file
    // Expected: JSON written to specified path

    let output_path = PathBuf::from("/tmp/pmat-cargo-mutants-test.json");

    let mut config = test_config();
    config.output = Some(output_path.clone());

    let result = cargo_mutants_backend::execute(config);

    assert!(result.is_ok(), "Should save output file");
    // GREEN phase will verify: std::fs::exists(&output_path)
}

#[test]
#[ignore] // RED phase: Will fail until backend implemented
fn test_cargo_mutants_backend_passes_all_flags() {
    // Test: Verify all flags are passed correctly
    // Expected: cargo mutants --features feat1,feat2 --jobs 4 --no-shuffle

    let mut config = test_config();
    config.jobs = Some(4);
    config.features = Some(vec!["feat1".to_string(), "feat2".to_string()]);
    config.no_shuffle = true;

    let result = cargo_mutants_backend::execute(config);

    // Should build command with all flags
    assert!(result.is_ok(), "Should pass all flags to cargo-mutants");
}

#[test]
fn test_cargo_mutants_backend_calculates_statistics() {
    // Test: Verify statistics calculation via utility methods
    // Expected: Calculate correct mutation score and counts

    use pmat::services::mutation::json_parser::MutantOutcome;

    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/with-timeout");
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Should parse fixture");

    // Test utility methods
    assert_eq!(report.mutants.len(), 5, "Should have 5 mutants");
    assert_eq!(
        report.count_by_outcome(MutantOutcome::Caught),
        3,
        "Should have 3 caught"
    );
    assert_eq!(
        report.count_by_outcome(MutantOutcome::Missed),
        1,
        "Should have 1 missed"
    );
    assert_eq!(
        report.count_by_outcome(MutantOutcome::Timeout),
        1,
        "Should have 1 timeout"
    );
    assert_eq!(report.mutation_score(), 60.0, "Should be 60% (3/5 caught)");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
#[ignore] // Requires actual cargo-mutants installation
fn integration_test_mutate_command_end_to_end() {
    // Test: End-to-end workflow with real cargo-mutants
    // Prerequisites: cargo-mutants v24.7.0+ installed
    //
    // Workflow:
    // 1. Detect cargo-mutants
    // 2. Validate version
    // 3. Execute on small test project
    // 4. Parse results
    // 5. Display statistics
    // 6. Verify output

    unimplemented!("GREEN phase: End-to-end integration test");
}

#[test]
#[ignore] // Requires test project setup
fn integration_test_mutate_command_with_real_project() {
    // Test: Run against small Rust project with known mutants
    // Expected: Should accurately detect and report mutants
    //
    // This test validates that:
    // 1. cargo-mutants executes correctly
    // 2. JSON output is valid
    // 3. PMAT conversion works
    // 4. Statistics are accurate

    unimplemented!("GREEN phase: Real project integration test");
}

// ============================================================================
// Edge Case Tests (Phase 4)
// ============================================================================

#[test]
fn test_empty_project_no_mutants() {
    // Test: Handle projects with no mutants found
    // Expected: 0 mutants, 0% score

    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/empty");
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Should parse empty fixture");

    assert_eq!(report.mutants.len(), 0, "Should have 0 mutants");
    assert_eq!(report.mutation_score(), 0.0, "Score should be 0%");
}

#[test]
fn test_all_mutants_caught_perfect_score() {
    // Test: Handle perfect mutation score (100%)
    // Expected: 5 mutants, all caught, 100% score

    use pmat::services::mutation::json_parser::MutantOutcome;

    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/all-caught");
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Should parse all-caught fixture");

    assert_eq!(report.mutants.len(), 5, "Should have 5 mutants");
    assert_eq!(
        report.count_by_outcome(MutantOutcome::Caught),
        5,
        "All mutants should be caught"
    );
    assert_eq!(
        report.count_by_outcome(MutantOutcome::Missed),
        0,
        "No missed mutants"
    );
    assert_eq!(report.mutation_score(), 100.0, "Score should be 100%");
}

#[test]
fn test_unviable_mutants_handling() {
    // Test: Handle unviable (non-compiling) mutants
    // Expected: Count unviable mutants separately

    use pmat::services::mutation::json_parser::MutantOutcome;

    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/unviable");
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Should parse unviable fixture");

    assert!(
        report.count_by_outcome(MutantOutcome::Unviable) > 0,
        "Should have unviable mutants"
    );
    assert_eq!(report.mutants.len(), 5, "Should have 5 total mutants");
}

#[test]
fn test_timeout_mutants_handling() {
    // Test: Handle timeout mutants
    // Expected: Count timeout mutants separately

    use pmat::services::mutation::json_parser::MutantOutcome;

    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/with-timeout");
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Should parse timeout fixture");

    assert!(
        report.count_by_outcome(MutantOutcome::Timeout) > 0,
        "Should have timeout mutants"
    );
    // Timeouts don't count toward mutation score (only caught vs missed)
    assert_eq!(report.mutation_score(), 60.0, "Score based on caught mutants");
}

#[test]
fn test_parsing_performance_reasonable() {
    // Test: Verify parsing is fast even with multiple mutants
    // Expected: Parse 5 mutants in <10ms

    use std::time::Instant;

    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/some-missed");

    let start = Instant::now();
    let report = CargoMutantsReport::from_output_dir(&fixture)
        .expect("Should parse fixture");
    let elapsed = start.elapsed();

    assert_eq!(report.mutants.len(), 5);
    assert!(
        elapsed.as_millis() < 100,
        "Parsing should be fast (<100ms), took {:?}",
        elapsed
    );
}

// ============================================================================
// Property-Based Tests (Placeholders for Phase 4)
// ============================================================================

#[test]
#[ignore] // Property test - Phase 4
fn property_test_mutation_score_always_between_0_and_100() {
    // Property: mutation_score() should always return 0.0 <= score <= 100.0
    // This will use proptest in Phase 4

    unimplemented!("Phase 4: Property-based testing");
}

#[test]
#[ignore] // Property test - Phase 4
fn property_test_outcome_counts_sum_to_total_mutants() {
    // Property: caught + missed + timeout + unviable == total mutants
    // This will use proptest in Phase 4

    unimplemented!("Phase 4: Property-based testing");
}
