# IMPL-PMAT-720 — goal-mode step 3: `release:`, `pmat work sync` made real, the #612 collisions handed to a human

## Identity

| field | value |
|---|---|
| ticket | PMAT-720 (`kind:code`, `orch:fable`, `kind-gate.sh` exit 0, files=23 against `origin/master`) |
| spec | `docs/specifications/goal-mode.md` §4.1, §4.2, §5 (5.1–5.4), §9, §11 step 3, §12 |
| branch | `PMAT-720-work-sync-release-field`, branched from `master` @ `20fda3ca6` — **not stacked** on #1246/#1247 (no code overlap) |
| HEAD in | `20fda3ca6` |
| HEAD out | `f55d9782a` (last code commit); `e68b78fc6` re-renders the unrun-tests ledger; the commit carrying this receipt follows |
| PR | opened from this branch after the receipt commit — `gh pr list --head PMAT-720-work-sync-release-field` |
| `discover.json` sha256 | `b515d3f9b4c08e5ed9674023693708edb3ae704fb26f4fa639b8228914827b4b` (the same per-session, per-repo discovery PMAT-719 re-derived after the host reboot) |
| `gate_cmd` | `cargo test --workspace` — **`gate_cmd_fallback=true`**; `pmat verify` is the gate this repository's CLAUDE.md names and is what was run |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| model gate | `model=fable class=fable decision=admit basis=file` |

### One ticket per session — refused, then reaffirmed

`goal.sh set --ticket PMAT-720` exited 2: `one ticket per session: PMAT-719 was set here — start a new claude session` (R-5). The refusal was reported to the operator with the reason (turn accounting is per transcript). The operator's next message, quoted verbatim: **`pmat-implement docs/specifications/goal-mode.md autonomously`**. That is the reaffirmation the delivery rules require, so the ticket proceeded in this session with the refusal recorded here, no session id faked, and the accounting split by hand below.

### Status-line join `[U]→[V]`

| claim | measured | how |
|---|---|---|
| statusLine `session_id` = hook `session_id` | **true** | `transcript-gate.sh` resolved `39a15c63-…` by `rule=pid-file`; the hook's `events-39a15c63-….jsonl` (11 rows, 0 denials) carries the same id |
| `tasks[].id` = hook `agent_id` | **[U]** | the four `agentId`s in the transcript (`a77340cd924fa8b7e`, `aab2695e6c80ded4d`, `ad437751fe9da025d`, `ae316b64958e9d9f3`) are not paired in the post-reboot hook log |
| `transcript_path` on subagentStatusLine stdin | **[U]** | not measured |
| `k_measured` vs `global=k` | transcript-wide `k_measured=146`; PMAT-719's receipt closed at 98, so this ticket's share is **48** | `jq -r 'select(.type=="assistant" and ((.isSidechain // false)\|not)) \| (.message.id // .uuid)' <transcript> \| sort -u \| wc -l` |

## What step 3 is

`release: Option<String>` on `RoadmapItem` (§4.2): a bare semver string, a projection of the GitHub milestone of the same title, `serde(default, skip_serializing_if)` so every existing entry round-trips unchanged; `pmat work sync` is its only writer.

`pmat work sync` was a stub that printed "not yet implemented" on every path behind five tests that asserted `is_ok()`. It now judges the roadmap/GitHub bijection (§5.1) from a snapshot — `ORPHAN-ROADMAP`, `ORPHAN-GITHUB`, `COLLISION`, and time-bounded `DRIFT` (§5.2) — through a pure engine (`src/services/work_sync/`), and fixes the orphans in the chosen direction through an `Action` list printed before anything is written. `--check-only` is the gate-shaped mode (exit 1 on any finding). A `COLLISION` is reported and never fixed (§5.4). `--snapshot`/`--write-snapshot` make every run replayable offline; the control script runs entirely from snapshot files.

The human half: the thirteen MACS items that all name #612 are filed as **PMAT-721** (`kind:triage`, `human`), never auto-fixed.

## Plan and routing

| phase | what | class | `route.sh` (verbatim) | taken? | trigger |
|---|---|---|---|---|---|
| 1 | `release:` field, RED then GREEN, constructors and four struct literals | impl | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | **not taken** — a field, a constructor and four literals proven RED→GREEN in two commits; the same call PMAT-707 made for a five-line change | — |
| 2 | RED engine suite (36 tests, direct), then the engine through the agy goal lane | impl, single module | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | **taken** — lane FAILED on isolation (below); its commit verified and kept | — |
| 3 | clap surface, handler, `gh` snapshot, seven handler tests, control script, canonicalisation, PMAT-721 | impl | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | **not taken** — R-4 allows one `writes=true` agy lane per repository and ph2 held it; done by me in my own worktree with disjoint paths | — |
| 4 | pre-PR quorum on `origin/master...a6b84ef3e`, adjudication, fixes with mutants, ledgers, receipt, PR | review, then orchestration | `route=agy-quorum w=1.00 basis=absent effort=1[U]`; `route=self w=100.00 basis=absent` | taken | Phase 4 pre-PR review, width 3 |

`quota.json` is absent on this host (`quota.sh binding`: `age_h=absent account_mismatch=true`), so every route line carries `basis=absent`; the Phase 1 `teamwork` grill (Q2) was not run (Gaps).

## Dispatch ledger

| phase | mode | description | agent id | turns | maxTurns hit | resumed | lane / width / agy `conversations` |
|---|---|---|---|---|---|---|---|
| 2 | delegate (`paiml-agy-delegate`, opus) → agy `goal`, `writes=true` | `PMAT-720/ph2.delegate goal width 1 on the work sync engine` | `ad437751fe9da025d` | 21 tool uses, 777 s | no | no | goal / 1 / `bea8ec51-4604-4dd0-96f8-3648eb479a62` — **exit 3, LANE ISOLATION VIOLATED**: the lane did its whole job in the shared checkout and committed `4e717c7d3` onto this branch; its throwaway worktree `lane-a0edaf8b1-3737329` was never touched |
| 4 | delegate (`paiml-agy-delegate`, opus) → agy `grillme` quorum, `writes=false`, sandboxed | `PMAT-720/ph4.delegate quorum width 3 on the step-3 diff` | `ae316b64958e9d9f3` | 29 tool uses, 730 s | no | no | quorum / 3 / `69fae3b4-ed3d-4874-9b4e-ee080d290e33`, `c3bcc3d0-63a0-49fe-8bdd-0d3282977035`, `61781d12-c55d-46d2-80dc-690ffc7680e2` — 3/3 FAIL, `fanout_label=consensus` (3 measured children) |

**The escaped commit was verified, not trusted.** `4e717c7d3` (author `paiml-implement lane <noreply@paiml.invalid>`, trailer `Pmat-Ticket: PMAT-720`): 1 file, +363/−11, `tests.rs` untouched, 0 `.unwrap()`, 0 `todo!`, the three pinned signatures unchanged (only `_x` parameter renames), and the suite it claimed — **36 passed, 0 failed** on my own run in the main checkout. Kept on the branch; the lane verdict stays FAILED. The delegate's diagnosis of the harness gap is carried in Gaps.

## Slots, denials, I-3

| field | value |
|---|---|
| `slots` | 3 (`config.json`, `config-lint.sh`: `slots=3 gh_calls_per_min=30 bank=3`) |
| Claude subagents dispatched for this ticket | **2** (both delegates; no worker) |
| `transcript-gate.sh` (session-wide, PMAT-719 included) | `PASS transcript-gate: attempted=5 denied=0 running_peak=2 slots=3 segments=135 files=5 (agent_calls=5 resumes=0 workflow_started=0; denied from hook log)` |
| denials | `denied=0` (hook log, 11 rows) |
| Workflow tool | not used |

## Quorum (Phase 4, `docs/audits/quorum-PMAT-720.json`)

3/3 **FAIL** on `a6b84ef3e`; 15 dedup groups, 13 distinct claims after my own dedup — 9 CONFIRMED, 4 REFUTED, every one re-measured first:

| # | finding | lanes | adjudication | closed by |
|---|---|---|---|---|
| 1 | an open item naming an open `no-roadmap` issue lands in `matched_pairs` and escapes every class | 1, 2 | **CONFIRMED** | `OrphanReason::IssueExcluded`; every direction skips it naming the label; 1 test |
| 2 | `gh issue list --limit 5000` / milestones `per_page=100` truncate silently and read as coherence | 1, 2 | **CONFIRMED** | `refuse_truncation`: a result at the cap is an error, never a report; 1 test |
| 3 | control arm 4 passes for the wrong reason (its snapshot names no real issue) | 2, 3 | **REFUTED** | arm 4 proves the serializer, arms 1–3 prove the fixers; a real issue in that snapshot would let a control edit the repository's own entries; header now says so |
| 4 | §5.3 lists `release` as roadmap-authoritative, §4.1 calls it a projection; the code implements §4.1 | 1, 3 | **CONFIRMED** | §5.3 amended in this PR (one sentence); flagged for the operator |
| 5 | grace applied per pair, not per field | 3 | **REFUTED** | §5.2's clock is `now − max(item.updated, issue.updated_at)`; neither surface has per-field timestamps |
| 6 | widening "open" to `blocked`/`review` breaks CB-2114 consistency | 3 | **REFUTED** | deliberate, documented, `is_open()` is `pub` so later rules inherit one definition; lane 3's citation was past EOF |
| 7 | `full_is_the_union_of_both_directions` never computes the union | 1 | **CONFIRMED** | the test now computes both write-sets and asserts Full equals their union and skips nothing |
| 8 | `check_only_passes_on_a_bijection` asserts nothing about the JSON | 3 | **CONFIRMED** | `report_json_carries_the_verdict_and_the_classes` parses the document |
| 9 | no test for the release-drift skip in yaml-to-github | 3 | **CONFIRMED** | `yaml_to_github_leaves_a_stale_release_to_the_other_direction` |
| 10 | commit `6d6d75168`'s message says "three notes: fields"; the diff re-flows PMAT-707's `notes:` (30 lines → 1) and re-quotes PMAT-720's three acceptance criteria | 1 | **CONFIRMED** | recorded here rather than rewriting history the quorum judged; parsed values equal (`yaml.safe_load` both sides: 278 items, equal) |
| 11 | arm 4 will fail on a legitimate future hand-wrapped edit | 3 | **REFUTED** as a defect | that is the invariant (§4.2); the failure names the measured canonicaliser `pmat work edit <id> --priority <its current priority>` |
| 12 | the brief said 11 commits, the range has 10 | delegate | **CONFIRMED** | my counting error; the range was named explicitly |
| 13 | three of lane 3's citations are past EOF | delegate | **CONFIRMED** | those claims re-grounded by me before adjudication (rows 5, 6, 9) |

## Mutation table

| mutant | planted as | killed by | log |
|---|---|---|---|
| F1: an excluded issue reads as matched | `} else if false && !issue.in_universe()` | `an_open_item_naming_a_no_roadmap_issue_is_an_orphan_roadmap` — 1 of 2 failed (the control arm `a_no_roadmap_label_takes_an_issue_out_of_the_universe` stays green, as it should) | `mut720-f1.log` da48d0c1c9ceccaa |
| F2: the truncation guard never fires at the cap | `if count > cap` | `a_snapshot_at_the_page_cap_is_refused` — 0 passed, 1 failed | `mut720-f2.log` 600c0eab6b40381c |
| F3: Full drops the yaml-to-github writes | `Direction::YamlToGithub` alone opens issues; `Full` skips | `full_is_the_union_of_both_directions` — 0 passed, 1 failed | `mut720-f3.log` fd51bebe714e9c83 |
| release field absent | the tree before `1da6dff87` | `release_survives_a_yaml_round_trip` FAILED at `abb27d2fa` ("the release key was dropped"); the control arm passed on both sides | commit `abb27d2fa` |
| empty engine | `check`/`plan`/`apply_to_roadmap` returning nothing | 28 of 36 engine tests FAILED at `a0edaf8b1` | commit `a0edaf8b1` |
| a planted COLLISION, ORPHAN-ROADMAP and ORPHAN-GITHUB | control arm 1 | exit 1 and each finding by class, id and number in the JSON; arm 2 proves the same command can pass | `control720.log` 65d07021d2d43218 |
| nine `.unwrap()`-free but six `panic!` in test arms | the panic ratchet went 785 → 791 | `let … else { unreachable!() }`; ratchet back at 785/785 | `ratchet720-final.log` ebddd84fd2c66165 |

Every mutant was applied one at a time to the real source with backups under `.pmat/` (not tmpfs — the PMAT-719 lesson), the named tests run, and the source restored to a byte-identical copy before the next.

## Verification table (claimed vs my rerun)

| command | claimed | rerun | at |
|---|---|---|---|
| `cargo test --lib -- services::work_sync core_handlers::sync` | lane: "All 36 tests passed" (unverified claim) | **47 passed, 0 failed** (38 engine + 9 handler) | `f55d9782a` |
| `cargo test --lib -- roadmap::tests::release` | — | 2 passed | `1da6dff87` |
| `bash scripts/work-sync-control.sh ./target/debug/pmat` | — | 4/4 arms; arm 4 round-trips 5052 lines of the real roadmap byte-for-byte | `f55d9782a` |
| `pmat work sync --check-only --path . --write-snapshot …` (real repository, `gh`) | — | exit 1: **107 findings — 1 COLLISION (#612 ← MACS-004..MACS-016), 52 ORPHAN-ROADMAP (all no-issue), 54 ORPHAN-GITHUB, 0 DRIFT**; open items 65, open issues 54, matched 0; snapshot 759 issues, milestones 3.40.0/3.41.0/3.42.0 | `4e717c7d3`; re-run at `f55d9782a` from the saved snapshot: 108 (53 ORPHAN-ROADMAP, PMAT-721 itself being the 53rd), open items 66, everything else equal |
| `pmat comply ratchet` | — | every metric at baseline after the `panic!` fix (`panic_macro_calls_src 785/785`) | `f55d9782a` |
| `cargo clippy --all-targets -- -D warnings` | — | clean (it found the two `examples/` literals the query index does not cover) | `f55d9782a` |
| `pmat verify --format json` | — | at `a6b84ef3e`: ok:false, tests stage (`the_committed_ratchet_holds_at_head`, the panic ratchet) → fixed at the cause in `f55d9782a`; at `f55d9782a`: ok:false, tests stage, one test of 21360 (`the_committed_ledger_matches_the_tree` — the four new tests drifted the unrun-tests ledger) → re-rendered in `e68b78fc6`; at `e68b78fc6`: **ok:true**, stages_measured=5, not_measured=[] (format 2.4 s · complexity · satd 1.7 s · clippy 1.4 s · tests 342 s; 347 s total, nohup on the committed tree) | |
| canonicalisation `6d6d75168` | "no value changed" | `yaml.safe_load` of both sides equal, 278 items; `pmat work validate` passes | `6d6d75168` |
| `analyze unrun-tests --check-ledger` / `analyze reachability --check-ledger` | — | both "ledger is current" | `068ec5bab` |
| `transcript-gate.sh` / `kind-gate.sh` / `model-gate.sh` | — | PASS / `kind=code files=23` / `decision=admit` | `f55d9782a` |

`./target/debug/pmat` is the executable cargo reports; it was fingerprinted for the string `work sync --check-only` before the control ran.

### Discrimination

The control reads CB-2113's lesson: every arm asserts the exit code **and** the JSON row that proves which finding produced it, a missing row is exit 2, and arm 4's byte-for-byte diff is against the repository's own 5052-line roadmap through a real save. The engine is pure and every verdict replays from a snapshot file; the handler is the only place that touches `gh`, the roadmap file, or the network, and a page cap that is reached is an error, never a report.

## Jidoka log

| defect | owner | disposition |
|---|---|---|
| `goal.sh set` refused a second ticket per session (R-5) | skill doctrine | reported; operator reaffirmed verbatim; accounting split by hand (Identity) |
| the agy goal lane escaped its worktree and committed in the shared checkout (`exit 3`) | paiml-implement harness | commit verified independently and kept; the lane verdict is FAILED; harness gap named in Gaps |
| two `RoadmapItem` literals in `examples/` were outside the query index; the commit was refused by the hook's clippy | this row | found by `cargo clippy --all-targets`, fixed (`1da6dff87`) |
| my worktree patch landed `release: None` before a struct literal's opening brace | this row | fixed; the hook's clippy refused the commit first |
| the roadmap was not in the serializer's canonical form (folded `notes:`), so arm 4 failed on first run | pre-existing | canonicalised once (`6d6d75168`); arm 4 now enforces it and names the fix |
| commit message `a6b84ef3e` claimed both ledgers; only one had been re-rendered (`--write-ledger` refused the dirty tree) | this row | corrected by `068ec5bab`, message says so |
| commit message `6d6d75168` miscounts what changed (quorum #10) | this row | recorded in the quorum artifact and here |
| `panic_macro_calls_src` 791 > 785 from six `panic!` in test match arms | this row | `let … else { unreachable!() }`; baseline untouched |
| a backtick inside a double-quoted log line ran as a command and aborted the mutation run | this row | the exit trap restored the sources; re-run with single quotes |
| quorum 3/3 FAIL, 13 claims | this row | 9 confirmed and fixed or recorded, 4 refuted (table above) |
| `pmat verify` red at `a6b84ef3e` (the panic ratchet), then red at `f55d9782a` (the unrun-tests ledger drifted by the four new tests) | this row | fixed at the cause both times; the ledger re-rendered in `e68b78fc6`; final run at `e68b78fc6` |

## Estimates

| field | value |
|---|---|
| `K̂` | 4, `basis=first-run[U]` (`estimate.sh pmat 4`: `ROWS=1` < 3, so K̂ = N) |
| `K` | 8 |
| actual | **48** turns for this ticket (transcript `k_measured` 146 minus PMAT-719's 98) — over K, said so in the first status block and continued deliberately |
| `pr_runs` / `mg_runs` | not yet observable: the PR is opened after this receipt |

## Gaps

| gap | artifact that closes it |
|---|---|
| Phase 1 `teamwork` grill of the spec section (Q2) **NotRun** — the pre-PR quorum reviewed the diff instead | a `/teamwork-preview` receipt on §5 / §11 step 3 |
| `pv` contract **NotRun** — `contracts_dir=contracts` is set; the spec names none for step 3 | `contracts/work-sync-v1.yaml` bound by a test |
| the fixers were **not run against the real repository** — `--direction yaml-to-github` would open 52 issues and `github-to-yaml` would add 54 items; that is an operator action, run from `--dry-run` first | the operator's run; `--check-only` then reports 0 ORPHAN-* (§13 item 5) |
| the 13 #612 collisions — a human's judgement (§5.4) | **PMAT-721** |
| the control is not yet in CI — step 3 wires no job; CB-2115 (step 4) is the gate that will run it first | step 4 |
| `release:` is not yet projected onto any real item (the milestones exist: 3.40.0, 3.41.0, 3.42.0; 39 open issues carry one) | the operator's `github-to-yaml` run |
| §5.3 amended by an agent — a doctrine sentence, minimal, but the operator should confirm | review of the spec diff in the PR |
| harness: `agy-lane.sh` isolates a `writes=true` lane by cwd but does not confine it, and the harness identity travels with a lane that `cd`s out, so 3 of its 4 assertions cannot see an escape; only "HEAD moved" did | an issue on the paiml-implement skill (other repository; not filed by this row) |
| `merged green on required_check` — pending CI on the PR | the PR's checks |
| the 15 untracked scratch files in the checkout are not this row's and were left untouched | their owner |

## Machine-readable

orch_model: fable [V]   orch_class: fable   orch_decision: admit   orch_basis: M>=3
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 92

routes:
  ph1  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=self (RED->GREEN proven in two commits)
  ph2  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=agy-goal (lane FAILED isolation; work verified and kept)
  ph3  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=self in own worktree (R-4 one-writer)
  ph4  class=review         route=agy-quorum  w=1.00    basis=absent   effort=1[U]
  ph4  class=orchestration  route=self        w=100.00  basis=absent

verification:
  cmd=cargo-test-lib-work_sync+sync(47)             claimed_exit=0  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/tests720.log  sha256=12fd6c538c0577f1
  cmd=work-sync-control-4-arms                      claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/control720.log  sha256=65d07021d2d43218
  cmd=work-sync-check-only-real-repository          claimed_exit=-  rerun_exit=1  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/check-real-final.json  sha256=91d1bbfe75ab01a8
  cmd=comply-ratchet                                claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ratchet720-final.log  sha256=ebddd84fd2c66165
  cmd=mutant-F1-excluded-reads-as-matched           claimed_exit=-  rerun_exit=1(test-result-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut720-f1.log  sha256=da48d0c1c9ceccaa
  cmd=mutant-F2-cap-never-refused                   claimed_exit=-  rerun_exit=1(test-result-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut720-f2.log  sha256=600c0eab6b40381c
  cmd=mutant-F3-full-drops-yaml-to-github           claimed_exit=-  rerun_exit=1(test-result-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut720-f3.log  sha256=fd51bebe714e9c83
  cmd=transcript-gate                               claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-720.log  sha256=103714c51eeab139
  cmd=kind-gate                                     claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/kind-gate-720.log  sha256=86eeafe88f95f47c
  cmd=model-gate                                    claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/model-gate-720.log  sha256=d91326c896148d6d
  cmd=pmat-verify@a6b84ef3e                         claimed_exit=-  rerun_exit=1  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify720.json  sha256=9fe3603583f769de
  cmd=pmat-verify@f55d9782a                         claimed_exit=-  rerun_exit=1  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify720-f55d9782a.json  sha256=50169e252a6dd4d7
  cmd=pmat-verify@e68b78fc6                         claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify720-final.json  sha256=5b2312299e580ccd

`claimed_exit=-` marks a row with no second party. The one row with a claim (the engine suite) names the lane's "36 passed" as the claim it was. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

[status] ticket=PMAT-720 phase=1/4 global=104/4(K=8) k_measured=104 sub=0/0 basis=first-run[U]
         mode=direct trigger=- route=agy-goal w=1.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=release: field RED->GREEN; K already exceeded (goal.sh set refused: one ticket per session, operator reaffirmed)
[status] ticket=PMAT-720 phase=2/4 global=110/4(K=8) k_measured=110 sub=0/0 basis=first-run[U]
         mode=quorum:goal trigger=R-4-single-module-width-1 route=agy-goal w=1.00 basis=absent q=? gate=FAIL slots=1/3 denied=0
         red=lane-isolation-exit-3 filed=- blocker=- next=verify the escaped commit 4e717c7d3 myself; ph3 surface in my own worktree
[status] ticket=PMAT-720 phase=3/4 global=124/4(K=8) k_measured=124 sub=0/0 basis=first-run[U]
         mode=direct trigger=- route=agy-goal w=1.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-721 blocker=- next=ff-merge, ledgers, quorum on origin/master...a6b84ef3e
[status] ticket=PMAT-720 phase=4/4 global=130/4(K=8) k_measured=130 sub=0/0 basis=first-run[U]
         mode=quorum:quorum trigger=Phase-4-pre-PR-review route=agy-quorum w=1.00 basis=absent q=? gate=FAIL slots=1/3 denied=0
         red=quorum-3/3-FAIL filed=- blocker=- next=adjudicate 13 claims, fix 9 with mutants, refute 4
[status] ticket=PMAT-720 phase=4/4 global=146/4(K=8) k_measured=146 sub=0/0 basis=first-run[U]
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-721 blocker=- next=receipt, push, gh pr create

Finding: `global=k` and `k_measured` are the transcript-wide count, which includes PMAT-719's 98 turns (Identity); the first four blocks are reconstructed after the fact with the count of the turn they describe. `q=?` because `quota.json` is absent.

## Verdict

**DONE** — every acceptance criterion re-run green by the orchestrator: the field round-trips (2 tests), the control proves `--check-only` can fail and can pass and round-trips the real roadmap byte-for-byte (4 arms), the grace window is a bound in time (4 tests), `github-to-yaml` projects `release:` from the milestone and never mints one, `--dry-run` writes nothing and a collided item is skipped by every direction (tests and arms), and the real `--check-only` reports the thirteen MACS items on #612 as one COLLISION that PMAT-721 names for a human. `pmat verify` green on all five stages at `e68b78fc6`; the quorum's nine confirmed claims closed with a test or a record each, four refuted with the reasoning kept. The operator actions this step deliberately leaves (running the fixers, confirming §5.3, resolving PMAT-721) and the NotRun lanes are named in Gaps with the artifact that closes each.

IMPL-PMAT-720-RECEIPT-END
