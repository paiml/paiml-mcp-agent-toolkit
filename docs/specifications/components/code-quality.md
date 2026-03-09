# Code Quality & Analysis

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 13

## Five Whys Root Cause Analysis

### Toyota Way Methodology

```bash
pmat five-whys "Stack overflow in parser"
pmat why "Memory leak in cache" --depth 3
```

### Evidence Sources (v2 — Implemented, PMAT-510)

Reduced TDG overlap (was complexity 25% + TDG 25% = 50% redundant). Added temporal
and coverage signals that answer "is it getting worse?" not just "is it bad?"

| Source | Weight | Description |
|--------|--------|-------------|
| Complexity | 25% | Cyclomatic/cognitive complexity |
| SATD | 20% | Self-admitted technical debt |
| Git churn | 15% | Change frequency (reduced from 20% in v1) |
| EvoScore trajectory | 15% | Is the affected area improving or regressing? (CB-142) |
| Coverage delta | 15% | Did recent changes decrease coverage? |
| Dead code | 10% | Unused code ratio |

Removed direct TDG dependency (TDG already incorporates complexity + churn).
EvoScore reads `.pmat-metrics/commit-*-tests.json` (falls back to neutral if <3 commits).
Coverage delta reads `.pmat/coverage-cache.json` (falls back to neutral if unavailable).

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

### Planned: Absorb into Rust Project Score v3.0

**Phase 1 — Keep gateway, deprecate command**: `pmat popper-score` emits deprecation
warning pointing to `pmat rust-project-score`. Category A (Falsifiability >= 60%) becomes
a precondition in Rust Project Score — if it fails, the project gets grade F regardless
of other scores. This is the unique value: "are your claims testable?"

**Phase 2 — Fold B-F into Reproducibility category**: Categories B (Reproducibility
Infrastructure), C (Transparency), D (Statistical Rigor), E (Historical Integrity), and
F (ML/AI Reproducibility) become subchecks of the new 10-point Reproducibility category
in Rust Project Score v3.0. See repo-health.md for the full rebalanced design.

**What stays unique**: The falsifiability gateway. No other metric asks "can this project's
claims be disproven?" That's worth keeping as a hard gate.

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
| `src/cli/handlers/five_whys_handlers.rs` | Five Whys CLI handler |
| `src/services/language_analyzer.rs` | Complexity analysis |
| `src/services/satd_detector/mod.rs` | SATD detection |
| `src/services/lightweight_provability_analyzer.rs` | Provability analysis |

## References

- Consolidated from: auto-clippy-fix-guide, pmat-debug-five-whys,
  popper-nullification-100point-score, entropy, entropy-spec,
  enhance-pmat-mutation-spec, learn-from-rust-giants-spec, dbc, pmat-improve-safety
