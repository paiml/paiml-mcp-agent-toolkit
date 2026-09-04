# Implementation receipt — PMAT-664 (AD-10: the four lane modes)

Spec: `docs/specifications/agentic-delivery-pmat.md` §6 / §9.10. Epic #1153. Branch `PMAT-664-lane-modes`.
Bundle side: paiml/paiml-implement#10 (`skills/paiml-implement/scripts/agy-lane.sh`, `agy/goal-schema.json`, `agy/grillme-schema.json`, `agents/paiml-agy-delegate.md`, `SKILL.md` Phase 3, `install.sh`, `verify.sh`). Routing: direct.

## Verification (orchestrator runs)

| check | result |
|---|---|
| RED (before) | agy 1.1.25 has no `/goal` or `/grillme` (AIS-006) and nothing composed a per-mode calling form: a lane's mode was prose in a brief |
| GREEN — `agy-lane.sh --self-test` | ten ✓: unknown mode refused; no prompt refused; **each mode's `--dry-run` prints its calling form** (teamwork prefixes `/teamwork-preview` and keeps the 20m floor — a 5m teamwork is refused; plan passes `--mode plan`; goal attaches its schema; grillme attaches its schema and the refutation doctrine; **every mode** — goal, teamwork, grillme, plan — carries `--sandbox` unless `--writes`, and `--writes` lifts it, asserted per mode in the self-test since paiml/paiml-implement#12, fourteen ✓) — the dry run IS the control the ticket names: nothing is spawned, the composed command is asserted |
| AD-04 quorum on head 2b10b6598 | 1 FAIL, 2 PASS: the self-test asserted sandboxing only for `goal` while the composer sandboxes every mode — the self-test now asserts all four (bundle #12) and this receipt says so |
| bundle `verify.sh` | `lane-modes` PASS among 28 rows |
| bashrs on the composer | 0 errors |

## Not done, said plainly
- `goal` and `grillme` are templates over the same agy call; when agy ships the native modes the composer swaps the prompt prefix for the command and the schemas stay.
- No live goal or grill-me lane has been run through the composer yet; the first real run is the control that closes that gap (the quorum lanes that reviewed AD-03…AD-09 ran through `quorum-review.sh`, which predates the composer).

Verdict: **DONE when this PR merges green** (docs only on the pmat side). Until then the roadmap row is `in_progress` — deliberately: the branch's commits name PMAT-664 in their `Pmat-Ticket` trailer, and CB-1340 (AD-07) fails a commit that names a completed ticket; the row is closed with `pmat work edit PMAT-664 -s completed` in the follow-up after the merge, as for every ticket in this series.
