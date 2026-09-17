# impl-PMAT-636 — receipt

Verdict at this commit: **PARTIAL(blocker)**. The branch is green on everything it owns. The merge is blocked on the quorum round and on master's CB-2115 state (below). A commit cannot record its own merge, so the merge outcome is in the session's closing JSON receipt, not in this file.

## Identity

| field | value |
|---|---|
| ticket | PMAT-636 (issue #1266), `kind:code` |
| branch | `PMAT-636-cb200-back-under-baseline`, PR #1394 |
| base | `origin/master` `f25d7f1cc` (behind = 0 at every measurement below, unless a row names another tree) |
| discovery | `discover.json` sha256 prefix `c68a87a1ff488d21`. `gate_cmd_fallback=true`: discovery still reports `cargo test --workspace`, so this session ran `make gate` itself |
| sessions | 2. The first (k_measured 94) was killed by the 15:30Z host crash. The second resumed from the tree |

## Scope, and why the refactors are in it

The ticket row names the symptom: the ratchet drifted and no leg could see it. The release-3.41.0 brief that dispatched this work set three jobs, now recorded on the row's `acceptance_criteria` through `pmat work edit`:

1. re-measure with one binary and publish the set difference;
2. bring the measured count back to at most the banked 1688 through behaviour-preserving refactors of below-A definitions, then bank the measured count, which may only go down;
3. make a required CI job measure it and go red on a planted below-A definition.

Raising the baseline to 1741 was forbidden, and a gate that goes red on master the moment it lands would block the release. So job 2 is the refactors, and it is the only way to satisfy job 3 without raising the number.

Every extracted helper and every new test serves job 2. The two new tests, `2c8fdc395` (deep-context text high-complexity highlighting) and `de14fdb51` (markdown TDG component table), pin output before the extraction that touches it. That is the "covered by existing or new tests" condition for a behaviour-preserving refactor.

**Note for reviewers (load discipline).** The host running this review has hard-crashed four times in 24 hours, and load is a suspect. Please do not run `cargo build`, `cargo test`, `make gate`, `pmat comply check` or an index build in this checkout: one review round left a 15 GB `target/`. Every measurement above names the command, tree and binary that produced it, and the CI runs on the PR re-execute them on GitHub's runners.

## Measurement: same binary, cold indexes, clean clones

The binaries are `pmat` built at `ce945d81e` (`.pmat/pmat-636/bin/pmat-master-ce945d81e`) and at `683d6994d`, the commit that banked 1688 (`git log -S'baseline = 1688' -- .pmat-gates.toml`). The scope is CB-200's own: non-test paths minus `.pmat.yaml` and `.pmat-gates.toml` excludes. A replica of that scope reproduced CB-200's count exactly on every tree it was checked against.

- same binary (pmat-master-ce945d81e) on 683d6994d: 1688; banked binary (pmat-banked-683d6994d) on 683d6994d: 1688; multiset equal: True
- same binary on ce945d81e: 1741 (+53): 85 newly below A, 32 no longer below A
- this branch (integrated slices, same binary): 1680: 61 brought to grade A (33 of them among the 85 new), 0 newly below A

<details><summary>newly below A, 683d6994d → ce945d81e (85)</summary>

| file | definition |
|---|---|
| `build.rs` | `emit_build_provenance` |
| `build_support.rs` | `rerun_if_changed_paths_exist_inside_the_tree` |
| `src/cli/analysis_utilities/quality_checks_part2_coverage_sections.rs` | `coverage_cache_guard` |
| `src/cli/handlers/analysis_handlers/mod.rs` | `report_reachability_ledger_drift` |
| `src/cli/handlers/analysis_handlers/mod.rs` | `route_reachability` |
| `src/cli/handlers/complexity_handlers/analysis.rs` | `analyze_files_by_mode_with_census` |
| `src/cli/handlers/comply_handlers/check_handlers/check.rs` | `compliance_check_groups` |
| `src/cli/handlers/comply_handlers/check_handlers/check.rs` | `run_check_groups` |
| `src/cli/handlers/comply_handlers/check_handlers/check_contract_surfaces.rs` | `record` |
| `src/cli/handlers/comply_handlers/check_handlers/check_evidence_gates.rs` | `ci_gated_shortfalls` |
| `src/cli/handlers/comply_handlers/check_handlers/check_roadmap_inputs.rs` | `github_inputs` |
| `src/cli/handlers/comply_handlers/check_handlers/check_spec_epics.rs` | `check_spec_epics` |
| `src/cli/handlers/comply_handlers/check_handlers/check_spec_reviews.rs` | `check_spec_reviews` |
| `src/cli/handlers/comply_handlers/check_handlers/check_tdg_grade.rs` | `resolve_tdg_index` |
| `src/cli/handlers/comply_handlers/check_handlers/check_traceability.rs` | `check_commit_traceability` |
| `src/cli/handlers/comply_handlers/numeric_claims_handler.rs` | `handle_numeric_claims` |
| `src/cli/handlers/configuration_handlers_validation.rs` | `validate_configuration` |
| `src/cli/handlers/hooks_command_handlers/command_dispatch.rs` | `consult_hooks_cache` |
| `src/cli/handlers/hooks_command_handlers/hook_debt_scope.rs` | `diff_scoped_verdict` |
| `src/cli/handlers/hooks_command_handlers/hook_debt_scope.rs` | `staged_verdict` |
| `src/cli/handlers/new_tdg_handler.rs` | `sarif_project_result` |
| `src/cli/handlers/popper_score_format_text.rs` | `format_text_category` |
| `src/cli/handlers/query_handler/indexing.rs` | `maybe_save_incremental` |
| `src/cli/handlers/tdg_handlers/formatting.rs` | `format_tdg_score_markdown` |
| `src/cli/handlers/work_handlers/core_handlers/handlers.rs` | `handle_work_cot_derive` |
| `src/cli/handlers/work_handlers/core_handlers/sync.rs` | `handle_work_sync` |
| `src/cli/handlers/work_handlers/core_handlers/sync.rs` | `print_plan` |
| `src/cli/handlers/work_handlers/ticket_crud.rs` | `handle_work_add` |
| `src/cli/language_analyzer/ast_fallback.rs` | `tally` |
| `src/mcp_pmcp/tool_functions/analysis_tools.rs` | `add_path` |
| `src/services/agent_context/function_index/build.rs` | `index_one_file` |
| `src/services/agent_context/function_index/build.rs` | `push_chunk_functions` |
| `src/services/agent_context/function_index/build_persistence.rs` | `load` |
| `src/services/commit_traceability/mod.rs` | `default_branch_mode` |
| `src/services/commit_traceability/mod.rs` | `measure_against` |
| `src/services/commit_traceability/mod.rs` | `pr_mode` |
| `src/services/complexity/formatting.rs` | `format_offenders` |
| `src/services/deep_context/analyzer_core/spawn.rs` | `execute_parallel_analyses_with_progress` |
| `src/services/duplicate_detector/engine.rs` | `collapse_identical_signatures` |
| `src/services/gate_effect/invocation.rs` | `collect_from_script` |
| `src/services/git_analysis.rs` | `capture_git_log_output` |
| `src/services/git_analysis.rs` | `parse_log_into_file_stats` |
| `src/services/metrics_ratchet/measure.rs` | `measure_analyzer` |
| `src/services/numeric_claims/annotate.rs` | `tokenize` |
| `src/services/numeric_claims/cohort.rs` | `find_replicated_divergence` |
| `src/services/numeric_claims/corpus.rs` | `collect` |
| `src/services/numeric_claims/extract.rs` | `byte_factor` |
| `src/services/numeric_claims/extract.rs` | `candidates` |
| `src/services/numeric_claims/extract.rs` | `comment_of` |
| `src/services/numeric_claims/extract.rs` | `extract_file` |
| `src/services/numeric_claims/frame.rs` | `label` |
| `src/services/numeric_claims/frame.rs` | `scan_lines` |
| `src/services/numeric_claims/frame.rs` | `structural_drop` |
| `src/services/numeric_claims/mod.rs` | `as_str` |
| `src/services/numeric_claims/mod.rs` | `title` |
| `src/services/numeric_claims/rules.rs` | `breach` |
| `src/services/numeric_claims/rules.rs` | `c1_self_breach` |
| `src/services/numeric_claims/rules.rs` | `c5_named_xref` |
| `src/services/numeric_claims/rules.rs` | `resolve` |
| `src/services/path_glob.rs` | `expand_paths_with_extensions` |
| `src/services/quality_proxy_analysis.rs` | `run_lint_checks` |
| `src/services/reachability.rs` | `walk` |
| `src/services/reachability_ledger.rs` | `check` |
| `src/services/reachability_ledger.rs` | `render` |
| `src/services/roadmap_text.rs` | `append_item` |
| `src/services/spec_epic/mod.rs` | `bind_epics` |
| `src/services/spec_epic/mod.rs` | `class` |
| `src/services/spec_epic/mod.rs` | `process_line` |
| `src/services/spec_epic/mod.rs` | `render` |
| `src/services/spec_epic/mod.rs` | `render` |
| `src/services/spec_epic/mod.rs` | `strip_inline_comment` |
| `src/services/spec_review/mod.rs` | `class` |
| `src/services/spec_review/mod.rs` | `judge` |
| `src/services/spec_review/mod.rs` | `render` |
| `src/services/spec_review/record.rs` | `record` |
| `src/services/spec_review/record.rs` | `render` |
| `src/services/tdg_baseline.rs` | `measure_below_floor` |
| `src/services/tdg_baseline.rs` | `the_committed_baseline_is_the_measured_count` |
| `src/services/work_sync/github.rs` | `parse_snapshot` |
| `src/services/work_sync/linkage.rs` | `class` |
| `src/services/work_sync/linkage.rs` | `render` |
| `src/services/work_sync/mod.rs` | `apply_to_roadmap` |
| `src/services/work_sync/mod.rs` | `check` |
| `src/services/work_sync/mod.rs` | `plan` |
| `src/services/work_sync/mod.rs` | `render` |

</details>

<details><summary>no longer below A, 683d6994d → ce945d81e (32)</summary>

| file | definition |
|---|---|
| `src/cli/handlers/complexity_handlers/analysis.rs` | `analyze_files_by_mode` |
| `src/cli/handlers/complexity_handlers/analysis.rs` | `unanalyzed_summary` |
| `src/cli/handlers/comply_cb_detect/spec_work_traceability.rs` | `collect_ticket_ids` |
| `src/cli/handlers/comply_cb_detect/spec_work_traceability.rs` | `detect_cb148_spec_work_gaps` |
| `src/cli/handlers/comply_handlers/check_handlers/check.rs` | `build_all_compliance_checks` |
| `src/cli/handlers/comply_handlers/check_handlers/check_best_practices.rs` | `check_shell_makefile_quality` |
| `src/cli/handlers/comply_handlers/check_handlers/check_contract_surfaces.rs` | `check_contract_surface_classification` |
| `src/cli/handlers/comply_handlers/check_handlers/check_contract_surfaces.rs` | `check_tui_widget_contracts` |
| `src/cli/handlers/comply_handlers/check_handlers/check_contract_surfaces.rs` | `extract_dep_version` |
| `src/cli/handlers/dead_code_handlers_output.rs` | `write_dead_code_header` |
| `src/cli/handlers/hooks_command_handlers/command_dispatch.rs` | `handle_run` |
| `src/cli/handlers/score_handler.rs` | `compute_evoscore` |
| `src/cli/handlers/score_handler_compute.rs` | `compute_contract_drift` |
| `src/cli/handlers/score_handler_compute.rs` | `compute_pipeline_depth` |
| `src/cli/handlers/score_handler_compute.rs` | `compute_pv_lint` |
| `src/cli/handlers/score_handler_compute.rs` | `cross_validate` |
| `src/cli/handlers/work_falsification/deny_refresh.rs` | `run_with_timeout` |
| `src/cli/language_analyzer/mod.rs` | `is_included_by_sibling` |
| `src/contracts/mcp_impl_server.rs` | `handle_tool_call` |
| `src/mcp_pmcp/quality_proxy_handler_impl.rs` | `handle` |
| `src/mcp_pmcp/tool_functions/analysis_tools.rs` | `analyze_dead_code` |
| `src/services/agent_context/function_index/build.rs` | `build` |
| `src/services/cargo_dead_code_analyzer/analysis.rs` | `named_targets` |
| `src/services/cargo_dead_code_analyzer/parsing.rs` | `parse_cargo_warnings` |
| `src/services/git_analysis.rs` | `get_file_metrics` |
| `src/services/hardcoded_paths.rs` | `candidates` |
| `src/services/hardcoded_paths.rs` | `classify` |
| `src/services/hardcoded_paths.rs` | `feed` |
| `src/services/path_glob.rs` | `expand_paths_to_source_files` |
| `src/services/quality_proxy_operations.rs` | `proxy_operation` |
| `src/services/reachability.rs` | `analyze` |
| `src/services/satd_detector/detection_analysis.rs` | `collect_files_including_tests` |

</details>

<details><summary>brought to grade A by this branch (61)</summary>

| file | definition | new since 683d6994d |
|---|---|---|
| `src/ast/polyglot/language_mapper_base.rs` | `map_directory` | |
| `src/cli/analysis_utilities/churn.rs` | `write_summary_top_files` | |
| `src/cli/analysis_utilities/quality_checks_part2_coverage_sections.rs` | `coverage_cache_guard` | yes |
| `src/cli/analysis_utilities/quality_gate_config.rs` | `load_entropy_gate_config` | |
| `src/cli/defect_prediction_formatters.rs` | `format_summary_output` | |
| `src/cli/handlers/advanced_analysis_handlers.rs` | `format_deep_context_text` | |
| `src/cli/handlers/comply_handlers/check_handlers/check.rs` | `run_check_groups` | yes |
| `src/cli/handlers/comply_handlers/check_handlers/check_commit_enforcement.rs` | `check_hook_performance` | |
| `src/cli/handlers/comply_handlers/check_handlers/check_evidence_gates.rs` | `ci_gated_shortfalls` | yes |
| `src/cli/handlers/comply_handlers/cross_crate_handlers/helpers.rs` | `compute_signatures` | |
| `src/cli/handlers/comprehensive_analysis_handler/output.rs` | `format_as_sarif` | |
| `src/cli/handlers/configuration_handlers_validation.rs` | `validate_configuration` | yes |
| `src/cli/handlers/hooks_command_handlers/hook_debt_scope.rs` | `diff_scoped_verdict` | yes |
| `src/cli/handlers/hooks_command_handlers/hook_debt_scope.rs` | `staged_verdict` | yes |
| `src/cli/handlers/kaizen_handler/scanning_analysis.rs` | `comply_findings_from_json` | |
| `src/cli/handlers/new_tdg_handler.rs` | `sarif_project_result` | yes |
| `src/cli/handlers/popper_score_format_text.rs` | `format_text_category` | yes |
| `src/cli/handlers/tdg_handlers/formatting.rs` | `format_tdg_score_markdown` | yes |
| `src/cli/handlers/work_handlers/core_handlers/handlers.rs` | `handle_work_cot_derive` | yes |
| `src/cli/handlers/work_handlers/ticket_crud.rs` | `handle_work_add` | yes |
| `src/cli/language_analyzer/ast_fallback.rs` | `tally` | yes |
| `src/contracts/contract_validation.rs` | `validate` | |
| `src/graph/aprender_adapter_conversion.rs` | `extract_edge_weight` | |
| `src/graph/builder_import_parsing.rs` | `parse_typescript_imports` | |
| `src/graph/builder_symbol_parsing.rs` | `parse_rust_symbols` | |
| `src/graph/builder_symbol_parsing.rs` | `parse_typescript_symbols` | |
| `src/handlers/resources.rs` | `handle_resource_read` | |
| `src/handlers/tools/extended_tools_complexity.rs` | `format_complexity_output` | |
| `src/maintenance/ticket_parsing.rs` | `list_tickets` | |
| `src/maintenance/updater.rs` | `format_roadmap_markdown` | |
| `src/models/roadmap_status.rs` | `levenshtein_distance` | |
| `src/red_team/intent_classifier_core.rs` | `aggregate_signals` | |
| `src/roadmap/commands/commands_tasks.rs` | `handle_start` | |
| `src/services/agent_context/function_index/build_persistence.rs` | `load` | yes |
| `src/services/commit_traceability/mod.rs` | `default_branch_mode` | yes |
| `src/services/complexity/formatting.rs` | `format_offenders` | yes |
| `src/services/deep_context/analyzer_core/spawn.rs` | `execute_parallel_analyses_with_progress` | yes |
| `src/services/duplicate_detector/engine.rs` | `collapse_identical_signatures` | yes |
| `src/services/file_split_graph.rs` | `connected_components` | |
| `src/services/gate_effect/invocation.rs` | `collect_from_script` | yes |
| `src/services/git_analysis.rs` | `capture_git_log_output` | yes |
| `src/services/git_analysis.rs` | `parse_log_into_file_stats` | yes |
| `src/services/metrics_ratchet/mod.rs` | `run` | |
| `src/services/numeric_claims/cohort.rs` | `find_replicated_divergence` | yes |
| `src/services/numeric_claims/frame.rs` | `scan_lines` | yes |
| `src/services/numeric_claims/frame.rs` | `structural_drop` | yes |
| `src/services/numeric_claims/rules.rs` | `c1_self_breach` | yes |
| `src/services/numeric_claims/rules.rs` | `c5_named_xref` | yes |
| `src/services/numeric_claims/rules.rs` | `resolve` | yes |
| `src/services/path_glob.rs` | `expand_paths_with_extensions` | yes |
| `src/services/polyglot_analyzer_dependencies.rs` | `count_files_recursive` | |
| `src/services/quality_proxy_analysis.rs` | `run_lint_checks` | yes |
| `src/services/roadmap_text.rs` | `append_item` | yes |
| `src/services/spec_epic/mod.rs` | `bind_epics` | yes |
| `src/services/spec_epic/mod.rs` | `process_line` | yes |
| `src/services/spec_review/record.rs` | `record` | yes |
| `src/services/work_sync/mod.rs` | `render` | yes |
| `src/tdg/cuda_simd/detection_barriers.rs` | `detect_memory_patterns` | |
| `src/tdg/quality_gate/critical_defect.rs` | `check` | |
| `src/tdg/tdg_graph_viz.rs` | `to_vis_graph` | |
| `src/unified_quality/metrics.rs` | `quality_score` | |

</details>


**Where the "10" comes from (1752 vs 1742).** On `ce945d81e`, `src/` holds 1751 below-A rows. CB-200 subtracts 21 under `src/cli/command_dispatcher/**` (a `.pmat-gates.toml` exclude) and adds 11 outside `src/`: `fuzz/` 6, `build.rs` 4, `build_support.rs` 1. That gives 1751 − 21 + 11 = 1741. The `pmat query`-terms count reported earlier is the `src/` count, and 1752 − 21 + 11 = 1742 matches the CB-200 figure measured on `441d198e7`. The offset is reproduced on `ce945d81e`; `441d198e7` itself was not re-indexed.

**After.** CB-200's own verdict at `35da3cc6d`, from the pmat built from this branch over a cold out-of-tree index: `1680 definition(s) below minimum grade A across 969 file(s) … 8 under the recorded baseline`. The baseline is banked 1688 → 1680. At `238cefa09`, the strict twin over an in-project index built by the same binary with `--no-docs`: `CB-200: 1680 of 25296 indexed definitions are below grade A (recorded baseline Some(1680))`, PASS.

## Gate: RED before GREEN

| control | RED | GREEN |
|---|---|---|
| CI `tdg-ratchet`, planted `pmat_636_planted_below_a` (A-, complexity 10) | run 35247840393 job 105292469325 on `ed99987b5`: control 10/10 arms GREEN, then `OVER: 1681 definitions below grade A, 1 over the recorded baseline of 1680`, exit 1 | the planted commit is reverted in the next commit; the CI run on the final head is in the PR |
| `scripts/cb200-ratchet-gate.sh --classify`, CB-200's report on `ce945d81e` against master's `.pmat-gates.toml` | `OVER: 1741 … 53 over the recorded baseline of 1688` | `make gate` leg `tdg-ratchet` PASS: `1680 definitions below grade A, exactly the recorded baseline of 1680` |
| `make_gate_tests::the_tdg_ratchet_job_reaches_the_required_gate_check`, with `needs.tdg-ratchet.result` removed from `gate`'s loop | FAIL: `ci.yml gate needs tdg-ratchet but never reads its result` | PASS, and the other 6 `make_gate_tests` pass |
| `scripts/gate-control.sh` arm 12 FRESH-BINARY | FAIL on `f05e7f1b7`: `legs never ran the pmat cargo built from the tree: tdg-ratchet-control tdg-ratchet`. The script took `--pmat BIN`, and arm 12 stubs a leg's scripts with `exec "$1"` | GREEN on `8409d8cd1`, 14 legs |
| `make gate` on `8409d8cd1` | 27 PASS, 3 FAIL: `lib-tests` and `unrun-tests`, both the unrun-tests ledger (+3 lib tests), and `cb-2113-cb-2115` (below) | the ledger is re-rendered in `238cefa09` (24305/27432 → 24308/27435); `--check-ledger` exit 0; `the_committed_ledger_matches_the_tree` PASS |

## Five whys

1. **Why did master reach 1741 against a banked 1688 while every required check was green?** No required job ran CB-200's comparison.
2. **Why not `ci / gate`'s `cargo test --lib`?** `the_committed_baseline_is_the_measured_count` needs `.pmat/context.db`. A CI checkout has none, and the no-index branch passes.
3. **Why not the ladder's `pmat comply`?** It is `continue-on-error`.
4. **Why did the local test flip with index freshness?** `pmat query` searches documents by default. `run_document_query` opens `.pmat/context.db` read-write after `AgentContextIndex::save` has written `manifest.json`, and `build_document_index` writes into it. The next `load` hits `check_manifest_not_older_than_db` and discards the whole index. Reproduced with the `ce945d81e` release binary: after `--rebuild-index`, the db mtime was 0.41 s past the manifest's, and the next query logged `the index is stale: … discarding the index and rebuilding`. With `--no-docs` the manifest is newer and the index survives.
5. **Mechanism.** The only re-derivation sat behind an artifact CI never builds and local readers delete. It is replaced by `tdg-ratchet`, which has CB-200 build its own out-of-tree index and passes only a measured count equal to the baseline.

## Contract

`contracts/cb200-ratchet-ci-v1.yaml`: **6 obligations, 6 evaluated, 0 failed.**

- Five of the six test commands resolve to `scripts/cb200-ratchet-gate.sh <pmat> --control`: 10/10 arms, locally and in CI.
- The sixth is `cargo test --lib make_gate_tests::the_tdg_ratchet_job_reaches_the_required_gate_check`: PASS.
- `pv validate` 0 errors. `pv lint` PASS. `scripts/pv-obligation-gate.py` reports 0 problems over 40 contracts, and `make gate`'s `pv-obligations` leg PASS.

## Plan, routing and dispatch

| phase | what | route | executor |
|---|---|---|---|
| 1 | same-binary measurement matrix | direct | session 1 |
| 2 | slice B `src/services` (17 commits) | subagent: sonnet worker B | session 1 |
| 2 | slice C `src/cli` (19 commits + 3 recovered) | subagent: sonnet worker C | session 1 |
| 2 | slice A (other modules) | agy goal lane (delegate), then worker A as fallback for the rejected lane diff | session 1; the worker's 17-file diff was recovered uncommitted from its clone and committed in session 2 |
| 3 | integrate, bank, CI gate, contract | direct | session 2 |
| 4 | quorum on the final diff | delegate `quorum-review.sh` width 3 | session 2 |

- **Session 2 transcript gate:** `attempted=0 denied=0 stalled=0 running_peak=0 slots=3` before Phase 4.
- **Session 1 transcript:** four Agent dispatches (`PMAT-636/ph2.B`, `ph2.C`, `ph2.delegate`, `ph2.A`).

## Gaps and findings not fixed here

- **CB-2115 red on master and GitHub state.** `ORPHAN-GITHUB #1393` was opened at 15:27Z by a sibling session with no roadmap row. It fails `traceability` on this PR (CI run 35247840393) and `make gate`'s `cb-2113-cb-2115` leg. This PR does not carry that row; the orchestrator's lifecycle PR does. CB-2113 passes (45 commits).
- **The index-discard defect (why 4) is not fixed.** It is in `src/cli/handlers/query_handler/modes_docs.rs` and `function_index/build_persistence.rs`. It makes every default `pmat query` after a rebuild discard and rebuild the index. Not filed here: the brief forbids new issues this PR does not carry.
- **`the_committed_baseline_is_the_measured_count` still passes with no index.** libtest has no not-measured outcome. The enforcement point is now `tdg-ratchet`, where NOT MEASURED exits 1.
- **A slice A pinning test was added and then removed on this branch, so the net diff shows neither.** `test_format_quality_claude_pins_report_text` was added to `src/agent/mcp_server_tests.rs` in `35da3cc6d` and removed in `da13898c8`. It pinned `format_quality_claude`, which this branch never changes: the crashed worker wrote the pin before an extraction it never made. It also sat in the `agent-daemon` module, which no default leg runs. Check with `git log -S test_format_quality_claude_pins_report_text --oneline`.

## Corrections to the brief

1. **"+54 (1742 on 441d198e7)".** It was +53 on `ce945d81e` (1741). The same binary on `683d6994d` gives 1688, and the multiset is identical to the banked binary's, so **none of it is grader drift**.
2. **"1752 in pmat query terms".** That is the `src/` count. It differs from CB-200's scope by −21 excluded dispatcher rows and +11 counted rows outside `src/`.
3. **"a required CI job must build the index".** CB-200 already builds its own out-of-tree index on a checkout without one (#1008). The job does not need `pmat query --rebuild-index`, and a rebuilt in-project index would be discarded by the next default `pmat query` (why 4).
4. **"Locally it flips red/green with index freshness".** The mechanism is the documents-table write after `save()`, not source edits alone.

## Estimates

K̂=35 (`docs/audits/impl-estimates.jsonl:L24-L34`), K=220. Actual `k_measured`: 94 in session 1, and 105 in session 2 at this receipt, before the quorum, CI waits and merge.

## Verdict

PARTIAL(blocker): the gate, baseline and refactors are complete and green on everything the branch owns. The merge waits for a 3/3 quorum on the final head and for master's CB-2115 orphan #1393 to be resolved by a lifecycle PR.
