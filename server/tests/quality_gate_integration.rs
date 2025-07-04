//! Integration tests for quality gate command
//!
//! Tests that quality gate properly fails when metrics exceed thresholds

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_quality_gate_fails_on_high_complexity() {
    let dir = tempdir().unwrap();
    
    // Create a file with very high complexity
    let complex_file = dir.path().join("complex.rs");
    fs::write(&complex_file, r#"
fn very_complex_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 20 {
                if x > 30 {
                    if x > 40 {
                        if x > 50 {
                            if x > 60 {
                                if x > 70 {
                                    if x > 80 {
                                        if x > 90 {
                                            return x * 2;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    x
}
    "#).unwrap();

    // Run quality gate with low complexity threshold
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("quality-gate")
        .arg("-p")
        .arg(dir.path())
        .arg("--checks")
        .arg("complexity")
        .arg("--max-complexity-p99")
        .arg("5")  // Very low threshold
        .arg("--fail-on-violation");

    // Should fail with exit code 1
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Quality gate FAILED"));
}

#[test]
fn test_quality_gate_fails_on_satd() {
    let dir = tempdir().unwrap();
    
    // Create files with SATD markers
    let file1 = dir.path().join("main.rs");
    fs::write(&file1, r#"
// TODO: This is technical debt that needs fixing
fn main() {
    // FIXME: This is a hack
    println!("Hello");
    // HACK: Quick workaround
}
    "#).unwrap();

    let file2 = dir.path().join("lib.rs");
    fs::write(&file2, r#"
// TODO: Refactor this mess
pub fn process() {
    // FIXME: Memory leak here
    // XXX: Security issue
}
    "#).unwrap();

    // Run quality gate checking for SATD
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("quality-gate")
        .arg("-p")
        .arg(dir.path())
        .arg("--checks")
        .arg("satd")
        .arg("--fail-on-violation");

    // Should fail due to SATD violations
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Quality gate FAILED"));
}

#[test]
fn test_quality_gate_fails_on_security() {
    let dir = tempdir().unwrap();
    
    // Create a file with security issues
    let file = dir.path().join("config.rs");
    fs::write(&file, r#"
const API_KEY = "sk-1234567890abcdef";
const PASSWORD = "admin123";
const SECRET = "very-secret-key";

fn connect() {
    let password = "hardcoded_password";
    // Connect with hardcoded credentials
}
    "#).unwrap();

    // Run quality gate with security check
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("quality-gate")
        .arg("-p")
        .arg(dir.path())
        .arg("--checks")
        .arg("security")
        .arg("--fail-on-violation");

    // Should fail due to security violations
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Quality gate FAILED"));
}

#[test]
fn test_quality_gate_passes_clean_code() {
    let dir = tempdir().unwrap();
    
    // Create clean code files
    let file = dir.path().join("clean.rs");
    fs::write(&file, r#"
/// A simple function
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Main entry point
fn main() {
    let result = add(2, 3);
    println!("Result: {}", result);
}
    "#).unwrap();

    // Create README with required sections
    let readme = dir.path().join("README.md");
    fs::write(&readme, r#"# Test Project

## Installation
Install via cargo.

## Usage
Run the main function.

## Contributing
Pull requests welcome.

## License
MIT License.
    "#).unwrap();

    // Run quality gate with specific checks that should pass
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("quality-gate")
        .arg("-p")
        .arg(dir.path())
        .arg("--checks")
        .arg("complexity,satd,security,dead-code")
        .arg("--format")
        .arg("human");

    // Should pass
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Quality gate PASSED"));
}

#[test]
fn test_quality_gate_json_output() {
    let dir = tempdir().unwrap();
    
    // Create a file with issues
    let file = dir.path().join("issues.rs");
    fs::write(&file, r#"
// TODO: Fix this
fn complex() {
    if true {
        if false {
            // Nested complexity
        }
    }
}
    "#).unwrap();

    // Run quality gate with JSON output
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("quality-gate")
        .arg("-p")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .arg("--checks")
        .arg("all");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"passed\":"))
        .stdout(predicate::str::contains("\"violations\":"));
}

#[test]
fn test_quality_gate_ci_integration() {
    let dir = tempdir().unwrap();
    
    // Create multiple files with different issues
    fs::write(dir.path().join("complex.rs"), r#"
fn nested() {
    if true { if true { if true { if true { if true {
        // Too deeply nested
    }}}}}
}
    "#).unwrap();
    
    fs::write(dir.path().join("debt.rs"), r#"
// FIXME: Critical bug here
// TODO: Needs refactoring
fn debt() {}
    "#).unwrap();

    // Run with human format and fail-on-violation
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("quality-gate")
        .arg("-p")
        .arg(dir.path())
        .arg("--format")
        .arg("human")
        .arg("--checks")
        .arg("complexity,satd")
        .arg("--max-complexity-p99")
        .arg("3")
        .arg("--fail-on-violation");

    // Should fail and show human-readable output
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("# Quality Gate Report"))
        .stdout(predicate::str::contains("Status: ❌ FAILED"))
        .stdout(predicate::str::contains("## Violations:"));
}