# Code Quality & Analysis

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 13

## Five Whys Root Cause Analysis

### Toyota Way Methodology

```bash
pmat five-whys "Stack overflow in parser"
pmat why "Memory leak in cache" --depth 3
```

### Evidence Sources

| Source | Weight | Description |
|--------|--------|-------------|
| Complexity | 25% | Cyclomatic/cognitive complexity |
| TDG | 25% | Technical debt gradient |
| SATD | 20% | Self-admitted technical debt |
| Git churn | 20% | Change frequency |
| Dead code | 10% | Unused code ratio |

### Output Formats

- `--format text` (default): Human-readable analysis
- `--format json`: Machine-parseable for CI
- `--format markdown`: Documentation-ready
- `--auto-analyze`: Auto-run with project context

## Automated Clippy Fix

### Confidence Scoring

| Confidence | Action | Examples |
|-----------|--------|----------|
| High (>90%) | Auto-apply | Unused imports, redundant clones |
| Medium (50-90%) | Suggest with diff | Lifetime elision, type simplification |
| Low (<50%) | Report only | Complex refactorings |

### Production-Grade Pipeline

```bash
pmat auto-fix --confidence-threshold 90  # Only high-confidence fixes
pmat auto-fix --dry-run                   # Preview changes
```

## Popper Falsifiability Score

### 100-Point Scoring (6 Categories)

| Category | Points | What It Checks |
|----------|--------|----------------|
| A. Falsifiability & Testability | 25 | Claims testable, mutation testing, property tests, benchmarks |
| B. Reproducibility Infrastructure | 25 | Cargo.lock, Nix/devcontainer, Makefile, install docs |
| C. Transparency & Openness | 20 | LICENSE, README, API docs, CHANGELOG, ADRs |
| D. Statistical Rigor | 15 | Sample sizes, confidence intervals, effect sizes |
| E. Historical Integrity | 10 | CODEOWNERS, roadmap, release tags, semver |
| F. ML/AI Reproducibility | 5 | Model versioning, dataset docs, seed configs |

### Real-World Assessment

**Current score: 87.5/100 (A-).** Mostly infrastructure-existence checks (does LICENSE
exist? does CI run?). Overlap with Rust Project Score categories: Documentation (15 pts),
Rust Tooling & CI/CD (130 pts), and Testing Excellence (20 pts) cover similar ground.

**Recommendation**: Consider folding into Rust Project Score as a "Reproducibility &
Transparency" category rather than maintaining as a separate top-level command. The
falsifiability gateway (Category A >= 60%) is the most unique/valuable part.

## Entropy & Similarity Detection

### Code Entropy

Information-theoretic diversity measurement:
- Per-function token entropy
- File-level pattern diversity
- Module-level similarity clustering

### Actionable Insights

| Entropy | Interpretation | Action |
|---------|---------------|--------|
| <30% | Repetitive boilerplate | Extract abstraction |
| 30-80% | Normal variation | No action |
| >80% | Unique code | Review for consistency |

## Design-by-Contract (DBC)

### Assertion Generation

Automatically generates:
- Preconditions from function signatures
- Postconditions from return types
- Invariants from struct definitions

### Contract Types

```rust
#[requires(x > 0)]
#[ensures(result > 0)]
fn sqrt(x: f64) -> f64 { ... }
```

## Mutation Testing Enhancement

### ML-Based Survivability Prediction

Features for mutation survival prediction:
- Complexity of surrounding code
- Test coverage of target lines
- Historical churn rate
- TDG grade

### Targeted Mutation

Focus mutations on:
1. Uncovered code paths
2. High-complexity functions
3. Recently changed code
4. Boundary conditions

## Best Practices (Learn from Rust Giants)

Evidence-based patterns from high-scoring Rust projects:
- Error handling: `thiserror` for libraries, `anyhow` for applications
- Testing: property-based testing with proptest
- Documentation: doc-tests for all public APIs
- Performance: benchmark before optimizing

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/five_whys_handler.rs` | Five Whys implementation |
| `src/services/language_analyzer.rs` | Complexity analysis |
| `src/services/satd_detector.rs` | SATD detection |
| `src/services/lightweight_provability_analyzer.rs` | Provability analysis |

## References

- Consolidated from: auto-clippy-fix-guide, pmat-debug-five-whys,
  popper-nullification-100point-score, entropy, entropy-spec,
  enhance-pmat-mutation-spec, learn-from-rust-giants-spec, dbc, pmat-improve-safety
