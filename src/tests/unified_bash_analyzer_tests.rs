//! EXTREME TDD Tests for Unified Bash/Shell Analyzer
//!
//! These tests are written FIRST (RED phase) and must ALL FAIL initially.
//! Once implementation is complete, they must ALL PASS.

use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

// Helper to create temporary shell files for testing
fn create_temp_shell_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::with_suffix(".sh").expect("Failed to create temp file");
    write!(temp_file, "{}", content).expect("Failed to write to temp file");
    temp_file
}

/// Test 1: Basic Structure
#[test]
fn red_test_unified_bash_analyzer_can_be_created() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let path = PathBuf::from("test.sh");
    let analyzer = UnifiedBashAnalyzer::new(path.clone());
    assert_eq!(analyzer.file_path(), &path);
}

/// Test 2: Single Parse Guarantee
#[tokio::test]
async fn red_test_unified_bash_parses_only_once() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let temp_file = create_temp_shell_file(
        r#"#!/bin/bash

function greet() {
    echo "Hello, $1"
}

greet "World"
    "#,
    );

    let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Should parse successfully");

    #[cfg(test)]
    {
        assert_eq!(analyzer.parse_count(), 1, "Must parse exactly once!");
    }
}

/// Test 3: Returns Both AST and Complexity
#[tokio::test]
async fn red_test_unified_bash_returns_both_ast_and_complexity() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let temp_file = create_temp_shell_file(
        r#"#!/bin/bash

function add() {
    echo $(($1 + $2))
}

add 3 5
    "#,
    );

    let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse successfully");

    // Must have AST items
    assert!(!result.ast_items.is_empty(), "Must extract AST items");
    assert!(
        !result.ast_items.is_empty(),
        "Should find at least 1 function"
    );

    // Must have complexity metrics
    assert!(
        !result.file_metrics.functions.is_empty(),
        "Must extract complexity"
    );
}

/// Test 4: AST Items Extracted
#[tokio::test]
async fn red_test_unified_bash_ast_extraction() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let temp_file = create_temp_shell_file(
        r#"#!/bin/bash

multiply() {
    echo $(($1 * $2))
}

divide() {
    echo $(($1 / $2))
}
    "#,
    );

    let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    // Should find 2 functions
    assert!(
        result.ast_items.len() >= 2,
        "Should find at least 2 functions"
    );
}

/// Test 5: Handles Invalid Syntax Gracefully
#[tokio::test]
async fn red_test_unified_bash_handles_invalid_syntax() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let temp_file = create_temp_shell_file(
        r#"
broken shell syntax {{{ !!!
    "#,
    );

    let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    // GREEN PHASE: Pattern-based extraction handles invalid syntax gracefully
    assert!(
        result.is_ok(),
        "Shell analyzer handles invalid syntax gracefully"
    );
}

/// Test 6: Complexity Increases with Control Flow
#[tokio::test]
async fn red_test_bash_complexity_reflects_control_flow() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let simple_script = create_temp_shell_file(
        r#"#!/bin/bash
echo "simple"
    "#,
    );

    let complex_script = create_temp_shell_file(
        r#"#!/bin/bash

if [ "$1" = "test" ]; then
    echo "test mode"
else
    echo "normal mode"
fi

for i in {1..5}; do
    echo $i
done
    "#,
    );

    let simple_analyzer = UnifiedBashAnalyzer::new(simple_script.path().to_path_buf());
    let complex_analyzer = UnifiedBashAnalyzer::new(complex_script.path().to_path_buf());

    let simple_result = simple_analyzer
        .analyze()
        .await
        .expect("Should parse simple");
    let complex_result = complex_analyzer
        .analyze()
        .await
        .expect("Should parse complex");

    let simple_complexity = simple_result.file_metrics.total_complexity.cyclomatic;
    let complex_complexity = complex_result.file_metrics.total_complexity.cyclomatic;

    // Complex script with if/else and loop should have higher complexity
    assert!(
        complex_complexity > simple_complexity,
        "Complex script should have higher complexity: {} > {}",
        complex_complexity,
        simple_complexity
    );
}

/// Test 7: Property-Based Test - Various Function Counts
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn red_property_unified_bash_handles_multiple_functions(
            function_count in 1usize..10,
        ) {
            use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

            let mut source = String::from("#!/bin/bash\n\n");
            for i in 0..function_count {
                source.push_str(&format!(
                    "func_{}() {{\n    echo \"test\"\n}}\n\n",
                    i
                ));
            }

            let temp_file = create_temp_shell_file(&source);
            let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());

            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(analyzer.analyze());

            prop_assert!(result.is_ok(), "Must handle any valid shell script");
            let analysis = result.unwrap();
            prop_assert!(!analysis.ast_items.is_empty(), "Should find functions");
        }
    }
}

/// Test 8: Empty Script
#[tokio::test]
async fn red_test_unified_bash_handles_empty_script() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let temp_file = create_temp_shell_file("#!/bin/bash\n");

    let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Empty script should parse successfully");
    let analysis = result.unwrap();
    assert_eq!(
        analysis.ast_items.len(),
        0,
        "Empty script should have 0 items"
    );
}

/// Test 9: Pipeline Complexity
#[tokio::test]
async fn red_test_bash_pipeline_increases_complexity() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let temp_file = create_temp_shell_file(
        r#"#!/bin/bash

cat file.txt | grep "pattern" | sort | uniq | wc -l
    "#,
    );

    let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse pipeline");

    // Pipeline with multiple commands should increase complexity
    assert!(
        result.file_metrics.total_complexity.cyclomatic >= 2,
        "Pipeline should increase complexity"
    );
}

/// Test 10: Multiple Control Flow Constructs
#[tokio::test]
async fn red_test_bash_multiple_control_flow() {
    use crate::services::unified_bash_analyzer::UnifiedBashAnalyzer;

    let temp_file = create_temp_shell_file(
        r#"#!/bin/bash

function complex_logic() {
    if [ "$1" = "start" ]; then
        while [ $count -lt 10 ]; do
            if [ $((count % 2)) -eq 0 ]; then
                echo "even"
            fi
            count=$((count + 1))
        done
    fi
}
    "#,
    );

    let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer
        .analyze()
        .await
        .expect("Should parse nested control flow");

    // Nested if/while statements should significantly increase complexity
    assert!(
        result.file_metrics.total_complexity.cyclomatic >= 4,
        "Nested control flow should have complexity >= 4"
    );
}
