# impl receipt — PMAT-1366 (the estimate ledger gets one gated, append-only writer)

| field | value |
|---|---|
| ticket | PMAT-1366 · kind=code (label added with `pmat work edit PMAT-1366 --tags kind:code`) · issue #1366 |
| branch | `PMAT-1366-estimate-ledger-unit`, cut from `origin/master` 441d198e7 |
| discover.json | sha256 prefix `30313d3502d93b9a`; `gate_cmd=cargo test --workspace` with `gate_cmd_fallback=true`, so the gate actually run was `pmat verify --format json` |
| discovery repair | this clone's `origin/HEAD` pointed at `origin/feat/roadmap-fragments`, and discovery refused (`no required status check on feat/roadmap-fragments`). GitHub's default branch is `master` (`gh repo view --json defaultBranchRef`). Fixed locally with `git remote set-head origin master`; nothing committed |
| verdict | DONE pending the required checks and the merge of this PR |

## RED — the defect as measured on 441d198e7

```
$ estimate.sh paiml-mcp-agent-toolkit 4
estimate: excluded L2 ticket=PMAT-631 phase=P1-P3 unit=- reason=no-unit
… (L5 L6 L7 L11 L12 L13 L14 L15 L16, all reason=no-unit)
estimate: ENV docs/audits/impl-estimates.jsonl: 16 measured rows for repo=paiml-mcp-agent-toolkit and none enters a total — the writer and the reader disagree
exit=2
$ estimate.sh pmat 4
K_HAT=35 BASIS=docs/audits/impl-estimates.jsonl:L21-L30 cycles=absent[U] ROWS=12 MEDIAN=35 EXCLUDED=0 UNMEASURED=2
$ pmat query --literal "impl-estimates" --files-with-matches     # indexed functions: none; only docs/ files
```

The new `--lib` test, committed before the ledger changed (aef89f37d), failed on the ledger itself:

```
the_committed_estimates_ledger_passes_the_write_time_gate panicked:
docs/audits/impl-estimates.jsonl carries 19 row(s) the writer would refuse:
L1 ticket=PMAT-631: no unit … L18 ticket=PMAT-689: no unit
ledger carries 2 repo keys (paiml-mcp-agent-toolkit,pmat): one repository's ledger has one key
```

## GREEN

```
$ estimate.sh paiml-mcp-agent-toolkit 4        # at 4c10236c5
K_HAT=35 BASIS=docs/audits/impl-estimates.jsonl:L21-L30 cycles=absent[U] ROWS=13 MEDIAN=35 EXCLUDED=9 UNMEASURED=8
$ pmat work estimate check                     # target/debug/pmat
./docs/audits/impl-estimates.jsonl: 30 rows, 13 poolable, 8 unmeasured (repo=all)
excluded L2 ticket=PMAT-631 reason=range-phase … excluded L16 ticket=PMAT-680 reason=unit-not-turn   (9 lines)
```

## Five whys

1. Why did `estimate.sh paiml-mcp-agent-toolkit` exit 2? All 10 measured rows under that key lacked `unit`, so every one was excluded as `no-unit`. The 14 rows written for this repository's later tickets were under a second key, `pmat`.
2. Why did those rows lack `unit`? L1–L18 were appended by hand between 2026-09-03 and 2026-09-08. The first row carrying `unit` is L19 (75ebdf009, 2026-09-09). The reader's hard unit rule (paiml-implement PMAT-066) merged on 2026-09-12.
3. Why did nothing refuse them, then or later? No code in this repository writes the ledger (the query above). The only schema check, `receipt-lint.sh --estimates`, lives in the skill repository and runs only when an orchestrator types it. No job in this repository reads the ledger.
4. Why was there no writer? SKILL.md Phase 4 specifies the producer as prose ("Append the ticket's row …"), and the reader sits in another repository. Producer and reader never meet inside one test.
5. Mechanism: a schema enforced only at read time fails as silence (an excluded row), never as an error at write time. Fix: one code writer that carries the schema and refuses at write time, plus a `cargo test --lib` over the committed file in this repository's CI.

## What changed

- `src/cli/handlers/work_estimate_ledger.rs` — `pmat work estimate record` and `check`.
  - `gate_new_row` refuses a missing or blank unit, `unknown`, any unit outside turn|session, a range phase, an `est` without `basis`, and a blank repo, ticket or mode.
  - `gate_repo_split` refuses a key that differs from the ledger's existing key.
  - `append_row` does O_APPEND with one `write_all` and returns the line number from the file offset. A refused row writes nothing: no file and no directory.
  - `check_ledger_text` reports `N rows, M poolable, U unmeasured` using estimate.sh's reason strings. Every row lacking a unit, and more than one key, is a violation.
- `.gitattributes` — `docs/audits/impl-estimates.jsonl merge=union`.
- `docs/audits/impl-estimates.jsonl` — the one sanctioned rewrite (table below).
- `contracts/estimate-ledger-v1.yaml`.
- `docs/status/unrun-tests-ledger.md` — regenerated (27 new lib tests).
- The record flag for the row's `mode` is `--exec-mode`. `--mode` is pmat's global cli|mcp flag, and a second arg with id `mode` panicked at argument access on every call. Unit tests missed this; running the built binary found it (see Jidoka).

## Backfill (L1–L18 unit; L17–L30 repo) — no other byte of any row changed

Each line was checked by script before commit: the new object equals the old object plus `unit` (and the new `repo`), with key order preserved and `unit` placed immediately after `phase`.

| L | ticket | phase | actual | before | after | basis |
|---|---|---|---|---|---|---|
| L1 | PMAT-631 | P1-P3 | null | unit absent | unit `turn` | receipt impl-PMAT-631-receipt.md Estimates header: `actual turns (this ticket)` |
| L2 | PMAT-631 | P1-P3 | 48 | unit absent | unit `turn` | row note: `actual counts every orchestrator turn` |
| L3 | PMAT-634 | P1-P3 | null | unit absent | unit `turn` | row note: `~30 orchestrator turns` |
| L4 | PMAT-637 | P1-P3 | null | unit absent | unit `turn` | row note: `~135 orchestrator turns`; receipt impl-PMAT-637-receipt.md Estimates: `~135 orchestrator turns` |
| L5 | PMAT-640 | P1-P4 | 63 | unit absent | unit `turn` | row basis: `transcript count of tool-use turns`; receipt impl-PMAT-640-receipt.md: `the transcript count above is the orchestrator's own turns` |
| L6 | PMAT-642 | P1-P4 | 101 | unit absent | unit `turn` | row basis: `transcript count of tool-use turns`; receipt impl-PMAT-642-receipt.md Estimates header `turns this invocation` |
| L7 | PMAT-648 | P1-P4 | 111 | unit absent | unit `turn` | row basis: `transcript count of tool-use turns`; receipt impl-PMAT-648-receipt.md Estimates header `turns this invocation` |
| L8 | PMAT-650 | spec | null | unit absent | unit `turn` | row basis: `turns not counted` |
| L9 | PMAT-651 | AD-01 | null | unit absent | unit `turn` | row basis: `turns not counted (same wipe)` |
| L10 | release-3.36.0 | cut+publish+dogfood | null | unit absent | unit `unknown` | no unit in the row; no impl-release-3.36.0-receipt.md exists (release-3.36.0-dogfood.md names none) |
| L11 | PMAT-673 | 1-4 | 45 | unit absent | unit `turn` | row basis: `actual ≈ orchestrator turns on this ticket` |
| L12 | PMAT-674 | 1-4 | 24 | unit absent | unit `turn` | row basis: `actual ≈ orchestrator turns on this ticket` |
| L13 | PMAT-685 | 1-3 | 31 | unit absent | unit `turn` | row basis: `actual ≈ orchestrator turns from the urgent interrupt` |
| L14 | PMAT-676 | 1 | 12 | unit absent | unit `turn` | row basis: `orchestrator turns on this ticket` |
| L15 | PMAT-679 | 1 | 6 | unit absent | unit `unknown` | row: `single dispatch, no resume` names no unit; receipt states only the worker's `37 tool uses`. est=34 derives from turn rows, but L2 shows est and actual in one row can differ in unit (`K̂=3 was per-phase-count; actual counts every orchestrator turn`), so est does not fix actual's unit |
| L16 | PMAT-680 | 1 | 6 | unit absent | unit `unknown` | as L15; receipt impl-PMAT-680-receipt.md states no orchestrator count |
| L17 | PMAT-707 | ph1-ph3 | null | unit absent, repo `pmat` | unit `unknown`, repo `paiml-mcp-agent-toolkit` | row: `actual far over K=4` names no unit; receipt Estimates `actual \| far over K`; re-key: impl-PMAT-707-receipt.md is in docs/audits/ |
| L18 | PMAT-689 | ph1-ph3 | null | unit absent, repo `pmat` | unit `unknown`, repo `paiml-mcp-agent-toolkit` | row: `k_measured 228 … is not this row's cost`; receipt gives no unit for this row's actual; re-key: impl-PMAT-689-receipt.md is in docs/audits/ |
| L19–L30 | PMAT-719 720 722 724 728 721 1253 723 727 1296 1299 1303 | all | 98 48 46 53 100 14 13 34 36 19 83 5 | repo `pmat` | repo `paiml-mcp-agent-toolkit` | re-key only: every one of these twelve tickets has impl-<ticket>-receipt.md in this repository's docs/audits/ |

Two scope decisions came from the plan-grill quorum, not from the brief. The re-key (D7) was 3/3 lanes. Backfilling L17–L18, which the brief's count of 16 missed, was raised by lane 2 and by the worker. L15/L16 went against the lane majority, 2 lanes `turn` to 1 `unknown`. Lane 1's cited evidence, "the worker consumed 37/45 turns", does not appear in impl-PMAT-679-receipt.md, which says `37 tool uses`. Lane 3's argument, that est and actual share a unit, is contradicted by L2 in the same ledger.

## Plan, routing, dispatch

routes:
  ph1  class=impl           route=self  w=100.00  basis=absent   note=route.sh printed route=agy-goal w=1.00; overridden to an opus worker (below) because four earlier agy-goal lanes in this ledger FAILED isolation (L20-L23 mode text), with the orchestrator applying the quorum changes directly
  ph1.delegate  class=plan  route=agy-plan w=1.00 basis=absent effort=1[U]   note=grillme width 3 on the plan and backfill table
  ph2  class=impl           route=self  w=100.00  basis=absent   note=backfill + re-key + .gitattributes, direct
  ph3  class=impl           route=self  w=100.00  basis=absent   note=contract, mutants, --exec-mode fix, ratchet fix, direct
  ph4.delegate  class=review  route=agy-quorum w=1.00 basis=absent effort=1[U]   note=pre-PR diff quorum width 3 (lanes ran --mode plan: agy-lane.sh refuses mode=quorum)
  ph4  class=orchestration  route=self  w=0.00  basis=absent

| description | mode | agent id | turns | maxTurns hit | resumed | notes |
|---|---|---|---|---|---|---|
| `PMAT-1366/ph1.impl worker B: estimate ledger writer + check` | subagent:opus (paiml-impl-worker) | `a02f798879e8206fe` | 18 tool uses | no | no | receipt partial=false; acceptance exit 101 with exactly the committed-ledger test RED, as briefed |
| `PMAT-1366/ph1.delegate grillme width 3 on the writer plan` | paiml-agy-delegate (opus) → agy 1.2.5 | `a7ba71a32306d9684` | 29 tool uses | no | no | conversations `ea2da4de-…`, `cfa00574-…`, `bcc8f7d9-…`; child_conversations 3; 3/3 do-not-implement-as-written; artifact docs/audits/quorum-PMAT-1366-plan.json |
| `PMAT-1366/ph4.delegate quorum width 3 on the PR diff` | paiml-agy-delegate (opus) → agy 1.2.5 | `a5194cd3c6a827349` | 30 tool uses | no | no | conversations `1fde0594-…`, `91ce8dff-…`, `bbd09182-…`; child_conversations 3; FAIL/PASS/PASS; artifact docs/audits/quorum-PMAT-1366.json |

Slots: peak 2 of 3 (worker + delegate in one message). `transcript-gate.sh`: `PASS attempted=3 denied=0 stalled=0 running_peak=2 slots=3`. Denials: 0.

## Quorum

- **Plan grill (ph1)**: 3/3 `do-not-implement-as-written` (gemini-3.1-pro-high, gemini-3.8-flash-high, gemini-3.7-flash-high; author claude-opus-5).
  - Adopted: re-key the `pmat` rows (3/3); backfill L17–L18 (lane 2, confirmed by the delegate at 441d198e7); `--repo` overrides origin, and a clone without origin must pass `--repo` (lanes 1, 2, 3).
  - Adopted, and moved: the key-split guard now reads the ledger's existing key instead of the origin, so it needs no remote.
  - Rejected with evidence: a repo-key cascade through the Cargo package name or the directory name (lanes 2, 3). The package name is `pmat`, the exact key that split this ledger, and SKILL.md says "never the directory name".
  - Rejected: `unknown` on L1/L3/L4 (lane 1). `unit` names the unit of `actual`, as L2's own note separates est from actual; a null actual is never pooled.
  - Kept: range phases refused at write time (lane 1 alone objected).
- **Pre-PR diff (ph4)**: lane 1 FAIL, lanes 2 and 3 PASS. Lane 1's only blocking finding is that no commit message carries a closing keyword; this PR's body line `Closes #1366` answers it, and its own proposed fix names the PR body. Non-blocking, not changed:
  - `check` accepts a blank-string ticket that `record` refuses (lane 2).
  - Lanes 2 and 3 contradict each other on how `classify_row` handles a missing unit. The row never reaches `classify_row`, because `check_line` tallies only violation-free rows and a missing unit is a violation.
  - Lane 1 wrote two files into its own sandboxed clone, which agy-lane.sh KEPT; the orchestrator read and deleted them (`check_diff.py`, a diff checker; `test_classifier.rs`, a hello-world). The shared checkout was never touched.

## Verification (claimed vs rerun)

verification:
  cmd=cargo-test-lib-estimate_ledger-RED(worker)  claimed_exit=101  rerun_exit=101(1-failed: the_committed_estimates_ledger_passes_the_write_time_gate, 19 violations)  log_path=.pmat/PMAT-1366/red-committed-ledger.txt  sha256=3d029903cf120040
  cmd=cargo-test-lib-work_estimate_ledger_tests-GREEN  claimed_exit=-  rerun_exit=0(27-passed)  log_path=.pmat/PMAT-1366/pv-eval.txt  sha256=4eb24588a6750255
  cmd=mutants-M1..M7  claimed_exit=-  rerun_exit=0(7-killed-0-survived)  log_path=.pmat/PMAT-1366/mutants.txt  sha256=4fa5883ee40f186a
  cmd=pmat-verify-at-42469eba8  claimed_exit=-  rerun_exit=1(tests: ratchet 20365/787/498 over 20325/785/497; unrun-tests ledger drift)  log_path=.pmat/PMAT-1366/verify.json  sha256=ef4aa362ff1f2eb1
  cmd=pmat-verify-at-46b58e876  claimed_exit=-  rerun_exit=0(format,satd,clippy,tests ok; complexity not_measured: clean tree vs HEAD; ok=null)  log_path=.pmat/PMAT-1366/verify2.json  sha256=519ff3206e2dd116
  cmd=ledger-before-backfill  claimed_exit=-  rerun_exit=-  log_path=.pmat/PMAT-1366/impl-estimates.before.jsonl  sha256=e8cd7bb91b0d76b7

The logs live under this clone's `.pmat/PMAT-1366/`, which is gitignored and survives a /tmp wipe. Complexity was measured by hand on the new file: `pmat analyze complexity --file src/cli/handlers/work_estimate_ledger.rs` reports 28 functions, max cyclomatic 5, max cognitive 6. The pre-commit hook's complexity check passed on every commit.

Dogfood on the built binary, against a copy of the ledger:
- A row without `--unit` was refused. Before b994265b7 it panicked instead (exit 101, `Mismatch between definition and access of mode`); after the fix it gets the gate's refusal.
- The ledger copy's sha256 was unchanged after every refused call.
- The old bytes stayed an exact prefix (`cmp -n`).

## Mutation (discrimination) — 7 planted, 7 killed, file restored byte-identical after each

| mutant | change | killed by |
|---|---|---|
| M1 | any non-empty unit admitted | estimate_ledger_an_unrecognised_unit_is_refused |
| M2 | missing unit admitted | estimate_ledger_a_row_without_unit_is_refused_and_creates_nothing (+2) |
| M3 | `.append(true)` → `.write(true).truncate(true)` | estimate_ledger_appends_never_truncate_or_reorder (+2) |
| M4 | key-split gate removed | estimate_ledger_a_row_that_would_split_the_key_is_refused_byte_identical |
| M5 | no newline before a record when the ledger ends mid-line | estimate_ledger_missing_trailing_newline_starts_a_new_line |
| M6 | check ignores a missing unit | estimate_ledger_check_flags_unit_less_rows_measured_or_not (+3) |
| M7 | `#[arg(long = "exec-mode", id = "mode")]` | estimate_ledger_record_parses_and_leaves_the_global_mode_flag_alone |

## pv contract

`contracts/estimate-ledger-v1.yaml`:
- `pv validate`: valid, 0 errors.
- `pv status`: 3 equations, 6 proof obligations, 15 falsification tests.
- `pv lint contracts --severity error`: PASS.
- `python3 scripts/pv-obligation-gate.py` (the provable ladder job): 0 problems over 36 contracts.
- Evaluation: every falsification test was run by exact name, and each proof obligation counted only if its `applies_to` fn exists and all its covering tests ran green. **21 obligations, 21 evaluated, 0 failed.**

The skill's `pv-gate.sh` reports 14 dangling citations here, all in pre-existing contracts (for example `quorum-review-v1.yaml` citing skill-repository paths) and none in this contract. It is the skill repository's gate, not this repository's.

## Reader side — filed, not edited

filed=paiml/paiml-implement#216 covers five asks:
- `receipt-lint.sh --estimates` refuses `unit: "unknown"` on L15/L16 (exit 1, bad=2); the brief requires that marker.
- The ENV message counts all rows for the key as "measured rows" (16, of which 6 had `actual: null`).
- The reader should print `N rows, M poolable`.
- SKILL.md Phase 4 still prescribes a hand-append; it should name `pmat work estimate record`.
- Key and range-phase text.

`estimate.sh` already pools by unit and names every exclusion, so it needed no change for the falsifier here.

## Corrections to the brief

1. The ledger had 30 rows, not only the 16 keyed `paiml-mcp-agent-toolkit`. 14 more rows for this repository's own tickets were keyed `pmat`, and `estimate.sh pmat 4` already printed `K_HAT=35` from them.
2. 18 rows lacked `unit`, not 16: L17 (PMAT-707) and L18 (PMAT-689) were keyed `pmat`.
3. Of the 16 rows keyed `paiml-mcp-agent-toolkit`, 10 were measured and 6 had `actual: null`. estimate.sh's "16 measured rows" message is itself a miscount (filed).
4. `estimate.sh paiml-mcp-agent-toolkit <N>` returned ENV exit 2, never `first-run[U]`.
5. The reader already pooled by unit and named exclusions (PMAT-066); the missing piece was the writer and a check in this repository.
6. PMAT-1365 had not merged (`Makefile` has no `gate:` target), so the check is a `--lib` test.
7. The clone's `origin/HEAD` pointed at `feat/roadmap-fragments`; see Identity.

## Jidoka

| defect | owner | whys | disposition |
|---|---|---|---|
| `record --mode` shadowed the global `--mode` and panicked at argument access | this PR (src/cli/commands/work_commands_work.rs) | unit tests exercised the domain functions, never clap parsing; the global arg is `global = true` with id `mode` | renamed `--exec-mode`; parse test on an 8 MiB thread; mutant M7 |
| `pmat verify` tests red at 42469eba8: ratchet +40 `.unwrap()`, +2 `panic!(`, +1 `#[allow(` | this PR | the ratchet counts test code too; the handler had 12 args | expect/assert in tests; handler takes `EstimateRow` + `EstimateRecordTarget`; baselines unchanged, not raised |
| unrun-tests ledger drift | this PR | 27 new lib tests change the rendered total | `pmat analyze unrun-tests --write-ledger` from a clean tree |
| mutant waiter never started | orchestrator | `pgrep -f "cargo build --bin pmat"` matched its own `sh -c` command line | rerun directly; no source was mutated while waiting |
| andon line 0.8K = 56 crossed with the phase gate not yet PASS, and not fired | orchestrator | K=70 came from K̂=35 on the `pmat`-keyed rows; this ticket ran 4 phases, 3 dispatches, 2 quorums and 3 verify runs | finding: continued under the brief's merge mandate; recorded here, not waved through |

## Estimates

| field | value |
|---|---|
| `K̂` | 35, `basis=docs/audits/impl-estimates.jsonl:L21-L30` (`estimate.sh pmat 4`: `ROWS=12 MEDIAN=35`, `cycles=absent[U]`). The repository's own key exited 2, which is the defect itself |
| `K` | 70 |
| actual | k_measured at the estimates row below (the whole session is this ticket) |

The row was written by `pmat work estimate record` (target/debug/pmat at this branch), the first row the gated writer produced.

## Gaps

| gap | artifact that closes it |
|---|---|
| `complexity` stage of `pmat verify` not_measured on a clean tree | hand measurement above; the pre-commit complexity check on each commit |
| lanes all gemini-family (independent of the author, not of each other) | recorded in both quorum artifacts |
| author model read from model-gate.sh `basis=transcript`, no `model-<sid>` file | recorded (author.source=flag) |
| reader/lint changes | paiml/paiml-implement#216 |
| installed `pmat` (3.40.2) has no `work estimate` until the release that carries this PR | release |

## Status

[status] ticket=PMAT-1366 phase=4/4 global=107/35(K=70) k_measured=107 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L21-L30
         mode=direct trigger=Q2 route=self w=0.00 q=? gate=PASS slots=0/3 denied=0 stalled=0
         red=- filed=paiml/paiml-implement#216 blocker=- next=push, PR, CI, merge

## Machine-readable

orch_model: opus-5 [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 25

IMPL-PMAT-1366-RECEIPT-END
