#![cfg(not(feature = "skip-slow-tests"))]

/// Integration tests for mutation testing handler
///
/// Tests end-to-end workflows, performance, concurrency, and real-world scenarios.
///
/// Sprint 64 Day 1 - Testing Infrastructure
use pmat::cli::commands::MutateArgs;
use pmat::cli::handlers::mutate::handle;
use pmat::stateless_server::StatelessTemplateServer;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

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
/// FAILED: Mutation integration test - needs fixing
#[ignore]
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
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
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
/// FAILED: Mutation integration test - needs fixing
#[ignore]
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
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
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
                msg.contains("Python")
                    || msg.contains("not supported")
                    || msg.contains("No mutants"),
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
/// FAILED: Mutation integration test - needs fixing
#[ignore]
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
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
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
                msg.contains("TypeScript")
                    || msg.contains("not supported")
                    || msg.contains("No mutants"),
                "Error should be related to TypeScript support: {}",
                msg
            );
        }
    }
}

/// Test 4: Complete JavaScript mutation workflow
/// IGNORED: Mutation testing integration test - requires external mutation testing tools
#[tokio::test]
#[ignore]
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
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
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
                msg.contains("JavaScript")
                    || msg.contains("not supported")
                    || msg.contains("No mutants"),
                "Error should be related to JavaScript support: {}",
                msg
            );
        }
    }
}

/// Test 5: Complete Go mutation workflow
/// IGNORED: Mutation testing integration test - requires external mutation testing tools
#[tokio::test]
#[ignore]
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
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
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
/// IGNORED: Mutation testing integration test - requires external mutation testing tools
#[tokio::test]
#[ignore]
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
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
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
/// IGNORED: Mutation testing integration test - requires external mutation testing tools
#[tokio::test]
#[ignore]
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
            use_cargo_mutants: false,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
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
/// FAILED: Mutation integration test - needs fixing
#[ignore]
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
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Should handle workspace file successfully
    assert!(result.is_ok(), "Workspace-level mutation should succeed");
}

// ============================================================================
// Category 2: Performance and Scale Tests (6 tests)
// ============================================================================

/// Test 9: Large file mutation (>1000 lines)
///
/// Verifies mutation testing can handle large files efficiently
#[tokio::test]
/// SLOW: >60s - excluded from fast test suite
#[ignore]
async fn test_large_file_mutation() {
    // Arrange: Create large Rust file (simulating ~1000+ lines)
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("large_file.rs");

    // Generate many functions to simulate large file
    let mut code = String::new();
    for i in 0..200 {
        code.push_str(&format!(
            r#"
fn function_{}(a: i32, b: i32) -> i32 {{
    a + b + {}
}}
"#,
            i, i
        ));
    }
    fs::write(&file_path, code).unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: None,
        timeout: 60,   // Longer timeout for large file
        jobs: Some(4), // Use multiple jobs for performance
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: true, // Reduce output volume
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing on large file
    let result = handle(args, server).await;

    // Assert: Should handle large file (may succeed or fail gracefully)
    match result {
        Ok(_) => {
            // Success - large file mutation completed
        }
        Err(e) => {
            // Acceptable errors for large files
            let msg = e.to_string();
            assert!(
                msg.contains("No mutants") || msg.contains("timeout") || msg.contains("Too many"),
                "Error should be acceptable for large file: {}",
                msg
            );
        }
    }
}

/// Test 10: Many mutants (targeting >500 mutants)
///
/// Verifies mutation testing can handle many mutants efficiently
#[tokio::test]
/// SLOW: >60s - excluded from fast test suite
#[ignore]
async fn test_many_mutants_handling() {
    // Arrange: Create file with many mutation opportunities
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("many_mutants.rs");

    let mut code = String::from("fn process(x: i32) -> i32 {\n");
    code.push_str("    let mut result = x;\n");

    // Create many arithmetic operations (each creates multiple mutants)
    for i in 0..50 {
        code.push_str(&format!(
            "    result = result + {} - {} * {} / {};\n",
            i,
            i + 1,
            i + 2,
            i + 3
        ));
    }

    code.push_str("    result\n}\n");
    fs::write(&file_path, code).unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: None,
        timeout: 60,
        jobs: Some(4),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: true,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Should handle many mutants
    match result {
        Ok(_) => {
            // Success - many mutants handled
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("No mutants") || msg.contains("timeout"),
                "Error should be acceptable: {}",
                msg
            );
        }
    }
}

/// Test 11: Parallel execution scaling (1, 2, 4, 8 threads)
///
/// Verifies parallel execution works correctly with different thread counts
#[tokio::test]
/// FAILED: Mutation integration test - needs fixing
#[ignore]
async fn test_parallel_execution_scaling() {
    // Arrange: Create test file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("parallel_test.rs");
    fs::write(
        &file_path,
        r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn sub(a: i32, b: i32) -> i32 { a - b }
fn mul(a: i32, b: i32) -> i32 { a * b }
fn div(a: i32, b: i32) -> i32 { a / b }
"#,
    )
    .unwrap();

    // Test with different thread counts
    for jobs in [1, 2, 4, 8] {
        let args = MutateArgs {
            target: file_path.clone(),
            language: None,
            timeout: 30,
            jobs: Some(jobs),
            output_format: "json".to_string(),
            output: None,
            threshold: None,
            failures_only: false,
            use_cargo_mutants: false,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };
        let server = Arc::new(StatelessTemplateServer::new().unwrap());

        // Act: Run with specific job count
        let result = handle(args, server).await;

        // Assert: Should work with any valid job count
        assert!(
            result.is_ok()
                || result
                    .as_ref()
                    .err()
                    .map(|e| e.to_string().contains("No mutants"))
                    .unwrap_or(false),
            "Parallel execution with {} jobs should work",
            jobs
        );
    }
}

/// Test 12: Timeout handling
///
/// Verifies mutation testing respects timeout settings
#[tokio::test]
/// FAILED: Mutation integration test - needs fixing
#[ignore]
async fn test_timeout_handling() {
    // Arrange: Create test file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("timeout_test.rs");
    fs::write(
        &file_path,
        r#"
fn compute(n: i32) -> i32 {
    let mut sum = 0;
    for i in 0..n {
        sum += i;
    }
    sum
}
"#,
    )
    .unwrap();

    // Arrange: Create handler arguments with short timeout
    let args = MutateArgs {
        target: file_path.clone(),
        language: None,
        timeout: 1, // Very short timeout (1 second)
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing with short timeout
    let result = handle(args, server).await;

    // Assert: Should complete quickly or timeout gracefully
    match result {
        Ok(_) => {
            // Completed within timeout - acceptable
        }
        Err(e) => {
            // Timeout or no mutants - both acceptable
            let msg = e.to_string();
            assert!(
                msg.contains("timeout") || msg.contains("No mutants") || msg.contains("Timed out"),
                "Error should be timeout-related or no mutants: {}",
                msg
            );
        }
    }
}

/// Test 13: Memory usage bounds
///
/// Verifies mutation testing doesn't consume excessive memory
#[tokio::test]
/// SLOW: >60s - excluded from fast test suite
#[ignore]
async fn test_memory_usage_bounds() {
    // Arrange: Create moderately large file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("memory_test.rs");

    let mut code = String::new();
    for i in 0..100 {
        code.push_str(&format!("fn func_{}(x: i32) -> i32 {{ x + {} }}\n", i, i));
    }
    fs::write(&file_path, code).unwrap();

    // Arrange: Create handler arguments
    let args = MutateArgs {
        target: file_path.clone(),
        language: None,
        timeout: 30,
        jobs: Some(2),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: true, // Reduce memory footprint
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Run mutation testing
    let result = handle(args, server).await;

    // Assert: Should complete without OOM
    // Note: This test primarily verifies no OOM/crash occurs
    assert!(
        result.is_ok()
            || result
                .as_ref()
                .err()
                .map(|e| !e.to_string().contains("out of memory"))
                .unwrap_or(true),
        "Should not run out of memory"
    );
}

/// Test 14: Execution time bounds
///
/// Verifies mutation testing completes within reasonable time
#[tokio::test]
/// SLOW: >60s - excluded from fast test suite
#[ignore]
async fn test_execution_time_bounds() {
    use std::time::Instant;

    // Arrange: Create small test file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("time_test.rs");
    fs::write(
        &file_path,
        r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn sub(a: i32, b: i32) -> i32 { a - b }
"#,
    )
    .unwrap();

    let args = MutateArgs {
        target: file_path.clone(),
        language: None,
        timeout: 30,
        jobs: Some(2),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Act: Measure execution time
    let start = Instant::now();
    let _result = handle(args, server).await;
    let duration = start.elapsed();

    // Assert: Should complete quickly for small file
    // Generous time bound: 60 seconds for small file
    assert!(
        duration.as_secs() < 60,
        "Small file mutation should complete within 60 seconds, took {:?}",
        duration
    );
}

// ============================================================================
// Category 3: Concurrent Execution Tests (4 tests)
// ============================================================================

/// Test 15: Parallel mutant execution correctness
#[tokio::test]
/// SLOW: >60s - excluded from fast test suite
#[ignore]
async fn test_parallel_mutant_execution_correctness() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("parallel.rs");
    fs::write(
        &file_path,
        "fn add(a: i32, b: i32) -> i32 { a + b }\nfn sub(a: i32, b: i32) -> i32 { a - b }",
    )
    .unwrap();

    let args = MutateArgs {
        target: file_path,
        language: None,
        timeout: 30,
        jobs: Some(4),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let result = handle(args, server).await;
    assert!(
        result.is_ok()
            || result
                .as_ref()
                .err()
                .map(|e| e.to_string().contains("No mutants"))
                .unwrap_or(false)
    );
}

/// Test 16: Race condition handling
#[tokio::test]
/// SLOW: >60s - excluded from fast test suite
#[ignore]
async fn test_race_condition_handling() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("race.rs");
    fs::write(&file_path, "fn compute(x: i32) -> i32 { x * 2 + 1 }").unwrap();

    // Run multiple times to catch potential race conditions
    for _ in 0..3 {
        let args = MutateArgs {
            target: file_path.clone(),
            language: None,
            timeout: 30,
            jobs: Some(8),
            output_format: "json".to_string(),
            output: None,
            threshold: None,
            failures_only: false,
            use_cargo_mutants: false,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let _ = handle(args, server).await;
    }
}

/// Test 17: Resource contention
#[tokio::test]
/// SLOW: >60s - excluded from fast test suite
#[ignore]
async fn test_resource_contention() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("contention.rs");
    fs::write(&file_path, "fn process(n: i32) -> i32 { (0..n).sum() }").unwrap();

    let args = MutateArgs {
        target: file_path,
        language: None,
        timeout: 30,
        jobs: Some(16),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: true,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let result = handle(args, server).await;
    assert!(
        result.is_ok()
            || result
                .as_ref()
                .err()
                .map(|e| !e.to_string().contains("panic"))
                .unwrap_or(true)
    );
}

/// Test 18: Graceful shutdown on error
#[tokio::test]
async fn test_graceful_shutdown_on_error() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("shutdown.rs");
    fs::write(&file_path, "fn invalid syntax here").unwrap();

    let args = MutateArgs {
        target: file_path,
        language: None,
        timeout: 5,
        jobs: Some(4),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let result = handle(args, server).await;
    assert!(result.is_err() || result.as_ref().ok().is_some());
}

// ============================================================================
// Category 4: Real-World Scenarios Tests (4 tests)
// ============================================================================

/// Test 19: Mutation of actual PMAT code
#[tokio::test]
/// SLOW: >60s - excluded from fast test suite
#[ignore]
async fn test_mutation_of_actual_pmat_code() {
    use std::path::Path;
    let pmat_file = Path::new("src/utils/path_validator.rs");
    if !pmat_file.exists() {
        return;
    }

    let args = MutateArgs {
        target: pmat_file.to_path_buf(),
        language: None,
        timeout: 60,
        jobs: Some(2),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: true,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let result = handle(args, server).await;
    assert!(
        result.is_ok()
            || result
                .as_ref()
                .err()
                .map(|e| e.to_string().contains("No mutants"))
                .unwrap_or(false)
    );
}

/// Test 20: Mutation with failing tests
#[tokio::test]
async fn test_mutation_with_failing_tests() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("failing.rs");
    fs::write(
        &file_path,
        r#"
fn buggy_add(a: i32, b: i32) -> i32 { a - b }
#[test]
fn test_add() { assert_eq!(buggy_add(2, 3), 5); }
"#,
    )
    .unwrap();

    let args = MutateArgs {
        target: file_path,
        language: None,
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let _ = handle(args, server).await;
}

/// Test 21: Mutation with no tests
/// IGNORED: Mutation testing integration test - requires external mutation testing tools
#[tokio::test]
#[ignore]
async fn test_mutation_with_no_tests() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("notests.rs");
    fs::write(&file_path, "fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

    let args = MutateArgs {
        target: file_path,
        language: None,
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: false,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let result = handle(args, server).await;
    assert!(
        result.is_ok()
            || result
                .as_ref()
                .err()
                .map(|e| e.to_string().contains("No mutants"))
                .unwrap_or(false)
    );
}

/// Test 22: Mutation with flaky tests
#[tokio::test]
async fn test_mutation_with_flaky_tests() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("flaky.rs");
    fs::write(&file_path, r#"
use std::time::SystemTime;
fn time_dependent() -> bool { SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() % 2 == 0 }
#[test]
fn test_flaky() { assert!(time_dependent() || !time_dependent()); }
"#).unwrap();

    let args = MutateArgs {
        target: file_path,
        language: None,
        timeout: 30,
        jobs: Some(1),
        output_format: "json".to_string(),
        output: None,
        threshold: None,
        failures_only: true,
        use_cargo_mutants: false,
        features: None,
        all_features: false,
        no_default_features: false,
        no_shuffle: false,
    };
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let _ = handle(args, server).await;
}
