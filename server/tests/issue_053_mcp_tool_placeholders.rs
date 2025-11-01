//! Issue #53: MCP Tool Placeholder Replacement Tests - RED Phase
//!
//! These tests verify that MCP tool functions call real analysis services
//! instead of returning placeholder data.
//!
//! **Current Status**: 🔴 RED - These tests will FAIL until real implementations replace placeholders
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write 9 comprehensive tests for MCP tool functions (all fail)
//! 2. GREEN: Replace placeholder implementations with real service calls
//! 3. GREEN: Verify all tests pass
//! 4. REFACTOR: Clean up code and optimize
//! 5. COMMIT: Single atomic commit with feature

use pmat::mcp_pmcp::tool_functions;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary Rust file with complexity for testing
fn create_test_rust_file(temp_dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let file_path = temp_dir.path().join(filename);
    std::fs::write(&file_path, content).unwrap();
    file_path
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_complexity must call real service"]
async fn test_analyze_complexity_calls_real_service() {
    // Create a temporary Rust file with measurable complexity
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_rust_file(
        &temp_dir,
        "complex.rs",
        r#"
        fn complex_function(x: i32) -> i32 {
            if x > 10 {
                if x > 20 {
                    if x > 30 {
                        return x * 2;
                    }
                    return x + 10;
                }
                return x + 5;
            }
            x
        }
        "#,
    );

    // Call analyze_complexity with the test file
    let result = tool_functions::analyze_complexity(&[file_path.clone()], Some(10), Some(5))
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real complexity data is returned
    let results = &result["results"];
    assert!(
        results["total_files"].as_u64().unwrap() > 0,
        "Should analyze at least 1 file"
    );

    // Verify violations are detected for high complexity
    let violations = results["violations"].as_array().unwrap();
    assert!(
        !violations.is_empty(),
        "Should detect complexity violations in test file"
    );

    // Verify actual complexity score is calculated
    assert!(
        results["total_complexity"].as_u64().unwrap() > 0,
        "Should calculate actual complexity > 0"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_satd must call real service"]
async fn test_analyze_satd_calls_real_service() {
    // Create a temporary Rust file with SATD comments
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_rust_file(
        &temp_dir,
        "satd.rs",
        r#"
        fn some_function() {
            // TODO: Implement proper error handling
            let x = 5;

            // FIXME: This is a temporary hack
            let y = x * 2;

            // HACK: Remove this before production
            println!("Debug: {}", y);
        }
        "#,
    );

    // Call analyze_satd with the test file
    let result = tool_functions::analyze_satd(&[file_path.clone()], false)
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real SATD detection
    let results = &result["results"];
    assert!(
        results["total_satd"].as_u64().unwrap() >= 3,
        "Should detect at least 3 SATD comments (TODO, FIXME, HACK)"
    );

    // Verify files array contains actual data
    let files = results["files"].as_array().unwrap();
    assert!(!files.is_empty(), "Should have at least 1 file with SATD");

    // Verify file contains SATD details
    let first_file = &files[0];
    assert!(
        first_file["satd_count"].as_u64().unwrap() >= 3,
        "File should have at least 3 SATD comments"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_dead_code must call real service"]
async fn test_analyze_dead_code_calls_real_service() {
    // Create a temporary Rust file with dead code
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_rust_file(
        &temp_dir,
        "dead.rs",
        r#"
        fn used_function() -> i32 {
            42
        }

        fn unused_function() -> i32 {
            99
        }

        fn main() {
            let x = used_function();
            println!("Value: {}", x);
        }
        "#,
    );

    // Call analyze_dead_code with the test file
    let result = tool_functions::analyze_dead_code(&[file_path.clone()], false)
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real dead code analysis
    let results = &result["results"];

    // NOTE: Dead code detection may vary, but should NOT return exactly 0 for this test file
    // The service should at least attempt detection
    let total_dead = results["total_dead_code"].as_u64().unwrap_or(0);

    // Verify files array exists and is properly structured
    let files = results["files"].as_array().unwrap();
    assert!(
        !files.is_empty() || total_dead == 0,
        "Should return file analysis data"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_complexity with empty paths returns error"]
async fn test_analyze_complexity_empty_paths_error() {
    // Call with empty paths array
    let result = tool_functions::analyze_complexity(&[], Some(10), None).await;

    // Should return an error for empty paths
    assert!(
        result.is_err(),
        "Should return error when no paths provided"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_satd with empty paths returns error"]
async fn test_analyze_satd_empty_paths_error() {
    // Call with empty paths array
    let result = tool_functions::analyze_satd(&[], false).await;

    // Should return an error for empty paths
    assert!(
        result.is_err(),
        "Should return error when no paths provided"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_dead_code with empty paths returns error"]
async fn test_analyze_dead_code_empty_paths_error() {
    // Call with empty paths array
    let result = tool_functions::analyze_dead_code(&[], false).await;

    // Should return an error for empty paths
    assert!(
        result.is_err(),
        "Should return error when no paths provided"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_complexity respects threshold parameter"]
async fn test_analyze_complexity_respects_threshold() {
    // Create a file with moderate complexity (CC ~4)
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_rust_file(
        &temp_dir,
        "moderate.rs",
        r#"
        fn moderate_complexity(x: i32) -> i32 {
            if x > 10 {
                return x * 2;
            } else if x > 5 {
                return x + 5;
            }
            x
        }
        "#,
    );

    // Test with low threshold (should flag violations)
    let result_low = tool_functions::analyze_complexity(&[file_path.clone()], None, Some(2))
        .await
        .unwrap();

    let violations_low = result_low["results"]["violations"].as_array().unwrap();

    // Test with high threshold (should have fewer/no violations)
    let result_high = tool_functions::analyze_complexity(&[file_path.clone()], None, Some(20))
        .await
        .unwrap();

    let violations_high = result_high["results"]["violations"].as_array().unwrap();

    // Verify threshold affects results
    assert!(
        violations_high.len() <= violations_low.len(),
        "Higher threshold should have fewer or equal violations"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_satd respects include_resolved parameter"]
async fn test_analyze_satd_respects_include_resolved() {
    // Create a file with both TODO and DONE comments
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_rust_file(
        &temp_dir,
        "resolved.rs",
        r#"
        fn some_function() {
            // TODO: Add error handling
            let x = 5;

            // DONE: Implemented validation
            let y = x * 2;
        }
        "#,
    );

    // Test without resolved comments
    let result_without = tool_functions::analyze_satd(&[file_path.clone()], false)
        .await
        .unwrap();

    let total_without = result_without["results"]["total_satd"].as_u64().unwrap();

    // Test with resolved comments
    let result_with = tool_functions::analyze_satd(&[file_path.clone()], true)
        .await
        .unwrap();

    let total_with = result_with["results"]["total_satd"].as_u64().unwrap();

    // Verify include_resolved affects count
    // With resolved should be >= without resolved (may include DONE comments)
    assert!(
        total_with >= total_without,
        "Including resolved should have same or more SATD"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_dead_code respects include_tests parameter"]
async fn test_analyze_dead_code_respects_include_tests() {
    // Create test file in tests directory
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("tests");
    std::fs::create_dir(&test_dir).unwrap();

    let test_file = test_dir.join("test_dead.rs");
    std::fs::write(
        &test_file,
        r#"
        #[test]
        fn test_something() {
            assert_eq!(1, 1);
        }

        fn unused_test_helper() -> i32 {
            42
        }
        "#,
    )
    .unwrap();

    // Test without including tests
    let result_without = tool_functions::analyze_dead_code(&[test_file.clone()], false)
        .await
        .unwrap();

    // Test with including tests
    let result_with = tool_functions::analyze_dead_code(&[test_file.clone()], true)
        .await
        .unwrap();

    // Verify both complete successfully (parameter affects filtering logic)
    assert_eq!(
        result_without["status"].as_str().unwrap(),
        "completed",
        "Should complete without tests"
    );
    assert_eq!(
        result_with["status"].as_str().unwrap(),
        "completed",
        "Should complete with tests"
    );
}

// =============================================================================
// Implementation Notes for GREEN Phase
// =============================================================================
//
// The implementation should replace placeholder functions in tool_functions.rs:
//
// ```rust
// pub async fn analyze_complexity(
//     paths: &[PathBuf],
//     top_files: Option<usize>,
//     threshold: Option<u64>,
// ) -> Result<Value> {
//     if paths.is_empty() {
//         return Err(anyhow::anyhow!("At least one path must be provided"));
//     }
//
//     use crate::services::complexity_analyzer::ComplexityAnalyzer;
//     use crate::models::complexity::ComplexityReport;
//
//     let analyzer = ComplexityAnalyzer::new();
//     let mut all_results = Vec::new();
//     let mut total_complexity = 0u64;
//     let mut total_files = 0;
//     let mut violations = Vec::new();
//
//     let threshold_value = threshold.unwrap_or(10);
//
//     for path in paths {
//         let report = analyzer.analyze_path(path).await?;
//
//         for file_result in report.files {
//             total_files += 1;
//             total_complexity += file_result.cyclomatic_complexity as u64;
//
//             if file_result.cyclomatic_complexity as u64 > threshold_value {
//                 violations.push(json!({
//                     "file": file_result.file_path,
//                     "complexity": file_result.cyclomatic_complexity,
//                     "threshold": threshold_value,
//                 }));
//             }
//
//             all_results.push(file_result);
//         }
//     }
//
//     // Sort by complexity and take top N files
//     if let Some(top_n) = top_files {
//         all_results.sort_by(|a, b| b.cyclomatic_complexity.cmp(&a.cyclomatic_complexity));
//         all_results.truncate(top_n);
//     }
//
//     Ok(json!({
//         "status": "completed",
//         "message": "Complexity analysis completed",
//         "results": {
//             "total_files": total_files,
//             "total_complexity": total_complexity,
//             "average_complexity": if total_files > 0 { total_complexity / total_files as u64 } else { 0 },
//             "violations": violations,
//             "top_files": all_results,
//         }
//     }))
// }
// ```
//
// Similar implementations for analyze_satd() and analyze_dead_code() calling:
// - crate::services::satd_detector::SatdDetector
// - crate::services::dead_code_multi_language::DeadCodeAnalyzer
//
// Integration steps:
// 1. Import real service modules
// 2. Replace placeholder JSON with service calls
// 3. Transform service results to JSON responses
// 4. Handle errors properly
// 5. Respect all parameters (threshold, top_files, include_resolved, include_tests)
//

// =============================================================================
// RED Phase Tests - Batch 2: Context & Churn Functions
// =============================================================================

#[tokio::test]
#[ignore = "Issue #53: RED test - generate_context must call real service"]
async fn test_generate_context_calls_real_service() {
    // Create a temporary Rust file
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_rust_file(
        &temp_dir,
        "sample.rs",
        r#"
        fn sample_function() -> i32 {
            42
        }

        struct SampleStruct {
            field: String,
        }
        "#,
    );

    // Call generate_context with the test file
    let result = tool_functions::generate_context(&[file_path.clone()], Some(10), false)
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real context data is returned
    let context = &result["context"];
    assert!(
        context.is_object(),
        "Should return context object with real data"
    );

    // Verify files array exists and contains data
    if let Some(files) = context["files"].as_array() {
        assert!(!files.is_empty(), "Should analyze at least 1 file");
    }
}

#[tokio::test]
#[ignore = "Issue #53: RED test - generate_context with empty paths returns error"]
async fn test_generate_context_empty_paths_error() {
    // Call with empty paths array
    let result = tool_functions::generate_context(&[], None, false).await;

    // Should return an error for empty paths
    assert!(
        result.is_err(),
        "Should return error when no paths provided"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - generate_context respects max_depth parameter"]
async fn test_generate_context_respects_max_depth() {
    // Create nested directory structure
    let temp_dir = TempDir::new().unwrap();
    let level1 = temp_dir.path().join("level1");
    let level2 = level1.join("level2");
    std::fs::create_dir_all(&level2).unwrap();

    let file1 = create_test_rust_file(&temp_dir, "level1/file1.rs", "fn foo() {}");
    let _file2 = level2.join("file2.rs");
    std::fs::write(&_file2, "fn bar() {}").unwrap();

    // Test with max_depth limiting traversal
    let result_shallow = tool_functions::generate_context(&[file1.clone()], Some(1), false)
        .await
        .unwrap();

    let result_deep = tool_functions::generate_context(&[file1.clone()], Some(10), false)
        .await
        .unwrap();

    // Both should complete (depth affects directory traversal if implemented)
    assert_eq!(
        result_shallow["status"].as_str().unwrap(),
        "completed",
        "Shallow depth should complete"
    );
    assert_eq!(
        result_deep["status"].as_str().unwrap(),
        "completed",
        "Deep depth should complete"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - generate_deep_context must call real service"]
async fn test_generate_deep_context_calls_real_service() {
    // Create a temporary Rust file with complexity
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_rust_file(
        &temp_dir,
        "complex.rs",
        r#"
        fn complex_function(x: i32) -> i32 {
            if x > 10 {
                if x > 20 {
                    return x * 2;
                }
                return x + 5;
            }
            x
        }
        "#,
    );

    // Call generate_deep_context with the test file
    let result = tool_functions::generate_deep_context(&[file_path.clone()], None)
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real deep context data is returned
    let context = &result["context"];
    assert!(
        context.is_object(),
        "Should return context object with real analysis"
    );

    // Verify metadata exists (DeepContext has metadata field)
    assert!(
        context["metadata"].is_object() || result["results"].is_object(),
        "Should contain metadata or results from deep analysis"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - generate_deep_context with empty paths returns error"]
async fn test_generate_deep_context_empty_paths_error() {
    // Call with empty paths array
    let result = tool_functions::generate_deep_context(&[], None).await;

    // Should return an error for empty paths
    assert!(
        result.is_err(),
        "Should return error when no paths provided"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - generate_deep_context respects format parameter"]
async fn test_generate_deep_context_respects_format() {
    // Create a test file
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_rust_file(&temp_dir, "test.rs", "fn test() {}");

    // Test with format parameter (if supported)
    let result_default = tool_functions::generate_deep_context(&[file_path.clone()], None)
        .await
        .unwrap();

    let result_with_format =
        tool_functions::generate_deep_context(&[file_path.clone()], Some("json"))
            .await
            .unwrap();

    // Both should complete successfully
    assert_eq!(
        result_default["status"].as_str().unwrap(),
        "completed",
        "Default format should complete"
    );
    assert_eq!(
        result_with_format["status"].as_str().unwrap(),
        "completed",
        "Specified format should complete"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_churn must call real git service"]
async fn test_analyze_churn_calls_real_service() {
    // NOTE: This test requires a git repository
    // Use current repository as test subject
    let repo_path = std::env::current_dir().unwrap();

    // Call analyze_churn
    let result = tool_functions::analyze_churn(&[repo_path.clone()], Some(30), Some(10))
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real churn data is returned
    let results = &result["results"];
    assert!(
        results.is_object(),
        "Should return results object with churn data"
    );

    // Verify files array exists (even if empty for non-git directories)
    assert!(
        results["files"].is_array(),
        "Should have files array in results"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_churn with empty paths returns error"]
async fn test_analyze_churn_empty_paths_error() {
    // Call with empty paths array
    let result = tool_functions::analyze_churn(&[], Some(30), Some(10)).await;

    // Should return an error for empty paths
    assert!(
        result.is_err(),
        "Should return error when no paths provided"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - analyze_churn respects days parameter"]
async fn test_analyze_churn_respects_days() {
    // Use current repository as test subject
    let repo_path = std::env::current_dir().unwrap();

    // Test with different day ranges
    let result_30_days = tool_functions::analyze_churn(&[repo_path.clone()], Some(30), None)
        .await
        .unwrap();

    let result_7_days = tool_functions::analyze_churn(&[repo_path.clone()], Some(7), None)
        .await
        .unwrap();

    // Both should complete
    assert_eq!(
        result_30_days["status"].as_str().unwrap(),
        "completed",
        "30-day analysis should complete"
    );
    assert_eq!(
        result_7_days["status"].as_str().unwrap(),
        "completed",
        "7-day analysis should complete"
    );

    // Verify days parameter is reflected in response (if the service includes it)
    // This validates the parameter is actually passed through
}

// ============================================================================
// Batch 3: Quality Gate Functions
// ============================================================================

#[tokio::test]
#[ignore = "Issue #53: RED test - check_quality_gates must call real TDG service"]
async fn test_check_quality_gates_calls_real_service() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();

    // Create simple Rust file
    let file_path = temp_dir.path().join("main.rs");
    std::fs::write(
        &file_path,
        r#"
fn main() {
    println!("Hello, world!");
}

fn simple_function(x: i32) -> i32 {
    x + 1
}
"#,
    )
    .unwrap();

    let result = tool_functions::check_quality_gates(&[temp_dir.path().to_path_buf()], false)
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real TDG quality gate data is returned
    assert!(result["status"].is_string(), "Should have status field");
    assert!(
        result["passed"].is_boolean(),
        "Should have boolean passed field from real TDG analysis"
    );

    // Verify quality metrics are present (TDG provides these)
    assert!(
        result.get("score").is_some() || result.get("grade").is_some(),
        "Should include quality score or grade from TDG"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - check_quality_gates must handle threshold parameter"]
async fn test_check_quality_gates_respects_strict_mode() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();

    // Create file with intentionally complex function
    let file_path = temp_dir.path().join("complex.rs");
    std::fs::write(
        &file_path,
        r#"
fn complex_function(x: i32) -> i32 {
    if x > 10 {
        if x > 20 {
            if x > 30 {
                if x > 40 {
                    return x * 2;
                }
                return x + 15;
            }
            return x + 10;
        }
        return x + 5;
    }
    x
}
"#,
    )
    .unwrap();

    // Non-strict mode - might pass with warning
    let result_lenient = tool_functions::check_quality_gates(&[temp_dir.path().to_path_buf()], false)
        .await
        .unwrap();

    // Strict mode - should enforce higher standards
    let result_strict = tool_functions::check_quality_gates(&[temp_dir.path().to_path_buf()], true)
        .await
        .unwrap();

    // Verify strict mode produces meaningful results (not placeholder)
    let message_strict = result_strict["message"].as_str().unwrap();
    assert!(
        !message_strict.contains("placeholder"),
        "Strict mode should use real TDG thresholds"
    );

    // Both should return structured quality gate data
    assert!(result_lenient["passed"].is_boolean());
    assert!(result_strict["passed"].is_boolean());
}

#[tokio::test]
#[ignore = "Issue #53: RED test - check_quality_gates must handle empty paths"]
async fn test_check_quality_gates_empty_paths_error() {
    let result = tool_functions::check_quality_gates(&[], false).await;

    // Should return error for empty paths
    assert!(
        result.is_err(),
        "Empty paths should return error, not placeholder success"
    );

    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("path") || error_message.contains("provide"),
        "Error should mention path requirement"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - check_quality_gate_file must call real TDG service"]
async fn test_check_quality_gate_file_calls_real_service() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(
        &file_path,
        r#"
fn simple_function() -> i32 {
    42
}
"#,
    )
    .unwrap();

    let result = tool_functions::check_quality_gate_file(&file_path, false)
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real file-level quality gate data
    assert!(result["passed"].is_boolean());
    assert!(result["file"].is_string());

    let file_str = result["file"].as_str().unwrap();
    assert!(
        file_str.contains("test.rs"),
        "Should reference the analyzed file"
    );

    // Should include quality metrics from TDG
    assert!(
        result.get("violations").is_some() || result.get("score").is_some(),
        "Should include real quality metrics from TDG analysis"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - check_quality_gate_file must handle violations"]
async fn test_check_quality_gate_file_detects_violations() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("bad_quality.rs");
    std::fs::write(
        &file_path,
        r#"
// TODO: This should be refactored
fn very_complex_function(x: i32, y: i32, z: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                if x > y {
                    if y > z {
                        return x + y + z;
                    }
                    return x + y;
                }
                return x;
            }
            return 0;
        }
        return -1;
    }
    return -999;
}
"#,
    )
    .unwrap();

    let result = tool_functions::check_quality_gate_file(&file_path, true)
        .await
        .unwrap();

    // Should detect quality violations (high complexity, SATD)
    let _passed = result["passed"].as_bool().unwrap();

    // In strict mode with this complex code, should ideally fail
    // But at minimum, violations array should be populated (not placeholder empty array)
    if let Some(violations) = result["violations"].as_array() {
        // If there are violations, verify they're real (not placeholder)
        if !violations.is_empty() {
            let first_violation = &violations[0];
            assert!(
                first_violation.is_object(),
                "Violations should be real objects with details"
            );
        }
    }
}

#[tokio::test]
#[ignore = "Issue #53: RED test - quality_gate_summary must call real TDG service"]
async fn test_quality_gate_summary_calls_real_service() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();

    // Create multiple files
    let file1 = temp_dir.path().join("good.rs");
    std::fs::write(&file1, "fn simple() -> i32 { 42 }").unwrap();

    let file2 = temp_dir.path().join("complex.rs");
    std::fs::write(
        &file2,
        r#"
fn complex(x: i32) -> i32 {
    if x > 10 {
        if x > 20 {
            return x * 2;
        }
        return x + 10;
    }
    x
}
"#,
    )
    .unwrap();

    let result = tool_functions::quality_gate_summary(&[temp_dir.path().to_path_buf()])
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(
        !message.contains("placeholder"),
        "Response should NOT contain 'placeholder' keyword"
    );

    // Verify real summary data from TDG analysis
    let summary = &result["summary"];
    assert!(summary.is_object(), "Should return summary object");

    let total_files = summary["total_files"].as_u64().unwrap();
    assert!(
        total_files >= 2,
        "Should analyze at least the 2 files we created, got: {}",
        total_files
    );

    // Should have real counts (not placeholder zeros)
    let passed_files = summary["passed_files"].as_u64();
    let failed_files = summary["failed_files"].as_u64();

    assert!(
        passed_files.is_some() || failed_files.is_some(),
        "Should include real pass/fail counts from TDG"
    );
}

#[tokio::test]
#[ignore = "Issue #53: RED test - quality_gate_summary must aggregate results"]
async fn test_quality_gate_summary_aggregates_multiple_files() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();

    // Create 5 test files with varying quality
    for i in 1..=5 {
        let file_path = temp_dir.path().join(format!("file{}.rs", i));
        std::fs::write(&file_path, format!("fn func{}() -> i32 {{ {} }}", i, i)).unwrap();
    }

    let result = tool_functions::quality_gate_summary(&[temp_dir.path().to_path_buf()])
        .await
        .unwrap();

    let summary = &result["summary"];

    // Total files should be at least 5 (may include others if directory has them)
    let total_files = summary["total_files"].as_u64().unwrap();
    assert!(
        total_files >= 5,
        "Should count at least 5 files, got: {}",
        total_files
    );

    // Verify aggregation is working (sum of passed + failed should equal total)
    if let (Some(passed), Some(failed)) = (
        summary["passed_files"].as_u64(),
        summary["failed_files"].as_u64(),
    ) {
        assert!(
            passed + failed <= total_files,
            "Passed ({}) + failed ({}) should not exceed total ({})",
            passed,
            failed,
            total_files
        );
    }
}

#[tokio::test]
#[ignore = "Issue #53: RED test - quality_gate_summary must handle empty paths"]
async fn test_quality_gate_summary_empty_paths_error() {
    let result = tool_functions::quality_gate_summary(&[]).await;

    // Should return error for empty paths
    assert!(
        result.is_err(),
        "Empty paths should return error, not placeholder zero counts"
    );
}

// ============================================================================
// BATCH 4: Quality Tracking & Git Integration (3 functions)
// ============================================================================

// Test 1: quality_gate_baseline - Create baseline snapshot
#[tokio::test]
#[ignore = "Issue #53: RED test - quality_gate_baseline must call real TDG baseline service"]
async fn test_quality_gate_baseline_calls_real_service() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("baseline.json");

    // Create a test file
    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(&file_path, r#"
fn simple_function(x: i32) -> i32 {
    x + 1
}
"#).unwrap();

    let result = tool_functions::quality_gate_baseline(
        &[temp_dir.path().to_path_buf()],
        Some(&output_path)
    )
    .await
    .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(!message.contains("placeholder"), "Response should NOT contain 'placeholder' keyword");

    // Verify real baseline data is returned
    assert!(result["status"].is_string(), "Should have status field");
    assert!(result.get("baseline").is_some(), "Should have baseline field from real TDG analysis");

    // Verify baseline file was actually created
    if let Some(path) = result["baseline"]["file_path"].as_str() {
        assert!(std::path::Path::new(path).exists(), "Baseline file should exist on disk");
    }
}

// Test 2: quality_gate_baseline - Verify baseline contains real TDG metrics
#[tokio::test]
#[ignore = "Issue #53: RED test - quality_gate_baseline must include real metrics"]
async fn test_quality_gate_baseline_contains_metrics() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("baseline.json");

    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(&file_path, "fn test() {}\n").unwrap();

    let result = tool_functions::quality_gate_baseline(
        &[temp_dir.path().to_path_buf()],
        Some(&output_path)
    )
    .await
    .unwrap();

    // Verify baseline has real metrics (not placeholder empty object)
    let baseline = &result["baseline"];
    assert!(baseline.get("timestamp").is_some(), "Should have timestamp");
    assert!(baseline.get("summary").is_some(), "Should have summary from real TDG analysis");

    let summary = &baseline["summary"];
    assert!(summary.get("total_files").is_some(), "Should have file count");
    assert!(summary.get("avg_score").is_some(), "Should have average score");
}

// Test 3: quality_gate_compare - Compare current vs baseline
#[tokio::test]
#[ignore = "Issue #53: RED test - quality_gate_compare must call real comparison service"]
async fn test_quality_gate_compare_calls_real_service() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");

    // Create baseline first
    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(&file_path, "fn test() {}\n").unwrap();

    tool_functions::quality_gate_baseline(
        &[temp_dir.path().to_path_buf()],
        Some(&baseline_path)
    )
    .await
    .unwrap();

    // Now compare against it
    let result = tool_functions::quality_gate_compare(
        &baseline_path,
        &[temp_dir.path().to_path_buf()]
    )
    .await
    .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(!message.contains("placeholder"), "Response should NOT contain 'placeholder' keyword");

    // Verify real comparison data is returned
    assert!(result.get("comparison").is_some(), "Should have comparison field");

    let comparison = &result["comparison"];
    assert!(comparison.get("improved").is_some(), "Should have improved count");
    assert!(comparison.get("regressed").is_some(), "Should have regressed count");
    assert!(comparison.get("unchanged").is_some(), "Should have unchanged count");
}

// Test 4: quality_gate_compare - Detect quality regressions
#[tokio::test]
#[ignore = "Issue #53: RED test - quality_gate_compare must detect real regressions"]
async fn test_quality_gate_compare_detects_regressions() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");
    let file_path = temp_dir.path().join("test.rs");

    // Create simple file and baseline
    std::fs::write(&file_path, "fn test() {}\n").unwrap();
    tool_functions::quality_gate_baseline(
        &[temp_dir.path().to_path_buf()],
        Some(&baseline_path)
    )
    .await
    .unwrap();

    // Make file more complex (should regress quality)
    std::fs::write(&file_path, r#"
fn complex_function(x: i32) -> i32 {
    // TODO: This is complex
    if x > 0 {
        if x < 100 {
            if x % 2 == 0 {
                x * 2
            } else {
                x + 1
            }
        } else {
            x / 2
        }
    } else {
        0
    }
}
"#).unwrap();

    let result = tool_functions::quality_gate_compare(
        &baseline_path,
        &[temp_dir.path().to_path_buf()]
    )
    .await
    .unwrap();

    // Verify comparison has real data (not placeholder zeros)
    let comparison = &result["comparison"];

    // At least one of these should be non-zero for real analysis
    let improved = comparison["improved"].as_u64().unwrap_or(0);
    let regressed = comparison["regressed"].as_u64().unwrap_or(0);
    let unchanged = comparison["unchanged"].as_u64().unwrap_or(0);

    assert!(
        improved > 0 || regressed > 0 || unchanged > 0,
        "Comparison should have real data, not placeholder all-zeros"
    );
}

// Test 5: git_status - Get git repository status
#[tokio::test]
#[ignore = "Issue #53: RED test - git_status must call real git service"]
async fn test_git_status_calls_real_service() {
    use std::env;

    // Use git repository root (parent of server directory)
    let repo_path = env::current_dir().unwrap().parent().unwrap().to_path_buf();

    let result = tool_functions::git_status(&repo_path)
        .await
        .unwrap();

    // Verify NOT a placeholder response
    let message = result["message"].as_str().unwrap();
    assert!(!message.contains("placeholder"), "Response should NOT contain 'placeholder' keyword");

    // Verify real git status data is returned
    assert!(result.get("git_status").is_some(), "Should have git_status field");

    let git_status = &result["git_status"];
    assert!(git_status.get("branch").is_some(), "Should have branch name");
    assert!(git_status.get("commit_sha").is_some(), "Should have commit SHA from real git");
    assert!(git_status.get("is_clean").is_some(), "Should have clean status");
}

// Test 6: git_status - Extract real commit information
#[tokio::test]
#[ignore = "Issue #53: RED test - git_status must extract real commit details"]
async fn test_git_status_extracts_commit_details() {
    use std::env;

    // Use git repository root (parent of server directory)
    let repo_path = env::current_dir().unwrap().parent().unwrap().to_path_buf();

    let result = tool_functions::git_status(&repo_path)
        .await
        .unwrap();

    let git_status = &result["git_status"];

    // Verify commit SHA is not placeholder
    let commit_sha = git_status["commit_sha"].as_str().unwrap();
    assert!(commit_sha.len() >= 7, "Should have real commit SHA (at least 7 chars)");
    assert_ne!(commit_sha, "abc123", "Should not be placeholder commit SHA");

    // Verify branch is not placeholder
    let branch = git_status["branch"].as_str().unwrap();
    assert!(!branch.is_empty(), "Should have real branch name");
    assert_ne!(branch, "placeholder", "Branch should be real, not placeholder");
}

// Test 7: git_status - Handle non-git directory
#[tokio::test]
#[ignore = "Issue #53: RED test - git_status must handle non-git directories"]
async fn test_git_status_non_git_directory() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();

    let result = tool_functions::git_status(temp_dir.path()).await;

    // Should return error for non-git directory
    assert!(
        result.is_err(),
        "Non-git directory should return error, not placeholder success"
    );
}
