# IMPL-PMAT-1385: `pmat work migrate` writes under the repository lock, and every roadmap write goes through `RoadmapWriteLock`

## Identity

| field | value |
|---|---|
| ticket | PMAT-1385 (#1385, `kind:code` on the issue and on the roadmap row); `kind-gate.sh` exit 0 `kind=code`; `model-gate.sh` `model=opus-5 class=opus decision=admit basis=transcript` |
| PR | #1389, branch `PMAT-1385-roadmap-unlocked-write-bypass` (renamed from `fix/roadmap-unlocked-write-bypass`) |
| base | `origin/master` `8915fe3e6` at start; rebased onto `7c2aa59b8` (#1383) and then `7fa1be27d` (#1387); every measurement below carries its tree line |
| `discover.json` sha256 | `f0f7bca7b761326e6c4143114800efaf4fdba26f37faaaf5ff042f705ad7a9b4` |
| `gate_cmd` | `cargo test --workspace`, **`gate_cmd_fallback=true`**. The gates actually run were `pmat verify --format json` (this repository's CLAUDE.md) and the full lib suite under nextest |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| I-3 | `PASS transcript-gate: attempted=2 denied=0 stalled=0 running_peak=1 slots=3` (before the round-2 review dispatch) |

## Scope, as landed

| # | item | verdict | evidence |
|---|---|---|---|
| 1 | `pmat work migrate` reads, transforms and writes under one repository lock (`RoadmapService::migrate_text`; shared lock for `--dry-run`) | DONE | `work_migrate_waits_for_the_repository_lock` in both modes; RED with the pre-fix handler |
| 2 | whole-file mode keeps every byte the transform does not change; the `.bak` is the roadmap as read under the lock | DONE | `work_migrate_keeps_every_byte_it_does_not_normalise` (comment, unknown key, block scalar) |
| 3 | fragment mode (`docs/roadmaps/entries/`): `roadmap.yaml` never written, no `.bak`; changed fragments rewritten; a changed base row gets a superseding fragment; everything checked before the first write; a base row carrying trailing text refused | DONE | `work_migrate_in_fragment_mode_*` (3 tests); mutations M2, M3, M5, M6 RED |
| 4 | every raw write under `docs/roadmaps/` is a method of `RoadmapWriteLock`, a value that exists only while the exclusive flock is held | DONE | M1 RED (4 lock tests) |
| 5 | the writer gate: a `--lib` taint analysis at the serialisation site, run by the required `ci / gate` | DONE | 11 → 1 → 0; renamed-binding mutant RED; M4, M7, M8, M9 RED; CI job 105200164445 ran all gate tests `ok` |
| 6 | `make gate` row | DONE | #1368 merged (`ef2a0b947`) while this PR waited on CI; rebased, and the `roadmap-writer-gate` row added at `scripts/gate.sh`'s extension point |

Contract `contracts/roadmap-writer-lock-v1.yaml`:
- `pv validate`: valid.
- `pv status`: 7 proof obligations, 7 falsification tests.
- `pv lint contracts --severity error`: PASS.
- `scripts/pv-obligation-gate.py`: 0 problems over 38 contracts.
- Evaluation: each falsification test was run by exact name (18 test names across the 7 entries, including the 3 PMAT-1363 lock tests RWL-F-004 names), and an obligation counts only if every covering test ran green. **7 obligations, 7 evaluated, 0 failed.**

## Plan and routing

| phase | what | route | executor | trigger |
|---|---|---|---|---|
| 0 | kind/model/config gates, discovery, ticket and issue | `route=self` | orchestrator | - |
| 1 | plan grill: how migrate writes, fragment-mode semantics, the gate, PMAT-1363 scope | `route=agy-plan w=1.00 basis=absent effort=1[U]` | delegate, grillme width 3 | Q1 (\|M\|≥3), Q2 |
| 2 | engine + mutants (RED), lock token, `migrate_text`, tests (GREEN) | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` printed; **executed direct** | orchestrator | - |
| 3 | script, contract, mutations, ratchets, ledgers | `route=self` | orchestrator | - |
| 4 | pre-merge review, then re-review after fixes | `route=agy-quorum w=1.00 basis=absent effort=1[U]` | delegate, quorum width 3, twice | Phase 4 |

The phase 2 deviation from R-4 is a finding, not an oversight. The gate's allow-list is the lock token, and the token was being cut into the same service files in the same phase. A writing lane in a linked worktree could not have run the tree test against that refactor. The implementation was done direct.

## Dispatch ledger

| dispatch | lane | width | verdicts | agy conversations |
|---|---|---|---|---|
| ph1 delegate | grillme | 3 | FAIL / FAIL / FAIL | `1cabaeba-dec6-49d1-a04f-33786d283c10`, `05af913e-0ccf-4838-a9bc-2c02f48a06f3`, `fcb5e9c0-436c-45ac-a036-b0db07b28a13` (children=3) |
| ph4 delegate, round 1 on `dbf8774ec` | quorum (grillme) | 3 | FAIL / PASS / PASS | `a68900f9-561c-4691-8d1d-da9bc6bdcb21`, `15a767d1-855a-46f0-b0b0-92f846475e01`, `3f0dc15d-e2d4-4f8f-a7f5-184ad3f14d96` (children=3) |
| ph4 delegate, round 2 on `287746ab0` | quorum (grillme) | 3 | FAIL / PASS / PASS | `b180962a-f1b2-4af7-aa74-ee8c3b87c3ce`, `43046cca-dbdf-408d-a786-55a19556647a`, `3c2cb5d2-e522-49f5-8c37-f0001f6a36f2` (children=3) |
| ph4 delegate, round 3 | quorum (grillme) | 3 | recorded in `docs/audits/quorum-PMAT-1385.json` | in the artifact |

- Lanes: `gemini-3.1-pro-high`, `gemini-3.8-flash-high`, `gemini-3.7-flash-high`. Each model is measured from the lane's own log, and none is the author's family.
- Slots: at most 1 live subagent at any time. Denials: 0. Stalls: 0.
- Author model: `$XDG_RUNTIME_DIR/paiml-implement/model-<sid>` does not exist, so the delegate passed `--author-model claude-opus-5`, the model `model-gate.sh` measured from the transcript.

### Plan quorum rulings, and what was done with them

- **D1** (how migrate writes): LOCKED-TEXT-PRIMITIVE, 3/3. Not routed through `RoadmapService::save()`, which re-serialises the whole model and drops comments, unknown keys and block scalars (PMAT-679).
- **D2** (base rows in fragment mode): SUPERSEDE-BASE-ROWS, 2/3. The dissent said superseding fragments make `pmat roadmap aggregate --check` fail on a feature branch. That check is the default-branch post-merge gate (its own message: "Run `pmat roadmap aggregate --write` on the default branch; never edit it in a pull request"). `work add` and `work edit` already write fragments on feature branches, and `replace_item_raw` supersedes base rows the same way. Round 1 of the review ruled D2 HONOURED 3/3.
- **D3** (the gate): CHANGE, 3/3. All three lanes found the lexical rule "a sink inside `impl RoadmapService` is locked" unsound, in both directions. `write_roadmap_unlocked` is inside the impl but takes no lock. `replace_atomically` is locked but sits outside the impl. Replaced by a lock token that the type system checks, plus a gate whose only allowed sinks are the token's methods, so the analysis needs no lock reasoning.
- **D4** (carry PMAT-1363's completion): 2/3 CARRY. Moot: #1383 completed PMAT-1363 on master before this branch was rebased, and this branch touches no roadmap row.

### Review round 1: lane 1's findings, answered with fixes, not argument

1. **Trailing text.** `split_entries` runs the last base row's block to the end of the file, so a footer after the list would travel into that row's superseding fragment. Fixed: `text_after_row` refuses such a row and writes nothing, while trailing blank lines still migrate. New test `work_migrate_in_fragment_mode_refuses_to_carry_text_after_the_last_row`; M5 disables the check and turns it RED.
2. **Shallow module walk.** The gate's walk read only top-level items and silently dropped an `include!` with a computed path. Now a full `syn` visit:
   - finds `mod` and `include!` in inline modules, impls and function bodies;
   - resolves `include!(concat!(env!("OUT_DIR"), …))` to the build script's real output;
   - fails on any other computed path.

   It reads 3037 files, including the 4 generated ones the old walk skipped (`tool_registry.rs`, `alias_table.rs`, `trigram_index.rs`, `mcp_tool_schemas_gen.rs`). The tree still reports 0.
3. **Contract nit.** RWL-F-004 named three tests by prefix; it now names them in full.

### Review round 2: lane 1's four BLOCKING claims, each read against the code and fixed

Lanes 2 and 3 marked round 1's findings RESOLVED; lane 1 kept both OPEN with four claims. Each was checked against the code before acting:

1. **`text_after_row` skipped `#`-lines. Real.** A block scalar whose text starts with `#`, or a comment indented inside the row, would have been read as text after the row and wrongly refused. The rule is now: a row's own lines are its first line and every later non-blank line deeper than the row's column. The trailing-text test gained a control arm (a block scalar with `#` lines and an inner comment migrate whole). M6 restores the old rule and turns that arm RED.
2. **`create_dir_all` under `docs/roadmaps/` without the token. Partly real.** Both calls (in `write_roadmap_unlocked` and `write_fragment`) already sat inside functions holding the token, so the lock was held. But directory creation was not a sink, and creating `entries/` is what turns fragment mode on. Now `fs::create_dir[_all]` is a sink and `RoadmapWriteLock::create_dir_all` is the only way to call it on a roadmap path. M9 puts the raw call back and turns the tree test RED. The new sink found one more site, `IdAuthority::in_git`. It creates `<git common dir>/pmat`, the lock's own directory inside `.git`; the path is tainted only because it is resolved from the roadmap's location. It is allowed for that one kind, with the reason recorded.
3. **The allow-list ignored `sink.kind`. Real for `open_lock_file`,** a free function, not a token method. `ALLOWED` is now `(file, function, kinds, reason)`, and liveness is checked per kind. M8 plants an `fs::write` in `open_lock_file` and turns the tree test RED.
4. **`#[path]` inside an inline module resolved from the file's directory. Real** per the Rust reference: inside an inline module, the path is relative to that module's own directory. Fixed. New test `roadmap_writer_gate_walk_reaches_nested_modules_and_includes` builds a synthetic crate with an `include!` inside an impl, a `#[path]` module inside an inline module, a module declared in a function body and a `cfg(test)` module. All three writers are reached, the test-only one is not, and an `include!` the walk cannot locate is reported. M7 resolves `#[path]` from the file's directory and turns it RED.

Lane 1 wrote 19 scratch files into its own review clone (a `syn` scratch crate and `#[path]` experiments). `agy-lane.sh` reported it `KEPT` and the shared checkout was untouched. The clone was deleted after its contents were listed.

## RED, then GREEN

| state | tree | gate | migrate tests |
|---|---|---|---|
| master + gate only | `HEAD=4a3bacb08 origin/master=8915fe3e6 behind=0` | **11** raw writes: `write_migration`; `replace_atomically` (write, rename, remove_file); `write_fragment` (write, rename); `remove_fragment`; `open_lock_file`; `persist_new_row`; `replace_item_raw`; `write_roadmap_unlocked` | — |
| token in, pre-fix `ticket_validate_migrate.rs` restored | `HEAD=4a3bacb08` + working tree | **1**: `write_migration: fs::write` | lock test RED ("wrote …/roadmap.yaml while another process held …/roadmap.yaml.lock"); fragment test RED; refusal test RED; byte-preservation GREEN (a regression pin) |
| same, every `roadmap_path` binding renamed to `zz` | same | **1**, still RED; `pmat query --literal "fs::write(roadmap_path" --files-with-matches` no longer lists `ticket_validate_migrate.rs` | — |
| fix | `HEAD=7c0029e9b origin/master=8915fe3e6 behind=0` | 0; allow-list live | 4/4 GREEN |
| after review round 1 | `HEAD=9553fd200 origin/master=7fa1be27d behind=0` | 0; allow-list live; 3037 files | 5/5 GREEN; the suite with the PMAT-1363 fragment tests is 59/59 |
| after review round 2 | `HEAD=287746ab0 origin/master=7fa1be27d behind=0` + round-2 fixes | 0; allow-list live per kind | 5/5 GREEN; suite 60/60 |

## Mutations (each applied, run, restored from git)

| id | mutation | RED |
|---|---|---|
| M1 | `RoadmapWriteLock::acquire` takes no `lock_exclusive` | 4: `work_migrate_waits_for_the_repository_lock`, `roadmap_fragments_an_add_waits_for_the_repository_lock`, `roadmap_fragments_a_save_waits_for_the_repository_lock`, `roadmap_fragments_aggregate_write_waits_for_the_repository_lock` |
| M2 | fragment mode skips base rows | 2: `work_migrate_in_fragment_mode_never_opens_the_aggregate`, `work_migrate_in_fragment_mode_refuses_before_writing_anything` |
| M3 | skip the check-before-write loop | 1: `work_migrate_in_fragment_mode_refuses_before_writing_anything` |
| M4 | the engine loses `let` propagation | 5: 4 planted-mutant tests plus the allow-list liveness test |
| M5 | disable the text-after-row refusal | 1: `work_migrate_in_fragment_mode_refuses_to_carry_text_after_the_last_row` |
| M6 | restore the rule that skipped `#`-lines when finding a row's own lines | 1: the same test's block-scalar arm |
| M7 | resolve `#[path]` inside an inline module from the file's directory | 1: `roadmap_writer_gate_walk_reaches_nested_modules_and_includes` |
| M8 | plant `std::fs::write(lock_path, "")` in `open_lock_file` | 1: the tree test (`open_lock_file: fs::write`), the kind is not allowed there |
| M9 | put a raw `fs::create_dir_all(parent)` back in `write_roadmap_unlocked` | 1: the tree test (`write_roadmap_unlocked: fs::create_dir_all`) |

## Every writer of a path under `docs/roadmaps/`, with its verdict

| writer | writes | verdict |
|---|---|---|
| `write_migration` (`pmat work migrate`) | `roadmap.yaml`, and `roadmap.yaml.bak` one block above | **the bypass**. Both writes were unlocked, after an unlocked read. Replaced by `migrate_text`; both writes now go under the lock, and fragment mode writes no `.bak` |
| `write_roadmap_unlocked` ← `save`, `upsert_item`, `upsert_item_checked`, `remove_item`, `initialize`, `add_item_with_next_id` (empty roadmap) | `roadmap.yaml` or `entries/` | locked by convention; now takes the token |
| `persist_new_row` (`work add`), `replace_item_raw` (`work edit`) | `roadmap.yaml` or `entries/` | locked by convention; now take the token |
| `roadmap_fragments::write_fragment`, `remove_fragment` | `entries/<id>.yaml` | `pub` and callable with no lock; now require the token |
| `replace_atomically` (`pmat roadmap aggregate --write`) | `roadmap.yaml` | locked via `with_write_lock`; now receives the token through `Access::Write` |
| `open_lock_file` | the lock file; outside git, `roadmap.yaml.lock` beside the roadmap | the lock itself; allowed for `OpenOptions::open` and `fs::create_dir_all` only, and it writes no roadmap content |
| `IdAuthority::in_git` | `<git common dir>/pmat/` (directory) | the lock's own directory inside `.git`, tainted only through the roadmap's location; allowed for `fs::create_dir_all` only |
| `write_roadmap_unlocked`, `write_fragment` directory creation | `docs/roadmaps/`, `docs/roadmaps/entries/` | already under the lock; now through `RoadmapWriteLock::create_dir_all` (creating `entries/` turns fragment mode on) |
| `apply_roadmap_changes` (`pmat maintain roadmap --fix`) | `--roadmap`, default **`ROADMAP.md`** | not this class: markdown checkboxes at a runtime path. The gate's clap rule keys defaults by subcommand, so another subcommand's `--roadmap docs/roadmaps/roadmap.yaml` does not taint this one |
| `handle_roadmap_sync` (`pmat roadmap sync`) | `<project>/ROADMAP.yaml` | not under `docs/roadmaps/` |
| `pmat spec sync` | via `RoadmapService::save` | locked |

## Where the gate runs, and how its failure reaches the build

- `ci / test`, a job of the required `ci / gate`, runs sovereign-ci's `cargo test --lib`. If that fails and the retry `cargo test --lib -p paiml-mcp-agent-toolkit` fails too, the step prints `::error::Tests failed` and exits 1. Run 35220796603, job 105200164445, on `dbf8774ec`, printed every gate and migrate test as `ok`, including `roadmap_writer_gate_every_roadmap_write_in_the_tree_goes_through_the_lock_token`; the result was `21745 passed; 0 failed`.
- `scripts/roadmap-writer-gate.sh` runs the 15 named tests. It refuses a vacuous filter, because `cargo test -- <filter>` exits 0 when nothing matches: each test must appear by name as `ok`. `--self-test` covers 5 arms: control, a filter that matched nothing, the tree test missing, a failed test, and no result line. Judged on the RED logs above: exit 1.
- `make gate`: the row `cmd | ci / gate | roadmap-writer-gate | sovereign-ci.yml test "Run tests" (roadmap_writer_gate_* and work_migrate_*) | … | bash scripts/roadmap-writer-gate.sh` sits below the extension marker in `scripts/gate.sh` (#1368). `scripts/gate.sh --list` accepts the table, and `src/make_gate_tests.rs` passes with it.

## Verification (claimed vs re-run)

| check | result |
|---|---|
| `cargo test --lib -- roadmap_writer_gate work_migrate_ roadmap_fragments_` at `9553fd200` | 59 passed, 0 failed |
| full lib suite, nextest, at `7c0029e9b` + fixes | 21740 passed, 3 failed. All 3 were ratchets; each is resolved below |
| `pmat analyze unrun-tests --check-ledger`, `pmat analyze reachability --check-ledger` (this tree's binary) | both exit 0 after regeneration |
| ratchet counts at HEAD | `panic!(` 785, `.unwrap()` 20325 total and 9177 outside `cfg(test)`: every one equals its baseline |
| CB-200 with this tree's binary | 1741 below A on a clean `origin/master` worktree and 1741 at HEAD. The delta is 0, and the red is pre-existing |
| `pmat verify --format json` at `9553fd200` (this tree's binary) | format ok, satd ok, clippy ok, tests ok; complexity `not_applicable` ("no Rust files changed vs HEAD"; the pre-commit hook measured complexity on every commit); `ok: null` with `not_measured: [complexity]` |
| `cargo test --lib -- roadmap_writer_gate work_migrate_ roadmap_fragments_` after review round 2 | 60 passed, 0 failed; `scripts/roadmap-writer-gate.sh --judge`: GREEN, 15 required tests |

## Jidoka

| defect | owner | five whys, to a mechanism |
|---|---|---|
| the bypass itself | `ticket_validate_migrate.rs` | Why did migrate race `work add`? It wrote with `std::fs::write` after an unlocked read. Why could it? The lock was a private detail of `RoadmapService`, and nothing made writing without it fail. Why was that not caught? The only instrument was a literal query on a variable name. Why is that not enough? A write's destination is data flow, not a name. Mechanism: nothing typed or checked tied a write to holding the lock. Now the token does, and the gate does |
| ratchet red on the first full run (`panic!` +2, `.unwrap()` +27) | the new test files | The metric counts text in every `src/*.rs`, and the fixture sources inside string literals and the test helpers carried `unwrap`/`panic!`. Removed: counts are at baseline |
| unrun-tests and orphan-files ledgers drifted | the generated ledgers | New tests and files change the rendered counts. Regenerated with this tree's binary, which adds no new unrun or orphan entries |
| `traceability` red on the first push | master: open issue #1386 had no roadmap row (CB-2115 ORPHAN-GITHUB) | Not this branch. #1387 (PMAT-1336 lifecycle) registered it, and this branch was rebased onto it |
| CB-200 red locally | pre-existing on master (1741 vs a baseline of 1688) | It is Unmeasurable in CI, which has no `.pmat/context.db`, and the local delta from this branch is 0 |

## Estimates

`K̂=35`, `K=70`, `basis=docs/audits/impl-estimates.jsonl:L23-L32`. **Actual 166** (`k_measured` from the transcript at the first receipt commit, before review rounds 2 and 3, CI and merge), recorded as `docs/audits/impl-estimates.jsonl` L35 (L33 when written; rows other tickets appended on master moved it) through `pmat work estimate record`. `0.8K` (56) was crossed; the andon did not fire because every commit past it was green (RED only in the deliberate RED commit). The estimate missed by 4.7×: the gate needed three precision rounds (350 → 287 → 11 raw writes) and the walk a fourth, none of which a first-run basis could price.

## Corrections to the brief

1. There is no `RoadmapServiceIo` type. The writer is `RoadmapService::save()`, defined in `src/services/roadmap_service_io.rs`. The brief's own context paragraph already said so.
2. "Route it through `save()`" would have regressed the file. `save()` re-serialises the whole model, and migrate exists for hand-written legacy files (PMAT-679). The quorum ruled 3/3 for a locked text primitive in `RoadmapService` instead.
3. "Build the gate so it is 1 today and 0 after the fix": on `8915fe3e6`, a serialisation-site gate counts **11** raw writes, not 1. The 10 besides `write_migration` were locked only by convention. The rule that would make it 1 ("inside `RoadmapService` ⇒ locked") is unsound, 3/3. The count is 1 once the lock token lands with the pre-fix handler, and 0 after the fix.
4. After the fix, `pmat query --literal "fs::write(roadmap_path" --files-with-matches` is not 0. It still lists `roadmap_handler_parsing.rs` (the `ROADMAP.md` writer) and the gate's own test files, which quote the string. Only the gate is 0.
5. `apply_roadmap_changes` is confirmed as the markdown writer. Its only caller is `pmat maintain roadmap --fix`, whose `--roadmap` defaults to `ROADMAP.md`.
6. The `.bak` write one block above belongs to the same class: an unlocked write of content read before any lock. Fixed with the main write.
7. `make gate` (#1368) merged while this PR was waiting on CI. Strict branch protection required a rebase anyway, and the row went in at its extension point, followed by a fifth review round.
8. "PMAT-1363's row is still `planned` on master": true at `8915fe3e6`. #1383 completed it before this PR, so it was not carried.
9. The ticket row: this branch filed PMAT-1385 with `pmat work add --github-issue 1385`. #1383 then registered the same row on master from the open issue, with identical bytes apart from `created`/`updated`, so this branch's row commit was dropped on rebase.
10. "Label `kind:code` the way #1366 is shaped": #1366 has no GitHub label. Its roadmap row carries `kind:code`. Both the issue and the row carry it here.
11. `timeout … command cargo` fails, because `command` is a shell builtin that `timeout` cannot exec. `timeout … cargo` execs the real binary and bypasses the zsh function too; `CARGO_TARGET_DIR=/mnt/nvme-raid0/targets/pmat-D2` was set on every invocation.

## Gaps

- The delegate could not record the author model from a file; it was passed by flag.
- `.claude/agent-memory/` appeared untracked in the clone during the session. It was not created by this branch's commands, and it is not committed.

## Verdict

DONE on the code, the gate, the contract and review rounds 1 and 2's findings. Rounds 3 and 4 returned 3/3 PASS, on `bcda37008` and on its rebase `8d948e6b4`. Adding the `make gate` row changed the judged diff, so merge waits for a fifth round to return 3 PASS (`docs/audits/quorum-PMAT-1385.json`, `agreed=true`) on the head that merges, and for CI to go green.
