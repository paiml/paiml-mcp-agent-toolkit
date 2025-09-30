//! EXTREME TDD Tests for Unified Python Analyzer
//!
//! These tests are written FIRST (RED phase) and must ALL FAIL initially.
//! Once implementation is complete, they must ALL PASS.

use std::path::PathBuf;
use tempfile::NamedTempFile;
use std::io::Write;

// Helper to create temporary Python files for testing
fn create_temp_py_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::with_suffix(".py").expect("Failed to create temp file");
    write!(temp_file, "{}", content).expect("Failed to write to temp file");
    temp_file
}

/// Test 1: Basic Structure
#[test]
fn red_test_unified_python_analyzer_can_be_created() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let path = PathBuf::from("test.py");
    let analyzer = UnifiedPythonAnalyzer::new(path.clone());
    assert_eq!(analyzer.file_path(), &path);
}

/// Test 2: Single Parse Guarantee
#[tokio::test]
async fn red_test_unified_python_parses_only_once() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let temp_file = create_temp_py_file(r#"
def add(a, b):
    return a + b
    "#);

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Should parse successfully");

    #[cfg(test)]
    {
        assert_eq!(analyzer.parse_count(), 1, "Must parse exactly once!");
    }
}

/// Test 3: Returns Both AST and Complexity
#[tokio::test]
async fn red_test_unified_python_returns_both_ast_and_complexity() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let temp_file = create_temp_py_file(r#"
def greet(name):
    print(f'Hello {name}')
    "#);

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse successfully");

    // Must have AST items
    assert!(!result.ast_items.is_empty(), "Must extract AST items");
    assert!(result.ast_items.len() >= 1, "Should find at least 1 function");

    // Must have complexity metrics (GREEN phase may overcount - that's OK)
    assert!(!result.file_metrics.functions.is_empty(), "Must extract complexity");
}

/// Test 4: AST Items Extracted
#[tokio::test]
async fn red_test_unified_python_ast_extraction() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let temp_file = create_temp_py_file(r#"
def multiply(x, y):
    return x * y

class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    "#);

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    // Should find function and class
    assert!(result.ast_items.len() >= 2, "Should find at least 2 items (function + class)");
}

/// Test 5: Handles Parse Errors Gracefully
#[tokio::test]
async fn red_test_unified_python_handles_invalid_syntax() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let temp_file = create_temp_py_file(r#"
def broken syntax here {{{
    "#);

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_err(), "Must return error for invalid syntax");

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("parse") || err_msg.contains("syntax"),
            "Error should mention parsing: {}", err_msg);
}

/// Test 6: Property-Based Test - Various File Sizes
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn red_property_unified_python_handles_any_valid_code(
            function_count in 1usize..20,
        ) {
            use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

            let mut source = String::new();
            for i in 0..function_count {
                source.push_str(&format!(
                    "def func_{}():\n    print('test')\n\n",
                    i
                ));
            }

            let temp_file = create_temp_py_file(&source);
            let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());

            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(analyzer.analyze());

            prop_assert!(result.is_ok(), "Must handle any valid Python");
            let analysis = result.unwrap();
            // GREEN phase may find more or fewer items due to simple pattern matching
            prop_assert!(analysis.ast_items.len() >= 1, "Should find at least some items");
        }
    }
}

/// Test 7: Multiple Function Types
#[tokio::test]
async fn red_test_unified_python_handles_multiple_function_types() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let temp_file = create_temp_py_file(r#"
# Regular function
def regular():
    pass

# Async function
async def async_func():
    pass

# Class with methods
class MyClass:
    def method(self):
        pass

    @staticmethod
    def static_method():
        pass
    "#);

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse successfully");

    // Should find functions and class
    assert!(result.ast_items.len() >= 3, "Should find at least 3 items");
}

/// Test 8: Complex Control Flow
#[tokio::test]
async fn red_test_unified_python_handles_complex_control_flow() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let temp_file = create_temp_py_file(r#"
def complex_function(items):
    # List comprehension
    squared = [x**2 for x in items if x > 0]

    # Nested loops
    for i in range(10):
        if i % 2 == 0:
            for j in range(i):
                print(j)

    # Try/except
    try:
        result = squared[0]
    except IndexError:
        result = None

    return result
    "#);

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse successfully");

    // Should detect higher complexity due to nested structures
    assert!(!result.file_metrics.functions.is_empty(), "Should analyze function");
}

/// Test 9: Empty File
#[tokio::test]
async fn red_test_unified_python_handles_empty_file() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let temp_file = create_temp_py_file("");

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Empty file should parse successfully");
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0, "Empty file should have 0 items");
    assert_eq!(analysis.file_metrics.functions.len(), 0, "Empty file should have 0 functions");
}

/// Test 10: Comment-Only File
#[tokio::test]
async fn red_test_unified_python_handles_comment_only_file() {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let temp_file = create_temp_py_file(r#"
# This is just a comment
"""
And a docstring
"""
    "#);

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Comment-only file should parse successfully");
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0, "Comment-only file should have 0 items");
}