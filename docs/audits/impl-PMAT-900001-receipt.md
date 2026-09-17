# impl receipt — PMAT-900001 (PARTIAL: andon)

Verdict: **PARTIAL(andon)**. The turn budget ran out after discovery, the id-scheme blocker and the plan
grill, before any code phase. This receipt carries the plan, the grill's resolved decisions and the evidence,
so a relaunch with an explicit `--budget-turns` can start at Phase 2 and skip rediscovery.

## Identity

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

## Gaps

- Phases 2–6 not run: no code, no contract, no gate, no RED/GREEN evidence, no diff quorum, and no issue created (so CB-2115 on master is untouched).
- `make gate` row blocked on PR #1368.
- Out of scope, noted: aprender owns retiring its `no-close:` convention (aprender#3400); `--no-verify` in `auto_commit_work_files`; `pmat query --exclude-tests` still returns functions from `tests.rs` files.
