//! Integration tests for --include pattern filtering

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_complexity_include_patterns() {
    // Create a temporary project structure
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create some test files
    fs::create_dir_all(project_path.join("src")).unwrap();
    fs::create_dir_all(project_path.join("tests")).unwrap();
    fs::create_dir_all(project_path.join("examples")).unwrap();
    
    // Create Rust files with varying complexity
    fs::write(
        project_path.join("src/main.rs"),
        r#"
fn main() {
    println!("Hello, world!");
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
"#,
    ).unwrap();
    
    fs::write(
        project_path.join("src/lib.rs"),
        r#"
pub fn simple_function() -> &'static str {
    "simple"
}
"#,
    ).unwrap();
    
    fs::write(
        project_path.join("tests/test.rs"),
        r#"
#[test]
fn test_something() {
    assert_eq!(1 + 1, 2);
}
"#,
    ).unwrap();
    
    fs::write(
        project_path.join("examples/example.rs"),
        r#"
fn main() {
    println!("Example");
}
"#,
    ).unwrap();
    
    // Test 1: Include only src files
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--include")
        .arg("src/**/*.rs")
        .arg("--format")
        .arg("json");
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("tests/test.rs").not())
        .stdout(predicate::str::contains("examples/example.rs").not());
    
    // Test 2: Include only test files
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--include")
        .arg("tests/**/*.rs")
        .arg("--format")
        .arg("json");
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tests/test.rs"))
        .stdout(predicate::str::contains("src/main.rs").not());
    
    // Test 3: Multiple include patterns
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--include")
        .arg("src/**/*.rs")
        .arg("--include")
        .arg("examples/**/*.rs")
        .arg("--format")
        .arg("json");
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("examples/example.rs"))
        .stdout(predicate::str::contains("tests/test.rs").not());
}

#[test]
fn test_complexity_single_file_mode() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create a test file
    let file_path = project_path.join("test.rs");
    fs::write(
        &file_path,
        r#"
fn complex_function(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            x + y
        } else {
            x - y
        }
    } else if y > 0 {
        y - x
    } else {
        0
    }
}
"#,
    ).unwrap();
    
    // Test single file analysis
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze")
        .arg("complexity")
        .arg("-p")
        .arg(project_path.to_str().unwrap())
        .arg("--file")
        .arg(file_path.to_str().unwrap())
        .arg("--format")
        .arg("json");
        
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("complex_function"));
}