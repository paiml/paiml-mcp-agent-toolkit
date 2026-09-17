# impl receipt — PMAT-900001

Verdict: **VERDICT_PENDING**. Session 1 ended in PARTIAL(andon) after discovery, the id-scheme blocker and the
plan grill. Its record is kept below. Session 2 resumed after a host crash and ran the code phases 2–6.

## Identity (session 1)

| field | value |
|---|---|
| ticket | PMAT-900001 (`kind:code`) |
| branch | `PMAT-900001-issue-closure-contract` |
| base | `origin/master` 85b798e0d (behind=0 at every measurement) |
| discover.json sha256 | a241850e33925470025fb62a106dccfff90a81df3aec11df5b9d70ed98476da8 |
| gate_cmd | `cargo test --workspace` — **gate_cmd_fallback=true** (no `make gate` until PR #1368 merges) |
| required checks (branch protection) | `ci / gate`, `feature-gate`, `docs build (docs.rs environment)`, `pmat score`, `provable ladder` |
| session model | measured `opus-5`, admitted class `opus` by `model-gate.sh` (basis=transcript) |
| GitHub issue | none yet, by decision D1 below |

## Budget (why andon)

`estimate.sh paiml-mcp-agent-toolkit 6` gave `K_HAT=35 BASIS=docs/audits/impl-estimates.jsonl:L24-L33`, so
K = 2×K̂ = 70 and the andon threshold is 0.8K = 56. `k_measured` was 55 (distinct assistant message ids in the
session transcript) when the grill returned, with no gate run (gate=NOT-RUN). The ticket's scope is four
deliverables (contract + call-site gate, emitter audit, commit/PR-body lint, a roadmap cross-reference), and the
per-ticket median of 35 turns does not cover them. Relaunch with `--budget-turns` set by the operator; a
larger K̂ has no basis in the ledger, so this receipt does not invent one.

## Plan (routing per phase)

| phase | what | route | acceptance |
|---|---|---|---|
| 1 | ticket, plan, footprint, plan grill | direct + `quorum:grillme` width 3 (Q1 \|M\|≥3, Q2 plan) | grill artifact below |
| 2 | `src/services/closing_keywords.rs`: find + neutralise, fixture table (RED `no-close: #3091`, `No-Close #12`, `this fixes #5`, `re-fixes: #7`, `fixes owner/repo#9`; GREEN `Closes #1`, `keeps-open #3091`, `prefix #12`, `fixture #3`) | agy-goal → sonnet-worker fallback | `cargo test --lib closing_keywords` |
| 3 | one bash lint snippet shared by all three commit-msg writers (hooks_command.rs template, services/hook_manager.rs, quality/git_hooks_install.rs); differential lib test running the snippet and the Rust predicate over one fixture table | sonnet-worker | `cargo test --lib closing_keyword` |
| 4 | emitters: work complete commit title interpolation neutralised; "Next: gh issue close N" advice replaced; `prompts/github-ticket.yaml` rewritten | sonnet-worker (disjoint scope from 3) | `cargo test --lib work_handlers::core_handlers::commit prompt` |
| 5 | `scripts/issue-closure-gate.sh` (pmat-query legs + non-indexed git-grep leg, anti-vacuity, `--self-test` planted mutants), ci.yml `traceability` steps (gate + PR-body lint), `contracts/pmat-issue-closure-v1.yaml`, `make gate` row if #1368 merged | direct | `bash scripts/issue-closure-gate.sh --self-test "$PMAT_BIN"` RED arms then GREEN; `pv validate` N obligations, K evaluated, 0 failed |
| 6 | PMAT-1369 cross-reference via `pmat work edit`; bind the issue with `pmat work sync --direction yaml-to-github`; diff quorum; merge | direct | 3/3 PASS artifact; CI green |

## Evidence (measured at 85b798e0d)

- E1 `pmat query --literal ".close_issue(" --format json` → `[]`. `close_issue` (src/services/github_client.rs:196) has in_degree 0; the module is `#[cfg(feature = "github-api")] pub mod github_client`, public API when that feature is on.
- E2 `pmat query --regex 'gh\s+issue\s+close|"issue"\s*,\s*"close"|closeIssue'` → 2 functions, both `println!` advice in src/cli/handlers/work_handlers/core_handlers/commit.rs (`print_complete_next_steps`, `auto_commit_work_files`). That is advice text, not a call.
- E3 `pmat query` indexes functions only. Rust `const` string bodies (the commit-msg hook templates), shell, YAML and Makefile are invisible to it. Measured: `--literal "set -euo pipefail"` returns 3 Rust functions and 0 scripts. Non-indexed emitter: prompts/github-ticket.yaml (`include_str!` at src/cli/handlers/prompt_handlers.rs:76) teaches a `(fixes #${ISSUE_NUMBER})` subject, a `Fixes #${ISSUE_NUMBER}` PR body, a hand `gh issue close`, and a Fixes/Closes/Resolves keyword list.
- E4 Non-test commit emitters: work complete (`feat: {item.title} (Refs {id})`, interpolates the free-text title, runs `git commit --no-verify`), roadmap `complete_task`, kaizen `commit_changes`, refactor `create_auto_commit`, unified_quality `commit_fixes`, `execute_split_plan`. Issue-body emitters: bug report `create_github_issue`, kaizen `create_github_issues`, work `create_github_issue_from_item`. There is no Rust PR-body emitter or `gh pr create` wrapper.
- E5 There are three independent commit-msg hook writers: `pmat hooks install` (CB-2113 trailer), `pmat work init` via services/hook_manager.rs (`Refs`), and quality/git_hooks_install.rs (conventional prefix, reached from tests only).
- E6 Last 3000 commits on origin/master, brief regex: 22 subjects match (16 parenthesised), 29 body lines match outside `Closes`-leading lines (narrative "closed #N"), 48 body lines start with `Closes`, and 0 merge subjects match. Commit 1cdffdcca records an earlier incident here: a PR body closed issue #1339.
- E7 Roadmap titles matching the regex: 0.
- E8 `pmat work add` refuses without `--github-issue N` or `--id ID` (#1240). CB-2112 (id tail must equal the issue number) is withheld from CI. CB-2115 (traceability job, needed by `gate`) is a bijection over open issues. Master: 113 items ↔ 113 issues, coherent. A scratch row with no issue produces 1 ORPHAN-ROADMAP, and `pmat work sync --direction yaml-to-github --dry-run` plans exactly one `create-issue` for it.
- E9 A cold `pmat query` index with the debug build of this tree took 3m08s (single-threaded). A warm query takes 2.4s.
- E10 PR #1368 (scripts/gate.sh extension point) is unmerged. The gh token has the `workflow` scope, and ci.yml edits merged before in #1334, #1316 and #1295.
- E11 `pmat work edit` has no `--notes`. `-d` replaces `acceptance_criteria` with one entry (ticket_crud.rs:292).
- E12 src/services/github_issues_client.rs is an uncompiled orphan file (docs/status/orphan-files-ledger.md:200, pending-#1017) that pmat query still indexes.

## Plan grill — dispatch ledger

| dispatch | mode | agent | turns | maxTurns hit | resumed |
|---|---|---|---|---|---|
| ph1 delegate | quorum `grillme` width 3, writes=false | paiml-agy-delegate `ae8295232758b603e` (opus) | 30 | **yes** | no. All 3 lanes and `lane-reduce.json` were already on disk, so the orchestrator read the artifact directly instead of spending a resume |

Lanes (agy conversations): lane 1 `gemini-3.1-pro-high` b4817262-edb7-4452-81e1-f10dc1db196d **FAIL**; lane 2
`gemini-3.8-flash-high` 9bb89658-b03c-46c9-85b8-6d4028b22bc3 **FAIL**; lane 3 `gemini-3.7-flash-high`
25bd14a9-da6a-4de8-bb23-5941cf2f3e1e **PASS**. lane-reduce: agreed=false, partial=false, author claude-opus-5.
Slots used: 1 of 3. Denials: 0. Stalls: 0.

## Grill resolutions (each FAIL answered with a fix or evidence)

| # | decision | lanes | resolution |
|---|---|---|---|
| D1 | ticket/issue order | L1 (c), L2 (b) + clarify red-until-bound, L3 (b) | **(b)**: id `PMAT-900001` outside GitHub's range, and the issue is created and bound by `pmat work sync --direction yaml-to-github` immediately before the final CI run. L1's premise ("never green") does not hold because the binding precedes the final run. This PR's traceability job is red (ORPHAN-ROADMAP) until then, as expected. Cost: CB-2112 TAIL-MISMATCH on an open row, and CB-2112 is not in CI |
| D2 | lint surface | L1 accept, L2 wants PR bodies, L3 accept | subject+body in every commit-msg hook, parenthesised `(closes #N)` refused, qualified `owner/repo#N` form added, **plus** a CI step linting the PR body (L2's fix: both incidents, aprender#3091 and #1339, came in through PR bodies) |
| D3 | one predicate, two renderings + differential test | all accept | as proposed |
| D4 | neutralising interpolated text | L1, L3 accept keeps-open; L2 semantic inversion | **L2's fix**: interpolated text gets `issue` inserted (`fixes issue #5`), which keeps the author's meaning. `keeps-open #N` is used only where pmat itself says the issue stays open |
| D5 | uncalled closer definitions | L1, L3 accept; L2 found github_issues_client.rs | keep (feature-gated public API). Definitions are counted apart from call sites. The definitions list names github_client.rs `close_issue`/`update_issue` and the orphan github_issues_client.rs `update_issue` (E12), and finding each named definition is the query leg's positive control |
| D6 | gate shape | L1 anti-vacuity, others accept | **L1's fix**: every leg fails when it scanned 0 files, or when a named definition is not found |
| D7 | work complete advice | all accept | replace "gh issue close N" with the Closes-line guidance |
| D8 | prompts/github-ticket.yaml | L1 "Closes #N violates (2)", L2/L3 accept | keep a `Closes #N` line. It is the single form the brief exempts, and this repository's merge flow writes it (E6: 48 lines) |
| D9 | other hook writers, `--no-verify` | L2: unify hook_manager | **L2's fix, narrowed**: the one lint snippet goes into all three writers. `--no-verify` stays: the work-complete commit names a completed item, which the CB-2113 hook refuses. It is recorded here |
| D10 | PMAT-1369 cross-reference | all accept | `pmat work edit PMAT-1369 -d "<existing entry> + upstream aprender#3397/#3398/#3399"`, the only tool path (E11) |

## Corrections to the brief

1. Local `master` did **not** track origin/master: it was 185 commits behind. It is now fast-forwarded to 85b798e0d.
2. `pmat work add` at HEAD cannot run "without creating the GitHub issue" in the plain form. It refuses to mint an id without `--github-issue` or `--id` (#1240), and CB-2112 ties the id's tail to the issue number. The quorum picked D1 (b).
3. `traceability` is not in branch protection's required contexts. It is required transitively, through the top-level `gate` job the org ruleset requires.
4. `pmat work edit` cannot write `notes:`. The PMAT-1369 cross-reference can only go in through `-d`, which replaces `acceptance_criteria`.
5. The session model is opus-5, not Fable.
6. `pmat query` indexes function bodies only, so a gate cannot be purely query-based: consts, scripts, YAML and prompts need a separate leg (E3).
7. The brief says no custom closer exists. That holds for call sites (E1, E2), but pmat does *advise* a hand close after `pmat work complete` and in `prompts/github-ticket.yaml`.

## Gaps (session 1)

- Phases 2–6 not run: no code, no contract, no gate, no RED/GREEN evidence, no diff quorum, and no issue created (so CB-2115 on master is untouched).
- `make gate` row blocked on PR #1368.
- Out of scope, noted: aprender owns retiring its `no-close:` convention (aprender#3400); `--no-verify` in `auto_commit_work_files`; `pmat query --exclude-tests` still returns functions from `tests.rs` files.

## Session 2 — identity

| field | value |
|---|---|
| resumed at | HEAD 75a9a28bd, 42 commits unpushed, 2 uncommitted files (core_handlers/commit.rs, core_handlers/github.rs), after a host crash |
| first action | the 42 commits pushed (WIP stashed so the pre-push fmt check judged committed code only), then Phase 0 re-run |
| discover.json sha256 | 57875a66fbcd2b2d6a82bc15e0d468b920bd14e0ebbd2711177f398f22cfffc4 |
| gate_cmd | `make gate`, **gate_cmd_fallback=false** (PR #1368 merged as ef2a0b947) |
| kind-gate / model-gate / config-lint / target-guard | kind=code · model=opus-5 class=opus decision=admit basis=transcript · slots=3 · PASS |
| base at every measurement | `behind=0` (origin/master ef2a0b947, then f25d7f1cc after merging #1392 at 7b1fc68f1) |
| build isolation | `CARGO_TARGET_DIR=/mnt/nvme-raid0/targets/pmat-D0 CARGO_BUILD_JOBS=8`; one heavy job at a time; no `./target` was left in the clone after any commit |

## Session 2 — phases, routing, commits

| phase | what | route.sh printed | executed | commit |
|---|---|---|---|---|
| 2 | predicate + shared shell snippet (session 1 WIP, committed before the crash) | — | direct | 75a9a28bd |
| 3+4 | three commit-msg hook writers splice the snippet; 6 commit builders and 4 issue builders neutralise; work complete message/advice; github-ticket prompt | `route=agy-goal w=1.00 basis=absent` | **direct** | e0f1ad8e4 |
| 5 | `scripts/issue-closure-gate.sh`, `scripts/pr-body-closing-lint.sh`, contract, ci.yml + pr-checks.yml steps, `scripts/gate.sh` row | `route=agy-goal w=1.00 basis=absent` | **direct** | ce0209730 |
| 6a | `pmat work edit --notes`; PMAT-1369 cross-reference written by this tree's binary | — | direct | 0943bc9b3 |
| 6b | merge origin/master (#1392) | — | direct | 7b1fc68f1 |
| 6c | CB-200 share and ledgers found by `make gate` | — | direct | 65debfcaa, 3c791bcfa |

Routing deviation, named: route.sh printed `agy-goal` for the implementation phases. They ran direct. The
crash-resume brief set load discipline (one heavy job at a time) and the phases shared files with the
uncommitted WIP the crash left, so a writing lane in a linked worktree would have had to be declared a
concurrent scope over the same paths. No Claude subagent and no agy lane ran for code: slots used 0 of 3 in
phases 2–6a, denials 0, stalls 0.

## Session 2 — RED before GREEN (every row re-run by the orchestrator)

| claim | RED (mutant or pre-fix tree) | GREEN (as committed) |
|---|---|---|
| work complete neutralises its title | raw `item.title` interpolated → `work_complete_commit_message_neutralises_a_closing_title` FAILED | passes |
| github-ticket prompt teaches only `Closes #N` | origin/master's prompt restored → `the_github_ticket_prompt_teaches_only_the_sanctioned_close` FAILED | passes |
| all three hook writers refuse exactly the closing fixtures | lint call dropped from hook_manager's template → FAILED naming `hook_manager` and the `no-close` fixture | passes |
| (the three mutants above, one build) | `cargo test --lib closing_keyword`: 8 passed, 3 failed | 11 passed, 0 failed |
| no pmat code path can close an issue | origin/master's commit.rs + prompt restored → gate FAIL: 2 CALL SITEs (`print_complete_next_steps`, `auto_commit_work_files`), 3 TEXT HITs | gate PASS: definitions 4/4, call_sites 0, text hits 0 (installed pmat and this tree's binary alike) |
| the gate can fail (self-test) | 6 arms, each exit 1 naming its own plant: rust-call, gh-args (`CALL SITE src/lib.rs:13 finish`), script, rust-const (`TEXT HIT`), definition-renamed (`BLIND`), vacuous (`VACUOUS: 0 Rust files`, `DEFINITION GONE`) | clean arm exit 0 |
| a first-draft query pattern was over-broad | `Some("closed")` as a literal matched a READ in `work_sync/github.rs parse_snapshot` → CALL SITE (false positive) | narrowed to argument position; PASS; self-test unchanged |
| the step row runs the built pmat | `scripts/gate-control.sh` arm 12 RED: `legs never ran the pmat cargo built from the tree: issue-closure` (the flag came before PMAT_BIN, so the control's stubs never reached pmat) | PMAT_BIN moved to `$1`; gate-control GREEN, arm 12 counts 13 pmat legs |
| PR title/body lint | `--self-test`: `no-close: #3091`, `No-Close #12`, `this fixes #5`, `re-fixes: #7` exit 1; an unreadable PR (`--pr 999999999`) exits 2 | `Closes #1`, `keeps-open #3091`, `prefix #12`, `fixture #3` exit 0; PR #1391's live body exit 0 |
| `pmat work edit --notes` | flag parsed and ignored → `work_add_append_only_edit_notes_sets_notes_and_keeps_the_criteria` FAILED ("--notes must reach the row") | passes; append-only + refuses-invalid suites 12 passed |
| hooks in this clone refuse a close | this tree's `pmat hooks install --strict --force`, then `.git/hooks/commit-msg` on 6 messages: `this fixes #5`, `no-close: #3091`, `(closes #8)` exit 1 | `Closes #1`, `keeps-open #3091`, a `Merge remote-tracking branch` subject exit 0 |
| `make gate` at 7b1fc68f1 (behind=0) | **RED: 25 PASS, 4 FAIL**. `issue-closure` PASSED (21 s). (a) `lib-tests` 21753/21755. One failure is CB-200 `the_committed_baseline_is_the_measured_count`, 1741 below A against baseline 1688; it is master's #1266 (measured 1741–1742 on clean master by the PMAT-1365 and PMAT-1363 receipts). The other is the unrun-tests ledger text. (b) `cb-2113-cb-2115`: CB-2113 ✓, CB-2115 ORPHAN-ROADMAP PMAT-900001 (no issue, by D1 until it is bound) and ORPHAN-GITHUB #1393 (opened on master today with no row; not this ticket's to carry). (c, d) `unrun-tests` and `reachability-ledger`: count-only drift from this branch's own additions | this branch's own share fixed: grading the changed Rust files before and after found **one** new below-A definition (`auto_commit_work_files` B+). 65debfcaa brings it to A- (the lint moves into `ready_to_commit`, A+; a mutant disabling the lint turns `ready_to_commit_refuses_a_closing_message_before_staging` RED). 3c791bcfa re-renders both ledgers (+1 tracked .rs file, +13 executed lib tests); `--check-ledger` exits 0 for both |

Contract `contracts/pmat-issue-closure-v1.yaml`: `pv validate` 0 errors, 0 warnings; `pv status` 8 equations,
**8 proof obligations, 8 falsification tests; 8 evaluated (every `test:` command run above), 0 failed**.
`scripts/pv-obligation-gate.py`: 0 problems over 40 contracts. `cargo test --lib make_gate_tests closing_keyword`: 17 passed.

## Session 2 — measurements that changed a decision

- **PR-body history.** Over the last 60 merged PRs, 15 would be refused by the lint. The hits are narrative
  close references ("merged and closed #1373") and titles ending `(closes #N)`. GitHub reads them as closes. For
  #1380 `closingIssuesReferences` lists 1373, and for #1377 it lists 1371. A title closes through the merge commit
  that carries it. The lint therefore stays strict, and the refusal says how to write each intent.
  The scan is at `.pmat/d0/s2/pr-body-scan.txt`, not committed.
- **`--no-verify` in `pmat work complete`.** It cannot simply be dropped. Its message carries `(Refs ID)` and
  no `Pmat-Ticket:` trailer. This clone's strict hook exits 1 on it ("no Pmat-Ticket trailer in the message's
  LAST paragraph", measured), and CB-2113 also refuses a trailer that names a completed item. The fix is in the
  commit path instead: `auto_commit_work_files` runs the same predicate in-process on the exact message and
  does not commit a closing one. So the commit no hook sees is still linted.
- **`pmat query --exclude-tests`** returned 2 inline `#[test]` functions, so the gate runs its own
  attribute check.
- **CB-2115 reads `github_issue` only** (`work_sync` `find_item_by_github_issue`). A cross-repository reference
  in `notes:` cannot be taken for an issue of this repository.

## Session 2 — where each check runs

| check | local | required CI |
|---|---|---|
| call-site gate + self-test | `make gate` row `issue-closure` (kind `step`: it runs the ci.yml step text verbatim, `./target/debug/pmat` rewritten to `$PMAT_BIN`) | ci.yml job `traceability`, step "control — no pmat code path can close a GitHub issue (PMAT-900001)". The top-level `gate` job `needs: [ci, windows-check, reusable-pin-drift, roadmap-validate, traceability]`, and `gate` is the org-ruleset context |
| PR title/body lint | `scripts/pr-body-closing-lint.sh --pr N` | same job, `if: github.event_name == 'pull_request'` (live body through gh). pr-checks.yml re-runs it on `edited` and is not required. This is not a `make gate` leg because the step needs `${{ }}` |
| predicate, hooks, emitters, prompt, `--notes` | `cargo test --lib closing_keyword` / `edit_notes` | `ci / gate` lib tests |

## Session 2 — five whys (to a mechanism)

1. Why could pmat close an issue nobody asked it to close? It wrote a closing keyword next to `#N` into text
   that reaches the default branch: commit subjects built from roadmap titles, a prompt that taught
   `(fixes #N)`, and advice to run `gh issue close`.
2. Why did pmat write that text? The emitters interpolate free text pmat did not author, and nothing judges
   that text before `git commit` or `gh issue create`.
3. Why did nothing judge it? No predicate for "this text closes an issue" existed. Each of the three
   commit-msg hook writers enforced only its own ticket or format rule.
4. Why did no predicate exist? GitHub's rule is wider than a reader expects. It accepts any tense, an
   optional colon, and no word boundary (`no-close: #3091`), so a human-readable rule was never written down.
   Every incident (aprender#3091, #1339) looked like a one-off.
5. Why did no incident force one? The close is a side effect of a merge. The emitting code, the merged text
   and the closed issue sit in three places, and no gate joined them.

**Mechanism now:** one predicate (`closing_keywords::PATTERN`), rendered in Rust and in one shell snippet that is
proven equal by a differential test. The emitters neutralise their output, every hook writer and CI refuse a
close outside a `Closes #N` line, and a call-site gate with planted-defect arms keeps pmat from closing an issue itself.

## Corrections to the session-2 brief

1. The brief said HEAD e03760258. The tree at resume was 75a9a28bd with 42 unpushed commits, as the crash note says. They were pushed first.
2. "6 commit builders and 3 issue-body builders": there are **4** issue builders. `test_discovery_handlers_tickets.rs create_github_issue` is the fourth, and it is fixed too.
3. `--no-verify` in `pmat work complete` stays, measured above. The fix lints that commit's message in-process instead.
4. D10 revised: PMAT-1369's cross-reference went into `notes:` through a new `pmat work edit --notes` (RED→GREEN). `-d` would have replaced its acceptance criteria.
5. Only the call-site step can be a `make gate` row. The PR-body step needs `${{ }}` and a pull_request event.
6. PMAT_BIN must be the first argument of any control script `scripts/gate.sh` runs. gate-control arm 12 enforces this through its stubs, which is how the first draft was caught.
7. The PR-body lint would have refused 15 of the last 60 merged PRs. That is the measured cost of the rule, and the hits are real GitHub close references.

## Session 2 — dispatch ledger, quorum, issue, merge

| dispatch | mode | executor | result |
|---|---|---|---|
| diff quorum round 1 on b36d579dc (diff_sha256 eb7be0ce…) | `quorum-review.sh --base origin/master --ticket PMAT-900001 --pr 1391 --author-model claude-opus-5`, width 3, writes=false | agy lanes: gemini-3.1-pro-high `2e151ff0-07cc-45f4-9005-d2bbdad4f762` **FAIL** · gemini-3.8-flash-high `9cee6cc3-0a07-4b55-9aa5-dd57f18d9363` PASS · gemini-3.7-flash-high `b15c53bb-abaf-49a2-8fea-e03bd5ac82e6` PASS (model_measured equals the declared model on all three) | NOT AGREED. The single finding (lane 1, cited `src/cli/commands/work_commands_work.rs:157`): `--notes` is not asked for by the ticket |

Claude subagents this session: 0 (slots used 0 of 3, denials 0, stalls 0). The quorum ran through
`quorum-review.sh` from the orchestrator's own turn and not through the delegate. The merge helper reads that
script's artifact, and one script call cost fewer turns than briefing a delegate. This is a named deviation from §6.3.

**Round 1 FAIL, answered with evidence and a fix, not an override.** Lane 1 is right that the ticket row did not
name the capability. The row's acceptance criteria, written in session 1, left out item 4 of the operator's brief,
which reads verbatim: "PMAT-1369 cross-reference to aprender#3397/#3398/#3399 — `pmat work edit` cannot write notes,
so either add that capability RED→GREEN if it is small, or record the cross-reference where the sanctioned writer
CAN put it and say which; never hand-edit roadmap.yaml". Commit `chore(PMAT-900001): the ticket row names brief item 4`
adds that item to the row through `pmat work edit -d`, keeping the old criterion text verbatim. The capability stays.

**Order of the last steps.** `pmat-merge` accepts a verdict only for the head, for its parent when the head commit
adds only the verdict, or for an identical judged diff. Binding the issue rewrites `github_issue:` and changes the
diff. So the issue was bound after round 1 and before round 2, on a PR whose checks were all green except
`traceability`. That job's code steps passed in CI (issue-closure-gate self-test and PASS on a cold index, PR-body
lint PASS). Its one red step was CB-2115: ORPHAN-ROADMAP PMAT-900001 (unbound by design) and ORPHAN-GITHUB #1393
(master's). Round 2 judges the final diff.

**Issue.** `pmat work sync --direction yaml-to-github` planned 2 actions: `create-issue PMAT-900001`, and `skip #1393`
("the fix is on the roadmap side"). It opened **#1395** and wrote `github_issue: 1395`. The issue's title and body
pass `pr-body-closing-lint --file`. The PR references the issue as `Refs #1395`, not with a Closes line. Closing
it on merge would leave the row `planned` against a closed issue, the ORPHAN-ROADMAP that 1cdffdcca recorded. The
row and the issue are completed together by a later lifecycle PR, as this repository does.

QUORUM2_AND_MERGE_PENDING

## Gaps

- Rust `const` hits after a mid-file `#[cfg(test)]` module are skipped by the text leg. Function bodies there are still covered by the query leg. The gap is named in the contract.
- ci.yml does not trigger on `edited`. If the body is edited after the last CI run, the required check keeps the verdict of that run until the job is re-run. pr-checks.yml gives a fresh, non-required signal.
- A PR body is Markdown. GitHub's treatment of close references inside code spans was inconsistent across #1342, #1390 and #1323, so the lint judges code spans too. It is the stricter side.
- Out of scope, as in session 1: aprender owns retiring its `no-close:` convention (aprender#3400).
