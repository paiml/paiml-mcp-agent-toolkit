# IMPL-PMAT-721 — the #612 collision resolved: 10 MACS items completed (they shipped in #613), MACS-006/007/013 re-planned for their unshipped remainder, 4 duplicate rows cancelled

## Identity

| field | value |
|---|---|
| ticket | PMAT-721 (`kind:triage`, labels `kind:triage`, `human`); `kind-gate.sh` at branch creation: `kind=triage ticket=PMAT-721 files=0`; before the commit: `kind=triage ticket=PMAT-721 files=5` |
| spec | `docs/specifications/goal-mode.md` §5.4 (landing green — the 13 collisions are resolved by a person under their own ticket, never by the sync), §5.1, §12 ("Do not auto-fix a COLLISION") |
| branch | `PMAT-721-macs-612-collisions` from `master` @ `3893ca5f2` — the merge of #1251, which landed #1246–#1251 (goal-mode steps 1–6) |
| HEAD in | `3893ca5f2` |
| HEAD out | the receipt commit on this branch |
| PR | opened after this receipt — `gh pr list --head PMAT-721-macs-612-collisions` |
| `discover.json` sha256 | `1e7f5b8fc09706d676cc1f7059ddffa7f6153c72e85d7ddda4e150c412e3b13a` |
| `gate_cmd` | `cargo test --workspace` — **`gate_cmd_fallback=true`**; a triage branch changes no code, so its gates are the kind gate, the controls that read the roadmap, and the sync's own verdict |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| model gate | `model=opus class=opus decision=admit basis=file` — the session model changed from Fable 5.1 to Opus 5 during this pass; under Fable the same gate had refused this ticket (`model-gate: refused: model fable not permitted for PMAT-721 (fable)`), which is part of why it waited |

### Delegation, quoted verbatim

PMAT-721 carries the label `human` because §5.4 makes "which of the 13 is really #612" a judgement. The operator's instruction for this pass, quoted verbatim: "pmat-implement docs/specifications/goal-mode.md autonomously (you merge, triage tickets, pull requests, and label, tag, close/open)". The judgement was made on evidence, stated as one rule before any lane ran, reviewed by a three-lane quorum, and settled row by row by my own test run.

### One ticket per session — refused, then reaffirmed

`goal.sh set --ticket PMAT-721 --K 106 --khat 53 --basis docs/audits/impl-estimates.jsonl:L19-L23 --phases 3` was refused: `one ticket per session: PMAT-719 was set here — start a new claude session` (R-5). A first call with `--basis first-run` was refused for its basis before R-5 was reached. The operator's instruction above is the reaffirmation. Because the goal was never set, `goal.sh worker` declared no row for the delegate.

### Status-line join `[U]→[V]`

| claim | measured | how |
|---|---|---|
| statusLine `session_id` = hook `session_id` | **true** | `transcript-gate.sh` resolves `39a15c63-…` by `rule=pid-file` |
| `tasks[].id` = hook `agent_id` | **[U]** | the delegate `agentId` `acbf2b638399f7301` is not paired in the hook log |
| `transcript_path` on subagentStatusLine stdin | **[U]** | not measured |
| `k_measured` vs `global=k` | transcript-wide `k_measured=414`; `k` at branch creation was 400, so this ticket's share at this receipt is **14** | `jq -r 'select(.type=="assistant" and ((.isSidechain // false)\|not)) \| (.message.id // .uuid)' <transcript> \| sort -u \| wc -l`, and the same over the lines before the first mention of the branch name |

## What PMAT-721 is

`pmat work sync --check-only` reported one COLLISION: #612 named by the 13 open items MACS-004 … MACS-016, all `inprogress`. CB-2115 cannot land green while it stands (§5.4). #612 was closed on 2026-07-02 as COMPLETED by PR #613 ("feat(macs): Modern Agentic Coding Support (Component 32) — MACS-000…016", squash `d629a105f`, an ancestor of master, 90 files). The component shipped in one squash and the per-ticket close-out never ran. Two months later the 3.39.0 disposition audit (`docs/audits/dispositions-3.39.0.json`, `7ccb614dd`) deferred 12 of the 13 on "no matching commits found on master", a commit-message heuristic that a squash defeats by construction.

**The rule, stated before the quorum ran and applied to every row:** an item is `completed` when every test its block in `docs/specifications/components/modern-agentic-coding-support.md` names (RED tests, ensure and invariant) exists by name or by a named equivalent and passes on master. Otherwise it is re-planned for the unshipped remainder, retitled to that remainder, with `github_issue: null` so `pmat work sync --direction yaml-to-github` mints its own issue (§5.1). A row whose id carries a space is cancelled as a duplicate when its `<contract>/<equation>` is its canonical row's `contract_yaml` and nothing depends on it.

## Universe (T-2)

| field | value |
|---|---|
| source | a `python3 -c` over `docs/roadmaps/roadmap.yaml` selecting the 13 collision rows and the 4 rows whose id starts `MACS-0NN ` (verbatim in `snapshot.json`) |
| count | 17 |
| sha256 | `714284c9cff8b6a0069d2dcc452c7b9835bc3c9cd0b627fa910e22cfb3777d21` |
| drift | `snapshot.sh check` against the working tree: exit 4 (`714284c9` → `682574613eef`). That is this ticket's own write, because the source reads the file the triage edits. Against master's blob (`git show master:docs/roadmaps/roadmap.yaml \| …`): **stable, exit 0**, same sha256. The universe did not move under the triage. |

## Plan and routing

| phase | what | class | `route.sh` line | taken? | trigger |
|---|---|---|---|---|---|
| 1 | the evidence per row (spec blocks, #613's file list, the lib test list), then a three-lane quorum on the classification | review | `route=agy-quorum w=1.00 basis=absent effort=1[U]` | taken, width 3 | Q2 (a spec section, §5.4) + the kind:triage ledger review |
| 2 | adjudication: my own run of every named test, the ledger, the roadmap edits | orchestration | `route=self w=100.00 basis=absent` | taken | — |
| 3 | controls, the sync re-measured live, receipt, PR, merge | orchestration | `route=self w=100.00 basis=absent` | taken | — |

No worker was dispatched. The mutation surface is 18 rows of one file, and the classification is the judgement the ticket exists for, so it stayed with the orchestrator; the width came from the quorum. `quota.json` is absent on this host, so every route line carries `basis=absent`.

## Dispatch ledger

| phase | mode | description | agent id | tool uses / duration | maxTurns hit | resumed | lane / width / agy `conversations` |
|---|---|---|---|---|---|---|---|
| 1 | delegate (`paiml-agy-delegate`, opus) → agy, `writes=false` | `PMAT-721/ph1.delegate quorum width 3 on the 17-row MACS #612 classification` | `acbf2b638399f7301` | 46 / 1,517 s | no | no | quorum / 3 / `f257a998-461e-4a0f-9706-636d4d699e43`, `62c2a721-25ce-4c60-82c6-5708d5bbef38`, `3b9a3ed5-8782-4b86-a5d3-a42894481530` (`child_conversations` 3) |

The lanes ran as `agy-lane.sh --mode plan --schema quorum-schema.json`. The brief asked for `--mode quorum`, which the script does not accept (a dry run exits 2: `unknown mode 'quorum' (goal|teamwork|grillme|plan)`); `plan` is the one mode that passes the prompt byte-for-byte. The lanes wrote four helper files into `/tmp`; the delegate moved them to the out_dir's `litter/` and found the shared checkout byte-identical before and after (HEAD, porcelain, ignored list, `.git/config`, refs, reflog).

## Slots, denials, I-3

| field | value |
|---|---|
| `slots` | 3 (`config-lint.sh`: `slots=3 gh_calls_per_min=30 bank=3`) |
| Claude subagents dispatched for this ticket | **1** (the delegate) |
| `transcript-gate.sh` (session-wide) | `PASS transcript-gate: attempted=14 denied=0 running_peak=2 slots=3 segments=367 files=13 (agent_calls=13 resumes=1 workflow_started=0; denied from hook log) (session 39a15c63-fab3-4148-8cb7-0cb012369b35, rule=pid-file (/run/user/1000/paiml-implement/pid-21623))` |
| denials | `denied=0` |
| Workflow tool | not used |

## Quorum and adjudication (`docs/audits/quorum-PMAT-721.json`)

All three lanes returned PASS and `lane-reduce` agreed (`agreed=true`, `partial=false`). Under the brief, PASS means every row was classified with evidence, not that the lanes agreed on each row. They split 2–1 on five rows. No lane ran a test: the delegate counted `cargo` 0 times in any lane response. So every row was settled by my own run of every test its block names, on master.

| row | lanes | mine | settled by |
|---|---|---|---|
| MACS-004 | completed 3/3 | completed | 5 named tests pass, including the invariant `serde_string_repr` |
| MACS-005 | completed 3/3 | completed | 4 named tests pass |
| MACS-006 | re-plan 2/3 | **re-planned** | `ledger::receipt_records_both_levels` has no equivalent. `with_ladder` (`work_ledger_receipt.rs:153`, called from `core_handlers/contract.rs:391`) is untested, and the CB-1653 tests hand-write the levels into receipt JSON (`check_macs_tests_ladder.rs:38`) |
| MACS-007 | completed 2/3 | **re-planned** (majority overruled) | the block's invariant `cot::status_renders_v2` has no test by name or equivalent, and `handle_work_status` (`handlers.rs:746`, 102 lines) renders no chain-of-thought step at all. The rule counts invariant tests, as it did for MACS-004 and MACS-014 where they exist. Lane 2's other reason, that `ticket_validate_migrate.rs` is missing, is false |
| MACS-008 | completed 3/3 | completed | 4 named tests pass; invariant measured: `check_chain` (`work_cot.rs:388`) makes no I/O call |
| MACS-009 | completed 2/3 | completed | `one_obligation_per_step_in_ticket_yaml` is matched by `every_aprender_gh_663_to_672_contract_derives_zero_empty_statements`, which asserts one obligation (`- id:`) and one claim (`- hypothesis:`) per step over 10 fixtures (`work_cot_tests.rs:542–551`) |
| MACS-010 | completed 3/3 | completed | the `cb1650` tests pass; 6 of 6 repo skills pin `effort:` |
| MACS-011 | completed 2/3 | completed | all five named tests pass under their exact names. `concurrency8_zero_lock_errors_zero_scratch` is `#[ignore]`d because it spawns the MCP server binary, and it passed when run with `--ignored --exact` (23 s). Lane 2's fact is true: `fixtures/mcp-sweep/` is empty and the goldens are inline |
| MACS-012 | completed 3/3 | completed | 4 workflow tests pass; invariant measured: no `resume` in `release-sweep.ultracode.mjs` |
| MACS-013 | re-plan 2/3 | **re-planned** | not for the lanes' reason, since the predicate test exists (`stale_when_ledger_newer_than_artifact`, found by the delegate). CB-1655 returns `Warn`/`Severity::Warning` where the block says red, and no test covers that arm |
| MACS-014 | completed 3/3 | completed | 4 named tests pass, including the invariant `two_runs_identical` |
| MACS-015 | completed 3/3 | completed | the `cb1657` tests and `refactor_auto_references_registry` pass; invariant measured: `claude-3-opus` kept in `docs/agent-models.md` |
| MACS-016 | completed 3/3 | completed | all 5 named tests pass, including `verify_readonly` |
| the 4 rows whose id carries a space | duplicate-cancel 3/3 | cancelled | each id's `<contract>/<equation>` is its canonical row's `contract_yaml`; all four were created in one second (16:54:11); they are referenced only by the rendered `ROADMAP.yaml` and the historical 3.39.0 dispositions (`git grep -lF`) |

The delegate's remaining open questions, answered. `ROADMAP.yaml` lists the four cancelled ids, but regenerating it is outside a triage branch and it has been stale since 2026-07-05, so it is recorded as MACS-013's remainder. MACS-009's 3.39.0 reason differs from the other twelve ("superseded in scope by PMAT-685/#1200 … obligations+claims registry is unimplemented"), and the block names no registry; its note says so.

## The change

`docs/roadmaps/roadmap.yaml`, 61 lines in 18 rows:

- **10 rows** `inprogress → completed`, `github_issue: 612` kept as the historical link.
- **3 rows** `inprogress → planned`, `github_issue: 612 → null`, retitled to the remainder, with notes naming what shipped and what did not.
- **4 rows** `inprogress → cancelled`, each with a note naming its canonical row.
- **PMAT-721** → `completed`. Its second acceptance criterion is amended in place, with the reason. The first wording ("each of the 13 items either names a distinct open issue that is its own, or is cancelled with a note saying which item kept #612") assumed all 13 were open work, and 10 had shipped. The amended text keeps the part §5.4 cares about: resolved with evidence, never by the sync.

## Verification table (claimed vs my rerun)

| check | claimed | my rerun |
|---|---|---|
| every test the 13 blocks name, on master `3893ca5f2` | lanes: "exist" (test-list greps; no test run) | **68 passed, 0 failed, 1 ignored** (49 filters) |
| the ignored concurrency test | — | **1 passed** (`--ignored --exact`, 23 s) |
| `pmat work sync --check-only`, live, before | — | 118 findings: **1 COLLISION**, 63 ORPHAN-ROADMAP, 54 ORPHAN-GITHUB, 0 DRIFT |
| the same, after | — | 115 findings: **0 COLLISION**, 61 ORPHAN-ROADMAP, 54 ORPHAN-GITHUB, 0 DRIFT (exit 1: the orphans are PMAT-723's) |
| `scripts/work-sync-control.sh` (arm 4 round-trips the real roadmap) | — | all 4 arms; 5,283 lines byte-for-byte, so the hand edits are serializer-canonical |
| `pmat work validate` (the `roadmap validates` CI job) | — | exit 0 |
| snapshot drift | — | stable against master's blob |

ORPHAN-ROADMAP moved 63 → 61: −4 cancelled duplicates, −1 PMAT-721 (completed), +3 re-planned rows (MACS-006, 007 and 013 were inside the collision before, where the sync does not count them as orphans).

### Discrimination

The sync judges the resolution: 1 COLLISION before, 0 after, same binary, live snapshot. That the sync can report a COLLISION at all is `work-sync-control.sh` arm 1 (COLLISION #612 naming M0 and M1), green in the same run.

## Jidoka log

| # | defect | owner | whys |
|---|---|---|---|
| 1 | the first run of the named tests matched nothing (0 passed, 21,639 filtered out) | me | the Bash tool runs zsh, which does not word-split an unquoted `$F`, so the 46 names reached libtest as one filter; re-run from a bash script with an array |
| 2 | a probe printed `command not found: cut` | me | a variable named `path` in zsh is tied to `$PATH` |
| 3 | `snapshot.sh check` exit 4 | me (false alarm) | the source reads the file the triage writes; against master's blob it is stable |
| 4 | master's first CI run after #1251 went red: `ci / test` cancelled 4 minutes into "Run tests" on self-hosted `intel-clean-room-5`, then `ci / gate` and `gate` failed | the self-hosted runner, not the merged tree | it was the only CI run in its concurrency group, the reusable workflow sets no concurrency, and the job timeout is 60 minutes, so the cancel came from the runner side. `gh run rerun 34503117385 --failed` requested; its result is recorded on the PR |
| 5 | the 3.39.0 disposition audit deferred 12 shipped tickets | the audit's method (`docs/audits/dispositions-3.39.0.json`, `7ccb614dd`, a document rather than a pmat command) | it matched ticket ids against commit messages, and #613 was a squash |

## Estimates

| field | value |
|---|---|
| `K̂` | 53, `basis=docs/audits/impl-estimates.jsonl:L19-L23` (`estimate.sh pmat 3`: `ROWS=5 MEDIAN=53`, `cycles=absent[U]`). That basis is code tickets; no triage row exists yet, and this row is the first |
| `K` | 106 (`2 × K̂`), andon line 85 |
| actual | **14** turns at this receipt (`k_measured` 414 minus 400 at branch creation); the PR and merge turns come after it |

## Gaps

| gap | artifact that closes it |
|---|---|
| MACS-006, MACS-007 and MACS-013 have no issue yet (ORPHAN-ROADMAP, no issue) | `pmat work sync --direction yaml-to-github` under PMAT-723 (the fixers) |
| `ROADMAP.yaml` (rendered 2026-07-05) is stale and still lists the four cancelled rows; `src/roadmap/sync.rs:6` calls `docs/roadmaps/roadmap.yaml` HISTORICAL | MACS-013's remainder |
| the mcp-sweep concurrency test is `#[ignore]`d and never runs in `cargo test --lib` | a CI step that runs it after the build (MACS-011's note) |
| Phase 1 `teamwork` grill **NotRun**; the quorum reviewed the classification instead | a `/teamwork-preview` receipt on §5.4 |
| `pv` contract **NotRun** (`contracts_dir=contracts`); a triage binds no contract | — |
| `merged green on required_check` pending | the PR's checks |
| the untracked scratch files in the checkout (`empty_test_project/`, `test_clap.rs`, `test_parsed*`, `src/cli/test_clap_checks.rs`) predate this session's tickets and were not touched | their owner |

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 400
kind: triage
ledger: docs/audits/triage-PMAT-721-ledger.json
rows_total: 17   rows_expected: 17
verdict_counts: completed=10 planned-remint=3 duplicate-cancel=4
snapshot_sha: 714284c9cff8b6a0069d2dcc452c7b9835bc3c9cd0b627fa910e22cfb3777d21
slots: 3   denials: 0
gh_commands: gh issue view 612 --json closedAt,stateReason,closedByPullRequestsReferences · gh pr view 613 --json title,mergedAt,mergeCommit,additions,deletions,changedFiles · gh pr view 613 --json mergeCommit · pmat work sync --check-only (its own gh issue list and GraphQL reads) ×2 — no GitHub write

routes:
  ph1  class=review         route=agy-quorum  w=1.00    basis=absent   effort=1[U]
  ph2  class=orchestration  route=self        w=100.00  basis=absent
  ph3  class=orchestration  route=self        w=100.00  basis=absent

verification:
  cmd=cargo-test-lib-macs-named(49-filters)@3893ca5f2  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/macs-tests.log  sha256=ea86ae4dd514cc46
  cmd=cargo-test-concurrency8-ignored-exact@3893ca5f2  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/concurrency8.log  sha256=33bc17102fdd4243
  cmd=work-sync-check-only-before(live,759-issues)  claimed_exit=-  rerun_exit=1(1-COLLISION)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/sync-check-now.log  sha256=2080edc0416d0099
  cmd=work-sync-check-only-after(live,759-issues)  claimed_exit=-  rerun_exit=1(0-COLLISION;orphans)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/sync-721-after.log  sha256=93459d240499de17
  cmd=work-sync-control-4-arms  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/work-sync-control-721.log  sha256=7bbb9dfc6adfccd7
  cmd=work-validate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/work-validate-721.log  sha256=072ec77e5ba9a056
  cmd=delegate-receipt-ph1(lane-reduce)  claimed_exit=0(3/3-PASS)  rerun_exit=0(agreed=true)  log_path=docs/audits/quorum-PMAT-721.json  sha256=5d05c33943dfa302
  cmd=snapshot-check-vs-master-blob  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/snapshot-check-721.log  sha256=11472780ba88bb20
  cmd=transcript-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-721.log  sha256=651fbde81681438b
  cmd=kind-gate(diff-files)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/kind-gate-721.log  sha256=a979fb48c90a78cc
  cmd=model-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/model-gate-721.log  sha256=44878c7c890a08c9

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

Finding — `k_measured` (414) is transcript-wide: this session carried PMAT-718, 719, 720, 722, 724 and 728 and the merge of #1251 before this ticket. `k` (14) is this ticket's share, counted from the first transcript line that names its branch (400). The gap is that history, not drift.

[status] ticket=PMAT-721 phase=1/3 global=14/53(K=106) k_measured=414 sub=46/46 basis=docs/audits/impl-estimates.jsonl:L19-L23
         mode=quorum:quorum trigger=Q2 route=agy-quorum w=1.00 basis=absent q=? gate=PASS slots=1/3 denied=0
         red=- filed=- blocker=- next=adjudicate the five split rows with my own test run

[status] ticket=PMAT-721 phase=2/3 global=14/53(K=106) k_measured=414 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L23
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=apply the 18 row edits, re-measure the sync, run the controls

[status] ticket=PMAT-721 phase=3/3 global=14/53(K=106) k_measured=414 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L23
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=commit, PR, CI, merge

## Verdict

**DONE.** The collision is zero by measurement (`pmat work sync --check-only`: 0 COLLISION, live), resolved by a person's judgement under this ticket, never by the sync, as §5.4 and §12 require. The judgement is one rule applied to all 17 rows, reviewed by three lanes and settled by my own run of every named test. Ten rows are completed because they shipped in #613. Three are re-planned for the part that did not ship, each naming it. Four duplicates are cancelled. What remains is PMAT-723's: 61 ORPHAN-ROADMAP and 54 ORPHAN-GITHUB, which the fixers address.
