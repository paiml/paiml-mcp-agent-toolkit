# IMPL-PMAT-1363 — roadmap fragments: `save()` and every model writer emit `entries/<id>.yaml`; `pmat roadmap aggregate`

## Identity

| field | value |
|---|---|
| ticket | PMAT-1363 (#1363, `kind:code`; `kind-gate.sh` exit 0 `kind=code ticket=PMAT-1363` against `origin/master` — `--base master` exits 2 in this clone, which has no local `master` ref) |
| PR | #1364 (`feat/roadmap-fragments`), kept as the PR of record; pushed with `--force-with-lease` from local branch `PMAT-1363-roadmap-fragments` |
| session | fourth on this ticket; the second session's uncommitted tree (13 paths) was judged, kept, and committed as `223b973c7` |
| base | `origin/master` `441d198e7` for every measurement in this receipt; rebased onto `e89a827f7` (#1382) after master moved 11 commits, and re-verified there |
| HEAD out | `eef6881f8` (review fixes), then the commit carrying this receipt |
| `discover.json` sha256 | `b947e12a6fa69f268b371eb5bc0de3c61224799bb11b401d86d80db0132dd259` |
| `gate_cmd` | `cargo test --workspace` — **`gate_cmd_fallback=true`**; `pmat verify --format json` is the gate this repository's CLAUDE.md names, and is what was run |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| model gate | `model=opus-5 class=opus decision=admit basis=transcript` |
| I-3 | `PASS transcript-gate: attempted=4 denied=0 stalled=0 running_peak=1 slots=3` |
| `k_measured` | 183 (this session) |

## Scope, as landed

| # | item | verdict | evidence |
|---|---|---|---|
| 1 | `RoadmapService::save()` writes `entries/<id>.yaml` when `docs/roadmaps/entries/` exists, the whole file otherwise — the predicate of paiml/.github `sovereign-ci.yml` `roadmap-fragment-parity` (`[ -d docs/roadmaps/entries ]`) | DONE | M1 RED (4 tests); plan quorum 3/3 on the predicate's equivalence |
| 2 | `pmat roadmap aggregate [--write\|--check]`: idempotent, deterministic, supersede-on-duplicate, sorted insert; aprender's 14 selftest rows; contract obligations | DONE | `case_01..case_14`; M6, M7 RED; aprender `origin/main` `754c48225`: 3 runs byte-identical to each other, to the committed file and to the Python (sha256 `e1a984ee…`); 22/22 differential scenarios byte-identical; `--check` 0, and 1 naming `PMAT-3296` after a hand edit |
| 3 | `work add` refuses non-`PREFIX-N`; `work list` / `work status` read base ⊕ fragments | DONE | M2, M3 RED; binary e2e: six bad ids exit 1, zero fragments written |
| 4 | lock scope | CONFIRMED + TESTED | lock is `<git common dir>/pmat/roadmap-id.lock` (not keyed on `roadmap.yaml`); M4a (no lock), M4b (exclusive→shared), M5 (`--write` takes the read lock) RED after the lock tests learned a shared holder |
| 5 | PMAT-1370 (#1370) | LEAVES | plan quorum 3/3 `DECISION-1370: LEAVES`: its done-when needs changes only aprender and infra can make, and it names `sync` + `.pmat-work/`; recorded on #1370 with the one in-repo bypass all lanes found (`pmat work migrate`) |

Contract: `contracts/roadmap-fragments-v1.yaml` — `pv validate` valid; `pv status` 8 proof obligations, 8 falsification tests; `scripts/pv-obligation-gate.py` checks clean for it.

## Plan and routing

| phase | what | route | executor |
|---|---|---|---|
| 0 | kind/model/config gates, discovery | `route=self` | orchestrator |
| 1 | plan grill + PMAT-1370 decision | `route=agy-plan w=1.00 basis=absent effort=1[U]` | delegate, grillme width 3 |
| 2 | commit the kept tree; mutations M1–M7, M4a/M4b; real-data + differential + e2e; contract | `route=self` | orchestrator |
| 3 | `pmat verify` reds → fixes | `route=self` | orchestrator |
| 4 | pre-merge review, then re-review after fixes | `route=agy-quorum w=1.00 basis=absent effort=1[U]` | delegate, grillme width 3, twice |

## Dispatch ledger

| dispatch | agent | lanes / models | verdict | conversations |
|---|---|---|---|---|
| ph1 grill | `a079e1cfc69e8dbe4` | gemini-3.1-pro-high, gemini-3.8-flash-high, gemini-3.7-flash-high | 3/3 PASS, LEAVES ×3 | `fcc20791…`, `91eaeb01…`, `f8d39ecd…` |
| ph4 review | `ab02292b83d10b59c` (first brief refused: `mode=quorum` is not a lane mode — the orchestrator's error; resumed once with `mode=grillme`) | same three | 2 FAIL / 1 PASS | `087535e1…`, `44102525…`, `2f7e27cb…` |
| ph4b re-review | `ab8bcc479f6d09377` | same three | 3/3 PASS, `agreed=true` | `06f59625…`, `ba82b1f0…`, `1592bc60…` → `docs/audits/quorum-PMAT-1363.json` |

Slots: never more than one Claude subagent live; denials 0.

## Verification — claimed vs re-run

| claim | by | re-run here |
|---|---|---|
| lock test would fail without the lock | ph1 lanes 2, 3 (`measured`) | M4a: the three blocking tests RED, `concurrent_fragment_writers_lose_nothing` **green with no lock** — lane 3's claim false; the contract no longer counts that test |
| Rust diverges from the Python on accepted input | ph1 lane 1 | 22/22 differential byte-identical; all 11 real aprender fragments pass `check_fragment` |
| fragment names read by a second parser (`rsplit`) | ph4 lanes 1, 2 | real duplication, no collision (minted ids are exactly `PREFIX-N`); fixed to one reader `roadmap_text::id_number`; MX1 RED `left: Some(70) right: Some(50)` |
| `aggregate --write` not atomic | ph4 lanes 1, 2 | fixed (stage + rename); MX2 RED |
| refactors behaviour-preserving | ph4b 3/3 | `cargo test --lib roadmap` 341 passed; `pmat verify` 5/5 |

## Jidoka

| defect | owner | resolution |
|---|---|---|
| another session's build overwrote `target/debug/pmat` (commit `ae9914c2`, no `aggregate` subcommand): the `cargo()` shell function keys `CARGO_TARGET_DIR` on the origin remote name, so every clone shares one dir, and cargo's artifact hash ignores the checkout path | environment (shell function), not this repository | built with `command cargo` and a private target dir; reported, not fixed here |
| M5 survived the original `aggregate --write` lock test | `src/tests/roadmap_fragments_wiring_tests.rs` | every lock test holds the lock exclusive AND shared |
| `pmat verify` red: +54 `.unwrap()` and +1 `panic!(` over the CB-2102 ratchet, all in the two new `#[path]` test files (no `#[cfg(test)]` line for the awk scope) | this branch | `.expect(reason)` / `assert!(matches!)`; ratchet at baseline 20325 / 785 |
| `pmat verify` red: unrun-tests ledger rendered text drifted | this branch | regenerated from a clean tree |
| CB-2113: the commit hook warned that a `Pmat-Ticket` trailer naming a completed item is refused in CI | this branch | PMAT-1363's row stays `planned`; the next ticket's PR completes it |
| CB-200 (`the_committed_baseline_is_the_measured_count`) red locally: 1747/1748 below grade A vs baseline 1688 | **master** — a clean `origin/master` clone measures 1742, 54 over; this branch added 5 | the 5 refactored to grade A (branch = master = 1752 in the query index); master's 54 recorded on #1266 (PMAT-636, "unmeasurable in CI"), which is why it drifted unseen |
| touching `roadmap_text.rs` surfaced pre-existing `titles_by_id` cognitive 29 > 25 | master | split into `open_row` / `claim_title` |
| CI `traceability` → `work-sync-control` arm 4: a hand-written acceptance line was quoted where the serializer emits a plain scalar | this branch | unquoted; all 4 arms green locally. `pmat verify` does not run the CI control scripts — green there is not green here |
| andon threshold (0.8K = 96 turns, K=120) crossed while `pmat verify` was red, no draft/andon emitted | orchestrator | recorded; work continued to a green gate |
| at Phase 0 the estimate ledger carried two repo keys: `estimate.sh paiml-mcp-agent-toolkit` pooled nothing ("none enters a total — the writer and the reader disagree"), so K̂=60 was declared `first-run[U]`; `pmat` pooled 12 rows to K̂=35 | the ledger (PMAT-1366, #1382, merged during this ticket, unified the key and added `pmat work estimate record`) | this ticket's row was recorded through `pmat work estimate record` after the rebase; `pmat work estimate check` exit 0 |
| rebasing onto #1382 with `merge=union` on the estimates ledger kept both the old and the re-keyed copy of every row (62 lines, 19 violations) | `.gitattributes` `merge=union` meets a whole-file rewrite | master's ledger restored byte for byte, then the one row appended by the gated writer |

## Gaps

- `pmat work migrate` rewrites `roadmap.yaml` directly, unlocked, fragment-unaware — on #1370.
- The live aprender measurement is a recorded command, not a CI test: CI has no aprender checkout.
- `pmat work add --sequential-id` after `--id PMAT-7` minted `PMAT-009`: the pre-existing high-water mark stores "next" and is read as "spent" — a gap, never a collision; untouched here.
- Status-line join `tasks[].id` = hook `agent_id` and `transcript_path` on subagentStatusLine stdin: `[U]`, not measured.

## Estimates

K̂ 60 declared (`first-run[U]`; the ledger's split keys pooled nothing) · K̂ 35 from the 12 rows under the other key · K 120 · actual 183 (`k_measured` at the receipt; the rebase onto `e89a827f7` and the CI waits came after).

## Verdict

DONE at the PR: every scope item landed with RED before GREEN, `pmat verify` ok 5/5, quorum 3/3 PASS. Merge is armed from the CLI once CI is green; the merged sha is reported outside this file.
