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

## 2.2 Five Whys: Root Cause of the 73% vs 96% Delta

Applied 2026-04-21. The narrow/broad gap is not an accident — it is a direct, predictable consequence of the gate's definition.

1. **Why is broad coverage 73% while the gate reports 96%?**
   Because the gate measures `covered / measured`, not `covered / total-Rust-LOC`. `COVERAGE_EXCLUDE` (17 regex patterns) drops 83 files from the denominator; `--skip cli_integration_tests` drops the runner; `#[coverage(off)]` drops 8,097 functions across 938 files. The gate arithmetic then inflates the remaining ratio.

2. **Why was the measured set narrowed?**
   Commit `e698faa55` moved the project from 64% to 95% in one stroke by adding `/cli/`, `/handlers/`, `/services/`, `/tdg/`, `/roadmap/`, `/scaffold/`, `/workflow/` to `COVERAGE_EXCLUDE`. Commit `7d1eda3c0` ("Extend exclusions to maintain 95%+") widened them further. The gate rewarded exclusion over testing.

3. **Why did the team default to exclusion?**
   Because writing `#[cfg_attr(coverage_nightly, coverage(off))]` on a module takes 30 seconds; writing tests for 91k lines of dispatch boilerplate takes weeks. With only one coverage number on the CI dashboard, exclusion is the locally rational choice. The cargo-cult spread: 2,832 attrs across 1,967 files, most on code that has nothing to do with LLVM instrumentation limits.

4. **Why is there only one coverage number?**
   Because `make coverage-broad` didn't exist until Phase 0 (PR #390, 2026-04-21). Before then, the narrow number was the only number, so selection bias was invisible. You cannot optimize what you do not measure.

5. **Why does this matter for v3.15.0 and beyond?**
   Because the 95% badge advertises a property the project does not have. Downstream consumers (certeza, aprender, trueno-graph) honestly measure at 91–97% *broad*; PMAT measures at 73% broad / 96% narrow. The stack's sovereignty claim depends on coverage being honest. Fix the denominator and the problem becomes addressable; leave it and the badge keeps lying.

### 2.3 Three-Bucket Classification of Current Exclusions

Every existing exclusion falls into one of three buckets. Only bucket A survives Phase 3.

**Bucket A — Legitimate (stays, documented in `COVERAGE_POLICY.md`):**
- WASM runtime shim — LLVM cannot instrument wasm32 targets when built for native test binaries.
- Generated code (`explain.rs` template output, protobuf stubs if added).
- Vendored third-party source (none currently in `src/`).
- `test_performance_suite.rs` — benchmark harness, not production.

**Bucket B — Masking (selection-bias, must be re-included one directory at a time):**
- `/cli/` (91k LOC, 65% cov) — dispatch boilerplate, exercised by `cli_integration_tests` which are `--skip`ped. *Un-skipping alone recovers several points.*
- `/handlers/` (3.2k LOC, 36% cov) — the MCP surface that hallucinated in R21 D90/D92/D100. Already has parity-test machinery (D101/D102/D103); extend, don't exclude.
- `/services/` (75k LOC, 80% cov) — already close to threshold; exclusion is purely to keep the ratio clean.
- `/tdg/` (13k LOC, 78% cov) — same pattern.
- `/roadmap/`, `/workflow/`, `/red_team/`, `/contracts/`, `/qdd/`, `/unified_quality/`, `/state/`, `/protocol/`, `/docs_enforcement/` — all production code, all in the excluded set.
- `/scaffold/` (~0% cov) — template generators; the fix is `insta` snapshot tests, not exclusion.
- `/mcp[^/]*/` — MCP server/client code is integration-tested but the numbers don't count. Gets its own co-equal gate in Phase 3.

**Bucket C — Cargo-culted `#[cfg_attr(coverage_nightly, coverage(off))]` (2,832 attrs, 1,967 files):**
The attribute was pasted as part of a template on every module file, regardless of whether the code is reachable under normal test runs. Verified by sampling: the attr appears on modules that have live tests exercising them. The attribute only takes effect on nightly + `coverage_nightly` cfg; on the default `make coverage` run (stable-ish nightly without that cfg), llvm-cov ignores it — which is why broad coverage is still *measurable* at 73%. This bucket is mechanical deletion (`sed` over all 1,967 files in one PR), zero behavioral risk.

### 2.4 Ordered Remediation (replaces prior Phase-1 bullets)

1. **P0 (this spec):** Make `coverage-broad` the CI gate; demote `make coverage` to informational. Forces the denominator to be honest. Gate threshold moves to 75% for v3.15.1, ratcheting +5pp per minor until 95%.
2. **P1:** Add `make coverage-cli-integration` target. `cli_integration_tests.rs` is all `#[ignore]`-ed (spawns the `pmat` binary as a subprocess), so the naive "delete `--skip`" is a no-op — the real fix is `cargo build --bin pmat` + `--ignored` + running the coverage instrumentation across binary+lib. Separate target because the wall-clock budget differs.
3. **P2:** Delete cargo-culted `#[cfg_attr(coverage_nightly, coverage(off))]` attrs in one mechanical PR. File-level only; keep the legitimate item-level uses in Bucket A. Since `make coverage-broad` already passes `--no-cfg-coverage-nightly`, deletion is behaviorally a no-op *now* — the work is preparing for PR #380's toolchain flip, which will otherwise re-mask 8,097 functions.
4. **P3:** Treat `/scaffold/` at ~0% as a test-debt bug (not an exclusion target). `insta` snapshots per generator.
5. **P4:** Give `/mcp[^/]*/` its own co-equal coverage number (`make coverage-mcp`) in CI output, so the number is visible even when it isn't the gate.

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

### 4.4 Empirical evidence from the autonomous loop (2026-04-24..25)

The Phase-1 autonomous loop demonstrated the §4.3 prediction in practice. While covering the comply checks and shared compliance helpers, we landed direct improvements to the provable-contracts surface:

**PV check coverage landed:**

| PR | File | Covers | What broke / found |
|----|------|--------|--------------------|
| #527 | `check_pv_quality.rs` | parse_metric / parse_float_metric helpers + 4 skip arms | — |
| #528 | `check_pv_quality_gate.rs` (CB-1202/CB-1208/CB-1209) | no-src/ + no-contracts/ skip arms | — |
| #528 | `check_pv_verification_ladder.rs` (CB-1204/CB-1205/CB-1207) | all 4 "no contracts/ → Skip" arms + Pass arms | — |
| #527 | `migrate_handlers_init::generate_default_pmat_yaml` | round-trip through `PmatYamlConfig::load_from_path` | **DEFECT FIXED**: scaffolded yaml had `severity: high`, but `PmatYamlConfig` only accepts `info/warning/error/critical` — fresh `pmat comply init` produced an unloadable `.pmat.yaml`. Caught by the round-trip test; fixed to `severity: error`. |

**Concrete realisation of §4.3.1:**

The default-yaml defect is exactly the failure mode §4.3 predicts: a function existed (with line coverage from other tests), but **no test asserted the post-condition that what it wrote could be loaded by its sibling parser**. Adding the round-trip test (which is what a `binding.yaml` post-condition would have generated automatically) immediately exposed it. The fix was 1 line; finding it required co-evolving coverage and contracts.

**Update to the §4.2 priority table:**

The "extend `pmat query --coverage-gaps` with `--contract-gap` flag" row is now scoped: until that flag exists, the autonomous loop should treat **every PR that touches a `pub` fn missing a `binding.yaml` entry** as an opportunity to add one. PRs #527/#528/#529 covered PV-check entry points — these should appear in `binding.yaml` alongside the new tests, so the contract status follows the test status.

**Loop convergence model (revised):**

| Phase | Mechanism | Per-PR coverage gain | Per-PR contract gain |
|-------|-----------|----------------------|----------------------|
| Wave 1 (PRs #521..#522) | Bundle + check.rs split | +0.31pp | 0 (refactor only) |
| Wave 2 (PRs #523..#549) | One file per PR, pure-compute leaves | +0.05–0.20pp each | 1–2 functions per PR ready for `binding.yaml` |
| Wave 3 (next) | Async handlers + RefactorContext fixtures | +0.15–0.30pp est | Contract surface area expands to handlers |

Each test added to a 0%-cov pure-compute helper *also* establishes the "this function's behaviour is observable from outside" property that a `binding.yaml` precondition needs. The two efforts compound rather than compete.

### 4.5 Five Whys: 95% convergence ceiling (2026-04-25)

After Wave 30 (21 PRs, ~280 tests, +4.03pp from 73.42% → 77.45%), measured rate is **~70 tests per percentage point** of broad coverage. At observed cadence (≈14 tests/PR), reaching 95% requires ~90 more PRs (~4–5 sessions). Five Whys identifies why drip-feed alone won't converge:

**Why 1 — per-PR delta is 0.05–0.20pp:** Denominator math. Broad measures 329,231 lines (1pp ≡ 3,292 lines). A typical pure-helper test fires 5–20 lines, so 1pp ≈ 70–250 tests. Observed exactly.

**Why 2 — denominator is so large:** `make coverage-broad` strips `coverage_nightly` from 707+ files (1,257 functions) and unsupresses 2,987 functions in 769 files matched by Makefile `COVERAGE_EXCLUDE` regex. These were narrow-gate-invisible by design.

**Why 3 — so many files are excluded:** Phase 0 dual-gate model: narrow gate measures testable code, broad gate is honest. Exclusions cover CLI handlers (subprocess), MCP infra (network I/O), printers (stdout side-effects). Each individually defensible; collectively they're 42% of non-test source.

**Why 4 — wave 30 yields diminishing returns:** Three structural ceilings bind simultaneously:
- **Easy targets exhausted.** Of 21 wave-30 PRs, 16 hit 0-prior-test files. Remaining 0%-cov files are subprocess-bound (`mutate_output.rs` skipped — uses cargo subprocess), filesystem-bound (`spec_handlers_sync.rs` had 9 fs-helpers I had to skip), or async-with-fixture (`oracle_handlers` async fns).
- **Stale-test gates.** `qa_work_handler/tests.rs`, `mutate_tests.rs`, `spec_handlers/tests.rs` are gated behind `feature = "broken-tests"` because file-splitting "broke syntax". ~50–100 tests are dark in the default build.
- **CI starvation.** PR #552 restarted CI on every push (21×). The `pmat score` Quality Gate is the only pending check; CI takes ~10 min. Sequential merge cadence is the long pole, not test authoring.

**Why 5 — trajectory predicted but not redirected:** Spec §4.3 + §4.4 + memory `feedback_autonomous_scaffold_loop` all predicted diminishing returns. User's standing instruction prioritizes "measure often, don't ask for approval." Loop is doing what was specified — but the spec did NOT include exit conditions like "stop drip-feed when rate < 0.10pp/PR and pivot to bundle/refactor." Those escalation actions need explicit authorization.

**Root cause:** The drip-feed cadence is correct for what was authorized, but 95% is not reachable at this rate within reasonable session count. Three independent ceilings (target exhaustion, stale-test gates, CI throughput) all bind at the same time.

#### Corrective actions (recommendations)

Ordered by ROI; each is a discrete escalation from drip-feed to higher-leverage work.

**R1 — Revive `feature = "broken-tests"`-gated tests (highest single-PR ROI).**
- 3 modules have ~50–100 tests gated and dark: `qa_work_handler/tests.rs` (uses `handle_generate_checklist` + `print_task_status` + epic summary fixtures), `mutate_tests.rs` (uses stale `MutationOperator::ArithmeticReplace` enum names + `MutationResult.test_output` field — needs renames to current `MutationOperatorType::ArithmeticReplacement` / current struct shape), `spec_handlers/tests.rs` (broken by include!() file split — likely just needs path fixes).
- Per module: read the gated file, fix the type/field mismatch, rename to current identifiers, drop the `feature = "broken-tests"` gate, run tests, push.
- Expected gain: +0.5–1.0pp on broad per module reanimated, atomic per-PR.
- Owner: autonomous loop OR explicit user instruction.

**R2 — Bundle PR for wave 30 (CI throughput unlock).**
- Wave 30's 21 commits are independently good but each push restarts ~10-min CI. Bundle pattern from PR #520 (wave 22, 22 sub-PRs squashed): single CI run, single squash merge.
- Mechanics: branch from current master, replay each wave-30 commit via `git cherry-pick`, `git reset --soft origin/master && git commit` to flatten into one squash-ready commit, push and merge.
- Expected gain: 0pp coverage (cumulative work already on branch) but **unblocks the next wave** by freeing CI runners and removing the BLOCKED merge state.
- Owner: explicit user authorization (rebase/squash is destructive on the working branch).

**R3 — Audit `coverage(off)` decisions (denominator reduction).**
- 707 files have `#![cfg_attr(coverage_nightly, coverage(off))]`. Many are pure-compute (e.g., `src/services/satd_detector/types.rs` — types-only file with 100% pure-compute, opted out anyway). Sampling suggests ~10% over-suppression.
- Per file: read top of file, check whether the contents are pure-compute (no `Command::new`, no `tokio::fs`, no terminal I/O) — if pure-compute, lift the attribute. Run `make coverage` to verify narrow gate still passes (these files were excluded from narrow originally; lifting is safe IF they have tests).
- Expected gain: 0.5–2.0pp on broad cumulative (mechanical pass, no new tests required).
- Owner: explicit user authorization (changes coverage policy at file-level).

**R4 — Set 85% intermediate target before pushing to 95%.**
- Spec §5 Phase 2 milestone is 80% → 90%. Reaching 85% from 77.45% is 7.5pp away (~525 tests at observed rate, ~37 PRs).
- Ship the 80% milestone (Phase 1 exit) first, declare a v3.16.0 release boundary, then reassess whether 95% is still the right Phase-2 exit or whether 90% is more honest given stack-wide constraints.
- Expected impact: re-prioritization, not coverage gain. Lets the team ship a real milestone instead of grinding indefinitely.
- Owner: spec author / user.

**R5 — Refactor subprocess-spawning helpers (long-term denominator unlock).**
- `qa_work_handler/impl_validation.rs` runs `cargo test`/`cargo clippy`/`pmat analyze complexity` via `Command::new`. The decision logic (parse output → ValidationStatus arm) is pure-compute trapped behind subprocess invocation.
- `git_history_parsing.rs` parses `git log` output — same pattern. Pure parser inside, subprocess outside.
- Refactor: extract pure parser/decision functions taking `&str` (the captured stdout). Subprocess-spawning wrapper becomes thin and is itself coverage(off) by ergonomics.
- Expected gain: 1–3pp on broad per refactored module, plus contract surface area unlocks.
- Owner: explicit user authorization (architectural change, not autonomous-loop scope).

**Default if no escalation authorized:** continue R5-style autonomous loop on remaining 0%-cov pure-compute slice. Convergence to 95% will take ~4–5 sessions at observed rate.

---

## 5. Phased Milestones

Each phase is independently shippable. Do not chase the next phase until the current one holds for 2 weeks on `main`.

### Phase 0 — Honesty baseline (1 day, v3.15.1)

- [ ] Add `make coverage-broad` target: no `COVERAGE_EXCLUDE`, no `--skip`, no `coverage(off)` via `--no-cfg-coverage --no-cfg-coverage-nightly`.
- [ ] Report both numbers in CI: narrow (gate) and broad (informational).
- [ ] Commit `COVERAGE_POLICY.md` cataloguing every current exclusion with justification.
- [ ] Document PROPTEST_CASES=3 for coverage runs (bashrs pattern).

### Phase 1 — 73% → 80% (1 week, v3.16.0)

**Strategy: delete, don't test.** (Original plan; superseded by §4.5 R1–R5 corrective actions after wave-30 evidence showed drip-feed ceiling.)

- [ ] Delete confirmed dead code surfaced by `pmat query --faults --dead-code` + manual verification (project memory: include!() files need all-includer inspection). *(memory `project_coverage_95_baseline.md` reports `pmat analyze dead-code` returns 0 dead lines in `src/`; this bullet is exhausted.)*
- [ ] Un-skip `cli_integration_tests` in `make coverage`. These are the highest-ROI tests. *(memory reports test bodies are TODO stubs; un-skipping adds zero coverage; this bullet is exhausted.)*
- [ ] Snapshot-test every `src/scaffold/**` generator via `insta`. Target: 0% → 90% in that tree. *(Done via PRs #498–#506, PMAT-633..641; sweep complete.)*
- [ ] Collapse 10+ CLI dispatch `match` arms into the existing macro pattern. Lower denominator. *(memory reports main dispatcher has only 10 arms; not a big-denominator collapse; this bullet is exhausted.)*
- [ ] **R1 — Revive `feature = "broken-tests"`-gated tests** (qa_work_handler, mutate, spec_handlers — see §4.5).
- [ ] **R3 — Audit `coverage(off)` over-suppression** (mechanical lift on pure-compute files; see §4.5).
- [ ] Continue drip-feed on remaining 0%-cov pure-compute slice (autonomous loop default).

Exit criteria: `make coverage-broad` reports ≥80% and `make coverage` (gate) stays ≥95%.

#### Phase 1 drip-feed log (appended per PR; keeps intent durable between sessions)

| PR | PMAT ticket | Surface | Line/branch targets | Strategic tactic |
|----|-------------|---------|---------------------|------------------|
| #394, #396 | PMAT-625 | `src/entropy/pattern_extractor_ruchy.rs` | pipeline-regex + `>3` / `>15` break, location capping, score variation | drip-feed (mutation) |
| #398 | PMAT-626 | `src/graph/builder_import_parsing.rs` | `parse_rust/python/typescript_imports` branches | drip-feed (mutation) |
| #415 | PMAT-627 | polyglot `NameResolver` | `can_resolve` fall-through branches | drip-feed (mutation) |
| #420 | PMAT-628 | polyglot `resolve_against_name_map` | target-missing-from-name-map branch | drip-feed (mutation) |
| #494 | PMAT-629 | `src/services/rust_wasm_analyzer.rs` (`deep-wasm`-gated) | `analyze_impl_method` disjunction + guard | drip-feed (mutation) |
| #495 | PMAT-630 | `src/services/accurate_complexity_analyzer_core.rs` | `analyze_function` BH-MUT-0002 `&&` truth-table + `has_suppress_annotation` branches | drip-feed (mutation) |
| #496 | PMAT-631 | `src/services/rust_project_score/known_defects_scorer_scoring.rs` | `score_internal` Cargo.toml-missing, `recommendations` empty/Err arms, 99/100 boundary, `score_with_mode` delegation | drip-feed (mutation) |
| #487 + #497 | — / PMAT-632 | `src/models/refactor_impls.rs` `Violation::to_op` | fall-through arms (#487) + ExtractFunction constant pins (#497 — location-field `+10` offset, `byte: 0`/`100`, `params: vec![]`) | drip-feed (mutation) |
| next | PMAT-633 | `src/scaffold/agent/templates.rs` | MCP + state-machine generators: Standard/Strict/Extreme branching, validate_context err, ctx.name flowthrough, all generated file paths, Cargo.toml pmcp dep pinning, AgentTemplate serde round-trip | **tactic #3: scaffold 0→90%** |
| Wave 30 (2026-04-25, branch `coverage/wave30-helpers`) | — | 21 files: deps_audit/graph, quality_gate_service, qa_work_handler/impl_print, roadmap_impl, dead_code_handlers_output, duplicates_output, satd_handler_formatting, oracle_handlers_formatting, project_diag_advanced_formatters, qa_work_handler/format_checklist_text, help_generator_formatting, satd_detector/types, spec_handlers_sync, markdown_best_practices, extended_tools_complexity, formatters_helpers, proof_annotation_helpers_report, enrichment, satd_detector/detection_extraction, work_handlers/resolution | ~280 tests; +4.03pp (73.42% → 77.45% measured) | drip-feed (CLI handlers + format dispatchers); see §4.5 for diminishing-returns analysis |
| Wave 31 (2026-04-25, branch `coverage/wave30-helpers` cont'd) | — | 10 files: comprehensive_handler_analysis (30), cli/colors (32), lua_best_practices/cb611_cb612_cb613 (47), cb616_cb617 (40), contracts/mcp_mapping (20), defect_helpers/format_markdown (29), lua_best_practices/cb614_cb615 (32), cb618_cb619 (25), cb608_cb609_cb610 (25), cb604_cb605 (11) | **291 tests across 10 PRs** (slightly exceeds wave 30 in a single session); broad pre-wave-31 baseline = **77.87%**, mid-wave measurement = **77.98%** (LCOV captured 2026-04-25T14:16-14:46Z, snapshotted ~149 tests of the 291; final post-wave figure pending re-run) | drip-feed shifted to **`coverage(off)` parent files** — broad-gate-only modules with high pure-compute density (lua compliance helpers cb6XX series 7 of 9 + ANSI formatters + MCP-CLI contract bridge + defect markdown). All 10 picks were standalone files (not include!()'d) with 0 prior tests; no broken-tests revival attempted in this wave. **Observed rate ≈ 1,350 tests/pp on these tiny lua-helper files** — much worse than wave 30's ~70 tests/pp on handler-format files; root cause = tiny per-test line surface (3–5 lines/test on cb6XX vs 15–30 on wave-30 handlers). |

Five Whys pivot (2026-04-23): the drip-feed pattern (#394..#497) optimizes for mutation-killing on files *already in the narrow-gate measured set* and leaves the 73% broad baseline effectively unchanged (~0.03 pp per PR on a 324k denominator). Phase 1 exit requires ≥80% broad, which needs one of the four listed tactics, not more drip-feed. PMAT-633 starts tactic #3 (scaffold). Next Phase-1 picks should come from tactics #1 (dead-code delete), #2 (un-skip `cli_integration_tests`), or #4 (CLI match-arm collapse) — each moves the denominator or adds bulk numerator, not individual functions.

Five Whys revision (2026-04-25, post-wave-30): drip-feed continued through wave 30 (~280 tests, +4.03pp) but rate held at ~70 tests/pp. §4.5 identifies three structural ceilings (target exhaustion, stale-test gates, CI throughput) and proposes 5 escalation paths (R1–R5). **Phase 1 exit (80%) now requires R1 (revive broken-tests gates) or R3 (lift over-suppressed `coverage(off)`) in addition to drip-feed continuation.** R2 (bundle PR) is needed to unblock CI cadence regardless of which numerator-growth path chosen.

Wave 31 strategy note (2026-04-25): wave 31 explicitly targeted the `coverage(off)` files §4.5-R3 names but did **not** lift the attribute — instead added tests *inside* the off-counted modules. Rationale: tests still contribute to broad gate (which doesn't honor `coverage(off)`), give regression value, and avoid the per-file policy review R3 strict mechanical-lift would require. Net effect: same broad-gate gain as R3 with smaller blast radius. Pre-pick screening this wave caught two dead-end candidates: `unified_protocol/adapters/cli_helpers.rs` (broken `unified-protocol` feature), `popper_score_format_markdown.rs`/`refactor_handlers_status.rs`/`refactor_auto_types.rs` (already covered by sibling _tests.rs files — initial heuristic missed parent test files).

Pattern for pickers (per-session loop): run `pmat query --coverage-gaps --rank-by impact --limit 30 --exclude-tests` when coverage data is present, else fall back to `pmat query --faults --max-complexity 12 --rank-by impact`. Always: skip feature-gated code unless its feature is on in the coverage invocation, skip `coverage(off)` modules for the narrow gate but remember they still count in broad, prefer surfaces where the tests-per-function ratio on the existing suite is <1. **Before picking, classify the target against the four Phase-1 tactics — if none apply, the pick is drip-feed not Phase-1 critical-path.**

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
