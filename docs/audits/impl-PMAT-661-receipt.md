# Implementation receipt — PMAT-661 (AD-07: `pmat work link` and the Pmat-Ticket trailer check)

Spec: `docs/specifications/agentic-delivery-pmat.md` §8 / §9.7. Epic #1153. Branch `PMAT-661-work-link-trailer-check`.
Routing: `subagent:opus` (|M| ≥ 2: models, work commands, comply checks). No quorum lane at implementation time; the AD-04 review runs on the PR.

## Dispatch ledger

| dispatch | outcome |
|---|---|
| worker ab19c7c5b3a0475fa, brief `pmat-release/brief-PMAT-661.txt` | 40 turns of reading, nothing written |
| resume (the one permitted after a turn limit; brief cut to an ordered write list) | implemented and committed df2845893 (CB-1340, `work link`, annotate Links, dispatch, roadmap field) then hit the limit again before the gate and the receipt — no receipt JSON, so `gate` is the orchestrator's measurement below |
| slot | the paused worker kept the session's subagent slot, which blocks every push; the orchestrator removed its own session's stale slot after confirming no worker process was alive and the work was committed |

## Verification (orchestrator runs)

| check | result |
|---|---|
| RED on pmat 3.36.0 (`scripts/ticket-trailer-audit.sh`) | six ✗: CB-1340 absent from `comply check --format json`; `work link` does not parse |
| GREEN on this tree (binary from `cargo build --message-format json`, carries CB-1340) | six ✓: trailered branch passes; untrailered commit fails naming its sha; completed ticket fails; default branch passes; `work link` records a commit and a PR; `annotate` shows the commit — exit 0 |
| targeted lib tests (`check_ticket_trailer`, `work_link`) | 4 passed, 0 failed |
| named mutation (line 107: any known ticket accepted regardless of status) | RED — only the completed-ticket leg went red (`pass 1 commit(s) … carry a Pmat-Ticket trailer`); reverted and REBUILT before any further measurement |
| `pmat verify --format json` on df2845893 | format, satd, clippy ok; tests red on exactly one test, `the_committed_ledger_matches_the_tree` (the new lib tests were not yet in the ledger) — ledgers regenerated as the last commit; re-run recorded in the PR body |
| complexity on the changed files, direct | `analyze complexity --max-cyclomatic 30 --max-cognitive 25 --fail-on-violation --files <14 changed files>` exit 0; a 3/2 control exit 1 |
| pv contract | `contracts/ticket-trailer-v1.yaml` — `pv validate` and `pv lint` PASS |
| ratchet literals | `#[allow(` 497, `panic!(` 781, `.unwrap()` 20336 — all at baseline |

Verdict: **DONE** once the PR merges green; the worker's implementation, every claim re-measured by the orchestrator.
