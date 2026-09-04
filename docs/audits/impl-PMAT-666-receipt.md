# Implementation receipt — PMAT-666 (hooks_command.rs under the 500-line cap)

Branch `PMAT-666-split-hooks-command`. Routing: direct (a mechanical move). Follow-up to AD-03 (#1175) and AD-05 (#1177), whose `oversized_source_files_src` ratchet this drives down.

| check | result |
|---|---|
| RED | `src/cli/handlers/hooks_command_handlers/hooks_command.rs` 572 lines; the shell predicate the ratchet runs (`git ls-files … | awk '$1>500'`) counts **261** oversized non-test `src/*.rs` on master 58826a736 (the 262 the ratchet baseline names was measured on the AD-05 branch's own tree) |
| GREEN | the five commit-msg / strictness functions (`configured_strict`, `installed_strict`, `install_commit_msg_hook`, `generate_commit_msg_hook`, `remove_commit_msg_hook` — the ticket lists all five) moved to `commit_msg_hook.rs` (103 lines) as a second `impl HooksCommand`; `hooks_command.rs` 479 lines; the same predicate on this branch counts **260** — exactly one fewer, the one file this change shrinks |
| AD-04 quorum on head bdcc4454a | 3 FAIL, cited: the receipt said 262 → 260 (two files) where one file changed — the 262 was another tree's number; measured with one command on master and on the branch: 261 → 260. And `configured_strict` was moved though the ticket listed four functions — the ticket now lists the five that belong together |
| AD-04 quorum on head f4f2a6532 | 3 FAIL, cited: `pmat work edit` had re-serialised every row's status as `inprogress` (the enum's serde spelling), so the diff touched AD-06's row and this ticket's row contradicted the receipt's `in_progress`; the roadmap is rebuilt as master's rows plus this row spelled as master spells it |
| tests | `cargo test --lib -- commit_enforcement_tests hooks_command_handlers commit_msg_hook`: 102 passed (the AD-03 acceptance tests drive `git commit` through both hooks) |
| ratchet literals | at baseline |

Verdict: **DONE** when merged; the row is `in_progress` until then.
