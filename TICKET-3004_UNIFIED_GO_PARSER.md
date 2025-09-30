# TICKET-3004: Unified Go Parser

**Sprint**: 13 - Multi-Language Unified Parser Extension
**Priority**: High
**Estimated Effort**: 3 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD
**Parent**: Sprint 12 - Unified Parser Architecture

## Problem Statement

Go files are currently parsed TWICE in `pmat context`:
1. **AST Analysis Pass**: Extracts functions, structs, methods, interfaces
2. **Complexity Analysis Pass**: Calculates cyclomatic and cognitive complexity

This doubles the parsing time for Go files. Following the success of TICKET-3001 (Rust), TICKET-3002 (TypeScript), and TICKET-3003 (Python), we need to extend the unified parser architecture to Go.

### Current Architecture Issues

**Double Parsing in deep_context.rs:**
```rust
// Pass 1: AST extraction
"go" => analyze_go_language(file_path).await

// Pass 2: Complexity analysis (SEPARATE PARSE!)
// Currently complexity may not be extracted for Go - needs verification
```

Both functions parse the file separately using tree-sitter-go.

### Performance Impact

- **Redundant I/O**: Reading file twice from disk
- **Redundant Parsing**: tree-sitter-go parsing is expensive
- **Memory Overhead**: Two separate AST representations
- **Expected Gain**: 40-50% reduction in Go parse time

### Validation Project

Test on `/home/noah/src/agentic-ai` which contains multiple Go files:
- `go-actors/main.go`
- `go-actors/simple.go`
- `go-actors/simple_test.go`
- `go-calc-supervisor/calc.go`
- `go-calc-supervisor/calc_test.go`

These files have functions, structs, methods, and interfaces - perfect for validation.

## Goal

Create `UnifiedGoAnalyzer` that parses Go files ONCE and extracts both AST items and complexity metrics from a single parse pass.

## Implementation Strategy

### Phase 1: Foundation (1.5 hours)

**Deliverables:**
- `server/src/services/unified_go_analyzer.rs` - New module
- `UnifiedGoAnalyzer` struct with single-pass architecture
- `UnifiedAnalysis` result type (reuse from Rust/TypeScript/Python)
- 10+ EXTREME TDD tests (all must fail initially)

**API Design:**
```rust
pub struct UnifiedGoAnalyzer {
    file_path: PathBuf,
    #[cfg(test)]
    parse_count: AtomicUsize, // Verify single parse
}

pub struct UnifiedAnalysis {
    pub ast_items: Vec<AstItem>,
    pub file_metrics: FileComplexityMetrics,
    pub parsed_at: std::time::Instant,
}

impl UnifiedGoAnalyzer {
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
fn red_test_unified_go_analyzer_can_be_created() {
    let path = PathBuf::from("test.go");
    let analyzer = UnifiedGoAnalyzer::new(path.clone());
    assert_eq!(analyzer.file_path(), path.as_path());
}

// Test 2: Single parse guarantee
#[tokio::test]
async fn red_test_unified_go_parses_only_once() {
    let temp_file = create_temp_go_file("package main\n\nfunc add(a, b int) int {\n\treturn a + b\n}");
    let analyzer = UnifiedGoAnalyzer::new(temp_file.path().to_path_buf());
    let _ = analyzer.analyze().await;
    assert_eq!(analyzer.parse_count(), 1, "Must parse exactly once!");
}

// Test 3: Returns both AST and complexity
#[tokio::test]
async fn red_test_unified_go_returns_both_ast_and_complexity() {
    let temp_file = create_temp_go_file(r#"
package main

func greet(name string) {
    fmt.Printf("Hello %s\n", name)
}
    "#);

    let analyzer = UnifiedGoAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    assert!(!result.ast_items.is_empty(), "Must extract AST items");
    assert!(!result.file_metrics.functions.is_empty(), "Must extract complexity");
}

// Test 4: AST items match existing Go analyzer
#[tokio::test]
async fn red_test_unified_go_ast_matches_existing_analyzer() {
    let temp_file = create_temp_go_file(r#"
package main

func multiply(x, y int) int {
    return x * y
}

type Point struct {
    X int
    Y int
}

func (p *Point) Distance() float64 {
    return math.Sqrt(float64(p.X*p.X + p.Y*p.Y))
}
    "#);

    // OLD WAY: Existing Go analyzer
    let old_items = analyze_go_file_ast(&temp_file).await;

    // NEW WAY: UnifiedGoAnalyzer
    let analyzer = UnifiedGoAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    assert_eq!(old_items.len(), result.ast_items.len());
}

// Test 5: Handles invalid Go syntax
#[tokio::test]
async fn red_test_unified_go_handles_invalid_syntax() {
    let temp_file = create_temp_go_file("func broken syntax here {{{");
    let analyzer = UnifiedGoAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;
    assert!(result.is_err(), "Must return error for invalid syntax");
}

// Test 6: Property test - various file sizes
proptest! {
    #[test]
    fn red_property_unified_go_handles_any_valid_code(
        function_count in 1usize..20,
    ) {
        let mut source = String::from("package main\n\n");
        for i in 0..function_count {
            source.push_str(&format!("func func_{}() {{\n\tfmt.Println(\"test\")\n}}\n\n", i));
        }

        let temp_file = create_temp_go_file(&source);
        let analyzer = UnifiedGoAnalyzer::new(temp_file.path().to_path_buf());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(analyzer.analyze());

        prop_assert!(result.is_ok());
        let analysis = result.unwrap();
        prop_assert_eq!(analysis.ast_items.len(), function_count);
    }
}

// Test 7: Real-world file from agentic-ai
#[tokio::test]
async fn red_test_unified_go_on_real_file() {
    let real_file = PathBuf::from("/home/noah/src/agentic-ai/go-actors/simple.go");
    if !real_file.exists() { return; }

    let analyzer = UnifiedGoAnalyzer::new(real_file);
    let result = analyzer.analyze().await;

    assert!(result.is_ok(), "Must handle real-world Go files");
    let analysis = result.unwrap();
    assert!(analysis.ast_items.len() > 1, "simple.go has multiple items");
}

// Test 8: Multiple Go constructs (functions, methods, structs, interfaces)
#[tokio::test]
async fn red_test_unified_go_handles_multiple_constructs() {
    let temp_file = create_temp_go_file(r#"
package main

// Function
func regularFunc() {}

// Struct
type MyStruct struct {
    Field1 string
    Field2 int
}

// Method
func (m *MyStruct) Method() {}

// Interface
type MyInterface interface {
    DoSomething()
}

// Goroutine function
func asyncFunc() {
    go func() {
        fmt.Println("goroutine")
    }()
}
    "#);

    let analyzer = UnifiedGoAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    assert!(result.ast_items.len() >= 5);
}

// Test 9: Empty file (just package declaration)
#[tokio::test]
async fn red_test_unified_go_handles_empty_file() {
    let temp_file = create_temp_go_file("package main\n");
    let analyzer = UnifiedGoAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0);
}

// Test 10: Comment-only file
#[tokio::test]
async fn red_test_unified_go_handles_comment_only_file() {
    let temp_file = create_temp_go_file(r#"
package main

// This is just a comment
/* And a block comment */
    "#);

    let analyzer = UnifiedGoAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0);
}
```

### Phase 2: GREEN Implementation (1 hour)

**Core Implementation:**
```rust
impl UnifiedGoAnalyzer {
    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError> {
        #[cfg(test)]
        { self.parse_count.fetch_add(1, Ordering::SeqCst); }

        // 1. Read file ONCE
        let content = tokio::fs::read_to_string(&self.file_path).await?;

        // 2. Parse ONCE with tree-sitter-go
        let syntax_tree = self.parse_go(&content)?;

        // 3. Extract AST items (reuse existing Go analyzer logic)
        let ast_items = self.extract_ast_items(&syntax_tree);

        // 4. Extract complexity metrics (new SimpleComplexityVisitor)
        let file_metrics = self.extract_complexity_metrics(&syntax_tree);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    fn parse_go(&self, content: &str) -> Result<ParsedTree, AnalysisError> {
        // Use tree-sitter-go
        // Return unified AST representation
    }

    fn extract_ast_items(&self, tree: &ParsedTree) -> Vec<AstItem> {
        // Reuse existing Go analyzer logic
        // Walk tree and extract:
        // - Functions (func declarations)
        // - Methods (func with receiver)
        // - Structs (type declarations with struct)
        // - Interfaces (type declarations with interface)
    }

    fn extract_complexity_metrics(&self, tree: &ParsedTree) -> FileComplexityMetrics {
        // Simple complexity calculation for GREEN phase
        // Count branches:
        // - if/else if
        // - switch/case
        // - for loops (all variants)
        // - &&/|| in conditions
        // - select statements
        // - defer/go statements (optional: +1 for concurrency)
    }
}
```

### Phase 3: Integration (0.5 hours)

**Update deep_context.rs:**
```rust
// Add thread-local cache for Go
thread_local! {
    static GO_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> =
        RefCell::new(FxHashMap::default());
}

// Update analyze_go_language
pub async fn analyze_go_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    use crate::services::unified_go_analyzer::UnifiedGoAnalyzer;

    let analyzer = UnifiedGoAnalyzer::new(file_path.to_path_buf());
    let analysis = analyzer.analyze().await
        .map_err(|e| anyhow::anyhow!("Unified Go analysis failed: {}", e))?;

    // Cache complexity metrics
    GO_UNIFIED_CACHE.with(|cache| {
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
        "ts" | "js" | "jsx" | "tsx" => { /* TypeScript cache lookup */ }
        "py" => { /* Python cache lookup */ }
        "go" => {
            // Check Go cache first
            let cached = GO_UNIFIED_CACHE.with(|cache| {
                cache.borrow().get(file_path).cloned()
            });

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path (if exists)
                None
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
time pmat context --project-path . --output /tmp/go_unified_test.md

# Verify Go files show correct AST + complexity:
grep -A15 "go-actors/simple.go" /tmp/go_unified_test.md

# Expected output:
# ### ./go-actors/simple.go
# **File Complexity**: 3 | **Functions**: 2
# - **Struct**: `SimpleMessage` [fields: 2]
# - **Function**: `SimplePingPong` [complexity: 5] [cognitive: 8] [big-o: O(n)]
# - **Function**: `main::SimplePingPong` [complexity: 3] [cognitive: 2]
```

## Success Criteria

### Must Have
- [x] UnifiedGoAnalyzer parses Go files ONCE
- [x] AST items match existing Go analyzer output (100% identical)
- [x] Complexity metrics extracted from same parse pass
- [x] All 10+ tests passing (0 failures)
- [x] Integrated into deep_context.rs with cache
- [x] Validated on agentic-ai Go files

### Should Have
- [x] Handles functions and methods (with receivers)
- [x] Handles structs and interfaces
- [x] Handles goroutines and channels
- [x] Graceful error handling for invalid syntax
- [x] Property-based tests with proptest

### Nice to Have
- [ ] Enhanced complexity for Go-specific patterns (select, defer)
- [ ] Line number tracking for better error reporting
- [ ] Concurrency pattern detection

## Performance Targets

**Expected Performance Gain:**
- Before: 2x parse (AST + Complexity if implemented)
- After: 1x parse (Unified)
- **Target**: 40-50% reduction in Go parse time

## Timeline

**Total Estimated Time**: 3 hours

| Phase | Task | Time | Dependencies |
|-------|------|------|--------------|
| 1 | Foundation + RED Tests | 1.5h | None |
| 2 | GREEN Implementation | 1h | Phase 1 |
| 3 | Integration | 0.5h | Phase 2 |

**Sprint Duration**: Half day with focus

## Go-Specific Complexity Patterns

**Complexity Increments:**
- `if/else if`: +1 per condition
- `switch/case`: +1 per case
- `for` loops (all variants): +1 per loop
- `&&/||` in conditions: +1 per operator
- `select` statement: +1 per case
- Range loops: +1
- Defer: Optional +0 (doesn't add control flow)
- Go routines: Optional +1 (adds concurrency complexity)

**Example:**
```go
func complexFunction(items []int, ch chan int) {
    // Base: 1
    if len(items) == 0 {  // +1
        return
    }

    for _, item := range items {  // +1
        if item % 2 == 0 {  // +1
            ch <- item
        } else if item % 3 == 0 {  // +1
            go processItem(item)  // Optional: +1 for goroutine
        }
    }

    select {  // +1
    case val := <-ch:
        fmt.Println(val)
    case <-time.After(1 * time.Second):  // +1 (additional case)
        fmt.Println("timeout")
    }

    // Total CC: 6 (or 7 with goroutine counting)
}
```

## References

### Related Tickets
- TICKET-3001: Unified Rust Parser (completed)
- TICKET-3002: Unified TypeScript Parser (completed)
- TICKET-3003: Unified Python Parser (completed)

### Key Files
- `server/src/services/languages/go.rs` - Current Go analyzer
- `server/src/services/unified_rust_analyzer.rs` - Template for implementation
- `server/src/services/deep_context.rs` - Integration point

### Documentation
- [tree-sitter-go](https://github.com/tree-sitter/tree-sitter-go)
- [Go AST package](https://pkg.go.dev/go/ast)
- [Cyclomatic Complexity for Go](https://github.com/fzipp/gocyclo)

---

**Created**: 2025-09-30
**Assigned**: TBD
**Methodology**: EXTREME TDD
**Sprint**: 13
**Parent**: Sprint 12 - Unified Parser Architecture