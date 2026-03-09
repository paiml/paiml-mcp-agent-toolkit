# Quality Gates

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 2

## O(1) Quality Gate Enforcement

### Architecture

Pre-commit hooks validate cached metrics in <30ms instead of re-running full analysis.

```
Developer commits -> Pre-commit hook -> Read .pmat-metrics/ -> Compare thresholds -> Pass/Fail
```

### Metric Recording

Metrics recorded during development populate `.pmat-metrics/`:
- `make lint` -> lint duration
- `make test-fast` -> test duration
- `make coverage` -> coverage percentage
- `make release` -> binary size, dependency count

### Thresholds (`.pmat-metrics.toml`)

| Metric | Threshold | Staleness |
|--------|-----------|-----------|
| lint | <=30s | 7 days |
| test-fast | <=5min | 7 days |
| coverage | <=10min | 7 days |
| binary size | <=50MB | 7 days |
| dependencies | <=3,000 | 7 days |

### Staleness Policy

- Metrics older than 7 days trigger warnings
- Emergency bypass: `git commit --no-verify`
- Metric files: `.pmat-metrics/commit-<sha>-meta.json`

## Phase 3.2: Trueno-Graph Integration

### CSR Graph Database

trueno-graph provides:
- O(1) symbol lookups via HashMap + CSR
- PageRank-based importance scoring
- Bidirectional NodeId mapping

### Integration Points

1. **Context Generation** (`context.rs`, `context_graph.rs`):
   - `analyze_project_with_cache()` builds ProjectContextGraph
   - 8/8 tests passing

2. **TDG Analysis** (`tdg/tdg_graph.rs`):
   - TdgGraph provides O(1) function dependency tracking
   - PageRank criticality scoring
   - 7/7 tests passing

### Key Insight

CSR graphs only track nodes with edges. Use `node_map.len()` not `graph.num_nodes()`.

## Phase 4: Predictive Quality Gates

### ML-Based Early Detection

Using Aprender ML for predicting quality issues:
- Features: complexity trend, churn velocity, TDG trajectory
- Model: gradient boosting on historical data
- Prediction: probability of CI failure before full suite runs

### Triggered Actions

| Prediction | Confidence | Action |
|-----------|------------|--------|
| CI will pass | >90% | Skip slow tests |
| CI will fail | >80% | Block commit, suggest fixes |
| Uncertain | <80% | Run full suite |

## Pre-Commit Complexity Gates

### Thresholds

- Cyclomatic complexity: <=30 per function
- Cognitive complexity: <=25 per function

### Implementation

Hook runs `pmat analyze complexity --file` using `language_analyzer.rs` RustAnalyzer.

### Known Issues

- `include!()` helper files cause false positives (tree-sitter fallback)
- Solution: inline methods in main file
- Cascading violations: fix ALL in same file at once

## Key Files

| File | Purpose |
|------|---------|
| `.pmat-metrics/` | Cached metric values |
| `.pmat-metrics.toml` | Threshold configuration |
| `src/services/context_graph.rs` | trueno-graph integration |
| `src/services/tdg/tdg_graph.rs` | TDG O(1) graph |

## References

- Consolidated from: quick-test-build-O(1)-checking, O1-quality-gates-phase-3.2-trueno-graph,
  o1-quality-gates-phase3.2-trueno-graph, o1-quality-gates-phase4-predictive, quality-gate-specification
