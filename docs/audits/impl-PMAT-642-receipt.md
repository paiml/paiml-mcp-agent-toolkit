# impl receipt — PMAT-642 (CRUX-02: `quality-gate` renders three unmeasured dimensions as clean)

| field | value |
|---|---|
| ticket | PMAT-642 (spec §8.2; epic #1153; related #1035) |
| branch | PMAT-642-quality-gate-not-measured (off master 3a5b162d4, after #1163) |
| discover.json sha256 (first 16) | f4e004020f0f78c1 — `gate_cmd_fallback=true` (discover.sh still misses the Makefile's `gate-artifact`; PMAT-632) |
| phase gate | `pmat verify` (via `greenbin01/pmat`, the PMAT-637 tree build; never a bare PATH binary); spec §8.2 script `scripts/quality-gate-not-measured-audit.sh` |
| DoD gate | `make gate-artifact` (CARGO_BUILD_JOBS=2) |
| quorum | never (`--quorum never`); Q1 would have fired (|M|=4: dead-code check, coverage reader, duplicates check + results model, clap definition) |
| subagents | 0 |

## Plan and routing

| phase | acceptance command | route |
|---|---|---|
| P1 results model (`not_measured`/`not_applicable`, `identical_files`) + dead-code outcome at both gate paths | `cargo test --lib -- dead_code_outcome_tests unmeasured_shape_tests` | direct |
| P2 coverage-cache guards (git_hash, mtime, breadth) + rejection disclosure | `cargo test --lib -- coverage_sections_tests` | direct |
| P3 duplicates disclosure + rename + `--help` long_about | `bash scripts/quality-gate-not-measured-audit.sh` legs 3–4 | direct |
| P4 script both sides, pv contract, receipt | `pv lint contracts/quality-gate-not-measured-v1.yaml` | direct |

## Verification (every row re-run by the orchestrator; no worker claims exist)

| check | result |
|---|---|
| acceptance script, pre-fix binary (`greenbin01/pmat`) | **exit 1 — `FAIL: leg 1`** (no `not_measured` key at all; jq: Cannot iterate over null) — RED |
| acceptance script, fixed binary (`/mnt/nvme-raid0/targets/paiml-mcp-agent-toolkit/release/pmat`, this tree, from `cargo build --release --message-format=json`) | **exit 0, PASS** — legs 1 (+A, +B), 2 (+control), 3, 4 — GREEN |
| `cargo test --lib -- dead_code_outcome_tests` | 4 passed (broken crate → not_measured "could not compile"; compiling → none; no Cargo.toml → not_applicable; reason classifier) |
| `cargo test --lib -- coverage_sections_tests` | 36 passed incl. accepted-from-HEAD, git_hash, mtime, breadth, rejection-disclosed |
| `cargo test --lib -- unmeasured_shape_tests` | 1 passed (no `duplicate_violations` key; both lists serialize empty) |
| `cargo test --lib -- quality_gate quality_checks tests_core tests_extreme comprehensive gate_suite dead_code coverage` | 5,276 passed, 0 failed |
| `cargo clippy --all-targets -- -D warnings` | exit 0 (after fixing `examples/quality_gate_violations.rs`'s literal) |
| `pmat comply ratchet` | held; `--lower` unwrap_calls_src_total 20343→20339 (the rewritten tests dropped four) |
| `pv validate` / `pv lint contracts/quality-gate-not-measured-v1.yaml` | valid; PASS |
| `bashrs lint scripts/quality-gate-not-measured-audit.sh` | 0 errors |
| `pmat verify --format json` run 1 (committed tree b1b45b753) | **exit 1**: format ✓ complexity withdrawn (clean tree) satd ✓ clippy ✓ (96 s) tests ✗ — 1 of 21,134: `the_committed_ratchet_holds_at_head`, `panic_macro_calls_src` 787 > 781. Cause: the six new coverage-guard tests each carried an `other => panic!(..)` arm. Fixed at the source (ce2178577: `assert!(matches!(..))`, same assertions, no new `panic!`); ratchet 781 again. **Process finding:** my earlier `pmat comply ratchet` read was piped through a grep that kept only the unwrap and satd rows, so the red panic row was filtered out before `--lower` — the phase gate caught what my filtered read did not |
| `pmat verify --format json` run 2 (committed tree ce2178577) | **exit 0, `ok: null`, `stages_measured: 4`, `not_measured: ["complexity"]`** — format ✓ satd ✓ clippy ✓ (50 s) tests ✓ (303 s, 21,134 passed, 0 failed); complexity withdrawn because the tree was clean at both runs (CRUX-01 semantics) |
| complexity, measured directly with verify's own command (`analyze complexity --max-cyclomatic 30 --max-cognitive 25 --fail-on-violation --files <10 changed non-test src files>`) | **exit 0**; control at `--max-cyclomatic 5 --max-cognitive 5` → **exit 1** with 22 flagged lines, so the measurement can fail on these files |
| collateral of the substring rename (found by reading `git diff master...HEAD`) | the rename had also mangled entropy's `deduplicate_violations` helper and renamed the field of the separate `models::quality_gate` results type — both reverted (de078070a); `entropy::violation_detector` + `models::quality_gate` tests 47 passed. Follow-up PMAT-647 (that model still says `duplicate_violations`) |
| CI events on #1164 | pushes ce2178577, a80527dcb, de078070a and the ready-for-review event created **no** workflow runs (`actions/runs?head_sha=` → 0), while #1155's push at 07:37Z did; `workflow_dispatch` on the same head creates runs. Required workflows dispatched by hand on de078070a (ci, feature-matrix, docsrs, quality-gate). Filed PMAT-646; not a rerun of a failed leg — no leg had run |
| `make gate-artifact` run 1 (committed tree de078070a) | **exit 2** — flag-efficacy PASS; `gate-differential` FAILED: `metrics_must_respond_to_the_corpus` found two numeric leaves identical for the empty and the defect-rich corpus: `results.not_measured[].len = 1`, `results.not_applicable[].len = 0`. They are disclosure lists — properties of the run, not of the corpus — and the duplicates disclosure is 1 by design. Declared in `ALLOWED_CONSTANTS` with the reason and the guards that keep them falsifiable (the acceptance script's broken-crate and fabricated-cache fixtures move exactly these lists); `make gate-differential` → exit 0, `0 constant leaf/leaves` (678726838) |
| `make gate-artifact` run 2 (committed tree 678726838) | **exit 0** — gate-differential 80 s, 1 passed (`0 constant leaf/leaves`); flag-efficacy sweep 526 s, 1 passed |

## Named mutation (both sides)

Mutant: `check_dead_code_outcome`'s `Err` arm returns the empty outcome without setting `not_measured` (the pre-fix behaviour).
RED: `a_crate_that_does_not_compile_is_reported_as_not_measured` FAILED ("not_measured must be set for an uncompilable crate") — 3 passed; 1 failed. Restored: 4 passed.

## Jidoka

None filed: no defect outside this item's scope surfaced. One pre-existing test (`test_read_coverage_from_cache_detail_preferred_over_metrics`) encoded the old trust in a report from nowhere; rewritten on a git fixture in the same PR.

## Decisions taken as the conservative option ([A] — not Noah's; nothing was asked)

- [A] Two lists (`not_measured`, `not_applicable`), both always serialized, entries `{check, path, reason}`.
- [A] Breadth guard denominator = the tree's Rust source files (`.rs`, gitignore honoured), not `files_examined` (which counts every extensioned file); N = 25 %. Guards run cheapest-first; the first to trip is the reason.
- [A] A rejected detail cache still falls through to `.pmat-metrics/coverage.json` if that exists; the rejection finding is emitted only when no other report is available.
- [A] Leg 4 satisfied with a clap `long_about` (shown by `--help`), not the one-line `about`.

## Estimates

| K̂ | K | turns this invocation at draft | basis |
|---|---|---|---|
| 4 [U] (estimate.sh: 2 numeric rows, below its 3-row floor) | 150 | ~50 | `docs/audits/impl-estimates.jsonl` L1–L5 (48, 63 per ticket) |

## Gaps

- pv contract `status: draft` until the PR merges.
- `transcript-gate.sh` scans the memory directory (vacuous PASS) — skill defect, not this repo; 0 subagents were used.

## Verdict

All phase and DoD gates hold; PR #1164 open with auto-merge armed (CI runs created by `workflow_dispatch` on the head — see the CI-events row and PMAT-646; no leg was rerun). **DONE** the moment #1164 merges green on the required checks (recorded in the next receipt commit).
