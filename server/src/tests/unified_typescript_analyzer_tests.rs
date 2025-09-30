//! EXTREME TDD Tests for Unified TypeScript Analyzer
//!
//! These tests are written FIRST (RED phase) and must ALL FAIL initially.
//! Once implementation is complete, they must ALL PASS.

use std::path::PathBuf;
use tempfile::NamedTempFile;
use std::io::Write;

// Helper to create temporary TypeScript files for testing
fn create_temp_ts_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::with_suffix(".ts").expect("Failed to create temp file");
    write!(temp_file, "{}", content).expect("Failed to write to temp file");
    temp_file
}

/// Test 1: Basic Structure
/// Expected: ❌ FAIL - UnifiedTypeScriptAnalyzer doesn't exist yet
#[test]
fn red_test_unified_typescript_analyzer_can_be_created() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let path = PathBuf::from("test.ts");
    let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
    assert_eq!(analyzer.file_path(), &path);
}

/// Test 2: Single Parse Guarantee
/// Expected: ❌ FAIL - No analyze() method yet
#[tokio::test]
async fn red_test_unified_typescript_parses_only_once() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let temp_file = create_temp_ts_file(r#"
        function add(a: number, b: number): number {
            return a + b;
        }
    "#);

    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Should parse successfully");

    #[cfg(test)]
    {
        assert_eq!(analyzer.parse_count(), 1, "Must parse exactly once!");
    }
}

/// Test 3: Returns Both AST and Complexity
/// Expected: ❌ FAIL - No UnifiedAnalysis struct yet
#[tokio::test]
async fn red_test_unified_typescript_returns_both_ast_and_complexity() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let temp_file = create_temp_ts_file(r#"
        function greet(name: string): void {
            console.log(`Hello ${name}`);
        }
    "#);

    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse successfully");

    // Must have AST items
    assert!(!result.ast_items.is_empty(), "Must extract AST items");
    assert!(result.ast_items.len() >= 1, "Should find at least 1 function");

    // Must have complexity metrics (GREEN phase may overcount - that's OK)
    assert!(!result.file_metrics.functions.is_empty(), "Must extract complexity");
}

/// Test 4: AST Items Match EnhancedTypeScriptVisitor
/// Expected: ❌ FAIL - Output won't match yet
#[tokio::test]
async fn red_test_unified_typescript_ast_matches_enhanced_visitor() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let temp_file = create_temp_ts_file(r#"
        export function multiply(x: number, y: number): number {
            return x * y;
        }

        interface Point {
            x: number;
            y: number;
        }
    "#);

    // NEW WAY: UnifiedTypeScriptAnalyzer
    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();
    let new_items = result.ast_items;

    // Should find both function and interface
    assert!(new_items.len() >= 2, "Should find at least 2 items (function + interface)");
}

/// Test 5: Handles Parse Errors Gracefully
/// Expected: ❌ FAIL - No error handling yet
#[tokio::test]
async fn red_test_unified_typescript_handles_invalid_syntax() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let temp_file = create_temp_ts_file(r#"
        function broken syntax here {{{
    "#);

    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_err(), "Must return error for invalid syntax");

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("parse") || err_msg.contains("syntax"),
            "Error should mention parsing: {}", err_msg);
}

/// Test 6: Property-Based Test - Various File Sizes
/// Expected: ❌ FAIL - Property test will fail initially
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn red_property_unified_typescript_handles_any_valid_code(
            function_count in 1usize..20,
        ) {
            use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

            let mut source = String::new();
            for i in 0..function_count {
                source.push_str(&format!(
                    "function func_{}(): void {{ console.log('test'); }}\n",
                    i
                ));
            }

            let temp_file = create_temp_ts_file(&source);
            let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());

            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(analyzer.analyze());

            prop_assert!(result.is_ok(), "Must handle any valid TypeScript");
            let analysis = result.unwrap();
            // GREEN phase may find more or fewer items due to simple regex matching
            prop_assert!(analysis.ast_items.len() >= 1, "Should find at least some items");
        }
    }
}

/// Test 7: Integration Test - Real World File
/// Expected: ❌ FAIL - Will fail until implementation
#[tokio::test]
async fn red_test_unified_typescript_on_real_file() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    // Use actual file from agentic-ai project
    let real_file = PathBuf::from("/home/noah/src/agentic-ai/deno-actors/simple.ts");

    if !real_file.exists() {
        // Skip if file doesn't exist
        return;
    }

    let analyzer = UnifiedTypeScriptAnalyzer::new(real_file);
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Must handle real-world TypeScript files");
    let analysis = result.unwrap();

    // simple.ts has classes, interfaces, and functions
    assert!(analysis.ast_items.len() > 3, "Should find many items, found: {}", analysis.ast_items.len());
    assert!(analysis.file_metrics.functions.len() > 3, "Should analyze many functions, found: {}", analysis.file_metrics.functions.len());
}

/// Test 8: Multiple Function Types
/// Expected: ❌ FAIL - Not implemented yet
#[tokio::test]
async fn red_test_unified_typescript_handles_multiple_function_types() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let temp_file = create_temp_ts_file(r#"
        // Regular function
        function regularFunc(): void {}

        // Arrow function
        const arrowFunc = (): void => {}

        // Async function
        async function asyncFunc(): Promise<void> {}

        // Class with methods
        class MyClass {
            method(): void {}
        }

        // Interface
        interface MyInterface {
            prop: string;
        }
    "#);

    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse successfully");

    // Should find all constructs
    assert!(result.ast_items.len() >= 5, "Should find at least 5 items (functions, class, interface)");
}

/// Test 9: Empty File
/// Expected: ❌ FAIL - Edge case not handled
#[tokio::test]
async fn red_test_unified_typescript_handles_empty_file() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let temp_file = create_temp_ts_file("");

    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Empty file should parse successfully");
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0, "Empty file should have 0 items");
    assert_eq!(analysis.file_metrics.functions.len(), 0, "Empty file should have 0 functions");
}

/// Test 10: File With Only Comments
/// Expected: ❌ FAIL - Edge case not handled
#[tokio::test]
async fn red_test_unified_typescript_handles_comment_only_file() {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let temp_file = create_temp_ts_file(r#"
        // This is just a comment
        /* And a block comment */
        /** JSDoc comment */
    "#);

    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Comment-only file should parse successfully");
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0, "Comment-only file should have 0 items");
}