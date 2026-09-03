# impl receipt — PMAT-643 (CRUX-06), written retroactively from PR #1154

| field | value |
|---|---|
| ticket | PMAT-643 (retroactive: the work predates the paiml-implement pass and ran through agy workers; the ticket was opened after the merge so the DoD ledger is complete) |
| item | CRUX-06 — spec §8.06 ; epic #1153 |
| PR | #1154 — `fix(build): build.rs watched ../assets/demo/, a path outside the repo, so no build was ever incremental` |
| branch | `fix/crux-06-build-rs-stale-watch` (head 166964918) |
| merged | **yes** — merge commit b7f2bde62; head check-runs: success=41 skipped=4 (0 failure, 0 cancelled; no rerun) — verified by the orchestrator with `gh api` at receipt time |
| phase gate | as recorded in the PR body (below); `pmat verify` was not run by this orchestrator for this PR |
| DoD gate | CI required checks on the merge (`ci / gate`, `feature-gate`, `docs build`, `pmat score`, `provable ladder`) — all green on 166964918 |
| quorum | none |
| subagents | agy workers (pre-skill); no worker receipt JSON exists, so every claim below is the PR body's, re-verified only where marked |

## Evidence carried from the PR body (claims, not orchestrator reruns unless marked ✔)

> Closes #1149 (CRUX-06, the audit's top-ranked item).
> `build.rs:21` declared `cargo:rerun-if-changed=../assets/demo/` — a fossil of the `server/` layout deleted in January. Cargo treats a declared-but-missing watch as **permanently stale**, so the build script re-ran and the 1.1M-line crat
> ## Measured
> Release profile, same tree, same shared 48-core host:
> | | wall | CPU | peak RSS |
> |---|---|---|---|
> | **before** — no-op `cargo build --release` | 55.28 s | 263.5 s | 4.45 GB |
> | **after** — no-op `cargo build --release` | **0.27 s** | 0.17 s | 0.14 GB |
> Three consecutive no-ops after the fix: 0.28 s, 0.27 s, 0.27 s. One 57 s rebuild occurred between the first and second build after the fix; `find -newer` shows nothing in any watched path changed in that window, so its trigger is **not es
> ## Three changes
> 1. **The line is deleted.** The comment now says why and names the gate that guards it.
> 2. **A permanent gate** — `rerun_if_changed_paths_exist_inside_the_tree` in `build_support.rs`, under `cargo test --lib`. Every literal watch must be manifest-relative (checked by *shape* before existence, so `mkdir -p ../assets/demo` b
> 3. **`scripts/build-rs-watch-audit.sh`** — the full mutation-tested form for release evidence.
> ## RED-first, both sides
> | gate | pre-fix tree | fixed tree | defect re-planted |
> |---|---|---|---|
> | Rust test | — | `ok` | `FAILED: escapes the manifest dir: ../assets/demo/` |
> | audit script | `FAIL: leg 6: ESCAPES-MANIFEST-DIR:../assets/demo/` | `PASS` (leg 7 armed by a release build) | — |
> ## A defect in the spec's own acceptance test, found by implementing it
> The spec's hardened leg-5 mutation control **could never go green after the fix**: its mutants copied the live `build.rs` and assumed the defective line was still in it, so once the line was gone there was nothing for the auditor to rejec
> bashrs: 0 errors. `cargo fmt --check` clean.

## Verification by this orchestrator

| check | result |
|---|---|
| PR merged on the required checks, no rerun | ✔ `gh api pulls/1154` → merged=true, merge_commit_sha b7f2bde62; check-runs on 166964918: success=41 skipped=4 |
| the gate the PR names exists in the tree | ✔ see the file list in the PR (`git show --stat b7f2bde62`) |
| named mutation RED | claim (PR body "both sides" table) — not re-run here |
| pv contract same PR | **NotRun** — no contract file for CRUX-06 under contracts/ (only pmat-book-build-v1.yaml matches the build/reach/config family); the artifact that closes it is a pv contract in a follow-up PR |

## Jidoka

As recorded in the PR body (a defect in the spec's own acceptance test for CRUX-06; an awk separator correction for CRUX-12). No new ticket from this receipt.

## Estimates

Not measured for this PR (pre-skill); basis: none. `docs/audits/impl-estimates.jsonl` carries no row for PMAT-643.

## Gaps

- Orchestrator reruns of the PR's acceptance script and mutation: NotRun — closed by re-running the script named in the PR against a release build of master.
- pv contract: see the table above.

## Verdict

DONE (merged green, no rerun) — with the gaps above named, not hidden.
