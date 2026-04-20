# Improve Coverage 80 → 95% (World-Class Quality Goal)

> **Status**: Draft — v3.15.0 post-ship initiative
> **Related**: [Quality & Testing](components/quality-testing.md), [Provable Contracts](components/provable-contracts.md)
> **Owners**: core maintainers
> **Dogfood**: `pmat query --coverage-gaps --rank-by impact` is the canonical targeting tool

---

## 1. Problem Statement

PMAT has *two* coverage numbers, and the gap between them is the bug:

| Scope | Lines measured | Covered | % |
|-------|----------------|---------|---|
| **Narrow slice** (current `make coverage`) | 20,394 | 19,584 | **96.03%** ✅ |
| **Honest broad** (all `src/**/*.rs`, no exclusions, no `coverage(off)`) | 324,456 | 244,377 | **73.14%** ❌ |

That delta is **selection bias**: `COVERAGE_EXCLUDE` drops 83 files by regex, `#[coverage(off)]` drops **8,097 functions across 938 files** (2,832 attr occurrences across 1,967 files), and `cli_integration_tests` are `--skip`-ed in the runner. The narrow number is real for the modules it measures, but the *project* is 73%.

World-class open-source Rust hovers at **90–95% honestly measured** (ripgrep, cargo, tokio, sqlx). Our sovereign stack sits in the same band (§2). The goal is to converge PMAT on **95% honest, broad coverage** — same scope rules as the rest of the stack — without collapsing the contract-first design work.

---

## 2. Sovereign Stack Survey (Patterns to Replicate)

One paragraph per repo; measurements as of 2026-04-19.

**aprender** — `COV_THRESHOLD := 95` in the Makefile with LCOV-parsing awk gate. Tool: `cargo-llvm-cov`. Explicit regex excludes `trueno/`, `realizar/`, `fuzz/`, `golden_traces/`, `hf_hub/`, `models/`, `serialization/`, `voice/`, `speech/`, `transfer/`. Heavy `#[coverage(off)]` at file and item level. **Dual-exclusion pattern** (regex + attr). Documents CB-127-A: **never nextest for coverage** (profraw explosion). Mold-linker workaround temporarily moves `~/.cargo/config.toml`. PROPTEST_CASES reduced during coverage runs.

**trueno** — Target 90%, last measured **91.78%** (`COVERAGE_POLICY.md`). Regex excludes `aprender/`, `crates/`, `trueno-explain/`, `trueno-graph/`, `xtask/`, `backends/gpu/`, `wasm.rs`. Documents the **GPU/WGSL-can't-be-instrumented policy** in `COVERAGE_POLICY.md` — defuses "hiding uncovered code" critiques. Target-dir → `/mnt/nvme-raid0/coverage/trueno` for RAID speed. Mutation-score floor ≥80% via cargo-mutants.

**trueno-graph** — Target ≥95%, enforces via `pmat quality-gate --checks clippy,fmt,tests,coverage --coverage-threshold 95`. Dogfoods PMAT's own gate instead of awk parsing.

**trueno-db** — Target ≥90%, GPU excluded by policy. Split targets: `coverage` (report-only) and `coverage-check` (blocking). `quality-gate: lint test coverage-check` composes them. mutants.toml + golden_traces active.

**renacer** — No coverage threshold enforced (reporting-only), 10-min budget. Instead, dogfoods its own chaos/proptest/golden-tracing. Builds fixtures before coverage.

**certeza** — 85% minimum / 95% target, **260 tests, 97.7% mutation score**. Tier 1/2/3 gate ladder. `kaizen/improvement.log` writes coverage snapshot over time. Only repo in the stack promoting **mutation score as a co-equal metric** to coverage.

**probar** — 95% target, `cargo-llvm-cov --lib --workspace`, <1 min. Coverage is a **product feature**: ships `gui_coverage!` macro, `probador coverage --html`, budget/pixel-diff thresholds as user-facing APIs. PROPTEST_CASES=25.

**bashrs** — **95.04% line coverage** (README). Three tiers: `coverage-quick` (~3 min, 85%), `coverage` (~3.5 min, 94%), `coverage-full` (~5 min, 95%). `cargo +nightly fuzz coverage` for ast_parser + differential_optimization. Lone surviving tarpaulin call (`verify-coverage` legacy) — PMAT should not follow this.

**presentar** — 95% badge, Tier 1/2/3 with Tier 3 running mutation + coverage together. Ships `coverage.json` artifact + DVC/MLflow hints.

### 2.1 Patterns worth copying into PMAT

1. **Codify threshold as a Makefile variable** (`COV_THRESHOLD := 95`) + LCOV-parsing awk gate. Single source of truth.
2. **Two-phase coverage** — `cargo llvm-cov test --no-report` keeps profraw, `report --lcov` is a separate pass. Halves wall-clock on re-reports.
3. **Three tiers** — `coverage-quick` (~1 min, 85%, lib-only inner loop), `coverage` (~3 min, 94%, default), `coverage-full` (~5 min, 95%, CI gate).
4. **Delegate threshold to `pmat quality-gate --coverage-threshold`** — trueno-graph already does this. Dogfoods PMAT. Retires awk/shell logic across the stack.
5. **Publish mutation score alongside line coverage** (certeza 97.7%, trueno ≥80%). Target: ≥80% mutation, ≥95% line.
6. **Document exclusion policy in a checked-in file** — trueno's `COVERAGE_POLICY.md` pattern. Justifies every regex entry with "why LLVM can't/shouldn't instrument this."
7. **Set `PROPTEST_CASES=3–25` during coverage runs** — proptest explosion kills wall-clock. bashrs uses 3, aprender/probar use 25.
8. **Never nextest for coverage** — document CB-127-A in PMAT's coverage target.
9. **mold-linker temporary displacement** — wrap `.cargo/config.toml` move in a `coverage-prep` / `coverage-cleanup` target pair.
10. **Retire the one legacy tarpaulin reference** in the stack (bashrs `verify-coverage`) as part of this work — the sovereign stack policy is cargo-llvm-cov only.

---

## 3. The Tech Debt Creating the Coverage Gap

`pmat query --coverage-gaps --rank-by impact --limit 15 --exclude-tests` (2026-04-19) ranks top offenders. The pattern that dominates: **dispatch boilerplate and scaffold templates with zero tests.**

### 3.1 Observed coverage drags

| Category | Broad cov | Mitigation |
|----------|-----------|------------|
| `src/cli/` (handlers, dispatch) | 65% (91k lines) | Consolidate dispatch via macros; push logic into services/; golden-trace E2E via renacer |
| `src/handlers/tools/` | 36% (3.2k lines) | Parity tests for every tool (D101/D102/D103 already proved this works) |
| `src/scaffold/` (template generators) | ~0% | Snapshot tests: `insta` on generated output |
| `src/roadmap/` | 32% | Delete dead; contract-gate live surface |
| `src/services/` | 80% (75k lines) | TDD on hot paths surfaced by `--rank-by impact` |
| `src/tdg/` | 78% (13k lines) | Co-equal to services/; same approach |

### 3.2 Tech debt categories to retire

1. **`coverage(off)` selection bias** — 2,832 attrs across 1,967 files; 8,097 excluded functions. Remove module-level `#![coverage(off)]` wherever the module is reachable in normal operation; keep it only for:
   - unreachable polyfills (test-only feature gates),
   - LLVM-can't-instrument code (WASM shim, GPU shaders if any),
   - vendored third-party source.
   Document what remains in a `COVERAGE_POLICY.md` (trueno pattern).
2. **Dead-code tails** — `project_analysis.rs` (375 lines) was entirely dead and deleted; memory records `context_output.rs` had 857 dead lines. Run `pmat query "unused" --faults --dead-code` and delete, don't test.
3. **Dispatch boilerplate in `cli/`** — 91k lines at 65% is almost entirely `match arg { Sub::X(args) => handle_x(args).await, … }`. Consolidate via the `try_enrich!`-style macro pattern already used elsewhere. Fewer lines = higher % without writing new tests.
4. **`cli_integration_tests` skipped in `make coverage`** — `--skip cli_integration_tests` is on the current cargo-llvm-cov invocation. These are the highest-ROI tests to *un-skip*; they exercise the cli/ dispatch surface that drags the broad number.
5. **Scaffold templates with zero tests** — `src/scaffold/**` generates strings; add `insta` snapshot tests. One test per generator, near-100% coverage per file, cheap.
6. **Minimal-build cfg gates leaking into coverage** — PR #380 in flight. After merge, all `#[cfg_attr(coverage_nightly, coverage(off))]` sites become mechanical candidates for removal under §5.

---

## 4. Parallel Track: Improve Provable Contracts *While* Covering

The user's insight: "improve provable contracts at same time, which should flush out the issues." This is correct. Contracts force test coverage on the same functions. Concretely:

### 4.1 Contract-first coverage

For every function added to `.pmat/binding.yaml` (per `components/provable-contracts.md`):

1. **Pre-condition / post-condition tests are auto-required** — the L3 Kani / L4 Lean harness already runs as part of the contract's acceptance. The L2 debug-assert path runs under `cargo test`. Adding a contract *creates* coverage on its function by construction.
2. **Contract coverage gate (CB-1400) already wired** — file can't reach Grade A+ without contract. This spec extends it: **a file at <95% line coverage with `status: implemented` contracts is a P1 bug.** Surface in `pmat tdg --with-coverage-gaps`.
3. **Prioritize binding coverage on the uncovered cli/ and handlers/ surface** — these are the coverage drags (§3.1) *and* the places where hallucinating MCP tools live (R21 D90/D92/D100 memory). Contracts fix both.

### 4.2 Provable-contract improvements this spec drives

| Improvement | Coverage impact | Contract impact |
|-------------|-----------------|-----------------|
| Ship `pmat tdg --with-coverage-gaps` that flags A-graded files whose covered-% < threshold | Makes selection bias *observable* per-file | Forces contract coverage to track real test coverage |
| Extend `pmat query --coverage-gaps` with `--contract-gap` flag (covered but unbound, or bound but uncovered) | Dogfoods the gap | Surfaces contract-adoption gaps |
| Wire `cargo-mutants` into `make coverage-full` — certeza pattern | 97.7% mutation target | Mutants that survive == weak contract + weak test pair |
| `probar` property tests on every `binding.yaml` L3+ contract | Exercises boundary cases | Matches contract expression language coverage |
| Retire tarpaulin references across stack | N/A | Stack hygiene |

### 4.3 Falsification feedback loop

The user-requested loop (rephrased): *contracts flush out coverage issues*. Mechanism:

1. Add a contract → L3 harness fails → test or contract is wrong.
2. Add a contract → L3 passes but `cargo-mutants` survives → test is too weak.
3. Add a contract → covered, passes, survives mutation → **real coverage on this function.**
4. A file that "had 100% line coverage" without surviving mutation was lying; contract-driven mutation forces the issue.

This ties directly into CB-1400 (contract gate) and KAIZEN-0190 (SCHEMA-003 etc.) — the same machinery.

---

## 5. Phased Milestones

Each phase is independently shippable. Do not chase the next phase until the current one holds for 2 weeks on `main`.

### Phase 0 — Honesty baseline (1 day, v3.15.1)

- [ ] Add `make coverage-broad` target: no `COVERAGE_EXCLUDE`, no `--skip`, no `coverage(off)` via `--no-cfg-coverage --no-cfg-coverage-nightly`.
- [ ] Report both numbers in CI: narrow (gate) and broad (informational).
- [ ] Commit `COVERAGE_POLICY.md` cataloguing every current exclusion with justification.
- [ ] Document PROPTEST_CASES=3 for coverage runs (bashrs pattern).

### Phase 1 — 73% → 80% (1 week, v3.16.0)

**Strategy: delete, don't test.**

- [ ] Delete confirmed dead code surfaced by `pmat query --faults --dead-code` + manual verification (project memory: include!() files need all-includer inspection).
- [ ] Un-skip `cli_integration_tests` in `make coverage`. These are the highest-ROI tests.
- [ ] Snapshot-test every `src/scaffold/**` generator via `insta`. Target: 0% → 90% in that tree.
- [ ] Collapse 10+ CLI dispatch `match` arms into the existing macro pattern. Lower denominator.

Exit criteria: `make coverage-broad` reports ≥80% and `make coverage` (gate) stays ≥95%.

### Phase 2 — 80% → 90% (3 weeks, v3.17.0)

**Strategy: TDD on hot paths ranked by impact.**

- [ ] Loop: `pmat query --coverage-gaps --rank-by impact --limit 20 --exclude-tests` → pick top-3 by `impact_score = missed_lines * pagerank / complexity` → write tests → re-run → commit → repeat.
- [ ] Target directories, in order: `handlers/` (36% → 85%), `roadmap/` (32% → 85%), `cli/` (65% → 85%).
- [ ] For every PR, require the `cli_integration_tests` path it touches to grow.
- [ ] Introduce `pmat quality-gate --checks coverage --coverage-threshold 90` as the broad-coverage gate (replaces awk).

Exit criteria: broad ≥90%, narrow ≥96%.

### Phase 3 — 90% → 95% + mutation ≥80% (6 weeks, v3.18.0)

**Strategy: contract-driven + mutation-forced.**

- [ ] Wire `cargo-mutants` into `make coverage-full`. Floor: ≥80% mutation score.
- [ ] Remove `#[coverage(off)]` from every module that isn't justified in `COVERAGE_POLICY.md`. Audit the remaining 2,832 attrs.
- [ ] Extend `binding.yaml` coverage for every file at Grade A- that should be A+ (CB-1400).
- [ ] Ship `pmat tdg --with-coverage-gaps` and `pmat query --coverage-gaps --contract-gap`.
- [ ] Migrate threshold enforcement to `pmat quality-gate --coverage-threshold 95` — dogfood.

Exit criteria: broad ≥95%, narrow ≥97%, mutation ≥80%, every A-graded file has contract coverage.

### Phase 4 — Stack hygiene (ongoing)

- [ ] Retire tarpaulin reference in bashrs `verify-coverage` (stack-wide policy).
- [ ] Publish `COVERAGE_POLICY.md` as a template in `pmat scaffold` (new sovereign-repo bootstrap).
- [ ] Add `pmat coverage` wrapper that handles mold displacement + nightly + cfg flags automatically — one command for the whole stack.

---

## 6. Acceptance Criteria

This spec is satisfied when all are true:

1. `make coverage-broad` ≥ 95% honestly measured (no `--skip`, no regex, no `coverage(off)` except items in `COVERAGE_POLICY.md`).
2. `make coverage-full` ≥ 80% mutation score via cargo-mutants.
3. Every Grade-A+ file has contract coverage in `binding.yaml` (CB-1400 hard-gated).
4. `pmat quality-gate --checks coverage --coverage-threshold 95` is the gate in CI — no awk.
5. `COVERAGE_POLICY.md` exists and justifies every remaining exclusion.
6. `pmat query --coverage-gaps --contract-gap` is live and dogfooded in pre-commit.
7. The narrow-vs-broad delta is ≤ 2% (honest baseline == reported baseline).

---

## 7. Non-Goals

- **Not chasing 100%.** 95% is world-class; 100% wastes engineering time on trivial branches.
- **Not removing all `coverage(off)`.** LLVM-can't-instrument code (WASM shim, any future GPU shaders) stays out. The test is: "is there a justification in `COVERAGE_POLICY.md`?"
- **Not replacing contract-first design.** CB-1400 still stands; this spec *strengthens* it by forcing coverage on contracted functions.
- **Not feature-gating coverage.** `make coverage` works on a clean nightly checkout, no feature flags required.

---

## 8. References

- [Quality & Testing (CB-1400 provable-contract gate)](components/quality-testing.md)
- [Provable Contracts](components/provable-contracts.md)
- [trueno COVERAGE_POLICY.md](/home/noah/src/trueno/COVERAGE_POLICY.md)
- [aprender Makefile COV_THRESHOLD](/home/noah/src/aprender/Makefile)
- [certeza Tier 1/2/3 gate ladder](/home/noah/src/certeza/Makefile)
- [bashrs three-tier coverage](/home/noah/src/bashrs/Makefile)
- rust-lang/rust#84605 — `coverage_attribute` stabilization (still open on nightly 2026-04-18; PR #380 tracks the toolchain workaround)
