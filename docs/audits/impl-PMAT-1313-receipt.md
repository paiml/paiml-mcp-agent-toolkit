# IMPL-PMAT-1313 — the PR lane, a 12.2x that CI refuted, and a coverage gate that had never run

## Identity

| field | value |
|---|---|
| ticket | PMAT-1313 (`kind:code`), filed issue-first as #1313 |
| spec | `docs/specifications/goal-mode.md` §7.3 (written by this ticket) |
| branch | `PMAT-1313-pareto-pr-lane`, from master `204eb8959`; merged as **`91b5c00c7`** (PR #1316) |
| `gate_cmd` | `pmat verify` — **`gate_cmd_fallback=true`**: the full CI lane was run instead, twelve times, because this ticket's subject IS the lane |
| model gate | `model=opus class=opus decision=admit basis=file` |
| operator instruction | "Pareto optimized PR vs pre-release process using 80/20 rule … 20% of tests will test 80% of code in PR so time should be 80% faster"; later "continously dogfood progress by doing cargo install ."; later "coverage always at 95%" |

## The claim I shipped, and the number that refuted it

| runner | 21,536 lib tests | where |
|---|---|---|
| `cargo test --lib` | 1227.83s | what `ci / test` ran |
| `cargo nextest run --lib` | **100.51s** | 12.2x — on this workstation |

I set `use_nextest: true` and **CI got slower**. `sovereign-ci.yml` sets `NEXTEST_TEST_THREADS=4`
in a container given `--cpus 8`, while `cargo test` uses available parallelism — 8. Halving the
workers while paying a process spawn per test loses more than isolation wins. Measured on this
PR's own first head: **9,047 of 21,536 tests after 11 minutes**, against 20.5 minutes for the
whole suite under `cargo test`.

**A speedup measured on a 32-core workstation is not a speedup; it is a hypothesis about the
runner.** `use_nextest: false`, the numbers sit beside the input, and `pr-lane-control.sh` arm 2
refuses a workflow that re-enables it while the cap stands — the falsifier now protects the
refutation, not the claim. The cap is upstream: PMAT-1315.

## What actually made the lane faster

Two tests built an `AgentContextIndex` over the **whole repository**, 160s each.
`test_make_cluster_item_basic` never read the result — its only use was `is_err()`, and every
assertion below it is on a struct literal. A third cost came from `handle_localize` shelling out
to `cargo llvm-cov --version` to decide whether to print a hint: 1.7s in a shell, **75s inside
`cargo llvm-cov nextest`**, where the nested cargo contends for the outer build lock.

| job | before | after |
|---|---|---|
| `ci / test` | 34 min (1227.83s execution) | **24 min** (1004.82s) |
| `ci / coverage` | 29 min | **25 min** |

`cargo test`'s thread pool had hidden all three for as long as they existed. Per-test isolation
found them on its first run — the argument for nextest, restated as the defect it found.

## The coverage gate had never executed

`Enforce coverage floor` is skipped unless `coverage_min` is set, and this repository had never
set it: **~29 minutes of every pull request measured a number nothing read.** Armed at 80.

Its first real verdict, run `34599237562`:

```
Measured line coverage: 85.57% (245585/287014 lines)
Ratchet baseline (.coverage-baseline.txt): 85.43%
Coverage 85.57% ≥ floor 85.43% — ratchet satisfied
```

**The baseline file could not have worked where the gate looks for it.** The default is
`.pmat/coverage-baseline.txt` and this repository gitignores `**/.pmat/`; git cannot re-include a
file under an ignored directory, so a baseline there never reaches CI and the "ratchet" is
silently `coverage_min` alone. It lives at `.coverage-baseline.txt`.

**And the ratchet failed its own second run.** Identical Rust, two consecutive runs: 85.57% then
85.56% — two lines of 287,014 moved. A baseline set to the exact last reading fails on noise, so
the committed value sits ~0.07 points below the measurement, ten times the observed swing, and
the rule for raising it is written beside the input.

95% is 29,032 lines away; a gate set there today reddens every PR, which §5.4 forbids. The climb
is PMAT-1317.

## What covering four 0.0% files found

Twenty tests, each under a tenth of a second, over `analysis_sections.rs` (0/209),
`work_quality_handlers.rs` (0/263), `cache_handlers.rs` (0/202) and `test_handlers.rs` (0/245).
Three defects, each pinned by a test whose message says what to do when it starts failing:

| | what |
|---|---|
| **PMAT-1319** (#1319) | `include_analyses: [Complexity]` without `Ast` returns `Some(report)` with zero files — the phase reads a cache only the AST phase fills. **Fixed and merged** (#1322). |
| **PMAT-1320** (#1320) | Two Popper falsifiers pass on MISSING evidence: no coverage history and no release binary both printed "validated". **Fixed** (#1323). |
| **PMAT-1321** (#1321) | 97 tests for the CLI's central dispatch have not compiled since a file split — lifted alone, the failures are stale imports and a removed `demo` module, not the "broken syntax" the quarantine comment claims. |

## What this ticket's own gates caught in it

- **the ratchet**: a `panic!(` in a test and a `// TODO` opening a line inside a fixture string moved two baselines by one each. Both fixed at the cause; neither baseline was raised.
- **the unrun-tests ledger**: rendered twice from a binary that had not finished building, so "no change" read as "current" and failed both long jobs on `RENDERED TEXT DIFFERS`. A ledger check must run with a binary that demonstrably carries the change.
- **feature-matrix shard 3**: the test module's `use super::*` was gated behind `not(feature = "skip-slow-tests")`, which was fine while the slow test was the module's only test; the fast tests added here failed to compile under that feature.

## Verification (every command re-run by the orchestrator)

| what | result |
|---|---|
| `cargo nextest run --lib` | 21,536 passed, 0 failed, 100.51s |
| `cargo test --lib -- services::file_split` | 24 passed in **0.01s** (was 320s for two of them) |
| `cargo llvm-cov nextest --lib` | 85.25% lines by llvm-cov's summary; **85.43%** by the gate's own lcov arithmetic |
| the gate's enforcement step, extracted verbatim and run on real `lcov.info` | passes at the floor, fails at min 90, fails at baseline 86.00, refuses an empty lcov |
| `scripts/pr-lane-control.sh` | 11/11 arms, 5 of them falsifiers |
| every other control, run against the **installed** binary | traceability 5/5, roadmap-coherence 11/11, work-sync 4/4, ticket-release 17/17, spec-epic 20/20, spec-review 27/27 |
| `pmat work sync --check-only` (live) | exit 0, bijection |
| CI on the merged head | 45 checks, 0 failures |

## Dogfooding

`cargo install --path .` was re-run at every landing; `pmat --version` reports the commit it was
built from, and the controls above were run with that binary rather than a debug build. At the
merge: `commit: 91b5c00c7`.

## Process finding, recorded against myself

Polling three PRs' check-runs with `--paginate` every ~55s tripped GitHub's **secondary** rate
limit. `gh api rate_limit` reported core 5000/5000 the whole time — it reports only the primary
bucket. CB-2115 then read `not_measured` on a rate-limited `gh api` call and failed closed, which
is correct, and **reddened a PR for a reason that had nothing to do with its diff**. The gate was
right; my API budget was the defect. Poll with one `gh pr checks` at minutes, and reproduce a red
`traceability` locally with `comply check --checks CB-2113,CB-2115` before touching CI.

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 601

routes:
  ph1  class=measurement     route=self  w=100.00  basis=absent   note=per-test timing, both runners, one machine one binary
  ph2  class=impl            route=self  w=100.00  basis=absent   note=the two repo-scanning tests and the 75s subprocess hint
  ph3  class=impl            route=self  w=100.00  basis=absent   note=the coverage floor armed, the baseline file moved out of the gitignore
  ph4  class=impl            route=self  w=100.00  basis=absent   note=twenty tests over four 0.0% files
  ph5  class=impl            route=agy-goal w=1.00 basis=absent effort=1[U]  note=PMAT-1319 and PMAT-1320 dispatched in parallel to two paiml-impl-workers on disjoint scopes, each verified by the orchestrator with its own RED control
  ph6  class=orchestration   route=self  w=100.00  basis=absent   note=merge, tickets, receipt

verification:
  cmd=cargo-nextest-run--lib  claimed_exit=-  rerun_exit=0(21536-passed,100.51s)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/nextest.log
  cmd=cargo-test--lib(CI)  claimed_exit=-  rerun_exit=0(1227.83s)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/measure2.log
  cmd=cargo-test--lib--services::file_split  claimed_exit=-  rerun_exit=0(24-passed,0.01s)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/measure-cov.out
  cmd=cargo-llvm-cov-nextest--lib  claimed_exit=0  rerun_exit=0(85.43%-by-lcov-DA)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/measure-lcov.out
  cmd=gate-enforcement-step(4-cases,extracted-verbatim)  claimed_exit=-  rerun_exit=0,1,1,1  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/gate-step.sh
  cmd=pr-lane-control(11-arms,5-falsifiers)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/clippy2.log
  cmd=every-control-vs-INSTALLED-binary  claimed_exit=-  rerun_exit=0(5,11,4,17,20,27-arms)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/dogfood-install.out
  cmd=pmat-work-sync--check-only(live)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/check-rt2.log
  cmd=PMAT-1320-worker-RED(mutant-restores-pre-fix-arms)  claimed_exit=-  rerun_exit=1(exactly-2-of-6-die)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/clippy2.log
  cmd=PMAT-1319-worker-RED(spawn.rs-reverted)  claimed_exit=-  rerun_exit=1(inverted-test-fails)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/clippy.log
  cmd=ci-on-the-merged-head  claimed_exit=-  rerun_exit=0(45-checks-0-failures)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/pareto/pr-body4.md
