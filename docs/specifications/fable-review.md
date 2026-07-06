# PMAT Architecture Review — EV-Ordered, Falsifiable Engineering Roadmap

> **Provenance.** This document is the output of a gate-integrity audit of
> `paiml/paiml-mcp-agent-toolkit` (crate `pmat`) run against `origin/master`
> HEAD **`f4ce4f980`** (pushed 2026-07-05T19:35Z), verified 2026-07-05/06.
> Every factual claim is backed by a live artifact fetched during the audit:
> a raw file (`path:line`), a command with its output, a `gh api` result, or
> crates.io metadata. No claims are from training data or prior belief.
>
> **Access.** The audit had better-than-live access: a local clone pinned and
> `git diff`-verified to `origin/master`, live `gh api` (branch protection,
> rulesets, check-runs, issues, the `paiml/.github` reusable workflow), live
> crates.io API, and **execution of `pmat` 3.24.2 (== HEAD version) against
> itself**, including red-team violation injections in an isolated git worktree.
>
> **Method.** A 10-agent verification fleet: 8 parallel readers (manifest/dead-tests,
> CI-gates, roadmap, contracts/Verus, mutation engine, analyzers, deps/clean-room,
> work-sync/MCP) + 2 self-application agents (green-path self-gates, worktree
> red-team injections). Full per-agent evidence (121 ledger facts, 82 findings,
> raw command outputs) archived in the session scratchpad.
>
> **Scope note.** pmat has **no** pillar-vision/beat-scoreboard document; this
> frame is derived from pmat's own correctness surface, not imported from aprender.
> Roadmap ids follow pmat's `github_enabled` convention (`GH-<n>` or descriptive
> slugs), **not** the `PMAT-xxx` scheme.

---

## 1. Executive summary — the load-bearing findings

pmat is the tool that enforces the stack's doctrine on ~30 fleet repos and
generates the AI context those repos are reasoned about with. A false negative
in pmat's own gate/analysis engines therefore propagates as false confidence
fleet-wide — the CF-4 blast radius is stack-wide, not local. The audit found
that **pmat's most doctrinally load-bearing surfaces are theater**:

1. **Merge gating is nullified at the top.** The active org ruleset (13878864,
   "Green Main") requires status checks `kani`, `lake-build`, `workspace-test`
   that **no pmat workflow produces**, with `OrganizationAdmin bypass_mode: always`.
   Every merge (verified on PR #617's `statusCheckRollup`) rides the admin bypass,
   so **no required check — including `gate` — is hard-blocking in practice**.
   Classic protection additionally has `enforce_admins: false`.

2. **Merge CI runs only `cargo test --lib` on the root crate.** All 7 `[[test]]`
   integration targets (203 reachable files, incl. mutation fixtures, `pmat context`
   tests, MCP parity corpus) and the `pmat-dashboard` workspace member never
   execute at the gate. Everything else CI "runs" (fmt, deny, coverage %, score,
   ladder, comply, mutants, MSRV) is advisory, computed-only, or post-publish.

3. **The dead-test-corpus hazard §1 predicted is realized at scale.** The entire
   `src/tests/` tree — **189 files, 76,501 LOC, 3,877 `#[test]` fns** — has been
   orphaned (compiled by nothing) since commit `c99f3c7a8` (2026-04-05); the
   removing commit's stated justification is false for 188 of 189 files. A further
   26 `tests/` files (72 tests) are unregistered under `autotests = false`.

4. **The mutation-testing product cannot kill a mutant.** `pmat mutate`'s native
   path writes mutants to `temp_dir` and runs tests against the **unmutated** tree
   (`engine.rs:150-177`) — structurally, a mutant can only ever be `CompileError`
   or `Survived`. Four of six language backends (Go/Python/C++/WASM) hardcode
   `passed:true`. This is the mechanization of "gates or theater" that the fleet
   depends on, and it is itself theater.

5. **`pmat verify`'s SATD gate is structurally incapable of failing.** Its strict
   classifier requires a literal `//` prefix that the comment extractor has already
   stripped (`detection_extraction.rs:145`), so `--strict` finds 0 violations
   repo-wide; `--fail-on-violation` is a dead flag never read in the dispatched
   handler. Injecting `// TODO: HACK ... FIXME` leaves `pmat verify` green.

6. **`pmat comply check` — the gate sold to the fleet — is non-reproducible.** It
   returns `COMPLIANT` (0 fail) on the dev tree but exits 1 on a fresh checkout of
   the *same commit* (CB-1201: `pmat-core.yaml kani_harnesses[0]` schema parse
   error). A compliance verdict that depends on gitignored local state is broken
   by construction.

**Genuine positives worth crediting:** the L5 Lean lane (`lake build` +
`sorry`/`admit` grep) is a real hard-fail in CI and hole-free; `build.rs` panics
on bad contract-binding status (real compile-time gate); `pmat verify`'s **format
and complexity** stages turn genuinely RED on injected violations; the PMAT-154
self-dirty-loop fix is real and unit-tested; the live pmcp MCP server (20 tools)
has genuine compiled parity tests; the `pmat serve` HTTP stub follows an honest
exit-2 failure policy. The complexity `?`-counting divergence (GH-573) is fixed
and pinned by a regression test.

---

## 2. Doctrine that constrains this roadmap

(Stack-wide; obeyed, not relitigated.)

1. **Proofs/gates over breadth (EV rule).** Never propose a breadth feature (an
   Nth analyzer, an Nth mutation backend, another `query` mode) when a falsifiable
   correctness gate/proof is on the table. A wrong analyzer poisons fleet-wide
   decisions.
2. **Gates or theater.** Every gate must be mutation-verified. A green gate with a
   too-weak falsifier is worse than no gate. For pmat this is recursive: its own
   gates must be mutation-verified, and its mutation/verification engines must
   themselves be adversarially verified.
3. **Contract per feature.** Invariants + `falsification_tests` (+ Kani/Verus where
   applicable) ship in the same PR as the feature.
4. **Stop the line.** No skipped tests, no `--skip`, no "upstream issue" excuses;
   five-whys to the owning module, fix at root.
5. **Sovereign by construction.** Pure Rust, self-hosted CI, clean-room
   publishability (builds from crates.io alone) as a HARD release gate.
6. **Quorum-validated design.** Architecturally-significant designs validated
   against ≥3 world-class reference systems before build.
7. **Failure archaeology.** Every escaped defect → post-mortem + permanent
   falsifier; the case-file corpus is append-only.

---

## 3. Verification ledger

`verdict`: **VERIFIED** (confirmed at HEAD with source) · **STALE** (snapshot ≠ HEAD; both given) · **UNVERIFIED** (could not fetch).

| claim | snapshot value | HEAD value | source | verdict |
|---|---|---|---|---|
| Default branch + protection target | `master` | `master`; classic protection contexts=`["ci / gate"]`, `enforce_admins=false`; org ruleset 13878864 active on master | `gh api .../branches/master/protection`, `.../rulesets/13878864` | VERIFIED |
| Package identity | pmat 3.24.2, edition 2021, rust 1.91.0, MIT OR Apache-2.0 | identical | Cargo.toml:1-10 | VERIFIED |
| Workspace: ONE member `crates/pmat-dashboard` | one member | confirmed; `[workspace.package]` version **3.17.0 lags root 3.24.2**; dashboard has 3 deps, never tested in CI | Cargo.toml:785-794 | VERIFIED |
| No `rust-toolchain.toml` | 404 | absent at HEAD | `git show origin/master:rust-toolchain.toml` (fails) | VERIFIED |
| MSRV enforcement | unverified | **absent from all merge-gating CI**; sole check = `post-release.yml msrv-check` (runs after crates.io publish); failure mode already shipped once (v3.24.1 hotfix) | post-release.yml; 15 workflows enumerated | VERIFIED (gap) |
| `autotests=false` + "193 .rs files" note | note as quoted | setting confirmed (Cargo.toml:17-18); count stale: **236** files under tests/ | Cargo.toml:15-18; `find tests -name '*.rs'` | STALE (comment) |
| Dead-test hazard | "structurally baked in" | 26 unregistered tests/ files (72 tests, 7,411 LOC) + **entire src/tests/: 189 files, 76,501 LOC, 3,877 tests orphaned since c99f3c7a8**; rationale false for 188/189 | reachability fixpoint; `git show c99f3c7a8 -- src/lib.rs`; src/lib.rs:639-641 | VERIFIED |
| Binaries | pmat + pmat-agent (`mcp-integration`) | confirmed (actix 0.13 + actix-rt 2.10, `#[actix_rt::main]`) | Cargo.toml:24-27; src/bin/pmat-agent.rs | VERIFIED |
| Real surface ≫ "work/query/comply" | list in §1 | **69 top-level commands; `analyze` alone has 32 subcommands**; 7 `[[test]]`, 10 `[[bench]]`, 30 `[[example]]` | `pmat --help`; Cargo.toml:539-709 | VERIFIED |
| `aprender-orchestrate 0.50` + `org analyze` stub | as stated | pin 0.50 (lock 0.50.0); stub bails citing 0.41 API removal; crates.io latest **0.51.0**; API absent even at aprender monorepo HEAD 0.59.0-dev → unstub **blocked upstream**; org-intelligence test profile fails E0004; **no tracking issue exists** | org_handlers.rs:15-21,96-120; /home/noah/src/aprender (171816661); `gh issue list --search` | VERIFIED |
| `batuta-common = "0.1"` pre-consolidation name | as stated | pinned 0.1 = crates.io latest 0.1.0 (name legacy, version current) | Cargo.toml:31; crates.io API | VERIFIED |
| ML/text/graph consume aprender from crates.io | Sprints 45–46 | aprender 0.50 direct + 10-crate family; no trueno-* direct dep | Cargo.toml:61-275 | VERIFIED |
| roadmap.yaml `roadmap_version: '1.0'`, `github_enabled: true` | as stated | confirmed; 197 items; mtime 2026-07-04 | docs/roadmaps/roadmap.yaml:1-3 | VERIFIED |
| roadmap ids "GH-`<n>` + slugs, NOT PMAT-xxx" | as stated | **FALSIFIED: 118/197 ids are `PMAT-*`**, 20 `GH-*`, 24 `MACS-*`, plus slugs, file-paths, free-text-sentence ids | `grep '^- id:'` (197 ids) | STALE |
| Roadmap recency ("2025-11") | concern raised | refuted: modal created-month 2026-02 (60/197); 24 MACS items added 2026-07-04; 2025-11 = 18% | month histogram | VERIFIED |
| GH-`<n>` ids vs live issues | reconcile | 8/8 sampled in sync (completed↔CLOSED); **but 27 planned/inprogress items map to no issue and repo has 0 open issues**; MACS-004..019 marked inprogress/planned though shipped | `gh issue view` ×8; `gh issue list --state open` → 0 | VERIFIED (local-id drift) |
| ROADMAP.md 220 KB | 220 KB | 220,308 B; explicitly frozen "HISTORICAL — SUPERSEDED v2.192.0"; banner rot ("project is on 3.17.0+", points at empty issue backlog) | ROADMAP.md:1-20; git log | VERIFIED |
| deny.toml present | present | present ("Deployed from paiml/infra"); CI runs `deny check advisories licenses sources` **with continue-on-error; `bans` never runs** while lock carries 4 duplicate-version pairs | deny.toml; sovereign-ci.yml:433-440 | VERIFIED |
| CI: `gate` needs `[ci]`, reusable workflow | confirm sovereign-ci | confirmed: `paiml/.github/.github/workflows/sovereign-ci.yml@main` (**unpinned**; nightly-bench pins a SHA) | ci.yml:13-17 | VERIFIED |
| cargo-mutants job post-gate, non-blocking | as stated | worse: `continue-on-error: true` at job AND step, master-push only (never PRs), artifact `mutants.out/` has **zero consumers**, no threshold | ci.yml:38-58 | VERIFIED |
| Required check `gate` (Green Main) | as stated | ruleset requires `gate, kani, lake-build, workspace-test`; **last 3 phantom; OrganizationAdmin bypass_mode=always ⇒ every merge uses bypass (PR #617 rollup has none of the 3)** | `gh api .../rules/branches/master`; `gh pr view 617` | VERIFIED |
| What gate-CI actually enforces | verify enforced vs computed | enforced: `cargo test --lib` (root only), clippy `-D warnings -A unused-variables` (fallback leg drops `--all-targets`), llvm-cov command success (no % floor), cargo audit. Advisory: fmt, deny, codecov, score, ladder, comply, mutants, bench, provenance, MSRV. Internal gate passes on `cancelled` | sovereign-ci.yml:243-270,414-440,569-607,981-1007 | VERIFIED |
| `contracts/` corpus exists? | README 404 → "absence is a finding" | **EXISTS: 141 tracked files** — 19 pv-style registry YAMLs (invariants + falsification_tests + kani_harnesses), 116 work snapshots, real Lean 4 proofs (6 theorems, 0 sorry/admit); enforcement: build.rs binding gate (hard), Lean lake-build (hard, job not required-check), pv binary invoked nowhere, comply advisory | `git ls-files contracts/`; contracts/pmat-core.yaml; build.rs:1494-1557 | STALE (snapshot wrong) |
| Verus lane: CI verifies discharge | verify | **no Verus discharge anywhere**: demo writes toy `verus!{}` strings to a TempDir and regex-scores them ("no subprocess calls"); Verus in 0/15 workflows; Full mode runs `cargo kani --only-codegen` yet parses for "VERIFICATION:- SUCCESSFUL" | examples/verus_verification_demo.rs:38-166; formal_verification_scoring.rs:34-54 | VERIFIED (theater) |
| Kani harnesses | — | 30 real `#[kani::proof]` across 8 files; executed by nothing (no CI job, no Makefile target, `min_kani_coverage: 0`); pmat-core.yaml claims `l3_kani_proved: 10` with no evidence | grep `kani::proof`; .pmat.yaml:33 | VERIFIED |
| Mutation PASS semantics | "what constitutes PASS" | no default threshold; zero-mutants → `Ok(())` **before** threshold check; CompileError excluded from denominator; **native `pmat mutate` tests the unmutated tree — can never Kill**; Go/Py/C++/WASM adapters hardcode `passed:true`; `--language`/`--jobs` ignored; default features exclude mutation-testing | engine.rs:150-177; mutation_handlers_core.rs:82-90; types_impls.rs:30-36; go_adapter.rs:74-84; Cargo.toml:307 | VERIFIED |
| Coverage enforced? | verify | computed only; floor step `if: coverage_min != ''`, pmat passes only `repo` ⇒ skipped; "95% minimum" enforced nowhere | sovereign-ci.yml:569-607 | VERIFIED |
| crates.io publish-lag | 3.24.2 snapshot | 3.24.2 published 2026-07-05T12:43Z; HEAD same day — **lag ≈ 0**; but CHANGELOG.md has **no 3.24.2 entry** | crates.io API; CHANGELOG.md:10 | VERIFIED |
| Clean-room membership | "guide omits pmat; verify" | full clean-room pipeline exists **but is `.disabled`** (release.yml.disabled); live path = interactive `Makefile crate-release` (`cargo publish` behind echo-only checklist); **no OOM caps** anywhere | release.yml.disabled; Makefile:1228-1250 | VERIFIED (finding) |
| Doc-path drift | aprender-ml-integration.md 404 | confirmed + 2 more Cargo.toml-cited specs missing; README's 4 refs resolve; Cargo.toml description claims "HTTP" while `pmat serve` is honest exit-2 stub; docker-publish tags `:2.10.0` on a 3.24.2 crate | Cargo.toml:7,60,68,449; utility_serve_handlers.rs; docker-publish.yml:42 | VERIFIED |
| Self-application: green path | highest leverage | comply **154 checks, 0 fail, 12 warn, COMPLIANT (dev tree)**; verify ok (fmt/complexity/satd); rust-project-score 238.1/289 (A-; Build Performance ✗ 10/15); dead-code 0.0%; query self-run healthy (20,425 fns, TDG/complexity/Big-O annotations); last 10 master CI runs green | commands run 2026-07-05 | VERIFIED |
| Self-application: do gates turn RED? | the load-bearing question | **format RED, complexity RED (CC=37 injected)**, **SATD GREEN-despite-violation** (strict requires `//` the extractor strips; `--fail-on-violation` dead); `pmat quality-gate` exits 0 printing 200 violations; dead-code misses injected private uncalled fn; **comply exit 1 on BOTH clean+injected fresh worktree (CB-1201) — differs from dev tree** | worktree injections; verify.rs:220-223; classifier.rs:250-283; detection_extraction.rs:145 | VERIFIED |
| Complexity #573 divergence | 2.4× (memory) | '?'-counting FIXED (GH-573 CLOSED + regression test); **new measured 2× split remains: `--file` reports CC 73/cog 143 vs `--path`/verify 37/36 on the same true-CC-37 fn**; naive substring-count fallback still ships | accurate_complexity_analyzer_visitor.rs:130-135; red-team measurement | STALE (issue) + VERIFIED (residual) |
| F11 work-complete self-dirty loop | known non-idempotence | filter fix real (PMAT-154); residual non-idempotence: `Completed→Completed` hard-errors; crash-mid-sync wedges ticket; ticket-B-after-A fails on pmat's own unpushed auto-commit; e2e retry tests dead behind `broken-tests` + stale signature | pmat_owned_state.rs:15-34; roadmap_status.rs:101-113; commit.rs:165-245 | VERIFIED |
| MCP surface | pmat-agent actix | live pmcp server = 20 tools with genuine compiled parity tests; **≥9 tool-list artifacts** repo-wide; agent-server has 3 divergent internal inventories (4 vs 6 vs 6) incl. **uncallable advertised tool `get_quality_status`**; mcp.json drift check (CB-1656) advisory-only | tool_manifest.rs:16-197; mcp_server_protocol.rs:229-307 | VERIFIED |
| AI-context correctness gate | "context reflects repo state" | none enforced: bug_007 renders its **own** markdown (production formatter untested); the only real content assertions (extreme_tdd_context_fix.rs) are in orphaned src/tests; mcp docs-sync tests all `#[ignore]` AND path-broken; validate-readme in no workflow | bug_007_function_count_tests.rs; mcp_documentation_sync.rs:36-44 | VERIFIED |
| O(1) metric gates (.pmat-metrics) | CLAUDE.md claims populated | store has ONLY 14 commit-meta snapshots + empty trends/; no lint/test/coverage/binary metrics recorded; `fail_on_missing_metrics=false` ⇒ silently passes; local release binary **50,782,952 B > 50 MB threshold**; toml comments stale; newest meta records grade F contradicting live green | .pmat-metrics/; .pmat-metrics.toml:9-14,28-29 | VERIFIED (gate hollow) |
| Lock-graph hygiene | — | duplicates: aprender 0.25.9 (via ruchy 4.2.1) + 0.50.0; aprender-db 0.50+0.51; aprender-graph 0.50+0.51; **arrow 57.3.1 + 59.0.0 across the boundary Cargo.toml:78 declares requires type identity** | Cargo.lock parse; Cargo.toml:78 | VERIFIED |
| User-CLAUDE.md claims ("ci/gate + workspace-test required, 95% coverage, 80% mutation") | — | workspace-test is a phantom org context; no coverage floor; mutation threshold read by nothing | protection/ruleset APIs; sovereign-ci | STALE |
| 14-day commit activity | re-verify recency | 29 commits since 2026-06-21: 6 version bumps (3.21.0→3.24.2), MACS Component 32, comply/provability hardening, macOS/aarch64 CI, MCP path-traversal fix, MSRV unbreak | `git log origin/master --since=2026-06-21` | VERIFIED |

---

## 4. EV-ranked backlog

Ordering gate: falsifiable proof/gate available (all below: yes), then rank by
(leverage × unblocks) ÷ cost. Self-application / meta-integrity / enforced-not-computed
conversions rank above new analyzers, languages, and plumbing because pmat's
blindspots are fleet-multiplied.

```yaml
- id: unbypass-required-checks
  surface: infra
  type: gate
  ev_rank: 1
  ev_rationale: Every gate item below is void while merges ride the OrganizationAdmin always-bypass past 3 phantom required contexts — near-zero cost, unblocks the entire gate program, fleet-multiplied because pmat's green-main is the fleet's exemplar.
  definition_of_done: "Ruleset 13878864 (or a repo ruleset) requires only contexts pmat emits (gate, and later 'pmat score'/kani); bypass_mode set to pull-requests-only or removed; enforce_admins=true on classic protection. Falsifier: a PR with a deliberately failing `gate` check must be unmergeable by the org admin via UI/API without an audited bypass event (PR #617-style merges prove the opposite today)."
  blocked_by: none
  artifact_on_completion: falsifier
  workflow_note: org/repo settings change + a doc commit recording the ruleset JSON; no code PR; verify on a throwaway PR against master.

- id: run-declared-test-targets-in-gate
  surface: self-application
  type: gate
  ev_rank: 2
  ev_rationale: Merge CI runs only `cargo test --lib` — all 7 [[test]] targets (203 files incl. mutation fixtures, bug_007 context tests, MCP parity corpus) and the pmat-dashboard workspace member never execute at the gate; the largest single false-confidence generator.
  definition_of_done: "ci.yml passes test args (or test_workspace) so `ci / test` runs `cargo test --lib --test all --test contract_traits --test unit_core -p pmat` + `cargo test -p pmat-dashboard`. Falsifier: commit `assert!(false)` in tests/modules/mutate_command_tests.rs on a branch — `ci / test` must go RED (today stays GREEN: sovereign-ci.yml:243-270 TEST_SCOPE='--lib')."
  blocked_by: none  # feature-gated targets (integration/e2e/perf-tests) may phase in later
  artifact_on_completion: falsifier
  workflow_note: ticket → branch fix/ci-run-test-targets → PR → `ci / gate` (master); watch runtime budget — tests/all.rs is large.

- id: revive-orphaned-test-corpus
  surface: self-application
  type: gate
  ev_rank: 3
  ev_rationale: 189 src/tests files (76,501 LOC, 3,877 #[test]) compiled by NOTHING since c99f3c7a8, plus 26 unregistered tests/ files (72 tests) — includes the only tests asserting `pmat context` output content; the dead-test-corpus hazard §1 predicted, now quantified.
  definition_of_done: "A CI reachability gate walks the mod/#[path]/include! graph from src/lib.rs + all [[test]] roots and fails on any unreachable *.rs containing #[test] (fixtures allowlisted). RED at HEAD (189+26 files); GREEN only after each file is re-wired (restore the #[cfg(test)] mod block c99f3c7a8 deleted, or migrate to tests/modules/), fixed for signature rot, or deleted with rationale. Mutation: orphan any test file → gate RED."
  blocked_by: run-declared-test-targets-in-gate  # revived files must actually execute in CI
  artifact_on_completion: fixture_corpus
  workflow_note: ticket → branch → staged PRs (corpus is 76KLOC; expect rot triage) → `ci / gate` (master).

- id: satd-gate-red-path
  surface: analyzer:satd
  type: gate
  ev_rank: 4
  ev_rationale: Empirically proven GREEN-despite-violation — pmat verify's SATD stage is structurally incapable of failing (strict regexes require the `//` prefix detection_extraction.rs:145 already stripped; --fail-on-violation is a dead flag); every fleet repo trusting `pmat verify` inherits the blindspot.
  definition_of_done: "Fix classifier strict patterns to match post-strip text, wire fail_on_violation through satd_handler (exit nonzero on violations), pass it from verify.rs stage_satd, and remove the filename-contains-'test' exemption for src/ files (key on tests/ paths + #[cfg(test)]). Falsifier: `// TODO: HACK - placeholder kludge FIXME` injected in src/mcp_integration/prompts.rs must make `pmat verify --skip clippy,tests` exit nonzero with satd ok:false (today ok:true, exit 0); same comment in src/test_performance.rs must appear in the default scan."
  blocked_by: none
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); ships in next patch release — name the (still-missing) clean-room gate in the release checklist.

- id: mutation-kill-path
  surface: mutation:rust
  type: analyzer
  ev_rank: 5
  ev_rationale: pmat mechanizes "gates or theater" fleet-wide, yet `pmat mutate` can never Kill a mutant (tests run against the unmutated tree) and 4 of 6 language backends hardcode passed:true — the flagship product is the canonical CF-4.
  definition_of_done: "Route `pmat mutate` through the existing in-place MutantExecutor (MutantGuard + atomic_write, executor_single.rs) instead of MutationEngine's temp-dir path; honor --language/--jobs; delete or honestly-error the Go/Python/C++/WASM stub adapters (bail 'backend not implemented', never passed:true). Falsifier: `pmat mutate -t examples/rust-mutation-testing/src/lib.rs` must report killed>0 AND survived>0 (structurally killed=0 today); a Go run must error, not report 0% silently."
  blocked_by: none
  artifact_on_completion: fixture_corpus
  workflow_note: ticket → branch → PR → `ci / gate` (master); mutation-testing feature is outside default build — decide whether it enters `default` or stays opt-in with honest docs.

- id: mutation-meta-test
  surface: mutation:rust
  type: fixture-corpus
  ev_rank: 6
  ev_rationale: Doctrine §2 recursively applied — the harness itself must be adversarially verified; today a backend stubbed to return "all caught" passes every existing test, and all cargo-mutants fixtures derive from one hand-edited 5-mutant run.
  definition_of_done: "Harness-level meta-test: run the executor on a fixture project with a known-weak test suite and assert >=1 Survived and >=1 Killed; stub run_tests to always-Killed → meta-test RED. Plus: zero-mutants with --min-score set must return Err (today Ok before threshold check); [9x CompileError,1x Killed] must NOT report 100% (denominator fix or compile_error_rate>50% ⇒ FAIL); unknown cargo-mutants summary strings must error, not map to Unviable; fresh multi-file fixture recorded from cargo-mutants 27.x."
  blocked_by: mutation-kill-path
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); fixtures under tests/fixtures/ MUST be reachable per revive-orphaned-test-corpus gate.

- id: comply-fresh-clone-parity
  surface: self-application
  type: gate
  ev_rank: 7
  ev_rationale: `pmat comply check` — the gate pmat sells to ~30 fleet repos — returns COMPLIANT on the dev tree but exit-1 (CB-1201 pmat-core.yaml kani_harnesses schema error) on a fresh checkout of the SAME commit; a compliance verdict that depends on gitignored local state is non-reproducible by construction.
  definition_of_done: "Fix pmat-core.yaml kani_harnesses[0] (string vs struct KaniHarness, line ~146); make comply inputs hermetic (no verdict-bearing read of gitignored contracts/.pv or .pmat caches, or cache-miss ⇒ recompute identically). Falsifier: `git worktree add /tmp/w && cd /tmp/w && pmat comply check` must exit 0 and byte-match the dev-tree verdict; CI runs comply from its fresh checkout so the quality-gate log is the standing witness."
  blocked_by: none
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); prerequisite for comply-becomes-required.

- id: comply-becomes-required
  surface: self-application
  type: gate
  ev_rank: 8
  ev_rationale: The ALADR-012 "30-day grace" continue-on-error makes the entire CB ladder advisory; converting one computed gate to enforced is the doctrine's core move and unblocks CB-1656 (mcp.json drift), CB-1409 (uncontracted AI commits), and the six undocumented disabled checks.
  definition_of_done: "Flip continue-on-error:false on the 'Ladder gate — pmat comply' step and add the job's context to required checks; document or re-enable the 6 reason-less disabled checks (cb-081,120,500,600,900,1202) cb-125-style. Falsifier: add `sorry` to contracts/lean/Theorems/Macs/Ladder.lean OR hand-delete a tool from mcp.json — the PR must be unmergeable (today: red optional check, merges anyway)."
  blocked_by: comply-fresh-clone-parity, unbypass-required-checks
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` + ruleset edit (master).

- id: coverage-floor-ratchet
  surface: infra
  type: gate
  ev_rank: 9
  ev_rationale: Coverage is measured, uploaded, and enforces nothing (coverage_min unset ⇒ floor step skipped) while doctrine claims a 95% minimum — the textbook computed-not-enforced conversion, one `with:` line to fix.
  definition_of_done: "ci.yml passes coverage_min set to the measured current baseline (ratchet, not aspiration) and commits the baseline artifact. Falsifier: a PR deleting a covered test (dropping % below baseline) must turn `ci / coverage` and gate RED; today it merges green."
  blocked_by: run-declared-test-targets-in-gate  # baseline must include the revived corpus
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); coordinate with CB-125-B interim-accept roadmap.

- id: kani-context-goes-real
  surface: verification
  type: proof
  ev_rank: 10
  ev_rationale: 30 genuine #[kani::proof] harnesses exist and nothing ever executes them, while the ruleset already demands a check literally named 'kani' — highest proof-per-effort in the repo: the harness code is written, only the runner is missing.
  definition_of_done: "CI job (context name 'kani') runs `cargo kani --harness verify_*` over the 8 harness files and fails on any verification failure; min_kani_coverage raised from 0; pmat-core.yaml's 'l3_kani_proved: 10' regenerated from actual runner output. Obligation: all 30 harnesses discharge with no assume-escapes beyond the audited 42 domain-bounds. Falsifier: invert the ordering assertion in verify_ladder_ord_matches_numeric (work_verification_level.rs:208) — job RED (today the repo stays fully green)."
  blocked_by: unbypass-required-checks  # to make the context required rather than phantom
  artifact_on_completion: kani_harness
  workflow_note: ticket → branch → PR → `ci / gate` (master); self-hosted runner needs kani-verifier toolchain.

- id: context-output-fixture-gate
  surface: mcp-context
  type: fixture-corpus
  ev_rank: 11
  ev_rationale: pmat generates the context every stack agent reasons from, and the PRODUCTION formatter has zero content gates (bug_007 renders its own markdown; the real assertions are in the orphaned src/tests) — a drifting generator silently corrupts every downstream agent decision.
  definition_of_done: "Fixture repo with N known functions/types; test asserts the real `pmat context` / handle_context output (markdown AND llm-optimized) contains each declaration and the correct function count, via the production formatter. Falsifier: mutate the formatter to emit 'Functions: 0' or drop code blocks — test RED (bug_007 stays GREEN today). Revive extreme_tdd_context_fix.rs as part of the corpus."
  blocked_by: revive-orphaned-test-corpus, run-declared-test-targets-in-gate
  artifact_on_completion: fixture_corpus
  workflow_note: ticket → branch → PR → `ci / gate` (master).

- id: mcp-inventory-parity
  surface: mcp-context
  type: gate
  ev_rank: 12
  ev_rationale: 9 tool-list artifacts exist; only the live pmcp cluster is parity-locked (by fragile source-text parsing), the agent-server advertises an uncallable tool across 3 divergent internal inventories, and committed mcp.json has no compiled byte-check.
  definition_of_done: "(a) Compiled test: include_str!(mcp.json) == render_manifest(CARGO_PKG_VERSION); (b) agent-server parity test: tools_list ∪ capabilities ∪ dispatch name-sets equal, and every advertised name dispatches without Err('Unknown tool') — RED at HEAD on get_quality_status and the 4-vs-6 mismatch; (c) replace manifest_matches_server's source-text scan with a runtime tools/list assertion. Falsifier: hand-delete one mcp.json entry → compiled test RED (today merges)."
  blocked_by: none
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master).

- id: work-complete-idempotence
  surface: work-sync
  type: contract
  ev_rank: 13
  ev_rationale: Compounding-state surface with no live e2e tests (corpus dead behind feature=broken-tests with a stale signature): retry-after-success hard-errors, crash-mid-sync wedges the ticket, completing ticket B right after A trips on pmat's own auto-commit.
  definition_of_done: "Idempotent re-entry: `pmat work complete T` on an already-Completed item converges (exit 0 no-op or re-verify), crash between service.save and auto_commit_work_files converges on retry, test_github_sync exempts pmat-authored unpushed auto-commits. Tests call PRODUCTION test_github_sync() (not an inline re-implementation — mutation: drop the is_pmat_owned_state filter at core_checks.rs:477 → suite RED, today green). Falsifiers: the three scenarios as integration tests in a tempdir git repo, each RED at HEAD."
  blocked_by: run-declared-test-targets-in-gate
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master).

- id: dead-code-recall-corpus
  surface: analyzer:dead-code
  type: fixture-corpus
  ev_rank: 14
  ev_rationale: Red-team proved the detector misses a textbook dead function (private, uncalled, 12 lines) — a fleet-visible analyzer with unmeasured recall; "0.0% dead code" self-reports are currently unfalsifiable.
  definition_of_done: "Labeled fixture corpus (dead private fn, dead pub(crate) fn, dead module, transitively-dead cluster, live look-alike) with precision/recall assertions. Falsifier: `pmat analyze dead-code --path <fixture> --min-dead-lines 1` must list __definitely_dead_injected-style seeded functions (misses it entirely today) and `quality-gate --checks dead-code --fail-on-violation` must exit nonzero on the seeded fixture."
  blocked_by: none
  artifact_on_completion: fixture_corpus
  workflow_note: ticket → branch → PR → `ci / gate` (master).

- id: complexity-single-oracle
  surface: analyzer:complexity
  type: fixture-corpus
  ev_rank: 15
  ev_rationale: The only enforced analyzer still has >=3 code paths that disagree 2x on the same function (--file: 73/143 vs --path/verify: 37/36, true CC 37) — the pre-commit gate and the project scan enforce different realities (GH-573 fixed one axis, not the split).
  definition_of_done: "Cross-analyzer parity test: every complexity entry point (--file, --files, --path, AccurateComplexityAnalyzer) emits identical cyclomatic/cognitive for a reference fixture set including the 36-branch chain (expected CC=37, tolerance 0); delete the naive substring-count fallback (language_analyzer_analyses.rs:100-105). Falsifier: parity test RED at HEAD (73 != 37)."
  blocked_by: none
  artifact_on_completion: fixture_corpus
  workflow_note: ticket → branch → PR → `ci / gate` (master); after fixing, `cargo install --path .` before committing (pre-commit hook uses PATH binary).

- id: msrv-pr-gate
  surface: infra
  type: gate
  ev_rank: 16
  ev_rationale: MSRV 1.91.0 is verified at exactly one point — after the crate has already published — and this exact failure shipped once (v3.24.0 broke `cargo install pmat`, hotfixed in 3.24.1).
  definition_of_done: "PR-triggered job: dtolnay/rust-toolchain@<Cargo.toml rust-version> + `cargo check -p pmat --all-features`, wired into gate's needs. Falsifier: a PR using a post-1.91 stdlib API must turn gate RED pre-merge (today caught, at earliest, post-publish)."
  blocked_by: none
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); also add rust-toolchain.toml or document why not.

- id: mutation-ci-threshold
  surface: mutation:rust
  type: gate
  ev_rank: 17
  ev_rationale: The cargo-mutants CI job (runs the real external tool, independent of pmat's engine) is double continue-on-error with an artifact nobody reads and an 80% target read by no code — converting it is the repo's own "gates or theater" homework.
  definition_of_done: "Job parses mutants.out/outcomes.json and fails when caught-rate < recorded baseline (ratchet toward min_mutation_score_pct=80); remove both continue-on-error; fix mutants.toml schema (cargo-mutants 27 rejects 'timeout' field) and stale server/src/** paths so config actually applies. Falsifier: delete one test that kills a known-caught mutant → job RED on master push (today nothing can ever go red)."
  blocked_by: none  # independent of mutation-kill-path — uses external cargo-mutants
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); self-hosted runner budget: sample with --in-diff for PRs, full run nightly.

- id: sovereign-ci-hardening
  surface: infra
  type: gate
  ev_rank: 18
  ev_rationale: Four one-line theater fixes live in the shared reusable workflow, so each fix multiplies across every fleet repo: fmt advisory, deny advisory (bans never run), clippy fallback drops --all-targets, and the internal gate passes on 'cancelled'.
  definition_of_done: "In paiml/.github sovereign-ci.yml: fmt loses `|| echo`+continue-on-error; deny step enforced and includes `bans`; clippy fallback leg keeps --all-targets; gate compares results != 'success' (excepting declared skips). Plus pmat's ci.yml pins the reusable workflow to a reviewed SHA. Falsifiers: a misformatted-but-clippy-clean PR must turn `ci / lint` RED; a deny-level clippy warning inside #[cfg(test)] must turn lint RED; cancelling the test job mid-run must leave gate RED."
  blocked_by: none
  artifact_on_completion: PR
  workflow_note: cross-repo PR to paiml/.github (fleet-wide blast radius — stage behind an input flag), then pmat PR bumping the pinned SHA → `ci / gate` (master).

- id: clean-room-publish-gate
  surface: deps
  type: gate
  ev_rank: 19
  ev_rationale: The tool holding the fleet to "clean-room publishability as a HARD release gate" publishes via an interactive Makefile target whose checklist is echo-only text; the complete clean-room pipeline already exists — it is just .disabled.
  definition_of_done: "make crate-release gains an executing prerequisite: `cargo package` + build-and-test from the packaged tarball in a container with CARGO_BUILD_JOBS=2 and doctest --test-threads=4 (huge-crate OOM caps), non-zero on failure — either by re-enabling release.yml.disabled's verify job as a dispatchable workflow or a local containerized target. Falsifier: reference an include!()'d file excluded from the package — publish path RED before `cargo publish` runs (today nothing executes)."
  blocked_by: none  # manual-publish doctrine preserved: the gate precedes, not replaces, the human `cargo publish`
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); respects the documented manual-release preference — do NOT re-enable auto-publish.

- id: aprender-family-convergence
  surface: deps
  type: infra
  ev_rank: 20
  ev_rationale: The lock graph splits the sovereign foundation (aprender 0.25.9/0.50/0.51 triple, aprender-db+graph dual, arrow 57.3.1 vs 59.0.0 across a boundary Cargo.toml:78 itself declares requires type identity) — the CF-1599 facade/duplicate hazard, live in pmat's own lockfile.
  definition_of_done: "Uniform family bump to 0.51 (or graph back to 0.50); `cargo tree -d` shows zero duplicate aprender-*/arrow versions; ruchy bumped or its aprender 0.25.9 accepted with a documented waiver; `cargo deny check bans` added to CI with multiple-versions scoped-deny on aprender-*/arrow. Falsifier: the bans check is RED at HEAD (4 duplicate pairs) and GREEN after convergence; `cargo check --features analytics-simd` compiles against a single arrow."
  blocked_by: sovereign-ci-hardening  # for the enforced bans step; or a repo-local workflow step
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); verify aprender 0.51 API compat before bump (0.50→0.51 was breaking for orchestrate historically).

- id: org-analyze-resolution
  surface: deps
  type: infra
  ev_rank: 21
  ev_rationale: Live stubbed-feature regression with NO tracking issue, whose test profile doesn't even compile (E0004) — proving no lane builds it; unstub is blocked upstream (API absent even at aprender 0.59-dev), so the honest moves are decide-and-delete or rebuild natively.
  definition_of_done: "(1) File the tracking issue (stop-the-line accounting); (2) fix the E0004 non-exhaustive match (org_handlers.rs:232, missing Localize arm) and add a CI matrix leg `cargo check -p pmat --features org-intelligence --lib --profile test` (RED today); (3) decision recorded: delete the Analyze variant OR rebuild on pmat-native services (github_client + git_clone + defect analyzers) — no waiting on upstream. Falsifier: the feature-matrix check compiles green and an integration test pins whichever behavior was chosen."
  blocked_by: none
  artifact_on_completion: PR
  workflow_note: ticket (new GH issue) → branch → PR → `ci / gate` (master).

- id: self-metrics-honesty
  surface: self-application
  type: gate
  ev_rank: 22
  ev_rationale: pmat's self-scoring fabricates credit — RPS fast-mode awards full mutation points for a dead config file, coverage_improvement returns hardcoded 85/100, the O(1) metrics store is empty (binary already over its 50MB threshold unnoticed), and the freshest recorded meta grades the repo F while live gates are green.
  definition_of_done: "coverage_improvement returns Err (never 85.0/100.0) when cargo-mutants is absent/fails, and its --in-diff invocation takes a real diff file; RPS mutation credit requires `cargo mutants --list` to actually parse the config; metric recording wired so .pmat-metrics/ holds real lint/test/coverage/binary measurements with fail_on_missing_metrics=true. Falsifiers: PATH-stripped run must not return 85.0 (RED today); the recorded 50,782,952-byte binary must block a commit against binary_max_bytes=50_000_000 (silently passes today)."
  blocked_by: none
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master).

- id: docs-accuracy-required
  surface: mcp-context
  type: gate
  ev_rank: 23
  ev_rationale: pmat ships a documentation-accuracy enforcement engine that its own repo does not run in CI, while carrying live contradictions (Cargo.toml advertises HTTP against an exit-2 stub, 3 phantom spec paths, CHANGELOG missing the shipped 3.24.2, docker image tagged 2.10.0, ~20 phantom lean_theorem refs).
  definition_of_done: "A required CI job runs `pmat validate-readme --fail-on-contradiction` plus: Cargo.toml/CHANGELOG version-entry parity, doc-path existence for paths cited in Cargo.toml, lean_theorem resolution against built .olean, docker tag derived from git ref. Falsifier: the job is RED at HEAD on all five defect classes above and stays RED if a README claim contradicting the AST is seeded."
  blocked_by: unbypass-required-checks
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master); fix the five current contradictions in the same PR so the gate lands green.

- id: quality-gate-exit-semantics
  surface: self-application
  type: gate
  ev_rank: 24
  ev_rationale: `pmat quality-gate` exits 0 while printing 200 violations, and its enforcing flag is permanently red against a 197-violation baseline — the command fleet repos are told to trust cannot signal a NEW regression by exit code in either mode.
  definition_of_done: "Baseline/ratchet mode: default run fails on violations NOT in a committed baseline (and the baseline only shrinks). Falsifier: on a tree whose only delta is one injected CC=37 function, `pmat quality-gate` must exit nonzero (today: exit 0 with the violation printed); on the clean tree it must exit 0."
  blocked_by: none
  artifact_on_completion: falsifier
  workflow_note: ticket → branch → PR → `ci / gate` (master).
```

---

## 5. Do-not-do list

Breadth-treadmill items and unwinnable fronts that must **not** enter the backlog.

- **Do NOT add an Nth language mutation backend** (Ruby, Java, …) — four of the six
  existing backends are `passed:true` stubs and the Rust kill-path is broken;
  breadth on a foundation that cannot kill a mutant is negative-EV.
- **Do NOT add any new analyzer or query mode without a labeled fixture corpus in
  the same PR** — zero of 12 existing engines have ground-truth validation; a 13th
  unvalidated classifier just widens the fleet-poisoning surface.
- **Do NOT add another score command** — eight already exist (`score`, `repo-score`,
  `rust-project-score`, `popper-score`, `demo-score`, `brick-score`, `infra-score`,
  `perfection-score`) and none is a required check; consolidate and enforce one
  before minting more.
- **Do NOT re-implement aprender's ML/text/graph inside pmat** — the dependency
  direction converged in Sprints 45–46; fixing the 0.50/0.51 split is convergence
  work, forking back is regression.
- **Do NOT rebuild `pmat org analyze` by vendoring the removed OIP code wholesale** —
  the upstream API is gone at aprender HEAD; either delete the variant or rebuild
  thin on pmat-native services; waiting on upstream violates stop-the-line.
- **Do NOT trust or advertise `l3_kani_proved` / mutation-kill numbers until the
  harness/runner executes in CI** — publishing unexecuted proof counts is the exact
  theater the doctrine bans.
- **Do NOT chase the "95% coverage" number before the test corpus actually
  executes** — coverage of a suite where 189 src/tests files compile into nothing
  ratchets a fiction; sequence: run targets → revive corpus → then floor.
- **Do NOT expand AI-context scope (new formats, more MCP tools) before
  `context-output-fixture-gate` exists** — the production formatter currently has
  zero content gates.
- **Do NOT re-enable auto-release/auto-tag workflows** — deliberately `.disabled`;
  the maintainer's manual-publish doctrine is recorded in-repo; the clean-room item
  gates the manual path, it does not automate it.
- **Do NOT treat ROADMAP.md or `performance-optimization-trueno-depyler.yaml` as
  live planning inputs** — the former is explicitly frozen HISTORICAL at v2.192.0;
  the latter claims IN_PROGRESS untouched since January.
- **Do NOT "fix" the `pmat verify` complexity gate's changed-files scoping to
  full-repo** — permanently-red gates get bypassed (`--no-verify` culture); the
  grandfathering is a deliberate ratchet — extend it with a baseline, don't flip it
  to always-red. Same reasoning bans flipping `quality-gate --fail-on-violation` on
  today against the 197-violation baseline.

---

## 6. UNVERIFIED / needs-live-access appendix

| gap | exact closing artifact |
|---|---|
| Root cause of the comply dev-tree-vs-fresh-worktree split (gitignored `contracts/.pv` cache is the hypothesis, not proven) | `pmat comply check` in a fresh `git clone` with and without `contracts/.pv` copied in; diff the CB-1201 outcome; plus the CI log of quality-gate.yml's advisory comply step on master |
| Whether `cargo clippy`/rustc `dead_code` lint would catch the injected dead function (verify's clippy stage skipped per protocol) | `cargo clippy --lib --bins -- -D warnings` on the injected worktree (verify.rs:252-260 command) |
| Compile outcome of the arrow 57/59 split under `analytics-simd` (lockfile violation verified; build not run) | `cargo check -p pmat --features analytics-simd` (heavy build, off-site target dir) |
| Published `aprender-orchestrate 0.51.0` tarball contents (local monorepo at 0.59.0-dev lacks the OIP API) | `curl -L https://crates.io/api/v1/crates/aprender-orchestrate/0.51.0/download \| tar tz` and grep for analyzer/github/summarizer modules |
| Individual crates.io latest versions for aprender-{compute,rag,viz,zram-core,present-core,contracts-macros} (family assumed to track 0.51.0 from 5 curled members) | One crates.io API call each |
| Org-level clean-room membership list (lives outside this repo; only the `.disabled` pipeline and `machines/clean-room/deploy-workflows.sh` references found in-repo) | The `paiml/infra` deploy-workflows.sh and its repo list |
| Feature set of the 50.78 MB local release binary (whether a default-features build exceeds 50 MB) | `cargo build --release -p pmat` (default features) + `stat` on the produced binary |
| Durability of all sovereign-ci enforcement findings (ci.yml pins `@main`, a mutable ref; contents fetched 2026-07-05) | Re-fetch `paiml/.github/.github/workflows/sovereign-ci.yml@main` at review time, or land `sovereign-ci-hardening`'s SHA pin |
| Whether any private paiml infra roadmap holds ids referenced from pmat (none encountered — all 197 roadmap.yaml ids local or GH-resolvable) | Maintainer confirmation or the private roadmap itself |
| The `workspace-test` and `lake-build` contexts' provenance (sovereign-ci comments imply other paiml repos emit them) | `gh api` check-runs on a recent commit of the repos the org ruleset was designed for (e.g. aprender) |

---

## 7. Definition of done for this analysis

A reviewer takes any §4 backlog item and, from `id + definition_of_done + source`,
either opens the pmat ticket or rejects the item — with zero follow-up questions.
Every item carries its own falsifier with the exact mutation that must turn it RED,
cites HEAD artifacts verified this session, and is neither prose nor a summary. No
item is `UNDERSPECIFIED`. Findings not backed by a fetched artifact live in §6, not
in the roadmap.

*Generated by a Claude Fable 5 multi-agent audit, 2026-07-05/06, against
`origin/master` `f4ce4f980`.*
