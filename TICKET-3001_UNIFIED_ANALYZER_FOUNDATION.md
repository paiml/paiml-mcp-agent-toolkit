# TICKET-3001: Unified Analyzer Foundation

**Sprint**: 12 - Unified AST+Complexity Parser
**Priority**: High
**Estimated Time**: 2 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD

## Objective

Create the foundational structure for `UnifiedRustAnalyzer` that will parse files once and extract both AST items and complexity metrics.

## Problem Statement

Currently, every Rust file is parsed twice:
1. `analyze_rust_file()` → `syn::parse_file()` for AST
2. `analyze_rust_file_with_complexity()` → `syn::parse_file()` again for complexity

This ticket creates the unified structure to eliminate double parsing.

## Technical Requirements

### Must Implement

1. **New Module**: `server/src/services/unified_rust_analyzer.rs`
2. **Core Struct**: `UnifiedRustAnalyzer`
3. **Result Type**: `UnifiedAnalysis`
4. **Single Parse Guarantee**: Must prove only one `syn::parse_file()` call

### API Design

```rust
/// Unified analyzer that parses once, extracts twice
pub struct UnifiedRustAnalyzer {
    file_path: PathBuf,
}

/// Combined result from unified analysis
pub struct UnifiedAnalysis {
    /// AST items (functions, structs, enums, traits)
    pub ast_items: Vec<AstItem>,

    /// File-level complexity metrics
    pub file_metrics: FileComplexityMetrics,

    /// Parse timestamp (for cache validation)
    pub parsed_at: std::time::Instant,
}

impl UnifiedRustAnalyzer {
    /// Create new analyzer for a file
    pub fn new(file_path: PathBuf) -> Self;

    /// Analyze file with single parse
    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError>;

    /// Get parse count (for testing - must be 1)
    #[cfg(test)]
    pub fn parse_count(&self) -> usize;
}
```

## EXTREME TDD: RED Phase Tests

All tests must be written FIRST and must FAIL initially.

### Test 1: Basic Structure

```rust
#[test]
fn red_test_unified_analyzer_can_be_created() {
    let path = PathBuf::from("test.rs");
    let analyzer = UnifiedRustAnalyzer::new(path);
    assert!(analyzer.file_path.to_str().unwrap().ends_with("test.rs"));
}
```

**Expected**: ❌ FAIL - `UnifiedRustAnalyzer` doesn't exist yet

### Test 2: Single Parse Guarantee

```rust
#[tokio::test]
async fn red_test_unified_analyzer_parses_only_once() {
    let temp_file = create_temp_rust_file(r#"
        fn main() {
            println!("Hello, world!");
        }
    "#);

    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok());
    assert_eq!(analyzer.parse_count(), 1, "Must parse exactly once!");
}
```

**Expected**: ❌ FAIL - No analyze() method yet

### Test 3: Returns Both AST and Complexity

```rust
#[tokio::test]
async fn red_test_unified_analyzer_returns_both_ast_and_complexity() {
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
```

**Expected**: ❌ FAIL - No UnifiedAnalysis struct yet

### Test 4: AST Items Match EnhancedAstVisitor

```rust
#[tokio::test]
async fn red_test_unified_ast_matches_enhanced_visitor() {
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
    assert_eq!(old_items, new_items, "AST items must match exactly");
}
```

**Expected**: ❌ FAIL - Output won't match yet

### Test 5: Handles Parse Errors Gracefully

```rust
#[tokio::test]
async fn red_test_unified_analyzer_handles_invalid_syntax() {
    let temp_file = create_temp_rust_file(r#"
        fn broken syntax here {{{
    "#);

    let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_err(), "Must return error for invalid syntax");
}
```

**Expected**: ❌ FAIL - No error handling yet

### Test 6: Property-Based Test - Various File Sizes

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn red_property_unified_analyzer_handles_any_valid_rust(
        function_count in 1usize..20,
    ) {
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
```

**Expected**: ❌ FAIL - Property test will fail initially

### Test 7: Integration Test - Real World File

```rust
#[tokio::test]
async fn red_test_unified_analyzer_on_real_file() {
    // Use actual file from our codebase
    let real_file = PathBuf::from("server/src/services/context.rs");

    if !real_file.exists() {
        return; // Skip if file doesn't exist
    }

    let analyzer = UnifiedRustAnalyzer::new(real_file);
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Must handle real-world files");
    let analysis = result.unwrap();

    // Context.rs has many functions
    assert!(analysis.ast_items.len() > 10, "Should find many items");
    assert!(analysis.file_metrics.functions.len() > 10, "Should analyze many functions");
}
```

**Expected**: ❌ FAIL - Will fail until implementation

## Implementation Checklist

### RED Phase (Write Tests First)
- [ ] Create `server/src/tests/unified_rust_analyzer_tests.rs`
- [ ] Write all 7 RED tests above
- [ ] Run `cargo test unified_rust_analyzer` - ALL MUST FAIL
- [ ] Verify failure messages are clear

### GREEN Phase (Make Tests Pass)
- [ ] Create `server/src/services/unified_rust_analyzer.rs`
- [ ] Implement `UnifiedRustAnalyzer` struct
- [ ] Implement `UnifiedAnalysis` struct
- [ ] Implement `new()` method
- [ ] Implement `analyze()` method (minimal, just to pass)
- [ ] Run tests - ALL MUST PASS

### REFACTOR Phase (Improve Code)
- [ ] Add error handling with proper types
- [ ] Add documentation comments
- [ ] Extract helper functions
- [ ] Optimize parse_count tracking
- [ ] Run tests - MUST STAY GREEN

## Acceptance Criteria

### Must Have
- [x] All 7 RED tests written and failing
- [ ] All 7 tests passing after implementation
- [ ] Parse happens exactly once per file (measured)
- [ ] No regression in existing tests

### Should Have
- [ ] Parse count accessible for testing
- [ ] Clear error messages
- [ ] Module documented with examples

### Nice to Have
- [ ] Additional property-based tests
- [ ] Benchmark showing parse is O(1) per file

## Definition of Done

1. ✅ All RED phase tests written
2. ✅ All tests initially failing
3. ✅ GREEN phase implementation complete
4. ✅ All tests now passing (100%)
5. ✅ Code reviewed for quality
6. ✅ Documentation complete
7. ✅ No regression in existing tests
8. ✅ Ready for TICKET-3002

## Dependencies

- **Requires**: syn >= 2.0
- **Requires**: tokio fs
- **Blocks**: TICKET-3002 (Complexity Visitor)

## Notes

This is PURE FOUNDATION. We're not extracting complexity yet - that's TICKET-3002.
This ticket just proves we can parse once and have a structure ready for dual extraction.

The parse_count() method is test-only infrastructure to PROVE we're parsing once.

---

**Status**: Ready to Start
**Blocked By**: None
**Blocks**: TICKET-3002, TICKET-3003, TICKET-3004