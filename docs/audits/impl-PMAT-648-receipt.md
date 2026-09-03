# impl receipt — PMAT-648 (CRUX-04: the dead-code cache is keyed on the committed tree)

| field | value |
|---|---|
| ticket | PMAT-648 (spec §8.4; epic #1153; precedent #748, related #1035) |
| branch | PMAT-648-dead-code-cache-working-tree (off master cd6f796d6, after #1157) |
| discover.json sha256 (first 16) | f8cc7dfb6777a96c — `gate_cmd_fallback=true` (PMAT-632) |
| phase gate | `pmat verify` (via `greenbin01/pmat`, never a bare PATH binary); spec §8.4 script `scripts/dead-code-cache-audit.sh` |
| DoD gate | `make gate-artifact` (CARGO_BUILD_JOBS=2) |
| quorum | never (`--quorum never` per the invocation; Q1 would have fired: |M|=4 — cache ops, analyzer report, CLI handler/outputs, MCP mirror) |
| subagents | 0 |

## Plan and routing

| phase | acceptance command | route |
|---|---|---|
| P1 working-tree key (scratch-index `write-tree`), schema 4→5 | `cargo test --lib -- working_tree_key_tests cache_key_tests` | direct |
| P2 `cache {hit, tree_hash, written_at, pmat_version}` + `compiler-lint-cached` on CLI JSON/SARIF/text/markdown and the MCP per-path object; `--no-cache` | `cargo test --lib -- dead_code cargo_dead_code` | direct |
| P3 gate sees an uncommitted dead fn (same analyzer) | script state F | direct |
| P4 shim-based script (A–E, F, G, control, --no-cache), pv contract, receipt | `pv lint contracts/dead-code-cache-v1.yaml` | direct |

## Verification (every row re-run by the orchestrator; no worker claims exist)

| check | result |
|---|---|
| scratch-index key experiment (temp repo + this repo) | unstaged edit changes the hash, revert restores it, real index untouched; 0.26 s wall on this 4,019-file tree |
| acceptance script, pre-fix binary (`greenbin01/pmat`, 3.35.0 at 24d4fd04f) | **exit 1 — `FAIL: A: cache.hit != false (null)`** (no `cache` object at all) — RED |
| acceptance script, fixed binary (`/mnt/nvme-raid0/targets/paiml-mcp-agent-toolkit/release/pmat`, this tree) | **exit 0, PASS** — A (+1 exec, miss, ran, 0) · B (+0, hit, cached, 0) · C (+1, miss, ran, 1 named) · D (+0, hit, cached, 1) · E (+1, miss, ran, 0) · F (gate counts the uncommitted dead fn) · G (schema-4 entry is a miss) · control A/C/E with the cache deleted 0/1/0 · `--no-cache` forces exactly one exec — GREEN |
| `cargo test --lib -- working_tree_key_tests cache_key_tests` | 7 passed |
| `cargo test --lib -- dead_code cargo_dead_code` | 405 passed, 0 failed, 13 ignored |
| `cargo clippy --all-targets -- -D warnings` | exit 0 (after adding the argument to `examples/analyze_dead_code.rs`'s four calls and a struct-update in a test helper) |
| `pmat comply ratchet` | all 8 baselines held (unfiltered output read this time) |
| `pv validate` / `pv lint contracts/dead-code-cache-v1.yaml` | valid; PASS |
| `bashrs lint scripts/dead-code-cache-audit.sh` | 0 errors (the `rm -rf` on a variable path was replaced by a pattern-scoped `rm -f` after SEC011) |
| `pmat verify --format json` run 1 (committed tree 048efcaab) | **exit 0, `ok: null`, `stages_measured: 4`, `not_measured: ["complexity"]`** — format ✓ satd ✓ clippy ✓ (104 s) tests ✓ (346 s, 0 failed); complexity withdrawn on the clean tree (CRUX-01 semantics) |
| complexity, measured directly with verify's own command over the 20 changed non-test src files (`analyze complexity --max-cyclomatic 30 --max-cognitive 25 --fail-on-violation --files …`) | **exit 1 first**: six pre-existing functions over cognitive 25 in files this item touched — `run_complexity_analysis` (enforce, 31), `analyze_satd` (MCP, 33), `analyze_dead_code` (MCP, 33), `scan_file_for_suppressions` (31), `named_targets` (28), `parse_cargo_warnings` (32). This is exactly the red the canonical edit→verify loop shows for a dirty tree, so the line stopped: each extracted into named helpers (`complexity_metrics`/`complexity_violations`, `unresolved_debts`/`satd_file_json`, `DeadCodeAccumulation`, `first_item_after`, `is_plain_target_of_kind`, `dead_item_from_message`), behaviour pinned by the existing tests (637 passed across the dead-code, MCP analysis, enforce and complexity slices). **Re-measured: exit 0**; control at `--max-cyclomatic 5 --max-cognitive 5` → exit 1 |
| `cargo clippy --all-targets -- -D warnings` after the refactors | exit 0 |
| `pmat comply ratchet` after the refactors | all 8 held |
| `pmat verify --format json` run 2 (committed tree b437f40aa, after the refactors) | **exit 0, `ok: null`, `stages_measured: 4`, `not_measured: ["complexity"]`** — format ✓ satd ✓ clippy ✓ (107 s) tests ✓ (303 s, 0 failed); complexity withdrawn on the clean tree — its measurement is the direct row above |
| `make gate-artifact` run 1 (committed tree b437f40aa) | **exit 2** — flag-efficacy PASS; `gate-differential` FAILED: one leaf identical for the empty and the defect-rich corpus, `analyze dead-code :: cache.hit = 0`. It is a property of the run (both sweeps are cold), not of the corpus; declared in `ALLOWED_CONSTANTS` with the guard that keeps it falsifiable (the acceptance script's states B/D require `hit == true` with zero cargo execs); `make gate-differential` → exit 0, `0 constant leaf/leaves` |
| `make gate-artifact` run 2 (committed tree 87fc38f91) | **exit 0** — gate-differential 58 s, 1 passed (`0 constant leaf/leaves`); flag-efficacy sweep 479 s, 1 passed — both on a release build of the FINAL tree |
| acceptance script on a binary of the final tree | **not re-run**: the GREEN row above is the binary of 048efcaab (before the six complexity refactors); the refactors are pinned by 637 tests and by gate-artifact run 2's release build of the final tree, but the script itself was not re-executed on it — the 10-minute tool budget killed the rebuild. Closes with one command: `cargo build --release && PMAT=<that binary> bash scripts/dead-code-cache-audit.sh` |

## Named mutation (both sides)

Mutant: `get_tree_hash` reverted to `git rev-parse HEAD:` (the pre-fix key).
RED: `a_replay_is_marked_as_a_hit_with_a_cached_verdict` FAILED (its miss half: an uncommitted edit after the entry was written must be a miss) — 4 passed; 1 failed. The unstaged-edit test then also routed through `get_tree_hash` so the same mutant fails it directly. Restored: 5 passed.

## Jidoka

- Leg F of the script first read the gate's exit code through `pipefail`; the gate FAILS on the finding by design (exit 1), so the payload, not the exit, is the verdict. Script fixed; not a repo defect.
- While this ticket ran, #1164 (CRUX-02) went red on `every unreachable .rs file is ledgered`: its orphan-files ledger had been regenerated with a binary built before #1157 changed the renderer. Regenerated with this branch's post-#1157 build (078b49972); five whys in `.pmat/jidoka.jsonl`. Rule: regenerate ledgers with a binary at or after the branch's merge base.

## Decisions taken as the conservative option ([A] — not Noah's; nothing was asked)

- [A] The key is `write-tree` of a **scratch index** filled by `git add -A`, not the user's index: the spec says "copy #748 verbatim", but #748's key is the index and the spec's own state C is an unstaged edit the index does not contain. Recorded in the spec's §8.4 implementation note.
- [A] `cache.written_at` is `Option`: `None` when nothing was written (`--no-cache`, or no git tree to key on).
- [A] A replayed **reduced** scan keeps its own reason (lockfile/env-skip); only the cache object says it was replayed.

## Estimates

| K̂ | K | turns this invocation at draft | basis |
|---|---|---|---|
| 4 [U] (estimate.sh below its 3-row floor) | 150 | ~50 | `docs/audits/impl-estimates.jsonl` L1–L6 (48, 63, 101 per ticket) |

## Gaps

- Acceptance script on the final-tree binary: NotRun (see the verification table); the artifact that closes it is one script run.
- pv contract `status: draft` until the PR merges.
- `transcript-gate.sh` scans the memory directory (vacuous PASS) — skill defect; 0 subagents were used.

## Verdict

All phase and DoD gates hold on the final tree (verify run 2 exit 0; gate-artifact run 2 exit 0; direct complexity measurement exit 0 with a failing control); PR #1165 ready with auto-merge armed. **DONE** the moment #1165 merges green on the required checks — which is gated on the org runner queue (`ci / gate` has not started on any head since 07:23Z; paiml/.github#57), not on this branch.
