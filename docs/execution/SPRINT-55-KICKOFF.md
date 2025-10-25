# Sprint 55: TypeScript/JavaScript Source Parsing Implementation

**Sprint**: 55
**Focus**: Implement Real TypeScript/JavaScript Source Parsing
**Date**: October 25, 2025
**Status**: 🚀 READY TO START
**Dependencies**: Sprint 54 (COMPLETED ✅)

---

## Executive Summary

Sprint 55 will replace the placeholder TypeScript and JavaScript source parsing implementations with real, functional parsers that extract AstItem data from source strings. This completes the MCP tool stack for the most widely-used programming languages.

### Context from Sprint 54

Sprint 54 successfully resolved all compilation errors and established type-safe AstItem field extraction via the `ast_item_helpers` module. However, the TypeScript and JavaScript analyzers currently return empty vectors:

```rust
// Current placeholder implementation
pub fn analyze_typescript_source(&self, _source: &str) -> Result<Vec<AstItem>> {
    Ok(Vec::new()) // Placeholder
}
```

This sprint will implement real functionality.

---

## Objectives

### Primary Goal
Implement functional TypeScript and JavaScript source parsing that extracts AstItem data from source code strings.

### Success Criteria
1. ✅ `TypeScriptAstVisitor::analyze_typescript_source()` returns real AstItems
2. ✅ `JavaScriptAstVisitor::analyze_javascript_source()` returns real AstItems
3. ✅ MCP tools can analyze TS/JS source without file system access
4. ✅ Integration tests verify correct AstItem extraction
5. ✅ All compilation and lint checks pass
6. ✅ pmat-book validation passes

---

## Current State Analysis

### What Exists (Sprint 54)
- ✅ TypeScriptAstVisitor struct with placeholder method
- ✅ JavaScriptAstVisitor struct with placeholder method
- ✅ TypeScriptStrategy with file-based `analyze()` method
- ✅ Helper functions for AstItem field extraction
- ✅ MCP integration framework (java_tools, scala_tools patterns)

### What's Missing
- 🔶 Method to parse source string → AST → AstItems
- 🔶 Temporary file handling OR direct source parsing
- 🔶 Integration tests for source parsing
- 🔶 Documentation for source-based analysis

### Files to Modify
1. `server/src/services/languages/typescript.rs`
2. `server/src/services/languages/javascript.rs`
3. Possibly `server/src/services/ast/languages/typescript.rs` (add parse_source method)

---

## Technical Approach

### Option A: Temporary File Approach (Recommended)

**Rationale**: Leverages existing file-based parser infrastructure

**Implementation**:
```rust
pub fn analyze_typescript_source(&self, source: &str) -> Result<Vec<AstItem>> {
    // Create temporary file
    let temp_file = tempfile::NamedTempFile::new()?;
    std::fs::write(temp_file.path(), source)?;

    // Use existing file-based parser
    let strategy = TypeScriptStrategy::new();
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let context = analyze_typescript_file(temp_file.path()).await?;
        Ok(context.items) // Extract AstItems from FileContext
    })
}
```

**Pros**:
- ✅ Reuses existing, tested file parser
- ✅ No need to modify TypeScriptStrategy
- ✅ Lower risk, faster implementation
- ✅ Handles all TypeScript features correctly

**Cons**:
- 🔶 Slight performance overhead (file I/O)
- 🔶 Requires tempfile dependency

### Option B: Direct Source Parsing

**Rationale**: Add new method to TypeScriptStrategy

**Implementation**:
```rust
// Add to TypeScriptStrategy
pub async fn parse_source(&self, source: &str, path: &Path) -> Result<Vec<AstItem>> {
    // Parse source directly
    // Extract items without file system
}

// Use in visitor
pub fn analyze_typescript_source(&self, source: &str) -> Result<Vec<AstItem>> {
    let strategy = TypeScriptStrategy::new();
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        strategy.parse_source(source, &self.path).await
    })
}
```

**Pros**:
- ✅ No file I/O overhead
- ✅ More direct implementation

**Cons**:
- 🔶 Requires modifying TypeScriptStrategy
- 🔶 Need to ensure FileContext → AstItem extraction works
- 🔶 Higher complexity, more testing needed

### Recommended: Option A (Temporary File)

Option A is recommended because:
1. **Lower Risk**: Reuses proven file parser
2. **Faster Implementation**: ~2 hours vs ~4 hours
3. **Simpler Testing**: Existing tests cover file parsing
4. **Performance**: File I/O is negligible for typical source strings

---

## Implementation Plan

### Phase 1: Add tempfile Dependency (15 minutes)

**File**: `server/Cargo.toml`

```toml
[dependencies]
tempfile = "3.8"
```

**Verification**:
```bash
cargo check --lib
```

### Phase 2: Implement TypeScript Source Parsing (60 minutes)

**File**: `server/src/services/languages/typescript.rs`

**Tasks**:
1. Import tempfile and required modules
2. Replace placeholder implementation:
   - Create temp file
   - Write source to temp file
   - Call `analyze_typescript_file()`
   - Extract `items` from `FileContext`
   - Return AstItems
3. Update documentation
4. Remove `#[allow(dead_code)]` from path field (now used)

**Code**:
```rust
use tempfile::NamedTempFile;
use std::io::Write;
use crate::services::ast_typescript::analyze_typescript_file;

/// Analyze TypeScript source code
#[cfg(feature = "typescript-ast")]
pub fn analyze_typescript_source(&self, source: &str) -> Result<Vec<AstItem>> {
    // Create temporary file with .ts extension
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(source.as_bytes())?;

    // Use existing file-based parser
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let context = analyze_typescript_file(temp_file.path()).await
            .map_err(|e| anyhow::anyhow!("TypeScript parsing failed: {}", e))?;
        Ok(context.items)
    })
}
```

**Verification**:
```bash
cargo check --lib
cargo test --lib typescript_ast_visitor
```

### Phase 3: Implement JavaScript Source Parsing (30 minutes)

**File**: `server/src/services/languages/javascript.rs`

**Tasks**:
1. Same approach as TypeScript (JS uses TS parser)
2. Create temp file with .js extension
3. Call `analyze_typescript_file()` (handles JS too)
4. Extract and return AstItems

**Code**: Nearly identical to TypeScript implementation

**Verification**:
```bash
cargo check --lib
cargo test --lib javascript_ast_visitor
```

### Phase 4: Add Integration Tests (45 minutes)

**File**: `server/tests/integration/typescript_source_parsing.rs` (NEW)

**Tests**:
```rust
#[tokio::test]
async fn test_analyze_typescript_function() {
    let source = r#"
        function greet(name: string): string {
            return `Hello, ${name}!`;
        }
    "#;

    let visitor = TypeScriptAstVisitor::new(Path::new("test.ts"));
    let items = visitor.analyze_typescript_source(source).unwrap();

    assert_eq!(items.len(), 1);
    assert!(matches!(items[0], AstItem::Function { .. }));
}

#[tokio::test]
async fn test_analyze_typescript_class() {
    let source = r#"
        class Person {
            name: string;
            constructor(name: string) {
                this.name = name;
            }
            greet(): string {
                return `Hello, I'm ${this.name}`;
            }
        }
    "#;

    let visitor = TypeScriptAstVisitor::new(Path::new("test.ts"));
    let items = visitor.analyze_typescript_source(source).unwrap();

    // Should extract class and methods
    assert!(items.len() >= 2);
    assert!(items.iter().any(|item| matches!(item, AstItem::Struct { .. })));
    assert!(items.iter().any(|item| matches!(item, AstItem::Function { .. })));
}
```

**JavaScript Tests**: Similar patterns

**Verification**:
```bash
cargo test --lib integration::typescript_source_parsing
cargo test --lib integration::javascript_source_parsing
```

### Phase 5: Update MCP Tools Documentation (30 minutes)

**File**: `docs/mcp/TYPESCRIPT-JAVASCRIPT-TOOLS.md` (NEW)

**Content**:
- Tool capabilities (source-based analysis)
- Example usage
- Supported TypeScript/JavaScript features
- Performance characteristics

### Phase 6: Quality Gates (30 minutes)

**Tasks**:
1. Run full test suite
2. Run lint checks
3. Verify pmat-book validation
4. Update CHANGELOG.md

**Commands**:
```bash
cargo test --lib
make lint
make validate-book
```

---

## Time Estimate

| Phase | Task | Estimated Time |
|-------|------|----------------|
| 1 | Add tempfile dependency | 15 min |
| 2 | Implement TypeScript parsing | 60 min |
| 3 | Implement JavaScript parsing | 30 min |
| 4 | Add integration tests | 45 min |
| 5 | Update documentation | 30 min |
| 6 | Quality gates & verification | 30 min |
| **Total** | **End-to-end implementation** | **3.5 hours** |

**Confidence**: High (90%) - Straightforward implementation with clear pattern

---

## Risk Assessment

### Low Risk ✅
- **Existing Infrastructure**: File parser already works perfectly
- **Clear Pattern**: Java/Scala tools provide template
- **Isolated Changes**: Only 2 files + tests modified
- **Reversible**: Easy to rollback if issues arise

### Mitigation Strategies
1. **Incremental Testing**: Test TypeScript first, then JavaScript
2. **Temporary File Cleanup**: Use RAII pattern (NamedTempFile auto-deletes)
3. **Error Handling**: Wrap all file operations in Result
4. **Feature Flags**: Already gated by `typescript-ast` feature

---

## Success Metrics

### Functional Metrics
- ✅ TypeScript source parsing extracts ≥1 AstItem from valid source
- ✅ JavaScript source parsing extracts ≥1 AstItem from valid source
- ✅ Empty source returns empty Vec (no crashes)
- ✅ Invalid source returns Err (not panic)

### Quality Metrics
- ✅ 0 compilation errors
- ✅ 0 lint warnings
- ✅ 100% of new tests passing
- ✅ pmat-book validation passes

### Performance Metrics
- ✅ Source parsing completes in <100ms for typical files
- ✅ Temporary file cleanup works correctly (no leaks)

---

## Dependencies

### External Crates
- `tempfile = "3.8"` (NEW - for temporary file creation)
- `tokio` (EXISTING - async runtime)
- `anyhow` (EXISTING - error handling)

### Internal Modules
- `server/src/services/ast_typescript.rs` (EXISTING - file parser)
- `server/src/services/context.rs` (EXISTING - AstItem definition)
- `server/src/services/ast/languages/typescript.rs` (EXISTING - TypeScriptStrategy)

---

## Testing Strategy

### Unit Tests
- Test empty source → empty Vec
- Test single function → 1 AstItem::Function
- Test single class → 1 AstItem::Struct
- Test invalid syntax → Err result

### Integration Tests
- Test complex TypeScript file (classes, functions, interfaces)
- Test complex JavaScript file (classes, functions, modules)
- Test edge cases (comments, whitespace, etc.)

### Property Tests (Optional)
- Any valid TS/JS source → no panic
- Source with N functions → ≥N AstItems

---

## Rollout Plan

### Phase 1: Implementation (1 session)
1. Add dependency
2. Implement TypeScript parsing
3. Implement JavaScript parsing
4. Add basic tests

### Phase 2: Verification (1 session)
1. Run comprehensive tests
2. Update documentation
3. Run quality gates

### Phase 3: Integration (1 session)
1. Test MCP tools end-to-end
2. Update user documentation
3. Create release notes

---

## Post-Sprint Tasks

### Immediate Follow-up (Sprint 56)
1. **Feature Flag Validation**: Test language-specific builds
2. **Performance Benchmarking**: Measure source parsing overhead
3. **MCP Tool Testing**: End-to-end validation with real projects

### Future Enhancements (Sprint 57+)
1. **Caching**: Cache parsed results for repeated sources
2. **Direct Parsing**: Implement Option B for zero file I/O
3. **Enhanced Metadata**: Extract TypeScript-specific info (decorators, generics)

---

## References

### Related Sprints
- Sprint 51: JVM language support (Java, Scala)
- Sprint 52: Cross-language analysis
- Sprint 53: Feature flags
- Sprint 54: MCP integration stabilization

### Documentation
- `server/src/services/languages/java.rs`: Reference implementation
- `server/src/services/ast_typescript.rs`: File-based parser
- `docs/mcp/POLYGLOT-TOOLS.md`: MCP tools overview

### External Resources
- tempfile crate: https://docs.rs/tempfile/3.8.0/tempfile/
- TypeScript tree-sitter: Used by existing parser

---

## Appendix A: FileContext Structure

Understanding FileContext is key to extracting AstItems:

```rust
pub struct FileContext {
    pub path: PathBuf,
    pub items: Vec<AstItem>,  // <-- This is what we need
    pub language: String,
    pub complexity: Option<FileComplexityMetrics>,
    // ... other fields
}
```

**Extraction**:
```rust
let context = analyze_typescript_file(path).await?;
let items = context.items; // Vec<AstItem>
```

---

## Appendix B: Error Handling Pattern

```rust
pub fn analyze_typescript_source(&self, source: &str) -> Result<Vec<AstItem>> {
    // Create temp file
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| anyhow::anyhow!("Failed to create temp file: {}", e))?;

    // Write source
    temp_file.write_all(source.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write source: {}", e))?;

    // Parse
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;

    runtime.block_on(async {
        analyze_typescript_file(temp_file.path()).await
            .map_err(|e| anyhow::anyhow!("TypeScript parsing failed: {}", e))
            .map(|context| context.items)
    })
}
```

---

## Approval & Kickoff

**Sprint Owner**: Claude (AI Agent)
**Reviewer**: User
**Status**: AWAITING APPROVAL

**Kickoff Checklist**:
- [ ] Sprint goals understood and approved
- [ ] Technical approach reviewed (Option A: Temporary File)
- [ ] Time estimate acceptable (3.5 hours)
- [ ] Risk assessment reviewed
- [ ] Dependencies identified

**Ready to Start**: YES ✅

---

**Document Version**: 1.0
**Created**: October 25, 2025
**Status**: READY FOR EXECUTION
