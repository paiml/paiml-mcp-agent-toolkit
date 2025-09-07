//! Test for quality-gate complexity counting
//! 
//! This test ensures that quality-gate only counts functions that ACTUALLY
//! exceed the complexity threshold, not all violations/warnings/info messages.
//!
//! TDD: Test written BEFORE implementation fix

use std::fs;
use tempfile::TempDir;

/// Test that quality-gate only counts functions exceeding threshold
#[test]
fn test_quality_gate_counts_only_threshold_violations() {
    // Create a test file with known complexity
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    // Write test code with:
    // - One function with complexity 5 (below threshold of 10)
    // - One function with complexity 8 (below threshold of 10)  
    // - One function with complexity 12 (above threshold of 10)
    let test_code = r#"
// Function with complexity 5 (below threshold)
fn simple_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x + 1
        }
    } else {
        0
    }
}

// Function with complexity 8 (below threshold)
fn medium_function(x: i32, y: i32) -> i32 {
    let mut result = 0;
    if x > 0 {
        if y > 0 {
            result = x + y;
        } else if y < -10 {
            result = x - y;
        } else {
            result = x * 2;
        }
    } else if x < -10 {
        if y > 0 {
            result = -x + y;
        } else {
            result = -x - y;
        }
    } else {
        result = y;
    }
    result
}

// Function with complexity 12 (above threshold of 10)
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

    // Run quality-gate with complexity threshold of 10
    // First ensure the binary exists
    let binary_path = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("pmat");
    
    if !binary_path.exists() {
        // Build the binary if it doesn't exist
        std::process::Command::new("cargo")
            .args(&["build", "--bin", "pmat"])
            .output()
            .expect("Failed to build pmat");
    }
    
    let output = std::process::Command::new(binary_path)
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

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Quality gate output:\n{}", stderr);

    // CRITICAL ASSERTION: Should find exactly 1 violation (the function with complexity 12)
    // NOT 3 violations (all functions)
    assert!(
        stderr.contains("1 violations found") || stderr.contains("1 violation found"),
        "Expected exactly 1 complexity violation for function exceeding threshold, but got:\n{}",
        stderr
    );

    // Should NOT report 3 violations
    assert!(
        !stderr.contains("3 violations found"),
        "Quality gate incorrectly counted all functions as violations instead of only those exceeding threshold"
    );
}

/// Test that quality-gate correctly identifies zero violations when all functions are below threshold
#[test]
fn test_quality_gate_zero_violations_when_all_below_threshold() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.rs");
    
    // Write test code with all functions below complexity 10
    let test_code = r#"
fn simple1(x: i32) -> i32 {
    if x > 0 { x } else { 0 }
}

fn simple2(x: i32, y: i32) -> i32 {
    if x > y { x } else { y }
}

fn simple3(x: bool) -> i32 {
    if x { 1 } else { 0 }
}
"#;

    fs::write(&test_file, test_code).unwrap();

    // Get binary path
    let binary_path = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("pmat");
    
    let output = std::process::Command::new(binary_path)
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

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should find 0 violations
    assert!(
        stderr.contains("0 violations found") || stderr.contains("Quality Gate: PASSED"),
        "Expected 0 complexity violations when all functions are below threshold, but got:\n{}",
        stderr
    );
}