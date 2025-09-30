# TICKET-3003: Unified Python Parser

**Sprint**: 13 - Multi-Language Unified Parser Extension
**Priority**: High
**Estimated Effort**: 3 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD
**Parent**: Sprint 12 - Unified Parser Architecture

## Problem Statement

Python files are currently parsed TWICE in `pmat context`:
1. **AST Analysis Pass**: Extracts functions, classes, methods
2. **Complexity Analysis Pass**: Calculates cyclomatic and cognitive complexity

This doubles the parsing time for Python files. Following the success of TICKET-3001 (Rust) and TICKET-3002 (TypeScript), we need to extend the unified parser architecture to Python.

### Current Architecture Issues

**Double Parsing in deep_context.rs:**
```rust
// Pass 1: AST extraction
"python" => analyze_python_language(file_path).await

// Pass 2: Complexity analysis (SEPARATE PARSE!)
"py" => analyze_python_file_with_complexity(file_path, None).await
```

Both functions parse the file separately using tree-sitter-python or Python's ast module.

### Performance Impact

- **Redundant I/O**: Reading file twice from disk
- **Redundant Parsing**: tree-sitter or ast.parse is expensive
- **Memory Overhead**: Two separate AST representations
- **Expected Gain**: 40-50% reduction in Python parse time

### Validation Project

Test on `/home/noah/src/agentic-ai` if it contains Python files, or create test fixtures with:
- Classes with methods
- Functions with decorators
- Async/await syntax
- List comprehensions and generators

## Goal

Create `UnifiedPythonAnalyzer` that parses Python files ONCE and extracts both AST items and complexity metrics from a single parse pass.

## Implementation Strategy

### Phase 1: Foundation (1.5 hours)

**Deliverables:**
- `server/src/services/unified_python_analyzer.rs` - New module
- `UnifiedPythonAnalyzer` struct with single-pass architecture
- `UnifiedAnalysis` result type (reuse from Rust/TypeScript)
- 10+ EXTREME TDD tests (all must fail initially)

**API Design:**
```rust
pub struct UnifiedPythonAnalyzer {
    file_path: PathBuf,
    #[cfg(test)]
    parse_count: AtomicUsize, // Verify single parse
}

pub struct UnifiedAnalysis {
    pub ast_items: Vec<AstItem>,
    pub file_metrics: FileComplexityMetrics,
    pub parsed_at: std::time::Instant,
}

impl UnifiedPythonAnalyzer {
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
fn red_test_unified_python_analyzer_can_be_created() {
    let path = PathBuf::from("test.py");
    let analyzer = UnifiedPythonAnalyzer::new(path.clone());
    assert_eq!(analyzer.file_path(), path.as_path());
}

// Test 2: Single parse guarantee
#[tokio::test]
async fn red_test_unified_python_parses_only_once() {
    let temp_file = create_temp_py_file("def add(a, b):\n    return a + b");
    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let _ = analyzer.analyze().await;
    assert_eq!(analyzer.parse_count(), 1, "Must parse exactly once!");
}

// Test 3: Returns both AST and complexity
#[tokio::test]
async fn red_test_unified_python_returns_both_ast_and_complexity() {
    let temp_file = create_temp_py_file("def greet(name):\n    print(f'Hello {name}')");
    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    assert!(!result.ast_items.is_empty(), "Must extract AST items");
    assert!(!result.file_metrics.functions.is_empty(), "Must extract complexity");
}

// Test 4: AST items match EnhancedPythonVisitor
#[tokio::test]
async fn red_test_unified_python_ast_matches_enhanced_visitor() {
    let temp_file = create_temp_py_file(r#"
def multiply(x, y):
    return x * y

class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    "#);

    // OLD WAY: EnhancedPythonVisitor
    let old_items = enhanced_python_visitor_extract(&temp_file);

    // NEW WAY: UnifiedPythonAnalyzer
    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await.unwrap();

    assert_eq!(old_items.len(), result.ast_items.len());
}

// Test 5: Handles invalid Python syntax
#[tokio::test]
async fn red_test_unified_python_handles_invalid_syntax() {
    let temp_file = create_temp_py_file("def broken syntax here {{{");
    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;
    assert!(result.is_err(), "Must return error for invalid syntax");
}

// Test 6: Property test - various file sizes
proptest! {
    #[test]
    fn red_property_unified_python_handles_any_valid_code(
        function_count in 1usize..20,
    ) {
        let mut source = String::new();
        for i in 0..function_count {
            source.push_str(&format!("def func_{}():\n    print('test')\n\n", i));
        }

        let temp_file = create_temp_py_file(&source);
        let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(analyzer.analyze());

        prop_assert!(result.is_ok());
        let analysis = result.unwrap();
        prop_assert_eq!(analysis.ast_items.len(), function_count);
    }
}

// Test 7: Multiple function types (regular, async, class methods, decorators)
#[tokio::test]
async fn red_test_unified_python_handles_multiple_function_types() {
    let temp_file = create_temp_py_file(r#"
# Regular function
def regular():
    pass

# Async function
async def async_func():
    pass

# Decorated function
@property
def decorated():
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
    let result = analyzer.analyze().await.unwrap();

    assert!(result.ast_items.len() >= 5);
}

// Test 8: List comprehensions and complex control flow
#[tokio::test]
async fn red_test_unified_python_handles_complex_control_flow() {
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
    let result = analyzer.analyze().await.unwrap();

    // Should detect higher complexity due to nested structures
    assert!(result.file_metrics.functions[0].metrics.cyclomatic > 5);
}

// Test 9: Empty file
#[tokio::test]
async fn red_test_unified_python_handles_empty_file() {
    let temp_file = create_temp_py_file("");
    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0);
}

// Test 10: Comment-only file
#[tokio::test]
async fn red_test_unified_python_handles_comment_only_file() {
    let temp_file = create_temp_py_file(r#"
# This is just a comment
"""
And a docstring
"""
    "#);

    let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());
    let result = analyzer.analyze().await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.ast_items.len(), 0);
}
```

### Phase 2: GREEN Implementation (1 hour)

**Core Implementation:**
```rust
impl UnifiedPythonAnalyzer {
    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError> {
        #[cfg(test)]
        { self.parse_count.fetch_add(1, Ordering::SeqCst); }

        // 1. Read file ONCE
        let content = tokio::fs::read_to_string(&self.file_path).await?;

        // 2. Parse ONCE with tree-sitter-python
        let syntax_tree = self.parse_python(&content)?;

        // 3. Extract AST items (reuse EnhancedPythonVisitor logic)
        let ast_items = self.extract_ast_items(&syntax_tree);

        // 4. Extract complexity metrics (new SimpleComplexityVisitor)
        let file_metrics = self.extract_complexity_metrics(&syntax_tree);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    fn parse_python(&self, content: &str) -> Result<ParsedTree, AnalysisError> {
        // Use tree-sitter-python
        // Return unified AST representation
    }

    fn extract_ast_items(&self, tree: &ParsedTree) -> Vec<AstItem> {
        // Reuse EnhancedPythonVisitor logic
        // Walk tree and extract:
        // - Functions (def, async def)
        // - Classes
        // - Methods
        // - Decorators
    }

    fn extract_complexity_metrics(&self, tree: &ParsedTree) -> FileComplexityMetrics {
        // Simple complexity calculation for GREEN phase
        // Count branches:
        // - if/elif
        // - for/while loops
        // - try/except/finally
        // - and/or in conditions
        // - list comprehensions with if
        // - match/case (Python 3.10+)
    }
}
```

### Phase 3: Integration (0.5 hours)

**Update deep_context.rs:**
```rust
// Add thread-local cache for Python
thread_local! {
    static PYTHON_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> =
        RefCell::new(FxHashMap::default());
}

// Update analyze_python_language
pub async fn analyze_python_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    use crate::services::unified_python_analyzer::UnifiedPythonAnalyzer;

    let analyzer = UnifiedPythonAnalyzer::new(file_path.to_path_buf());
    let analysis = analyzer.analyze().await
        .map_err(|e| anyhow::anyhow!("Unified Python analysis failed: {}", e))?;

    // Cache complexity metrics
    PYTHON_UNIFIED_CACHE.with(|cache| {
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
        "py" => {
            // Check Python cache first
            let cached = PYTHON_UNIFIED_CACHE.with(|cache| {
                cache.borrow().get(file_path).cloned()
            });

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path
                analyze_python_file_with_complexity(file_path, None).await.ok()
            }
        }
        // ... other languages
    }
}
```

## Validation

**Test on agentic-ai or create test fixtures:**
```bash
# Create test Python file if agentic-ai doesn't have any
cat > /tmp/test_python.py <<EOF
class Calculator:
    def add(self, a, b):
        return a + b

    async def multiply_async(self, x, y):
        return x * y

def complex_function(data):
    result = [x for x in data if x > 0]
    for item in result:
        if item % 2 == 0:
            print(item)
    return result
EOF

time pmat context --project-path /tmp --output /tmp/python_unified_test.md

# Verify Python files show correct AST + complexity:
grep -A10 "test_python.py" /tmp/python_unified_test.md

# Expected output:
# ### ./test_python.py
# **File Complexity**: 3 | **Functions**: 3
# - **Class**: `Calculator` [methods: 2]
# - **Method**: `Calculator.add` [complexity: 1] [cognitive: 0]
# - **Method**: `Calculator.multiply_async` [complexity: 1] [cognitive: 0]
# - **Function**: `complex_function` [complexity: 3] [cognitive: 4]
```

## Success Criteria

### Must Have
- [x] UnifiedPythonAnalyzer parses Python files ONCE
- [x] AST items match existing EnhancedPythonVisitor output (100% identical)
- [x] Complexity metrics extracted from same parse pass
- [x] All 10+ tests passing (0 failures)
- [x] Integrated into deep_context.rs with cache
- [x] Validated on Python files (real or test fixtures)

### Should Have
- [x] Handles async/await syntax
- [x] Handles decorators
- [x] Handles list comprehensions and generators
- [x] Graceful error handling for invalid syntax
- [x] Property-based tests with proptest

### Nice to Have
- [ ] Enhanced complexity visitor for Python-specific patterns
- [ ] Line number tracking for better error reporting
- [ ] Type hint parsing (PEP 484)

## Performance Targets

**Expected Performance Gain:**
- Before: 2x parse (AST + Complexity)
- After: 1x parse (Unified)
- **Target**: 40-50% reduction in Python parse time

## Timeline

**Total Estimated Time**: 3 hours

| Phase | Task | Time | Dependencies |
|-------|------|------|--------------|
| 1 | Foundation + RED Tests | 1.5h | None |
| 2 | GREEN Implementation | 1h | Phase 1 |
| 3 | Integration | 0.5h | Phase 2 |

**Sprint Duration**: Half day with focus

## Python-Specific Complexity Patterns

**Complexity Increments:**
- `if/elif`: +1 per condition
- `for/while`: +1 per loop
- `and/or` in conditions: +1 per operator
- `try/except`: +1 per except clause
- List comprehensions with `if`: +1
- Match/case (3.10+): +1 per case
- Ternary operator: +1
- Lambda expressions: +1

**Example:**
```python
def complex_function(items):
    # Base: 1
    if not items:  # +1
        return []

    # List comp with condition: +1
    filtered = [x for x in items if x > 0]

    for item in filtered:  # +1
        if item % 2 == 0:  # +1
            print(item)
        elif item % 3 == 0:  # +1
            print("divisible by 3")

    # Total CC: 6
    return filtered
```

## References

### Related Tickets
- TICKET-3001: Unified Rust Parser (completed)
- TICKET-3002: Unified TypeScript Parser (completed)
- TICKET-3004: Unified Go Parser (next)

### Key Files
- `server/src/services/enhanced_python_visitor.rs` - Current AST extraction
- `server/src/services/ast_python.rs` - Current complexity analysis
- `server/src/services/unified_rust_analyzer.rs` - Template for implementation
- `server/src/services/deep_context.rs` - Integration point

### Documentation
- [tree-sitter-python](https://github.com/tree-sitter/tree-sitter-python)
- [Python AST module](https://docs.python.org/3/library/ast.html)
- [Python Complexity Metrics](https://radon.readthedocs.io/en/latest/intro.html)

---

**Created**: 2025-09-30
**Assigned**: TBD
**Methodology**: EXTREME TDD
**Sprint**: 13
**Parent**: Sprint 12 - Unified Parser Architecture