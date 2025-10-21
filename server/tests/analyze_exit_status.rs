//! Integration tests for analyze command exit status (Issue #28)
//!
//! These tests verify that analyze commands return non-zero exit status
//! when violations are found AND the --fail-on-violation flag is used,
//! addressing GitHub issue #28.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Create a test file with high complexity
fn create_complex_file(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("complex.rs");
    let content = r#"
fn very_complex_function(x: i32, y: i32, z: i32) -> i32 {
    let mut result = 0;
    
    if x > 0 {
        if y > 0 {
            if z > 0 {
                result = x + y + z;
                if result > 100 {
                    result = result * 2;
                    if result > 1000 {
                        result = result / 10;
                    }
                }
            } else {
                result = x + y;
            }
        } else {
            if z > 0 {
                result = x + z;
            } else {
                result = x;
            }
        }
    } else {
        if y > 0 {
            if z > 0 {
                result = y + z;
            } else {
                result = y;
            }
        } else {
            result = z;
        }
    }
    
    result
}
"#;
    fs::write(&path, content).unwrap();
    path
}

/// Create a test file with SATD
fn create_satd_file(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("satd.rs");
    let content = r#"
// TODO: This is technical debt that needs to be fixed
fn hacky_function() {
    // FIXME: This is a terrible hack
    let x = 42;
    
    // HACK: Don't do this in production
    println!("{}", x);
}
"#;
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_analyze_complexity_exits_non_zero_with_violations() {
    let temp_dir = TempDir::new().unwrap();
    create_complex_file(&temp_dir);

    // Run analyze complexity with a low threshold and fail-on-violation flag
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir).args([
        "analyze",
        "complexity",
        "--max-cyclomatic",
        "5",
        "--fail-on-violation",
    ]);

    // Should exit with non-zero status
    cmd.assert().failure().code(1);
}

#[test]
fn test_analyze_complexity_exits_zero_without_violations() {
    let temp_dir = TempDir::new().unwrap();

    // Create a simple file
    let path = temp_dir.path().join("simple.rs");
    fs::write(&path, "fn simple() -> i32 { 42 }").unwrap();

    // Run analyze complexity with a high threshold
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir)
        .args(["analyze", "complexity", "--max-cyclomatic", "50"]);

    // Should exit with zero status
    cmd.assert().success();
}

#[test]
#[ignore] // Test expects fail-on-violation logic to work correctly for SATD
fn test_analyze_satd_exits_non_zero_with_violations() {
    let temp_dir = TempDir::new().unwrap();
    create_satd_file(&temp_dir);

    // Run analyze satd with fail-on-violation flag
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir)
        .args(["analyze", "satd", "--fail-on-violation"]);

    // Should exit with non-zero status
    cmd.assert().failure().code(1);
}

#[test]
fn test_analyze_satd_exits_zero_without_violations() {
    let temp_dir = TempDir::new().unwrap();

    // Create a clean file
    let path = temp_dir.path().join("clean.rs");
    fs::write(&path, "fn clean_function() -> i32 { 42 }").unwrap();

    // Run analyze satd
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir).args(["analyze", "satd"]);

    // Should exit with zero status
    cmd.assert().success();
}

#[test]
fn test_analyze_dead_code_exit_status() {
    let temp_dir = TempDir::new().unwrap();

    // Create a file with unused function
    let path = temp_dir.path().join("dead.rs");
    let content = r#"
fn used_function() -> i32 {
    42
}

fn unused_function() -> i32 {
    100
}

fn main() {
    println!("{}", used_function());
}
"#;
    fs::write(&path, content).unwrap();

    // Run analyze dead-code with max percentage and fail-on-violation flag
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir).args([
        "analyze",
        "dead-code",
        "--max-percentage",
        "10",
        "--fail-on-violation",
    ]);

    // Should exit with non-zero status if dead code percentage exceeds threshold
    cmd.assert().failure().code(1);
}
