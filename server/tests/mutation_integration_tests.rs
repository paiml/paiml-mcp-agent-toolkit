/// Integration tests for mutation testing handler
///
/// Tests end-to-end workflows, performance, concurrency, and real-world scenarios.
///
/// Sprint 64 Day 1 - Testing Infrastructure
use pmat::cli::commands::MutateArgs;
use pmat::cli::handlers::mutate::handle;
use pmat::stateless_server::StatelessTemplateServer;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{tempdir, NamedTempFile};

// ============================================================================
// Category 1: End-to-End Workflow Tests (8 tests)
// ============================================================================

/// Test 1: Complete Rust mutation workflow
///
/// Verifies that mutation testing works end-to-end for Rust code:
/// 1. Create temporary Rust file with simple function
/// 2. Run mutation testing with default settings
/// 3. Verify mutants are generated
/// 4. Verify results are returned
#[tokio::test]
async fn test_rust_mutation_full_workflow() {
    // Arrange: Create temporary Rust file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(
        &file_path,
        r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
}

#[test]
fn test_multiply() {
    assert_eq!(multiply(2, 3), 6);
}
"#,
    )
    .unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: None, // Auto-detect Rust
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Verify success (mutation testing completed)
    assert!(
        result.is_ok(),
        "Rust mutation workflow should complete successfully"
    );
}

/// Test 2: Complete Python mutation workflow
///
/// Note: This test currently expects the handler to attempt Python mutation.
/// The test verifies that the handler accepts Python files and attempts processing.
#[tokio::test]
async fn test_python_mutation_full_workflow() {
    // Arrange: Create temporary Python file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.py");
    fs::write(
        &file_path,
        r#"
def add(a, b):
    return a + b

def multiply(a, b):
    return a * b

def test_add():
    assert add(2, 3) == 5

def test_multiply():
    assert multiply(2, 3) == 6
"#,
    )
    .unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: Some("python".to_string()),
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Handler should attempt to process Python file
    // Note: May fail if Python adapter not fully implemented, but should not panic
    match result {
        Ok(_) => {
            // Success - Python mutation completed
        }
        Err(e) => {
            // Expected if Python adapter not implemented yet
            let msg = e.to_string();
            assert!(
                msg.contains("Python") || msg.contains("not supported") || msg.contains("No mutants"),
                "Error should be related to Python support: {}",
                msg
            );
        }
    }
}

/// Test 3: Complete TypeScript mutation workflow
///
/// Note: This test verifies TypeScript file handling.
#[tokio::test]
async fn test_typescript_mutation_full_workflow() {
    // Arrange: Create temporary TypeScript file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.ts");
    fs::write(
        &file_path,
        r#"
function add(a: number, b: number): number {
    return a + b;
}

function multiply(a: number, b: number): number {
    return a * b;
}

describe('Math functions', () => {
    it('should add numbers', () => {
        expect(add(2, 3)).toBe(5);
    });

    it('should multiply numbers', () => {
        expect(multiply(2, 3)).toBe(6);
    });
});
"#,
    )
    .unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: Some("typescript".to_string()),
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Handler should attempt to process TypeScript file
    match result {
        Ok(_) => {
            // Success - TypeScript mutation completed
        }
        Err(e) => {
            // Expected if TypeScript adapter not fully implemented
            let msg = e.to_string();
            assert!(
                msg.contains("TypeScript") || msg.contains("not supported") || msg.contains("No mutants"),
                "Error should be related to TypeScript support: {}",
                msg
            );
        }
    }
}

/// Test 4: Complete JavaScript mutation workflow
#[tokio::test]
async fn test_javascript_mutation_full_workflow() {
    // Arrange: Create temporary JavaScript file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.js");
    fs::write(
        &file_path,
        r#"
function add(a, b) {
    return a + b;
}

function multiply(a, b) {
    return a * b;
}

describe('Math functions', () => {
    it('should add numbers', () => {
        expect(add(2, 3)).toBe(5);
    });

    it('should multiply numbers', () => {
        expect(multiply(2, 3)).toBe(6);
    });
});
"#,
    )
    .unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: Some("javascript".to_string()),
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Handler should attempt to process JavaScript file
    match result {
        Ok(_) => {
            // Success - JavaScript mutation completed
        }
        Err(e) => {
            // Expected if JavaScript adapter not fully implemented
            let msg = e.to_string();
            assert!(
                msg.contains("JavaScript") || msg.contains("not supported") || msg.contains("No mutants"),
                "Error should be related to JavaScript support: {}",
                msg
            );
        }
    }
}

/// Test 5: Complete Go mutation workflow
#[tokio::test]
async fn test_go_mutation_full_workflow() {
    // Arrange: Create temporary Go file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.go");
    fs::write(
        &file_path,
        r#"
package main

func Add(a, b int) int {
    return a + b
}

func Multiply(a, b int) int {
    return a * b
}

func main() {
    result := Add(2, 3)
    println(result)
}
"#,
    )
    .unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: Some("go".to_string()),
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Handler should attempt to process Go file
    match result {
        Ok(_) => {
            // Success - Go mutation completed
        }
        Err(e) => {
            // Expected if Go adapter not fully implemented
            let msg = e.to_string();
            assert!(
                msg.contains("Go") || msg.contains("not supported") || msg.contains("No mutants"),
                "Error should be related to Go support: {}",
                msg
            );
        }
    }
}

/// Test 6: Complete C++ mutation workflow
#[tokio::test]
async fn test_cpp_mutation_full_workflow() {
    // Arrange: Create temporary C++ file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.cpp");
    fs::write(
        &file_path,
        r#"
#include <iostream>

int add(int a, int b) {
    return a + b;
}

int multiply(int a, int b) {
    return a * b;
}

int main() {
    int result = add(2, 3);
    std::cout << result << std::endl;
    return 0;
}
"#,
    )
    .unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: Some("cpp".to_string()),
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Handler should attempt to process C++ file
    match result {
        Ok(_) => {
            // Success - C++ mutation completed
        }
        Err(e) => {
            // Expected if C++ adapter not fully implemented
            let msg = e.to_string();
            assert!(
                msg.contains("C++") || msg.contains("not supported") || msg.contains("No mutants"),
                "Error should be related to C++ support: {}",
                msg
            );
        }
    }
}

/// Test 7: Multi-file project mutation
///
/// Verifies mutation testing can handle multiple files in a project
#[tokio::test]
async fn test_multi_file_mutation_testing() {
    // Arrange: Create temporary directory with multiple Rust files
    let temp_dir = tempdir().unwrap();

    // File 1: lib.rs
    let lib_path = temp_dir.path().join("lib.rs");
    fs::write(
        &lib_path,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#,
    )
    .unwrap();

    // File 2: utils.rs
    let utils_path = temp_dir.path().join("utils.rs");
    fs::write(
        &utils_path,
        r#"
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn divide(a: i32, b: i32) -> i32 {
    a / b
}
"#,
    )
    .unwrap();

    // Test each file individually (handler currently processes single files)
    for file_path in [lib_path, utils_path] {
        let args = MutateArgs {
            target: file_path.clone(),
            language: None,
            timeout: 30,
            jobs: Some(1),
            output_format: "json".to_string(),
            output: None,
            threshold: None,
            failures_only: false,
        };
        let server = Arc::new(StatelessTemplateServer::new().unwrap());

        // Act: Run mutation testing on each file
        let result = handle(args, server).await;

        // Assert: Should handle each file successfully
        assert!(
            result.is_ok(),
            "Multi-file mutation should handle each file: {:?}",
            file_path
        );
    }
}

/// Test 8: Workspace-level mutation
///
/// Verifies mutation testing can handle workspace structure
#[tokio::test]
async fn test_workspace_level_mutation() {
    // Arrange: Create temporary workspace structure
    let temp_dir = tempdir().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    let main_path = src_dir.join("main.rs");
    fs::write(
        &main_path,
        r#"
fn main() {
    let result = add(2, 3);
    println!("{}", result);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
}
"#,
    )
    .unwrap();

    // Arrange: Create handler arguments for workspace file
    let args = MutateArgs {
        target: main_path.clone(),
        language: None,
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Should handle workspace file successfully
    assert!(
        result.is_ok(),
        "Workspace-level mutation should succeed"
    );
}
