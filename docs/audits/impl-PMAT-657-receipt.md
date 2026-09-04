# Implementation receipt — PMAT-657 (AD-05: quality-gate lint, churn, file-size)

Spec: `docs/specifications/agentic-delivery-pmat.md` §5.1 / §9.5. Epic #1153. Branch
`PMAT-657-quality-gate-lint-churn-file-size`. Routing: `subagent:opus` (|M| ≥ 2 — enums, dispatchers,
analysis utilities, configuration); trigger Q1 not fired (no quorum lane; the AD-04 review runs on the PR).

## Dispatch ledger

| dispatch | outcome |
|---|---|
| worker adabf4516f7a653b7, brief `pmat-release/brief-PMAT-657.txt` | stopped at the scope boundary with the exact compile-forced files named (two exhaustive matches, two dispatch destructurings, two demo forwarders) and two design questions; no edits |
| resume 1 (scope widened; decisions [A]: config key `max_churn_commits_90d` with alias, sibling entry point instead of growing the 11-arg signature) | hit the 40-turn limit mid-tests |
| resume 2 (the one permitted max-turns resume) | committed ebf1580e2; receipt with `gate.ok=false` and every red test named, two of them in its forbidden list (ratchet, ledger) |

## Verification (orchestrator re-runs; the worker's numbers were claims until re-measured)

| check | worker claimed | orchestrator |
|---|---|---|
| RED on pmat 3.36.0 (`PMAT=<3.36.0> scripts/quality-gate-thresholds-audit.sh`) | — | seven ✗, all rc=2 (checks and flags do not parse), exit 1 |
| GREEN, binary from `cargo build --bin pmat --message-format json` | exit 0 | seven ✓, GREEN, exit 0 on ebf1580e2; again on the refactored tree (binary carries `QualityGateRequest`) |
| named mutation: file-size compares against `max_lines * 2` (line 138 of the checks file) | — | RED: the 502-line leg and the `--max-file-lines 400` leg both read rc=0 (`✗`), the other five stay green; mutant reverted, tree clean of it |
| ratchet literals | `allow_attributes_src` 498 > 497 (reported, not fixed — forbidden) | back to 497: the twelve-argument sibling became `handle_quality_gate_with_thresholds(QualityGateRequest)`, no new allowance; `panic!(` 783 → 781 (two test panics became `unreachable!`); `.unwrap()` 20336 = baseline |
| unrun-tests ledger | drift reported, not fixed (forbidden) | regenerated last, with the reachability ledger |
| `pmat verify` | `ok:false`, 5 stages measured, tests red on 6 (4 pre-existing worktree `git_context` failures, since fixed on master by #1174; 2 the worker's — ratchet and ledger) | on d3eaeb88a (after merging master at 5f719651a, ledgers regenerated last): exit 0 — format, satd, clippy, tests all ok; complexity withdrawn on a clean tree |
| complexity on the changed files, direct | verify's complexity stage green | `analyze complexity --max-cyclomatic 30 --max-cognitive 25 --fail-on-violation --files <25 changed files>` exit 0; a 3/2 control exit 1 |
| pv contract | — | `contracts/quality-gate-thresholds-v1.yaml`, lint PASS |
| 22 new lib tests | listed in the receipt | names re-read from the tree; they run in the verify tests stage |

## Decisions recorded as the orchestrator's ([A], conservative), never as the user's
- `[quality] max_churn_commits_90d` (the spec's name) with `#[serde(alias = "max_churn_commits")]`; flags `--max-file-lines`, `--max-churn-commits`.
- `QualityGateRequest` carries the eleven gate arguments plus the thresholds; the existing eleven-argument `handle_quality_gate` builds it and keeps its own pre-existing allowance.
- `file-size` and `churn` enter `default_checks()` (and so the MCP suite); `lint` is opt-in — it compiles the tree, and the suite must advertise exactly what it runs.
- The scope-row asymmetry under `--checks all` (inherited from the security check) is filed as PMAT-658, not silently kept.

## Jidoka
- The worker's first verify surfaced pre-existing debt in `configuration_handlers_validation.rs` (`report_settings_provenance`, cognitive 40); it backed its assertion out of that file rather than refactor under this ticket. The AD-03 branch (#1175) refactors that function.
- Four files outside even the widened scope were compile-forced by the three new `QualityGateResults` fields and edited mechanically (`examples/quality_gate_violations.rs`, `quality_checks_part3_tests.rs`, `quality_gate_formatter_tests.rs`, `tool_functions_gate_parity_tests.rs` — whose stale "no pmat gate has a lint check" message was reworded).

Verdict: **DONE** once the PR merges green; every row above re-measured by the orchestrator.
