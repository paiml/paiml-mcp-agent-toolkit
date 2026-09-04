# Implementation receipt — PMAT-665 (CRUX-07: the index is a faithful, reproducible view of the tree)

Spec: `docs/specifications/pmat-architecture-crux-audit.md` §8.7. Epic #1153. Branch `PMAT-665-crux07-index-faithful`.
Routing: `subagent:opus` for the five legs (|M| ≥ 3: function_index, churn, indexing, score handler, sqlite_docs), `subagent:sonnet` for the mechanical complexity refactor, orchestrator direct for the last two functions and every measurement.

## Dispatch ledger

| dispatch | outcome |
|---|---|
| worker a2d51c9269d15080a (opus), phase 1 | 40 turns, then one resume: committed fdede2e2e (41 files, legs a–e); stopped before the gate |
| worker a4bfe61d3862a6884 (sonnet), phase 2 (nine over-ceiling functions) | 40 turns: seven of nine under the ceiling, compiling, uncommitted |
| orchestrator | the last two (`compute_evoscore`: record reading extracted; the test whose raw-string fixture the analyzer counted as nesting: fixture hoisted to a const); two clippy findings from the refactor (a split doc comment, `is_multiple_of`) fixed; commit d1f6eaa70 through the hook |

## Verification (orchestrator runs)

| check | result |
|---|---|
| RED on pmat 3.36.0 | legs a (both assertions), b, c1, d, e ✗; the five controls ✓ (fast path survives; c2; author set = shortlog; clean pair; writable) — committed as 708bd9bdc |
| GREEN on fdede2e2e (binary from `cargo build --message-format json`) | eleven ✓, exit 0; targeted tests 704 passed |
| GREEN on d1f6eaa70 (after the refactor) | eleven ✓, exit 0; 903 targeted tests (`function_index churn manifest indexing score_handler git_analysis tests_core_part2`) passed |
| named mutation F3 (fast path always re-hashes: `if true \|\| mtime >= built_at`) | RED on the fast-path-survival control (`got: Checking for incremental updates...` — no `mtime-skipped` line) while leg a stays green — the control discriminates deletion of the optimisation. The FIRST attempt was invalid: the mutant did not compile, `BIN` was empty and the audit silently ran `pmat` from PATH (the 3.36.0 binary) — leg a red, control green, the released binary's signature. The script now refuses an empty or bare-name `PMAT` (exit 2). Mutant reverted and the binary REBUILT before any further measurement |
| complexity, direct, on the 38 changed files | before the refactor: nine functions over cognitive 25 (`build` 44, `compute_contract_drift` 58, …); after: gate 30/25 exit 0, 3/2 control exit 1 |
| ratchet literals | `.unwrap()` 20336, `panic!(` 781, `#[allow(` 497 — at baseline |
| **AD-04 quorum on 784256add** (run 1: three empty lanes — `agy: Argument list too long`, the 142 KB brief exceeded MAX_ARG_STRLEN; bundle #13 added a file-prompt mode; run 2 in file mode) | 3 FAIL, cited and right: the rotating 1/64 verify slice was absent; the ranker tie-break was never implemented (only the walks were sorted — leg b passed because persisted order = sorted); the manifest-older-than-db detector was missing (only torn/unparsable); the success line preceded the save; the complexity refactor was "unrequested" (the ticket description now records it) |
| phase 3 (worker a80f8a34b059bcc9, opus, one run) | `run_counter` persisted in the manifest and a 1/64 slice re-hashed per run (`in_verify_slice`); `rank_order` — score desc, then `(file_path, start_line)` — at every ranked sort; a stale-pair detector (manifest older than the db → "stale … rebuild"); "Index saved (N changes)" printed only after `save` returns Ok. Orchestrator re-ran: 906 targeted tests, acceptance eleven ✓, complexity gate 30/25 exit 0 on 53 changed files, ratchet literals at baseline, `cargo check --features full` clean |
| `pmat verify` on 784256add | red on exactly `the_committed_ratchet_holds_at_head`: `unwrap_calls_shipped_code` UNMEASURABLE — `cargo clippy --features full` failed on a feature-gated `HashMap::new()` constructor the BTreeMap change had missed (`src/demo/server_content.rs`); fixed 8910ff688, the metric reads 0 again. The ratchet reporting a build error as "unmeasurable, not compliant" is the design working |
| `pmat verify` on the final tree | VERIFY-ROW |
| pv contract | `contracts/index-faithful-v1.yaml` — `pv validate` and `pv lint` PASS |
| reported, not judged | `author_contributions` counts differ from `git shortlog -sn` on the fixture (ann 2 / bob 1 / cid 1 vs 2 / 2 / 2); PMAT-667 |

## Jidoka
- An empty `PMAT` in the acceptance script fell back to the PATH binary and measured the wrong thing for one mutation run; the script now refuses (the bare-binary trap the doctrine names, caught in the harness itself).
- The complexity analyzer counts nesting inside a raw-string literal as the enclosing function's cognitive complexity (`test_check_single_file_complexity_violations`: 26 with no branches of its own). Worked around by hoisting the fixture; the analyzer defect itself is not this ticket's.

Verdict: PENDING
