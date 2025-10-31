//! BUG-001, BUG-002, BUG-003: Embed Command Errors - RED Phase Tests
//!
//! These tests verify that the embed subcommands work correctly:
//! - BUG-001: `pmat embed status` should not show invalid 'summary' format error
//! - BUG-002: `pmat embed sync` should not show invalid 'summary' format error
//! - BUG-003: `pmat embed` help should show embed-specific examples, not generic PMAT examples
//!
//! Current Status: 🔴 RED - These tests will FAIL until embed commands are fixed
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write failing tests for correct embed command behavior
//! 2. GREEN: Fix OutputFormat default values and add embed-specific examples
//! 3. REFACTOR: Ensure clean code
//! 4. COMMIT: Single atomic commit with fix

use std::process::Command;

// =============================================================================
// RED TEST 1: `pmat embed status` Should Work With Default Arguments
// =============================================================================

#[test]
#[ignore = "BUG-001: RED test - will fail until default_value changed from 'summary' to 'table'"]
fn test_embed_status_works_with_defaults() {
    // Act: Run `pmat embed status` with no arguments
    let output = Command::new("cargo")
        .args(["run", "--bin", "pmat", "--", "embed", "status"])
        .output()
        .expect("Failed to run pmat embed status");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert: Should NOT show "invalid value 'summary'" error
    assert!(
        !stderr.contains("invalid value 'summary'"),
        "Should not show 'summary' format error. Stderr: {}",
        stderr
    );

    // Should NOT show error about possible values
    assert!(
        !stderr.contains("[possible values: table, json, yaml]"),
        "Should not show format error. Stderr: {}",
        stderr
    );

    // Either succeeds or shows a different error (like missing database)
    // but NOT format validation error
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        !combined.contains("invalid value 'summary' for '--format'"),
        "Should not have format validation error"
    );
}

// =============================================================================
// RED TEST 2: `pmat embed sync` Should Work With Default Arguments
// =============================================================================

#[test]
#[ignore = "BUG-002: RED test - will fail until default_value changed from 'summary' to 'table'"]
fn test_embed_sync_works_with_defaults() {
    // Act: Run `pmat embed sync` with no arguments
    let output = Command::new("cargo")
        .args(["run", "--bin", "pmat", "--", "embed", "sync"])
        .output()
        .expect("Failed to run pmat embed sync");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert: Should NOT show "invalid value 'summary'" error
    assert!(
        !stderr.contains("invalid value 'summary'"),
        "Should not show 'summary' format error. Stderr: {}",
        stderr
    );

    // Should NOT show error about possible values
    assert!(
        !stderr.contains("[possible values: table, json, yaml]"),
        "Should not show format error. Stderr: {}",
        stderr
    );

    // Either succeeds or shows a different error (like missing database)
    // but NOT format validation error
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        !combined.contains("invalid value 'summary' for '--format'"),
        "Should not have format validation error"
    );
}

// =============================================================================
// RED TEST 3: `pmat embed status --format table` Should Work Explicitly
// =============================================================================

#[test]
#[ignore = "BUG-001: RED test - verifies table format is valid"]
fn test_embed_status_with_table_format() {
    // Act: Run with explicit --format table
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "pmat",
            "--",
            "embed",
            "status",
            "--format",
            "table",
        ])
        .output()
        .expect("Failed to run pmat embed status");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert: Should not show format validation error
    assert!(
        !stderr.contains("invalid value"),
        "Table format should be valid. Stderr: {}",
        stderr
    );
}

// =============================================================================
// RED TEST 4: `pmat embed` Help Should Show Embed-Specific Examples
// =============================================================================

#[test]
#[ignore = "BUG-003: RED test - will fail until embed-specific examples added"]
fn test_embed_help_shows_embed_examples() {
    // Act: Run `pmat embed --help`
    let output = Command::new("cargo")
        .args(["run", "--bin", "pmat", "--", "embed", "--help"])
        .output()
        .expect("Failed to run pmat embed --help");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert: Should show embed-specific examples
    assert!(
        stdout.contains("pmat embed sync") || stdout.contains("pmat embed status"),
        "Help should show embed-specific examples. Stdout: {}",
        stdout
    );

    // Should NOT show generic examples that are unrelated to embed
    assert!(
        !stdout.contains("pmat analyze complexity"),
        "Should not show generic 'analyze complexity' example in embed help. Stdout: {}",
        stdout
    );

    assert!(
        !stdout.contains("pmat context"),
        "Should not show generic 'context' example in embed help. Stdout: {}",
        stdout
    );

    assert!(
        !stdout.contains("pmat quality-gate"),
        "Should not show generic 'quality-gate' example in embed help. Stdout: {}",
        stdout
    );
}

// =============================================================================
// RED TEST 5: Embed Examples Should Be Relevant
// =============================================================================

#[test]
#[ignore = "BUG-003: RED test - verifies embed examples are relevant"]
fn test_embed_examples_are_relevant() {
    // Act: Run `pmat embed --help`
    let output = Command::new("cargo")
        .args(["run", "--bin", "pmat", "--", "embed", "--help"])
        .output()
        .expect("Failed to run pmat embed --help");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert: Should mention sync, status, or clear in examples
    let has_embed_examples = stdout.contains("sync")
        || stdout.contains("status")
        || stdout.contains("clear")
        || stdout.contains("embedding");

    // Look for examples section
    if stdout.contains("EXAMPLES:") {
        assert!(
            has_embed_examples,
            "Examples section should mention embed-related commands. Stdout: {}",
            stdout
        );
    }
}

// =============================================================================
// RED TEST 6: Status and Sync Should Have Same Fix
// =============================================================================

#[test]
#[ignore = "BUG-001-002: RED test - both commands should have consistent defaults"]
fn test_status_and_sync_have_valid_defaults() {
    // Both commands should work with defaults (no format specified)

    // Test status
    let status_output = Command::new("cargo")
        .args(["run", "--bin", "pmat", "--", "embed", "status"])
        .output()
        .expect("Failed to run pmat embed status");

    let status_stderr = String::from_utf8_lossy(&status_output.stderr);

    // Test sync
    let sync_output = Command::new("cargo")
        .args(["run", "--bin", "pmat", "--", "embed", "sync"])
        .output()
        .expect("Failed to run pmat embed sync");

    let sync_stderr = String::from_utf8_lossy(&sync_output.stderr);

    // Both should not have format errors
    assert!(
        !status_stderr.contains("invalid value 'summary'"),
        "Status should not have format error"
    );

    assert!(
        !sync_stderr.contains("invalid value 'summary'"),
        "Sync should not have format error"
    );
}

// =============================================================================
// RED TEST 7: Embed Clear Should Also Work (Sanity Check)
// =============================================================================

#[test]
#[ignore = "BUG-001-002: RED test - clear command should also work"]
fn test_embed_clear_help_works() {
    // Act: Run `pmat embed clear --help`
    let output = Command::new("cargo")
        .args(["run", "--bin", "pmat", "--", "embed", "clear", "--help"])
        .output()
        .expect("Failed to run pmat embed clear --help");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert: Should show help without errors
    assert!(
        output.status.success() || stdout.contains("Clear all embeddings"),
        "Clear command help should work. Stdout: {}",
        stdout
    );
}
