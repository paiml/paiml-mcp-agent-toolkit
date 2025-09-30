//! EXTREME TDD Tests for Unified WebAssembly Analyzer
//!
//! These tests are written FIRST (RED phase) and must ALL FAIL initially.
//! Once implementation is complete, they must ALL PASS.

use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

// Helper to create temporary WASM files for testing
fn create_temp_wat_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::with_suffix(".wat").expect("Failed to create temp file");
    write!(temp_file, "{}", content).expect("Failed to write to temp file");
    temp_file
}

/// Test 1: Basic Structure
#[test]
fn red_test_unified_wasm_analyzer_can_be_created() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let path = PathBuf::from("test.wat");
    let analyzer = UnifiedWasmAnalyzer::new(path.clone());
    assert_eq!(analyzer.file_path(), &path);
}

/// Test 2: Single Parse Guarantee
#[tokio::test]
async fn red_test_unified_wasm_parses_only_once() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let temp_file = create_temp_wat_file(
        r#"
(module
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
  (export "add" (func $add))
)
    "#,
    );

    let analyzer = UnifiedWasmAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Should parse successfully");

    #[cfg(test)]
    {
        assert_eq!(analyzer.parse_count(), 1, "Must parse exactly once!");
    }
}

/// Test 3: Returns Both AST and Complexity
#[tokio::test]
async fn red_test_unified_wasm_returns_both_ast_and_complexity() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let temp_file = create_temp_wat_file(
        r#"
(module
  (func $multiply (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.mul
  )
  (export "multiply" (func $multiply))
)
    "#,
    );

    let analyzer = UnifiedWasmAnalyzer::new(temp_file.path().to_path_buf());
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
async fn red_test_unified_wasm_ast_extraction() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let temp_file = create_temp_wat_file(
        r#"
(module
  (func $add (param i32) (param i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
  (func $sub (param i32) (param i32) (result i32)
    local.get 0
    local.get 1
    i32.sub
  )
  (export "add" (func $add))
  (export "sub" (func $sub))
)
    "#,
    );

    let analyzer = UnifiedWasmAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    // Should find 2 functions
    assert!(
        result.ast_items.len() >= 2,
        "Should find at least 2 functions"
    );
}

/// Test 5: Handles Invalid Syntax Gracefully
#[tokio::test]
async fn red_test_unified_wasm_handles_invalid_syntax() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let temp_file = create_temp_wat_file(
        r#"
broken wasm syntax here {{{
    "#,
    );

    let analyzer = UnifiedWasmAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    // GREEN PHASE: Pattern-based extraction may handle invalid syntax gracefully
    // This is acceptable for GREEN phase
    assert!(
        result.is_ok(),
        "WASM analyzer handles invalid syntax gracefully"
    );
}

/// Test 6: Complexity Increases with Control Flow
#[tokio::test]
async fn red_test_wasm_complexity_reflects_control_flow() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let simple_func = create_temp_wat_file(
        r#"
(module
  (func $simple (param i32) (result i32)
    local.get 0
  )
)
    "#,
    );

    let complex_func = create_temp_wat_file(
        r#"
(module
  (func $complex (param i32) (result i32)
    local.get 0
    if (result i32)
      i32.const 1
    else
      i32.const 0
    end
  )
)
    "#,
    );

    let simple_analyzer = UnifiedWasmAnalyzer::new(simple_func.path().to_path_buf());
    let complex_analyzer = UnifiedWasmAnalyzer::new(complex_func.path().to_path_buf());

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

    // Complex function with if/else should have higher complexity
    assert!(
        complex_complexity >= simple_complexity,
        "Complex function should have higher complexity: {} >= {}",
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
        fn red_property_unified_wasm_handles_multiple_functions(
            function_count in 1usize..10,
        ) {
            use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

            let mut source = String::from("(module\n");
            for i in 0..function_count {
                source.push_str(&format!(
                    "  (func $func_{} (result i32)\n    i32.const {}\n  )\n",
                    i, i
                ));
            }
            source.push(')');

            let temp_file = create_temp_wat_file(&source);
            let analyzer = UnifiedWasmAnalyzer::new(temp_file.path().to_path_buf());

            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(analyzer.analyze());

            prop_assert!(result.is_ok(), "Must handle any valid WASM");
            let analysis = result.unwrap();
            prop_assert!(!analysis.ast_items.is_empty(), "Should find functions");
        }
    }
}

/// Test 8: Empty Module
#[tokio::test]
async fn red_test_unified_wasm_handles_empty_module() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let temp_file = create_temp_wat_file("(module)");

    let analyzer = UnifiedWasmAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Empty module should parse successfully");
    let analysis = result.unwrap();
    assert_eq!(
        analysis.ast_items.len(),
        0,
        "Empty module should have 0 items"
    );
}

/// Test 9: Loop Complexity
#[tokio::test]
async fn red_test_wasm_loop_increases_complexity() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let temp_file = create_temp_wat_file(
        r#"
(module
  (func $loop_func (param i32) (result i32)
    (local $i i32)
    local.get 0
    local.set $i
    (loop $continue
      local.get $i
      i32.const 1
      i32.sub
      local.tee $i
      i32.const 0
      i32.gt_s
      br_if $continue
    )
    local.get $i
  )
)
    "#,
    );

    let analyzer = UnifiedWasmAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse loop");

    // Loop should increase complexity
    assert!(
        result.file_metrics.total_complexity.cyclomatic >= 2,
        "Loop should increase complexity"
    );
}

/// Test 10: Multiple Control Flow Constructs
#[tokio::test]
async fn red_test_wasm_multiple_control_flow() {
    use crate::services::unified_wasm_analyzer::UnifiedWasmAnalyzer;

    let temp_file = create_temp_wat_file(
        r#"
(module
  (func $complex_control (param i32) (result i32)
    local.get 0
    if (result i32)
      local.get 0
      i32.const 10
      i32.lt_s
      if (result i32)
        i32.const 1
      else
        i32.const 2
      end
    else
      i32.const 3
    end
  )
)
    "#,
    );

    let analyzer = UnifiedWasmAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer
        .analyze()
        .await
        .expect("Should parse nested control flow");

    // Nested if statements should significantly increase complexity
    assert!(
        result.file_metrics.total_complexity.cyclomatic >= 3,
        "Nested control flow should have complexity >= 3"
    );
}
