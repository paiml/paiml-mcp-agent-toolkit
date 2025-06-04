# QA Provability Release Specification

## Overview

This specification defines the provability metrics and quality assurance requirements for AST analysis in the PAIML MCP Agent Toolkit. All code must be provably correct and analyzable with zero tolerance for slow tests.

## Core Requirements

### 1. Performance Requirements (Mandatory)

- **Test Execution Time**: ALL tests MUST complete in < 5 seconds
- **Zero Tolerance Policy**: NO slow tests allowed in codebase
- **Enforcement**: `make test` only runs fast tests via `cargo nextest`

### 2. AST Provability Metrics

The system must track and report provability confidence for all AST analysis:

```rust
pub struct ProvabilityMetrics {
    /// Total AST nodes analyzed
    pub total_nodes: u64,
    
    /// Nodes with high-confidence proofs (machine-checkable)
    pub proven_nodes: u64,
    
    /// Nodes with medium-confidence analysis (static analysis)
    pub analyzed_nodes: u64,
    
    /// Nodes with low-confidence heuristics
    pub heuristic_nodes: u64,
    
    /// Overall provability score (0.0 - 1.0)
    pub provability_score: f64,
}
```

### 3. Proof Annotation Requirements

All AST nodes MUST support proof annotations as defined in `models/unified_ast.rs`:

- **Memory Safety**: Rust borrow checker guarantees
- **Thread Safety**: Send/Sync trait compliance
- **Data Race Freedom**: No mutable aliasing
- **Termination**: Bounded loops and recursion
- **Functional Correctness**: Contract specifications

### 4. Quality Metrics

Each release must meet these quality thresholds:

```yaml
quality_thresholds:
  ast_coverage: 90%           # % of code with AST analysis
  provability_score: 0.75     # 75% of nodes have proofs
  test_coverage: 80%          # Line coverage requirement
  max_test_time: 5s           # No test exceeds 5 seconds
  zero_ignored_tests: true    # No #[ignore] allowed
```

### 5. Release Validation

Before any release:

1. **Fast Test Validation**
   ```bash
   make test-fast  # Must complete in < 30s total
   ```

2. **Provability Report**
   ```bash
   ./target/release/pmat analyze provability --threshold 0.75
   ```

3. **Performance Verification**
   ```bash
   make benchmark-all-interfaces  # Verify sub-second response times
   ```

### 6. Continuous Enforcement

- **Pre-commit Hook**: Reject commits with slow tests
- **CI Pipeline**: Fail builds exceeding time limits
- **Monitoring**: Track provability metrics over time

## Implementation Status

### Completed
- [x] Fast test infrastructure (cargo nextest)
- [x] Proof annotation system in UnifiedAstNode
- [x] Zero-tolerance Makefile updates
- [x] Test conversions to use temp directories

### In Progress
- [ ] Provability metric calculation
- [ ] Release validation script
- [ ] CI pipeline updates

### Future Work
- [ ] Formal verification integration
- [ ] Proof certificate generation
- [ ] Machine-checkable proof export

## Usage

To validate a release candidate:

```bash
# 1. Run all fast tests
make test

# 2. Generate provability report
./target/release/pmat analyze provability --output provability-report.json

# 3. Verify all thresholds met
./scripts/validate-release.ts provability-report.json
```

## References

- [Toyota Way Implementation](../docs/bugs/annotated-ast-bugs-june4.md)
- [AST Unified Model](../server/src/models/unified_ast.rs)
- [Performance Requirements](../CLAUDE.md#performance-engineering)