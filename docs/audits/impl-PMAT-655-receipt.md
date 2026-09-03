# impl receipt — PMAT-655 (AD-03: commit enforcement)

| field | value |
|---|---|
| ticket | PMAT-655 (agentic-delivery-pmat.md §4.7 / §9.3; fixes #1126) |
| branch | PMAT-655-commit-enforcement (off master e10b89c53, after the 3.36.0 release, CRUX-03 and the docs receipts) |
| phase gate | `pmat verify --format json` on the branch |
| DoD gate | the PR's required checks |
| quorum | never (`--quorum never` per the invocation) |
| subagents | 0 |

## Deliverables
`[hooks]` config section (`strict`, `ticket_pattern`); `PMAT_HOOKS_STRICT` exported from it (resolved once from `--strict` OR the config, for both hooks); the pre-commit SATD branch exits 1 under strict; the dead pre-commit task-ID block removed; a generated `commit-msg` hook (`install_commit_msg_hook`, `generate_commit_msg_hook`, `remove_commit_msg_hook`); `--strict` on `hooks install` and `hooks init`; `scripts/commit-enforcement-audit.sh`; `contracts/commit-enforcement-v1.yaml` (pv lint PASS); spec §9.3 note; 7 tests in `src/cli/handlers/hooks_command_handlers/commit_enforcement_tests.rs` — 4 drive `git commit` in a throwaway repository, 3 assert the generated hook text and the strict plumbing (2 of them added after the AD-04 quorum review of this branch).

## Verification (orchestrator runs)

| check | result |
|---|---|
| tests RED before the implementation | `install_commit_msg_hook` did not exist — the tests could not compile against the old tree (no commit-msg hook has ever been installed) |
| `cargo test --lib -- commit_enforcement_tests hook_generation hooks_command` | 96 passed (5 new + every existing hook test); re-run after the quorum fixes: see the row below |
| **AD-04 quorum review of e668cf180** (3 agy lanes, `docs/audits/quorum-PMAT-655.json` on this branch) | 3 FAIL, cited: `--strict` reached only the commit-msg hook (`hook_generation.rs` read `[hooks] strict` and ignored the flag); `hooks verify --fix` / `hooks update` / the comply auto-install re-generated the hook with `strict=false`; dead `PMAT_TASK_ID_PATTERN` export; receipt test count wrong. All four fixed here |
| regression tests RED on the pre-fix tree (e668cf180 + the two tests, `installed_strict()` assert removed) | `0 passed; 2 failed` — `--strict must reach the pre-commit hook, not only commit-msg` and `an automatic rewrite dropped --strict`; the hook text in both failures carries `PMAT_HOOKS_STRICT=0` |
| regression tests GREEN on this tree | see the full-suite row below |
| named mutation: strict branch `exit 1` → `exit 0` | `a_commit_without_a_ticket_trailer_is_refused_in_strict_mode` **FAILED** ("a commit without Pmat-Ticket must be refused"); 4 passed, 1 failed; restored → 5 passed |
| acceptance script, pre-fix binary (3.36.0 release candidate) | **exit 1 — `FAIL: 1: hooks install --strict failed`** (no such flag, no commit-msg hook) |
| acceptance script, this tree's release binary | PENDING |
| `bashrs lint scripts/commit-enforcement-audit.sh` | 0 errors |
| `pv validate` / `pv lint contracts/commit-enforcement-v1.yaml` | valid; PASS |
| `pmat verify --format json` | PENDING |

## Jidoka
The old pre-commit task-ID check read `$1`, which a pre-commit hook never receives — it had never run. Recorded in the hook text; the check now lives where the message is.

## Decisions taken as the conservative option ([A])
- [A] Default `ticket_pattern` accepts `PMAT-NNN` **or** `#NNN` so repositories without pmat work are not locked out; the trailer is preferred and read with `git interpret-trailers --parse`.
- [A] Non-strict keeps warning (never refuses); strict is opt-in per repository (`pmat.toml`) or per install (`--strict`); the bundle opts in.
- [A] `--no-verify` is not blocked here (it cannot be from inside a hook); AD-07's comply check over the branch is the catch.

## Verdict
PENDING — DONE when the GREEN run and verify are recorded and the PR merges.
