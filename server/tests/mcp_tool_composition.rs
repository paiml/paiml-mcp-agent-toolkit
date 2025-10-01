//! Tests for MCP-native tool composition
//!
//! These tests verify that commands can work together by passing file lists
//! directly between tools, enabling MCP client composition.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

#[test]
#[ignore = "MCP tool composition: --files parameter not yet implemented"]
fn test_complexity_files_parameter() {
    // Create a temporary project structure
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create some test files
    fs::create_dir_all(project_path.join("src")).unwrap();

    // Create Rust files with varying complexity
    fs::write(
        project_path.join("src/simple.rs"),
        r#"
pub fn simple_function() -> &'static str {
    "simple"
}
"#,
    )
    .unwrap();

    fs::write(
        project_path.join("src/complex.rs"),
        r#"
pub fn complex_function(x: i32, y: i32, z: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                x + y + z
            } else {
                x + y - z
            }
        } else {
            if z > 0 {
                x - y + z
            } else {
                x - y - z
            }
        }
    } else if y > 0 {
        if z > 0 {
            -x + y + z
        } else {
            -x + y - z
        }
    } else {
        if z > 0 {
            -x - y + z
        } else {
            -x - y - z
        }
    }
}
"#,
    )
    .unwrap();

    // Test complexity analysis with specific files (MCP tool composition)
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--files")
        .arg("src/simple.rs,src/complex.rs")
        .arg("--format")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/simple.rs"))
        .stdout(predicate::str::contains("src/complex.rs"));
}

#[test]
#[ignore = "MCP tool composition: --files parameter not yet implemented"]
fn test_mcp_composition_workflow() {
    // This test simulates how an MCP client would compose tools:
    // 1. Get top complex files
    // 2. Pass those files to comprehensive analysis

    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create some test files with known complexity patterns
    fs::create_dir_all(project_path.join("src")).unwrap();

    // Simple file (low complexity)
    fs::write(
        project_path.join("src/simple.rs"),
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    // Complex file (high complexity)
    fs::write(
        project_path.join("src/complex.rs"),
        r#"
pub fn complex_logic(a: i32, b: i32, c: i32, d: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    a + b + c + d
                } else {
                    a + b + c - d
                }
            } else {
                if d > 0 {
                    a + b - c + d
                } else {
                    a + b - c - d
                }
            }
        } else {
            if c > 0 {
                if d > 0 {
                    a - b + c + d
                } else {
                    a - b + c - d
                }
            } else {
                if d > 0 {
                    a - b - c + d
                } else {
                    a - b - c - d
                }
            }
        }
    } else {
        // Similar nested structure for a <= 0
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    -a + b + c + d
                } else {
                    -a + b + c - d
                }
            } else {
                if d > 0 {
                    -a + b - c + d
                } else {
                    -a + b - c - d
                }
            }
        } else {
            if c > 0 {
                if d > 0 {
                    -a - b + c + d
                } else {
                    -a - b + c - d
                }
            } else {
                if d > 0 {
                    -a - b - c + d
                } else {
                    -a - b - c - d
                }
            }
        }
    }
}
"#,
    )
    .unwrap();

    // Step 1: Get complexity analysis (simulating first MCP tool call)
    let mut cmd1 = Command::cargo_bin("pmat").unwrap();
    let output1 = cmd1
        .arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--format")
        .arg("json")
        .arg("--top-files")
        .arg("1")
        .output()
        .unwrap();

    assert!(output1.status.success());

    // Parse JSON output to extract file paths (simulating MCP client logic)
    let json_output = String::from_utf8(output1.stdout).unwrap();
    let complexity_result: Value = serde_json::from_str(&json_output).unwrap();

    // Extract the most complex file
    let files = complexity_result["files"].as_array().unwrap();
    assert!(!files.is_empty());

    let most_complex_file = &files[0]["path"].as_str().unwrap();

    // Step 2: Analyze just that file with the files parameter (simulating second MCP tool call)
    let mut cmd2 = Command::cargo_bin("pmat").unwrap();
    cmd2.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--files")
        .arg(most_complex_file)
        .arg("--format")
        .arg("json");

    cmd2.assert()
        .success()
        .stdout(predicate::str::contains(*most_complex_file))
        .stdout(predicate::str::contains("complex.rs"));
}

#[test]
#[ignore = "MCP tool composition: --files parameter not yet implemented"]
fn test_empty_files_parameter() {
    // Test that providing empty files list falls back to project mode
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create a simple test file
    fs::create_dir_all(project_path.join("src")).unwrap();
    fs::write(
        project_path.join("src/main.rs"),
        r#"
fn main() {
    println!("Hello, world!");
}
"#,
    )
    .unwrap();

    // Test without files parameter (should analyze the whole project)
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--format")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
#[ignore = "MCP tool composition: --files parameter not yet implemented"]
fn test_files_parameter_conflicts() {
    // Test that --files conflicts with --file and --include as expected
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Test conflict with --file
    let mut cmd1 = Command::cargo_bin("pmat").unwrap();
    cmd1.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--file")
        .arg("src/main.rs")
        .arg("--files")
        .arg("src/lib.rs");

    cmd1.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));

    // Test conflict with --include
    let mut cmd2 = Command::cargo_bin("pmat").unwrap();
    cmd2.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--include")
        .arg("**/*.rs")
        .arg("--files")
        .arg("src/lib.rs");

    cmd2.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}
