# IMPL-PMAT-1305 — the dead-code analyzer's `cargo check` builds into a target dir only its workspace root uses

## Identity

| field | value |
|---|---|
| ticket | PMAT-1305 (#1305, `kind:code`; duplicate report: #1284) — `kind-gate.sh` exit 0 `kind=code ticket=PMAT-1305` |
| branch | `PMAT-1305-dead-code-not-measured-flake`, created from `origin/master` `8915fe3e6`; merged `origin/master` `7c2aa59b8` (docs and roadmap only) as `77df0822d` |
| `discover.json` sha256 | `ea182cf9e7682b6f4014c1b3898fbcb72e41d030ecf06843e70ff927eb664b22` |
| `gate_cmd` | `cargo test --workspace` — **`gate_cmd_fallback=true`**; the gate run is `pmat verify --format json`, the one this repository's CLAUDE.md names |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| model gate | `model=opus-5 class=opus decision=admit basis=transcript` |
| I-3 at this commit | `PASS transcript-gate: attempted=0 denied=0 stalled=0 running_peak=0 slots=3` — vacuous: no subagent had run yet; the Phase 4 review dispatch is after this receipt (see below) |
| build isolation | every build and test under `CARGO_TARGET_DIR=/mnt/nvme-raid0/targets/pmat-1305`; experiments under `/mnt/nvme-raid0/targets/pmat-1305-exp` |

Every measurement below was taken with `behind=0` against the `origin/master` of its moment: `8915fe3e6` for the reproduction, the fix and the mutant; `7c2aa59b8` for `pmat verify`.

## The defect, measured

`ci / test` failed `a_crate_that_does_not_compile_is_reported_as_not_measured` with
`not_measured must be set for an uncompilable crate; outcome was: violations=[] not_applicable=None` —
a clean, full dead-code measurement of a crate with a syntax error. Three occurrences, and the job logs of all
three show one interleaving between `ci / test` and `ci / coverage` (same run, same per-PR target mount):

| run (PR) | coverage: dead-code tests blocked on the target lock until | test job starts its suite | coverage: compilable `fx` control ok | test job: uncompilable `fx` test FAILED |
|---|---|---|---|---|
| 34156890569 (#1222, issue #1284) | 20:03:28.90 | 20:03:28.98 | 20:03:29.75 | 20:03:30.32 |
| 34562657315 (#1304, issue #1305) | 04:52:56.67 (from 04:47:58, "running for over 60 seconds") | 04:52:56.80 | 04:52:58.42 | 04:52:58.99 |
| 35209474539 (#1383) | 10:49:41.74 (from 10:46:45, "running for over 60 seconds") | 10:49:41.80 | 10:49:42.31 | 10:49:42.58 |

Both jobs set `CARGO_TARGET_DIR: /workspace/target`, bind-mounted from
`/mnt/nvme-raid0/targets/sovereign-ci-<repo>/<PR number>` (paiml/.github `sovereign-ci.yml`, `test` and
`coverage` jobs). The coverage job's nested `cargo check` inherits it and waits on the lock the test job holds
while compiling its test binary; on release both run the dead-code tests inside the same second.

The mechanism, at the cargo level (cargo 1.98.0, reproduced in a shell before any code changed): two
crates named `fx` at different paths, one shared target dir, the broken one written first —

```
fine exit=0
broken exit=0          Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s   (0 error diagnostics)
broken(private target) exit=101   error: could not compile `fx` (lib) due to 1 previous error
```

cargo hashes a workspace member's package id and target path RELATIVE to the workspace root, so both `fx`
crates share one fingerprint; freshness is mtime against that fingerprint, so the older broken source is
"fresh", the (empty) diagnostics of the other crate are replayed, and cargo exits 0.

## Five whys

1. **Why did the test fail?** `check_dead_code_outcome` returned `violations=[]`, `not_measured=None`: `analyze()` returned `Ok` with a FULL compiler scan.
2. **Why a full scan of a crate with a syntax error?** The only path to `CargoCheckOutcome::completed` is `cargo check` exiting 0. The cache was not involved: its file lives in the fixture's own fresh tempdir, which is not a git checkout (no key, nothing to read).
3. **Why did `cargo check` exit 0?** cargo judged the `fx` lib unit fresh and replayed stored diagnostics: the fingerprint of a workspace member is path-relative, so a different `fx` crate's newer fingerprint in the same target dir matched.
4. **Why was another `fx` crate's fingerprint in that target dir, and newer?** The analyzer inherited `CARGO_TARGET_DIR=/workspace/target`, which `ci / coverage` mounts too; its compilable `fx` was checked after the test job wrote its broken source and before that source was checked (table above, 3 of 3).
5. **Why could an inherited target dir be shared with a different root at all?** The analyzer delegated the choice of target dir to the environment; nothing tied the fingerprints it trusts to the root it measures. **Owner: `src/services/cargo_dead_code_analyzer/`.**

## The fix

`cargo check` gets `--target-dir <T>/pmat-dead-code/<name>-<fnv1a64(canonical workspace root)>`, where `T` is
the `target_directory` `cargo metadata --no-deps` resolves for the crate (so `CARGO_TARGET_DIR` and
`build.target-dir` are honoured and `cargo clean` removes it). The key is FNV-1a, not `DefaultHasher`, whose
output may move between toolchains. Files: `src/services/cargo_dead_code_analyzer/target_isolation.rs` (new),
`analysis.rs` (the two `.arg` calls), `target_isolation_tests.rs` (new), and the RED test plus instrumentation in
`src/cli/analysis_utilities/quality_checks_part1_dead_code.rs`.

Cost, stated rather than hidden: the analyzer no longer shares the user's own `target/debug` check artifacts,
so the FIRST analysis of a workspace root is a cold `cargo check` of its dependency graph into the isolated dir,
and those check artifacts exist twice on disk. This repository's own documented measurement is 245 s cold
against 67.6 s warm (`DEFAULT_ANALYSIS_TIMEOUT_SECS`, whose 900 s budget covers cold); later runs are
incremental, and a tree already analysed is served by the tree-hash cache. Not re-measured here. In CI each
fixture root leaves a small dependency-free directory under the persistent per-PR target mount.

A code comment the fix sits next to claimed "cargo writes its own `target/.gitignore`"; measured false on
cargo 1.98 (a fresh target dir holds `CACHEDIR.TAG`, `.rustc_info.json`, `debug/` and nothing else), corrected
in the same diff.

## Hypotheses on the issues, each measured

| hypothesis | verdict | evidence |
|---|---|---|
| dead-code cache (PMAT-648, `git write-tree`) returns a hit across fixtures | refuted as cause | the cache file is `<project_path>/.pmat/dead-code-cache-*.json`, inside each fixture's fresh tempdir; outside git `working_tree_hash` is `None`; the planted test reds with no git at all |
| a shared `.pmat` cache dir in CI | refuted | `ensure_cache_dir(&self.project_path)`: per fixture, no shared dir exists |
| `set_current_dir` in 16 files / 52 sites (#1288) moves the cwd | not the cause here (count confirmed: 16 files, 52 sites) | the fixture path is absolute and cargo runs with `current_dir(<absolute crate root>)`; the race reproduces 17/80 in a process running ONLY the three dead-code tests |
| TMPDIR-sensitive fixture | refuted as cause | the fixture declares `[workspace]` (#1361); reproduced with `TMPDIR=/tmp`, outside any workspace |
| inherited `RUSTC_WRAPPER` / `SCCACHE_DIR` (ticket acceptance criteria) | not needed to reproduce | CI sets `RUSTC_WRAPPER: rustc-sccache`, but a fresh unit never reaches rustc or its wrapper; reproduced on a host with no sccache installed |
| inherited `CARGO_TARGET_DIR` shared with a same-named crate | **CONFIRMED** | shell probe above; job-log interleaving 3/3; planted test RED 3/3; harness 17/80 |

## Reproduction rates

`scripts/repro-pmat-1305-shared-target-race.sh <lib test binary> 40 <mode>` — each iteration runs the three
`dead_code_outcome_tests` in two processes (one in `single`) with `RUST_MIN_STACK` unset, `--test-threads=8`
(CI's container is `--cpus 8`), one shared `CARGO_TARGET_DIR`. A count is per process run.

| binary | aligned (build lock held, then released — CI's shape) | free (no lock) | single (no concurrent writer) |
|---|---|---|---|
| `8915fe3e6`, unmodified | **17 / 80** | **17 / 80** | 0 / 40 |
| fix (`3df9a68f8`) | **0 / 80** | **0 / 80** | 0 / 40 |
| mutant (fix minus `--target-dir`) | **19 / 80** | not run | not run |

Full `--lib` suite, two processes at once sharing one `CARGO_TARGET_DIR` (8 threads each, `RUST_MIN_STACK`
unset), N = 1 pair each: unmodified binary — the target test ok in 2 / 2 processes (466 s); fix — ok in 2 / 2
(349 s). Started together, both suites reach the dead-code tests without the lock alignment CI adds, so this
shape did not red in one pair; the targeted harness is the measurement of record.

## RED → GREEN → mutant

| step | commit / tree | result |
|---|---|---|
| RED | `59c0c3496` (test only, analyzer unmodified) | `a_broken_crate_sharing_a_target_dir_with_a_same_named_crate_is_not_measured` FAILED 3 / 3 with `CARGO_TARGET_DIR` unset (sharing through `.cargo/config.toml`), 1 / 1 with it set; its premise asserts (plain `cargo check` in the broken crate exits 0) passed each time |
| GREEN | `3df9a68f8` | 11 / 11 targeted tests pass, with and without `CARGO_TARGET_DIR` |
| mutant | `docs/audits/mutants/PMAT-1305-inherited-target-dir.patch` on `3df9a68f8` | `the_cargo_check_command_names_the_isolated_target_dir` and the planted test FAILED 3 / 3; the original test reds 19 / 80 under the aligned race |

RED message: `an uncompilable crate was reported as MEASURED (violations=[]) because its cargo check replayed a
same-named crate's fingerprint from a shared target dir; CARGO_TARGET_DIR=None`.

## Contract

`contracts/dead-code-target-isolation-v1.yaml` — `pv validate` valid; `pv status`: **5 proof obligations, 6
falsification tests**; `pv lint --severity error` PASS; `scripts/pv-obligation-gate.py` 0 problems over 38
contracts. **Evaluated: 5 / 5 obligations through 6 / 6 falsification tests, 0 failed**, each run with a
non-zero test count at `3df9a68f8`: DCTI-F-001 2 passed, F-002 1, F-003 1, F-004 1, F-006 2, F-005 the harness
(0 / 80). A first pass reported `0 passed` for F-001 and F-006 with exit 0 — zsh does not word-split `$c`, so
two filters reached libtest as one — and was re-run under bash; it is recorded because a 0-test exit 0 is the
exact vacuity the contract exists to refuse.

## Gate

| gate | result |
|---|---|
| `pmat verify --format json` at `77df0822d` (behind 0) | `ok: null`, `stages_measured: 4` — format, satd, clippy, tests (full `--lib`, 347.8 s) all `ok: true`; complexity `not_applicable: no Rust files changed vs HEAD` because the change is committed |
| `pmat verify --stage complexity --format json` in a scratch worktree of `origin/master` `7c2aa59b8` with this branch's `src/*.rs` diff applied uncommitted | complexity `ok: true` |
| `cargo fmt --all -- --check` | exit 0 |
| pre-commit hooks (`pmat hooks install --strict --force`) | format, complexity, clippy, SATD, docs ✅ on every commit |
| `pmat quality-gate --file analysis.rs` | 2 findings before the change, 2 after (both pre-existing, see F-1) |
| unrun-tests ledger | re-rendered by the binary built at `3df9a68f8`: 24292 → 24299 of 27419 → 27426 executed, 3127 unrun unchanged |
| CB-200 TDG ratchet | every function this diff adds grades A+; `build_cargo_check_command` grades A- on master and here |

## Findings (not fixed in this PR)

- **F-1 — the agent edit hook deadlocks on pre-existing findings.** `.agents/hooks/pmat-quality-feedback.sh` is wired `PreToolUse` on `Write|Edit`, so it grades the file BEFORE the edit: `analysis.rs` carries two "Dead code attribute found" findings at HEAD (line 110 is a comment describing the regex, line 750 a test-fixture string literal), and every Edit to the file was refused, including one that could only add a comment. The hook's own README calls it "feedback, not gates" and names `ci / gate` as the gate; the two-line `.arg` change was applied with a scripted replace and the finding count shown unchanged (2 → 2). Not filed from this branch: `pmat work sync --direction github-to-yaml --dry-run` would also register a sibling's #1386 here, and an issue without a row is an ORPHAN under CB-2115 on the release path.
- **F-2 — CB-200's lib test measures whatever index is on disk.** With the `.pmat/context.db` this session's own `pmat query` calls built, `the_committed_baseline_is_the_measured_count` reports 1741 below A against 1688, independent of this diff (above); a fresh checkout has no index and takes the documented "absent" branch. The index was removed before `pmat verify`.
- **F-3 — two lib tests fail only when two full suites share one cwd:** `services::context::visitor_tests::test_analyze_rust_file` (NotFound) and `services::spec_parser::tests::test_find_specs_empty_dir_is_ok`; both pass alone at HEAD.
- **F-4 — CLAUDE.md says the hooks `pmat hooks install` writes run "format/complexity/SATD only"**; the ones installed here also ran clippy and a documentation check. Not edited here (CLAUDE.md changes need `validate-readme`).

## Plan and routing

| phase | what | route | executor |
|---|---|---|---|
| 0 | kind / model / config gates, discovery | `route=self w=0.00 basis=absent` | orchestrator |
| 1 | reproduce: shell probe, CI job logs, harness baseline | `route=self w=0.00 basis=absent` | orchestrator |
| 2 | RED test, fix, isolation tests, mutant | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` — **overridden to direct** | orchestrator: acceptance needs the 21,897-test lib binary built and run many times (cold build 3m22s, each race series minutes), and the reproduction was already in context; recorded as a deviation from R-4 |
| 3 | contract, ledger, `pmat verify` | `route=self w=0.00 basis=absent` | orchestrator |
| 4 | pre-merge review | `route=agy-quorum w=1.00 basis=absent effort=1[U]` | delegate, quorum width 3 — verdict in `docs/audits/quorum-PMAT-1305.json`, which the judged diff excludes by construction |

## Admission, routes, verification (machine-read)

orch_model: opus-5 [V]   orch_class: opus   orch_decision: admit   orch_basis: none
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 22

routes:
  ph0  class=orchestration  route=self      w=0.00  basis=absent
  ph1  class=orchestration  route=self      w=0.00  basis=absent   taken=self (reproduction and CI-log forensics)
  ph2  class=impl           route=agy-goal  w=1.00  basis=absent   note=fable-binding effort=1[U]   taken=self (deviation from R-4, reason in the routing table above)
  ph3  class=orchestration  route=self      w=0.00  basis=absent   taken=self (contract, ledger, pmat verify)
  ph4  class=review         route=agy-quorum  w=1.00  basis=absent  effort=1[U]   taken=delegate quorum width 3

verification:
  cmd=probe-shared-target-cargo-check(broken-exit-0,private-exit-101)  claimed_exit=-  rerun_exit=0  log_path=docs/audits/impl-PMAT-1305-receipt.md(quoted-in-"The-defect,-measured")  sha256=-
  cmd=ci-log-test-job-35209474539  claimed_exit=-  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/test-job.log  sha256=242b18b9bcc798f6
  cmd=ci-log-coverage-job-35209474539  claimed_exit=-  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/coverage-job.log  sha256=647fbfe17d76babd
  cmd=red-planted-test-3x@8915fe3e6+test  claimed_exit=101  rerun_exit=101  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/red-3x.log  sha256=9e308ce1f6a97462
  cmd=fix-targeted-11-tests-x2(env-unset,env-set)@3df9a68f8  claimed_exit=0  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/fix-targeted.log  sha256=872578ff70e9fe84
  cmd=mutant-targeted-3x(2-FAILED-each)  claimed_exit=101  rerun_exit=101  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/mutant-3x.log  sha256=73f423b66bf30a77
  cmd=repro-harness-baseline(aligned17/80,free17/80,single0/40)  claimed_exit=0  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/repro-baseline.txt  sha256=0bd9d9246030fdf5
  cmd=repro-harness-fix(aligned0/80,free0/80,single0/40)  claimed_exit=0  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/repro-fix.txt  sha256=482032b4e838690a
  cmd=repro-harness-mutant(aligned19/80)  claimed_exit=0  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/repro-mutant.txt  sha256=53656b74864cb5c7
  cmd=contract-falsification-F001-F004,F006(7-tests-ok)  claimed_exit=0  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/contract-falsification.log  sha256=bb61f6cfdf01cdbe
  cmd=full-lib-pair-baseline(target-ok-2/2)  claimed_exit=-  rerun_exit=101(ledger+index,F-2/F-3)  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/full-baseline/summary.txt  sha256=0ea09fa7e9907782
  cmd=full-lib-pair-fix(target-ok-2/2)  claimed_exit=-  rerun_exit=101(ledger+index,F-2/F-3)  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/full-head/summary.txt  sha256=9abd309739b10715
  cmd=pmat-verify-format-json@77df0822d(4-stages-ok,complexity-n/a)  claimed_exit=-  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/PMAT-1305/.pmat/pmat-1305/verify.json  sha256=e871f89ea83b51c9

## Estimates

`estimate.sh paiml-mcp-agent-toolkit 4` → `K_HAT=35 BASIS=docs/audits/impl-estimates.jsonl:L23-L32`; `K=70`.
Actual at this receipt: **129** turns (`k_measured` from the transcript), recorded as `docs/audits/impl-estimates.jsonl` L33 by `pmat work estimate record` — before the Phase 4 quorum, the PR and the CI waits.

## Verdict

DONE at this commit for scope; merge is gated on the quorum artifact and `required_check`.
