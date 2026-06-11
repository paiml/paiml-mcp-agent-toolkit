# `pmat verify` — Autonomous Pre-Flight Verification

**Status**: Implemented (v3.18.0)
**Audience**: autonomous coding agents (e.g. Fable 5 in autonomous mode) and humans

## Motivation

In autonomous mode there is no human in the loop to catch a `clippy`/test failure
before a push. An agent therefore either (a) burns full CI cycles (~11 min) on
trivial failures, or (b) burns ~10 min/iteration running the toolchain locally.
The **verify loop — not the model — is the throughput bottleneck.**

The problem is a fidelity gap: the three "is my code OK?" checks that exist do
**not** match what actually blocks a merge.

| Layer | Runs | Misses |
|-------|------|--------|
| pre-commit hook | fmt, complexity, satd | **clippy, tests** |
| `pmat quality-gate` | dead-code, complexity, coverage, satd, provability, entropy | **clippy, tests** |
| **CI (the real gate)** | **clippy −D warnings, tests**, coverage, score | — |

So an agent can pass *both* local gates and still fail CI. (This was observed
directly: a PR passed pre-commit, then failed `ci/lint` on
`clippy::nonminimal_bool` 11 minutes later.)

`pmat verify` closes that gap: **green here ⇒ green in CI.**

## Design

A single command that runs the CI-faithful gate set, **fail-fast** (cheapest
stage first), with machine-readable output, scoped to the diff where possible.

### Stages (in order; stop at first failure unless `--no-fail-fast`)

1. **format** — `cargo fmt --all -- --check` (sub-second once built)
2. **complexity** — pmat's in-process analyzer, gate: cyclomatic ≤ 30, cognitive ≤ 25 (changed files only with `--changed`)
3. **satd** — pmat's in-process SATD detector, strict mode
4. **clippy** — `cargo clippy --lib --bins -- -D warnings` (CI-faithful — the Makefile `lint` target; **not** `--all-features`, which builds optional batuta-stack feature combos CI never compiles)
5. **tests** — `cargo test --lib` (or, with `--changed`, only the test modules reachable from changed files via the call graph)

Stages 1–3 are fast and catch the majority of issues, so an agent gets a red in
seconds for the common cases and only pays the clippy/test cost when the cheap
stages are clean.

### CLI

```
pmat verify [OPTIONS]
  --fix                 Auto-apply fixable issues (cargo fmt; cargo clippy --fix)
  --format <text|json>  Output format (default: text). json is for agents.
  --no-fail-fast        Run all stages even after a failure (full report)
  --skip <STAGES>       Comma-separated stages to skip (e.g. tests for a doc-only change)
  --stage <STAGE>       Run only one stage
```

The **complexity** stage is always scoped to files changed vs `HEAD` (matching
the incremental pre-commit gate); a whole-project scan would flag pre-existing
high-complexity *test* files that CI never gates. clippy and tests are
whole-crate (a single crate cannot scope clippy below the crate).

Exit code: `0` iff every (non-skipped) stage passed. Non-zero otherwise — the
agent's signal to fix before committing.

### Machine-readable output (`--format json`)

```json
{
  "ok": false,
  "duration_ms": 51234,
  "stages": [
    {"name": "format",     "ok": true,  "duration_ms": 320},
    {"name": "complexity", "ok": true,  "duration_ms": 410},
    {"name": "satd",       "ok": true,  "duration_ms": 290},
    {"name": "clippy",     "ok": false, "duration_ms": 49000,
     "violations": [
       {"file": "src/x.rs", "line": 230, "rule": "clippy::nonminimal_bool",
        "message": "...", "fixable": true}
     ]},
    {"name": "tests",      "ok": null, "skipped": "fail-fast"}
  ]
}
```

Agents read `ok` per stage and the `violations[]` (with `file:line:rule`) to
self-correct without parsing human text.

## Why pmat is the right home

The reason a naive "run clippy + all tests" is slow is that it is unscoped. pmat
already owns the primitives to scope it: the call/dependency graph
(trueno/aprender-graph CSR, O(1) context) maps changed files → affected tests;
incremental change detection (git churn, the `.pmat` index) bounds the work; and
the complexity/satd analyzers already run in-process for the pre-commit hook.
`verify` is the orchestration layer over primitives pmat already has.

## Autonomous-mode contract

The canonical agent loop becomes: **edit → `pmat verify --changed --format json` →
(self-fix on red) → repeat → commit only on green.** This is the pmat-paradigm
primitive for autonomous operation: one command, CI-faithful, machine-readable,
fail-fast. It is documented in `docs/agent-instructions/` as the required
pre-commit step for agents.

## Dogfooding

CI fidelity is verified by keeping the stage set in sync with `.github/workflows`
(the `ci/gate` job). `make verify` wraps `pmat verify` for humans; the
pre-commit hook may call `pmat verify --stage format --stage complexity` for the
fast subset it already runs.
