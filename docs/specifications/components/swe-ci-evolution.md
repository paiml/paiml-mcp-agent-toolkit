# SWE-CI & Evolution

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 20
> Based on: SWE-CI (arxiv:2603.03823, March 2026)

## Overview

Evolution-based code quality evaluation that measures long-term maintainability
through iterative CI loops rather than one-shot functional correctness.

## Core Concepts

### Normalized Change Metric a(c)

Quantifies progress toward the oracle (ideal) codebase:

```
a(c) = {
  [n(c) - n(c₀)] / [n(c*) - n(c₀)]   if n(c) >= n(c₀)   (improvement)
  [n(c) - n(c₀)] / n(c₀)              if n(c) <  n(c₀)   (regression)
}
```

Where:
- `n(c)` = number of passing tests in codebase state c
- `n(c₀)` = passing tests in base state (before changes)
- `n(c*)` = passing tests in oracle state (ideal target)

**Properties**:
- `a(c) = 1` means complete gap closure (all oracle tests pass)
- `a(c) = 0` means no progress from baseline
- `a(c) = -1` means all originally-passing tests now fail

### EvoScore

Aggregates performance across N iterations using future-weighted mean:

```
e = [sum_{i=1}^{N} gamma^i * a(c_i)] / [sum_{i=1}^{N} gamma^i]
```

Where:
- `gamma >= 1` weights later iterations more heavily
- `gamma = 1` reduces to simple average
- Higher gamma favors long-term stability over short-term gains

**Interpretation**: A truly maintainable codebase remains easy to modify as
evolution progresses. EvoScore penalizes early gains that create technical debt.

### CI Loop Model

Each iteration follows the require-code cycle:

```
r_i = require(c_i, c*)     -- derive requirements from test gap
c_{i+1} = code(r_i, c_i)   -- implement changes based on requirements
```

This iterative loop ensures consequences of earlier modifications propagate
into subsequent iterations, making long-term decision quality observable.

## Architect-Programmer Protocol

### Architect Agent

Three-step analysis:
1. **Summarize**: Review failing tests, identify root causes
2. **Locate**: Examine source code, attribute failures to implementation gaps
3. **Design**: Devise improvement plan, produce requirements document

### Programmer Agent

Three-step implementation:
1. **Comprehend**: Translate requirements into code specifications
2. **Plan**: Outline programming effort needed
3. **Code**: Implement specifications

**Key insight**: The Programmer is driven by the requirements document, not
directly by the test gap. This aligns with CI's rapid iteration philosophy.

## PMAT Integration

### EvoScore from Git History

PMAT computes EvoScore from real git history + CI results:

```
For each commit c_i in range [base..HEAD]:
  1. Count passing tests: n(c_i) from CI results or local test run
  2. Compute a(c_i) relative to base commit
  3. Accumulate into EvoScore with gamma weighting
```

### Data Sources

| Source | Purpose |
|--------|---------|
| `git log` | Commit sequence and timestamps |
| `.pmat-metrics/commit-*.json` | Per-commit test results |
| CI pipeline results | Pass/fail counts per commit |
| `.pmat/context.db` | Function-level quality metrics |

### Configuration

```yaml
# .pmat.yaml
comply:
  checks:
    cb-142:
      enabled: true
      severity: info
      options:
        gamma: 1.5          # Future-weighting factor (1.0 = equal, higher = penalize early debt)
        window: 90           # Days of git history to analyze
        min_commits: 10      # Minimum commits for meaningful score
        ci_source: "local"   # "local" (run tests) or "github" (fetch CI results)
```

### Score Interpretation

| EvoScore | Interpretation |
|----------|---------------|
| 0.8 - 1.0 | Excellent: consistent improvement, no regressions |
| 0.5 - 0.8 | Good: net positive with minor regressions |
| 0.0 - 0.5 | Fair: improvements offset by regressions |
| -0.5 - 0.0 | Poor: net regression trend |
| -1.0 - -0.5 | Critical: systemic quality degradation |

## Comply Check: CB-142

### Computation

CB-142 computes EvoScore over the configured window:

1. Enumerate commits in window: `git log --since="90 days ago" --format="%H"`
2. For each commit, load or compute test pass count from `.pmat-metrics/`
3. Compute `a(c_i)` for each commit relative to oldest commit in window
4. Compute weighted EvoScore with configured gamma

### Scoring

| EvoScore | CB-142 Status | Severity |
|----------|---------------|----------|
| >= 0.5 | Pass | Info |
| 0.0 - 0.5 | Warn | Warning |
| < 0.0 | Fail | Error |

### Fallback

If insufficient data (< min_commits), CB-142 returns Skip with message:
"Insufficient commit history for EvoScore (need >= 10 commits with test data)"

## Implementation Notes

### Computing n(c) Efficiently

For local computation without full CI replay:
1. Use `cargo test --no-fail-fast 2>&1 | grep "test result"` to count pass/fail
2. Cache results in `.pmat-metrics/commit-<sha>-tests.json`
3. For historical commits, only compute if cached data missing

### Gamma Selection Guide

| Project Phase | Recommended Gamma | Rationale |
|--------------|-------------------|-----------|
| Greenfield | 1.0 | Equal weight, expect volatile early history |
| Growth | 1.2 | Slight forward bias, reward stabilization |
| Mature | 1.5 | Penalize regressions in established codebase |
| Legacy rescue | 2.0 | Heavily reward sustained improvement |

## Future Work

1. **Per-function EvoScore**: Track evolution at function granularity
2. **Cross-project EvoScore**: Compare evolution trajectories across repos
3. **Predictive EvoScore**: ML model predicting future trajectory
4. **Architect-Programmer mode**: `pmat evolve` command implementing the dual-agent protocol

## References

- SWE-CI: arxiv:2603.03823 (Sun Yat-sen University, Alibaba Group, March 2026)
- SWE-EVO: arxiv:2512.18470 (Long-horizon software evolution scenarios)
- SWE-Bench: https://www.swebench.com/ (Original SWE benchmark)
