# IMPL-PMAT-1296 — `pmat comply check --checks` selects before it runs: a group holding no selected rule is not run, its rules are still reported, and a drift test keeps the declared ids true

## Identity

| field | value |
|---|---|
| ticket | PMAT-1296 (`kind:code`), filed issue-first from #1296 by PMAT-723, so its id ends in its issue number; `kind-gate.sh`: `kind=code ticket=PMAT-1296 files=6` |
| spec | `docs/specifications/goal-mode.md` §7.2 prerequisite P1 (`--checks` selects rules by id, PMAT-718), §7 row CB-2113/CB-2115 (the traceability job's step) |
| branch | `PMAT-1296-checks-select-first`, worktree `/mnt/nvme-raid0/agent-wt/pmat-1296` (own target dir), rebased onto `6593cf309` |
| `discover.json` sha256 | `1e7f5b8fc09706d676cc1f7059ddffa7f6153c72e85d7ddda4e150c412e3b13a` |
| `gate_cmd` | `cargo test --workspace`, **`gate_cmd_fallback=true`**; `pmat verify` was run |
| model gate | `model=opus class=opus decision=admit basis=file` |

### Delegation, quoted verbatim

"pmat-implement docs/specifications/goal-mode.md autonomously (you merge, triage tickets, pull requests, and label, tag, close/open)". R-5 refuses any ticket after PMAT-719 in this session; that instruction is the reaffirmation.

## What was wrong

`handle_check` built the whole report (`compute_compliance_report_with`, all 21 rule groups), and only then did `select_checks` relabel the unselected rules as `Skip (not selected)`. The traceability job's `--checks CB-2113` step took **9.5 minutes** in CI (run 34508467866, 17:33:30 → 17:42:56) to report one rule. A local `--checks CB-2115` ran past a 6-minute tool limit, with codegen at 272 s and commit-enforcement at 205 s. Neither group holds a selected rule.

## The change

- **Every group declares the rule ids it can emit.** That is 162 CB ids, taken from the builders' `"cb-NNNN"` literals, plus the ten foundation rules that have no CB id and are selected by their whole name (`Version Currency`, `Config Files`, `Git Hooks`, `Quality Thresholds`, `Deprecated Features`, `Cargo.lock Present`, `MSRV Defined`, `CI Configured`, `PAIML Deps Workspace`, `Sovereign Stack Patterns`).
- **The selection reaches `run_check_groups`** through `CheckOverrides`. A group holding no selected id is not run. Instead it emits one row per declared rule, `CB-NNNN: not run`, Skip, with "not selected (--checks); its group `<name>` was not run", in its declared place. `select_checks` keeps that message.
- **With no `--checks`, every group runs,** exactly as before.

## RED, then GREEN

| step | commit | result |
|---|---|---|
| RED | `d47e7d1c6` | 3 passed, **2 failed**: all three fake groups ran under a one-id selection (`[1, 1, 1]` vs `[0, 1, 0]`), and a skipped rule reported `Pass` |
| GREEN | `dad49084c` | **80 passed, 0 failed** across the selection, traceability, ticket-release, spec-epic and roadmap-coherence tests |

**The drift test earned its place before GREEN.** `every_group_declares_every_rule_it_emits` runs every real group on a fixture crate. Its first run named the ten foundation rules with no CB id, which the literal scan had missed. Without them, `--checks "Version Currency"` would have skipped the one group that emits it.

## Measured

| command | before (master, all 21 groups run) | after (this branch) |
|---|---|---|
| `pmat comply check --checks CB-2113`, CI traceability step | 9.5 min (run 34508467866, 17:33:30 → 17:42:56) | measured on this PR's CI |
| `pmat comply check --checks CB-2113`, locally on this repository | over 6 min, killed by the tool limit | `CB-2113: exit=0 wall=0.1s rows=172 not_run=171` |
| `pmat comply check --checks CB-2113,CB-2115` (GH_TOKEN, live), locally | — | `CB-2113,CB-2115: exit=0 wall=6.4s rows=172 not_run=170` |

Every rule is still reported: 172 rows (162 CB ids + the ten whole-name rules), the unselected ones as `CB-NNNN: not run`, Skip.

The five controls, re-run with this branch's binary on the rebased tree: traceability-control exit=0; roadmap-coherence-control exit=0; work-sync-control exit=0; ticket-release-control exit=0; spec-epic-control exit=0.

## Mutation table

| mutant | killed by |
|---|---|
| M1-every-group-runs | `a_group_holding_no_selected_rule_is_not_run a_skipped_group_still_reports_each_rule_as_not_selected` |
| M2-skipped-rules-vanish | `a_skipped_group_still_reports_each_rule_as_not_selected` |
| M4-traceability-id-undeclared | `every_group_declares_every_rule_it_emits` |
| M5-not-run-reason-overwritten | `select_checks_keeps_the_reason_a_rule_was_not_run` |
| M3-case-sensitive-match | `a_group_holding_no_selected_rule_is_not_run` |

Every mutant compiled (`compile_errors=0`, a `test result: FAILED` line in each log). M3 was first planted against the pre-rustfmt text and did not apply (PATCH-FAILED); it was re-anchored to the formatted closure and re-run.

## Verification

`pmat verify --format json` at `af464ba99`: **ok: none** — format: pass; complexity: not applicable (no Rust files changed vs HEAD, so nothing was measured); satd: pass; clippy: pass; tests: pass. A null verdict is not a pass, so it is written down: the complexity stage judges only Rust files changed against HEAD, and a committed tree has none. Complexity was measured where it applies: the pre-commit hook passed it on each commit's staged change.

## Lifecycle

This PR marks PMAT-727 completed and closes #1289 in the same step. The close was made only after master `6593cf309`'s traceability job had passed (89 open items ↔ 89 open issues). Closing #1286 earlier, while `3f3602401`'s job was still queued, turned that commit red. That incident is recorded in memory and in the gaps below.

## Jidoka log

| # | defect | owner | whys |
|---|---|---|---|
| 1 | the first registry, built from the builders' `"cb-NNNN"` literals, missed ten foundation rules | me | those rules have no CB id and are selected by their whole name; the drift test named all ten before RED was committed |
| 2 | the first RED-stub run stopped at its own guard | me | `CheckOverrides` is also built in two test files; both got `..Default::default()` |
| 3 | M3 did not apply (PATCH-FAILED) | me | rustfmt had re-wrapped the closure. The original replacement, `s == id`, would not have compiled (`&String` vs `&&str`); re-planted as `s.as_str() == *id` |
| 4 | the first two-rule timing exited 1 | a stale tree | before the rebase the worktree still had PMAT-723 open while #1286 was closed; re-timed after the rebase |
| 5 | master `3f3602401` went red at its traceability job | me (PMAT-727's finish) | I closed #1286 before that commit's CI had read GitHub; this branch's finish refuses a close while master's newest traceability job is pending |

## Estimates

| field | value |
|---|---|
| `K̂` | 46, `basis=docs/audits/impl-estimates.jsonl:L19-L27` |
| actual | **19** turns at this receipt (`k_measured` 502 minus 483 at the first transcript line naming the branch); it includes waits on CI for #1297 |

## Gaps

| gap | artifact that closes it |
|---|---|
| the CI timing of the traceability step is measured on this PR, not yet here | this PR's traceability job |
| CB-2115 on pushes to master: any issue open or close, by anyone, turns master's next CI red until a PR catches the roadmap up (zero tolerance, live read); `3f3602401` went red this way | a spec decision for the operator: run the step on `pull_request` and `merge_group` only, give the set predicate a transition grace, or accept red master CI as the signal |
| a rule added to a builder without its id declared | `every_group_declares_every_rule_it_emits` fails and names it |

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 483

routes:
  ph1  class=impl           route=self  w=100.00  basis=absent   note=agy-goal not taken: four earlier writes lanes escaped their worktrees
  ph2  class=orchestration  route=self  w=100.00  basis=absent

verification:
  cmd=cargo-test-lib-RED@d47e7d1c6  claimed_exit=-  rerun_exit=101(2-failed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/red-1296b.log  sha256=1568eff1709eadbf
  cmd=cargo-test-lib-GREEN(80)@dad49084c  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/green-1296.log  sha256=181630f18674dbd1
  cmd=cargo-test-lib-pin-not-run-reason(6)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/green2-1296.log  sha256=f946f6a8d874d0cb
  cmd=timing-on-the-rebased-tree  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/time2-1296.txt  sha256=ad8296883fd0d527
  cmd=five-controls-with-this-binary  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ctl2-1296.txt  sha256=fa876b74f5745dbd
  cmd=pmat-verify  claimed_exit=-  rerun_exit=0(complexity-not-applicable-on-a-clean-tree)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify1296.json  sha256=e8fcc8b572e0fef1
  cmd=mutant-M1-every-group-runs  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut1296/M1-every-group-runs.log  sha256=87f1028331712701
  cmd=mutant-M2-skipped-rules-vanish  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut1296/M2-skipped-rules-vanish.log  sha256=4d0b44d800336fd8
  cmd=mutant-M4-traceability-id-undeclared  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut1296/M4-traceability-id-undeclared.log  sha256=19013363e39e240f
  cmd=mutant-M5-not-run-reason-overwritten  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut1296/M5-not-run-reason-overwritten.log  sha256=993dcb52d984ba1d
  cmd=mutant-M3-case-sensitive-match  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut1296/M3-case-sensitive-match.log  sha256=33297ecf666a8d1d
  cmd=transcript-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-1296.log  sha256=1ace288d7e1f246b
  cmd=kind-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/kind-gate-1296.log  sha256=29907c9759908e99
  cmd=model-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/model-gate-1296.log  sha256=44878c7c890a08c9

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

Finding — `k_measured` (502) is transcript-wide; `k` (19) counts from the first transcript line naming this branch (483).

[status] ticket=PMAT-1296 phase=2/2 global=19/46(K=92) k_measured=502 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L27
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=CI, merge

## Verdict

**DONE.** `--checks` now selects before it runs: a one-rule check no longer runs 21 groups, and every rule is still reported. A drift test keeps the declarations true, and it found ten rules the first registry missed. All six mutants die under named tests, and `pmat verify` passes every stage it measures on the committed tree.
