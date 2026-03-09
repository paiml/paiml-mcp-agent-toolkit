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
- `n(c₀)` = passing tests in base state (first commit in window)
- `n(c*)` = passing tests in oracle state (max observed across all commits)

**Properties**:
- `a(c) = 1` means complete gap closure (reached oracle level)
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

**Status**: Planned (not yet implemented). Will be a dual-agent protocol.

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

## PMAT Integration

### Data Recording

**Status**: No automated recording command exists yet. Data must be written
manually or by CI scripts.

Test results are stored as JSON files in `.pmat-metrics/`:

**Primary format** (`commit-<sha>-tests.json`):
```json
{"commit": "abc123", "pass": 19500, "total": 19700}
```

**Fallback format** (`commit-<sha>-meta.json` with `tests` key):
```json
{"commit": "abc123", "tests": {"pass": 19500, "total": 19700}}
```

Files are sorted lexicographically by filename to determine chronological
order. Use zero-padded or timestamp-prefixed names if order matters.

### Recording Methods

**CI pipeline (recommended)**:
```bash
# In .github/workflows/ci.yml or post-test hook:
SHA=$(git rev-parse --short HEAD)
RESULT=$(cargo test 2>&1 | grep "test result:")
PASS=$(echo "$RESULT" | grep -oP '\d+ passed' | grep -oP '\d+')
FAIL=$(echo "$RESULT" | grep -oP '\d+ failed' | grep -oP '\d+')
TOTAL=$((PASS + FAIL))
echo "{\"commit\":\"$SHA\",\"pass\":$PASS,\"total\":$TOTAL}" \
  > .pmat-metrics/commit-${SHA}-tests.json
```

**Manual seeding** (for bootstrapping):
```bash
# Record current test state
cargo test 2>&1 | grep "test result"
# → test result: ok. 19795 passed; 0 failed; 167 ignored
echo '{"commit":"current","pass":19795,"total":19795}' \
  > .pmat-metrics/commit-$(date +%Y%m%d)-tests.json
```

### Configuration

CB-142 is registered in `comply_config_defaults.rs` with:
- **Severity**: Info
- **Threshold**: 0.5 (minimum EvoScore for Pass)

The following are hardcoded in the implementation (not yet configurable):
- **gamma**: 1.5
- **min_commits**: 3

## Comply Check: CB-142

### Data Loading

1. Scan `.pmat-metrics/` for `commit-*-tests.json` files
2. Parse each: extract `pass` and `total` fields, skip if `total == 0`
3. If no `-tests.json` found, fall back to `commit-*-meta.json` files
   with a `tests` sub-object containing `pass` and `total`
4. Sort files by filename (lexicographic) for chronological order
5. If fewer than 3 data points, return Skip

### Computation

1. **Base state** (`c₀`): first file's pass count
2. **Oracle** (`c*`): max pass count across all files
3. For each subsequent commit `c_i` (i=1..N):
   - If `pass >= base_pass`: `a(c) = (pass - base) / (oracle - base)`
     (or 1.0 if oracle == base)
   - If `pass < base_pass`: `a(c) = (pass - base) / base`
     (or 0.0 if base == 0)
4. Weight: `w_i = gamma^i` where i is the 0-based index in the sequence
5. EvoScore: `sum(w_i * a_i) / sum(w_i)`

### Scoring

| EvoScore | CB-142 Status | Severity |
|----------|---------------|----------|
| >= 0.5 | Pass | Info |
| 0.0 - 0.5 | Warn | Warning |
| < 0.0 | Fail | Error |
| < 3 data points | Skip | Info |

### Score Interpretation

| EvoScore | Interpretation |
|----------|---------------|
| 0.8 - 1.0 | Excellent: consistent improvement, no regressions |
| 0.5 - 0.8 | Good: net positive with minor regressions |
| 0.0 - 0.5 | Fair: improvements offset by regressions |
| -0.5 - 0.0 | Poor: net regression trend |
| -1.0 - -0.5 | Critical: systemic quality degradation |

### Numerical Example

5 commits: pass counts [18000, 18500, 18200, 19000, 19500], total=19700 each.

```
base = 18000, oracle = 19500, gap = 1500

a(c₁) = (18500 - 18000) / 1500 = 0.333
a(c₂) = (18200 - 18000) / 1500 = 0.133
a(c₃) = (19000 - 18000) / 1500 = 0.667
a(c₄) = (19500 - 18000) / 1500 = 1.000

weights (γ=1.5): γ¹=1.5, γ²=2.25, γ³=3.375, γ⁴=5.0625
numerator:   1.5(0.333) + 2.25(0.133) + 3.375(0.667) + 5.0625(1.000) = 8.114
denominator: 1.5 + 2.25 + 3.375 + 5.0625 = 12.1875
EvoScore:    8.114 / 12.1875 = 0.666 → Pass
```

### Gamma Selection Guide

| Project Phase | Recommended Gamma | Rationale |
|--------------|-------------------|-----------|
| Greenfield | 1.0 | Equal weight, expect volatile early history |
| Growth | 1.2 | Slight forward bias, reward stabilization |
| Mature | 1.5 | Penalize regressions in established codebase |
| Legacy rescue | 2.0 | Heavily reward sustained improvement |

## Relationship to TDG

EvoScore and TDG are **independent, complementary metrics** — not candidates for merging.

| Dimension | TDG (CB-200) | EvoScore (CB-142) |
|-----------|-------------|-------------------|
| Question answered | "Is this code well-structured now?" | "Is the project improving over time?" |
| Granularity | Per-file (4435 files) | Per-project (single scalar) |
| Input | AST, source code | Test pass/fail across commits |
| Timescale | Instantaneous snapshot | Rolling window (90 days) |
| Determinism | Same source = same score | Depends on git history + CI state |
| Toyota Way | Jidoka (stop-the-line) | Kaizen (continuous improvement) |

**Why they must stay separate:**
1. TDG is per-file; EvoScore is per-project — no meaningful per-file EvoScore without per-file test attribution
2. TDG is deterministic and cached for O(1) pre-commit gates; EvoScore needs disk I/O across N commit files
3. Combining them loses both signals — a project can have excellent TDG but stagnant EvoScore (or vice versa)

### Future Cross-Metric Work

These extensions bridge the gap without merging the metrics:

**Churn-weighted TDG** — `tdg_priority = tdg_score * git_churn_factor`. High-TDG files
that change often are worse than high-TDG files that never change. Stays per-file and
deterministic for a given commit. Data source: `pmat query --churn`.

**Per-function EvoScore** — Track individual function test coverage trajectories across
commits. Requires mapping test failures to specific functions (via coverage data). Would
enable "this function's tests are regressing" alerts at the function level.

**Dashboard correlation** — Show TDG grade distribution trends alongside EvoScore in
`pmat rust-project-score` output. Let humans observe the relationship without
algorithmically coupling the metrics. E.g., "TDG avg improved 2.3 points while EvoScore
held at 0.7 — quality improving without regression."

## Planned: `pmat test --record`

### Design

New subcommand that wraps `cargo test`, parses output, and writes test data:

```bash
pmat test --record                    # Run cargo test, write results
pmat test --record --dry-run          # Show what would be recorded
pmat test --record --from-stdin       # Parse piped cargo test output
```

### Implementation

1. Run `cargo test --no-fail-fast 2>&1` and capture output
2. Parse summary line: `test result: ok. N passed; M failed; K ignored`
3. Get current commit: `git rev-parse --short HEAD`
4. Write `.pmat-metrics/commit-<sha>-tests.json`:
   ```json
   {"commit":"abc1234","pass":19795,"total":19795,"failed":0,"ignored":167,
    "timestamp":"2026-03-09T14:00:00Z"}
   ```
5. Print summary: "Recorded: 19795/19795 pass (commit abc1234)"

### Makefile Integration

```makefile
test-record:
	cargo test --no-fail-fast 2>&1 | pmat test --record --from-stdin
```

### Per-Function EvoScore (Phase 2)

Track coverage trajectory per function across commits:

1. After `pmat test --record`, also run `cargo llvm-cov --json`
2. For each function, record `{function, covered_lines, total_lines}`
3. Store in `.pmat-metrics/commit-<sha>-coverage.json`
4. CB-142 computes per-function EvoScore: which functions are losing coverage?
5. Surface via `pmat query --coverage --evoscore` — shows functions trending down

This bridges the granularity gap: project EvoScore says "regressing",
per-function EvoScore says "parser::tokenize lost 12 lines of coverage in 3 commits".

### Configurable Gamma (Phase 2)

Read from `.pmat.yaml` options map instead of hardcoding:

```rust
let gamma = comply_config
    .and_then(|c| c.checks.get("cb-142"))
    .and_then(|c| c.options.get("gamma"))
    .and_then(|v| v.parse::<f64>().ok())
    .unwrap_or(1.5);
```

Requires threading `comply_config` into `check_swe_ci_evoscore()`.

## Implementation Status

| Feature | Status | Location |
|---------|--------|----------|
| EvoScore math | Implemented | `check_mono_spec.rs:296-332` |
| `-tests.json` loading | Implemented | `check_mono_spec.rs:224-251` |
| `-meta.json` fallback | Implemented | `check_mono_spec.rs:254-281` |
| CB-142 comply check | Implemented | `check_mono_spec.rs:217-359` |
| Unit tests (3 cases) | Implemented | `check_mono_spec.rs:461-500` |
| `pmat test --record` | Implemented | `src/cli/command_dispatcher/test_record.rs` |
| Configurable gamma | Not implemented | Hardcoded to 1.5 |
| Configurable window | Not implemented | No time-based filtering |
| CI source: github | Not implemented | Local files only |
| Architect-Programmer | Planned | Dual-agent protocol (future work) |

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/comply_handlers/check_handlers/check_mono_spec.rs` | CB-142 implementation |
| `src/models/comply_config_defaults.rs` | CB-142 registration (severity: Info, threshold: 0.5) |
| `.pmat-metrics/commit-*-tests.json` | Primary test data (not yet generated) |
| `.pmat-metrics/commit-*-meta.json` | Fallback data (exists but lacks `tests` key) |

## References

- SWE-CI: arxiv:2603.03823 (Sun Yat-sen University, Alibaba Group, March 2026)
- SWE-EVO: arxiv:2512.18470 (Long-horizon software evolution scenarios)
- SWE-Bench: https://www.swebench.com/ (Original SWE benchmark)
