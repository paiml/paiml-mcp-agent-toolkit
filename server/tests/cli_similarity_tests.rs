//! CLI integration tests for similarity detection
//!
//! Tests the analyze duplicates command with entropy detection

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cli_analyze_duplicates_exact() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test files with exact duplicates
    let file1 = temp_dir.path().join("file1.rs");
    fs::write(&file1, r#"
fn process_data(x: i32) -> i32 {
    let result = x * 2;
    println!("Result: {}", result);
    result
}

fn another_function() {
    println!("Hello");
}
"#).unwrap();
    
    let file2 = temp_dir.path().join("file2.rs");
    fs::write(&file2, r#"
fn process_data(x: i32) -> i32 {
    let result = x * 2;
    println!("Result: {}", result);
    result
}

fn different_function() {
    println!("World");
}
"#).unwrap();
    
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("duplicates")
        .arg("--project-path")
        .arg(temp_dir.path())
        .arg("--detection-type")
        .arg("exact")
        .arg("--min-lines")
        .arg("3")
        .arg("--format")
        .arg("summary");
    
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Advanced similarity analysis"))
        .stdout(predicate::str::contains("Code Similarity Analysis Summary"));
}

#[test]
fn test_cli_analyze_duplicates_fuzzy() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test files with renamed variables
    let file1 = temp_dir.path().join("module1.rs");
    fs::write(&file1, r#"
fn calculate(a: i32, b: i32) -> i32 {
    let sum = a + b;
    sum * 2
}
"#).unwrap();
    
    let file2 = temp_dir.path().join("module2.rs");
    fs::write(&file2, r#"
fn calculate(x: i32, y: i32) -> i32 {
    let total = x + y;
    total * 2
}
"#).unwrap();
    
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("duplicates")
        .arg("--project-path")
        .arg(temp_dir.path())
        .arg("--detection-type")
        .arg("fuzzy")
        .arg("--threshold")
        .arg("0.7")
        .arg("--min-lines")
        .arg("3");
    
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Analysis Complete"));
}

#[test]
fn test_cli_analyze_duplicates_semantic() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create semantically similar code
    let file1 = temp_dir.path().join("impl1.rs");
    fs::write(&file1, r#"
fn sum_array(arr: &[i32]) -> i32 {
    let mut total = 0;
    for val in arr {
        total += val;
    }
    total
}
"#).unwrap();
    
    let file2 = temp_dir.path().join("impl2.rs");
    fs::write(&file2, r#"
fn sum_array(arr: &[i32]) -> i32 {
    arr.iter().sum()
}
"#).unwrap();
    
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("duplicates")
        .arg("--project-path")
        .arg(temp_dir.path())
        .arg("--detection-type")
        .arg("semantic")
        .arg("--threshold")
        .arg("0.6");
    
    cmd.assert()
        .success();
}

#[test]
fn test_cli_analyze_duplicates_all_types() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create various types of duplicates
    let file1 = temp_dir.path().join("mixed1.rs");
    fs::write(&file1, r#"
// Exact duplicate
fn exact_dup() {
    println!("exact");
}

// Renamed variables
fn process(x: i32) -> i32 {
    x * 2
}

// Different implementation
fn find_max(nums: &[i32]) -> Option<i32> {
    if nums.is_empty() {
        return None;
    }
    let mut max = nums[0];
    for &n in &nums[1..] {
        if n > max {
            max = n;
        }
    }
    Some(max)
}
"#).unwrap();
    
    let file2 = temp_dir.path().join("mixed2.rs");
    fs::write(&file2, r#"
// Exact duplicate
fn exact_dup() {
    println!("exact");
}

// Renamed variables
fn process(val: i32) -> i32 {
    val * 2
}

// Different implementation
fn find_max(nums: &[i32]) -> Option<i32> {
    nums.iter().max().copied()
}
"#).unwrap();
    
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("duplicates")
        .arg("--project-path")
        .arg(temp_dir.path())
        .arg("--detection-type")
        .arg("all")
        .arg("--format")
        .arg("json");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"exact_duplicates\""))
        .stdout(predicate::str::contains("\"structural_similarities\""))
        .stdout(predicate::str::contains("\"entropy_analysis\""));
}

#[test]
fn test_cli_analyze_duplicates_with_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("report.json");
    
    // Create test file
    let file = temp_dir.path().join("test.rs");
    fs::write(&file, r#"
fn test1() { println!("test"); }
fn test2() { println!("test"); }
fn test3() { println!("different"); }
"#).unwrap();
    
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("duplicates")
        .arg("--project-path")
        .arg(temp_dir.path())
        .arg("--detection-type")
        .arg("all")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_file);
    
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Report written to"));
    
    // Verify output file was created
    assert!(output_file.exists());
    let content = fs::read_to_string(output_file).unwrap();
    assert!(content.contains("metrics"));
}

#[test]
fn test_cli_analyze_duplicates_csv_format() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test files
    let file1 = temp_dir.path().join("a.rs");
    fs::write(&file1, "fn dup() { println!(\"x\"); }\n").unwrap();
    
    let file2 = temp_dir.path().join("b.rs");
    fs::write(&file2, "fn dup() { println!(\"x\"); }\n").unwrap();
    
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("duplicates")
        .arg("--project-path")
        .arg(temp_dir.path())
        .arg("--format")
        .arg("csv");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Type,File1,Start1,End1,File2,Start2,End2"));
}

#[test]
fn test_cli_analyze_duplicates_sarif_format() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test file
    let file = temp_dir.path().join("test.rs");
    fs::write(&file, "fn x() { let a = 1; }\nfn y() { let a = 1; }\n").unwrap();
    
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("duplicates")
        .arg("--project-path")
        .arg(temp_dir.path())
        .arg("--format")
        .arg("sarif");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"$schema\""))
        .stdout(predicate::str::contains("sarif-2.1.0"));
}

#[test]
fn test_cli_analyze_duplicates_performance_metrics() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test file
    let file = temp_dir.path().join("perf.rs");
    fs::write(&file, "fn test() { println!(\"test\"); }\n").unwrap();
    
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("duplicates")
        .arg("--project-path")
        .arg(temp_dir.path())
        .arg("--perf");
    
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Performance Metrics"));
}