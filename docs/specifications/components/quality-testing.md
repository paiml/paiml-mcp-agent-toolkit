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
