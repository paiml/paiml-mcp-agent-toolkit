# Quality & Testing

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 1

## TDG (Technical Debt Gradient)

### Scoring Model

TDG computes a composite score per function:

```
TDG = w_complexity * C + w_churn * H + w_coverage * (1 - V) + w_duplication * D
```

Where:
- **C** = cyclomatic complexity (normalized 0-1)
- **H** = churn score from git history (commits/timeframe)
- **V** = line coverage ratio
- **D** = duplication ratio from MinHash/LSH

### Grade Thresholds

| Grade | Score Range | Action Required |
|-------|-------------|-----------------|
| A | 0.0 - 0.2 | None |
| B | 0.2 - 0.4 | Monitor |
| C | 0.4 - 0.6 | Plan refactoring |
| D | 0.6 - 0.8 | Refactor next sprint |
| F | 0.8 - 1.0 | Immediate action |

### TDG Enforcement (CB-200)

`pmat comply` check CB-200 validates minimum grade gate:
- Default minimum: grade C
- Configurable via `.pmat.yaml`: `comply.checks.cb-200.threshold`
- `pmat tdg --explain` shows score decomposition

### Real-World Assessment

**TDG is the most actionable metric in PMAT.** Per-file granularity means developers know
exactly what to fix. Average score of 95.2/100 across 4435 files with clear grade distribution
(A+: 2658, A: 1595, F: 8) identifies the 8 problem files immediately. CB-200 grade gate
enforces minimum quality at commit time. No changes recommended.

**Versus Popper Score (87.5)**: TDG tells you "query_handler.rs has complexity 15 and 3% duplication."
Popper tells you "LICENSE exists." TDG is 10x more actionable per point.

**Versus Rust Project Score (71.2%)**: TDG is per-file, Rust Project Score is per-project.
Both useful at different zoom levels — TDG for daily development, Rust Project Score for
quarterly project health reviews.

### Churn-Weighted TDG Priority (Implemented)

New computed field for ranking which files to fix first:

```
tdg_priority = tdg_score * (1 + churn_factor)
```

Where `churn_factor` = normalized commit count in 90-day window (0.0-1.0).

**Rationale**: A file with TDG grade C that changes 3x/week is more urgent than a
file with TDG grade C that hasn't changed in 6 months. The static file's debt is
dormant; the active file's debt compounds with every change.

**Surfacing**: `pmat query --churn --rank-by priority` sorts by churn-weighted TDG.
Aliases: "priority", "tdg-priority", "churn-priority". Implementation in
`src/services/agent_context/query/engine_query.rs`. Also add to `pmat tdg --hotspots`
output so the top-10 list reflects actual urgency, not just static severity.

**Data source**: Git log already parsed for `--churn` enrichment. No new I/O needed.
Computation is O(1) per function (multiply two cached values).

### Transactional Updates

TDG scores use BLAKE3 hashing for incremental updates:
- Only recompute changed files (git diff)
- Store in `.pmat/context.db` SQLite
- Cache invalidation on HEAD hash change

## Test Coverage

### Targets

- **Minimum**: 95% line coverage (upgraded from 85%)
- **Tool**: `cargo llvm-cov` exclusively (never cargo-tarpaulin)
- **Command**: `make coverage` (canonical invocation)

### Coverage Workflow

```bash
# Find coverage gaps (MANDATORY approach)
pmat query --coverage-gaps --limit 30 --exclude-tests

# Target highest-impact functions first
pmat query --coverage-gaps --rank-by impact --limit 20

# Verify improvement
pmat query "function_name" --coverage --include-source --limit 1
```

### Coverage Checks

- `pmat comply` validates coverage >= 95%
- `.pmat/coverage-cache.json` stores per-function line data
- Cache invalidated on HEAD hash change
- MCP files (`mcp_pmcp/`, `mcp_server/`) have `coverage(off)` — excluded

### Pareto Analysis (80/20)

Coverage improvement prioritizes:
1. Functions with most uncovered lines (highest ROI)
2. Functions with highest PageRank (most important)
3. Functions with highest complexity (most bug-prone)

Impact score: `missed_lines * pagerank / complexity`

## Mutation Testing

### AST Fuzzing

Mutation operators:
- **Arithmetic**: `+` -> `-`, `*` -> `/`
- **Boolean**: `&&` -> `||`, `!` -> identity
- **Boundary**: `<` -> `<=`, `>=` -> `>`
- **Return**: `Ok(x)` -> `Err(...)`, `Some(x)` -> `None`
- **Deletion**: Remove statement, replace with `todo!()`

### Survival Prediction

ML-based prediction of mutation survival:
- Features: complexity, coverage, churn, TDG grade
- Model: Aprender gradient boosting
- Target: >80% mutation kill rate

## TDD Implementation

### Three-Interface Pattern

Every feature must have tests across:
1. **CLI**: `cargo test --lib -- cli_handler`
2. **MCP**: `cargo test --lib -- mcp_tool`
3. **HTTP**: `cargo test --lib -- http_endpoint`

### Test Organization

- Unit tests: same file as implementation
- Integration tests: `src/tests/` directory
- Property tests: proptest (never quickcheck)
- Coverage-off: `#[cfg_attr(coverage_nightly, coverage(off))]` for test modules

## Key Files

| File | Purpose |
|------|---------|
| `src/services/tdg/` | TDG scoring implementation |
| `src/services/tdg/tdg_graph.rs` | TdgGraph with O(1) dependency tracking |
| `src/cli/handlers/comply_handlers/check_handlers/check_tdg_grade.rs` | CB-200 TDG gate |
| `Makefile` (coverage target) | Canonical coverage invocation |

## References

- Consolidated from: tdg-specification, tdg-simplified-spec, tdg-enhanced-score,
  tdg-explain-mode, tdg-enforcement-system, transactional-hashed-tdg-spec,
  COVERAGE, 80-20-to-95, make-coverage-just-works, pmat-coverage-improve-command,
  mutant-fuzz-ast-testing, tdd-mcp-implementation
