# impl receipt — PMAT-670 (GitHub #1186): a ticket claims the ladder level its bindings evidence

## Identity

| field | value |
|---|---|
| ticket | PMAT-670 (issue #1186) |
| branch | fix/1186-work-ladder-claim |
| base | origin/master at fed961669 |
| contract | contracts/work-ladder-claim-v1.yaml (pv validate: PASS) |
| acceptance | scripts/work-ladder-claim-audit.sh |
| routing | direct (orchestrator), no subagent, no quorum trigger (Q1 fired: 5 owning modules — quorum ran as the AD-04 review lanes on the PR instead) |

## Defect (as reported in #1186)

1. A ticket started without `--implements` claimed **L3**; nothing had evidenced it.
2. `--implements` existed only on `work start`, which refuses an InProgress ticket, so the normal flow could never reach L2 legitimately.
3. `work complete` ran the quality gate first; the ladder shortfall surfaced behind unrelated gate output.

Found while fixing: `work edit --level X` alone exited 0 with "No changes specified" because the early return sat above the new flag handling; and a binding to an equation the contract does not declare was accepted (the sha covered the file, nothing read it).

## Plan

| phase | change | files |
|---|---|---|
| P1 | `initial_verification_level(explicit, bound)`: explicit, else L2 if bound, else L1; `parse_level_arg` via `parse_strict` | src/cli/handlers/work_contract_core.rs, work_handlers/core_handlers/contract.rs |
| P2 | `--level` on add/start/edit; `--implements` on edit; `rebind_contract` acts on the saved contract; rebind runs before the "no changes" early return | src/cli/commands/work_commands_work.rs, command_dispatcher_work.rs, work_handlers/core_handlers/handlers.rs, work_handlers/ticket_crud.rs |
| P3 | ladder shortfall judged before `run_quality_check` on complete | work_handlers/core_handlers/handlers.rs |
| P4 | a binding must name a declared equation (`require_equation`) | src/cli/handlers/work_contract_binding.rs |

## Verification (claimed vs re-run — one actor, so one column, every row re-executed)

| leg (scripts/work-ladder-claim-audit.sh) | 3.36.0 binary | this branch |
|---|---|---|
| 1 unbound start claims L1 | RED (L3) | GREEN |
| 2 start --implements claims L2 with the binding | RED | GREEN |
| 3a add --level L2 is the claim after start | RED (no flag) | GREEN |
| 3b edit --level L2 rewrites the claim | RED (no flag) | GREEN (after moving rebind above the early return) |
| 3c control: --level L9 refused | GREEN (clap) | GREEN |
| 4 edit --implements binds an InProgress ticket, lifts to L2 | RED (no flag) | GREEN |
| 4b control: unknown equation refused | GREEN (clap) | GREEN (after `require_equation`) |
| 5a over-claim refused with LadderShortfall | GREEN | GREEN |
| 5b the refusal precedes any quality-gate output | RED | GREEN |
| 6 control: honest L1 not refused by the ladder | GREEN | GREEN |
| whole script | RED (exit 1) | GREEN (exit 0) — log: 1186-acceptance-4 |

Unit tests: `ladder_claim_tests` (3) and `work_contract_binding::tests` (10, incl. `resolve_refuses_an_equation_the_contract_does_not_declare`); four pre-existing binding fixtures declared no equation and bound anyway — they encoded the gap and now declare `eq`.

## Named mutation (observed RED)

Mutant: `initial_verification_level` unbound arm `L1` → `L3` (the 3.36.0 behaviour), one line in `src/cli/handlers/work_contract_core.rs`.

| run | result |
|---|---|
| mutant binary (built with `--target-dir target`, freshness asserted) | RED: leg 1 "got L3", leg 4 "level L3" — exit 1 |
| restored binary | GREEN — exit 0 |

A first attempt read GREEN on both sides: a temporary `.cargo/config.toml` left by a failed verify run redirected `cargo build` to another target dir while the script ran the stale binary. The chain now builds with an explicit target dir and refuses a binary older than the mutated file.

## Gate (`pmat verify --format json`, 3.36 rc binary, re-run by the orchestrator)

| run | format | complexity | satd | clippy | tests |
|---|---|---|---|---|---|
| 1 | ok | ok | ok | ok | red: `the_committed_ratchet_holds_at_head` — `.unwrap()` 20337 vs baseline 20336 (one added in the new unit test; now `.expect`) |
| 2 | ok | ok | ok | ok | red only on `the_committed_ledger_matches_the_tree` (new tests; the ledger is regenerated last, on the clean tree, and that test passes alone afterwards) |

Complexity re-measured directly on the changed files at 30/25: no violation; 3/2 control exits 1. Ratchet literals vs master: `#[allow(` +0, `panic!(` +0, `.unwrap()` +0 net.

## Jidoka log

- 3b/3c/4/4b red on the first green build: `handle_work_edit` returned Ok(()) "No changes specified" before the rebind block. Fix: rebind precedes the early return; the warning names the two flags.
- 4b red after that: `resolve_binding` never read the YAML. Fix: `require_equation`.

## Gaps

- `pmat work migrate --levels` (mentioned in the report) is not part of this change; existing contracts keep their stored claim and can be corrected with `work edit --level`.
- ledger: none skipped.

## Verdict

DONE — pending the PR checks and the quorum verdict (docs/audits/quorum-PMAT-670.json).
