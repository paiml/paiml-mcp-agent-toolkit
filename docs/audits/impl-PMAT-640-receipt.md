# impl receipt — PMAT-640 (CRUX-10: `quality_proxy` → `quality_check_content`, never writes)

| field | value |
|---|---|
| ticket | PMAT-640 (spec §8.10, issue #1151; epic #1153) |
| linked defect | PMAT-641 — the spec's own acceptance script contradicted itself on its preferred branch |
| branch | PMAT-640-quality-check-content (off master 5dbbfe88a, after #1161) |
| discover.json sha256 (first 16) | a99ed8eb4b1c2434 |
| phase gate | `pmat verify` (via the PMAT-637 tree build `greenbin01/pmat`, not a bare PATH binary); spec §8.10 script `scripts/quality-check-content-audit.sh` |
| DoD gate | `make gate-artifact` (CARGO_BUILD_JOBS=2) |
| quorum | never (`--quorum never`); no trigger fired (Q1 |M|=4 ≥ 3 would have, but the flag disables it) |
| subagents | 0 |

## Plan and routing

| phase | acceptance command | route |
|---|---|---|
| P1 disclosure + advisory verdict + floor + count==list (service) | `cargo test --lib -- quality_proxy proxy_` | direct |
| P2 rename + one-release alias + schema without `operation` (MCP handler, manifest, TOOLS.md, mcp.json) | `cargo test --lib -- tool_manifest quality_proxy_handler` | direct |
| P3 spec script, both branches | `PMAT=<bin> bash scripts/quality-check-content-audit.sh` | direct |
| P4 pv contract | `pv lint contracts/quality-check-content-v1.yaml` | direct |

## Verification (every row re-run by the orchestrator; no worker claims exist)

| check | result |
|---|---|
| acceptance script, pre-fix binary (`greenbin01/pmat`, PMAT-637 tree) | **exit 1 — `FAIL: B0: no boolean 'written' in the response`** (RED) |
| acceptance script, fixed binary (`/mnt/nvme-raid0/targets/paiml-mcp-agent-toolkit/release/pmat`, this tree, from `cargo build --release --message-format=json`) | **exit 0** — S selects `quality_check_content`, `write_in_enum=false`; B0 B1 B2 R1 R2 R3 R4 R5 pass (GREEN) |
| `cargo test --lib -- quality_proxy proxy_ tool_manifest mcp_json` | 122 passed, 0 failed (before the complexity refactor); `quality_proxy proxy_` 112 passed after it |
| `cargo test --test all -- quality_proxy` (integration) | 8 passed |
| property tests (`quality_proxy_property_tests`) | 8 passed; `test_quality_report_consistency` 38 s |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `pmat comply ratchet` | all 6 baselines held (unwrap 20343, satd markers 324) — re-run after the refactor: held |
| `pv validate` / `pv lint contracts/quality-check-content-v1.yaml` | valid; lint PASS, 0 errors |
| `bashrs lint scripts/quality-check-content-audit.sh` | 0 errors |
| spec §8.10 block vs repo script | byte-identical (python comparison) |
| `pmat verify --format json` run 1 | **exit 1: complexity** — `proxy_operation` cyclomatic 15 / cognitive 34 (pre-existing; surfaced because the file changed). Stop-the-line: decision and auto-fix branches extracted (`auto_fix_decision`, `operation_name`, `Decision`); no `--skip` |
| `pmat verify --format json` run 2 (dirty tree, after the refactor) | **exit 1**: format ✓ complexity ✓ (35 ms — `proxy_operation` now under the gate) satd ✓ clippy ✓ (94 s) tests ✗ — 5 of 21,123: two 19-tool pins (`test_all_live_tools_advertise_description_and_schema`, `simulated_refactor_tools_are_not_advertised`), `readme_tool_count_matches` (`\| MCP Tools \| 19 available \|`), `the_committed_baselines_have_no_slack_left_in_them` (panic_macro_calls_src 781 < 784), `the_committed_ledger_matches_the_tree` (rendered text). Each fixed at its source: pins → 20 with the retire-to-19 note; README → 20; `pmat comply ratchet --lower` (784→781); ledger regenerated on the committed tree |
| `pmat verify --format json` run 3 (committed tree eb4b30401) | **exit 0, `ok: null`, `stages_measured: 4`, `not_measured: ["complexity"]`** — format ✓ satd ✓ clippy ✓ (52 s) tests ✓ (343 s, 0 failed); complexity withdrawn because the gate measures changed files and the tree is clean (CRUX-01 semantics) — its measurement is run 2 |
| `make gate-artifact` (CARGO_BUILD_JOBS=2, committed tree) | **exit 0** — flag-efficacy sweep 509 s, 1 passed; differential leg 77 s, 1 passed |

## Named mutation (both sides)

Mutant: `src/services/quality_proxy_operations.rs`, advisory arm, `ProxyStatus::Rejected` → `ProxyStatus::Accepted` (the pre-fix behaviour: advisory laundered `passed:false`).
RED: `advisory_rejects_failing_content_and_returns_it_unwritten` FAILED ("advisory laundered passed=false as Accepted") and `test_advisory_mode_reports_the_verdict_and_returns_the_content` FAILED — `0 passed; 2 failed`.
Restored: `2 passed; 0 failed`. (A first attempt planted the mutant in the Strict arm by regex — both tests stayed green, correctly: that arm was not the defect. Recorded so the survivor is not misread as a weak test.)

## Jidoka

- PMAT-641 (this PR): spec §8.10 script — `req()` always sent `operation:"write"`, B2 hard-coded it; on the rename branch that key is refused with -32602 (R1 requires it) so B0–B2 could never pass. The branch was labelled "DEFERRED at HEAD" and had never been run. Fixed: request shape follows the selector, `req_stale` is R1's probe, selector prefers the live name, B2 built from `req()`; spec block replaced by the repo script with a correction paragraph. Five whys in `.pmat/jidoka.jsonl`.
- Complexity gate red on `proxy_operation` — refactored in the owning module (above). Not a ticket: the debt was inside the function this item changes.

## Decisions taken as the conservative option ([A] — not Noah's; nothing was asked)

- [A] `operation` removed from the schema rather than left as an empty enum; `#[serde(deny_unknown_fields)]` on the input so a stale `operation` is refused with -32602 (spec: "-32602 and nothing created"). Any unknown key is now refused, where the spec's evidence noted unknown keys used to be ignored — stricter, disclosed in the description and TOOLS.md §11.
- [A] With no `pmat.toml` above the target, `QualityConfig::default()` is the floor (B2 measured `allow_satd:true` being honoured on a bare temp file otherwise).
- [A] The alias is a separate `tools/list` entry (20 tools), so `docs/mcp/TOOLS.md` counts it (§12) and `manifest_matches_server` pins 20 with a note to drop back to 19 when the alias is retired.
- [A] `scaffold_project`/`git_operation` renames the spec lists as "independently" are not in this PR; they are separate tools and get their own ticket at triage.

## Estimates

| K̂ (per estimate.sh) | K (budget) | global turns since the governing invocation | basis |
|---|---|---|---|
| 3 per ticket | 120 | 296 at this receipt's first draft (transcript count) | `.pmat/estimates.jsonl` L3–5: 48 / ~30 / ~135 per ticket. **K was exceeded during PMAT-637**; no user message reaffirmed the budget after it was crossed. |

## Gaps

- pv contract `status: draft` until the PR merges.
- `transcript-gate.sh` reports `PASS … 0 subagents ran in …/memory/ (vacuous but honest)`: it scanned the memory directory, not the session transcript. No subagent was dispatched in this pass (the transcript count above is the orchestrator's own turns), but the gate's evidence is the wrong directory — skill defect, to file against `~/.claude/skills/paiml-implement/scripts/transcript-gate.sh`, not this repo.
- `status-lint.sh`: PASS, 7 blocks, all with `basis=`.

## Verdict

All phase and DoD gates hold; PR #1163 open (draft while gate-artifact ran, then ready) with auto-merge armed. **DONE** the moment #1163 merges green on the required checks without a rerun (recorded in the final receipt commit).
