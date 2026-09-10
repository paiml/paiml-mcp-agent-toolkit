# IMPL-PMAT-719 — goal-mode step 2: CB-2113 commit traceability, its control, and the `traceability` job

## Identity

| field | value |
|---|---|
| ticket | PMAT-719 (`kind:code`, `orch:fable`, `kind-gate.sh` exit 0, files=27 against `origin/master`) |
| spec | `docs/specifications/goal-mode.md` §7 (CB-2113 row), §7.1 (job), §8.4 (PR-only until merge-only), §11 step 2, §13 |
| branch | `PMAT-719-cb2113-traceability-gate` |
| base | `master` @ `20fda3ca6` (merge-base) — **stacked on** `PMAT-718-comply-checks-selector` @ `943fda5cd` (PR #1246, open, CLEAN, every check green at receipt time); P1 `--checks` is this job's selector |
| HEAD in | `943fda5cd` |
| HEAD out | `91c3a86e2` (last code/ledger commit; the commit carrying this receipt follows it) |
| PR | opened from this branch after the receipt commit — `gh pr list --head PMAT-719-cb2113-traceability-gate` |
| diff, this ticket only (`943fda5cd..HEAD`) | 20 files: 20 files changed, 1980 insertions(+), 31 deletions(-) |
| `discover.json` sha256 | `b515d3f9b4c08e5ed9674023693708edb3ae704fb26f4fa639b8228914827b4b` — re-derived after the host reboot (the pre-reboot state dir was tmpfs and did not survive); `rule=pid-file claude_pid=21623` |
| `gate_cmd` | `cargo test --workspace` — **`gate_cmd_fallback=true`**, no repo-declared gate was found; `pmat verify` is the gate this repository's own CLAUDE.md names, and it is what was run |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| `quorum_tool` / `contracts_dir` / `code_search` | `agy` / `contracts` / `pmat query` |
| model gate | `model=fable class=fable decision=admit basis=file` (`orch:fable` label + `notes: 'orch-basis:M>=3 …'` on the roadmap entry) |

### Status-line join `[U]→[V]`

| claim | measured | how |
|---|---|---|
| statusLine `session_id` = hook `session_id` | **true** | `discover.sh --state-dir` resolved `39a15c63-…` by `rule=pid-file`; the hook's own `events-39a15c63-….jsonl` exists under the same id |
| `tasks[].id` = hook `agent_id` | **[U]** after the reboot | the three `agentId`s in the transcript (`a77340cd924fa8b7e`, `aab2695e6c80ded4d`, `a535fcf49f8ede164`) were logged by the hook to a tmpfs file the reboot erased; the surviving log holds only post-reboot rows, so the pairing cannot be re-measured |
| `transcript_path` present on subagentStatusLine stdin | **[U]** | not measured this session |
| `k_measured` vs `global=k` | `k_measured_at_set=92` (`goal.sh show` after re-declaration); final pair in the last status block below | `jq -r 'select(.type=="assistant" and ((.isSidechain // false)\|not)) \| (.message.id // .uuid)' <transcript> \| sort -u \| wc -l` |

## What step 2 is

CB-2113 judges every non-merge commit in `merge-base(base, HEAD)..HEAD` on a non-default
branch: each must carry a `Pmat-Ticket:` git trailer naming a roadmap item that is not
Completed/Cancelled. On the default branch it counts the commits since the latest `v*` tag and
does not judge them (Pass with a `not_applicable:` reason, §8.4 — merges are not restricted to
merge commits yet). A git, base-resolution or roadmap-parse failure is Fail `not_measured:`;
a roadmap that was never committed is Skip; a roadmap that was committed and is now gone is
Fail `not_measured:` (deleting a gate's input is not a way of passing it — the CB-2102
precedent, quorum finding 1).

The `traceability` job runs `scripts/traceability-control.sh` **before** the check, so the
gate is proven able to fail on every arm — untrailered commit, unknown id, terminal item — and
able to pass — trailed commit, default branch — before it is trusted on the real tree. The job
sits in `gate`'s `needs` **and** its result loop, and `merge_group` is a trigger.

Because the job selects one rule with `--checks CB-2113`, the enforcement ledger (CB-2100) had
to learn per-rule attribution: a `--checks` subset used to read as "the invocation restricts
which rules run", neutering every rule. Now an invocation carries a `selected` set; a rule is
ENFORCED when a reachable invocation covers it and nothing else suppresses it; the ledger names
the direct `run` step over a control hop that runs the same rule; and an invocation that points
comply at another tree (`--path`/`-p` to anything but `.`/`$PWD`/`$GITHUB_WORKSPACE`) is a
suppression, so the control's fixture run can never be credited as the carrier.

## Plan and routing

| phase | what | class | `route.sh` (verbatim) | trigger |
|---|---|---|---|---|
| 1 | RED falsification suite, then CB-2113 engine (`src/services/commit_traceability/`), check (`check_traceability.rs`), builder, severity default, README count 157→158 | impl, single module | `route=agy-goal w=1.00 basis=quota.json@44h` | — (Q2 teamwork grill of the spec section **NotRun**, see Gaps) |
| 2 | gate_effect per-rule attribution: `Invocation.selected`, `covers_rule`/`enforces_rule`/`carries_selected_rules`, `reachable_invocations()`, per-rule `unreachable_rules`, `rule_status` | impl | `route=agy-goal w=1.00 basis=quota.json@44h` — **not taken**: R-4 allows one `writes=true` agy lane per repository and ph1 held it; fell back to `subagent:sonnet` (`paiml-impl-worker`) in its own worktree, named here | — |
| 3 | `.github/workflows/ci.yml` (`traceability` job, `gate` needs + loop, `merge_group`), `scripts/traceability-control.sh`, `docs/status/*` ledgers | orchestration (workflows are on the worker's forbidden list) | `route=self w=11.11 basis=quota.json@44h` | — |
| 4 | pre-PR quorum on the diff, adjudication, fixes F1–F3 with mutants, DoD gates, receipt, push, PR | review, then orchestration | `route=agy-quorum w=1.00 basis=quota.json@46h` (quorum); `route=self w=11.11 basis=quota.json@46h` (the rest, re-run after the reboot) | Phase 4 pre-PR review, width 3 |

## Dispatch ledger

| phase | mode | description | agent id | turns | maxTurns hit | resumed | lane / width / agy `conversations` |
|---|---|---|---|---|---|---|---|
| 1 | delegate (`paiml-agy-delegate`, opus) → agy `goal`, `writes=true`, in throwaway worktree `lane-2467f92b5-18145` | `PMAT-719/ph1.delegate goal width 1 on CB-2113 engine+check` | `a77340cd924fa8b7e` | — | no | no | goal / 1 / `af57a148-4c0a-474a-af97-c1bcb04f704e` — agy's transport was interrupted before the receipt; the lane had already committed `84e583020` in its worktree, verified and cherry-picked as `c3a3985c7` |
| 2 | subagent:sonnet (`paiml-impl-worker`) in worktree `.claude/worktrees/PMAT-719-ph2` (branch `PMAT-719-ph2-gate-effect`) | `PMAT-719/ph2 worker B: --checks per-rule attribution in gate_effect` | `aab2695e6c80ded4d` | 40 | **yes** | no (not resumed; escalated to self) | — |
| 4 | delegate (`paiml-agy-delegate`, opus) → agy `quorum`, `writes=false`, `--sandbox` | `PMAT-719/ph4.delegate quorum width 3 on the step-2 diff` | `a535fcf49f8ede164` | — | no | no | quorum / 3 / `d6d656a1-49b2-466c-9aef-d8fa73789540`, `84e68a51-4d54-491f-bf1b-fa56b62d7603`, `ef3e3ac5-d6a4-4f3f-9454-57b7d25f1960`; interrupted first attempts `6b9183b7-effe-4814-a0c5-3a56d54677c6`, `9d8684dc-a433-4bf2-bd7a-44c0b9f70c14` |

Phase 3 and everything after the quorum was direct. The worker's diff was re-run by me (86
gate_effect tests green in its worktree), the two parse tests it had not written were written by
me, committed in its worktree as `4374e9e07` and cherry-picked as `90023eece`. Both worktrees
were removed after `git cherry` confirmed every commit applied.

## Slots, denials, I-3

| field | value |
|---|---|
| `slots` | 3 (`config.json`, `config-lint.sh`: `slots=3 gh_calls_per_min=30 bank=3`) |
| Claude subagents dispatched | **3** (two delegates, one worker); the first delegate and the worker ran in the same message on disjoint `scope_paths` |
| `transcript-gate.sh` | `PASS transcript-gate: attempted=3 denied=0 running_peak=2 slots=3 segments=83 files=3 (agent_calls=3 resumes=0 workflow_started=0; denied from hook log)` |
| denials | `denied=0` — measured on the hook log that survived the reboot (post-reboot rows only); no denial was observed before it either |
| Workflow tool | not used |

## Quorum (Phase 4, `docs/audits/quorum-PMAT-719.json`)

3/3 **FAIL** on `487946d5f`, four distinct findings after dedup. Every one was re-measured
before it was believed:

| # | finding | lanes | adjudication | closed by |
|---|---|---|---|---|
| 1 | `Inputs::NoRoadmap` → Skip lets deleting `docs/roadmaps/roadmap.yaml` bypass the gate | 3 | **CONFIRMED** | `Inputs::RoadmapDeleted` (committed-then-gone, via `metrics_ratchet::history::was_ever_committed`) → Fail `not_measured:`; 2 tests |
| 2 | `rule_status` falls back to `enforcing.first()`, so with the direct step deleted the control's fixture run (`--path "$repo"`) would be credited | 3 | **CONFIRMED** | `foreign_tree()`: `--path`/`-p` to anything but here is a suppression; 2 tests |
| 3 | `current branch name == base short name` misclassifies a local branch literally named `master` ahead of `origin/master` | 2 | **REFUTED** | a local branch named `master` *is* the default branch locally, and in CI HEAD is detached so the name test never fires; documented |
| 4 | mutant `enforces_rule == covers_rule` survives | 1 | **CONFIRMED** | `enforces_rule_needs_more_than_coverage` |

## Mutation table (a control needs its own mutant)

| mutant | planted as | killed by | log |
|---|---|---|---|
| F1: deletion reads as `NoRoadmap` | `Ok(true) => Inputs::NoRoadmap` | `a_deleted_roadmap_is_not_measured_and_fails`, `a_roadmap_that_was_committed_and_is_now_gone_is_not_an_absence` — 0 passed, 2 failed | `mut-f1.log` 2dc2f46a735c19ae |
| F2: `foreign_tree` always `None` | `if !line.is_empty() { return None; }` as its first line | `an_invocation_that_points_comply_at_another_tree_is_not_evidence_for_this_one`, `the_ledger_names_the_direct_step_over_a_control_hop_that_runs_the_same_rule` — 0 passed, 2 failed | `mut-f2.log` 4341cfe5eef27a5a |
| F3: `enforces_rule` = `covers_rule` | `self.covers_rule(id)` alone | `enforces_rule_needs_more_than_coverage` — 0 passed, 1 failed | `mut-f3.log` f729bdecd5e0d829 |
| original suite RED | commit `2467f92b5` — the 19 engine tests, 6 check tests and 4 attribution tests were committed before the engine existed and did not compile/pass until `c3a3985c7`/`90023eece` | the RED commit itself | `git show 2467f92b5 --stat` |
| control script arms 1, 3, 4 | an untrailered commit, `PMAT-999`, a completed `PMAT-002` planted in a throwaway repo | exit 1 + `Fail` + the named commit/id on each arm; arms 2 and 5 prove the rule can also pass | `control.log` `1f0f891d501de483` |

Every mutant was applied one at a time to the real source, the named tests run, and the source
restored to a byte-identical copy (`diff -q`) before the next. The first attempt at F2 was
killed by the host reboot with the mutant applied and the backups on tmpfs; the mutant line was
removed by hand, the 118-test baseline re-established green, and F2/F3 re-run.

## Verification table (claimed vs my rerun)

| command | claimed | rerun | at |
|---|---|---|---|
| `env -u RUST_MIN_STACK cargo test --lib -- commit_traceability gate_effect tests_traceability` | lane: pass; worker: 86 pass | **118 passed, 0 failed** | `91c3a86e2` |
| `bash scripts/traceability-control.sh ./target/debug/pmat` | — | exit 0, 5 arms, 0.7 s | `91c3a86e2` |
| `pmat comply ledger --write` | — | 158 rules; `CB-2113 … ENFORCED … ci.yml:traceability step 'the closed loop holds (CB-2113)' (run; selected by --checks CB-2113)`; `gate` root; tree unchanged after write | `91c3a86e2` |
| `pmat comply check --checks CB-2113,CB-1703` on this repository | — | CB-2113 Pass (7 commits in `20fda3c..HEAD`, base `origin/master`); CB-1703 158==158; nohup, 920 s, 36.5 GB RSS | `487946d5f` — **not re-run** at `91c3a86e2` (cost); proxy below |
| `git log --no-merges --format='%h %(trailers:key=Pmat-Ticket,valueonly)' origin/master..HEAD` | — | 13/13 non-merge commits trailered (10 `PMAT-719`, 3 `PMAT-718`), both ids open | `91c3a86e2` |
| `cargo test --lib -- the_committed_ratchet_holds_at_head` | red at `487946d5f` (unwrap 9186 > 9177) | **ok** after `.expect()`; baseline untouched | `91c3a86e2` |
| `pmat analyze unrun-tests --check-ledger` → `--write-ledger` | — | drifted (+4 tests) → re-rendered, `23897 of 27126` | `91c3a86e2` |
| `transcript-gate.sh` / `kind-gate.sh` / `model-gate.sh` | — | PASS / `kind=code files=27` / `decision=admit` | `91c3a86e2` |
| `pmat verify --format json` | — | **ok:true**, stages_measured=5, not_measured=[] (format 2.6 s · complexity · satd 1.8 s · clippy 1.5 s · tests 367 s; 373 s total, nohup on the committed tree) | `91c3a86e2` |

`./target/debug/pmat` is the executable cargo itself reported (`--message-format json`); the
off-site `target_directory` in `cargo metadata` held a binary from 17:49 without the new
strings. The binary used was fingerprinted for two strings this diff added before any run.

### Discrimination

The control's five arms pin the verdict to CB-2113 by reading its own JSON row, and a report
with no CB-2113 row is exit 2, never a pass. The ledger names the **direct** step as the carrier
and, after F2, cannot name the control's fixture run even if the direct step is deleted. The
two `--checks` attribution paths (a subset run enforces the rules it names; a subset run with
a real suppression enforces nothing) each have a test whose mutant kills them.

## Jidoka log

| defect | owner | disposition |
|---|---|---|
| pre-commit format check refused the RED commit | this row | `cargo fmt`, `2467f92b5` |
| `pmat work add -d` put the `orch-basis:` token into `acceptance_criteria`; `model-gate.sh` refused | this row | moved to `notes:`, real acceptance criteria written; admitted |
| ph2 worker hit `maxTurns=40` without committing | this row | diff verified (86 pass), 2 missing parse tests written by me, committed in its worktree, cherry-picked |
| ledger credited the control's fixture hop as CB-2113's carrier | this row | `rule_status` prefers `via=="run"` (`5613266d3`); quorum #2 showed the fallback still unsafe → `foreign_tree` suppression (`0ee5719e1`) |
| full-repo `comply check` exceeded the 10-minute tool cap (exit 143) | this row | re-run under `nohup`; never two at once |
| unwrap ratchet 9186 > 9177: nine `.unwrap()` in an `include!`'d test file gated by `#[cfg(all(test, not(coverage_nightly)))]`, which the metric's `#[cfg(test)]` stop-line does not recognise | this row | `.expect()` (`85c93e4b8`); baseline untouched |
| bashrs SEC011 on `rm -rf "$work"` | this row | `${work:?}` |
| quorum 3/3 FAIL | this row | 3 confirmed and fixed with mutants, 1 refuted (`0ee5719e1`) |
| host reboot at 18:26 mid-Phase-4 killed the mutation run with mutant F2 applied; scratchpad backups lost | environment | mutant line removed by hand; 118 green re-established; F2/F3 re-run one at a time with fresh backups |
| stale binary at the off-site target dir | environment | executable path taken from cargo, binary fingerprinted |
| `impl-estimates.jsonl` rows L1/L3/L4 carried prose in `actual`, so `receipt-lint.sh --estimates` was red before this row | prior rows | `actual` → `null`, the prose kept verbatim in `note` (PMAT-015) |
| `pmat verify` tests stage red at `487946d5f` (`the_committed_ratchet_holds_at_head`) | this row | fixed at the cause (`85c93e4b8`); at `91c3a86e2` all 5 stages are green — the TMPDIR-class tests-stage failures seen on earlier branches did not recur on this run |

## Estimates

| field | value |
|---|---|
| `K̂` | 4, `basis=first-run[U]` (`estimate.sh pmat 4`, `ROWS=0` — K̂ = N with no qualifying rows) |
| `K` | 8 |
| actual | 98 (`k_measured`, this ticket only — one ticket per session — including the reboot recovery and the quorum-fix round); K was exceeded during Phase 0, which was said in the first status block and continued deliberately, as the two previous goal-mode rows did |
| `pr_runs` / `mg_runs` | not yet observable: the PR is opened after this receipt |

## Gaps

| gap | artifact that closes it |
|---|---|
| Phase 1 `teamwork` grill of the spec section (Q2) **NotRun** — the pre-PR quorum reviewed the diff instead | a `/teamwork-preview` receipt on §7.1 / §11 step 2 |
| `pv` contract **NotRun** — `contracts_dir=contracts` is set; the spec names no contract for step 2 | `contracts/comply-traceability-v1.yaml`, bound by a test the way `tests_cb2100.rs` binds `comply-gate-effect-v1.yaml` |
| merged green on `required_check` — pending; this branch is stacked on #1246, which must merge first | the PR's `ci / gate` run, after #1246 |
| §13 item 2 (RED in CI at least once, linked) — the `traceability` job has not run in CI yet | the first CI run of the job on this PR (its control arms 1/3/4 are the RED observation) |
| full-repo CB-2113 self-check not re-run at `91c3a86e2` (920 s / 36.5 GB) | the `traceability` job on the PR, or one more nohup run |
| pre-reboot hook event log lost; `denied=` and the `tasks[].id` join are measured on the surviving log only | — |
| §13 items 4–5 (`#612` collisions, `work sync --check-only`) belong to later steps | steps 3+ of §11 |
| 15 untracked scratch files in the checkout (`all_cids.txt`, `patch.diff`, `empty_test_project/`, …) are not this row's and were left untouched | their owner |

## Machine-readable

orch_model: fable [V]   orch_class: fable   orch_decision: admit   orch_basis: M>=3
fable_binding: true   quota_age_h: 46   quota_mark: A   k_measured_at_set: 92

routes:
  ph1  class=impl           route=agy-goal    w=1.00   basis=quota.json@44h
  ph2  class=impl           route=agy-goal    w=1.00   basis=quota.json@44h   taken=sonnet-worker (R-4 one-writer fallback, named)
  ph3  class=orchestration  route=self        w=11.11  basis=quota.json@44h
  ph4  class=review         route=agy-quorum  w=1.00   basis=quota.json@46h
  ph4  class=orchestration  route=self        w=11.11  basis=quota.json@46h

verification:
  cmd=cargo-test-lib-118(commit_traceability+gate_effect+tests_traceability)  claimed_exit=0  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/tests-118.log  sha256=ff06bb098a6e9dd6
  cmd=traceability-control-5-arms                    claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/control.log  sha256=1f0f891d501de483
  cmd=comply-ledger-write                            claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ledger-write.log  sha256=6486b49f42703a87
  cmd=comply-ledger-CB-2113-row                      claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ledger-cb2113-row.log  sha256=636bf26094fd75c2
  cmd=comply-check-CB-2113-CB-1703-self@487946d5f    claimed_exit=-  rerun_exit=0  log_path=.pmat/pmat719-resume/comply-self.log  sha256=1549dc53eead5304
  cmd=branch-trailers-proxy@91c3a86e2                claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/branch-trailers.log  sha256=3fbc8c635f71c50d
  cmd=the_committed_ratchet_holds_at_head            claimed_exit=1  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ratchet.log  sha256=b0d8cfa35d658466
  cmd=analyze-unrun-tests-write-ledger               claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/unrun-write.log  sha256=2162bef474272463
  cmd=transcript-gate                                claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate.log  sha256=780db1cfb2fc0258
  cmd=kind-gate                                      claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/kind-gate.log  sha256=94df8b55c0583b7d
  cmd=model-gate                                     claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/model-gate.log  sha256=f2146d74e864f015
  cmd=mutant-F1-deletion-reads-as-NoRoadmap          claimed_exit=-  rerun_exit=101  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut-f1.log  sha256=2dc2f46a735c19ae
  cmd=mutant-F2-foreign_tree-always-None             claimed_exit=-  rerun_exit=101  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut-f2.log  sha256=4341cfe5eef27a5a
  cmd=mutant-F3-enforces_rule-eq-covers_rule         claimed_exit=-  rerun_exit=101  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut-f3.log  sha256=f729bdecd5e0d829
  cmd=pmat-verify@91c3a86e2                          claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify.json  sha256=ae7fdc123e45ac14

`claimed_exit=-` marks a row with no second party: my own execution, recorded once. The two
rows with a claim (the 118 tests, the ratchet test) name who claimed what. `log_path` points at
session-scoped scratch that will not outlive this session; the `sha256` prefix pins the bytes
that were read, and the durable copy of the same legs is the CI record on the PR.

## Status blocks

[status] ticket=PMAT-719 phase=1/4 global=94/4(K=8) k_measured=94 sub=0/0 basis=first-run[U]
         mode=quorum:goal trigger=R-4-single-module-width-1 route=agy-goal w=1.00 basis=quota.json@44h q=stale gate=NOT-RUN slots=1/3 denied=0
         red=- filed=- blocker=- next=CB-2113 engine+check via the agy goal lane; K already exceeded in Phase 0 (gate_cmd_fallback=true)
[status] ticket=PMAT-719 phase=2/4 global=94/4(K=8) k_measured=94 sub=40/40 basis=first-run[U]
         mode=subagent:sonnet trigger=- route=agy-goal w=1.00 basis=quota.json@44h q=stale gate=PASS slots=2/3 denied=0
         red=- filed=- blocker=- next=gate_effect per-rule attribution (worker B, own worktree; R-4 fallback from agy-goal)
[status] ticket=PMAT-719 phase=3/4 global=94/4(K=8) k_measured=94 sub=0/0 basis=first-run[U]
         mode=direct trigger=- route=self w=11.11 basis=quota.json@44h q=stale gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=ci.yml traceability job + control + merge_group + ledgers
[status] ticket=PMAT-719 phase=4/4 global=94/4(K=8) k_measured=94 sub=0/0 basis=first-run[U]
         mode=quorum:quorum trigger=Phase-4-pre-PR-review route=agy-quorum w=1.00 basis=quota.json@46h q=stale gate=FAIL slots=1/3 denied=0
         red=quorum-3/3-FAIL filed=- blocker=- next=adjudicate 4 findings, fix F1/F2/F3 with mutants, refute #3
[status] ticket=PMAT-719 phase=4/4 global=98/4(K=8) k_measured=98 sub=0/0 basis=first-run[U]
         mode=direct trigger=- route=self w=11.11 basis=quota.json@46h q=stale gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=receipt, push, gh pr create (stacked on #1246)

Finding: the first four blocks are reconstructed from the transcript after the reboot and carry
the k_measured of the block they precede (the pre-reboot state dir did not survive); the last
block's pair is measured at write time (`k_measured` = `global`). `q=stale` because
`quota.json` is 46 h old, past nothing but named as `statusline.sh` would render it.

## Verdict

**DONE** — every acceptance criterion re-run green by the orchestrator at `91c3a86e2`; `pmat verify` green on all 5 stages; the quorum's three confirmed findings fixed with a mutant each and the fourth refuted; the NotRun lanes (Phase 1 teamwork grill, `pv` contract) and the observations that can only happen on the PR (first CI run of the `traceability` job, merge after #1246) are named in Gaps, each with the artifact that closes it.

IMPL-PMAT-719-RECEIPT-END
