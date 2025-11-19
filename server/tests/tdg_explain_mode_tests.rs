//! TDG --explain Mode Integration Tests (Issue #78)
//!
//! EXTREME TDD - RED Phase
//! These tests document the expected behavior of the --explain mode
//! and should ALL FAIL until the implementation is complete (GREEN phase).
//!
//! Test Categories:
//! 1. Function Complexity Extraction (from Rust source code)
//! 2. Pattern Detection (repeated match statements, oversized modules)
//! 3. Recommendation Generation (actionable refactoring suggestions)
//! 4. Baseline Comparison (tracking progress over time)
//! 5. Output Formatting (text and JSON)
//!
//! References:
//! - Specification: docs/specifications/tdg-explain-mode.md
//! - Issue: #78

use pmat::tdg::{
    ActionableRecommendation, ComplexitySeverity, ExplainedTDGScore, FunctionComplexity,
    RecommendationType,
};
use std::path::PathBuf;
use tempfile::TempDir;

/// GREEN TEST 1: Extract function complexity from simple Rust file
///
/// Verifies that the FunctionAnalyzer can parse a Rust file and extract
/// function-level complexity metrics.
///
/// GREEN: FunctionAnalyzer implemented, test should pass
#[test]
fn test_extract_function_complexity_from_rust_file() {
    // Create a test Rust file
    let test_code = r#"
        fn simple_function() -> i32 {
            return 42;
        }

        fn medium_complexity_function(x: i32) -> i32 {
            if x > 10 {
                if x > 20 {
                    return x * 2;
                } else {
                    return x + 5;
                }
            } else {
                return x - 3;
            }
        }

        fn high_complexity_function(value: i32) -> String {
            match value {
                0 => "zero".to_string(),
                1 => "one".to_string(),
                2 => "two".to_string(),
                3 => "three".to_string(),
                4 => "four".to_string(),
                5 => "five".to_string(),
                6 => "six".to_string(),
                7 => "seven".to_string(),
                8 => "eight".to_string(),
                9 => "nine".to_string(),
                10 => "ten".to_string(),
                _ => "many".to_string(),
            }
        }
    "#;

    // Write test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, test_code).unwrap();

    // GREEN: FunctionAnalyzer now working!
    let functions = extract_function_complexity(&test_file).unwrap();

    // Verify 3 functions extracted
    assert_eq!(functions.len(), 3, "Should extract 3 functions");

    // Verify simple_function
    let simple = functions.iter().find(|f| f.name == "simple_function").unwrap();
    assert_eq!(simple.cyclomatic, 1, "simple_function should have cyclomatic complexity 1");
    assert_eq!(simple.severity, ComplexitySeverity::Low);

    // Verify medium_complexity_function
    let medium = functions
        .iter()
        .find(|f| f.name == "medium_complexity_function")
        .unwrap();

    // Nested if/else: 1 (base) + 2 (two if statements) = 3
    assert_eq!(
        medium.cyclomatic, 3,
        "medium_complexity_function should have cyclomatic = 3"
    );

    // Complexity 3 is Low according to standard McCabe thresholds (Low: 0-5)
    assert_eq!(
        medium.severity,
        ComplexitySeverity::Low,
        "Complexity 3 should be Low severity per McCabe standards"
    );

    // Verify high_complexity_function
    let high = functions
        .iter()
        .find(|f| f.name == "high_complexity_function")
        .unwrap();
    assert!(
        high.cyclomatic >= 11,
        "high_complexity_function should have cyclomatic >= 11 (match with 11 arms)"
    );
    assert_eq!(
        high.severity,
        ComplexitySeverity::High,
        "high_complexity_function should be High severity"
    );
}

/// RED TEST 2: Detect repeated match/switch dispatch pattern
///
/// Verifies that the PatternDetector can identify repeated match statements
/// that are candidates for macro extraction.
///
/// Expected to FAIL: PatternDetector not implemented yet
#[test]
#[ignore] // RED: Will fail until PatternDetector is implemented
fn test_detect_dispatch_pattern() {
    let test_code = r#"
        fn operation_a(backend: Backend) -> i32 {
            match backend {
                Backend::Simd => simd_impl_a(),
                Backend::Gpu => gpu_impl_a(),
                Backend::Scalar => scalar_impl_a(),
            }
        }

        fn operation_b(backend: Backend) -> i32 {
            match backend {
                Backend::Simd => simd_impl_b(),
                Backend::Gpu => gpu_impl_b(),
                Backend::Scalar => scalar_impl_b(),
            }
        }

        fn operation_c(backend: Backend) -> i32 {
            match backend {
                Backend::Simd => simd_impl_c(),
                Backend::Gpu => gpu_impl_c(),
                Backend::Scalar => scalar_impl_c(),
            }
        }
    "#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("dispatch.rs");
    std::fs::write(&test_file, test_code).unwrap();

    // RED: This should fail - detect_dispatch_pattern doesn't exist yet
    let patterns = detect_dispatch_pattern(&test_file).unwrap();

    // Should detect the repeated match pattern
    assert!(
        !patterns.is_empty(),
        "Should detect at least one dispatch pattern"
    );
    assert_eq!(
        patterns[0].branch_count, 3,
        "Should detect 3 branches in match statement"
    );
}

/// GREEN TEST 3: Generate actionable recommendations
///
/// Verifies that the RecommendationEngine can generate specific, actionable
/// recommendations with estimated impact and effort.
///
/// GREEN: RecommendationEngine now implemented
#[test]
fn test_generate_recommendations_from_complexity() {
    use pmat::tdg::generate_recommendations;

    // Create ExplainedTDGScore with high-complexity functions
    let mut explained = ExplainedTDGScore::new(pmat::tdg::TdgScore::default());

    explained.add_function(FunctionComplexity {
        name: "complex_function_1".to_string(),
        line_number: 100,
        cyclomatic: 25,
        cognitive: 30,
        tdg_impact: 4.5,
        severity: ComplexitySeverity::Critical,
    });

    explained.add_function(FunctionComplexity {
        name: "complex_function_2".to_string(),
        line_number: 200,
        cyclomatic: 22,
        cognitive: 28,
        tdg_impact: 4.2,
        severity: ComplexitySeverity::Critical,
    });

    // GREEN: generate_recommendations now works!
    let recommendations = generate_recommendations(&explained);

    // Should generate recommendations for high complexity
    assert!(
        !recommendations.is_empty(),
        "Should generate recommendations for complex functions"
    );

    // First recommendation should target complex_function_1 (highest impact)
    assert_eq!(
        recommendations[0].rec_type,
        RecommendationType::ReduceComplexity
    );
    assert!(
        recommendations[0].lines.contains(&100),
        "Recommendation should reference line 100"
    );
    assert!(
        recommendations[0].expected_impact > 0.0,
        "Should have positive expected impact"
    );
    assert!(
        recommendations[0].estimated_hours > 0.0,
        "Should have estimated effort"
    );
    assert_eq!(recommendations[0].priority, 1, "Highest impact should be priority 1");
}

/// RED TEST 4: Baseline comparison tracking
///
/// Verifies that the BaselineAnalyzer can compare current state against
/// a baseline and track progress.
///
/// Expected to FAIL: BaselineAnalyzer not implemented yet
#[test]
#[ignore] // RED: Will fail until BaselineAnalyzer is implemented
fn test_baseline_comparison() {
    let test_code = r#"
        fn refactored_function() -> i32 {
            // This function was previously complex but has been refactored
            simple_implementation()
        }

        fn still_complex_function(value: i32) -> String {
            match value {
                0..=9 => format!("digit_{}", value),
                _ => "other".to_string(),
            }
        }
    "#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("baseline_test.rs");
    std::fs::write(&test_file, test_code).unwrap();

    // RED: This should fail - compare_with_baseline doesn't exist yet
    let comparison = compare_with_baseline(&test_file, "baseline_ref").unwrap();

    // Verify baseline comparison structure
    assert_eq!(comparison.baseline_ref, "baseline_ref");
    assert!(
        comparison.delta != 0.0,
        "Delta should be non-zero (improvement or regression)"
    );

    // If we improved, should have completed recommendations
    if comparison.delta > 0.0 {
        assert!(
            !comparison.completed.is_empty(),
            "Should track completed recommendations"
        );
    }

    // Should still have pending recommendations
    assert!(
        !comparison.pending.is_empty(),
        "Should track pending recommendations"
    );
}

/// GREEN TEST 5: Text output formatting
///
/// Verifies that the explain mode produces human-readable text output
/// with function breakdown and recommendations.
///
/// GREEN: format_explain_text now implemented
#[test]
fn test_explain_text_output_format() {
    use pmat::tdg::format_explain_text;

    let mut explained = ExplainedTDGScore::new(pmat::tdg::TdgScore::default());

    explained.add_function(FunctionComplexity {
        name: "test_function".to_string(),
        line_number: 42,
        cyclomatic: 15,
        cognitive: 18,
        tdg_impact: 3.2,
        severity: ComplexitySeverity::High,
    });

    explained.add_recommendation(ActionableRecommendation {
        rec_type: RecommendationType::ExtractMacro,
        action: "Extract dispatch pattern into macro".to_string(),
        lines: vec![100, 200, 300],
        expected_impact: 8.5,
        estimated_hours: 5.0,
        priority: 1,
        pattern: "match_dispatch".to_string(),
    });

    // GREEN: format_explain_text now works!
    let output = format_explain_text(&explained).unwrap();

    // Verify output contains key sections
    assert!(output.contains("Function-Level Complexity"), "Should have function section");
    assert!(output.contains("test_function"), "Should show function name");
    assert!(output.contains("line 42"), "Should show line number");
    assert!(output.contains("Complexity: 15"), "Should show cyclomatic complexity");
    assert!(
        output.contains("Recommendations"),
        "Should have recommendations section"
    );
    assert!(output.contains("[+8.5 pts]"), "Should show expected impact");
    assert!(output.contains("Extract dispatch pattern"), "Should show action");
}

/// GREEN TEST 6: JSON output formatting
///
/// Verifies that the explain mode produces valid JSON output
/// suitable for CI/CD integration.
///
/// GREEN: format_explain_json now implemented
#[test]
fn test_explain_json_output_format() {
    use pmat::tdg::format_explain_json;

    let mut explained = ExplainedTDGScore::new(pmat::tdg::TdgScore::default());

    explained.add_function(FunctionComplexity {
        name: "test_function".to_string(),
        line_number: 42,
        cyclomatic: 15,
        cognitive: 18,
        tdg_impact: 3.2,
        severity: ComplexitySeverity::High,
    });

    // GREEN: format_explain_json now works!
    let output = format_explain_json(&explained).unwrap();

    // Parse JSON to verify structure
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Verify JSON structure
    assert!(json.get("functions").is_some(), "Should have functions array");
    let functions = json["functions"].as_array().unwrap();
    assert_eq!(functions.len(), 1, "Should have 1 function");

    let func = &functions[0];
    assert_eq!(func["name"].as_str().unwrap(), "test_function");
    assert_eq!(func["line"].as_u64().unwrap(), 42);
    assert_eq!(func["cyclomatic"].as_u64().unwrap(), 15);
    assert_eq!(func["tdg_impact"].as_f64().unwrap(), 3.2);
}

/// GREEN TEST 7: Threshold filtering
///
/// Verifies that --threshold flag correctly filters functions
/// based on cyclomatic complexity.
///
/// GREEN: Functionality already implemented in explain.rs data model
#[test]
fn test_threshold_filtering() {
    let mut explained = ExplainedTDGScore::new(pmat::tdg::TdgScore::default());

    explained.add_function(FunctionComplexity {
        name: "simple".to_string(),
        line_number: 10,
        cyclomatic: 5,
        cognitive: 6,
        tdg_impact: 1.0,
        severity: ComplexitySeverity::Low,
    });

    explained.add_function(FunctionComplexity {
        name: "complex".to_string(),
        line_number: 50,
        cyclomatic: 20,
        cognitive: 25,
        tdg_impact: 4.5,
        severity: ComplexitySeverity::Critical,
    });

    // Apply threshold of 10 (should filter out simple function)
    explained.filter_functions_by_threshold(10);

    assert_eq!(explained.total_functions(), 1, "Should have 1 function after filtering");
    assert_eq!(
        explained.functions[0].name, "complex",
        "Should keep only complex function"
    );
}

// ============================================================================
// RED Test Stub Functions (NOT IMPLEMENTED - these should fail to compile)
// ============================================================================
//
// The following functions are intentionally NOT implemented.
// They represent the interfaces we need to create in the GREEN phase.
//
// Compilation will fail with "unresolved import" or "cannot find function"
// errors, which is EXPECTED in the RED phase of EXTREME TDD.

/// Extract function complexity from Rust source file
///
/// GREEN: Now implemented using FunctionAnalyzer
fn extract_function_complexity(file: &PathBuf) -> Result<Vec<FunctionComplexity>, String> {
    use pmat::tdg::FunctionAnalyzer;

    let mut analyzer = FunctionAnalyzer::new()
        .map_err(|e| format!("Failed to create analyzer: {}", e))?;

    analyzer.analyze_file(file)
        .map_err(|e| format!("Failed to analyze file: {}", e))
}

/// Detect dispatch pattern in Rust source file
///
/// RED: Not implemented - this is the interface we need to create
#[allow(dead_code)]
struct DispatchPattern {
    branch_count: usize,
}

fn detect_dispatch_pattern(_file: &PathBuf) -> Result<Vec<DispatchPattern>, String> {
    // RED: Compilation will fail here - function not implemented
    unimplemented!("RED PHASE: detect_dispatch_pattern not implemented yet")
}

/// Compare with baseline git ref
///
/// RED: Not implemented - this is the interface we need to create
#[allow(dead_code)]
struct BaselineComparison {
    baseline_ref: String,
    delta: f64,
    completed: Vec<String>,
    pending: Vec<String>,
}

fn compare_with_baseline(_file: &PathBuf, _baseline_ref: &str) -> Result<BaselineComparison, String> {
    // RED: Compilation will fail here - function not implemented
    unimplemented!("RED PHASE: compare_with_baseline not implemented yet")
}

// ============================================================================
// RED Phase Summary
// ============================================================================
//
// All 7 tests above should FAIL in the RED phase:
//
// 1. test_extract_function_complexity_from_rust_file - FAIL (extract_function_complexity not implemented)
// 2. test_detect_dispatch_pattern - FAIL (detect_dispatch_pattern not implemented)
// 3. test_generate_recommendations_from_complexity - FAIL (generate_recommendations not implemented)
// 4. test_baseline_comparison - FAIL (compare_with_baseline not implemented)
// 5. test_explain_text_output_format - FAIL (format_explain_text not implemented)
// 6. test_explain_json_output_format - FAIL (format_explain_json not implemented)
// 7. test_threshold_filtering - PASS (basic data model functionality works)
//
// Expected Compilation: FAIL (unimplemented functions)
// Expected Test Execution: 0 passing, 7 failing (when #[ignore] is removed)
//
// Next Step (GREEN Phase):
// - Implement FunctionAnalyzer module (server/src/tdg/function_analyzer.rs)
// - Implement PatternDetector module (server/src/tdg/pattern_detector.rs)
// - Implement RecommendationEngine module (server/src/tdg/recommendation_engine.rs)
// - Implement BaselineAnalyzer module (server/src/tdg/baseline_analyzer.rs)
// - Implement formatters (server/src/tdg/explain_formatters.rs)
// - Update TDG handler to integrate --explain mode
// - Remove #[ignore] from tests
// - Verify all 7 tests PASS (GREEN phase)
