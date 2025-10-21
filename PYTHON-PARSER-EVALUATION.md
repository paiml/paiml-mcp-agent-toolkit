# Python Parser Evaluation & Recommendation

**Date**: 2025-10-21
**Context**: SECURITY-AUDIT.md Action Item - Evaluate replacement for `rustpython-parser`
**Motivation**: `rustpython-parser` brings 6 unmaintained UNIC dependencies (RUSTSEC warnings)

## Executive Summary

**RECOMMENDATION**: ✅ **Migrate to tree-sitter-python** (already a project dependency)

**Impact**:
- ✅ Eliminates 6 RUSTSEC warnings (all UNIC crates)
- ✅ No new dependencies (tree-sitter-python already in use)
- ✅ Proven functionality (already used in 5 files)
- ✅ MIT/Apache-2.0 licensed (compatible)
- ⚠️ Requires refactoring 8 Rust files using rustpython-parser

**Timeline**: Medium complexity - Estimated 1-2 sprints

---

## Problem Statement

### Current Dependency: `rustpython-parser 0.4.0`

**Unmaintained Transitive Dependencies** (from SECURITY-AUDIT.md):
1. `unic-char-property` 0.9.0 (RUSTSEC-2025-0081)
2. `unic-char-range` 0.9.0 (RUSTSEC-2025-0075)
3. `unic-common` 0.9.0 (RUSTSEC-2025-0080)
4. `unic-emoji-char` 0.9.0 (RUSTSEC-2025-0090)
5. `unic-ucd-ident` 0.9.0 (RUSTSEC-2025-0100)
6. `unic-ucd-version` 0.9.0 (RUSTSEC-2025-0098)

**Status**: Optional dependency (only with `python-ast` feature)
**Usage**: 8 Rust files across AST parsing, mutation testing, and analysis

---

## Alternatives Evaluated

### Option 1: enderpy_python_parser ❌

**Source**: https://github.com/Glyphack/enderpy

**Evaluation**:
- ❌ **Not stable** - "breaking changes", "not ready to use" (per README)
- ❌ **AGPL-3.0 license** - Incompatible with MIT/Apache-2.0 dual licensing
- ❌ **Not published to crates.io** - 404 error on crates.io page
- ❌ **Experimental** - "Under active development"

**Verdict**: **REJECTED** - Licensing incompatibility & stability concerns

---

### Option 2: ruff_python_parser ❌

**Source**: https://github.com/astral-sh/ruff (internal crate)

**Evaluation**:
- ✅ **High quality** - Hand-written recursive descent parser (2x faster than LALRPOP)
- ✅ **Well-maintained** - Actively developed by Astral (Ruff team)
- ✅ **MIT licensed** - Compatible
- ❌ **Not published to crates.io** - Internal crate only
- ❌ **No public API** - Designed for Ruff's internal use

**Verdict**: **REJECTED** - Not available as standalone crate

---

### Option 3: tree-sitter-python ✅ **RECOMMENDED**

**Source**: https://github.com/tree-sitter/tree-sitter-python

**Evaluation**:
- ✅ **Already a dependency** - Version 0.23 (server/Cargo.toml:157)
- ✅ **Already in use** - 5 files successfully using it:
  - `server/src/services/semantic/chunker.rs`
  - `server/src/services/mutation/python_mutation_generator.rs`
  - `server/src/services/mutation/python_tree_sitter_mutations.rs`
  - `server/src/tdg/scorers/documentation.rs`
  - `server/src/tdg/language.rs`
- ✅ **Well-maintained** - Part of tree-sitter ecosystem
- ✅ **MIT licensed** - Compatible
- ✅ **No unmaintained dependencies** - Clean dependency tree
- ✅ **Proven track record** - Used by GitHub, Neovim, and many editors
- ✅ **Consistent API** - Matches other tree-sitter languages already in use

**Verdict**: **ACCEPTED** - Best option available

---

## Current tree-sitter-python Usage

### Working Example: `server/src/services/semantic/chunker.rs`

**Current functionality** (lines 335-397):
```rust
fn chunk_python_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|e| format!("Failed to set Python language: {e}"))?;

    let tree = parser.parse(source, None)
        .ok_or("Failed to parse Python source")?;
    let root = tree.root_node();

    let mut chunks = Vec::new();
    extract_python_items(root, source, &mut chunks);
    Ok(chunks)
}

fn extract_python_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    if node.kind() == "class_definition" {
        // Extract class with name
    }
    if node.kind() == "function_definition" {
        // Extract function with name and docstring
    }
    // Recursively process child nodes
}
```

**Capabilities demonstrated**:
- ✅ Parse Python source to AST
- ✅ Extract class definitions with names
- ✅ Extract function definitions with names
- ✅ Extract docstrings
- ✅ Recursive node traversal

---

## Migration Plan

### Files Requiring Migration (8 total)

1. **`server/src/ast/languages/python.rs`** (Primary implementation)
   - Convert `rustpython_parser::{ast, Parse}` → `tree_sitter + tree_sitter_python`
   - Rewrite `PythonAstVisitor` to use tree-sitter Node API
   - Update pattern matching from rustpython AST types to tree-sitter node kinds

2. **`server/src/services/unified_python_analyzer.rs`**
3. **`server/src/services/mutation/python_adapter.rs`**
4. **`server/src/services/duplicate_detector.rs`**
5. **`server/src/services/enhanced_python_visitor.rs`**
6. **`server/src/services/ast_python_compat.rs`**
7. **`server/src/tdg/analyzer_ast.rs`**
8. **`server/src/cli/analysis_utilities.rs`**

### API Mapping: rustpython → tree-sitter

| rustpython-parser | tree-sitter-python |
|-------------------|-------------------|
| `ast::ModModule::parse(content, filename)` | `parser.parse(source, None)` |
| `ast::Stmt::FunctionDef(f)` | `node.kind() == "function_definition"` |
| `ast::Stmt::ClassDef(c)` | `node.kind() == "class_definition"` |
| `ast::Stmt::Import(_)` | `node.kind() == "import_statement"` |
| `ast::Stmt::If(_)` | `node.kind() == "if_statement"` |
| Pattern matching on AST enums | String-based node kind matching |
| Direct field access (e.g., `f.name`) | `child_by_field_name("name")` |

### Testing Strategy

1. **Unit tests** - All existing Python AST tests must pass
2. **Integration tests** - `server/tests/storage_backend_tests.rs` and related
3. **Regression tests** - Compare AST output before/after migration
4. **Benchmark** - Measure parsing performance (tree-sitter is typically faster)

---

## Implementation Phases

### Phase 1: Proof of Concept (1-2 days)
- [ ] Rewrite `server/src/ast/languages/python.rs` to use tree-sitter
- [ ] Verify basic functionality (parse, extract functions/classes)
- [ ] Run unit tests for python.rs

### Phase 2: Full Migration (3-5 days)
- [ ] Migrate remaining 7 files
- [ ] Update tests to handle API changes
- [ ] Run full test suite

### Phase 3: Cleanup & Validation (1-2 days)
- [ ] Remove `rustpython-parser` dependency from Cargo.toml
- [ ] Remove `python-ast` feature (if no longer needed)
- [ ] Verify RUSTSEC warnings are gone (`cargo audit`)
- [ ] Update documentation

### Phase 4: Regression Testing (1 day)
- [ ] Compare AST extraction accuracy
- [ ] Benchmark performance
- [ ] Validate all Python analysis features

---

## Risk Assessment

### Low Risks

- ✅ **API differences** - tree-sitter API is well-documented and similar to existing usage
- ✅ **Dependency management** - tree-sitter already in use, no conflicts
- ✅ **Licensing** - MIT license compatible

### Medium Risks

- ⚠️ **AST structure differences** - tree-sitter produces concrete syntax trees (CST), not abstract syntax trees (AST)
  - **Mitigation**: chunker.rs already handles this successfully
  - **Impact**: May need to adjust node kind string matching

- ⚠️ **Test coverage** - 8 files to migrate, potential for regressions
  - **Mitigation**: Comprehensive test suite + regression testing
  - **Impact**: Require careful validation of each migration

### Zero High Risks

No critical blockers identified.

---

## Cost-Benefit Analysis

### Benefits

1. **Security** - Eliminates 6 RUSTSEC warnings
2. **Maintenance** - Reduces dependency on unmaintained crates
3. **Consistency** - Uses same parsing approach as other languages (C, C++, Rust, etc.)
4. **Performance** - tree-sitter is typically faster than recursive descent parsers
5. **Quality** - Well-tested, battle-proven parser used industry-wide

### Costs

1. **Development time** - 1-2 sprints for migration
2. **Testing effort** - Comprehensive regression testing required
3. **Code churn** - 8 files need refactoring

**Net Benefit**: **Positive** - Security and maintainability gains outweigh short-term migration cost

---

## Recommendation

✅ **PROCEED** with migration from `rustpython-parser` to `tree-sitter-python`

**Justification**:
1. tree-sitter-python is already a dependency and proven to work
2. Eliminates all 6 unmaintained UNIC dependencies
3. Aligns with existing language parsing strategy
4. Low risk, moderate effort, high value

**Next Steps**:
1. Create feature branch: `refactor/python-tree-sitter-migration`
2. Start with Phase 1 (proof of concept) on `python.rs`
3. Validate approach before full migration
4. Document API patterns for team

---

## References

- **SECURITY-AUDIT.md**: Lines 24-28 (rustpython-parser unmaintained dependencies)
- **Current usage**: `server/src/services/semantic/chunker.rs:335-397`
- **tree-sitter-python**: https://github.com/tree-sitter/tree-sitter-python
- **Rejected alternatives**:
  - enderpy: https://github.com/Glyphack/enderpy (AGPL-3.0)
  - ruff_python_parser: https://github.com/astral-sh/ruff/tree/main/crates/ruff_python_parser (unpublished)

---

**Evaluation By**: Claude Code
**Last Update**: 2025-10-21
