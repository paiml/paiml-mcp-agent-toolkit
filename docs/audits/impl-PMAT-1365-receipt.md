# impl-PMAT-1365 — receipt (fifth session)

Verdict at the judged head: **PARTIAL(blocker)**. `make gate` is declared, and discovery now finds it.
The branch is rebased onto master. Two things block the merge, and neither comes from this branch:

1. The required `traceability` job fails on CB-2115 `ORPHAN-ROADMAP PMAT-1366`. #1382 closed #1366 but left its row `planned`. The fix is in its own housekeeping PR, **#1384** (`chore(PMAT-1336)`), and is kept out of this diff (see Scope).
2. `make gate` is red locally on the CB-200 lib test (#1266), exactly as on clean master. It is not waived and not re-baselined.

PR #1368 is armed only when the quorum artifact on top of this commit is 3/3 PASS, #1384 has merged, and CI is green.

## Identity

| field | value |
|---|---|
| ticket | PMAT-1365 (`kind:code`, #1365) |
| branch | `PMAT-1365-declare-gate-land`, pushed as `PMAT-1365-declare-gate` (PR #1368) |
| base | `origin/master` `e89a827f7` (#1382 merged), behind = 0 |
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

Re-render evidence: `pmat analyze reachability --check-ledger` → 0; `pmat analyze unrun-tests --executed '' --check-ledger` → 0.

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

## Verification — re-run by the orchestrator on the rebased head `bfe4e7acd`

| claim | re-run |
|---|---|
| discovery finds the gate | `discover.sh` → `gate_cmd=make gate`, `gate_cmd_fallback=false` |
| `make gate` | 605 s, **26 PASS, 2 FAIL**: `lib-tests` (21690/21691 — only `tdg_baseline::tests::the_committed_baseline_is_the_measured_count`, CB-200 #1266) and `cb-2113-cb-2115` (CB-2113 ✓ 13 commits; CB-2115 ✗ `ORPHAN-ROADMAP PMAT-1366: #1366 is closed`). The CI-only rows are printed by name. |
| CB-2115 red is master's | master e89a827f7 carries `PMAT-1366 status: planned`; `gh issue view 1366` → CLOSED 2026-09-17T08:55:11Z. On #1384's tree: `pmat comply check --checks CB-2115` → ✓ 114/114, `pmat work sync --check-only` coherent |
| CI on `bfe4e7acd` (first read) | 22 pass, 1 fail (`traceability`, the same CB-2115 finding), 19 pending |
| the CB-200 red is master's (fourth session) | clean clone of `441d198e7`: 1742 below A against a baseline of 1688, the same as the branch |
| contract | `contracts/make-gate-v1.yaml`: `pv-obligations` leg PASS inside `make gate` |
| controls | `gate-control` leg PASS (`scripts/gate-control.sh`) |

## Scope

- **#1381 row: gone from this diff.** It was the fourth session's one blocking quorum FAIL. Master now carries it (#1382).
- **PMAT-1366 completion: not in this diff.** The fourth session's rule was: a foreign roadmap row that draws a scope FAIL comes out and must reach master on its own. #1380 (PMAT-1373) set the precedent for exactly this lifecycle gap. So the fix is #1384 (`pmat work sync --direction github-to-yaml`, one close-item), opened from this session because #1368 cannot pass `traceability` without it.
- Still in this diff: `PMAT-1365`'s own row gains `kind:code` (the label `kind-gate.sh` reads).

## Jidoka

| defect | owner | five whys → mechanism | disposition |
|---|---|---|---|
| feature-gate red on 1d8c64f40 (orphan-files ledger drift) | scripts/gate.sh | a new .rs file moved the ledger header count → no lib test covers that ledger → gate.sh marked the leg CI-only → the "cost" reason was never measured | fixed (fourth session): RED `d52bd0f82`, GREEN `01a67b512` |
| CB-2115 ORPHAN-GITHUB #1381 | roadmap lifecycle | an issue opened after master's last green has no row → CB-2115 is a bijection over open issues → every PR goes red | fixed on master by #1382; this branch's copy was dropped in the rebase |
| dependabot-alerts-live false credential reason | scripts/gate.sh | a CI-only reason is accepted as asserted, and `gh` already holds the token | fixed (fourth session): RED `d30834a1e`, GREEN `5161561f3` |
| CB-2115 ORPHAN-ROADMAP PMAT-1366 | roadmap lifecycle (PMAT-1336) | a merged PR closes its issue → a ticket cannot complete its own row in the PR that closes it → the row stays `planned` → CB-2115 reds every PR. The fifth instance of this gap in two days | **not fixed here**: #1384 |
| lib-tests red locally: CB-200, 1742 vs 1688 | src/services/tdg_baseline.rs (#1266) | debt below grade A was added on master → the lib test measures only where `.pmat/context.db` exists → CI checkouts have no index, so `ci / gate` passes it unmeasured | **not fixed, not waived**: #1266 |

## Dispatch ledger (fifth session)

No subagents were dispatched. transcript-gate.sh: `PASS attempted=0 denied=0 stalled=0 running_peak=0 slots=3`, which is vacuous. The pre-merge review is `quorum-review.sh` (three agy lanes), run on this commit. Its artifact is `docs/audits/quorum-PMAT-1365.json`, committed on top.

## Estimates

- `estimate.sh paiml-mcp-agent-toolkit 5` → `K_HAT=35 BASIS=docs/audits/impl-estimates.jsonl:L22-L31`, 14 pooled rows.
- The fourth session declared K̂=60 as `first-run[U]`. That was wrong: estimate.sh exited 2.
- `k_measured`: fourth session 104; fifth session 37 at the last measurement before this receipt was written. Sessions 1–3 were not measured.
- The row is therefore recorded with `unit: session` and is never pooled.

## Gaps

- Routing R-4 was not followed in any session: the implementation phases were done directly.
- CI green and the merge are not proven by this receipt. They wait on #1384 and on the rest of the checks.
- The fourth session never timed the `cost:` CI-only rows one by one. The fifth did not either.
