# impl receipt — PMAT-674 (work validate: duplicate ids RED, unparseable RED with file:line, wired into `ci / gate`)

| field | value |
|---|---|
| ticket | PMAT-674 · kind=code (label `kind:code`; `kind-gate.sh` exit 0, files=12) |
| branch | `PMAT-674-work-validate-duplicate-ids` · PR #1196 · base `master` (cut from 7aff1179d; merged 97e826ae8 = PMAT-673 back in after it landed, strict protection) |
| HEAD at receipt | the commit that adds this file (child of 9ebb0fd9d, the post-merge ledger regen) |
| discover.json sha256 | `bf89395844d56f371521201a11ea787b1dac79f0d2df03dbd7f0417b0400180f` (same discovery as PMAT-673; `gate_cmd_fallback=true`, `gate_cmd: cargo test --workspace` recorded, not fixed, not run — `pmat verify` was the gate) |
| required checks | `ci / gate`, `feature-gate`, `docs build (docs.rs environment)`, `pmat score`, `provable ladder` |
| status-line join | `k_measured` = 130 distinct non-sidechain assistant message ids in the session transcript at receipt time; status blocks counted per-ticket turns (`global=1..3`), so `|k_measured − k| > 1` with a reason: one session carried the bootstrap, PMAT-673 and this ticket. `statusLine session_id = hook session_id`: true (`session=900c85a4-…`, `rule=pid-file`, `claude_pid=2246093`). `tasks[].id = hook agent_id` and `transcript_path` on subagentStatusLine stdin: not measured. |

## Defect (measured before the fix)

`handle_work_validate` (`src/cli/handlers/work_handlers/ticket_validate_migrate.rs`) strict-parsed the roadmap and printed "Validation passed". Two rows sharing an id are two well-formed rows to serde, so on `master` 914fe6246 — `PMAT-654` at `docs/roadmaps/roadmap.yaml:4001` and `:4035`, byte-identical — `pmat work validate` 3.37.0 exited 0 (fixed in the data by #1194). A parse failure bailed with the bare string "Roadmap validation failed"; the position lived only in a stdout context block. No CI job ran the validator.

## Plan (routing + trigger per phase)

| phase | what | route | trigger | A_i |
|---|---|---|---|---|
| 1 | duplicate ids RED with both `file:line`; parse error RED with `file:line:column`; `--help` exit codes; 10 lib tests incl. a CI-wiring test | `subagent:opus` (worker B) | Q2 | `env -u RUST_MIN_STACK cargo test --lib -- work_validate_duplicate` |
| 1.direct | `.github/workflows/ci.yml`: `roadmap-validate` job + `gate.needs` + result loop (workers may not edit workflows); `--help` doc (the brief named the wrong file) | direct | — | the CI-wiring test + `pmat work validate --help` |
| 1.delegate | quorum ×3 on `git diff master...HEAD -- src .github/workflows/ci.yml` | `paiml-agy-delegate` (opus) → agy `--mode plan` ×3, `writes=false` | Q1 (\|M\|=3) | `agy/quorum-schema.json` |
| 2 | mutation RED once in CI, reverted | direct | — | feature-matrix + feature-gate + `roadmap validates` red on the mutation commit |
| 3 | pv contract | direct | — | `pv validate` + `pv lint contracts/work/PMAT-674.yaml` |
| 4 | DoD, receipt, PR, auto-merge, foreground watch | direct | — | `gh pr checks 1196 --watch` |

Estimates: `K̂=4 basis=first-run[U] ROWS=0`, `K=30` (operator budget), andon at 24. Orchestrator turns on this ticket ≈ 24 of the 130 session-wide (per-ticket boundary approximate).

## Dispatch ledger

| dispatch | mode | agent id | turns | maxTurns hit | resumed | notes |
|---|---|---|---|---|---|---|
| ph1 worker B | `paiml-impl-worker` opus | `ae09230a3e59a73d2` | 40 + resume | yes | once (SendMessage) | RED ecfae580f, GREEN d4d11c7fa; `partial=true`: V6 (ci.yml not yet patched by the orchestrator) and V7 (the `Validate` variant lives in `src/cli/commands/work_commands_work.rs:405`, not `definition.rs` as the brief said — the worker refused to widen scope, correctly) |
| ph1.delegate | `paiml-agy-delegate` opus → agy 1.1.27 quorum width 3, `--mode plan` | `ab085d76705b17457` | 18 tool uses | no | no | conversations `94b41fbb-7717-4dd2-a2d9-18702e252227`, `3f9b1a86-2466-4047-b69e-fb34069b80b0`, `3dd2952e-e448-4b3c-b69e-e61b0a9fbbac`; child_conversations 0; lanes at `/run/user/1000/paiml-implement/agy/ph1-674/lane-{1,2,3}.json` |

Slots: `slots=3`, peak live 1. I-3: `PASS transcript-gate: attempted=6 denied=0 running_peak=1 slots=3` (agent_calls=4 resumes=2, session-wide). Denials: 0.

## Verification (claimed vs orchestrator rerun)

| check | claimed | orchestrator |
|---|---|---|
| A_1 `cargo test --lib -- work_validate_duplicate` | exit 101 (V6, V7 red for the two reasons above) | exit 0 after the direct edits: 10 passed; **11 passed** after the scanner hardening; **25 passed** on the merged tree together with PMAT-673's 13 and the ledger test |
| RED before the fix | V1, V2, V6, V7 failed on ecfae580f | as claimed (worker `red_observed`); re-observed by the mutation below |
| binary exit codes (`cargo run --bin pmat -- work validate --path …`) | valid 0 · duplicate 1 · unparseable 1 · missing file 1 | pre-fix roadmap 914fe6246 through the built binary: exit 1, `error: duplicate id PMAT-654 at …/roadmap.yaml:4001, …/roadmap.yaml:4035`; `status: bogus`: exit 1, `…/roadmap.yaml:3:5: roadmap[0]: unknown status 'bogus'`; `--help` shows `Exit codes: 0 — …; 1 — …`. Trap found on the way: `./target/debug/pmat` in the repo is a stale Sep-2 binary (the real target dir is off-site) — it said "Validation passed"; only `cargo metadata`'s target directory is trustworthy |
| `pmat verify --format json` | ok=false: V6, V7 and the unrun-tests ledger test (all expected at that moment) | `--skip tests` on 3164e6296: format ✓ satd ✓ clippy ✓; ledger regenerated three times (e995b64b3→91969d16d, 4484b5b4f, 9ebb0fd9d) and the ledger test passes on the merged tree |
| `cargo clippy --all-targets -- -D warnings` | — | exit 0 (background run on 3164e6296) |
| quorum ×3 | unanimous needs-changes: (a) FALSE RED — an `id:` line inside a block scalar counted; (b) FALSE GREEN — flow-style `{id: X}` / `? id` never matched. CI wiring, control-step polarity, exit codes, `#[path]` registration judged sound by 3/3 | (a) is live: the roadmap carries one `notes: \|` block (line 653). Both reproduced RED on the previous scanner (`left: [(2,PMAT-001),(6,PMAT-001),(7,PMAT-002),(9,PMAT-002)]`), fixed in 74bb8462f (block-scalar state + flow mappings), test `work_validate_duplicate_scanner_skips_block_scalars_and_reads_flow_mappings`. `? id` stays unrecognised and is named in the doc. |
| pv contract | — | `contracts/work/PMAT-674.yaml`: `pv validate` 0 errors 0 warnings; `pv lint` PASS |
| discrimination (mutation) | — | pre-fix validator planted (3fc966180: `duplicates` forced empty): locally 10 passed / **1 failed** (`ids_are_refused_and_both_lines_reported`) |

## Mutation observed RED in CI (Phase 2)

Commit 3fc966180 pushed as the PR head (draft). feature-matrix run 33989881592: `run the tests / full` 23091 / **1 failed**, `unified-protocol` 22108 / **1**, `mcp-integration` 21479 / **1** — the failing test is `work_validate_duplicate_ids_are_refused_and_both_lines_reported` in every leg; `feature-gate` FAILURE. **The new gate proved it can fail**: job `roadmap validates` (101370103249) — build from the tree 4 m 10 s, then the step `control — a duplicated id must be refused (exit 1)` FAILED because the mutated validator exited 0, and the validate step was skipped. Reverted by 479386980 (`git diff 3164e6296 479386980 -- src` is empty). The green train is the head after this receipt.

## Jidoka

`.pmat/jidoka.jsonl` (gitignored): one row for the quorum finding (`collect_id_lines`), five whys ending at "fixtures were written in the shape serde emits; the lanes wrote the counter-examples". Brief defect, mine: scope_paths named `commands_enum/definition.rs` for the `Validate` variant; it lives in `work_commands_work.rs` (an `include!` fragment). The worker refused to widen scope and said so; the edit was done directly. No `--no-verify` on this branch except the planted mutation commit (a deliberate red, said in its message).

## Findings outside the ticket (recorded, not fixed)

1. PMAT-673 and PMAT-674 each carry a raw-text id scanner (`id_key_value` in `roadmap_service_operations.rs`; `collect_id_lines` in `ticket_validate_migrate.rs`). The validate one is the stricter (block scalars, flow style); the allocator's should be pointed at it. Follow-up.
2. `./target/debug/pmat` inside the repo is stale and misleading (see the exit-code row).
3. `strict=true` branch protection: a branch cut from the pre-merge commit is BEHIND its own base after the base PR merges by merge commit; every second PR in a cascade pays one update train.

## Gaps

- pv lane: **Run** (contract in this PR). Kani/probar/lean: NotRun — closed by `pv generate` in a follow-up.
- Explicit-key `? id` spelling: not recognised (documented; never seen in a roadmap; a false green only if hand-written that way).
- Windows: `windows-check` compiles; the validator is not executed there.

## Verdict

**DONE** pending the green train's required checks on PR #1196 (auto-merge armed after this commit; watched in the foreground).
