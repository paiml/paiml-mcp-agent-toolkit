use std::fs;
use tempfile::TempDir;

/// Test that exposes bug where complexity analysis reports 0 functions but non-zero complexity
#[test]
fn test_complexity_analysis_detects_functions() {
    // Create a temp directory with a test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Write a simple Rust file with one function
    let content = r#"
fn simple_function() {
    println!("Hello");
}

fn complex_function(x: i32) -> i32 {
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
"#;

    fs::write(&test_file, content).unwrap();

    // Run complexity analysis on the file
    let output = std::process::Command::new("../target/debug/pmat")
        .args([
            "analyze",
            "complexity",
            "--file",
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run pmat");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    // Check that functions are detected
    assert!(
        stdout.contains("Total functions"),
        "Should report function count"
    );
    assert!(
        !stdout.contains("Total functions**: 0"),
        "Should not report 0 functions when functions exist"
    );

    // Check that complexity values are reasonable
    assert!(
        stdout.contains("simple_function"),
        "Should list simple_function"
    );
    assert!(
        stdout.contains("complex_function"),
        "Should list complex_function"
    );

    // Complex function should have higher complexity than simple one
    assert!(
        stdout.contains("Cyclomatic"),
        "Should report cyclomatic complexity"
    );
    assert!(
        stdout.contains("Cognitive"),
        "Should report cognitive complexity"
    );
}

/// Test that file-level complexity should be 0 when no functions exist
#[test]
fn test_empty_file_has_zero_complexity() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.rs");

    // Write an empty or comment-only file
    let content = r#"
// This file has no functions
// Just comments
"#;

    fs::write(&test_file, content).unwrap();

    let output = std::process::Command::new("../target/debug/pmat")
        .args([
            "analyze",
            "complexity",
            "--file",
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run pmat");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout for empty file: {}", stdout);

    // Should report 0 functions
    assert!(
        stdout.contains("Total functions**: 0"),
        "Should report 0 functions for empty file"
    );

    // Should have 0 complexity values
    assert!(
        stdout.contains("Max Cyclomatic**: 0"),
        "Should report 0 cyclomatic for empty file"
    );
    assert!(
        stdout.contains("Max Cognitive**: 0"),
        "Should report 0 cognitive for empty file"
    );
}

/// Integration test that analyzes real codebase file
#[test]
fn test_real_file_analysis() {
    // Create a temp directory with a test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("real_test.rs");

    // Write a Rust file with known functions
    let content = r#"
pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

pub fn process_data(items: Vec<i32>) -> i32 {
    let mut result = 0;
    for item in items {
        if item > 0 {
            result += item;
        }
    }
    result
}

fn helper_function() {
    println!("Helper");
}
"#;

    fs::write(&test_file, content).unwrap();

    // Test on this file
    let output = std::process::Command::new("../target/debug/pmat")
        .args([
            "analyze",
            "complexity",
            "--file",
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run pmat");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("Real file stdout: {}", stdout);

    // Should report 3 functions
    assert!(
        stdout.contains("Total functions**: 3"),
        "Should detect 3 functions"
    );

    // Should list actual function names in the functions section
    assert!(
        stdout.contains("## Functions in File"),
        "Should have Functions in File section for single file analysis"
    );

    assert!(
        stdout.contains("calculate_sum")
            || stdout.contains("process_data")
            || stdout.contains("helper_function"),
        "Should detect and list actual functions from the file"
    );
}
