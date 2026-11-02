# Receipt: PMAT-1591 (GH-1591), FLOW-012 P1 "the branch is the ticket"

Spec row (verbatim): "The active ticket is derived from the branch name work/PMAT-NNNN. Declaration and key files are retired."
Scope of this change: only the first sentence (derivation from the branch). Retiring declaration and key files is NOT done here and is follow-up.
Falsifier (verbatim): "A fresh worktree on work/PMAT-123 with no setup: pmat work status reports PMAT-123".
Source: infra#2853. Feeds W1 (infra#2871) and H1.

## Change

- `src/cli/handlers/work_handlers/core_handlers/branch_ticket.rs` (new): `ticket_from_branch` (exact `^work/PMAT-[0-9]+$`), `current_branch`, `resolve_ticket_id` (explicit id first, then branch, else an error that names the fix).
- `pmat work status` with no id (and not `--active`) goes through `handle_work_status_derived` (a wrapper, so the existing handler stays under the complexity gate; the first attempt inline in handlers.rs was refused by the pre-commit complexity gate at Cyclomatic 30). It prints `Active ticket: PMAT-N (from branch work/PMAT-N)` and the roadmap detail when the row exists. Needs no roadmap.
- `pmat work continue` and `pmat work complete`: `id` is now optional; resolved in the dispatcher by `resolve_ticket_id`.
- Off a matching branch with no id: status keeps its old behaviour (no ticket is guessed); continue/complete exit non-zero with "does not match work/PMAT-NNNN. Pass an id, or switch to a branch named work/PMAT-NNNN."
- An explicit id that differs from the branch ticket wins and a stderr note says so.

## Falsifier output (GREEN)

Test: `tests/falsification_branch_is_the_ticket.rs` (temp git repo, fresh `git worktree add -b work/PMAT-123`, no setup, spawns the built `pmat work status`).

Falsifier command, run with the built binary in a fresh `git worktree add -b work/PMAT-123` of a throwaway repo (no roadmap, no setup):

```
$ cd <fresh worktree on work/PMAT-123> && pmat work status
🎯 Active ticket: PMAT-123 (from branch work/PMAT-123)
   Not in docs/roadmaps/roadmap.yaml on this branch; add it with: pmat work add
```

Integration test (tests/falsification_branch_is_the_ticket.rs), final run after the complexity refactor:

```
test a_near_miss_branch_is_not_a_ticket ... ok
test an_explicit_id_beats_the_branch ... ok
test continue_and_complete_on_main_refuse_with_the_fix ... ok
test continue_on_a_work_branch_uses_the_branch_ticket ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test status_on_a_work_branch_reports_the_ticket_with_no_setup ... ok
test status_on_main_reports_no_active_ticket ... ok
```

Unit tests: `cargo test --lib branch_ticket` 3 passed. `cargo fmt --all -- --check` clean. `cargo clippy --lib --tests -- -D warnings` clean.

## RED proof (implementation stashed, test + Cargo.toml target only)

```
test continue_and_complete_on_main_refuse_with_the_fix ... FAILED
test continue_on_a_work_branch_uses_the_branch_ticket ... FAILED
test status_on_main_reports_no_active_ticket ... ok
test an_explicit_id_beats_the_branch ... ok
test status_on_a_work_branch_reports_the_ticket_with_no_setup ... FAILED
test a_near_miss_branch_is_not_a_ticket ... ok
stdout lacked PMAT-123: 📋 No items found
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

The three cases that need the feature failed: status on the work branch did not report PMAT-123, and continue/complete with no id did not resolve or refuse with the fix. The negative cases passed before and after, as they should (no guessing).

## Not retired in this change

- No declaration or active-ticket-file reader exists in pmat's `work status/continue/complete` path (searched src/ with pmat query and grep for active-ticket, declare, .pmat-work/active); there is nothing here to make the branch "win over". The "branch wins, stderr says so" rule is implemented only for the explicit-id case. Retiring declaration/key-file readers elsewhere (infra side, hooks) is a later step and is untouched.
- `pmat work start/checkpoint/falsify/event` still take their id as before.
- `docs/roadmaps/roadmap.yaml` carries the PMAT-1591 row created by `pmat work add`.
