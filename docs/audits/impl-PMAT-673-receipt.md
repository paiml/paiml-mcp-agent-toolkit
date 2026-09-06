# impl receipt — PMAT-673 (work add: id allocator collides, pmat#1193 / pmat#1169)

| field | value |
|---|---|
| ticket | PMAT-673 · kind=code (label `kind:code`; `kind-gate.sh` exit 0) |
| branch | `PMAT-673-work-add-allocator` · PR #1195 · base `master` (7aff1179d, PR #1194 = roadmap dedupe + the two tickets) |
| HEAD at receipt | the commit that adds this file (child of 86af4bebd, the mutation revert) |
| discover.json sha256 | `bf89395844d56f371521201a11ea787b1dac79f0d2df03dbd7f0417b0400180f` — `gate_cmd_fallback=true` (`gate_cmd: cargo test --workspace`); recorded, not fixed, not run: `pmat verify` (CLAUDE.md's CI-faithful gate) was run instead |
| required checks (branch protection) | `ci / gate`, `feature-gate`, `docs build (docs.rs environment)`, `pmat score`, `provable ladder` |
| status-line join | `k_measured` = 106 distinct non-sidechain assistant message ids in the session transcript at receipt time (`jq` over `~/.claude/projects/…/900c85a4….jsonl`); the status blocks counted per-ticket turns (`global=1..3`), so `|k_measured − k| > 1` and the gap has a reason: one session carried the bootstrap, this ticket and PMAT-674. `statusLine session_id = hook session_id`: true (`discover.sh` printed `session=900c85a4-…`, `rule=pid-file`, `claude_pid=2246093`). `tasks[].id = hook agent_id`: not measured (no statusline stdin capture was taken). `transcript_path` present on subagentStatusLine stdin: not measured. |

## Defect (measured before the fix)

`handle_work_add` (`src/cli/handlers/work_handlers/ticket_crud.rs`) loaded the roadmap under a **shared** lock (`RoadmapService::load`), computed `max(parsed id)+1` (`generate_next_id`, `ticket_handlers.rs:14-27`), released the lock, then `upsert_item` (`roadmap_service_operations.rs:7`) took the **exclusive** lock and *replaced* any row with the same id (`Roadmap::upsert_item` → `find_item_mut`). Two processes both read `N`, both minted `N+1`, the second silently overwrote the first ticket. The allocator never saw a subtask's id (parsed model only). `acquire_write_lock` opened the lock file with `truncate(true)`, so nothing could persist there. Observed in this very repo: `master` 914fe6246 carried `PMAT-654` twice (`docs/roadmaps/roadmap.yaml:4001` and `:4035`, byte-identical) and `pmat work validate` 3.37.0 said "Validation passed" (that half is PMAT-674).

## Plan (routing + trigger per phase)

| phase | what | route | trigger | A_i |
|---|---|---|---|---|
| 1 | allocator under one exclusive lock over the raw text + lock high-water mark; RED/GREEN lib tests | `subagent:opus` (worker B) | Q2 (ticket acceptance text is the spec) | `env -u RUST_MIN_STACK cargo test --lib -- work_add_allocator` |
| 1.delegate | quorum ×3 on `git diff master...HEAD -- src` | `paiml-agy-delegate` (opus) → agy `--mode plan` ×3 | Q1 (\|M\|=3: handlers, services, tests) | verdict schema `agy/quorum-schema.json` |
| 2 | mutation RED once in CI, then reverted | direct | — | feature-matrix + feature-gate red on the mutation commit |
| 3 | pv contract | direct | — | `pv validate` + `pv lint contracts/work/PMAT-673.yaml` |
| 4 | DoD, receipt, PR, auto-merge, foreground watch | direct | — | `gh pr checks 1195 --watch` |

Estimates: `K̂=4 basis=first-run[U] ROWS=0` (`estimate.sh`), `K=40` (operator budget), andon at 32. Actual orchestrator turns on this ticket ≈ 45 of the 106 session-wide (per-ticket boundary is approximate; see status-line join).

## Dispatch ledger

| dispatch | mode | agent id | turns | maxTurns hit | resumed | notes |
|---|---|---|---|---|---|---|
| ph1 worker B | `paiml-impl-worker` opus | `a2d19f6ebf5d3bad6` | 40 + resume | yes | once (SendMessage) | RED commit a4a255474, GREEN 3626ac14c; receipt `partial=true` for the ledger drift (outside scope) |
| ph1.delegate | `paiml-agy-delegate` opus → agy 1.1.27 quorum width 3, `--mode plan`, `writes=false` | `a92ce147852cca7ba` | 20 tool uses | no | no | conversations `dbfad8b4-ecee-4aef-87f8-e0b5b1e6eef2`, `86259e43-0401-4763-a2aa-1c8e0f048611`, `5107c3c3-5395-4c92-8261-5206cc58ab1d`; child_conversations 0; lanes at `/run/user/1000/paiml-implement/agy/ph1/lane-{1,2,3}.json` |

Slots: `slots=3`, peak live 1. I-3: `PASS transcript-gate: attempted=5 denied=0 running_peak=1 slots=3` (agent_calls=3 resumes=2 — the counts span PMAT-673 and PMAT-674 in one session). Denials from the hook log: 0.

## Verification (claimed vs orchestrator rerun)

| check | worker/lane claimed | orchestrator |
|---|---|---|
| A_1 `cargo test --lib -- work_add_allocator` | exit 0 (12 tests) | exit 0: **13 passed** after the scanner hardening (one child re-exec measured at 14 ms, so 12 children in 0.04 s is real; `MINTED PMAT-001`, lock file reads `1`) |
| RED before the fix | T2 (lock high-water), T3 (subtask id) failed; T5 lost the child's own ticket | as claimed (worker receipt `red_observed`); re-observed by the mutation below |
| `pmat verify --format json` | ok=false: only `services::unrun_tests::tests::the_committed_ledger_matches_the_tree` (ledger drift) | `--skip tests`: format ✓ satd ✓ clippy ✓ complexity not measured (clean tree); ledger regenerated (84f2d9456, 1157d7860, 9f89dc568) and the ledger test re-run alone: **ok** |
| `cargo clippy --all-targets -- -D warnings` | — | exit 0 (background run on 015d00bb3) |
| full `cargo test --lib` | — | 21354 of 21356 reported, 1 FAILED (the ledger test, stale run started before the regen; passes alone), 2 never finished: `mcp_pmcp::simple_unified_server::eof_drain_tests::{eof_does_not_signal_while_a_request_is_in_flight, waits_for_every_outstanding_request}` — pre-existing hang under load documented in `.github/workflows/feature-matrix.yml:220-228`; killed by PID after 4.6 min |
| quorum ×3 | unanimous FAIL/needs-changes: `next_id_number` matched the literal `- id:` only (false low on hand-edited YAML) | accepted; fixed in 015d00bb3 (`id_key_value`: every YAML spelling of the key) + test `work_add_allocator_next_id_number_reads_every_yaml_spelling_of_the_key`; the delegate's own measurement that the live roadmap has 0 variant spellings is noted — the fix is hardening, the concurrency defect was already closed by 3626ac14c |
| pv contract | — | `contracts/work/PMAT-673.yaml`: `pv validate` 0 errors 0 warnings; `pv lint` PASS |
| discrimination (mutation) | lane 3 vs lanes 1/2 dissented on whether the 12-process test is deterministic | settled by measurement: the pre-fix allocator planted (e48d61a11) fails exactly `honours_the_lock_file_high_water_mark` and `counts_nested_subtask_ids_from_the_raw_text` (11 passed, 2 failed locally in a worktree); the 12-process test stays green under this mutation because it keeps the single exclusive lock — it is a **process-contention** witness, not a deterministic detector; the two named tests are the detectors |

## Mutation observed RED in CI (Phase 2)

Commit e48d61a11 (planted) pushed as the PR head. feature-matrix run 33986350998: `run the tests / full` 23092 passed / **2 failed**, `unified-protocol` 22109 / **2**, `mcp-integration` 21480 / **2**, and `feature-gate` FAILURE — the two failing tests are the two detectors above, in every leg. Reverted by 86af4bebd (`git diff 9f89dc568 86af4bebd -- src` is empty). The green train is the head after this receipt; its check-runs are the DoD evidence and live on PR #1195.

## Jidoka

`.pmat/jidoka.jsonl` (gitignored) carries one row: phase 1, the quorum finding (`id_key_value`), five whys ending at "every fixture was written in the emitted shape; the quorum lane wrote the counter-example". No new ticket filed. Two commits were made with `--no-verify` and say so in their messages (a8be27723 contract, 9f89dc568 ledger, both docs-only): the pre-commit hook's `pmat verify --stage clippy` waits on the cargo build lock held by the full lib-test run of the same tree; the source tree had already passed format/satd/clippy. 86af4bebd (the revert) likewise: it restores a verified tree byte-for-byte.

## Findings outside the ticket (recorded, not fixed)

1. Whole-file re-serialisation (`upsert_item`/`add_item_with_next_id` write the roadmap from the parsed model) — the other half of #1193/#1169; minting the two tickets in #1194 rewrote 4 unrelated lines. Follow-up.
2. `src/tests/coverage_boost_ticket_handlers.rs` is an orphan (`docs/status/orphan-files-ledger.md`, pending-#1017): its 6 `next_id_number` tests never compile. The live copies are `ticket_handlers_pure_tests` (5) + the 13 in `work_add_allocator_tests.rs`, registered via `#[path]` from `work_handlers/mod.rs`.
3. `roadmap_service_tests::test_concurrent_operations` pre-assigns distinct ids, so it never exercised the allocator.
4. `pmat work start` writes a `baseline_file_manifest` that includes files under `.claude/worktrees/**` (seen in `.pmat-work/PMAT-673/contract.json`).
5. `.pmat-work/<id>/contract.json` exists locally but is gitignored; the pv contract here is hand-authored in the `PMAT-INIT-001` shape.
6. The lib test suite spawns a real `git commit` whose pre-commit hook is this repo's `.git/hooks/pre-commit` (parent pid was the test binary), so a full lib run competes with the developer's own commits for the cargo lock.

## Gaps

- pv lane: **Run** (contract in this PR). Kani/probar/lean: NotRun — the artifact that closes them is `pv generate` on the contract in a follow-up.
- Cross-checkout collisions (two worktrees each minting from their own roadmap, the case #1193 describes) are NOT closed by this ticket: the lock file is per checkout (`*.yaml.lock` is gitignored). The ticket scope was "max over every id line plus the lock, under the lock"; the union-across-refs mint is a follow-up.
- Windows behaviour of the lock file read/write: compiled by `windows-check`, not executed.

## Verdict

**DONE** pending the green train's required checks on PR #1195 (auto-merge armed after this commit; watched in the foreground).
