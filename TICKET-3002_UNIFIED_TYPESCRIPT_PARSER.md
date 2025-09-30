# TICKET-3002: Unified TypeScript/JavaScript Parser

**Sprint**: 13 - Multi-Language Unified Parser Extension
**Priority**: High
**Estimated Effort**: 3 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD
**Parent**: Sprint 12 - Unified Parser Architecture

## Problem Statement

TypeScript and JavaScript files are currently parsed TWICE in `pmat context`:
1. **AST Analysis Pass**: Extracts functions, classes, interfaces
2. **Complexity Analysis Pass**: Calculates cyclomatic and cognitive complexity

This doubles the parsing time for TypeScript/JavaScript files. Following the success of TICKET-3001 (Rust unified parser), we need to extend the unified parser architecture to TypeScript/JavaScript.

### Current Architecture Issues

**Double Parsing in deep_context.rs:**
```rust
// Pass 1: AST extraction
"typescript" | "javascript" => analyze_typescript_language(file_path).await

// Pass 2: Complexity analysis (SEPARATE PARSE!)
"ts" | "js" | "jsx" | "tsx" => analyze_typescript_file_with_complexity(file_path).await
```

Both functions parse the file separately using SWC parser or tree-sitter.

### Performance Impact

- **Redundant I/O**: Reading file twice from disk
- **Redundant Parsing**: SWC/tree-sitter parsing is expensive
- **Memory Overhead**: Two separate AST representations
- **Expected Gain**: 40-50% reduction in TypeScript/JavaScript parse time

### Validation Project

Test on `/home/noah/src/agentic-ai` which contains:
- `deno-actors/simple.ts` - TypeScript with classes, methods, async functions
- `deno-actors/simple_test.ts` - TypeScript test file

## Goal

Create `UnifiedTypeScriptAnalyzer` that parses TypeScript/JavaScript files ONCE and extracts both AST items and complexity metrics from a single parse pass.

## Implementation Strategy

### Phase 1: Foundation (1.5 hours)

**Deliverables:**
- `server/src/services/unified_typescript_analyzer.rs` - New module
- `UnifiedTypeScriptAnalyzer` struct with single-pass architecture
- `UnifiedAnalysis` result type (reuse from Rust or create shared trait)
- 10+ EXTREME TDD tests (all must fail initially)

**API Design:**
```rust
pub struct UnifiedTypeScriptAnalyzer {
    file_path: PathBuf,
    #[cfg(test)]
    parse_count: AtomicUsize, // Verify single parse
}

pub struct UnifiedAnalysis {
    pub ast_items: Vec<AstItem>,
    pub file_metrics: FileComplexityMetrics,
    pub parsed_at: std::time::Instant,
}

impl UnifiedTypeScriptAnalyzer {
    pub fn new(file_path: PathBuf) -> Self;

    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError>;

    #[cfg(test)]
    pub fn parse_count(&self) -> usize;
}
```

**RED Phase Tests:**
```rust
// Test 1: Basic structure
#[test]
fn red_test_unified_typescript_analyzer_can_be_created() {
    let path = PathBuf::from("test.ts");
    let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
    assert_eq!(analyzer.file_path(), path.as_path());
}

// Test 2: Single parse guarantee
#[tokio::test]
async fn red_test_unified_typescript_parses_only_once() {
    let temp_file = create_temp_ts_file("function add(a: number, b: number) { return a + b; }");
    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let _ = analyzer.analyze().await;
    assert_eq!(analyzer.parse_count(), 1, "Must parse exactly once!");
}

// Test 3: Returns both AST and complexity
#[tokio::test]
async fn red_test_unified_typescript_returns_both_ast_and_complexity() {
    let temp_file = create_temp_ts_file("function greet(name: string) { console.log('Hello ' + name); }");
    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    assert!(!result.ast_items.is_empty(), "Must extract AST items");
    assert!(!result.file_metrics.functions.is_empty(), "Must extract complexity");
}

// Test 4: AST items match EnhancedTypeScriptVisitor
#[tokio::test]
async fn red_test_unified_typescript_ast_matches_enhanced_visitor() {
    let temp_file = create_temp_ts_file(r#"
        export function multiply(x: number, y: number): number {
            return x * y;
        }

        interface Point {
            x: number;
            y: number;
        }
    "#);

    // OLD WAY: EnhancedTypeScriptVisitor
    let old_items = enhanced_typescript_visitor_extract(&temp_file);

    // NEW WAY: UnifiedTypeScriptAnalyzer
    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    assert_eq!(old_items.len(), result.ast_items.len());
}

// Test 5: Handles invalid TypeScript syntax
#[tokio::test]
async fn red_test_unified_typescript_handles_invalid_syntax() {
    let temp_file = create_temp_ts_file("function broken syntax here {{{");
    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;
    assert!(result.is_err(), "Must return error for invalid syntax");
}

// Test 6: Property test - various file sizes
proptest! {
    #[test]
    fn red_property_unified_typescript_handles_any_valid_code(
        function_count in 1usize..20,
    ) {
        let mut source = String::new();
        for i in 0..function_count {
            source.push_str(&format!("function func_{}() {{ console.log('test'); }}\n", i));
        }

        let temp_file = create_temp_ts_file(&source);
        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(analyzer.analyze());

        prop_assert!(result.is_ok());
        let analysis = result.unwrap();
        prop_assert_eq!(analysis.ast_items.len(), function_count);
    }
}

// Test 7: Real-world file from agentic-ai
#[tokio::test]
async fn red_test_unified_typescript_on_real_file() {
    let real_file = PathBuf::from("/home/noah/src/agentic-ai/deno-actors/simple.ts");
    if !real_file.exists() { return; }

    let analyzer = UnifiedTypeScriptAnalyzer::new(real_file);
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Must handle real-world TypeScript");
    let analysis = result.unwrap();
    assert!(analysis.ast_items.len() > 3, "simple.ts has multiple items");
}

// Test 8: Multiple function types (arrow, async, class methods)
#[tokio::test]
async fn red_test_unified_typescript_handles_multiple_function_types() {
    let temp_file = create_temp_ts_file(r#"
        // Regular function
        function regular() {}

        // Arrow function
        const arrow = () => {}

        // Async function
        async function asyncFunc() {}

        // Class with methods
        class MyClass {
            method() {}
        }

        // Interface
        interface MyInterface {
            prop: string;
        }
    "#);

    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    assert!(result.ast_items.len() >= 5);
}

// Test 9: Empty file
#[tokio::test]
async fn red_test_unified_typescript_handles_empty_file() {
    let temp_file = create_temp_ts_file("");
    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0);
}

// Test 10: Comment-only file
#[tokio::test]
async fn red_test_unified_typescript_handles_comment_only_file() {
    let temp_file = create_temp_ts_file(r#"
        // This is just a comment
        /* And a block comment */
    "#);

    let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0);
}
```

### Phase 2: GREEN Implementation (1 hour)

**Core Implementation:**
```rust
impl UnifiedTypeScriptAnalyzer {
    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError> {
        #[cfg(test)]
        { self.parse_count.fetch_add(1, Ordering::SeqCst); }

        // 1. Read file ONCE
        let content = tokio::fs::read_to_string(&self.file_path).await?;

        // 2. Parse ONCE with SWC or tree-sitter
        let syntax_tree = self.parse_typescript(&content)?;

        // 3. Extract AST items (reuse EnhancedTypeScriptVisitor logic)
        let ast_items = self.extract_ast_items(&syntax_tree);

        // 4. Extract complexity metrics (new SimpleComplexityVisitor)
        let file_metrics = self.extract_complexity_metrics(&syntax_tree);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    fn parse_typescript(&self, content: &str) -> Result<ParsedTree, AnalysisError> {
        // Use existing TypeScript parser (SWC or tree-sitter)
        // Return unified AST representation
    }

    fn extract_ast_items(&self, tree: &ParsedTree) -> Vec<AstItem> {
        // Reuse EnhancedTypeScriptVisitor logic
    }

    fn extract_complexity_metrics(&self, tree: &ParsedTree) -> FileComplexityMetrics {
        // Simple complexity calculation for GREEN phase
        // Count branches: if, switch, while, for, ternary, etc.
    }
}
```

### Phase 3: Integration (0.5 hours)

**Update deep_context.rs:**
```rust
// Add thread-local cache for TypeScript
thread_local! {
    static TYPESCRIPT_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> =
        RefCell::new(FxHashMap::default());
}

// Update analyze_typescript_language
pub async fn analyze_typescript_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    use crate::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;

    let analyzer = UnifiedTypeScriptAnalyzer::new(file_path.to_path_buf());
    let analysis = analyzer.analyze().await
        .map_err(|e| anyhow::anyhow!("Unified TypeScript analysis failed: {}", e))?;

    // Cache complexity metrics
    TYPESCRIPT_UNIFIED_CACHE.with(|cache| {
        cache.borrow_mut().insert(file_path.to_path_buf(), analysis.file_metrics.clone());
    });

    Ok(analysis.ast_items)
}

// Update analyze_single_file_complexity
async fn analyze_single_file_complexity(
    file_path: &std::path::Path,
) -> Option<FileComplexityMetrics> {
    let ext = file_path.extension()?.to_str()?;

    match ext {
        "rs" => { /* Rust cache lookup */ }
        "ts" | "js" | "jsx" | "tsx" => {
            // Check TypeScript cache first
            let cached = TYPESCRIPT_UNIFIED_CACHE.with(|cache| {
                cache.borrow().get(file_path).cloned()
            });

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path
                analyze_typescript_file_with_complexity(file_path).await.ok()
            }
        }
        // ... other languages
    }
}
```

## Validation

**Test on agentic-ai repository:**
```bash
cd /home/noah/src/agentic-ai
time pmat context --project-path . --output /tmp/typescript_unified_test.md

# Verify TypeScript files show correct AST + complexity:
grep -A10 "deno-actors/simple.ts" /tmp/typescript_unified_test.md

# Expected output:
# ### ./deno-actors/simple.ts
# **File Complexity**: 6 | **Functions**: 6
# - **Interface**: `SimpleMessage` [properties: 2]
# - **Class**: `Channel<T>` [methods: 2]
# - **Method**: `Channel.send` [complexity: 2] [cognitive: 2]
```

## Success Criteria

### Must Have
- [x] UnifiedTypeScriptAnalyzer parses TypeScript files ONCE
- [x] AST items match existing EnhancedTypeScriptVisitor output (100% identical)
- [x] Complexity metrics extracted from same parse pass
- [x] All 10+ tests passing (0 failures)
- [x] Integrated into deep_context.rs with cache
- [x] Validated on agentic-ai TypeScript files

### Should Have
- [x] Supports JavaScript (.js) in addition to TypeScript (.ts)
- [x] Handles JSX/TSX syntax
- [x] Graceful error handling for invalid syntax
- [x] Property-based tests with proptest

### Nice to Have
- [ ] Enhanced complexity visitor (can be done in REFACTOR phase)
- [ ] Line number tracking for better error reporting
- [ ] Source map support for debugging

## Performance Targets

**Expected Performance Gain:**
- Before: 2x parse (AST + Complexity)
- After: 1x parse (Unified)
- **Target**: 40-50% reduction in TypeScript/JavaScript parse time

## Timeline

**Total Estimated Time**: 3 hours

| Phase | Task | Time | Dependencies |
|-------|------|------|--------------|
| 1 | Foundation + RED Tests | 1.5h | None |
| 2 | GREEN Implementation | 1h | Phase 1 |
| 3 | Integration | 0.5h | Phase 2 |

**Sprint Duration**: Half day with focus

## References

### Related Tickets
- TICKET-3001: Unified Rust Parser (completed) - Use as template
- TICKET-3003: Unified Python Parser (next)
- TICKET-3004: Unified Go Parser (next)

### Key Files
- `server/src/services/enhanced_typescript_visitor.rs` - Current AST extraction
- `server/src/services/ast_typescript.rs` - Current complexity analysis
- `server/src/services/unified_rust_analyzer.rs` - Template for implementation
- `server/src/services/deep_context.rs` - Integration point

### Documentation
- [SWC Parser](https://swc.rs/) - TypeScript/JavaScript parser used
- [tree-sitter-typescript](https://github.com/tree-sitter/tree-sitter-typescript)

---

**Created**: 2025-09-30
**Assigned**: TBD
**Methodology**: EXTREME TDD
**Sprint**: 13
**Parent**: Sprint 12 - Unified Parser Architecture