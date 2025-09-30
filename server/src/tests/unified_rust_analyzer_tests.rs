//! EXTREME TDD Tests for Unified Rust Analyzer
//!
//! These tests are written FIRST (RED phase) and must ALL FAIL initially.
//! Once implementation is complete, they must ALL PASS.

use std::path::PathBuf;
use tempfile::NamedTempFile;
use std::io::Write;

// Helper to create temporary Rust files for testing
fn create_temp_rust_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp_file, "{}", content).expect("Failed to write to temp file");
    temp_file
}

/// Test 1: Basic Structure
/// Expected: ❌ FAIL - UnifiedRustAnalyzer doesn't exist yet
#[test]
fn red_test_unified_analyzer_can_be_created() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

    let path = PathBuf::from("test.rs");
    let analyzer = UnifiedRustAnalyzer::new(path.clone());
    assert_eq!(analyzer.file_path(), &path);
}

/// Test 2: Single Parse Guarantee
/// Expected: ❌ FAIL - No analyze() method yet
#[tokio::test]
async fn red_test_unified_analyzer_parses_only_once() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

    let temp_file = create_temp_rust_file(r#"
        fn main() {
            println!("Hello, world!");
        }
    "#);

    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
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
async fn red_test_unified_analyzer_returns_both_ast_and_complexity() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

    let temp_file = create_temp_rust_file(r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#);

    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse successfully");

    // Must have AST items
    assert!(!result.ast_items.is_empty(), "Must extract AST items");
    assert_eq!(result.ast_items.len(), 1, "Should find 1 function");

    // Must have complexity metrics
    assert!(!result.file_metrics.functions.is_empty(), "Must extract complexity");
    assert_eq!(result.file_metrics.functions.len(), 1, "Should analyze 1 function");
}

/// Test 4: AST Items Match EnhancedAstVisitor
/// Expected: ❌ FAIL - Output won't match yet
#[tokio::test]
async fn red_test_unified_ast_matches_enhanced_visitor() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;
    use crate::services::enhanced_ast_visitor::EnhancedAstVisitor;

    let temp_file = create_temp_rust_file(r#"
        pub fn multiply(x: i32, y: i32) -> i32 {
            x * y
        }

        struct Point {
            x: i32,
            y: i32,
        }
    "#);

    // OLD WAY: EnhancedAstVisitor
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let syntax_tree = syn::parse_file(&content).unwrap();
    let visitor = EnhancedAstVisitor::new(temp_file.path());
    let old_items = visitor.extract_items(&syntax_tree);

    // NEW WAY: UnifiedRustAnalyzer
    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();
    let new_items = result.ast_items;

    // Must be identical
    assert_eq!(old_items.len(), new_items.len(), "Same number of items");

    // Compare each item
    for (old, new) in old_items.iter().zip(new_items.iter()) {
        assert_eq!(old, new, "AST items must match exactly");
    }
}

/// Test 5: Handles Parse Errors Gracefully
/// Expected: ❌ FAIL - No error handling yet
#[tokio::test]
async fn red_test_unified_analyzer_handles_invalid_syntax() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

    let temp_file = create_temp_rust_file(r#"
        fn broken syntax here {{{
    "#);

    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
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
        fn red_property_unified_analyzer_handles_any_valid_rust(
            function_count in 1usize..20,
        ) {
            use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let mut source = String::new();
                for i in 0..function_count {
                    source.push_str(&format!(
                        "fn func_{}() {{ println!(\"test\"); }}\n",
                        i
                    ));
                }

                let temp_file = create_temp_rust_file(&source);
                let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
                let result = analyzer.analyze().await;

                prop_assert!(result.is_ok(), "Must handle any valid Rust");
                let analysis = result.unwrap();
                prop_assert_eq!(analysis.ast_items.len(), function_count);
            });

            Ok(())
        }
    }
}

/// Test 7: Integration Test - Real World File
/// Expected: ❌ FAIL - Will fail until implementation
#[tokio::test]
async fn red_test_unified_analyzer_on_real_file() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

    // Use actual file from our codebase
    let real_file = PathBuf::from("server/src/services/context.rs");

    if !real_file.exists() {
        // Skip if file doesn't exist (e.g., running from different directory)
        return;
    }

    let analyzer = UnifiedRustAnalyzer::new(real_file);
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Must handle real-world files");
    let analysis = result.unwrap();

    // Context.rs has many functions
    assert!(analysis.ast_items.len() > 10, "Should find many items, found: {}", analysis.ast_items.len());
    assert!(analysis.file_metrics.functions.len() > 10, "Should analyze many functions, found: {}", analysis.file_metrics.functions.len());
}

/// Test 8: Multiple Function Types
/// Expected: ❌ FAIL - Not implemented yet
#[tokio::test]
async fn red_test_unified_analyzer_handles_multiple_function_types() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

    let temp_file = create_temp_rust_file(r#"
        // Regular function
        fn regular_function() {}

        // Async function
        async fn async_function() {}

        // Method in impl block
        struct MyStruct;
        impl MyStruct {
            fn method(&self) {}
        }

        // Trait method
        trait MyTrait {
            fn trait_method(&self);
        }
    "#);

    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.expect("Should parse successfully");

    // Should find all function types
    assert!(result.ast_items.len() >= 4, "Should find at least 4 items (functions, struct, trait)");
}

/// Test 9: Empty File
/// Expected: ❌ FAIL - Edge case not handled
#[tokio::test]
async fn red_test_unified_analyzer_handles_empty_file() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

    let temp_file = create_temp_rust_file("");

    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Empty file should parse successfully");
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0, "Empty file should have 0 items");
    assert_eq!(analysis.file_metrics.functions.len(), 0, "Empty file should have 0 functions");
}

/// Test 10: File With Only Comments
/// Expected: ❌ FAIL - Edge case not handled
#[tokio::test]
async fn red_test_unified_analyzer_handles_comment_only_file() {
    use crate::services::unified_rust_analyzer::UnifiedRustAnalyzer;

    let temp_file = create_temp_rust_file(r#"
        // This is just a comment
        /* And a block comment */
        //! Doc comment
    "#);

    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Comment-only file should parse successfully");
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0, "Comment-only file should have 0 items");
}