# IMPL-PMAT-1253 — goal-mode step 4a: `pmat work sync --check-only` reaches 0 — 18 shipped items completed, 6 paired, 3 epics exempted, #1240 closed, 37 issues minted, 44 items added

## Identity

| field | value |
|---|---|
| ticket | PMAT-1253 (`kind:triage`), filed issue-first from #1253 with `pmat work add --github-issue 1253`, so its id ends in its issue number (CB-2112's invariant from birth). `kind-gate.sh` at creation: `kind=triage ticket=PMAT-1253 files=0`; before the commit: `kind=triage ticket=PMAT-1253 files=5` |
| spec | `docs/specifications/goal-mode.md` §5.1 (the set predicate, tolerance zero), §5.3 (GitHub is authoritative for existence and state), §5.4, §4.3 (an epic's sub-issues are the tickets' issues), §12 |
| branch | `PMAT-1253-sync-fixers`, stacked on #1252 (PMAT-721) and rebased onto its fix commit `f5d8c148e` |
| PR | opened as a draft after this receipt. CB-2113 reads `origin/master..HEAD`, so #1252's commits, which name PMAT-721, sit in this PR's range until #1252 merges, and this branch marks PMAT-721 completed. Merge #1252 first, rebase, then mark ready |
| `discover.json` sha256 | `1e7f5b8fc09706d676cc1f7059ddffa7f6153c72e85d7ddda4e150c412e3b13a` |
| `gate_cmd` | `cargo test --workspace`, **`gate_cmd_fallback=true`**; a triage changes no code, so its gates are the kind gate, the round-trip control, `pmat work validate` and the sync's own verdict |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| model gate | `model=opus class=opus decision=admit basis=file` |

### Delegation, quoted verbatim

The GitHub writes below (labels, comments, a close, 37 issues opened) rest on the operator's instruction for this pass, quoted verbatim: "pmat-implement docs/specifications/goal-mode.md autonomously (you merge, triage tickets, pull requests, and label, tag, close/open)". The close of #1240 was made with that text as `mutate.sh close --operator` and a citation, because the quorum artifact's overall verdict is not a PASS (each lane disputed some row), though all three agreed on #1240.

### One ticket per session

Not re-attempted. `goal.sh set` refuses any ticket after PMAT-719 in this session (R-5), recorded on PMAT-721; the operator's instruction above is the reaffirmation, and no goal row exists for this ticket.

## What PMAT-1253 is

After PMAT-721, `pmat work sync --check-only` reported **0 COLLISION, 61 ORPHAN-ROADMAP, 54 ORPHAN-GITHUB** (live, 759 issues). CB-2115 cannot become a gate (PMAT-723) until that is zero on master. The spec's fixers, run blind, would have minted issues for work that already shipped and added roadmap items for epics. So the 115 rows were triaged first, with one rule per verdict, and the fixers ran on what remained:

| verdict | rule | rows |
|---|---|---|
| `completed` | the work is on master (a merged PR from its branch or naming it, or its deliverables verified in the tree) and its acceptance criteria are met | 18 |
| `pair` / `paired` | the item's own title names the orphan issue first | 6 + 6 |
| `open` | real remaining work: the issue fixer mints its issue | 37 |
| `no-roadmap` | labelled `epic`: a spec's container, not a ticket's issue (§4.3); exempt from G with the label, left open | 3 |
| `close` | fixed, with the merge that fixed it cited | 1 |
| `item` | real open work with no item: the item fixer adds `GH-<n>` | 44 |

## Universe (T-2)

| field | value |
|---|---|
| source | the sync's own findings over a live GitHub snapshot saved before any write (`/run/user/1000/paiml-implement/triage/PMAT-1253/github-snapshot.json`) and this branch's roadmap (verbatim in `snapshot.json`) |
| count | 115 |
| sha256 | `9bf0b7324f16aa24910e2d4bb58c3e5f57fa2e063727c18bd4cca9eb455c7448` |
| drift | the triage writes to both sides, so a re-run of the source after the writes differs by construction; the universe was frozen before the first write and every write is logged below |

## Plan and routing

| phase | what | class | `route.sh` line | taken? | trigger |
|---|---|---|---|---|---|
| 1 | evidence per row (merges, trailers, dispositions, issue bodies, CLI probes), the ledger, a three-lane quorum on it | review | `route=agy-quorum w=1.00 basis=absent effort=1[U]` | taken, width 3 | the kind:triage ledger review |
| 2 | adjudication, then the guarded apply (roadmap edits, GitHub writes, both fixers with dry-run counts asserted against the ledger) | orchestration | `route=self w=100.00 basis=absent` | taken | — |
| 3 | verdict, receipt, PR | orchestration | `route=self w=100.00 basis=absent` | taken | — |

## Dispatch ledger

| phase | mode | description | agent id | tool uses / duration | maxTurns hit | resumed | lane / width / agy `conversations` |
|---|---|---|---|---|---|---|---|
| 1 | delegate (`paiml-agy-delegate`, opus) → agy, `writes=false`, sandboxed | `PMAT-1253/ph1.delegate quorum width 3 on the 115-row sync-orphan triage ledger` | `adf41849ada6260d3` | 74 / 1,479 s | no | no | quorum (run as `--mode plan --schema quorum-schema.json`) / 3 / `736b169b-7b37-4d63-afc7-1b72e5607dba`, `aec32769-2bf1-4282-a518-30e4053b672c`, `06d78423-0393-4ee8-ac7e-e0d4cb644480` |

The harness flagged the delegate's output as instruction-shaped because it contains the agy calling form (`--dangerously-skip-permissions`) the skill documents for lanes. It was treated as data.

## Slots, denials, I-3

| field | value |
|---|---|
| `slots` | 3 |
| Claude subagents dispatched for this ticket | 1 (the delegate) |
| `transcript-gate.sh` (session-wide) | `PASS transcript-gate: attempted=15 denied=0 running_peak=2 slots=3 segments=434 files=14 (agent_calls=14 resumes=1 workflow_started=0; denied from hook log) (session 39a15c63-fab3-4148-8cb7-0cb012369b35, rule=pid-file (/run/user/1000/paiml-implement/pid-21623))` |
| denials | 0 |

## Quorum and adjudication (`docs/audits/quorum-PMAT-1253.json`)

All three lanes returned FAIL: each disputed at least one row. Each judged 43 rows (the 41 that take a row off the fixers' path, plus the two open rows carrying a `verify` field). The rows that changed, each on evidence:

| row | ledger said | lanes | now | settled by |
|---|---|---|---|---|
| MACS-017 | completed | completed 3/3 | **open** | the delegate measured it and I re-read it: `pmat agy sync` refuses, "agy sync is not implemented … nothing was written" (`src/cli/handlers/agy_handler.rs:55`). My evidence had been "the command parses", which a refusing command also does |
| MACS-018 | completed | open 2/3 | **open** | no parser for Anti-Gravity transcript bounds exists; lane 1 rested on a commit subject |
| MACS-019 | completed | completed 3/3 | **open** | `--agy` only selects a delegation target for the ledger record (`work_ledger_delegate.rs:52-60`); nothing translates contract requirements into define_subagent/invoke_subagent sequences |
| PMAT-707 | open | completed 3/3 | **completed** | `pmat analyze complexity --diff-scope` exists, and `dropping_the_flag_from_the_generated_hook_goes_red` and `complexity_gate_scopes_rust_to_the_diff_only` pass (26 BSE-12 tests, 0 failed) |
| PMAT-622 | open | open 2/3 | open | the variant is real, its dispatch is "not yet wired" (`work_falsification/runner.rs:258-264`); my evidence text was corrected |
| #999, #1031, #1034, #1035, #1090 | no-roadmap | no-roadmap 3/3 | **item** | by the rule's own condition: none is labelled `epic`, and their work is not filed as issues of its own (#1035 names #1017, #831 closed and #998 a merged PR; #1090 has one task filed, #1125). The delegate called the 3/3 "thin evidence" |

Agreed 3/3 and kept: the 13 merged-PR completions, PMAT-620 (the `sha_drift_*` tests settle its flag), PMAT-624, PMAT-629, PMAT-654 (`97d994f78`), the six pairs, the three epics and the #1240 close. Evidence text corrected without a verdict change: PMAT-694 (the 3.40.0 CHANGELOG does list #1202, under "Known, not fixed"). #1153 is an item, not an exempt tracker: only 7 of its 32 CRUX defects were ever filed on their own (#1146–#1152), so the recorded rationale "already filed individually" does not hold.

## Writes

Every GitHub write, in order:

1. `gh label create no-roadmap`. The first attempt returned HTTP 422 (the description was over 100 characters) and the guard stopped before any other write. Step 1 appends notes and is not idempotent, so the roadmap was restored from HEAD before the re-run.
2. `mutate.sh label` and `mutate.sh comment` on #1017, #1018 and #1019 (`no-roadmap`, each read back).
3. `mutate.sh close` on #1240, citing #1241 (`d3eef0198`) and #1242, with the operator's words as authority.
4. `gh label create` for the 11 item labels the issue fixer copies: compile-time-codegen, component-28, component-29, component-30, deferred:3.41.0, falsification-unification, kind:code, kind:measurement, proc-macro, provable-contracts, verification-ladder.
5. `pmat work sync --direction yaml-to-github` opened 37 issues, #1254 to #1290, each written back to its item:

| item | issue |
|---|---|
| MACS-006 | #1254 |
| MACS-007 | #1255 |
| MACS-013 | #1256 |
| MACS-017 | #1257 |
| MACS-018 | #1258 |
| MACS-019 | #1259 |
| PMAT-619 | #1260 |
| PMAT-621 | #1261 |
| PMAT-622 | #1262 |
| PMAT-623 | #1263 |
| PMAT-632 | #1264 |
| PMAT-633 | #1265 |
| PMAT-636 | #1266 |
| PMAT-638 | #1267 |
| PMAT-639 | #1268 |
| PMAT-646 | #1269 |
| PMAT-647 | #1270 |
| PMAT-653 | #1271 |
| PMAT-659 | #1272 |
| PMAT-677 | #1273 |
| PMAT-678 | #1274 |
| PMAT-681 | #1275 |
| PMAT-683 | #1276 |
| PMAT-684 | #1277 |
| PMAT-690 | #1278 |
| PMAT-692 | #1279 |
| PMAT-701 | #1280 |
| PMAT-702 | #1281 |
| PMAT-704 | #1282 |
| PMAT-706 | #1283 |
| PMAT-708 | #1284 |
| PMAT-709 | #1285 |
| PMAT-723 | #1286 |
| PMAT-725 | #1287 |
| PMAT-726 | #1288 |
| PMAT-727 | #1289 |
| PMAT-729 | #1290 |

6. `pmat work sync --direction github-to-yaml` added 44 items `GH-<n>` (no GitHub write); 30 of them carry `release:` projected from their milestone.

The roadmap side, in the same commit: 18 items `completed` with their evidence in `notes`, 6 paired, and PMAT-721 completed here because its own PR could not (CB-2113 refuses a trailer naming a terminal item).

## Verification table (claimed vs my rerun)

| check | result |
|---|---|
| live sync before | 115 findings: 0 COLLISION, 61 ORPHAN-ROADMAP, 54 ORPHAN-GITHUB |
| issue fixer dry-run vs ledger | 37 planned = 37 `open` rows (asserted before the real run) |
| item fixer dry-run vs ledger | 44 planned = 44 `item` rows (asserted before the real run) |
| live sync after | **exit 0, "coherent: R ↔ G is a bijection and no matched pair disagrees past the grace window"** |
| `work-sync-control.sh` (arm 4 round-trips the real roadmap) | 4/4 before and after the fixers |
| `pmat work validate` | exit 0 |

## Jidoka log

| # | defect | owner | whys |
|---|---|---|---|
| 1 | the `no-roadmap` label create returned HTTP 422 | me | GitHub caps a label description at 100 characters; the guard stopped before any write |
| 2 | #1252 went red on CB-2113 | me | PMAT-721's commits marked PMAT-721 completed; fixed in #1252 (`f5d8c148e`), completed here |
| 3 | MACS-017 and MACS-019 were first recorded completed on "the command parses" | me | a flag or command that parses and does nothing is a known class in this repository; the delegate measured the refusal |
| 4 | the minted issues' body reads "Created via `pmat work start --create-github`" | `create_github_issue_from_item` (`src/cli/handlers/work_handlers/core_handlers/github.rs`) | the sync reuses the work-start helper and its footer; a wording defect, not filed separately |
| 5 | the 3.39.0/3.40.0 dispositions were refuted on six rows (PMAT-620, 624, 629, 654 shipped; #1153 and #1035 not filed individually) | the audits' method | commit-subject matching, and a rationale copied forward without a check |

## Estimates

| field | value |
|---|---|
| `K̂` | 51, `basis=docs/audits/impl-estimates.jsonl:L19-L24` (`estimate.sh pmat 3`: `ROWS=6 MEDIAN=50.5`, `cycles=absent[U]`) |
| `K` | 102, andon line 82 |
| actual | **13** turns at this receipt (`k_measured` 434 minus 421 at branch creation) |

## Gaps

| gap | artifact that closes it |
|---|---|
| this PR is stacked on #1252 and red on CB-2113 until #1252 merges | merge #1252, rebase this branch onto master |
| CB-2115 is still withheld | PMAT-723 (#1286): measure `--check-only` = 0 on master after this merges, then add the step |
| the 37 minted issues carry no milestone, and their ids do not end in their issue numbers | PMAT-725 (#1287): release binding and the re-mint decision |
| epics are exempt by a label, not by the rule; goal-mode §5.1 does not say whether an epic is in G | a spec decision for the operator: exclude `epic` in `in_universe()`, or keep labelling |
| the quorum spot-checked 11 of the 74 open/item rows; the other 63 took the fixers' path unreviewed | reversible: an issue can be closed, an item completed |
| `pv` NotRun, `teamwork` grill NotRun | — (a triage) |

## Post-merge: drift on master

#1291 merged as `6df22b7f0`. The live `--check-only` on master's roadmap then read **15 findings**, not 0:

- **2 ORPHAN-GITHUB: #1292 and #1293.** Both are numbered after #1291 but were created on 2026-09-05 and 2026-08-13, so they were transferred into this repository after the triage. They are real work, and the item fixer gave them items.
- **6 release DRIFT.** The six paired train items had no `release:` while their issues sit on milestones; the item fixer projected them (§4.1: the sync is the only writer).
- **7 title DRIFT.** PMAT-1253 and the six train items have titles that differ from the issues they are paired with. The sync never rewrites a title, and PMAT-714 made roadmap titles immutable. The roadmap is authoritative for plan (§5.3), so each issue took its item's title; GitHub keeps the old one in the issue's timeline:

| issue | item | title before |
|---|---|---|
| #1253 | PMAT-1253 | goal-mode step 4a: pmat work sync --check-only reaches 0 on master — the fixers, run after |
| #1202 | PMAT-694 | flake: ci / coverage kills quality_proxy tests whose nested cargo clippy exceeds 600s unde |
| #1156 | PMAT-695 | build.rs downloads four assets from unpkg.com at build time, two pinned to @latest, into a |
| #1128 | PMAT-696 | Two Dependabot advisory gates ship; only one is wired, the orphan is RED, and 24 lines of  |
| #1124 | PMAT-697 | quality_proxy: gates_run claims complexity ran for bash, C and TypeScript, where the heuri |
| #1137 | PMAT-698 | The #1019 fix deleted the one .pmat-gates.toml section its own default reader consumes, an |
| #1141 | PMAT-699 | Two root pub mods nothing calls: src/protocol/ (2,047 lines) and src/state/ (3,896 lines)  |

The 60-minute grace window (§5.2) is why the check read coherent at 18:18: the pairings were written at 18:16 and surfaced as drift at 19:16. The retitles bumped the six issues' `updated_at`, which put their release disagreement back inside the window, so the item fixer planned no projection. It ran with `--grace-minutes 0`: projecting a milestone into `release:` is RR-RELEASE's own rule, not a tolerance call. The check itself ran with the default 60 minutes. After the fix, the live `--check-only` exits 0.

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 421
kind: triage
ledger: docs/audits/triage-PMAT-1253-ledger.json
rows_total: 115   rows_expected: 115
verdict_counts: roadmap:completed=18 roadmap:pair=6 roadmap:open=37 github:paired=6 github:no-roadmap=3 github:close=1 github:item=44
snapshot_sha: 9bf0b7324f16aa24910e2d4bb58c3e5f57fa2e063727c18bd4cca9eb455c7448
slots: 3   denials: 0
gh_commands: reads — gh issue view (#612 #1240 #1225 #1014 #1133 #1162 #1017 #1018 #1019 #1035 #1153 #999 #1031 #1034 #1090 #1202 #1156 #1127 #831 #998 #1254 #1286), gh issue list (open, CRUX, #1090 tasks), gh pr view (#1177 #1184 #1180 #1223 #1227 #1237 #1241 #1242 #1243 #1244 #1245 #1251), gh label list · writes — gh label create ×12, mutate.sh label ×3, mutate.sh comment ×3, mutate.sh close ×1 (#1240), gh issue create ×37 by pmat work sync --direction yaml-to-github (#1254–#1290)

routes:
  ph1  class=review         route=agy-quorum  w=1.00    basis=absent   effort=1[U]
  ph2  class=orchestration  route=self        w=100.00  basis=absent
  ph3  class=orchestration  route=self        w=100.00  basis=absent

verification:
  cmd=work-sync-check-only-before(live,759-issues)  claimed_exit=-  rerun_exit=1(115-findings)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/sync-1253-base.log  sha256=c6818eae95801936
  cmd=work-sync-yaml-to-github-dry-run(37-create-issue)  claimed_exit=-  rerun_exit=0  log_path=/run/user/1000/paiml-implement/triage/PMAT-1253/plan-y2g.json  sha256=37269295e144a645
  cmd=work-sync-yaml-to-github(37-opened)  claimed_exit=-  rerun_exit=0  log_path=/run/user/1000/paiml-implement/triage/PMAT-1253/run-y2g.log  sha256=def643ab2fd959b6
  cmd=work-sync-github-to-yaml-dry-run(44-create-item)  claimed_exit=-  rerun_exit=0  log_path=/run/user/1000/paiml-implement/triage/PMAT-1253/plan-g2y.json  sha256=17ea699bb5834a79
  cmd=work-sync-github-to-yaml(44-written)  claimed_exit=-  rerun_exit=0  log_path=/run/user/1000/paiml-implement/triage/PMAT-1253/run-g2y.log  sha256=b8688e52c68b35e2
  cmd=work-sync-check-only-after(live)  claimed_exit=-  rerun_exit=0(coherent)  log_path=/run/user/1000/paiml-implement/triage/PMAT-1253/check-after.log  sha256=6bedf907007cd691
  cmd=work-sync-control-before-fixers  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/control-1253-a.log  sha256=e50537b144d44676
  cmd=work-sync-control-after-fixers  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/control-1253-b.log  sha256=abb7d2e571cfbd13
  cmd=work-validate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/validate-1253.log  sha256=94efcd5db11f4719
  cmd=cargo-test-lib-bse12(26)@PMAT-707  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/bse12-tests.log  sha256=814d8e79cca583cb
  cmd=init-agy-skill-count(#1133)  claimed_exit=-  rerun_exit=0(1-skill)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/init-agy.log  sha256=0e3833589cde53de
  cmd=delegate-receipt-ph1(lane-reduce)  claimed_exit=3xFAIL  rerun_exit=0(adjudicated)  log_path=docs/audits/quorum-PMAT-1253.json  sha256=6932aa4d380097fb
  cmd=mutate-writes(label,comment,close)  claimed_exit=-  rerun_exit=0  log_path=/run/user/1000/paiml-implement/triage/PMAT-1253/mutations.jsonl  sha256=1f3f8060030b5b9e
  cmd=transcript-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-1253.log  sha256=1ace288d7e1f246b
  cmd=kind-gate(diff-files)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/kind-gate-1253.log  sha256=257391d00db44ff0
  cmd=model-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/model-gate-1253.log  sha256=44878c7c890a08c9

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

Finding — `k_measured` (434) is transcript-wide: this session carried seven tickets before this one. `k` (13) is this ticket's share, counted from the first transcript line naming its branch (421).

[status] ticket=PMAT-1253 phase=1/3 global=13/51(K=102) k_measured=434 sub=74/74 basis=docs/audits/impl-estimates.jsonl:L19-L24
         mode=quorum:quorum trigger=kind:triage-ledger route=agy-quorum w=1.00 basis=absent q=? gate=FAIL slots=1/3 denied=0
         red=- filed=- blocker=- next=adjudicate nine rows on evidence

[status] ticket=PMAT-1253 phase=2/3 global=13/51(K=102) k_measured=434 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L24
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=the guarded apply

[status] ticket=PMAT-1253 phase=3/3 global=13/51(K=102) k_measured=434 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L24
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=#1252-merge-order next=draft PR, merge #1252, rebase, ready

## Verdict

**DONE.** `pmat work sync --check-only` exits 0 against live GitHub on this branch: the open roadmap items and the open issues outside `no-roadmap` are in bijection. Nothing was fixed blind. Every verdict that removed a row from the fixers' path was reviewed by three lanes, and nine rows changed on evidence before any write. Every GitHub write was check-then-write or asserted against the ledger first. CB-2115 can become a gate once this is on master (PMAT-723).
