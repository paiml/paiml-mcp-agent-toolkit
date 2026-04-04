# Scoring Convergence & Hardening

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 21

## Problem Statement

Four scoring commands exist independently. None share analysis, none persist
results, none gate each other:

| Command | Scale | Writes metrics? | Gates anything? |
|---------|-------|-----------------|-----------------|
| `pmat comply check` | pass/fail + exit code | `.pmat/project.toml` (config + timestamp) | Exit code only |
| `pmat rust-project-score` | 274 pts, grade A-F | Nothing | Nothing |
| `pmat repo-score` | 0-110, separate formula | Optional `--output` file, `--update-badge` | Nothing |
| `pmat work score <id>` | DBC 5-dim, 0-1 | `.pmat-work/` contract files | `work complete` only |

Comply checks for pattern presence (e.g., CB-501 counts `unwrap()` calls,
CB-503 checks `.clippy.toml` existence) while RPS runs actual tools (e.g.,
`cargo clippy`, `cargo audit`). Both analyze dead code and dependency counts
independently. Neither writes to `.pmat-metrics/`, so there is no historical
trend data and no regression detection.

`pmat repo-score` is a third overlapping scorer with yet another formula.

## Design Principle: One Canonical Score Path

**`pmat score` is the single entry point.** All other scoring commands become
internal subsystems that feed into it. Users run ONE command. CI gates on
ONE number.

```
pmat score          ← the ONLY command users/CI should run
  ├── runs comply check internally (CB-xxx checks)
  ├── runs rust-project-score internally (11 category scorers)
  ├── reads coverage from cache or runs cargo llvm-cov
  ├── reads DBC portfolio from .pmat-work/
  ├── computes geometric composite
  ├── writes ALL results to .pmat-metrics/commit-<sha>-meta.json
  └── exits 0/1 based on gate threshold
```

### Deprecation Status

`pmat score` is implemented. Current status:

| Command | Status | Migration |
|---------|--------|-----------|
| `pmat comply check` | Keep as standalone + internal subsystem | Also feeds `pmat score` |
| `pmat rust-project-score` | Keep as standalone + internal subsystem | Also feeds `pmat score` |
| `pmat repo-score` | **Deprecated** — `score` alias freed | Replaced by `pmat score` |
| `pmat work score` | Keep | Per-contract scoring, feeds DBC sub-score |
| `pmat work codebase-score` | Keep | Portfolio aggregate, feeds DBC sub-score |

`pmat comply check` and `pmat rust-project-score` remain available as
standalone commands for debugging. `pmat score` is the canonical path
for gates, CI, and hooks.

## 1. Composite Score (`pmat score`)

### Sub-Score Normalization

| Sub-Score | Source | Normalization |
|-----------|--------|---------------|
| RPS | `RustProjectScoreOrchestrator` | Category-average % (already normalized) |
| Comply | `pmat comply check --format json` | `100 - (errors * 10 + warnings * 3)`, floor 0 |
| Coverage | `.pmat-metrics/coverage.result` | Line coverage % |
| Muda | CB-300 (size-normalized) | `100 - muda_score` (lower waste = higher) |
| EvoScore | `.pmat-metrics/commit-*-tests.json` | `pass/total * 100` |
| DBC | `compute_codebase_score()` | 50% coverage + 30% lint + 20% score |
| File Health | density-based | `(1 - over1000/total) * 100` |
| PV Lint | `pv lint --format json` (planned) | 3-gate pass rate * 100 |

### Formula

```
composite = (rps * comply * coverage * muda_inv * evo * dbc * file_health) ^ (1/7)
```

When PV Lint is integrated (§10), it becomes the 8th sub-score and
the exponent changes to 1/8.

Geometric mean: one zero kills the composite.

### Orchestration

`pmat score` runs both subsystems and aggregates:

```
1. Build agent context index (if stale)
2. Run comply checks → produces ComplianceReport
   - Extract: error_count, warning_count, muda_score, evoscore,
     file_health, dead_code_pct from individual check results
3. Run RPS scorers → produces CategoryScore[]
   - Note: comply pattern-checks and RPS tool-runs are complementary,
     not duplicative (comply checks .clippy.toml exists, RPS runs clippy)
4. Read coverage cache (or run cargo llvm-cov if stale)
5. Read DBC portfolio from .pmat-work/
6. Compute composite from all sub-scores
7. Write everything to .pmat-metrics/commit-<sha>-meta.json
8. Run cross-validation invariants (XV-001 through XV-010)
9. Run regression check against previous commits
10. Print report, exit 0/1
```

### Metric Persistence

Every `pmat score` run writes to `.pmat-metrics/commit-<sha>-meta.json`:

```json
{
  "sha": "abc123",
  "timestamp": "2026-03-20T10:30:00Z",
  "composite": 72.4,
  "sub_scores": {
    "rps": 69.4,
    "comply": 85.0,
    "coverage": 94.2,
    "muda_inv": 63.7,
    "evoscore": 84.8,
    "dbc": 78.0,
    "file_health": 71.0
  },
  "rps_categories": { ... },
  "comply_errors": 2,
  "comply_warnings": 7,
  "cross_validation": { "passed": 8, "failed": 2, "violations": ["XV-001", "XV-008"] }
}
```

This is the ONLY place scores are persisted. All trend, regression, and
CI reporting reads from this file.

### CLI

```bash
pmat score                           # Full run: comply + RPS + composite
pmat score --gate 70                 # Exit 1 if composite < 70
pmat score --format json             # Machine-readable for CI
pmat score --trend                   # Sparkline of last 10 commits
pmat score --regression-check        # Exit 1 if regression detected
```

## 2. Deep Integration with `pmat query`

`pmat query` already enriches results with TDG grade, coverage, churn,
faults, and duplicates. The composite score extends this in three ways:

### 2a. Score-Aware Ranking (`--rank-by score`)

Functions ranked by how much they contribute to (or drag down) the
composite score. Uses the sub-scores as weights:

```bash
pmat query "handler" --rank-by score --limit 10
# Ranks by: tdg_weight * churn_weight * (1 - fault_count/10)
# Functions that are high-TDG, high-churn, and have faults rank first
# These are the functions most likely to improve the composite if fixed
```

### 2b. Score-Gated Search (`--min-score`)

Filter query results to only functions in files that contribute to
sub-score failures. When composite is D (66.9), the bottleneck is
Code Quality (26.9%) and Muda (63.7%). `--min-score` surfaces functions
in files contributing to those weak categories:

```bash
pmat query "parse" --min-score 70    # Only functions in files scoring >= 70
pmat query "cache" --below-score 50  # Only functions in files dragging score down
```

### 2c. Score Diagnosis (`pmat query --score-diagnosis`)

Shows which functions are most responsible for each sub-score. Maps
the composite breakdown to concrete code locations:

```bash
pmat query --score-diagnosis --limit 5
# Output:
# COMPOSITE: 66.9 (Grade D)
#
# Dragging Code Quality (26.9%):
#   src/services/ast/languages/c_visitor.rs:parse_block  cc=81  grade=F
#   src/services/lightweight_provability.rs:analyze       cc=80  grade=F
#
# Dragging Muda (63.7):
#   Inventory: 129 dead items — top: src/services/cache.rs (23 dead)
#   Over-processing: cc>20 in 5 files — top: c_visitor.rs (cc=81)
#
# Dragging File Health (71.0):
#   46 files >1000 lines — top: definition.rs (1280 lines)
```

This replaces ad-hoc investigation with a single query that maps the
composite score to actionable code locations via the existing index.

### 2d. Enrichment Flag (`--score`)

Add composite score data to regular query results, similar to existing
`--coverage` and `--churn` flags:

```bash
pmat query "error handling" --score --limit 5
# Each result shows: function, TDG grade, composite sub-score impact
```

The composite score data comes from the persisted
`.pmat-metrics/commit-<sha>-meta.json`. If stale, `pmat query --score`
triggers a lightweight recomputation (comply + file health only, skip
full RPS which is slow).

## 3. CI Integration

### Single CI Gate

CI should run ONE command. Not `make lint && make test && pmat comply &&
pmat rust-project-score`. One command, one exit code, one artifact.

```yaml
# .github/workflows/quality-gate.yml
name: Quality Gate
on: [push, pull_request]
jobs:
  score:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Quality Gate
        run: pmat score --gate 70 --format json > score.json
      - uses: actions/upload-artifact@v4
        with:
          name: pmat-score
          path: score.json
        if: always()
```

### PR Status Check

`pmat score --format json` output is consumed by a GitHub Action or
CI script that posts the composite score as a PR status check:

```
✅ pmat score: 74.2 (+2.1 from base)
  RPS: 69.4 | Comply: 85 | Coverage: 94.2 | Muda: 63.7
```

### Branch Protection + Clean-Room

Configure branch protection on `pmat score` status check. Clean-room
gate: `pmat score --gate "${PMAT_GATE_THRESHOLD:-60}" --format json`.

### Pre-Push Hook

```bash
# Installed by: pmat hooks install
# Fast path: reads cached .pmat-metrics/ JSON, O(1)
pmat score --regression-check --fast
```

If no cached score exists (first push), runs full `pmat score`.

## 4. Score Regression Gates (CB-145)

Compare current composite against `.pmat-metrics/commit-*-meta.json` history.

```
Block if:
  - single_commit_delta < -5 (any sub-score dropped >5 pts)
  - window_delta < -10 (sliding 5-commit degradation >10 pts)
```

Configuration in `.pmat.yaml`:

```yaml
score:
  gate: 70
  regression:
    single_commit_max_drop: 5
    window_size: 5
    window_max_drop: 10
```

## 5. Cross-Validation Invariants (CB-146)

Run as part of `pmat score` step 8. Ten invariant rules detect contradictions
between sub-scores:

| ID | Rule | Rationale |
|----|------|-----------|
| XV-001 | CB-200 pass => RPS Code Quality >= 40% | TDG grades and code quality must agree |
| XV-002 | Muda Inventory > 20 => File Health != A | High waste contradicts good file health |
| XV-003 | Coverage >= 90% => RPS Testing >= 60% | Coverage and testing score must agree |
| XV-004 | CB-304 dead code < 2% => Muda Inventory < 30 | Low dead code means low inventory |
| XV-005 | EvoScore > 0.8 => no regression in window | Improving trend and regression contradict |
| XV-006 | DBC score >= 80% => comply errors < 5 | High DBC and many errors contradict |
| XV-007 | RPS Grade A => composite >= 75 | Category A and low composite contradict |
| XV-008 | Comply 0 errors => RPS >= 60% | Clean comply and low RPS contradict |
| XV-009 | File health A => Muda Over-processing < 15 | Good files mean low complexity waste |
| XV-010 | Coverage < 50% => composite < 80 | Low coverage must cap composite |

Warns on individual violations. Errors if 3+ fail (systemic inconsistency).

## 6. Cross-Crate Stack Compliance (CB-150)

`pmat score --stack` resolves sovereign deps from Cargo.toml, runs
lightweight scoring on each, caps project grade if deps score low.

```bash
pmat score --stack                   # Include sovereign dep scores
pmat score --stack --deps-only       # Show dep scores only
```

If any sovereign dep scores F, project caps at B.

## 7. Work-Contract-DBC Binding (CB-1250)

`pmat work complete` runs existing quality gates (`run_quality_gates()`:
changed-module tests, clippy, golden traces) then runs `pmat score --gate 60`
as an advisory composite check (CB-1250). DBC scoring (5 dimensions:
spec_depth, falsification_coverage, invariant_health, subcontracting,
traceability) feeds the DBC sub-score in the composite.

```
pmat work complete PMAT-456
  1. run_quality_gates() — tests, clippy, golden traces
  2. ALL falsifiable claims must pass
  3. DBC contract score >= threshold (default 70%)
  4. pmat score --gate 60 (advisory composite check)
  5. Evidence bundle written to .pmat-work/PMAT-456/
```

## 8. Spec-Work Linkage (CB-148)

Bidirectional traceability between `docs/specifications/components/*.md`
and `pmat work` tickets. Warns when spec planned sections have no
corresponding work items, or work items reference nonexistent specs.

```bash
pmat work list --spec repo-health    # Work items linked to spec
pmat work list --spec-gap            # Specs with no tickets
```

## 9. Leak-Driven Fault Checks

Empirical analysis of 241 fix commits across the sovereign stack (30-day
window, 2026-02-18 to 2026-03-20). Three toolable defect classes:

**CB-531 Silent Parameter Discard** (35/241, 15%): Clap `#[arg]` flags
accepted but never functionally read. Dataflow from Parser struct to handler.

**CB-534 Division Safety + DIV_RISK fault** (20/241, 8%): Division where
denominator can be zero without guard. Extends CB-528 (`.len()` only) to
all division patterns. Adds DIV_RISK to `detect_fault_patterns()`.
Note: CB-532 was previously used for "assert in library API" (removed for
high false-positive rate). This check uses CB-534 to avoid ID reuse.

**CB-533 Stale Path References** (20/241, 8%): Paths in Makefiles, CI YAML,
and string literals referencing deleted files/directories.

## 10. Provable Contracts Integration (CB-1201)

### Problem

`pv lint` runs as a standalone tool across the sovereign stack (trueno: 27
contracts, aprender: 49 contracts + .pv.toml with 55 bindings, entrenar: 2,
realizar: 2). But `pmat score` doesn't consume its results, and `pmat comply`
only checks for the existence of `contracts/` (CB-1200) without running
`pv lint`.

### Design

Integrate `pv lint` as the 8th sub-score in `pmat score` and as a comply
enforcement gate in `pmat comply`.

**`pmat score` sub-score (PV Lint)**:

Five Whys root cause: `mean_score` measures contract *completeness* (how many
optional sections filled in), not *quality*. A valid contract with basic
preconditions but no kani harness scores ~0.45. Scoring must weight gates:

**CB-1202: Contract Coverage** — checks if repos with critical ML/GPU/data
functions have contracts covering them. Keywords: forward, backward, optimizer,
checkpoint, loss, gradient, sampling, kv_cache, tokenize, quantize, kernel,
dispatch, softmax, matmul, gemm, batch. A repo with 93 `forward` functions
and 0 contracts (realizar) = FAIL. Threshold: >50% of critical keywords
must have at least 1 contract.

```
if no contracts/ AND has critical keywords:
    pv_score = 0.0 (FAIL — critical functions without contracts)
elif no contracts/:
    pv_score = 50.0 (neutral — no critical functions)
else:
    run: pv lint --format json
    existence    = 30 pts (contracts/ dir exists)
    validity     = 30 pts if validate gate passes, else 0
    cleanliness  = 25 pts if audit passes with 0 findings, 15 if findings, 0 if fails
    completeness = mean_score * 15 pts (0-15 based on maturity)
    pv_score     = existence + validity + cleanliness + completeness (0-100)
```

**Critical Five Whys finding**: `pv lint` checks YAML structure only — it never
reads source code. `audit_contract()` takes `&Contract` (YAML data), not a
project path. `FalsificationTest.test` is `Option<String>` parsed but never
resolved. `tests_file`/`tests_module` YAML fields are silently dropped by serde.
Result: trueno declares 105 test refs, 85 don't exist, `pv lint` reports 0 findings.

**pmat enforcement**: ANY missing or failing test = PV score 0.0 (kills composite
via geometric mean). CB-1201 FAILS with "Unfalsifiable" message. This is the
Popperian standard — a claim without evidence is not a claim.

**Upstream fix needed** (PMAT-521): `pv lint` Gate 4 (source verification) +
Gate 5 (test execution). Until then, pmat's `compute_contract_fulfillment`
fills the gap by scanning source for `fn test_*` matches.

**`pmat comply` check (CB-1201)**:
```
if contracts/ exists:
    run: pv lint --min-score 0.3
    PASS if pv lint exits 0
    WARN if pv lint exits 1 (findings below threshold)
    FAIL if pv lint exits 2+ (validation errors)
else:
    SKIP (no contracts directory)
```

**CI gate**: `quality-gate.yml` runs `pv lint --format sarif` and uploads
findings as GitHub code scanning results.

### Cross-Validation

New invariant: XV-011: if CB-1200 (provable contracts) passes AND
contracts/ has >10 YAML files, PV Lint score must be >= 30%.

### Stack Enforcement

`pmat score --stack` runs `pv lint` on each sovereign dep with contracts.
Failed PV Lint in a dependency flags as a warning.

### Target: `core::contracts` (Compiler-Enforced)

ONE contract type: `#[core::contracts::requires]`/`#[ensures]` (Rust nightly,
#128044). Zero cost by default, opt-in `-Z contract-checks=yes` for CI.
No external crates. YAML contracts bridge until nightly stabilizes.
See [provable-contracts.md](provable-contracts.md) for full design.

## New Comply Checks Summary

| Check | Name | Severity | Description |
|-------|------|----------|-------------|
| CB-145 | Score Regression | Error | Blocks on quality regression > threshold |
| CB-146 | Score Cross-Validation | Warning/Error | Detects sub-score contradictions |
| CB-147 | Composite Score Gate | Error | Blocks if geometric composite < threshold |
| CB-148 | Spec-Work Traceability | Warning | Specs and work items cross-reference |
| CB-150 | Stack Quality Index | Warning/Error | Cross-crate sovereign stack compliance |
| CB-531 | Silent Parameter Discard | Warning | Clap flags accepted but never read |
| CB-534 | Division Safety | Warning | Division where denominator can be zero |
| CB-533 | Stale Path References | Warning | Paths referencing deleted files |
| CB-1201 | PV Lint Gate | Warning/Error | Provable contracts lint pass rate |
| CB-1202 | Contract Coverage | Error | Critical ML/GPU keywords must have contracts |
| CB-1250 | Work-DBC Binding | Warning/Error | Evidence chain on work completion |

## Implementation Priority

| Phase | Items | Rationale |
|-------|-------|-----------|
| 1 | `pmat score` orchestrator, metric persistence | Foundation — one command, one file |
| 2 | `pmat query` integration (§2a-2d) | Score-aware search, diagnosis, ranking |
| 3 | CI workflow template, pre-push hook | Deep CI integration — single gate |
| 4 | CB-145 (regression), CB-146 (cross-validation) | Trend tracking + contradiction detection |
| 5 | CB-531, CB-534 (leak-driven) | Top defect classes by empirical volume |
| 6 | CB-150 (stack), CB-148 (spec-work), CB-1250 | Cross-repo + evidence chain |
| 7 | CB-1201 (PV Lint), 8th sub-score | Provable contracts in composite |

## Key Files

| File | Status | Purpose |
|------|--------|---------|
| `src/cli/handlers/score_handler.rs` | Exists | `pmat score` orchestrator + cross-validation |
| `src/cli/handlers/query_handler/` | Exists (extend) | Add `--score`, `--score-diagnosis`, `--rank-by score` |
| `src/cli/handlers/comply_handlers/check_handlers/check.rs` | Exists | Comply check orchestrator |
| `src/services/rust_project_score/orchestrator.rs` | Exists | RPS 11-category scorer |
| `src/cli/handlers/work_contract_scoring.rs` | Exists | DBC 5-dim per-contract scoring |
| `.pmat-metrics/commit-<sha>-meta.json` | Exists | Composite score persistence |
| `.github/workflows/quality-gate.yml` | Exists | CI template (single gate) |

## References

- RPS v3.0: [repo-health.md](repo-health.md)
- Quality Gates: [quality-gates.md](quality-gates.md)
- Work Management: [work-management.md](work-management.md)
- SWE-CI EvoScore: [swe-ci-evolution.md](swe-ci-evolution.md)
- Code Quality (DBC): [code-quality.md](code-quality.md)
