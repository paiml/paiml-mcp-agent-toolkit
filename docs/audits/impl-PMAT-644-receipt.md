# impl receipt — PMAT-644 (CRUX-12), written retroactively from PR #1157

| field | value |
|---|---|
| ticket | PMAT-644 (retroactive: the work predates the paiml-implement pass and ran through agy workers; the ticket was opened after the merge so the DoD ledger is complete) |
| item | CRUX-12 — spec §8.12 ; epic #1153 |
| PR | #1157 — `feat(reachability): a ledger and two ratchet metrics for the 489 files no build compiles` |
| branch | `fix/crux-12-reachability-ledger-ratchet` (head f48475be1) |
| merged | **yes** — merge commit cd6f796d6; head check-runs: success=42 skipped=4 (0 failure, 0 cancelled; no rerun) — verified by the orchestrator with `gh api` at receipt time |
| phase gate | as recorded in the PR body (below); `pmat verify` was not run by this orchestrator for this PR |
| DoD gate | CI required checks on the merge (`ci / gate`, `feature-gate`, `docs build`, `pmat score`, `provable ladder`) — all green on f48475be1 |
| quorum | none |
| subagents | agy workers (pre-skill); no worker receipt JSON exists, so every claim below is the PR body's, re-verified only where marked |

## Evidence carried from the PR body (claims, not orchestrator reruns unless marked ✔)

> Closes #1152 (CRUX-12). Population tracked on #1017.
> `pmat analyze reachability` has reported **407 orphaned `.rs` files** (126,933 lines, 6,292 `#[test]` fns) and **82 quarantined** (35,856 lines, 2,021 tests) since 3.34.0 — and nothing in CI, the Makefile or either hook ran it. A count 
> ## Four pieces
> 1. **`docs/status/orphan-files-ledger.md`** — one row per unreachable file, reason from a **closed enum the checker refutes**: `pending-#<issue>`, `quarantined-#<issue>`, `registered-<target>`, `deleted-<reason>`. `registered-pending` i
> 2. **Two ratchet metrics**, `orphan_files` (407) and `quarantined_files` (82), **measured in-process** via a new `analyzer` field on `MetricBaseline`. Not a `command` that spawns `pmat`: that resolves whatever is first on `$PATH` (CRUX-19
> 3. **`reachability-ledger` job** in `feature-matrix.yml`, in `feature-gate`'s `needs` *and* its failing loop.
> 4. **`scripts/reachability-ledger-audit.sh`** — the spec's acceptance test, with one correction: its awk separator `' *\| *'` is a plain `|` to gawk, so every reason read as `|`.
> ## Both sides
> | | result |
> |---|---|
> | master | `FAIL: leg 1a: no [metric.orphan_files] in .pmat-ratchet.toml` |
> | this branch | legs 1–7 **PASS**; stops at leg 8 (`298 orphan files still hold #[test] fns`) — the spec's #1017 completion criterion, not this PR's gate |
> | `pmat comply ratchet` | `orphan_files 407/407`, `quarantined_files 82/82` |
> | `--check-ledger` on the fresh ledger | current |
> 5 new ledger unit tests; every `metrics_ratchet` self-test green including the two that run the committed file; unrun-tests ledger regenerated.

## Verification by this orchestrator

| check | result |
|---|---|
| PR merged on the required checks, no rerun | ✔ `gh api pulls/1157` → merged=true, merge_commit_sha cd6f796d6; check-runs on f48475be1: success=42 skipped=4 |
| the gate the PR names exists in the tree | ✔ see the file list in the PR (`git show --stat cd6f796d6`) |
| named mutation RED | claim (PR body "both sides" table) — not re-run here |
| pv contract same PR | **NotRun** — no contract file for CRUX-12 under contracts/ (only pmat-book-build-v1.yaml matches the build/reach/config family); the artifact that closes it is a pv contract in a follow-up PR |

## Jidoka

As recorded in the PR body (a defect in the spec's own acceptance test for CRUX-06; an awk separator correction for CRUX-12). No new ticket from this receipt.

## Estimates

Not measured for this PR (pre-skill); basis: none. `docs/audits/impl-estimates.jsonl` carries no row for PMAT-644.

## Gaps

- Orchestrator reruns of the PR's acceptance script and mutation: NotRun — closed by re-running the script named in the PR against a release build of master.
- pv contract: see the table above.

## Verdict

DONE (merged green, no rerun) — with the gaps above named, not hidden.
