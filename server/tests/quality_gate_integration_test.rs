//! Integration test for quality-gate as single source of truth
//!
//! This test verifies that quality-gate properly delegates to underlying
//! analysis tools and provides consistent results across all interfaces
//! (CLI, MCP, services).
//!
//! TDD: Test written as part of Sprint 78 quality restoration

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test that quality-gate correctly counts only violations above threshold
#[test]
fn test_quality_gate_counts_violations_correctly() {
    // Create test files with known complexity
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Write a function with complexity 12
    let test_code = r#"
fn complex_function(x: i32, y: i32, z: i32) -> i32 {
    let mut result = 0;
    if x > 0 {
        if y > 0 {
            if z > 0 {
                result = x + y + z;
            } else if z < -10 {
                result = x + y - z;
            } else {
                result = x + y;
            }
        } else if y < -10 {
            if z > 0 {
                result = x - y + z;
            } else {
                result = x - y - z;
            }
        } else {
            result = x * 2;
        }
    } else if x < -10 {
        if y > 0 {
            result = -x + y;
        } else if y < -10 {
            result = -x - y;
        } else {
            result = -x;
        }
    } else {
        if z > 0 {
            result = z;
        } else {
            result = -z;
        }
    }
    result
}
"#;

    fs::write(&test_file, test_code).unwrap();

    // Get binary path
    let binary_path = get_pmat_binary_path();

    // Run quality-gate
    let quality_output = std::process::Command::new(&binary_path)
        .args(&[
            "quality-gate",
            "--file",
            test_file.to_str().unwrap(),
            "--checks",
            "complexity",
            "--max-complexity-p99",
            "10",
        ])
        .output()
        .expect("Failed to run quality-gate");

    let quality_stderr = String::from_utf8_lossy(&quality_output.stderr);
    let quality_stdout = String::from_utf8_lossy(&quality_output.stdout);

    // Should find exactly 1 violation (the function with complexity 12 > threshold 10)
    assert!(
        quality_stderr.contains("1 violations found")
            || quality_stderr.contains("1 violation found"),
        "quality-gate should find exactly 1 violation, got stderr:\n{}\nstdout:\n{}",
        quality_stderr,
        quality_stdout
    );

    // Verify it found the complex function
    assert!(
        quality_stdout.contains("complex_function")
            || quality_stdout.contains("Complexity Issues: 1"),
        "quality-gate should report the complex function violation in output"
    );
}

/// Test that quality-gate can run checks correctly
#[test]
fn test_quality_gate_multiple_checks() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_multi.rs");

    // Write code with complexity and SATD issues
    let test_code = r#"
// TODO: This is a SATD comment that should be detected
fn complex_function(x: i32) -> i32 {
    // High complexity function
    if x > 0 {
        if x > 10 {
            if x > 20 {
                return x * 2;
            }
            return x + 1;
        }
        return x;
    }
    0
}

// FIXME: Another SATD comment
fn unused_function() -> i32 {
    42
}
"#;

    fs::write(&test_file, test_code).unwrap();

    let binary_path = get_pmat_binary_path();

    // Run quality-gate with complexity check
    let complexity_output = std::process::Command::new(&binary_path)
        .args(&[
            "quality-gate",
            "--file",
            test_file.to_str().unwrap(),
            "--checks",
            "complexity",
            "--max-complexity-p99",
            "3", // Low threshold to trigger violations
        ])
        .output()
        .expect("Failed to run quality-gate for complexity");

    let complexity_stderr = String::from_utf8_lossy(&complexity_output.stderr);

    // Should check complexity
    assert!(
        complexity_stderr.contains("Checking complexity")
            || complexity_stderr.contains("🔍 Checking complexity"),
        "Should check complexity, got:\n{}",
        complexity_stderr
    );

    // Should find complexity violations
    assert!(
        complexity_stderr.contains("violations found"),
        "Should find complexity violations"
    );

    // Run quality-gate with SATD check
    let satd_output = std::process::Command::new(&binary_path)
        .args(&[
            "quality-gate",
            "--file",
            test_file.to_str().unwrap(),
            "--checks",
            "satd",
        ])
        .output()
        .expect("Failed to run quality-gate for SATD");

    let satd_stderr = String::from_utf8_lossy(&satd_output.stderr);

    // Should check SATD
    assert!(
        satd_stderr.contains("Checking SATD") || satd_stderr.contains("🔍 Checking SATD"),
        "Should check SATD, got:\n{}",
        satd_stderr
    );

    // Should find SATD violations
    assert!(
        satd_stderr.contains("2 violations found"),
        "Should find 2 SATD violations (TODO and FIXME)"
    );
}

/// Test that quality-gate respects thresholds correctly
#[test]
fn test_quality_gate_threshold_boundaries() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("threshold_test.rs");

    // Simple function with known complexity 5
    let test_code = r#"
fn simple_function(x: i32) -> i32 {
    // This function has complexity 5
    if x > 10 {        // +1
        if x > 20 {    // +1
            x * 2
        } else {
            x + 1
        }
    } else if x < 0 {  // +1
        if x < -10 {   // +1
            -x
        } else {
            0
        }
    } else {
        x
    }
}
"#;

    fs::write(&test_file, test_code).unwrap();

    let binary_path = get_pmat_binary_path();

    // Test with threshold 5 (should pass - equal to threshold)
    let output_5 = std::process::Command::new(&binary_path)
        .args(&[
            "quality-gate",
            "--file",
            test_file.to_str().unwrap(),
            "--checks",
            "complexity",
            "--max-complexity-p99",
            "5",
        ])
        .output()
        .expect("Failed to run quality-gate");

    let stderr_5 = String::from_utf8_lossy(&output_5.stderr);
    assert!(
        stderr_5.contains("0 violations found"),
        "Threshold 5 should find 0 violations for complexity 5, got:\n{}",
        stderr_5
    );

    // Test with threshold 4 (should fail - below threshold)
    let output_4 = std::process::Command::new(&binary_path)
        .args(&[
            "quality-gate",
            "--file",
            test_file.to_str().unwrap(),
            "--checks",
            "complexity",
            "--max-complexity-p99",
            "4",
        ])
        .output()
        .expect("Failed to run quality-gate");

    let stderr_4 = String::from_utf8_lossy(&output_4.stderr);
    assert!(
        stderr_4.contains("1 violations found") || stderr_4.contains("1 violation found"),
        "Threshold 4 should find 1 violation for complexity 5, got:\n{}",
        stderr_4
    );
}

/// Helper function to get the pmat binary path
fn get_pmat_binary_path() -> PathBuf {
    // Get the workspace root (parent of server directory)
    let workspace_root = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let binary_path = workspace_root.join("target").join("debug").join("pmat");

    if !binary_path.exists() {
        // Build the binary if it doesn't exist from workspace root
        std::process::Command::new("cargo")
            .current_dir(&workspace_root)
            .args(&["build", "--bin", "pmat", "-p", "pmat"])
            .output()
            .expect("Failed to build pmat");
    }

    assert!(
        binary_path.exists(),
        "pmat binary not found at {:?}",
        binary_path
    );
    binary_path
}
