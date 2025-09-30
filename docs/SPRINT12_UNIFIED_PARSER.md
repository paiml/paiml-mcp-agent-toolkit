# Sprint 12: Unified AST+Complexity Parser

## Executive Summary

**Goal**: Eliminate double file parsing by combining AST extraction and complexity analysis into a single pass.

**Expected Performance Gain**: 40-50% reduction in parse time for context generation

**Status**: Planning Phase

**Priority**: High - Direct impact on all `pmat context` users

## Problem Statement

### Current Architecture Issues

Currently, every Rust file is parsed **TWICE**:

1. **AST Analysis Pass** (`analyze_rust_file`)
   - File: `server/src/services/ast_rust_compat.rs:91`
   - Calls: `syn::parse_file(&content)` → Parses entire file
   - Extracts: Function names, structs, enums, traits via `EnhancedAstVisitor`
   - Output: `Vec<AstItem>`

2. **Complexity Analysis Pass** (`analyze_rust_file_with_complexity`)
   - File: `server/src/services/ast_rust_compat.rs:19`
   - Calls: `AccurateComplexityAnalyzer::analyze_file()` → Parses AGAIN
   - Extracts: Cyclomatic complexity, cognitive complexity
   - Output: `FileComplexityMetrics`

### Performance Impact

- **Redundant I/O**: Reading file twice from disk
- **Redundant Parsing**: `syn::parse_file()` is expensive (lexing, parsing, AST construction)
- **Memory Overhead**: Two separate AST representations in memory
- **CPU Waste**: 2x the parsing work for the same file

### Measured Baseline

On 48-core machine:
- Small project (3 files): 41ms total
- Multi-language project: 86ms total
- Estimated parse time: ~30-40% of total time

**Expected improvement**: 12-20ms savings (30-50% of parse time)

## Solution Design

### Unified Parser Architecture

```rust
pub struct UnifiedRustAnalyzer {
    file_path: PathBuf,
    syntax_tree: Option<syn::File>, // Parsed once, used twice
}

impl UnifiedRustAnalyzer {
    /// Single parse, dual extraction
    pub async fn analyze(&mut self) -> Result<UnifiedAnalysis> {
        // 1. Parse ONCE
        let content = tokio::fs::read_to_string(&self.file_path).await?;
        let syntax_tree = syn::parse_file(&content)?;
        self.syntax_tree = Some(syntax_tree);

        // 2. Extract AST items (existing EnhancedAstVisitor)
        let ast_items = self.extract_ast_items();

        // 3. Extract complexity metrics (new ComplexityVisitor)
        let complexity_metrics = self.extract_complexity();

        Ok(UnifiedAnalysis {
            ast_items,
            complexity_metrics,
        })
    }
}
```

### Key Benefits

1. **Single Parse**: Only one `syn::parse_file()` call per file
2. **Shared AST**: Both visitors operate on same parsed tree
3. **Memory Efficient**: One AST in memory, reused twice
4. **Backward Compatible**: Can replace existing functions transparently

## Implementation Plan

### Phase 1: Foundation (TICKET-3001)
**Estimated Time**: 2 hours

**Objective**: Create unified analyzer structure with EXTREME TDD

**Deliverables**:
- `server/src/services/unified_rust_analyzer.rs` - New module
- `UnifiedRustAnalyzer` struct
- `UnifiedAnalysis` result type
- Property-based tests (10+ test cases)

**Tests (RED Phase)**:
```rust
#[test]
fn red_unified_analyzer_parses_once() {
    // Must prove single parse call
}

#[test]
fn red_unified_analyzer_extracts_both_ast_and_complexity() {
    // Must return both types of data
}

#[test]
fn red_unified_analyzer_matches_existing_ast_output() {
    // AST items must match EnhancedAstVisitor exactly
}

#[test]
fn red_unified_analyzer_matches_existing_complexity_output() {
    // Complexity must match AccurateComplexityAnalyzer exactly
}
```

### Phase 2: Complexity Visitor (TICKET-3002)
**Estimated Time**: 3 hours

**Objective**: Extract complexity metrics from existing `syn::File` AST

**Deliverables**:
- `ComplexityVisitor` implementing `syn::visit::Visit`
- Cyclomatic complexity calculation
- Cognitive complexity calculation
- Halstead metrics (optional)

**Tests (RED Phase)**:
```rust
#[test]
fn red_complexity_visitor_calculates_cyclomatic() {
    // Simple function -> CC=1
    // Function with if -> CC=2
    // Function with if+else+while -> CC=4
}

#[test]
fn red_complexity_visitor_calculates_cognitive() {
    // Nesting depth increases cognitive complexity
}

#[test]
fn red_complexity_visitor_handles_edge_cases() {
    // Empty functions, macros, async, etc.
}
```

### Phase 3: Integration (TICKET-3003)
**Estimated Time**: 2 hours

**Objective**: Replace dual calls with unified analyzer

**Deliverables**:
- Update `analyze_rust_language()` in `deep_context.rs`
- Update `analyze_files_complexity()` to use unified analyzer
- Maintain backward compatibility

**Tests (RED Phase)**:
```rust
#[test]
fn red_integration_produces_identical_ast_output() {
    // Compare old vs new AST output
}

#[test]
fn red_integration_produces_identical_complexity_output() {
    // Compare old vs new complexity output
}

#[test]
fn red_integration_is_faster_than_dual_parse() {
    // Benchmark: unified must be faster
}
```

### Phase 4: Performance Validation (TICKET-3004)
**Estimated Time**: 1 hour

**Objective**: Measure and document performance gains

**Deliverables**:
- Benchmarks using `criterion`
- Performance comparison report
- Update ROADMAP.md

**Success Metrics**:
- ✅ 30-50% reduction in parse time
- ✅ All tests passing (0 failures)
- ✅ No regression in accuracy
- ✅ Memory usage same or better

## EXTREME TDD Methodology

### RED Phase Requirements

Each ticket MUST have:
1. **Failing Test First**: Write test that fails because feature doesn't exist
2. **Property-Based Tests**: Use `proptest` for edge case coverage
3. **Integration Tests**: Real-world file examples
4. **Benchmark Tests**: Performance regression tests

### GREEN Phase Requirements

1. **Minimal Implementation**: Just enough to pass tests
2. **No Premature Optimization**: Focus on correctness first
3. **All Tests Green**: 100% pass rate required

### REFACTOR Phase Requirements

1. **Code Quality**: Remove duplication, improve clarity
2. **Performance Tuning**: Optimize hot paths
3. **Documentation**: Inline comments for complex logic
4. **Tests Still Green**: No regression

## Risk Assessment

### Low Risk
- ✅ Backward compatible (can keep old functions)
- ✅ Incremental rollout possible
- ✅ Easy to A/B test

### Medium Risk
- ⚠️ Complexity calculation differences (mitigate with extensive tests)
- ⚠️ syn::visit trait requires deep knowledge

### Mitigation Strategies

1. **Keep Both Implementations**: Old functions remain until unified is proven
2. **Feature Flag**: `unified-parser` feature for gradual rollout
3. **Extensive Testing**: Property-based + integration + real-world files
4. **Benchmark Suite**: Continuous performance monitoring

## Success Criteria

### Must Have
- [ ] Single parse per file (measured)
- [ ] AST output matches existing (100% identical)
- [ ] Complexity output matches existing (within 5% tolerance)
- [ ] 30%+ faster than dual parse (benchmarked)
- [ ] All tests passing (0 failures)

### Should Have
- [ ] Memory usage ≤ dual parse
- [ ] Support for all Rust syntax (macros, async, etc.)
- [ ] Graceful error handling

### Nice to Have
- [ ] 50%+ faster than dual parse
- [ ] Halstead metrics included
- [ ] Extended to TypeScript/Python

## Timeline

**Total Estimated Time**: 8 hours

| Phase | Ticket | Time | Dependencies |
|-------|--------|------|--------------|
| 1. Foundation | TICKET-3001 | 2h | None |
| 2. Complexity | TICKET-3002 | 3h | TICKET-3001 |
| 3. Integration | TICKET-3003 | 2h | TICKET-3002 |
| 4. Validation | TICKET-3004 | 1h | TICKET-3003 |

**Sprint Duration**: 1-2 days with full focus

## Rollout Plan

### Stage 1: Development (Day 1)
- Complete TICKET-3001 through TICKET-3004
- All tests green
- Benchmarks passing

### Stage 2: Alpha Testing (Day 1-2)
- Enable `unified-parser` feature flag
- Test on internal projects
- Compare outputs with old implementation

### Stage 3: Beta Release (Day 2)
- Merge to master behind feature flag
- Monitor for issues
- Collect performance data

### Stage 4: Production (Day 3)
- Make unified parser the default
- Remove old implementation (or deprecate)
- Update documentation

## Metrics Dashboard

```
Implementation Progress: ░░░░░░░░░░░░░░░░░░░░  0%
Test Coverage:          ░░░░░░░░░░░░░░░░░░░░  0%
Performance Gain:       Target: 40-50%
```

## References

### Key Files
- `server/src/services/ast_rust_compat.rs` - Current implementation
- `server/src/services/enhanced_ast_visitor.rs` - AST extraction
- `server/src/services/accurate_complexity_analyzer.rs` - Complexity calculation
- `server/src/services/deep_context.rs` - Integration point

### Documentation
- [syn crate docs](https://docs.rs/syn/)
- [Visit trait](https://docs.rs/syn/latest/syn/visit/trait.Visit.html)
- [Cyclomatic Complexity](https://en.wikipedia.org/wiki/Cyclomatic_complexity)
- [Cognitive Complexity](https://www.sonarsource.com/docs/CognitiveComplexity.pdf)

---

**Created**: 2025-09-30
**Status**: Planning
**Sprint**: 12
**Assigned**: TBD
**Methodology**: EXTREME TDD