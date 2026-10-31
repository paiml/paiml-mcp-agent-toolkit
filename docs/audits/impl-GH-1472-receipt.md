# Implementation receipt — GH-1472

Partial fix, stated plainly: the issue's reported stdin hang of `pmat work start` could NOT be reproduced here (`Command::output()` does not inherit stdin). The issue stays open for that part.

## Measured
- Defect that did reproduce: `pmat work start <id>` on a ticket already InProgress was refused, because `InProgress` has no self-edge in `can_transition_to`, so a worker restarting its own ticket got "Invalid transition".
- Old code: new test `restart_in_progress_tests::start_is_idempotent_for_in_progress` FAILED (panicked at resolution.rs). Fixed code: `cargo test --lib restart_in_progress` -> 2 passed (`start_is_idempotent_for_in_progress`, `start_still_refuses_terminal_states`), 0 failed.

## CI follow-up
CI failed on two ratchets my diff moved: `.pmat-ratchet.toml` unwrap_calls_src_total (my two tests used `.unwrap()`, 20322 > 20320; now `.expect(..)`) and `docs/status/unrun-tests-ledger.md` (the executed-lib-test count rose by my 2 tests: 24407/27534 -> 24409/27536, regenerated with `pmat analyze unrun-tests --executed '' --write-ledger`; the unrun count 3127 is unchanged). No baseline raised.

## Scope
One source file (plus the regenerated ledger line above), `src/cli/handlers/work_handlers/core_handlers/resolution.rs`: in `resolve_yaml_ticket`, an InProgress ticket is returned as-is (re-activated) before the transition check; terminal states still bail. The work-dbc-v1 transition matrix is not edited. No gate content touched.
