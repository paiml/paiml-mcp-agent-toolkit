# impl-PMAT-1365 — receipt (seventh session)

Verdict at this commit: **the gate finding is fixed on the branch; merge pending** quorum and CI. A commit cannot record its own merge, so the merge outcome is in the session's closing JSON receipt, not in this file.

The seventh session did three things:

1. It merged master `7fa1be27d` (#1387). That PR registered PMAT-1386, the row the sixth session's quorum would not let this PR carry. **This PR adds no lifecycle row.**
2. It fixed the stale-binary finding, RED first. The finding was wider than recorded: **ten legs** ran a hand-written `./target/debug/pmat`, not two.
3. It ran `make gate` on `e94b2c6cc` with `CARGO_TARGET_DIR=/mnt/nvme-raid0/targets/pmat-1365`: **27 PASS, 1 FAIL**. The one FAIL is `lib-tests`, on exactly the two known reds, #1305 and CB-200 (#1266). Neither is waived, and no baseline was raised.

## The merge (seventh session)

| check | result |
|---|---|
| merge | `git merge origin/master` `7fa1be27d` at `d27c9344e`, with no conflicts. Incoming: `docs/audits/impl-PMAT-1336-receipt.md`, `docs/audits/quorum-PMAT-1336.json` and `docs/roadmaps/roadmap.yaml` (the PMAT-1386 row). Behind = 0 |
| `git diff origin/master -- docs/roadmaps/roadmap.yaml` | one hunk, and it is this ticket's own row (`labels: [kind:code]`, `updated`). The carried-then-reverted PMAT-1386 commits (`9dc60d624`, `7fbce2a6d`) leave no residue |
| `git diff origin/master -- docs/audits/impl-estimates.jsonl` | +1 line, this ticket's row. `merge=union` duplicated nothing, so nothing was restored |
| `pmat analyze reachability --check-ledger` | exit 0 on `48737ee01` and on `e94b2c6cc` |
| `pmat analyze unrun-tests --executed '' --check-ledger` | exit 1 on `48737ee01`, which added one lib test. Re-rendered in `e94b2c6cc` (24297/27424 → 24298/27425), exit 0 after. It was not measured on the merge commit `d27c9344e` itself |

The binary used for both checks was built from this tree into the isolated target dir. The `cargo build --message-format json` output named `/mnt/nvme-raid0/targets/pmat-1365/debug/pmat`.

## The stale-binary finding: RED `48737ee01`, GREEN `e7812f9a9`

**Mechanism.** `make gate` runs the `build-pmat` leg (`cargo build --bin pmat --locked`), which writes to `$CARGO_TARGET_DIR/debug/pmat`. Legs that then run `./target/debug/pmat` ignore both `CARGO_TARGET_DIR` and `.cargo/config.toml`. So with an isolated target dir, they judged whatever binary an earlier build had left in `./target`. In this clone that was one built at 11:30 by an earlier session. Their PASS was evidence about some other tree.

The sixth session named two such legs and proposed `${CARGO_TARGET_DIR:-target}/debug/pmat` as the fix. **Both were wrong.** The two `cmd` legs were not alone: eight `step` legs run the same path, because they execute ci.yml's `run:` text verbatim. And the proposed spelling still ignores a `build.target-dir` set in `.cargo/config.toml`.

**The fix.** `scripts/gate.sh` sets `$PMAT_BIN` once per run to the executable that `cargo build --locked --bin pmat --message-format json` reports:

- `cmd` rows run `"$PMAT_BIN"`.
- In a step's `run:` text, CI's `./target/debug/pmat` is rewritten to `$PMAT_BIN` at run time. The workflow file is not edited.
- A leg that runs pmat when cargo reports none is a FAIL. It never falls back to `./target`.
- Each pmat leg's log names the binary it ran, and the verdict block prints it.

**RED, then GREEN, on the same controls:**

| control | on `48737ee01`'s gate.sh (RED) | on `e7812f9a9` (GREEN) |
|---|---|---|
| `scripts/gate-control.sh` arm 12 FRESH-BINARY. It plants a stale pmat that prints `STALE-PMAT-RAN` at `./target/debug/pmat`, and puts first on PATH a cargo that builds a pmat printing `FRESH-PMAT-RAN` in another dir. It then runs the REAL table's rows that run pmat | exit 1. Ten legs ran STALE and never FRESH: roadmap-validate-control, roadmap-validate, traceability-control, roadmap-coherence-control, work-sync-control, ticket-release-control, spec-epic-control, spec-review-control, cb-2113-cb-2115, pmat-score. With cargo reporting no pmat, nine did not FAIL and fell back to STALE | exit 0. All 12 legs ran FRESH (the ten above plus unrun-tests and reachability-ledger). With cargo reporting no pmat, every non-`cargo run` pmat leg is FAIL and none ran STALE |
| `make_gate_tests::no_leg_runs_a_hand_written_pmat_binary_path` (nextest, one test binary, gate.sh swapped under it, since the test reads the file at run time) | FAIL: `cmd legs run a hand-written pmat path … ["cb-2113-cb-2115", "pmat-score"]` | PASS, and the other 5 `make_gate_tests` pass too |

The contract `contracts/make-gate-v1.yaml` gains `pmat_legs_run_the_built_binary`, with its obligation and falsification test. `pv validate` → 0 errors.

**Every leg checked for the pattern.** `pmat query --literal "target/debug/pmat"` returned raw-file hits only in `src/` comments and `docs/`, and its output was truncated at 40 lines. So `grep` was used for the non-Rust files: `scripts/*.sh`, `Makefile` and the workflows the step legs read.

| leg / site | runs | verdict |
|---|---|---|
| `cb-2113-cb-2115`, `pmat-score` (cmd) | `./target/debug/pmat` | **defect, fixed** → `"$PMAT_BIN"` |
| `roadmap-validate-control`, `roadmap-validate`, `traceability-control`, `roadmap-coherence-control`, `work-sync-control`, `ticket-release-control`, `spec-epic-control`, `spec-review-control` (step, ci.yml) | `./target/debug/pmat` in the step text | **defect, fixed** → rewritten to `$PMAT_BIN` at run time. The six control scripts take the binary as `$1` and hard-code no path |
| `unrun-tests`, `reachability-ledger` (cmd) | `cargo run --locked --quiet --bin pmat` | correct already. cargo runs the binary it built |
| `build-pmat` (step) | `cargo build --bin pmat --locked` | builds only and runs no pmat. Correct |
| `gate-control` | fixture tables only. Arm 12 uses a fixture cargo | n/a |
| `dependabot-alerts-live`, `dependabot-self-test`, `orphan-ledger`, `pr-lane-control`, `tests-dont-write-self-test`, `reusable-pin-drift`, `fmt`, `clippy-all-targets`, `lib-tests`, `cargo-deny`, `cargo-audit`, `lean-build`, `lean-no-holes`, `pv-obligations` | no pmat binary | n/a |
| `scripts/*.sh` with a `target/debug` or `target/release` hit: capture_golden_traces, dogfood-use, reduce-complexity, profile_context, record-metric, implement-dependency-reduction, run-full-qa, qa-retest, setup-quality, validate-timeout-feature | — | none is reached by a `make gate` leg. **Not fixed here** |
| `Makefile` targets `pre-release-checks` (L1602), `dev` (L2538), `sprint-close` (L2617), `quality-gate-full` (L2655) | `./target/debug/pmat` | the same pattern, but these are not legs of `make gate`. **Not fixed here**; named so nobody assumes otherwise |

**What earlier receipts claimed.** The sixth session's `make gate` on `9dc60d624` ran all ten legs above against the stale binary, not only `cb-2113-cb-2115`. Earlier sessions' runs, where `./target/debug/pmat` was not rebuilt by the same build, carry the same doubt. The run below is the first in which those legs judged this tree's binary under an isolated target dir.

## Verification — seventh session, re-run by the orchestrator

`make gate` on `e94b2c6cc` (HEAD=e94b2c6cc origin/master=7fa1be27d behind=0), `CARGO_TARGET_DIR=/mnt/nvme-raid0/targets/pmat-1365`:

| leg | result | leg | result |
|---|---|---|---|
| gate-control | PASS 4s (arms 1–12; arm 11 LIVE matched 6 contexts) | cb-2113-cb-2115 | PASS 9s. CB-2113 ✓ 27 non-merge commits; CB-2115 ✓ 115/115 |
| fmt | PASS 3s | orphan-ledger | PASS 0s |
| clippy-all-targets | PASS 122s | dependabot-self-test | PASS 0s |
| **lib-tests** | **FAIL 147s**: 21734/21736. `dead_code_outcome_tests::a_crate_that_does_not_compile_is_reported_as_not_measured` (#1305; sibling PR #1388) and `tdg_baseline::tests::the_committed_baseline_is_the_measured_count` (CB-200: 1741 vs 1688, #1266) | dependabot-alerts-live | PASS 1s |
| cargo-deny | PASS 1s | unrun-tests | PASS 38s |
| cargo-audit | PASS 2s | reachability-ledger | PASS 25s |
| reusable-pin-drift | PASS 0s | pmat-score | PASS 58s |
| build-pmat | PASS 31s | lean-build | PASS 3s |
| roadmap-validate-control, roadmap-validate, traceability-control, roadmap-coherence-control, work-sync-control, ticket-release-control, spec-epic-control, spec-review-control | PASS (0–3s each) | lean-no-holes | PASS 0s |
| pr-lane-control, tests-dont-write-self-test | PASS 0s | pv-obligations | PASS 1s |

- Every pmat leg's log opens with `gate.sh: pmat = /mnt/nvme-raid0/targets/pmat-1365/debug/pmat (reported by cargo build --message-format json)`.
- The verdict block prints: `every leg that ran pmat ran /mnt/nvme-raid0/targets/pmat-1365/debug/pmat … a step's ./target/debug/pmat was rewritten to it`.
- The 16 CI-only rows were printed by name.
- Verdict: RED on `lib-tests` only, and this PR does not cause it. The PR's job is to declare the gate, and #1305 and #1266 are owned elsewhere.

---

# Sixth session (history)

Verdict: **PARTIAL(blocker)**. `make gate` is declared, and discovery finds it.
The branch is up to date with master `7c2aa59b8` (#1383, which registered PMAT-1385 and completed PMAT-1363).

The fifth session left one blocker: the required `traceability` job (CB-2115) was red on master's own state. #1383 fixed those two findings. By the time it merged, master was red on CB-2115 again, this time with `ORPHAN-GITHUB #1386`. That issue was opened at 10:50:15Z by a sibling session and has no roadmap row.

This session carried that row (`9dc60d624`) and explained it in the next section. The quorum round on `ea3edddc2` then went **NOT AGREED**: lane 1 (gemini-3.1-pro-high) FAILed the row as out of scope, and lanes 2 and 3 PASSed. The artifact is committed (`91f1b5a9b`). Under the session's rule, **the row was dropped** (`7fbce2a6d`, a revert) and the scope is not argued a third time. So `traceability` stays red on master's state until #1386 gets its row **on master**.

**Next step, exactly:** once master carries a PMAT-1386 row (or #1386 is closed and CB-2115 is green on master), run `git merge origin/master`. Re-render the two ledgers only if `--check-ledger` fails. Then run `quorum-review.sh --base master --ticket PMAT-1365 --pr 1368 --author-model claude-opus-5`, because this receipt changed after the last agreed round. Commit the artifact, push, wait for CI, and run `bash ~/.claude/skills/quorum-review/pmat-merge 1368 --auto --merge`. Before that, fix the gate.sh finding below, or record why not.

## Lifecycle row PMAT-1386: carried in `9dc60d624`, FAILed on scope, DROPPED in `7fbce2a6d`

| field | value |
|---|---|
| outcome | quorum on `ea3edddc2`: lane 1 FAIL ("adds an unrelated roadmap item (`PMAT-1386`)"), lanes 2–3 PASS → NOT AGREED; the row is reverted by `7fbce2a6d`, and the net diff no longer touches the PMAT-1386 row |
| commit | `9dc60d624` `chore(PMAT-1365): register PMAT-1386 — issue #1386 is open with no roadmap row`. It changes one file, `docs/roadmaps/roadmap.yaml`, and adds 16 lines: one row and nothing else |
| writer | `pmat work add --github-issue 1386 "<issue title>"`, a sanctioned writer. #1383 wrote PMAT-1385 with the same writer (`475abd957`). The row was not typed by hand |
| label | none, because issue #1386 has no labels (`gh issue view 1386 --json labels` → `[]`) |
| before | on the merged tree `4de1b69ca`, `pmat comply check --checks CB-2113,CB-2115` gave CB-2113 ✓ (19 commits) and CB-2115 ✗, with 1 finding: `ORPHAN-GITHUB #1386` |
| after | on `9dc60d624`, the same command gave CB-2113 ✓ (19 commits) and CB-2115 ✓: 115 open items and 115 open issues in bijection |
| the issue | `gh issue view 1386` → OPEN, created 2026-09-17T10:50:15Z. It stays open: this row registers the issue and does not fix it |

**Why the row is in this PR and not in its own.** CB-2115 is a bijection between open issues and open roadmap items. It is checked against master's state, so an issue anyone opens reds `traceability` on every open PR until its row lands. A separate lifecycle PR for this one row costs a full CI cycle of about 40 minutes. That already happened twice today. #1384 was opened for PMAT-1366, then closed after #1364 fixed the row itself. #1383 fixed PMAT-1363 and PMAT-1385, and #1386 arrived before #1383 merged. The next orphan arrives before a lifecycle PR merges, so this PR could not go green by waiting for one.

**Precedent.** PR #1364 carried PMAT-1366's lifecycle row for exactly this reason, and its quorum ruled SCOPE ACCEPT 3/3. This ticket's own earlier scope FAIL, in the fourth session over the #1381 row, came from a round that ran without this receipt's explanation. Master now carries that row itself (#1382), so it is not in this diff.

**What a reviewer can check.** The commit touches no file other than `docs/roadmaps/roadmap.yaml`. It adds no status change to any other row, and it does not close or relabel any issue. `git show 9dc60d624 --stat` shows this.

## Identity

| field | value |
|---|---|
| ticket | PMAT-1365 (`kind:code`, #1365) |
| branch | `PMAT-1365-declare-gate-land`, pushed as `PMAT-1365-declare-gate` (PR #1368) |
| base | rebased onto `origin/master` `e89a827f7` (#1382); then `git merge origin/master` `8915fe3e6` (#1364) at `896a9bdff`; then (sixth session) `git merge origin/master` `7c2aa59b8` (#1383) at `4de1b69ca`, a clean merge. Merges, not rebases, because master's protection is strict; behind = 0 |
| model-gate | `opus-5`, class opus, admit, basis=transcript |
| discover.json on the rebased head `bfe4e7acd` | `gate_cmd=make gate`, `gate_cmd_fallback=false`, sha256 `4a969e3bf1a4ceab0d234a7ee1e2c565e4be95bc4c4ca5fdb68837ba945b2ad9` — the GREEN |
| discover on master `441d198e7` (fourth session, clean clone) | `gate_cmd=cargo test --workspace`, `gate_cmd_fallback=true` — the RED |
| required contexts (branch protection) | `ci / gate`, `feature-gate`, `docs build (docs.rs environment)`, `pmat score`, `provable ladder`; the org ruleset adds `gate` |

## The rebase (fifth session)

| step | what happened | how it was resolved |
|---|---|---|
| `48cd21d03` (the #1381 row) | conflicted with master's PMAT-1381 row from #1382; the bytes differ (master's row has acceptance_criteria and `labels: []`) | master's bytes taken; the commit became empty and was dropped. **The #1381 row is no longer in this diff.** |
| four ledger re-render commits | conflicted on the ledger summary lines | master's bytes taken, then both ledgers re-rendered once on the rebased tree, in two commits |
| receipt commit, `impl-estimates.jsonl` | `.gitattributes` gives the ledger `merge=union` (PMAT-1366), so the rebase re-added 31 rows that #1382 had rewritten | master's bytes restored; this ticket's row re-appended with `pmat work estimate record` |

Re-render evidence: `pmat analyze reachability --check-ledger` → 0; `pmat analyze unrun-tests --executed '' --check-ledger` → 0. After merging `8915fe3e6`, both ledgers were taken from master and re-rendered again (4489→4490 tracked; 24292/27419→24297/27424).

## The merge (sixth session)

| check | result on `4de1b69ca` |
|---|---|
| merge conflicts | none. Only `docs/audits/impl-PMAT-1336-receipt.md`, `docs/audits/quorum-PMAT-1336.json` and `docs/roadmaps/roadmap.yaml` came in |
| `pmat analyze reachability --check-ledger` | exit 0, `ledger is current: docs/status/orphan-files-ledger.md`, so no re-render |
| `pmat analyze unrun-tests --executed '' --check-ledger` | exit 0, `ledger is current: docs/status/unrun-tests-ledger.md`, so no re-render |
| `docs/audits/impl-estimates.jsonl` vs master | `git diff origin/master` shows +1 line, this ticket's row. `merge=union` duplicated nothing this time, so nothing was restored |
| code since the judged head `f6f0aea1d` | `git diff --stat f6f0aea1d HEAD` touches only `docs/audits/*`, `docs/roadmaps/roadmap.yaml` and this receipt. `src/`, `scripts/`, `Makefile`, `contracts/` and `.config/` are byte-identical |

## How a sibling gate adds a row (D0 issue-closure contract, D2 roadmap-write query gate)

Append **one line** to the `legs_table` heredoc in `scripts/gate.sh`, below the line `# ── EXTENSION POINT ──…`, before `LEGS`. Make no other edit: the runner, the CI-ONLY printout and the verdict pick the row up.

```
kind    | contexts | leg | source | note | command
cmd     | gate | issue-closure | ci.yml <job> "<step>" | - | <command run from the repo root>
step    | gate | roadmap-write-query | .github/workflows/<file>.yml#<job id>#<step name> | - | -
ci-only | gate | <leg> | <where CI runs it> | <platform|credential|cost|trigger|not-gating|not-required>: <reason> | -
```

- `step` runs a workflow step's `run:` text, read at run time. Use it when the step is plain bash with no `${{ }}`, `if:` or `env:`.
- `cmd` runs the command you give it. `<note>` says how that differs from CI.
- `ci-only` runs nothing, but its reason is printed on every exit. A credential reason that is really a GitHub token `gh` already holds is refused (`src/make_gate_tests.rs`).
- `<contexts>` is `;`-separated and must name a required context.
- Keep the marker line. `scripts/gate-control.sh` and `src/make_gate_tests.rs` assert that it exists.
- Check the table with `bash scripts/gate.sh --list`. It runs nothing.

## Verification — re-run by the orchestrator

| claim | re-run |
|---|---|
| `make gate` on the merged head `be34454e7` | 648 s, **27 PASS, 1 FAIL**: `cb-2113-cb-2115` (CB-2113 ✓ 17 commits; CB-2115 ✗ `ORPHAN-ROADMAP PMAT-1363`, `ORPHAN-GITHUB #1385`). `lib-tests` PASS, CB-200 included. The CI-only rows are printed by name. |
| CB-2115 red is master's | master `8915fe3e6` carries `PMAT-1363 status: planned`; `gh issue view 1363` → CLOSED 09:51:56Z; `gh issue view 1385` → OPEN 09:57:46Z; CI `traceability` on `be34454e7` fails |
| discovery finds the gate | `discover.sh` → `gate_cmd=make gate`, `gate_cmd_fallback=false` |
| `make gate` on the rebased head `bfe4e7acd` | 605 s, **26 PASS, 2 FAIL**: `lib-tests` (21690/21691 — only `tdg_baseline::tests::the_committed_baseline_is_the_measured_count`, CB-200 #1266) and `cb-2113-cb-2115` (CB-2113 ✓ 13 commits; CB-2115 ✗ `ORPHAN-ROADMAP PMAT-1366: #1366 is closed`). The CI-only rows are printed by name. |
| CB-2115 PMAT-1366 red was master's | master e89a827f7 carries `PMAT-1366 status: planned`; `gh issue view 1366` → CLOSED 2026-09-17T08:55:11Z. On #1384's tree: `pmat comply check --checks CB-2115` → ✓ 114/114. Master `8915fe3e6` (#1364) then completed the row itself, so #1384 was closed as redundant (quorum 3/3 PASS on `347d281b1` stays in its branch history) |
| CI on `bfe4e7acd` (first read) | 22 pass, 1 fail (`traceability`, the same CB-2115 finding), 19 pending |
| the CB-200 red is master's (fourth session) | clean clone of `441d198e7`: 1742 below A against a baseline of 1688, the same as the branch |
| contract | `contracts/make-gate-v1.yaml`: `pv-obligations` leg PASS inside `make gate` |
| controls | `gate-control` leg PASS (`scripts/gate-control.sh`) |

## Scope

- **#1381 row: gone from this diff.** It was the fourth session's one blocking quorum FAIL. Master now carries it (#1382).
- **Foreign roadmap rows: none in the net diff.** PMAT-1386 was carried in `9dc60d624`, FAILed on scope by quorum lane 1, and reverted in `7fbce2a6d`. The fourth session's rule still stands: a foreign row that draws a scope FAIL comes out and reaches master on its own. The sixth session's rule is stricter. If a lane FAILs this row on scope, the commit is dropped, the ticket stops at PARTIAL(blocker), and the scope is not argued a third time.
- Fifth session: #1384 was opened for PMAT-1366, and master fixed that row itself in #1364, so #1384 was closed. PMAT-1363 and #1385 were fixed by #1383.
- Still in this diff: `PMAT-1365`'s own row gains `kind:code` (the label `kind-gate.sh` reads).

## Jidoka

| defect | owner | five whys → mechanism | disposition |
|---|---|---|---|
| feature-gate red on 1d8c64f40 (orphan-files ledger drift) | scripts/gate.sh | a new .rs file moved the ledger header count → no lib test covers that ledger → gate.sh marked the leg CI-only → the "cost" reason was never measured | fixed (fourth session): RED `d52bd0f82`, GREEN `01a67b512` |
| CB-2115 ORPHAN-GITHUB #1381 | roadmap lifecycle | an issue opened after master's last green has no row → CB-2115 is a bijection over open issues → every PR goes red | fixed on master by #1382; this branch's copy was dropped in the rebase |
| dependabot-alerts-live false credential reason | scripts/gate.sh | a CI-only reason is accepted as asserted, and `gh` already holds the token | fixed (fourth session): RED `d30834a1e`, GREEN `5161561f3` |
| CB-2115 ORPHAN-ROADMAP PMAT-1366 | roadmap lifecycle (PMAT-1336) | a merged PR closes its issue → a ticket cannot complete its own row in the PR that closes it → the row stays `planned` → CB-2115 reds every PR. The fifth instance of this gap in two days | fixed on master by #1364 (`df6c351b2`); #1384 closed as redundant |
| CB-2115 ORPHAN-ROADMAP PMAT-1363, ORPHAN-GITHUB #1385 | roadmap lifecycle (PMAT-1336) | the same gap, plus an issue opened after master's last change → every PR red again, minutes after the previous fix | fixed on master by #1383 (`7c2aa59b8`) |
| CB-2115 ORPHAN-GITHUB #1386 | roadmap lifecycle (PMAT-1336) | a sibling session filed an issue at 10:50Z, before #1383 merged → no lifecycle PR can land before the next orphan → every open PR stays red | **not fixed here**. It was carried in `9dc60d624` and dropped in `7fbce2a6d` after a scope FAIL. Master has to land the row; the lifecycle gap stays open under PMAT-1336 |
| `make gate` runs a stale pmat under `CARGO_TARGET_DIR` | scripts/gate.sh (this ticket) | the `cb-2113-cb-2115` and `pmat-score` rows call `./target/debug/pmat` → `build-pmat` writes to `$CARGO_TARGET_DIR/debug/pmat` → with an isolated target dir, the legs run whatever binary is left in `./target` (here, one built at 09:30Z by the fifth session) → no control arm sets `CARGO_TARGET_DIR` | **fixed (seventh session)**: RED `48737ee01`, GREEN `e7812f9a9`. Ten legs, not two. The proposed `${CARGO_TARGET_DIR:-target}` spelling was also wrong, because it ignores `.cargo/config.toml`; `$PMAT_BIN` is what cargo reports |
| lib-tests red locally: `a_crate_that_does_not_compile_is_reported_as_not_measured` | #1305 (a sibling session is root-causing it) | failed in 0.261 s under nextest on `9dc60d624`; 21734/21735 passed | **not fixed, not re-run, not waived**: #1305. Red again in the seventh session's `make gate` on `e94b2c6cc` |
| lib-tests red locally: CB-200, 1742 vs 1688 | src/services/tdg_baseline.rs (#1266) | debt below grade A was added on master → the lib test measures only where `.pmat/context.db` exists → CI checkouts have no index, so `ci / gate` passes it unmeasured | **not fixed, not waived**: #1266. Passed in `make gate` on the merged head `be34454e7`; not investigated why. Red in the seventh session: 1741 vs 1688 |

## Dispatch ledger

Seventh session: no Claude subagents, and no agy delegate. `make gate` ran once, on `e94b2c6cc`, and is reported above. The pre-merge review is `quorum-review.sh` (three agy lanes) on the head that carries this receipt; its artifact is committed on top.

Sixth session: no Claude subagents. One `quorum-review.sh` round ran (three agy lanes) on `ea3edddc2`: NOT AGREED, lane 1 FAIL on scope, artifact `91f1b5a9b`. No second round was run, because the row it FAILed was dropped and the session stopped. `make gate` ran on `9dc60d624` in `/mnt/nvme-raid0/targets/pmat-1365`, 869 s: **27 PASS, 1 FAIL** (`lib-tests`: #1305's test, above). `cb-2113-cb-2115` PASSed, but it measured with the stale `./target/debug/pmat` (see Jidoka).

Fifth session: no subagents were dispatched. transcript-gate.sh: `PASS attempted=0 denied=0 stalled=0 running_peak=0 slots=3`, which is vacuous. The pre-merge review is `quorum-review.sh` (three agy lanes), run on this commit. Its artifact is `docs/audits/quorum-PMAT-1365.json`, committed on top.

## Estimates

- `estimate.sh paiml-mcp-agent-toolkit 5` → `K_HAT=35 BASIS=docs/audits/impl-estimates.jsonl:L22-L31`, 14 pooled rows.
- The fourth session declared K̂=60 as `first-run[U]`. That was wrong: estimate.sh exited 2.
- `k_measured`: seventh session 37 when this receipt was written; fourth session 104; fifth session 37 at the first receipt commit and 64 at this one. Sessions 1–3 were not measured.
- The row is therefore recorded with `unit: session` and is never pooled.

## Gaps

- Routing R-4 was not followed in any session: the implementation phases were done directly.
- The sixth-session blocker is gone: master carries PMAT-1386 (#1387). Merge state is recorded in the session's closing receipt.
- The Makefile targets `pre-release-checks`, `dev`, `sprint-close` and `quality-gate-full`, and ten non-gate scripts, still hand-write a target path. They are not legs of `make gate` and are not fixed here.
- `lib-tests` is red locally on #1305 and CB-200 (#1266). This session did not investigate either one.
- The fourth session never timed the `cost:` CI-only rows one by one. The fifth did not either.
