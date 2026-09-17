# impl-PMAT-1365 — receipt (fifth session)

Verdict at the judged head: **PARTIAL(blocker)**. `make gate` is declared, and discovery finds it.
The branch is up to date with master `8915fe3e6`. One thing blocks the merge, and it does not come from this branch:

1. The required `traceability` job fails on CB-2115 with two findings. `ORPHAN-ROADMAP PMAT-1363`: #1363 closed at 09:51Z with its row still `planned`. `ORPHAN-GITHUB #1385`: an issue opened at 09:57Z with no row. Both are master's lifecycle drift. The orchestrator's housekeeping PR **#1383** (`chore(PMAT-1336): register PMAT-1385 and complete PMAT-1363`) carries the fix, and it is kept out of this diff (see Scope).

`make gate`'s CB-200 lib test, red on `bfe4e7acd` (#1266), **passed** on the merged head `be34454e7`.

**Next step, exactly:** once #1383 is on master, run `git merge origin/master` on this branch and re-render the two ledgers only if `--check-ledger` fails. Push, then run `bash ~/.claude/skills/quorum-review/pmat-merge 1368 --auto --merge`. That refuses if the diff changed from the judged one; if it did, re-run `quorum-review.sh --base master --ticket PMAT-1365 --pr 1368` first.

## Identity

| field | value |
|---|---|
| ticket | PMAT-1365 (`kind:code`, #1365) |
| branch | `PMAT-1365-declare-gate-land`, pushed as `PMAT-1365-declare-gate` (PR #1368) |
| base | rebased onto `origin/master` `e89a827f7` (#1382), then `git merge origin/master` `8915fe3e6` (#1364) at `896a9bdff` because master's protection is strict; behind = 0 |
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
- **Foreign roadmap rows: none in this diff.** The fourth session's rule was: a foreign roadmap row that draws a scope FAIL comes out and must reach master on its own. #1380 (PMAT-1373) set the precedent for this lifecycle gap. This session opened #1384 for PMAT-1366; master fixed that row itself in #1364, so #1384 was closed. PMAT-1363 and #1385 are #1383's.
- Still in this diff: `PMAT-1365`'s own row gains `kind:code` (the label `kind-gate.sh` reads).

## Jidoka

| defect | owner | five whys → mechanism | disposition |
|---|---|---|---|
| feature-gate red on 1d8c64f40 (orphan-files ledger drift) | scripts/gate.sh | a new .rs file moved the ledger header count → no lib test covers that ledger → gate.sh marked the leg CI-only → the "cost" reason was never measured | fixed (fourth session): RED `d52bd0f82`, GREEN `01a67b512` |
| CB-2115 ORPHAN-GITHUB #1381 | roadmap lifecycle | an issue opened after master's last green has no row → CB-2115 is a bijection over open issues → every PR goes red | fixed on master by #1382; this branch's copy was dropped in the rebase |
| dependabot-alerts-live false credential reason | scripts/gate.sh | a CI-only reason is accepted as asserted, and `gh` already holds the token | fixed (fourth session): RED `d30834a1e`, GREEN `5161561f3` |
| CB-2115 ORPHAN-ROADMAP PMAT-1366 | roadmap lifecycle (PMAT-1336) | a merged PR closes its issue → a ticket cannot complete its own row in the PR that closes it → the row stays `planned` → CB-2115 reds every PR. The fifth instance of this gap in two days | fixed on master by #1364 (`df6c351b2`); #1384 closed as redundant |
| CB-2115 ORPHAN-ROADMAP PMAT-1363, ORPHAN-GITHUB #1385 | roadmap lifecycle (PMAT-1336) | the same gap, plus an issue opened after master's last change → every PR red again, minutes after the previous fix | **not fixed here**: #1383 |
| lib-tests red locally: CB-200, 1742 vs 1688 | src/services/tdg_baseline.rs (#1266) | debt below grade A was added on master → the lib test measures only where `.pmat/context.db` exists → CI checkouts have no index, so `ci / gate` passes it unmeasured | **not fixed, not waived**: #1266. Passed in `make gate` on the merged head `be34454e7`; not investigated why |

## Dispatch ledger (fifth session)

No subagents were dispatched. transcript-gate.sh: `PASS attempted=0 denied=0 stalled=0 running_peak=0 slots=3`, which is vacuous. The pre-merge review is `quorum-review.sh` (three agy lanes), run on this commit. Its artifact is `docs/audits/quorum-PMAT-1365.json`, committed on top.

## Estimates

- `estimate.sh paiml-mcp-agent-toolkit 5` → `K_HAT=35 BASIS=docs/audits/impl-estimates.jsonl:L22-L31`, 14 pooled rows.
- The fourth session declared K̂=60 as `first-run[U]`. That was wrong: estimate.sh exited 2.
- `k_measured`: fourth session 104; fifth session 37 at the first receipt commit and 64 at this one. Sessions 1–3 were not measured.
- The row is therefore recorded with `unit: session` and is never pooled.

## Gaps

- Routing R-4 was not followed in any session: the implementation phases were done directly.
- Not merged. CI green and the merge wait on #1383 reaching master (see Next step).
- The fourth session never timed the `cost:` CI-only rows one by one. The fifth did not either.
