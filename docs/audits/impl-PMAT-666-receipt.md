# Implementation receipt — PMAT-666 (hooks_command.rs under the 500-line cap)

Branch `PMAT-666-split-hooks-command`. Routing: direct (a mechanical move). Follow-up to AD-03 (#1175) and AD-05 (#1177), whose `oversized_source_files_src` ratchet this drives down.

| check | result |
|---|---|
| RED | `src/cli/handlers/hooks_command_handlers/hooks_command.rs` 572 lines; the shell predicate the ratchet runs counts 262 oversized non-test `src/*.rs` |
| GREEN | the commit-msg hook and the strictness helpers (`configured_strict`, `installed_strict`, `install_commit_msg_hook`, `generate_commit_msg_hook`, `remove_commit_msg_hook`) moved to `commit_msg_hook.rs` (103 lines) as a second `impl HooksCommand`; `hooks_command.rs` 479 lines; the predicate counts 260 |
| tests | `cargo test --lib -- commit_enforcement_tests hooks_command_handlers commit_msg_hook`: 102 passed (the AD-03 acceptance tests drive `git commit` through both hooks) |
| ratchet literals | at baseline |

Verdict: **DONE** when merged; the row is `in_progress` until then.
