# Project Context

## Project Structure

- **Language**: kotlin
- **Total Files**: 402
- **Total Functions**: 3074
- **Total Structs**: 832
- **Total Enums**: 225
- **Total Traits**: 23

## Quality Scorecard

- **Overall Health**: 75.0%
- **Complexity Score**: 80.0%
- **Maintainability Index**: 70.0%
- **Technical Debt Hours**: 40.0
- **Test Coverage**: 65.0%
- **Modularity Score**: 85.0%

## Files

### ./artifacts/dogfooding/churn-2025-06-30.json


### ./artifacts/dogfooding/complexity-2025-06-30.json


### ./artifacts/templates/rust-makefile-example.mk


### ./assets/demo/app.js

**File Metrics**: Complexity: 0, Functions: 0


### ./assets/project-state.d.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./assets/project-state.json


### ./fuzz/fuzz_targets/fuzz_dag_builder.rs

**File Metrics**: Complexity: 39, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `FuzzProject`
- **Struct**: `FuzzFile`
- **Enum**: `FuzzAstItem`
- **Enum**: `FuzzVisibility`
- **Function**: `convert_to_project_context` [complexity: 10] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `convert_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `sanitize_path` [complexity: 9] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `convert_ast_item` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `sanitize_name` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `sanitize_path_import` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `convert_visibility` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `assert_dag_invariants` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./fuzz/fuzz_targets/fuzz_github_urls.rs

**File Metrics**: Complexity: 0, Functions: 0

- **Use**: statement
- **Use**: statement
- **Use**: statement

### ./fuzz/fuzz_targets/fuzz_makefile_parser.rs

**File Metrics**: Complexity: 0, Functions: 0

- **Use**: statement
- **Use**: statement

### ./fuzz/fuzz_targets/fuzz_mermaid_escaping.rs

**File Metrics**: Complexity: 32, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `EscapeFuzzInput`
- **Struct**: `FuzzLabel`
- **Function**: `build_escape_test_graph` [complexity: 10] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `build_label_with_special_chars` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `assert_proper_escaping` [complexity: 4] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `assert_valid_mermaid_syntax` [complexity: 13] [cognitive: 25] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]

### ./fuzz/fuzz_targets/fuzz_mermaid_generation.rs

**File Metrics**: Complexity: 43, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `FuzzInput`
- **Struct**: `FuzzNode`
- **Enum**: `FuzzNodeType`
- **Struct**: `FuzzEdge`
- **Enum**: `FuzzEdgeType`
- **Struct**: `FuzzOptions`
- **Struct**: `ConvertedGraph`
- **Function**: `build_dependency_graph` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `sanitize_id` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `convert_node_type` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `convert_edge_type` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `assert_invariants` [complexity: 16] [cognitive: 23] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `has_unescaped_pipes` [complexity: 12] [cognitive: 33] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]

### ./fuzz/fuzz_targets/fuzz_mermaid_performance.rs

**File Metrics**: Complexity: 22, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `PerfFuzzInput`
- **Function**: `build_performance_graph` [complexity: 21] [cognitive: 39] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Function**: `assert_performance_bounds` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./rust-docs/metrics-summary.json


### ./scripts/ast-mermaid-integration.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/check-dead-functions.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/clear-swap-periodic.sh


### ./scripts/complexity-distribution.sh


### ./scripts/comprehensive-dead-code-check.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/config-swap.sh


### ./scripts/config-swap.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/configure-swap.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/create-release.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/create-release.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/dead-code-calibration.sh


### ./scripts/deep-context.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/dogfood-readme-integration.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/dogfood-readme.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/excellence-tracker.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/find-dead-terse-functions.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/find-function-lines.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/find-more-dead-functions.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/fix-build-pedantic.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/generate-fuzz-corpus.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/install.integration.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/install.sh


### ./scripts/install.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/install.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/kubuntu-swap.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/lib/create-release-utils-integration.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/lib/create-release-utils.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/lib/create-release-utils.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/lib/install-utils.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/lib/install-utils.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/mcp-install.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/mermaid-validator.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/mermaid-validator.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/monitor-swap.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/overnight-refactor.sh


### ./scripts/pre-release-analysis.sh


### ./scripts/profile_context.sh


### ./scripts/qa-retest.sh


### ./scripts/qa-verification-status.sh


### ./scripts/reduce-complexity.sh


### ./scripts/remove-dead-code-functions.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/remove-duplicate-inner-attributes.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/remove-specific-functions.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/run-full-qa.sh


### ./scripts/run-fuzzing.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/test-coverage.sh


### ./scripts/test-curl-install.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/test-low-memory.sh


### ./scripts/test-workflow-dag.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/update-rust-docs.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/update-version.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/validate-demo-assets.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/validate-demo-assets.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/validate-docs.py

**File Metrics**: Complexity: 40, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DocumentationValidator`
- **Function**: `__init__` [provability: 75%] [coverage: 65%]
- **Function**: `validate` [provability: 75%] [coverage: 65%]
- **Function**: `_collect_markdown_files` [provability: 75%] [coverage: 65%]
- **Function**: `_check_document_headers` [provability: 75%] [coverage: 65%]
- **Function**: `_check_todo_staleness` [provability: 75%] [coverage: 65%]
- **Function**: `_check_internal_links` [provability: 75%] [coverage: 65%]
- **Function**: `_check_orphaned_documents` [provability: 75%] [coverage: 65%]
- **Function**: `_check_file_naming` [provability: 75%] [coverage: 65%]
- **Function**: `_report_results` [provability: 75%] [coverage: 65%]
- **Function**: `main` [complexity: 0] [cognitive: 0] [big-o: O(?)] [provability: 75%] [coverage: 65%]

### ./scripts/validate-docs.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/validate-github-actions-status.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./scripts/validate-naming.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./server/Dockerfile


### ./server/benches/critical_path.rs

**File Metrics**: Complexity: 24, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `benchmark_cli_parsing` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `benchmark_template_generation` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `benchmark_dag_generation` [complexity: 13] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `benchmark_context_generation` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/benches/performance.rs

**File Metrics**: Complexity: 2, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `bench_core_operations` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/build.rs

**File Metrics**: Complexity: 57, Functions: 27

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `main` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `verify_dependency_versions` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `download_and_compress_assets` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `setup_asset_directories` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `get_asset_definitions` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `process_assets` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `should_skip_asset` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `ensure_asset_downloaded` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `download_asset` [complexity: 8] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `handle_download_failure` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `compress_asset` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_compressed_data` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `write_compressed_file` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `set_asset_hash_env` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `compress_templates` [complexity: 8] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `collect_template_files` [complexity: 8] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `read_template_file` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serde_json_to_string` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_hex_string` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `generate_template_code` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `minify_demo_assets` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `minify_js_file` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `minify_css_file` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `copy_demo_asset` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `simple_js_minify` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `simple_css_minify` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_asset_hash` [complexity: 8] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement

### ./server/deno.json


### ./server/fuzz/fuzz_targets/fuzz_github_urls.rs

**File Metrics**: Complexity: 0, Functions: 0

- **Use**: statement
- **Use**: statement
- **Use**: statement

### ./server/get-docker.sh


### ./server/installer.sh


### ./server/mcp.json


### ./server/src/bin/pmat.rs

**File Metrics**: Complexity: 14, Functions: 3

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `ExecutionMode`
- **Function**: `detect_execution_mode` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `init_tracing` [complexity: 7] [cognitive: 10] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `main` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]

### ./server/src/cli/analysis/defect_prediction.rs

**File Metrics**: Complexity: 0, Functions: 2

- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_defect_prediction` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_defect_prediction_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/analysis/duplicates.rs

**File Metrics**: Complexity: 101, Functions: 35

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DuplicateBlock`
- **Struct**: `DuplicateLocation`
- **Struct**: `DuplicateReport`
- **Struct**: `FileStats`
- **Function**: `handle_analyze_duplicates` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `detect_duplicates` [complexity: 14] [cognitive: 18] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Use**: statement
- **Function**: `extract_blocks` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `extract_exact_blocks` [complexity: 4] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `extract_fuzzy_blocks` [complexity: 6] [cognitive: 12] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Use**: statement
- **Function**: `normalize_block` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `count_tokens` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_block_start` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `is_function_declaration` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_type_declaration` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_block_opening` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `find_block_end` [complexity: 9] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `find_duplicate_blocks` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `should_process_file` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `is_source_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_output` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_json_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_human_output` [complexity: 31] [cognitive: 48] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Function**: `format_sarif_output` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_normalize_block` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_count_tokens` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_function_declaration` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_type_declaration` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_block_opening` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_block_start` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_source_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_should_process_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_block_end` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_exact_blocks` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_duplicate_blocks_no_duplicates` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_duplicate_blocks_with_duplicates` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_stats_calculation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_duplicates_empty_project` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_format_json_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_human_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/analysis/graph_metrics.rs

**File Metrics**: Complexity: 176, Functions: 28

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `NodeMetrics`
- **Struct**: `GraphMetricsResult`
- **Function**: `handle_analyze_graph_metrics` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `build_dependency_graph` [complexity: 10] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Function**: `collect_files` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `collect_files_recursive` [complexity: 15] [cognitive: 29] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `is_source_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_dependencies` [complexity: 14] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Use**: statement
- **Function**: `calculate_metrics` [complexity: 13] [cognitive: 23] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `calculate_betweenness` [complexity: 8] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `is_on_shortest_path` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `calculate_closeness` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `calculate_pagerank` [complexity: 11] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `filter_results` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `export_to_graphml` [complexity: 21] [cognitive: 26] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Use**: statement
- **Function**: `format_output` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `format_gm_as_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_gm_as_human` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `write_gm_human_header` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `write_gm_statistics` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `write_gm_top_nodes` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `write_gm_node_details` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `format_gm_as_csv` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `format_gm_as_markdown` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `write_gm_markdown_header` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `write_gm_markdown_summary` [complexity: 17] [cognitive: 17] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Use**: statement
- **Function**: `write_gm_markdown_top_nodes` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_is_source_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_dependencies` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_graph_metrics_result` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/analysis/mod.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_mod_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/analysis/name_similarity.rs

**File Metrics**: Complexity: 95, Functions: 11

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `NameMatch`
- **Struct**: `NameSimilarityResult`
- **Function**: `handle_analyze_name_similarity` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `collect_names` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `collect_source_files` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `collect_files_recursive` [complexity: 15] [cognitive: 29] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `is_code_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_names` [complexity: 14] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Use**: statement
- **Function**: `find_similar_names` [complexity: 11] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `format_output` [complexity: 47] [cognitive: 66] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_is_code_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_names` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_similar_names` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/analysis/symbol_table.rs

**File Metrics**: Complexity: 91, Functions: 16

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `Symbol`
- **Enum**: `SymbolKind`
- **Enum**: `Visibility`
- **Struct**: `Reference`
- **Enum**: `ReferenceKind`
- **Struct**: `SymbolTable`
- **Function**: `handle_analyze_symbol_table` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `build_symbol_table` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `collect_files` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `collect_files_recursive` [complexity: 15] [cognitive: 29] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `is_source_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_symbols_from_file` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_symbols_simple` [complexity: 7] [cognitive: 13] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `detect_visibility` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `find_unreferenced_symbols` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `find_most_referenced` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `apply_filters` [complexity: 14] [cognitive: 24] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `format_output` [complexity: 42] [cognitive: 73] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_detect_visibility` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_source_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_symbols_simple` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_symbol_table_creation` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./server/src/cli/analysis_helpers.rs

**File Metrics**: Complexity: 19, Functions: 20

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `write_analysis_output` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `merge_ranking_into_json` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `filter_by_severity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `apply_limit` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_write_analysis_output_to_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_write_analysis_output_to_stdout` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_write_analysis_output_invalid_path` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_merge_ranking_into_json_valid` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_merge_ranking_into_json_empty_object` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_merge_ranking_into_json_invalid_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_merge_ranking_into_json_array_input` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `TestItem`
- **Function**: `test_filter_by_severity_removes_low_severity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_filter_by_severity_keeps_all` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_filter_by_severity_removes_all` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_filter_by_severity_empty_input` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_limit_with_limit` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_limit_no_limit` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_limit_larger_than_size` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_limit_zero` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_limit_empty_input` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/args.rs

**File Metrics**: Complexity: 34, Functions: 24

- **Use**: statement
- **Use**: statement
- **Function**: `validate_params` [complexity: 11] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `validate_type` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `value_type_name` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `expand_env_vars` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `parse_key_val` [complexity: 8] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_validate_params_required_missing` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_params_optional_missing` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_params_unknown_parameter` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_params_type_validation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_params_success` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_type_string_accepts_all` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_type_boolean` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_value_type_name` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_expand_env_vars_no_vars` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_expand_env_vars_with_existing_var` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_expand_env_vars_with_missing_var` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_string` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_boolean_true` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_boolean_false` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_integer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_float` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_empty_value` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_no_equals` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_complex_string` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/command_dispatcher.rs

**File Metrics**: Complexity: 46, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `CommandHandler`
- **Trait**: `AnalyzeCommandHandler`
- **Struct**: `CommandDispatcher`
- **Use**: statement
- **Function**: `test_command_dispatcher_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/command_structure.rs

**File Metrics**: Complexity: 32, Functions: 20

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CommandExecutor`
- **Struct**: `CommandRegistry`
- **Struct**: `GenerateCommandGroup`
- **Struct**: `AnalyzeCommandGroup`
- **Struct**: `UtilityCommandGroup`
- **Struct**: `DemoCommandGroup`
- **Struct**: `CommandExecutorFactory`
- **Use**: statement
- **Function**: `test_command_registry_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_command_group_defaults` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/commands.rs

**File Metrics**: Complexity: 0, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `Cli`
- **Enum**: `Mode`
- **Enum**: `Commands`
- **Enum**: `AnalyzeCommands`
- **Enum**: `EnforceCommands`
- **Enum**: `RefactorCommands`
- **Use**: statement
- **Function**: `test_cli_parse_empty` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 6] [provability: 75%] [coverage: 65%]
- **Function**: `test_mode_enum` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 6] [provability: 75%] [coverage: 65%]

### ./server/src/cli/coverage_helpers.rs

**File Metrics**: Complexity: 63, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `setup_coverage_analyzer` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `get_changed_files_for_coverage` [complexity: 11] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `analyze_incremental_coverage` [complexity: 9] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `check_coverage_threshold` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_coverage_summary` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `format_coverage_json` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_coverage_markdown` [complexity: 20] [cognitive: 26] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 47%]
- **Function**: `format_coverage_lcov` [complexity: 13] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]

### ./server/src/cli/defect_helpers.rs

**File Metrics**: Complexity: 80, Functions: 7

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `discover_files_for_defect_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `analyze_defect_probability` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `format_defect_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_defect_summary` [complexity: 20] [cognitive: 22] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 47%]
- **Function**: `format_defect_markdown` [complexity: 50] [cognitive: 85] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `format_defect_sarif` [complexity: 8] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `generate_defect_rules` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/defect_prediction_helpers.rs

**File Metrics**: Complexity: 75, Functions: 14

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DefectPredictionConfig`
- **Struct**: `DefectAnalysisResult`
- **Function**: `discover_source_files_for_defect_analysis` [complexity: 9] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Function**: `calculate_simple_complexity` [complexity: 14] [cognitive: 20] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `calculate_simple_churn_score` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `collect_file_metrics` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `filter_predictions` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `RiskDistribution`
- **Function**: `calculate_risk_distribution` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_summary_output` [complexity: 12] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `generate_recommendations` [complexity: 10] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `format_detailed_output` [complexity: 12] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `format_json_output` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `format_markdown_output` [complexity: 11] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `format_csv_output` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_sarif_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_defect_prediction_helpers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/diagnose.rs

**File Metrics**: Complexity: 108, Functions: 27

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `DiagnosticFormat`
- **Struct**: `DiagnoseArgs`
- **Struct**: `DiagnosticReport`
- **Struct**: `BuildInfo`
- **Struct**: `FeatureResult`
- **Enum**: `FeatureStatus`
- **Struct**: `DiagnosticSummary`
- **Struct**: `CompactErrorContext`
- **Struct**: `SuggestedFix`
- **Struct**: `EnvironmentSnapshot`
- **Trait**: `FeatureTest`
- **Struct**: `RustAstTest`
- **Use**: statement
- **Struct**: `TypeScriptAstTest`
- **Struct**: `PythonAstTest`
- **Struct**: `CacheSubsystemTest`
- **Use**: statement
- **Struct**: `MermaidGeneratorTest`
- **Struct**: `ComplexityAnalysisTest`
- **Struct**: `DeepContextTest`
- **Use**: statement
- **Use**: statement
- **Struct**: `GitIntegrationTest`
- **Struct**: `SelfDiagnostic`
- **Function**: `handle_diagnose` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `print_pretty_report` [complexity: 10] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]

### ./server/src/cli/diagnose_tests.rs

**File Metrics**: Complexity: 9, Functions: 3

- **Use**: statement
- **Function**: `test_diagnostic_format_variants` [complexity: 10] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `test_build_info_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_feature_status_variants` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/enums.rs

**File Metrics**: Complexity: 872, Functions: 47

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `ExecutionMode`
- **Enum**: `OutputFormat`
- **Enum**: `ExplainLevel`
- **Enum**: `EnforceOutputFormat`
- **Enum**: `RefactorOutputFormat`
- **Enum**: `RefactorMode`
- **Enum**: `RefactorAutoOutputFormat`
- **Enum**: `RefactorDocsOutputFormat`
- **Enum**: `QualityProfile`
- **Enum**: `ContextFormat`
- **Enum**: `TdgOutputFormat`
- **Enum**: `MakefileOutputFormat`
- **Enum**: `LintHotspotOutputFormat`
- **Enum**: `ProvabilityOutputFormat`
- **Enum**: `DuplicateType`
- **Enum**: `DefectPredictionOutputFormat`
- **Enum**: `ComprehensiveOutputFormat`
- **Enum**: `GraphMetricType`
- **Enum**: `GraphMetricsOutputFormat`
- **Enum**: `SearchScope`
- **Enum**: `NameSimilarityOutputFormat`
- **Enum**: `DuplicateOutputFormat`
- **Enum**: `ComplexityOutputFormat`
- **Enum**: `DeadCodeOutputFormat`
- **Enum**: `SatdOutputFormat`
- **Enum**: `SatdSeverity`
- **Enum**: `SymbolTableOutputFormat`
- **Enum**: `BigOOutputFormat`
- **Enum**: `SymbolTypeFilter`
- **Enum**: `DagType`
- **Enum**: `DeepContextOutputFormat`
- **Enum**: `DeepContextDagType`
- **Enum**: `DeepContextCacheStrategy`
- **Enum**: `DemoProtocol`
- **Enum**: `ProofAnnotationOutputFormat`
- **Enum**: `PropertyTypeFilter`
- **Enum**: `VerificationMethodFilter`
- **Enum**: `IncrementalCoverageOutputFormat`
- **Enum**: `QualityGateOutputFormat`
- **Enum**: `ReportOutputFormat`
- **Enum**: `AnalysisType`
- **Enum**: `QualityCheckType`
- **Use**: statement
- **Function**: `test_execution_mode_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_output_format_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_satd_severity_ordering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_all_enum_displays` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_enum_equality` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/formatting_helpers.rs

**File Metrics**: Complexity: 49, Functions: 21

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `format_executive_summary` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `format_quality_scorecard` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `format_project_overview` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_build_info` [complexity: 8] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `format_defect_summary` [complexity: 10] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `format_recommendations` [complexity: 7] [cognitive: 11] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_deep_context` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_format_executive_summary` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_scorecard_high_health` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_scorecard_medium_health` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_scorecard_low_health` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_project_overview_complete` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_project_overview_minimal` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_build_info_complete` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_build_info_minimal` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_defect_summary_with_defects` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_defect_summary_no_defects` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_defect_summary_minimal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_recommendations_with_items` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_recommendations_empty` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_recommendations_single_item` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/advanced_analysis_handlers.rs

**File Metrics**: Complexity: 11, Functions: 9

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_deep_context` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `handle_analyze_tdg` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_makefile` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_provability` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_defect_prediction` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_comprehensive` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_graph_metrics` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_symbol_table` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_advanced_analysis_handlers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/analysis/complexity.rs

**File Metrics**: Complexity: 2, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_complexity` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/analysis.rs

**File Metrics**: Complexity: 19, Functions: 2

- **Use**: statement
- **Use**: statement
- **Function**: `analyze_router` [complexity: 21] [cognitive: 21] [big-o: O(n²)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Use**: statement
- **Struct**: `AnalysisHandlers`
- **Function**: `test_analysis_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/analysis_handlers.rs

**File Metrics**: Complexity: 33, Functions: 2

- **Use**: statement
- **Use**: statement
- **Function**: `route_analyze_command` [complexity: 35] [cognitive: 42] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Function**: `test_analysis_handlers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/auto_refactor_engine.rs


### ./server/src/cli/handlers/big_o_handlers.rs

**File Metrics**: Complexity: 39, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_big_o` [complexity: 17] [cognitive: 17] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `format_big_o_summary` [complexity: 14] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `format_big_o_detailed` [complexity: 12] [cognitive: 22] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `test_big_o_handlers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/complexity_handlers.rs

**File Metrics**: Complexity: 217, Functions: 16

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_complexity` [complexity: 15] [cognitive: 15] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Use**: statement
- **Function**: `handle_analyze_churn` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_dead_code` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `run_dead_code_analysis` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `format_dead_code_result` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_dead_code_as_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `format_dead_code_as_sarif` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `format_dead_code_as_summary` [complexity: 28] [cognitive: 30] [big-o: O(?)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 65%]
- **Use**: statement
- **Function**: `format_dead_code_as_markdown` [complexity: 58] [cognitive: 60] [big-o: O(?)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Function**: `write_dead_code_output` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `handle_analyze_satd` [complexity: 19] [cognitive: 23] [big-o: O(n²)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 44%]
- **Use**: statement
- **Function**: `generate_satd_sarif` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `format_satd_summary` [complexity: 19] [cognitive: 22] [big-o: O(n²)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 44%]
- **Use**: statement
- **Function**: `format_satd_markdown` [complexity: 30] [cognitive: 33] [big-o: O(?)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_dag` [complexity: 40] [cognitive: 83] [big-o: O(?)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_complexity_handlers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/demo_handlers.rs

**File Metrics**: Complexity: 7, Functions: 3

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_demo` [complexity: 9] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `handle_quality_gate` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_demo_handlers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/duplication_analysis.rs

**File Metrics**: Complexity: 0, Functions: 5

- **Use**: statement
- **Use**: statement
- **Struct**: `DuplicateAnalysisConfig`
- **Function**: `handle_analyze_duplicates` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_duplicate_analysis_config_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_analysis_config_with_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_analysis_config_defaults` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_analyze_duplicates_delegates` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/enforce_handlers.rs

**File Metrics**: Complexity: 95, Functions: 12

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `EnforcementState`
- **Struct**: `QualityViolation`
- **Struct**: `EnforcementProgress`
- **Struct**: `EnforcementResult`
- **Struct**: `QualityProfile`
- **Function**: `route_enforce_command` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_enforce_extreme` [complexity: 17] [cognitive: 19] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `run_enforcement_step` [complexity: 24] [cognitive: 25] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 56%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `load_quality_profile` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `list_all_violations` [complexity: 38] [cognitive: 50] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `validate_current_state` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `output_result` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `print_progress_bar` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_enforcement_state_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_quality_profile_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_enforcement_result_serialization` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/enhanced_reporting_handlers.rs

**File Metrics**: Complexity: 51, Functions: 9

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_generate_report` [complexity: 15] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `run_analyses` [complexity: 22] [cognitive: 38] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 51%]
- **Function**: `count_files_and_lines` [complexity: 19] [cognitive: 57] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 44%]
- **Use**: statement
- **Use**: statement
- **Function**: `run_complexity_analysis` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `run_dead_code_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `run_duplication_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `run_tdg_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `run_big_o_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_enhanced_reporting_handlers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/generation_handlers.rs

**File Metrics**: Complexity: 32, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_generate` [complexity: 9] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `handle_scaffold` [complexity: 24] [cognitive: 32] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 56%]
- **Use**: statement
- **Function**: `handle_validate` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_generation_handlers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/lint_hotspot_handlers.rs

**File Metrics**: Complexity: 149, Functions: 18

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `LintHotspotParams`
- **Struct**: `LintHotspotResult`
- **Struct**: `LintHotspot`
- **Struct**: `ViolationDetail`
- **Struct**: `FileSummary`
- **Struct**: `SeverityDistribution`
- **Struct**: `EnforcementMetadata`
- **Struct**: `RefactorChain`
- **Struct**: `RefactorStep`
- **Struct**: `QualityGateStatus`
- **Struct**: `QualityViolation`
- **Struct**: `FileMetrics`
- **Struct**: `ClippyMessage`
- **Struct**: `DiagnosticMessage`
- **Struct**: `DiagnosticCode`
- **Struct**: `DiagnosticSpan`
- **Struct**: `DiagnosticText`
- **Function**: `handle_analyze_lint_hotspot` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_lint_hotspot_with_params` [complexity: 18] [cognitive: 18] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Function**: `run_clippy_analysis` [complexity: 30] [cognitive: 53] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `run_clippy_analysis_single_file` [complexity: 20] [cognitive: 58] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 47%]
- **Function**: `count_top_lints` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `count_source_lines` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `process_diagnostic` [complexity: 9] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `find_hotspot` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `find_hotspot_with_details` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `calculate_enforcement_metadata` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_refactor_chain` [complexity: 11] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `check_quality_gates` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_output` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `format_summary` [complexity: 22] [cognitive: 23] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 51%]
- **Function**: `format_detailed` [complexity: 14] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `format_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `SimpleResult`
- **Function**: `format_sarif` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `find_workspace_root` [complexity: 8] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]

### ./server/src/cli/handlers/mod.rs

**File Metrics**: Complexity: 0, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_handler_exports` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_module_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/name_similarity_analysis.rs

**File Metrics**: Complexity: 16, Functions: 6

- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_name_similarity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_handle_analyze_name_similarity_basic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_analyze_name_similarity_with_options` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_name_similarity_parameters` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_scope_variations` [complexity: 8] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `test_output_format_variations` [complexity: 10] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement

### ./server/src/cli/handlers/refactor_auto_handlers.rs

**File Metrics**: Complexity: 731, Functions: 89

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `QualityProfile`
- **Struct**: `LintHotspotJsonResponse`
- **Struct**: `LintHotspotJson`
- **Struct**: `ViolationDetailJson`
- **Struct**: `RefactorState`
- **Struct**: `QualityMetrics`
- **Struct**: `RefactorProgress`
- **Enum**: `RefactorPhase`
- **Function**: `handle_single_file_refactor` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Struct**: `FileRewritePlan`
- **Struct**: `ViolationWithContext`
- **Struct**: `AstMetadata`
- **Struct**: `FunctionInfo`
- **Enum**: `FixStrategy`
- **Function**: `handle_refactor_auto` [complexity: 97] [cognitive: 211] [big-o: O(?)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `generate_context` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `run_lint_hotspot_analysis` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `check_file_coverage` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `is_tarpaulin_installed` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `determine_coverage_directory` [complexity: 7] [cognitive: 10] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `get_relative_file_path` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `run_tarpaulin_for_file` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `parse_tarpaulin_output` [complexity: 9] [cognitive: 13] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `extract_percentage_from_line` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `check_coverage` [complexity: 18] [cognitive: 29] [big-o: O(n²)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Use**: statement
- **Function**: `meets_quality_gates` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_file_severity_score` [complexity: 14] [cognitive: 35] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `get_cached_file_coverage` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `get_single_file_lint_violations` [complexity: 9] [cognitive: 12] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `analyze_file_complexity` [complexity: 10] [cognitive: 24] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `count_file_satd` [complexity: 6] [cognitive: 7] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `generate_single_file_refactor_request` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `extract_context_lines` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `print_single_file_summary` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `print_single_file_detailed` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `should_process_file` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `select_target_file` [complexity: 12] [cognitive: 19] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `create_rewrite_plan` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `apply_rewrite_plan` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `verify_build` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `rollback_file` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `load_or_init_state` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `save_state` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `should_refresh_context` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `output_progress` [complexity: 9] [cognitive: 10] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Struct**: `LintHotspotResult`
- **Struct**: `LintHotspot`
- **Struct**: `FileSummary`
- **Struct**: `Violation`
- **Struct**: `ViolationDetail`
- **Function**: `load_ast_metadata_from_context` [complexity: 18] [cognitive: 45] [big-o: O(n²)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Function**: `map_violations_to_ast` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `generate_refactored_content` [complexity: 43] [cognitive: 167] [big-o: O(?)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `output_ai_test_generation_request` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `output_ai_rewrite_request` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `find_or_create_test_file` [complexity: 12] [cognitive: 13] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `get_crate_name` [complexity: 7] [cognitive: 9] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `create_test_generation_plan` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `find_untested_functions` [complexity: 15] [cognitive: 53] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `parse_new_function_line` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `parse_function_line` [complexity: 4] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `generate_test_content` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `find_file_with_lowest_coverage` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `is_non_refactorable_file` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `select_coverage_or_extreme_target` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `find_file_with_low_coverage` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `select_extreme_quality_target` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `select_fallback_target` [complexity: 6] [cognitive: 9] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `find_rust_files` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `visit_dirs` [complexity: 8] [cognitive: 15] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `create_unified_rewrite_plan` [complexity: 14] [cognitive: 20] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `detect_satd_in_file` [complexity: 12] [cognitive: 23] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `generate_test_file_content` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `determine_fix_strategy` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `find_test_file_for` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `output_ai_unified_rewrite_request` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `calculate_refactor_progress` [complexity: 21] [cognitive: 27] [big-o: O(n²)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Function**: `display_refactor_progress` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `find_test_dependencies` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `extract_import_dependencies` [complexity: 4] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `resolve_import_path` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `extract_file_ref_dependencies` [complexity: 6] [cognitive: 10] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `find_workspace_root` [complexity: 9] [cognitive: 14] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `analyze_compilation_errors` [complexity: 15] [cognitive: 31] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Use**: statement
- **Struct**: `CompilationError`
- **Function**: `find_file_with_high_complexity` [complexity: 25] [cognitive: 135] [big-o: O(n²)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 58%]
- **Function**: `find_file_with_satd` [complexity: 13] [cognitive: 28] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `check_complexity_and_satd` [complexity: 14] [cognitive: 24] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Use**: statement
- **Function**: `extract_file_context` [complexity: 7] [cognitive: 13] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `apply_automated_refactoring` [complexity: 35] [cognitive: 55] [big-o: O(?)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `generate_automated_tests` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `extract_refactored_function` [complexity: 11] [cognitive: 17] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `replace_function_in_file` [complexity: 9] [cognitive: 15] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `extract_public_functions` [complexity: 5] [cognitive: 7] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `extract_function_name` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `generate_test_stubs` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `append_tests_to_module` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `create_ai_rewrite_request` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `check_file_coverage_llvm` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `generate_llvm_coverage_report` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `generate_grcov_coverage_report` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `parse_llvm_cov_output` [complexity: 11] [cognitive: 23] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `parse_grcov_output` [complexity: 6] [cognitive: 12] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `find_test_binary` [complexity: 10] [cognitive: 16] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Function**: `rewrite_file_with_coverage_guarantee` [complexity: 13] [cognitive: 14] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `parse_ai_response` [complexity: 17] [cognitive: 37] [big-o: O(n²)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `determine_test_file_path` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 19%]

### ./server/src/cli/handlers/refactor_docs_handlers.rs

**File Metrics**: Complexity: 144, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `FileCategory`
- **Struct**: `CruftFile`
- **Struct**: `CleanupSummary`
- **Struct**: `RefactorDocsResult`
- **Function**: `handle_refactor_docs` [complexity: 25] [cognitive: 25] [big-o: O(n²)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 58%]
- **Function**: `scan_for_cruft` [complexity: 22] [cognitive: 45] [big-o: O(n²)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 51%]
- **Function**: `collect_files_recursive` [complexity: 9] [cognitive: 13] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `collect_files_flat` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `should_preserve` [complexity: 4] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `matches_pattern` [complexity: 4] [cognitive: 5] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `handle_interactive_mode` [complexity: 11] [cognitive: 15] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `create_backup` [complexity: 9] [cognitive: 10] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `remove_files` [complexity: 9] [cognitive: 12] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `format_output` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_summary` [complexity: 15] [cognitive: 17] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `format_detailed` [complexity: 15] [cognitive: 23] [big-o: O(n log n)] [SATD: 2] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `format_json` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_file_category_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `test_should_preserve` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]
- **Function**: `test_matches_pattern` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 2] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/refactor_handlers.rs

**File Metrics**: Complexity: 88, Functions: 11

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `RefactorServeParams`
- **Function**: `route_refactor_command` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `handle_refactor_serve` [complexity: 25] [cognitive: 30] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 58%]
- **Function**: `handle_refactor_interactive` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `handle_refactor_status` [complexity: 19] [cognitive: 21] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 44%]
- **Function**: `handle_refactor_resume` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `load_refactor_config` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `load_refactor_config_json` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `sort_targets_by_priority` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_auto_commit` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `discover_refactor_targets` [complexity: 7] [cognitive: 10] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]

### ./server/src/cli/handlers/utility_handlers.rs

**File Metrics**: Complexity: 169, Functions: 32

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_list` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Struct**: `MarkdownBuilder`
- **Function**: `handle_search` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `handle_context` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `detect_or_use_toolchain` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `analyze_project` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement
- **Function**: `build_project_context` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `process_file_context` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `update_project_summary` [complexity: 10] [cognitive: 23] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `format_context_output` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `write_context_output` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `format_json_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `format_markdown_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `add_project_sections` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `add_project_structure` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `add_quality_scorecard` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `add_files_section` [complexity: 4] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `add_file_items` [complexity: 17] [cognitive: 32] [big-o: O(n²)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `add_recommendations` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `format_function_annotations` [complexity: 30] [cognitive: 58] [big-o: O(?)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `format_sarif_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `format_llm_optimized_output` [complexity: 32] [cognitive: 81] [big-o: O(?)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `detect_primary_language` [complexity: 36] [cognitive: 39] [big-o: O(?)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Use**: statement
- **Function**: `handle_serve` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_diagnose` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_utility_handlers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/src/cli/handlers/wasm_handlers.rs

**File Metrics**: Complexity: 80, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_assemblyscript` [complexity: 17] [cognitive: 40] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `handle_analyze_webassembly` [complexity: 22] [cognitive: 53] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 51%]
- **Function**: `collect_assemblyscript_files` [complexity: 13] [cognitive: 41] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `collect_wasm_files` [complexity: 7] [cognitive: 14] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `format_assemblyscript_results` [complexity: 14] [cognitive: 23] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `format_webassembly_results` [complexity: 15] [cognitive: 26] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_collect_assemblyscript_files` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_collect_wasm_files` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/mod.rs

**File Metrics**: Complexity: 49, Functions: 15

- **Use**: statement
- **Use**: statement
- **Struct**: `NameInfo`
- **Struct**: `NameSimilarityResult`
- **Struct**: `DuplicateHandlerConfig`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `EarlyCliArgs`
- **Function**: `parse_early_for_tracing` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `run` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `detect_primary_language` [complexity: 18] [cognitive: 37] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Use**: statement
- **Function**: `apply_satd_filters` [complexity: 14] [cognitive: 32] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Use**: statement
- **Struct**: `DeepContextConfigParams`
- **Function**: `build_deep_context_config` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `convert_dag_type` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `convert_cache_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `parse_analysis_filters` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `parse_analysis_type` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `handle_analyze_defect_prediction` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_duplicates` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_graph_metrics` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_name_similarity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_symbol_table` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_comprehensive` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/name_similarity_helpers.rs

**File Metrics**: Complexity: 71, Functions: 32

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `JsonResultsConfig`
- **Struct**: `OutputConfig`
- **Function**: `discover_source_files` [complexity: 7] [cognitive: 8] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `extract_all_identifiers` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_similarities` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `calculate_combined_similarity` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `build_results_json` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `output_results` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `format_summary_output` [complexity: 10] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `format_detailed_output` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `format_csv_output` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_markdown_output` [complexity: 7] [cognitive: 8] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_name_info` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_similarity_result` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_discover_source_files_empty_directory` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_discover_source_files_with_files` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_discover_source_files_with_include_filter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_all_identifiers` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_all_identifiers_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_similarities_exact_match` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_similarities_threshold_filter` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_similarities_case_insensitive` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_similarities_sorted_by_score` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_combined_similarity_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_combined_similarity_with_fuzzy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_build_results_json_basic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_build_results_json_with_performance` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_output_results_json_format` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_output_results_summary_format` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_summary_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_detailed_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_csv_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_markdown_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_markdown_output_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/proof_annotation_formatter.rs

**File Metrics**: Complexity: 71, Functions: 7

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `format_confidence_stats` [complexity: 9] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `format_method_stats` [complexity: 14] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Use**: statement
- **Function**: `format_property_stats` [complexity: 9] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `group_by_file` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_single_proof` [complexity: 27] [cognitive: 31] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 63%]
- **Function**: `format_provability_summary` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `generate_proof_sarif_rules` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/proof_annotation_helpers.rs

**File Metrics**: Complexity: 147, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ProofAnnotationFilter`
- **Function**: `filter_annotation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `filter_by_confidence` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `filter_by_property_type` [complexity: 15] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `filter_by_verification_method` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `format_as_json` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `setup_proof_annotator` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `collect_and_filter_annotations` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_as_table` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `format_as_summary` [complexity: 18] [cognitive: 20] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Use**: statement
- **Function**: `format_as_full` [complexity: 40] [cognitive: 80] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Function**: `format_as_markdown` [complexity: 41] [cognitive: 66] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Function**: `format_as_sarif` [complexity: 12] [cognitive: 18] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `test_proof_annotation_helpers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/provability_helpers.rs

**File Metrics**: Complexity: 64, Functions: 9

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `parse_function_spec` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `discover_project_functions` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `extract_function_name` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `filter_summaries` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_provability_json` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `format_provability_summary` [complexity: 17] [cognitive: 17] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `format_provability_detailed` [complexity: 29] [cognitive: 62] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 68%]
- **Function**: `format_provability_sarif` [complexity: 8] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `generate_provability_rules` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/regex_patterns.rs

**File Metrics**: Complexity: 0, Functions: 0

- **Use**: statement
- **Use**: statement

### ./server/src/cli/stubs.rs

**File Metrics**: Complexity: 705, Functions: 124

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_tdg` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_makefile` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `print_makefile_analysis_summary` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `filter_makefile_violations` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `handle_makefile_fix_mode` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `format_makefile_output` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_makefile_as_json` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `format_makefile_as_human` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `write_makefile_human_header` [complexity: 12] [cognitive: 12] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Use**: statement
- **Function**: `write_makefile_violations_table` [complexity: 13] [cognitive: 15] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `get_severity_display` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `write_makefile_fix_suggestions` [complexity: 7] [cognitive: 9] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `format_makefile_as_sarif` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `build_sarif_rules` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `build_sarif_results` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `get_sarif_level` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_makefile_as_gcc` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `get_gcc_level` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `handle_analyze_provability` [complexity: 19] [cognitive: 20] [big-o: O(n²)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 44%]
- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_defect_prediction` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_proof_annotations` [complexity: 15] [cognitive: 15] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Use**: statement
- **Use**: statement
- **Function**: `handle_analyze_incremental_coverage` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_churn` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Use**: statement
- **Use**: statement
- **Function**: `format_churn_as_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `format_churn_as_summary` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `write_summary_header` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Function**: `write_summary_hotspot_files` [complexity: 7] [cognitive: 9] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `write_summary_stable_files` [complexity: 7] [cognitive: 9] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `write_summary_top_contributors` [complexity: 7] [cognitive: 9] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `format_churn_as_markdown` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `write_markdown_header` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Function**: `write_markdown_summary_table` [complexity: 17] [cognitive: 17] [big-o: O(n²)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Use**: statement
- **Function**: `write_markdown_file_details` [complexity: 11] [cognitive: 13] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `write_markdown_author_contributions` [complexity: 11] [cognitive: 13] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `write_markdown_recommendations` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `format_churn_as_csv` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `write_churn_output` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_satd` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_dag` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `handle_quality_gate` [complexity: 40] [cognitive: 68] [big-o: O(?)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `handle_serve` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `handle_analyze_comprehensive` [complexity: 16] [cognitive: 16] [big-o: O(n²)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Use**: statement
- **Struct**: `QualityGateResults`
- **Struct**: `ComprehensiveReport`
- **Struct**: `ComplexityReport`
- **Struct**: `ComplexityHotspot`
- **Struct**: `SatdReport`
- **Struct**: `SatdItem`
- **Struct**: `TdgReport`
- **Struct**: `TdgFile`
- **Struct**: `DeadCodeReport`
- **Struct**: `DeadCodeItem`
- **Struct**: `DefectReport`
- **Struct**: `DefectPrediction`
- **Struct**: `DuplicateReport`
- **Struct**: `DuplicateBlock`
- **Struct**: `QualityViolation`
- **Function**: `is_source_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `check_complexity` [complexity: 9] [cognitive: 16] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Function**: `check_dead_code` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `check_satd` [complexity: 10] [cognitive: 21] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Use**: statement
- **Function**: `check_entropy` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `check_security` [complexity: 13] [cognitive: 37] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Use**: statement
- **Function**: `check_duplicates` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `check_coverage` [complexity: 5] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `check_sections` [complexity: 9] [cognitive: 18] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `check_provability` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_provability_score` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `format_quality_gate_output` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `format_qg_as_json` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `format_qg_as_human` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `write_qg_human_header` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `write_qg_violation_counts` [complexity: 5] [cognitive: 7] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `write_qg_violations_list` [complexity: 11] [cognitive: 15] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `format_qg_as_junit` [complexity: 18] [cognitive: 18] [big-o: O(n²)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Use**: statement
- **Function**: `format_qg_as_summary` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `format_qg_as_detailed` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `write_qg_detailed_header` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `write_qg_detailed_summary` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `write_qg_detailed_violations` [complexity: 11] [cognitive: 15] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `format_qg_as_markdown` [complexity: 16] [cognitive: 16] [big-o: O(n²)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Use**: statement
- **Function**: `detect_toolchain` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `build_complexity_thresholds` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_project_files` [complexity: 23] [cognitive: 29] [big-o: O(n²)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 54%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_file_complexity` [complexity: 33] [cognitive: 102] [big-o: O(?)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Function**: `extract_rust_function_name` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `extract_js_function_name` [complexity: 7] [cognitive: 9] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `extract_python_function_name` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `estimate_function_complexity` [complexity: 9] [cognitive: 12] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `add_top_files_ranking` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `format_dead_code_output` [complexity: 29] [cognitive: 83] [big-o: O(?)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 68%]
- **Use**: statement
- **Function**: `extract_identifiers` [complexity: 8] [cognitive: 18] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `calculate_string_similarity` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `get_ngrams` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_edit_distance` [complexity: 9] [cognitive: 10] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `calculate_soundex` [complexity: 9] [cognitive: 12] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `soundex_code` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `params_to_json` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `print_table` [complexity: 8] [cognitive: 9] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `estimate_cyclomatic_complexity` [complexity: 17] [cognitive: 28] [big-o: O(n²)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `estimate_cognitive_complexity` [complexity: 14] [cognitive: 21] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `run_complexity_analysis` [complexity: 9] [cognitive: 13] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Function**: `run_satd_analysis` [complexity: 13] [cognitive: 39] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Use**: statement
- **Function**: `run_tdg_analysis` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `run_dead_code_analysis` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `run_defect_prediction` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `run_duplicate_detection` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `format_comprehensive_report` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `format_comp_as_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `format_comp_as_markdown` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `write_comp_executive_summary` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `write_comp_analysis_sections` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `write_comp_complexity_section` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `write_comp_satd_section` [complexity: 12] [cognitive: 12] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Use**: statement
- **Function**: `write_comp_tdg_section` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Function**: `write_comp_dead_code_section` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `write_comp_defects_section` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `write_comp_duplicates_section` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_handle_analyze_makefile_basic` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_handle_analyze_makefile_with_rules` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_handle_analyze_provability` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_handle_analyze_defect_prediction` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_handle_analyze_proof_annotations` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_analyze_incremental_coverage` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_extract_identifiers` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_string_similarity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_edit_distance` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_soundex` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_serve_placeholder` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_output_format_completeness` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_estimate_complexity_functions` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_ngrams` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_soundex_code` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_gate_output_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_gate_output_human` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_gate_output_junit` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_gate_output_summary` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_gate_output_detailed` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_quality_gate_output_all_violation_types` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]

### ./server/src/cli/symbol_table_helpers.rs

**File Metrics**: Complexity: 42, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SymbolInfo`
- **Function**: `extract_symbol_from_ast_item` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `passes_type_filter` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `passes_query_filter` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_symbols_from_context` [complexity: 6] [cognitive: 11] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `count_by_type` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `count_by_visibility` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_symbol_table_summary` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `format_symbol_table_detailed` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_symbol_table_csv` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_symbol_table_helpers_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/cli/tdg_helpers.rs

**File Metrics**: Complexity: 64, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `filter_tdg_hotspots` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `format_tdg_json` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `format_tdg_table` [complexity: 11] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `format_tdg_markdown` [complexity: 40] [cognitive: 69] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Function**: `format_tdg_sarif` [complexity: 8] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `generate_tdg_rules` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/adapters/cli.rs

**File Metrics**: Complexity: 47, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CliDemoAdapter`
- **Struct**: `CliRequest`
- **Struct**: `CliResponse`
- **Struct**: `CliApiTrace`
- **Enum**: `CliDemoError`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_cli_adapter_metadata` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_request_from_value` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_key_generation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_api_trace_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/adapters/http.rs

**File Metrics**: Complexity: 69, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `HttpDemoAdapter`
- **Struct**: `HttpRequest`
- **Struct**: `HttpResponse`
- **Enum**: `HttpResponseBody`
- **Struct**: `HttpRequestInfo`
- **Struct**: `HttpEndpoint`
- **Struct**: `HttpParameter`
- **Enum**: `HttpDemoError`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_http_adapter_metadata` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_request_from_value` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_api_introspection` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_context_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement

### ./server/src/demo/adapters/mcp.rs

**File Metrics**: Complexity: 124, Functions: 18

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `McpDemoAdapter`
- **Struct**: `McpRequest`
- **Struct**: `McpResponse`
- **Struct**: `McpError`
- **Struct**: `DemoAnalyzeParams`
- **Struct**: `DemoAnalyzeResult`
- **Struct**: `DemoGetResultsParams`
- **Struct**: `DemoGetApiTraceParams`
- **Struct**: `McpTrace`
- **Enum**: `McpDemoError`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_mcp_adapter_metadata` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_request_from_value` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_demo_analyze` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_unknown_method` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_conversion` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/adapters/mod.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_mod_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/adapters/tui.rs

**File Metrics**: Complexity: 168, Functions: 28

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `ControlFlow`
- **Enum**: `PanelId`
- **Struct**: `TuiState`
- **Struct**: `AnalysisResults`
- **Struct**: `Hotspot`
- **Enum**: `Severity`
- **Struct**: `FileInfo`
- **Struct**: `NodeInfo`
- **Struct**: `AnalysisUpdate`
- **Enum**: `UpdateType`
- **Struct**: `FileComplexity`
- **Struct**: `FileChurn`
- **Struct**: `TuiDemoAdapter`
- **Struct**: `TuiRequest`
- **Struct**: `TuiResponse`
- **Enum**: `TuiDemoError`
- **Struct**: `TuiDemoAdapter`
- **Struct**: `TuiRequest`
- **Struct**: `TuiResponse`
- **Function**: `test_tui_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/assets.rs

**File Metrics**: Complexity: 11, Functions: 5

- **Use**: statement
- **Use**: statement
- **Struct**: `EmbeddedAsset`
- **Enum**: `AssetEncoding`
- **Function**: `get_asset` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `get_asset` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `decompress_asset` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `get_asset_hash` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_assets_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/config.rs

**File Metrics**: Complexity: 31, Functions: 18

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DisplayConfig`
- **Struct**: `PanelConfig`
- **Struct**: `DependencyPanelConfig`
- **Enum**: `GroupingStrategy`
- **Struct**: `ComplexityPanelConfig`
- **Struct**: `ChurnPanelConfig`
- **Struct**: `ContextPanelConfig`
- **Struct**: `ExportConfig`
- **Struct**: `PerformanceConfig`
- **Struct**: `ConfigManager`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_default_config` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_load_from_yaml` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_config_manager` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/export.rs

**File Metrics**: Complexity: 56, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ExportReport`
- **Struct**: `ComplexityAnalysis`
- **Struct**: `ChurnAnalysis`
- **Struct**: `ChurnFile`
- **Struct**: `ProjectSummary`
- **Trait**: `Exporter`
- **Struct**: `MarkdownExporter`
- **Struct**: `JsonExporter`
- **Struct**: `SarifExporter`
- **Struct**: `ExportService`
- **Function**: `create_export_report` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_full_export_report` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `test_markdown_export` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_export` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_export_service` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/mod.rs

**File Metrics**: Complexity: 117, Functions: 24

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `run_demo` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `load_demo_config` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_analyzer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `run_analyses` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `generate_output` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `handle_protocol_output` [complexity: 12] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `build_protocol_request` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `format_and_print_output` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `print_api_metadata` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `run_all_protocols` [complexity: 7] [cognitive: 10] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `run_single_protocol` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `protocol_to_string` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `print_protocol_banner` [complexity: 12] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Struct**: `DemoConfig`
- **Struct**: `DemoAnalyzer`
- **Enum**: `AnalysisResults`
- **Struct**: `ProtocolTrace`
- **Enum**: `DemoOutput`
- **Function**: `extract_analysis_from_demo_report` [complexity: 14] [cognitive: 32] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `parse_dag_data` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `run_web_demo` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Function**: `analyze_context` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_complexity` [complexity: 6] [cognitive: 8] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_dag` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `analyze_churn` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_system_architecture` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_defect_probability` [complexity: 8] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement
- **Struct**: `DemoArgs`
- **Enum**: `Protocol`
- **Function**: `run_tui_demo` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Function**: `test_mod_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/protocol_harness.rs

**File Metrics**: Complexity: 35, Functions: 32

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `DemoProtocol`
- **Struct**: `ProtocolMetadata`
- **Struct**: `DemoEngine`
- **Struct**: `DemoConfig`
- **Struct**: `ContextCache`
- **Struct**: `CacheEntry`
- **Struct**: `AnalysisResult`
- **Enum**: `AnalysisStatus`
- **Struct**: `TraceStore`
- **Struct**: `ApiTrace`
- **Struct**: `TimingInfo`
- **Enum**: `DemoError`
- **Struct**: `BoxedError`
- **Struct**: `ProtocolWrapper`
- **Use**: statement
- **Function**: `test_demo_engine_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_cache` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_trace_store` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/src/demo/router.rs

**File Metrics**: Complexity: 8, Functions: 7

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `Router`
- **Function**: `build_router` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_request` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `handle_request` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_router_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/runner.rs

**File Metrics**: Complexity: 298, Functions: 26

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DemoRunner`
- **Struct**: `DemoStep`
- **Struct**: `DemoReport`
- **Struct**: `Component`
- **Function**: `resolve_repository` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `resolve_repo_spec` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `get_canonical_path` [complexity: 8] [cognitive: 9] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `find_git_root` [complexity: 7] [cognitive: 8] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `is_interactive_environment` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `read_repository_path_from_user` [complexity: 12] [cognitive: 12] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `detect_repository` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_runner_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/src/demo/server.rs

**File Metrics**: Complexity: 66, Functions: 37

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DemoContent`
- **Struct**: `Hotspot`
- **Struct**: `DemoState`
- **Struct**: `AnalysisResults`
- **Struct**: `LocalDemoServer`
- **Function**: `handle_connection` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Struct**: `MinimalRequest`
- **Function**: `parse_minimal_request` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `serialize_response` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_dashboard` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_static_asset` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_static_asset` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_architecture_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_defect_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_statistics_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_system_diagram` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_analysis_stream` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_graph_density` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_avg_degree` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_analysis_data` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_summary_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_metrics_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `HotspotEntry`
- **Function**: `serve_hotspots_table` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_dag_mermaid` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `serve_system_diagram_mermaid` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_architecture_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `serve_defect_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_statistics_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_system_diagram` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_analysis_stream` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `serve_analysis_data` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_graph_density` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_avg_degree` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `spawn_sync` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/server_tests.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Function**: `test_demo_server_basics` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/templates.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Function**: `test_templates_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/demo/tests.rs

**File Metrics**: Complexity: 14, Functions: 12

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_demo_config_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_demo_config_with_path` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_variants` [complexity: 15] [cognitive: 25] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `test_demo_result_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_demo_metrics_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_demo_runner_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `health_check` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `not_found` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_health_check_handler` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_not_found_handler` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `create_router` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_create_router` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/handlers/initialize.rs

**File Metrics**: Complexity: 2, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_initialize` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_tools_list` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/handlers/mod.rs

**File Metrics**: Complexity: 10, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_request` [complexity: 12] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]

### ./server/src/handlers/prompts.rs

**File Metrics**: Complexity: 19, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_prompts_list` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_prompt_get` [complexity: 19] [cognitive: 19] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 44%]

### ./server/src/handlers/resources.rs

**File Metrics**: Complexity: 18, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_resource_list` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `handle_resource_read` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]

### ./server/src/handlers/tools.rs

**File Metrics**: Complexity: 454, Functions: 64

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_tool_call` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `parse_tool_call_params` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `dispatch_tool_call` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `is_template_tool` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_analysis_tool` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_template_tools` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `handle_analysis_tools` [complexity: 16] [cognitive: 16] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `handle_generate_template` [complexity: 11] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `handle_list_templates` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `handle_validate_template` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `parse_validate_template_args` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `ValidationResult`
- **Function**: `validate_template_parameters` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `find_missing_required_parameters` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `validate_parameter_values` [complexity: 5] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `validate_single_parameter` [complexity: 6] [cognitive: 12] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `create_validation_response` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_scaffold_project` [complexity: 21] [cognitive: 32] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Function**: `handle_search_templates` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `handle_get_server_info` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `AnalyzeCodeChurnArgs`
- **Function**: `handle_analyze_code_churn` [complexity: 15] [cognitive: 19] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `format_churn_summary` [complexity: 10] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `format_churn_as_markdown` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `format_churn_as_csv` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_relevance` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Struct**: `AnalyzeComplexityArgs`
- **Function**: `handle_analyze_complexity` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Use**: statement
- **Function**: `resolve_project_path_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `detect_toolchain` [complexity: 7] [cognitive: 15] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `build_complexity_thresholds` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `analyze_project_files` [complexity: 10] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Function**: `should_analyze_file` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `matches_include_filters` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `matches_pattern` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_file_complexity` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `format_complexity_output` [complexity: 9] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Function**: `format_complexity_rankings` [complexity: 8] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Struct**: `AnalyzeDagArgs`
- **Function**: `handle_analyze_dag` [complexity: 23] [cognitive: 23] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 54%]
- **Use**: statement
- **Use**: statement
- **Struct**: `GenerateContextArgs`
- **Function**: `handle_generate_context` [complexity: 17] [cognitive: 17] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `AnalyzeSystemArchitectureArgs`
- **Function**: `convert_node_type` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `convert_edge_type` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `build_call_graph` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `build_complexity_map` [complexity: 5] [cognitive: 8] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Use**: statement
- **Function**: `format_architecture_result` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `handle_analyze_system_architecture` [complexity: 18] [cognitive: 18] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Use**: statement
- **Use**: statement
- **Struct**: `AnalyzeDefectProbabilityArgs`
- **Function**: `handle_analyze_defect_probability` [complexity: 18] [cognitive: 19] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Use**: statement
- **Use**: statement
- **Struct**: `AnalyzeDeadCodeArgs`
- **Function**: `handle_analyze_dead_code` [complexity: 16] [cognitive: 16] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Use**: statement
- **Use**: statement
- **Function**: `format_dead_code_output` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `format_dead_code_summary_mcp` [complexity: 20] [cognitive: 34] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 47%]
- **Function**: `format_dead_code_as_sarif_mcp` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `format_dead_code_as_markdown_mcp` [complexity: 16] [cognitive: 24] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Struct**: `AnalyzeTdgArgs`
- **Function**: `handle_analyze_tdg` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Use**: statement
- **Function**: `format_tdg_summary` [complexity: 14] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Struct**: `AnalyzeDeepContextArgs`
- **Function**: `handle_analyze_deep_context` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `parse_deep_context_args` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `resolve_project_path` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `parse_analysis_types` [complexity: 12] [cognitive: 20] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Use**: statement
- **Function**: `parse_dag_type` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `parse_cache_strategy` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `build_deep_context_config` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `create_deep_context_analyzer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_deep_context_response` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_deep_context_as_sarif` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `format_deep_context_as_markdown` [complexity: 18] [cognitive: 22] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Use**: statement
- **Function**: `handle_analyze_makefile_lint` [complexity: 14] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Struct**: `MakefileLintArgs`
- **Use**: statement
- **Function**: `handle_analyze_provability` [complexity: 14] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Struct**: `ProvabilityArgs`
- **Use**: statement

### ./server/src/handlers/vectorized_tools.rs

**File Metrics**: Complexity: 67, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `is_vectorized_tool` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_vectorized_tools` [complexity: 12] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Struct**: `DuplicatesVectorizedArgs`
- **Function**: `handle_duplicates_vectorized` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Struct**: `GraphMetricsVectorizedArgs`
- **Function**: `handle_graph_metrics_vectorized` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Struct**: `NameSimilarityVectorizedArgs`
- **Function**: `handle_name_similarity_vectorized` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Struct**: `SymbolTableVectorizedArgs`
- **Function**: `handle_symbol_table_vectorized` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Struct**: `IncrementalCoverageVectorizedArgs`
- **Function**: `handle_incremental_coverage_vectorized` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Struct**: `BigOVectorizedArgs`
- **Function**: `handle_big_o_vectorized` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Struct**: `EnhancedReportArgs`
- **Function**: `handle_enhanced_report` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `get_vectorized_tools_info` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/lib.rs

**File Metrics**: Complexity: 33, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `S3Client`
- **Trait**: `TemplateServerTrait`
- **Struct**: `TemplateServer`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `run_mcp_server` [complexity: 17] [cognitive: 28] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Use**: statement
- **Use**: statement
- **Use**: statement

### ./server/src/models/churn.rs

**File Metrics**: Complexity: 26, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CodeChurnAnalysis`
- **Struct**: `FileChurnMetrics`
- **Struct**: `ChurnSummary`
- **Enum**: `ChurnOutputFormat`
- **Use**: statement
- **Use**: statement
- **Function**: `test_file_churn_metrics_calculate_score` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_churn_metrics_zero_max` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_churn_output_format_from_str` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_code_churn_analysis_creation` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_churn_summary_with_data` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_serialization` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/complexity_bound.rs

**File Metrics**: Complexity: 113, Functions: 35

- **Use**: statement
- **Use**: statement
- **Enum**: `BigOClass`
- **Enum**: `InputVariable`
- **Struct**: `ComplexityFlags`
- **Struct**: `ComplexityBound`
- **Struct**: `CacheComplexity`
- **Struct**: `RecurrenceRelation`
- **Struct**: `RecursiveCall`
- **Enum**: `ComplexityProofType`
- **Use**: statement
- **Function**: `test_complexity_bound_size` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_complexity_size` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_big_o_ordering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_bound_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_growth_estimation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_master_theorem` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/dag.rs

**File Metrics**: Complexity: 8, Functions: 6

- **Use**: statement
- **Use**: statement
- **Struct**: `DependencyGraph`
- **Struct**: `NodeInfo`
- **Struct**: `Edge`
- **Enum**: `NodeType`
- **Enum**: `EdgeType`
- **Function**: `test_dag_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Enum**: `DagType`

### ./server/src/models/dead_code.rs

**File Metrics**: Complexity: 26, Functions: 15

- **Use**: statement
- **Use**: statement
- **Struct**: `FileDeadCodeMetrics`
- **Enum**: `ConfidenceLevel`
- **Struct**: `DeadCodeItem`
- **Enum**: `DeadCodeType`
- **Struct**: `DeadCodeRankingResult`
- **Struct**: `DeadCodeSummary`
- **Struct**: `DeadCodeAnalysisConfig`
- **Use**: statement
- **Use**: statement
- **Function**: `test_file_dead_code_metrics_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dead_code_item_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dead_code_type_variants` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_confidence_levels` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dead_code_ranking_result` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dead_code_summary_from_files` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dead_code_analysis_config_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_metrics_add_different_item_types` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_score_calculation_with_different_confidence_levels` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `DeadCodeResult`

### ./server/src/models/deep_context_config.rs

**File Metrics**: Complexity: 83, Functions: 17

- **Use**: statement
- **Use**: statement
- **Struct**: `DeepContextConfig`
- **Struct**: `ComplexityThresholds`
- **Function**: `default_dead_code_threshold` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `default_cyclomatic_warning` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `default_cyclomatic_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `default_cognitive_warning` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `default_cognitive_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_default_config_validation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_entry_point_validation` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_threshold_validation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_entry_point_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_config_serialization` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/error.rs

**File Metrics**: Complexity: 164, Functions: 16

- **Use**: statement
- **Use**: statement
- **Enum**: `TemplateError`
- **Enum**: `AnalysisError`
- **Enum**: `PmatError`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `ErrorSeverity`
- **Function**: `test_error_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/mcp.rs

**File Metrics**: Complexity: 0, Functions: 3

- **Use**: statement
- **Use**: statement
- **Struct**: `McpRequest`
- **Struct**: `McpResponse`
- **Struct**: `McpError`
- **Struct**: `ToolCallParams`
- **Struct**: `GenerateTemplateArgs`
- **Struct**: `ListTemplatesArgs`
- **Struct**: `ResourceReadParams`
- **Struct**: `ValidateTemplateArgs`
- **Struct**: `ScaffoldProjectArgs`
- **Struct**: `SearchTemplatesArgs`
- **Struct**: `PromptGetParams`
- **Struct**: `Prompt`
- **Struct**: `PromptArgument`
- **Function**: `test_mcp_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/mod.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Function**: `test_mod_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/project_meta.rs

**File Metrics**: Complexity: 14, Functions: 3

- **Use**: statement
- **Use**: statement
- **Enum**: `MetaFileType`
- **Struct**: `MetaFile`
- **Struct**: `CompressedMakefile`
- **Struct**: `MakeTarget`
- **Struct**: `CompressedReadme`
- **Struct**: `CompressedSection`
- **Struct**: `BuildInfo`
- **Struct**: `ProjectOverview`
- **Function**: `test_project_meta_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/refactor.rs

**File Metrics**: Complexity: 44, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `RefactorStateMachine`
- **Enum**: `State`
- **Struct**: `StateTransition`
- **Struct**: `RefactorConfig`
- **Struct**: `Thresholds`
- **Struct**: `RefactorStrategies`
- **Struct**: `MetricSet`
- **Enum**: `RefactorOp`
- **Enum**: `NestingStrategy`
- **Struct**: `BytePos`
- **Struct**: `Location`
- **Enum**: `SatdFix`
- **Struct**: `FileId`
- **Struct**: `Violation`
- **Enum**: `ViolationType`
- **Enum**: `Severity`
- **Struct**: `DefectPayload`
- **Enum**: `RefactorType`
- **Struct**: `Summary`
- **Function**: `test_refactor_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/tdg.rs

**File Metrics**: Complexity: 12, Functions: 5

- **Use**: statement
- **Use**: statement
- **Struct**: `TDGScore`
- **Struct**: `TDGComponents`
- **Enum**: `TDGSeverity`
- **Struct**: `TDGConfig`
- **Struct**: `TDGSummary`
- **Struct**: `TDGHotspot`
- **Struct**: `TDGAnalysis`
- **Struct**: `TDGRecommendation`
- **Enum**: `RecommendationType`
- **Struct**: `TDGDistribution`
- **Struct**: `TDGBucket`
- **Use**: statement
- **Function**: `test_tdg_severity_from_value` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_tdg_config_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Struct**: `SatdItem`
- **Enum**: `SatdSeverity`

### ./server/src/models/template.rs

**File Metrics**: Complexity: 16, Functions: 3

- **Use**: statement
- **Use**: statement
- **Struct**: `TemplateResource`
- **Enum**: `Toolchain`
- **Enum**: `TemplateCategory`
- **Struct**: `ParameterSpec`
- **Enum**: `ParameterType`
- **Struct**: `GeneratedTemplate`
- **Struct**: `TemplateResponse`
- **Function**: `test_template_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/models/unified_ast.rs

**File Metrics**: Complexity: 32, Functions: 49

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `Language`
- **Struct**: `NodeFlags`
- **Enum**: `AstKind`
- **Enum**: `FunctionKind`
- **Enum**: `ClassKind`
- **Enum**: `VarKind`
- **Enum**: `ImportKind`
- **Enum**: `ExprKind`
- **Enum**: `StmtKind`
- **Enum**: `TypeKind`
- **Enum**: `ModuleKind`
- **Enum**: `MacroKind`
- **Struct**: `ProofAnnotation`
- **Enum**: `PropertyType`
- **Enum**: `ConfidenceLevel`
- **Enum**: `VerificationMethod`
- **Enum**: `EvidenceType`
- **Struct**: `Location`
- **Struct**: `Span`
- **Struct**: `BytePos`
- **Struct**: `QualifiedName`
- **Enum**: `RelativeLocation`
- **Struct**: `UnifiedAstNode`
- **Struct**: `ColumnStore`
- **Struct**: `AstDag`
- **Struct**: `LanguageParsers`
- **Use**: statement
- **Function**: `test_node_size` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_node_alignment` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_node_flags` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_dag` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/artifact_writer.rs

**File Metrics**: Complexity: 121, Functions: 16

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ArtifactWriter`
- **Struct**: `ArtifactMetadata`
- **Enum**: `ArtifactType`
- **Struct**: `VerificationReport`
- **Struct**: `IntegrityFailure`
- **Struct**: `ArtifactStatistics`
- **Struct**: `TypeStatistics`
- **Struct**: `CleanupReport`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_artifact_writer_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_directory_structure_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_atomic_write_with_hash` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_artifact_tree_writing` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_integrity_verification` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_statistics` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/src/services/artifact_writer_tests.rs

**File Metrics**: Complexity: 8, Functions: 7

- **Use**: statement
- **Use**: statement
- **Function**: `test_artifact_writer_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_artifact_writer_with_existing_manifest` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_create_directory_structure` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_artifact_type_determination` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `test_write_mermaid_artifacts` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_manifest_persistence` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_writing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_based_dependency_analyzer.rs

**File Metrics**: Complexity: 118, Functions: 19

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `AstBasedDependencyAnalyzer`
- **Struct**: `DependencyAnalysis`
- **Struct**: `Dependency`
- **Enum**: `ImportType`
- **Struct**: `Location`
- **Struct**: `BoundaryViolation`
- **Enum**: `ViolationType`
- **Struct**: `BuiltinModuleRegistry`
- **Struct**: `WorkspaceResolver`
- **Struct**: `ModuleBoundary`
- **Enum**: `ArchitectureLayer`
- **Struct**: `RustDependencyVisitor`
- **Enum**: `Scope`
- **Use**: statement
- **Struct**: `ExtractedDependency`
- **Use**: statement
- **Use**: statement
- **Function**: `test_rust_dependency_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_typescript_dependency_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_based_dependency_analyzer_tests.rs

**File Metrics**: Complexity: 4, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_ast_based_dependency_analyzer_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_builtin_module_registry_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_workspace_resolver_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_unsupported_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_rust_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_python_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_typescript_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_c_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_import_type_serialization` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_violation_type_serialization` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_architecture_layer_equality` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dependency_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_boundary_violation_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_c.rs

**File Metrics**: Complexity: 5, Functions: 9

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CAstParser`
- **Use**: statement
- **Use**: statement
- **Function**: `test_parse_simple_c_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_c_with_pointers` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_c_with_goto` [complexity: 4] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_c_with_restrict` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_c_ast_disabled` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compatibility_layer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_c_dispatch.rs

**File Metrics**: Complexity: 117, Functions: 81

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CNodeDispatchBuilder`
- **Struct**: `CInfoDispatchBuilder`
- **Function**: `map_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_variable` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_parameter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_struct` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_enum` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_union` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_typedef` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_if_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_while_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_do_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_for_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_switch_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_goto_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_label_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_return_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_block_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_case_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_break_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_continue_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_call_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_identifier` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_binary_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_unary_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_assignment_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_field_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_pointer_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_sizeof_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_cast_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_conditional_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_subscript_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_number_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_string_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_char_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_null_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_define` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_function_define` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_include` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_ifdef` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_if_directive` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_else_directive` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_endif_directive` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_function_info` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_variable_info` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_type_info` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `CAstDispatchParser`
- **Struct**: `CComplexityCalculator`
- **Struct**: `CNameExtractor`
- **Function**: `extract_name_from_node` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_function_flags` [complexity: 10] [cognitive: 26] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `extract_variable_flags` [complexity: 12] [cognitive: 34] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `hash_name` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_dispatch_parser_simple_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dispatch_builder` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_goto_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_c_dispatch_disabled` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_cpp.rs

**File Metrics**: Complexity: 2, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CppAstParser`
- **Use**: statement
- **Function**: `test_parse_simple_cpp_class` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_cpp_templates` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_cpp_lambdas` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cpp_ast_disabled` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compatibility_layer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_cpp_dispatch.rs

**File Metrics**: Complexity: 144, Functions: 94

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `NodeDispatchBuilder`
- **Struct**: `InfoDispatchBuilder`
- **Function**: `map_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_method` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_constructor` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_destructor` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_operator` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_variable` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_field` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_class` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_struct` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_enum` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_union` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_template` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_namespace` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_typedef` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_if_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_for_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_while_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_do_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_switch_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_case_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_return_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_break_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_continue_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_try_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_throw_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_binary_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_unary_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_assignment_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_call_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_field_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_lambda_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_new_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_delete_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_cast_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_sizeof_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_number_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_string_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_char_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_bool_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_null_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_include` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_define` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_ifdef` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_if_directive` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_function_define` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_using` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_for_each` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_catch` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_goto` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_label` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_block` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_identifier` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_function_info` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_variable_info` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_class_info` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `CppAstDispatchParser`
- **Struct**: `ComplexityCalculator`
- **Struct**: `NameExtractor`
- **Function**: `extract_name_from_node` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_function_flags` [complexity: 16] [cognitive: 50] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `extract_variable_flags` [complexity: 13] [cognitive: 38] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `extract_class_flags` [complexity: 5] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `hash_name` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_dispatch_parser_simple_class` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dispatch_builder` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cpp_dispatch_disabled` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_kotlin.rs

**File Metrics**: Complexity: 128, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `KotlinAstParser`
- **Struct**: `ParseContext`

### ./server/src/services/ast_python.rs

**File Metrics**: Complexity: 136, Functions: 29

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `read_file_content` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `parse_python_content` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_complexity_metrics` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `check_file_classification` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `extract_ast_items` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_python_file_with_complexity` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_python_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_python_file_with_classifier` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `extract_python_items` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Struct**: `PythonComplexityVisitor`
- **Function**: `create_function_item` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `count_class_attributes` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_import_from_items` [complexity: 4] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_python_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_rust.rs

**File Metrics**: Complexity: 112, Functions: 21

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_rust_file_with_complexity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_rust_file_with_complexity_and_classifier` [complexity: 12] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `analyze_rust_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_rust_file_with_classifier` [complexity: 12] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Struct**: `RustComplexityVisitor`
- **Function**: `test_ast_rust_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_rust_unified.rs

**File Metrics**: Complexity: 84, Functions: 22

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `RustAstParser`
- **Struct**: `FileContextBuilder`
- **Use**: statement
- **Function**: `test_rust_parser_capabilities` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rust_parser_can_parse` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rust_parser_parse_content` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_strategies.rs

**File Metrics**: Complexity: 214, Functions: 31

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `AstStrategy`
- **Struct**: `RustAstStrategy`
- **Struct**: `TypeScriptAstStrategy`
- **Struct**: `JavaScriptAstStrategy`
- **Struct**: `PythonAstStrategy`
- **Struct**: `StrategyRegistry`
- **Struct**: `CAstStrategy`
- **Use**: statement
- **Use**: statement
- **Struct**: `CppAstStrategy`
- **Use**: statement
- **Use**: statement
- **Struct**: `KotlinAstStrategy`
- **Use**: statement
- **Use**: statement
- **Function**: `test_ast_strategies_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_strategies_temp.rs

**File Metrics**: Complexity: 0, Functions: 0


### ./server/src/services/ast_strategies_tests.rs

**File Metrics**: Complexity: 13, Functions: 4

- **Use**: statement
- **Function**: `test_language_strategy_enum` [complexity: 14] [cognitive: 24] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `test_get_strategy_for_extension` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_strategy_for_path` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_ast_strategy_trait` [provability: 75%] [coverage: 65%]
- **Function**: `accepts_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_typescript.rs

**File Metrics**: Complexity: 40, Functions: 18

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `TypeScriptParser`
- **Struct**: `TypeScriptSymbol`
- **Enum**: `SymbolKind`
- **Function**: `symbol_to_ast_item` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `analyze_with_dispatch` [complexity: 15] [cognitive: 18] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Use**: statement
- **Function**: `calculate_complexity_with_dispatch` [complexity: 11] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `detect_language_simple` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `analyze_typescript_file_with_complexity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_typescript_file_with_complexity_cached` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_typescript_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_javascript_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_typescript_file_with_classifier` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_javascript_file_with_classifier` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_typescript_parser_simple` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_javascript_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_typescript_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_typescript_parser_disabled` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compatibility_layer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ast_typescript_dispatch.rs

**File Metrics**: Complexity: 120, Functions: 96

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `TsNodeDispatchBuilder`
- **Struct**: `TsInfoDispatchBuilder`
- **Function**: `map_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_method` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_constructor` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_arrow_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_function_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_getter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_setter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_variable` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_parameter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_property` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_field` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_class` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_interface` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_type_alias` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_enum` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_namespace` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_module` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_if_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_while_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_do_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_for_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_for_in_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_for_of_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_switch_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_case_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_return_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_break_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_continue_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_try_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_throw_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_block_stmt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_binary_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_unary_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_assignment_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_call_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_member_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_conditional_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_new_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_this_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_identifier` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_array_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_object_expr` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_number_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_string_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_bool_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_null_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_undefined_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_template_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_regex_literal` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_type_annotation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_generic_type` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_union_type` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_intersection_type` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_mapped_type` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_conditional_type` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_import` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_export` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `map_decorator` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_function_info` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_variable_info` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_type_info` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `TsAstDispatchParser`
- **Struct**: `TsComplexityCalculator`
- **Struct**: `TsNameExtractor`
- **Use**: statement
- **Use**: statement
- **Function**: `test_dispatch_parser_simple_typescript` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dispatch_builder` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_javascript_parsing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_typescript_dispatch_disabled` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/big_o_analyzer.rs

**File Metrics**: Complexity: 172, Functions: 18

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `BigOAnalyzer`
- **Struct**: `BigOAnalysisConfig`
- **Struct**: `BigOAnalysisReport`
- **Struct**: `ComplexityDistribution`
- **Struct**: `FunctionComplexity`
- **Struct**: `PatternMatch`
- **Use**: statement
- **Use**: statement
- **Function**: `test_analyzer_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/big_o_analyzer_tests.rs

**File Metrics**: Complexity: 12, Functions: 14

- **Use**: statement
- **Use**: statement
- **Function**: `test_big_o_analyzer_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_loop_keywords` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_recursive_call` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_sorting_operation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_binary_search` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_loop_depth` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_determine_time_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_space_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analysis_config_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_distribution_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_function_complexity_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pattern_match_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_empty_project` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_serialization` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/adapters.rs

**File Metrics**: Complexity: 4, Functions: 19

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ContentCacheAdapter`
- **Struct**: `PersistentCacheAdapter`
- **Use**: statement
- **Use**: statement
- **Function**: `test_content_cache_adapter` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/adapters_tests.rs

**File Metrics**: Complexity: 9, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `TestKey`
- **Struct**: `TestValue`
- **Function**: `test_content_cache_adapter_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_adapter_get_put` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_adapter_remove` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_adapter_clear` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_content_cache_adapter_stats` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_adapter_evict_if_needed` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_persistent_cache_adapter_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_persistent_cache_adapter_get_put` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_persistent_cache_adapter_remove` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_persistent_cache_adapter_clear` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_persistent_cache_adapter_stats` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_persistent_cache_adapter_evict_if_needed` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./server/src/services/cache/base.rs

**File Metrics**: Complexity: 2, Functions: 14

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `CacheStrategy`
- **Struct**: `CacheEntry`
- **Struct**: `CacheStats`

### ./server/src/services/cache/base_tests.rs

**File Metrics**: Complexity: 2, Functions: 10

- **Use**: statement
- **Use**: statement
- **Function**: `test_cache_entry_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_entry_access` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_entry_age` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_stats_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_stats_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `TestStrategy`
- **Function**: `test_cache_strategy_trait` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/cache_trait.rs

**File Metrics**: Complexity: 0, Functions: 0

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `AstCacheManager`

### ./server/src/services/cache/config.rs

**File Metrics**: Complexity: 18, Functions: 9

- **Use**: statement
- **Use**: statement
- **Struct**: `CacheConfig`
- **Function**: `test_config_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/content_cache.rs

**File Metrics**: Complexity: 23, Functions: 21

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ContentCache`
- **Struct**: `CacheMetrics`
- **Use**: statement
- **Use**: statement
- **Struct**: `TestStrategy`
- **Function**: `test_content_cache_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_clear` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_stats` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/content_cache_tests.rs

**File Metrics**: Complexity: 9, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `TestKey`
- **Struct**: `TestValue`
- **Function**: `test_content_cache_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_get_put` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_remove` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_clear` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_lru_eviction` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_ttl_expiration` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_update_existing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_metrics` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_invalidate_matching` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_memory_estimation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_content_cache_clone` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_metrics_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/diagnostics.rs

**File Metrics**: Complexity: 39, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CacheDiagnostics`
- **Struct**: `CacheStatsSnapshot`
- **Struct**: `CacheEffectiveness`
- **Struct**: `CacheDiagnosticReport`
- **Function**: `format_prometheus_metrics` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `test_cache_stats_snapshot` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_diagnostic_report_high_memory_pressure` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_diagnostic_report_low_hit_rate` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_effectiveness` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_report_healthy` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_prometheus_metrics` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/manager.rs

**File Metrics**: Complexity: 33, Functions: 18

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SessionCacheManager`
- **Use**: statement
- **Function**: `test_session_cache_manager_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_clear_all` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_diagnostics` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_pressure` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/mod.rs

**File Metrics**: Complexity: 0, Functions: 0

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement

### ./server/src/services/cache/persistent.rs

**File Metrics**: Complexity: 100, Functions: 19

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `PersistentCacheEntry`
- **Struct**: `PersistentCache`
- **Use**: statement
- **Use**: statement
- **Function**: `test_persistent_cache_entry_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_persistent_cache_entry_age` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_file_path` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/persistent_manager.rs

**File Metrics**: Complexity: 22, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `PersistentCacheManager`
- **Function**: `test_persistent_manager_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/strategies.rs

**File Metrics**: Complexity: 36, Functions: 28

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `AstCacheStrategy`
- **Struct**: `TemplateCacheStrategy`
- **Struct**: `DagCacheStrategy`
- **Struct**: `ChurnCacheStrategy`
- **Struct**: `GitStatsCacheStrategy`
- **Struct**: `GitStats`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_ast_cache_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_cache_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dag_cache_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_churn_cache_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_git_stats_cache_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/unified.rs

**File Metrics**: Complexity: 31, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `UnifiedCache`
- **Struct**: `VectorizedCacheKey`
- **Use**: statement
- **Struct**: `UnifiedCacheConfig`
- **Struct**: `LayeredCache`
- **Use**: statement
- **Function**: `test_vectorized_cache_key` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vectorized_cache_key_from_bytes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/cache/unified_manager.rs

**File Metrics**: Complexity: 36, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `UnifiedCacheManager`
- **Struct**: `UnifiedCacheDiagnostics`
- **Use**: statement
- **Function**: `test_unified_cache_manager` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_cache_functionality` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_cache_eviction` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/canonical_query.rs

**File Metrics**: Complexity: 71, Functions: 25

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `CanonicalQuery`
- **Struct**: `AnalysisContext`
- **Struct**: `CallGraph`
- **Struct**: `CallNode`
- **Enum**: `CallNodeType`
- **Struct**: `CallEdge`
- **Enum**: `CallEdgeType`
- **Struct**: `QueryResult`
- **Struct**: `GraphMetadata`
- **Struct**: `SystemArchitectureQuery`
- **Struct**: `Component`
- **Struct**: `ComponentEdge`
- **Enum**: `ComponentEdgeType`
- **Struct**: `ComponentMetrics`
- **Function**: `detect_architectural_components` [complexity: 4] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `infer_component_relationships` [complexity: 12] [cognitive: 27] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `aggregate_component_metrics` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `generate_styled_architecture_diagram` [complexity: 21] [cognitive: 35] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Function**: `sanitize_component_id` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `humanize_component_name` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `collect_component_nodes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `merge_coupled_components` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_graph_diameter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_call_graph_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_call_node_types` [complexity: 9] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `test_call_edge_types` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_graph_metadata` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_query_result` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analysis_context` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `TestQuery`
- **Function**: `test_canonical_query_trait` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_component_edge_type_hash` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_component_edge_type_equality` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/code_intelligence.rs

**File Metrics**: Complexity: 70, Functions: 14

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `AnalysisRequest`
- **Enum**: `AnalysisType`
- **Use**: statement
- **Struct**: `AnalysisReport`
- **Struct**: `ComplexityReport`
- **Struct**: `ComplexityHotspot`
- **Struct**: `DependencyGraphReport`
- **Struct**: `DefectScore`
- **Struct**: `GraphMetricsReport`
- **Struct**: `CentralityScore`
- **Struct**: `UnifiedCache`
- **Struct**: `CodeIntelligence`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_dag_enhanced` [complexity: 21] [cognitive: 26] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Use**: statement
- **Function**: `test_analysis_request_cache_key` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_cache` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_code_intelligence_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/complexity.rs

**File Metrics**: Complexity: 126, Functions: 54

- **Use**: statement
- **Use**: statement
- **Struct**: `ComplexityMetrics`
- **Struct**: `FileComplexityMetrics`
- **Struct**: `FunctionComplexity`
- **Struct**: `ClassComplexity`
- **Struct**: `ComplexityThresholds`
- **Enum**: `Violation`
- **Struct**: `ComplexitySummary`
- **Struct**: `ComplexityHotspot`
- **Struct**: `ComplexityReport`
- **Struct**: `ComplexityVisitor`
- **Function**: `compute_complexity_cache_key` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Trait**: `ComplexityRule`
- **Struct**: `CyclomaticComplexityRule`
- **Struct**: `CognitiveComplexityRule`
- **Function**: `aggregate_results` [complexity: 17] [cognitive: 25] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `format_complexity_summary` [complexity: 23] [cognitive: 27] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 54%]
- **Function**: `format_complexity_report` [complexity: 12] [cognitive: 22] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `format_as_sarif` [complexity: 8] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_metrics` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_metrics_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_metrics_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_thresholds_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_thresholds_custom` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_function_complexity_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_class_complexity_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_complexity_metrics_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_cognitive_increment` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_cognitive_increment_with_nesting` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_nesting_management` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_nesting_saturation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compute_complexity_cache_key` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compute_complexity_cache_key_different_paths` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cyclomatic_complexity_rule_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cyclomatic_complexity_rule_exceeds_threshold` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cyclomatic_complexity_rule_no_violation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cyclomatic_complexity_rule_warning` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cyclomatic_complexity_rule_error` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cyclomatic_complexity_rule_without_function_name` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cognitive_complexity_rule_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cognitive_complexity_rule_no_violation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cognitive_complexity_rule_warning` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cognitive_complexity_rule_error` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_complexity_hotspot_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_single_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_with_classes` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_median_calculation_odd` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_percentile_calculation` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_technical_debt_calculation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_hotspot_sorting` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_complexity_summary_empty` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_complexity_summary_with_data` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_complexity_report` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_as_sarif` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_as_sarif_empty` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_violation_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/complexity_comprehensive_tests.rs

**File Metrics**: Complexity: 49, Functions: 40

- **Use**: statement
- **Use**: statement
- **Function**: `create_test_metrics` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_metrics_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_metrics_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_thresholds_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_thresholds_custom` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_function_complexity_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_class_complexity_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_complexity_metrics_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_cognitive_increment` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_cognitive_increment_with_nesting` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_nesting_management` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_visitor_nesting_saturation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compute_complexity_cache_key` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compute_complexity_cache_key_different_paths` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cyclomatic_complexity_rule_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cyclomatic_complexity_rule_exceeds_threshold` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cyclomatic_complexity_rule_no_violation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cyclomatic_complexity_rule_warning` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cyclomatic_complexity_rule_error` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cyclomatic_complexity_rule_without_function_name` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cognitive_complexity_rule_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cognitive_complexity_rule_no_violation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cognitive_complexity_rule_warning` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cognitive_complexity_rule_error` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_complexity_hotspot_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_single_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_with_classes` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_median_calculation_odd` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_percentile_calculation` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_technical_debt_calculation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_hotspot_sorting` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_complexity_summary_empty` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_complexity_summary_with_data` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_complexity_report` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_as_sarif` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_as_sarif_empty` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_violation_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/complexity_patterns.rs

**File Metrics**: Complexity: 82, Functions: 24

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ComplexityPatternMatcher`
- **Struct**: `ComplexityPattern`
- **Enum**: `PatternType`
- **Struct**: `ComplexityAnalysisResult`
- **Use**: statement
- **Use**: statement
- **Function**: `test_pattern_matcher_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_bound_properties` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_loop_complexity_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/complexity_patterns_tests.rs

**File Metrics**: Complexity: 24, Functions: 8

- **Use**: statement
- **Function**: `test_pattern_type_enum` [complexity: 16] [cognitive: 28] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `test_complexity_pattern_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pattern_matcher_trait` [provability: 75%] [coverage: 65%]
- **Struct**: `TestMatcher`
- **Function**: `test_get_default_patterns` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_pattern_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `get_default_patterns` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_pattern_complexity` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/src/services/context.rs

**File Metrics**: Complexity: 225, Functions: 52

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ProjectContext`
- **Struct**: `ProjectSummary`
- **Struct**: `FileContext`
- **Enum**: `AstItem`
- **Struct**: `RustVisitor`
- **Function**: `analyze_rust_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_rust_file_with_cache` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `analyze_project` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_rust_file_with_persistent_cache` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `analyze_project_with_cache` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `build_gitignore` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `scan_and_analyze_files` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_file_by_toolchain` [complexity: 12] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_deno_file` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `build_project_summary` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_item_counts` [complexity: 10] [cognitive: 23] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `read_dependencies` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `read_rust_dependencies` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `read_deno_dependencies` [complexity: 8] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `read_python_dependencies` [complexity: 9] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `analyze_project_with_persistent_cache` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `scan_and_analyze_files_persistent` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `analyze_file_by_toolchain_persistent` [complexity: 12] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Use**: statement
- **Use**: statement
- **Function**: `format_context_as_markdown` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_header` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_summary` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `format_dependencies` [complexity: 4] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `format_files` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `GroupedItems`
- **Function**: `group_items_by_type` [complexity: 10] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `format_item_groups` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_item_group` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `format_module_item` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_struct_item` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `format_enum_item` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_trait_item` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_function_item` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_impl_item` [complexity: 7] [cognitive: 11] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `format_footer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_deep_context_as_markdown` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `format_quality_scorecard` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `format_project_summary` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `format_analysis_results` [complexity: 26] [cognitive: 29] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 61%]
- **Function**: `format_ast_summary` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `count_ast_items` [complexity: 10] [cognitive: 23] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `test_context_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/dag_builder.rs

**File Metrics**: Complexity: 134, Functions: 21

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DagBuilder`
- **Function**: `filter_call_edges` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `filter_import_edges` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `filter_inheritance_edges` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `add_pagerank_scores` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `prune_graph_pagerank` [complexity: 12] [cognitive: 16] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `test_dag_builder_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/src/services/dead_code_analyzer.rs

**File Metrics**: Complexity: 133, Functions: 34

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `HierarchicalBitSet`
- **Struct**: `CrossLangReferenceGraph`
- **Struct**: `ReferenceEdge`
- **Struct**: `ReferenceNode`
- **Enum**: `ReferenceType`
- **Struct**: `VTableResolver`
- **Struct**: `VTable`
- **Struct**: `CoverageData`
- **Struct**: `DeadCodeReport`
- **Struct**: `DeadCodeItem`
- **Enum**: `DeadCodeType`
- **Struct**: `UnreachableBlock`
- **Struct**: `DeadCodeSummary`
- **Struct**: `DeadCodeAnalyzer`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_hierarchical_bitset` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dead_code_analyzer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vtable_resolver` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_reference_edge_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_reference_node_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dead_code_analyzer_with_entry_points` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_coverage_data_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_cross_lang_reference_graph` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_with_ranking` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Use**: statement

### ./server/src/services/dead_code_prover.rs

**File Metrics**: Complexity: 109, Functions: 24

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SymbolId`
- **Enum**: `EntryPointType`
- **Struct**: `ReachabilityAnalyzer`
- **Struct**: `FFIReferenceTracker`
- **Struct**: `DynamicDispatchAnalyzer`
- **Enum**: `Usage`
- **Struct**: `DeadCodeProof`
- **Enum**: `DeadCodeProofType`
- **Struct**: `Evidence`
- **Enum**: `EvidenceType`
- **Struct**: `DeadCodeProver`
- **Use**: statement
- **Use**: statement
- **Function**: `test_ffi_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dead_code_prover` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/deep_context.rs

**File Metrics**: Complexity: 1312, Functions: 144

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DeepContextConfig`
- **Enum**: `AnalysisType`
- **Enum**: `DagType`
- **Struct**: `ComplexityThresholds`
- **Enum**: `CacheStrategy`
- **Struct**: `DeepContext`
- **Struct**: `DeepContextResult`
- **Struct**: `AstSummary`
- **Struct**: `DeadCodeAnalysis`
- **Struct**: `DeadCodeSummary`
- **Struct**: `ComplexityMetricsForQA`
- **Struct**: `FileComplexityMetricsForQA`
- **Struct**: `FunctionComplexityForQA`
- **Struct**: `ComplexitySummaryForQA`
- **Struct**: `ContextMetadata`
- **Struct**: `CacheStats`
- **Struct**: `AnnotatedFileTree`
- **Struct**: `AnnotatedNode`
- **Enum**: `NodeType`
- **Struct**: `NodeAnnotations`
- **Struct**: `AnalysisResults`
- **Struct**: `EnhancedFileContext`
- **Struct**: `FileChurnMetrics`
- **Struct**: `DefectAnnotations`
- **Struct**: `DeadCodeAnnotation`
- **Enum**: `ConfidenceLevel`
- **Struct**: `DeadCodeItem`
- **Enum**: `DeadCodeItemType`
- **Struct**: `TechnicalDebtItem`
- **Enum**: `TechnicalDebtCategory`
- **Enum**: `TechnicalDebtSeverity`
- **Struct**: `ComplexityViolation`
- **Enum**: `ComplexityMetricType`
- **Struct**: `CrossLangReference`
- **Enum**: `CrossLangReferenceType`
- **Struct**: `QualityScorecard`
- **Struct**: `TemplateProvenance`
- **Struct**: `DriftAnalysis`
- **Struct**: `DefectSummary`
- **Struct**: `DefectHotspot`
- **Struct**: `FileLocation`
- **Enum**: `DefectFactor`
- **Struct**: `RefactoringEstimate`
- **Enum**: `Priority`
- **Enum**: `Impact`
- **Struct**: `PrioritizedRecommendation`
- **Struct**: `CategorizedAstItems`
- **Struct**: `AstFunction`
- **Struct**: `AstStruct`
- **Struct**: `AstEnum`
- **Struct**: `AstTrait`
- **Struct**: `AstImpl`
- **Struct**: `AstModule`
- **Struct**: `AstUse`
- **Struct**: `DeepContextAnalyzer`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ParallelAnalysisResults`
- **Enum**: `AnalysisResult`
- **Function**: `analyze_ast_contexts` [complexity: 12] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_single_file` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `detect_language` [complexity: 11] [cognitive: 19] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `analyze_rust_file` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `analyze_typescript_file` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `analyze_python_file` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Function**: `analyze_c_file` [complexity: 7] [cognitive: 8] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_kotlin_file` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_complexity` [complexity: 11] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_churn` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `analyze_dead_code` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_file_for_dead_code` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Function**: `analyze_rust_dead_code` [complexity: 10] [cognitive: 18] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Function**: `analyze_typescript_dead_code` [complexity: 10] [cognitive: 18] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Function**: `analyze_python_dead_code` [complexity: 8] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `extract_function_name` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_struct_name` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_js_function_name` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_class_name` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_python_function_name` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_python_class_name` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_function_called_in_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_type_used_in_file` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `analyze_duplicate_code` [complexity: 18] [cognitive: 41] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 42%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_satd` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `analyze_provability` [complexity: 8] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_dag` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `analyze_big_o` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_deep_context_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/deep_context_orchestrator.rs

**File Metrics**: Complexity: 40, Functions: 11

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DeepContextOrchestrator`
- **Struct**: `DeepContextConfig`
- **Struct**: `FeatureFlags`
- **Enum**: `CacheStrategy`
- **Enum**: `PerformanceMode`
- **Struct**: `OrchestrationRequest`
- **Struct**: `DeepContextReport`
- **Struct**: `ComplexitySummary`
- **Struct**: `CodeHotspot`
- **Enum**: `HotspotType`
- **Enum**: `HotspotSeverity`
- **Struct**: `HotspotMetrics`
- **Struct**: `Recommendation`
- **Enum**: `RecommendationCategory`
- **Enum**: `RecommendationImpact`
- **Enum**: `RecommendationEffort`
- **Use**: statement
- **Use**: statement
- **Struct**: `DeepContextOrchestratorFactory`
- **Function**: `test_deep_context_orchestrator_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/deep_context_tests.rs


### ./server/src/services/defect_probability.rs

**File Metrics**: Complexity: 60, Functions: 18

- **Use**: statement
- **Use**: statement
- **Struct**: `DefectProbabilityCalculator`
- **Struct**: `DefectWeights`
- **Struct**: `FileMetrics`
- **Struct**: `DefectScore`
- **Enum**: `RiskLevel`
- **Function**: `interpolate_cdf` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Struct**: `ProjectDefectAnalysis`
- **Use**: statement
- **Function**: `test_defect_probability_calculation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cdf_interpolation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_project_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/deterministic_mermaid_engine.rs

**File Metrics**: Complexity: 106, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DeterministicMermaidEngine`
- **Enum**: `ComplexityBucket`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_pagerank_determinism` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mermaid_output_determinism` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_sanitize_id` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_escape_mermaid_label` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_service_module` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_styling` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_graph` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/dogfooding_engine.rs

**File Metrics**: Complexity: 146, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DogfoodingEngine`
- **Struct**: `FileContext`
- **Struct**: `ChurnMetrics`
- **Struct**: `FileHotspot`
- **Struct**: `DagMetrics`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_ast_context_generation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_combined_metrics_generation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_server_info_generation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/duplicate_detector.rs

**File Metrics**: Complexity: 314, Functions: 69

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `Language`
- **Enum**: `CloneType`
- **Enum**: `TokenKind`
- **Struct**: `Token`
- **Struct**: `MinHashSignature`
- **Struct**: `CodeFragment`
- **Struct**: `CloneInstance`
- **Struct**: `CloneGroup`
- **Struct**: `CloneSummary`
- **Struct**: `DuplicationHotspot`
- **Struct**: `CloneReport`
- **Struct**: `DuplicateDetectionConfig`
- **Struct**: `UniversalFeatureExtractor`
- **Struct**: `MinHashGenerator`
- **Struct**: `DuplicateDetectionEngine`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_tokens` [complexity: 8] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_token_creation_and_hash` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_all_token_kinds` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_minhash_signature_jaccard_similarity` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_clone_type_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_detection_config_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_universal_feature_extractor` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_normalize_tokens` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_normalize_tokens_keep_identifiers_and_literals` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_minhash_generator` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_code_fragment_creation` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_detection_engine_basic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_detection_different_languages` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_fragments` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_clone_pairs_with_lsh` [complexity: 10] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `test_group_clones` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_compute_summary` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compute_hotspots` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_representative` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_file_handling` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_c_and_cpp_languages` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_language_specific_edge_cases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_function_start` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_function_end` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_canonicalize_identifier` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_token_hash` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_minhash_similarity` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_feature_extraction` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_shingle_generation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/duplicate_detector_tests.rs

**File Metrics**: Complexity: 36, Functions: 25

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_tokens` [complexity: 8] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_token_creation_and_hash` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_all_token_kinds` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_minhash_signature_jaccard_similarity` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_clone_type_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_detection_config_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_universal_feature_extractor` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_identifiers` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_literals` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_normalize_tokens` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_normalize_tokens_keep_identifiers_and_literals` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_minhash_generator` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_code_fragment_creation` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_detection_engine_basic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplicate_detection_different_languages` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_fragments` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_clone_pairs_with_lsh` [complexity: 10] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `test_group_clones` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compute_summary` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_hotspots` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_significant_fragment` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_find_representative` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_file_handling` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_c_and_cpp_languages` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_language_specific_edge_cases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/embedded_templates.rs

**File Metrics**: Complexity: 84, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `EmbeddedTemplateMetadata`
- **Struct**: `EmbeddedParameter`
- **Function**: `convert_to_template_resource` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `parse_template_category` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `parse_toolchain` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `convert_embedded_parameters` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `convert_embedded_parameter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `parse_parameter_type` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `convert_json_value_to_string` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `build_s3_object_key` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `get_category_path` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `list_templates` [complexity: 8] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `get_template_metadata` [complexity: 16] [cognitive: 16] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `get_template_content` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `test_embedded_templates_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/enhanced_reporting.rs

**File Metrics**: Complexity: 192, Functions: 28

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `EnhancedReportingService`
- **Struct**: `ReportConfig`
- **Enum**: `ReportFormat`
- **Struct**: `UnifiedAnalysisReport`
- **Struct**: `ReportMetadata`
- **Struct**: `ExecutiveSummary`
- **Enum**: `RiskLevel`
- **Struct**: `ReportSection`
- **Enum**: `SectionType`
- **Struct**: `MetricValue`
- **Enum**: `Trend`
- **Struct**: `Finding`
- **Enum**: `Severity`
- **Struct**: `Location`
- **Enum**: `EffortLevel`
- **Struct**: `Recommendation`
- **Enum**: `Priority`
- **Struct**: `Visualization`
- **Enum**: `VisualizationType`
- **Struct**: `AnalysisResults`
- **Struct**: `ComplexityAnalysis`
- **Struct**: `DeadCodeAnalysis`
- **Struct**: `DuplicationAnalysis`
- **Struct**: `TdgAnalysis`
- **Struct**: `BigOAnalysis`
- **Use**: statement
- **Function**: `test_health_score_calculation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_risk_assessment` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/file_classifier.rs

**File Metrics**: Complexity: 94, Functions: 28

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `FileClassifierConfig`
- **Struct**: `FileClassifier`
- **Enum**: `ParseDecision`
- **Enum**: `SkipReason`
- **Function**: `calculate_shannon_entropy` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Struct**: `DebugReporter`
- **Struct**: `DebugEvent`
- **Struct**: `DebugReport`
- **Struct**: `DebugSummary`
- **Struct**: `VendorRules`
- **Use**: statement
- **Function**: `test_large_file_detection` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_include_large_files_flag` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_very_large_files_still_skipped` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_skip_reason_priorities` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_minified_vs_large_file_detection` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vendor_detection_determinism` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_performance_on_large_files` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_entropy_calculation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_line_length_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rust_target_directory_filtering` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_additional_build_artifacts` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_debug_reporter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/file_discovery.rs

**File Metrics**: Complexity: 131, Functions: 23

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `FileCategory`
- **Struct**: `FileDiscoveryConfig`
- **Struct**: `ProjectFileDiscovery`
- **Use**: statement
- **Use**: statement
- **Struct**: `DiscoveryStats`
- **Struct**: `ExternalRepoFilter`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_file_discovery_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_external_repo_filtering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_max_depth_limit` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_custom_ignore_patterns` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_extension_filtering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_cython_file_discovery` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_categorization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_discovery_stats` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]

### ./server/src/services/fixed_graph_builder.rs

**File Metrics**: Complexity: 55, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `GraphConfig`
- **Enum**: `GroupingStrategy`
- **Struct**: `FixedGraph`
- **Struct**: `FixedNode`
- **Struct**: `FixedGraphBuilder`
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_graph` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_deterministic_build` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_node_limit` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/git_analysis.rs

**File Metrics**: Complexity: 42, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `GitAnalysisService`
- **Struct**: `FileStats`
- **Struct**: `CommitInfo`
- **Function**: `test_git_analysis_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/git_clone.rs

**File Metrics**: Complexity: 126, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CloneProgress`
- **Struct**: `ClonedRepo`
- **Enum**: `CloneError`
- **Struct**: `GitCloner`
- **Struct**: `ParsedGitHubUrl`
- **Use**: statement
- **Use**: statement
- **Function**: `test_parse_github_urls` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_validate_github_name` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_key_generation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/src/services/incremental_churn.rs

**File Metrics**: Complexity: 80, Functions: 25

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ChurnCacheEntry`
- **Struct**: `IncrementalChurnAnalyzer`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_incremental_churn_cache` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_stats` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_commit_line` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_numstat_line` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_summary` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_get_file_churn_no_git` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_incremental_empty` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_batch_compute_churn_no_git` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_current_commit_hash_no_git` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_file_last_commit_hash_no_git` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_cache_valid` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compute_file_churn_parsing` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/src/services/incremental_coverage_analyzer.rs

**File Metrics**: Complexity: 117, Functions: 20

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `IncrementalCoverageAnalyzer`
- **Struct**: `FileId`
- **Struct**: `AstNode`
- **Struct**: `FunctionInfo`
- **Struct**: `CoverageUpdate`
- **Struct**: `FileCoverage`
- **Struct**: `AggregateCoverage`
- **Struct**: `DeltaCoverage`
- **Struct**: `ChangeSet`
- **Struct**: `CallGraph`
- **Function**: `extract_function_name` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_incremental_coverage` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./server/src/services/lightweight_provability_analyzer.rs

**File Metrics**: Complexity: 131, Functions: 20

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `LightweightProvabilityAnalyzer`
- **Struct**: `FunctionId`
- **Struct**: `ProofSummary`
- **Struct**: `VerifiedProperty`
- **Enum**: `PropertyType`
- **Struct**: `PropertyDomain`
- **Enum**: `NullabilityLattice`
- **Struct**: `IntervalLattice`
- **Enum**: `AliasLattice`
- **Enum**: `PurityLattice`
- **Struct**: `AbstractInterpreter`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_nullability_lattice` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_property_domain_join` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_incremental_analysis` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_compressor.rs

**File Metrics**: Complexity: 138, Functions: 14

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `MakefileCompressor`
- **Struct**: `ParsedTarget`
- **Function**: `extract_package_name` [complexity: 15] [cognitive: 24] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Use**: statement
- **Function**: `test_compress_basic_makefile` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compress_rust_makefile` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_recipe_summarization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dependency_extraction` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_toolchain_detection` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_linter/ast.rs

**File Metrics**: Complexity: 67, Functions: 23

- **Use**: statement
- **Struct**: `MakefileAst`
- **Struct**: `MakefileNode`
- **Enum**: `MakefileNodeKind`
- **Enum**: `NodeData`
- **Enum**: `AssignmentOp`
- **Struct**: `RecipeLine`
- **Struct**: `RecipePrefixes`
- **Struct**: `SourceSpan`
- **Struct**: `MakefileMetadata`
- **Use**: statement
- **Function**: `test_makefile_ast_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_add_node` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_rules_by_target` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_phony_targets` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_source_span` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_count_targets` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_count_phony_targets` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_has_pattern_rules` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_uses_automatic_variables` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_variables` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_metadata_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_linter/coverage_tests.rs

**File Metrics**: Complexity: 14, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_parser_edge_cases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_utf8_handling` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_all_assignment_operators` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_recipe_prefixes` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complex_makefile` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_line_continuation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_double_colon_rules` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pattern_rules` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_automatic_variables` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rule_registry` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_lint_result_methods` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_errors` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cursor_safety` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_performance_rules` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_source_span_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_makefile_node_types` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_quality_score_edge_cases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_linter/mod.rs

**File Metrics**: Complexity: 12, Functions: 9

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `lint_makefile` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `calculate_quality_score` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_calculate_quality_score_perfect` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_quality_score_with_errors` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_quality_score_with_warnings` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_calculate_quality_score_minimum` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_lint_result_methods` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_lint_makefile_async` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_lint_makefile_file_not_found` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_linter/parser.rs

**File Metrics**: Complexity: 189, Functions: 46

- **Use**: statement
- **Struct**: `MakefileParser`
- **Enum**: `ParseError`
- **Enum**: `LineType`
- **Use**: statement
- **Function**: `test_parser_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_empty_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_simple_rule` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_variable` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_comment` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_pattern_rule` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_phony_rule` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_double_colon_rule` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_include` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_recipe_with_prefixes` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_automatic_variables` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_errors` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_skip_functions` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_at_end` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_advance` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_starts_with` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_linter/property_tests.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Use**: statement
- **Use**: statement
- **Function**: `test_property_tests_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_linter/rules/checkmake.rs

**File Metrics**: Complexity: 161, Functions: 41

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `MinPhonyRule`
- **Struct**: `PhonyDeclaredRule`
- **Struct**: `MaxBodyLengthRule`
- **Struct**: `TimestampExpandedRule`
- **Struct**: `UndefinedVariableRule`
- **Struct**: `VariableRef`
- **Enum**: `VarRefType`
- **Struct**: `VariableScanner`
- **Function**: `check_undefined_in_text` [complexity: 4] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `extract_var_name` [complexity: 12] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `should_check_variable` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `create_undefined_violation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_automatic_var` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_function_call` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `PortabilityRule`
- **Use**: statement
- **Use**: statement
- **Function**: `test_min_phony_rule` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_phony_declared_rule` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_max_body_length_rule` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_timestamp_expanded_rule` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_undefined_variable_rule` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_portability_rule` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_automatic_var` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_function_call` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_linter/rules/mod.rs

**File Metrics**: Complexity: 27, Functions: 17

- **Use**: statement
- **Use**: statement
- **Enum**: `Severity`
- **Struct**: `Violation`
- **Struct**: `LintResult`
- **Trait**: `MakefileRule`
- **Struct**: `RuleRegistry`
- **Use**: statement
- **Use**: statement
- **Function**: `test_severity_ordering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_violation_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rule_registry_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rule_registry_register` [provability: 75%] [coverage: 65%]
- **Struct**: `TestRule`
- **Function**: `test_check_all_empty_ast` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_check_all_sorting` [provability: 75%] [coverage: 65%]
- **Struct**: `TestRule`
- **Function**: `test_default_trait_implementation` [provability: 75%] [coverage: 65%]
- **Struct**: `MinimalRule`
- **Function**: `test_makefile_with_phony_targets` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/makefile_linter/rules/performance.rs

**File Metrics**: Complexity: 83, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `RecursiveExpansionRule`
- **Function**: `extract_var_refs` [complexity: 16] [cognitive: 44] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `count_var_usage` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `is_function_call` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `is_automatic_var` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_recursive_expansion_rule` [complexity: 9] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `test_extract_var_refs` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_count_var_usage` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_expensive_propagation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_multiple_targets_with_expensive_prereq` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/mermaid_generator.rs

**File Metrics**: Complexity: 171, Functions: 42

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `MermaidGenerator`
- **Struct**: `MermaidOptions`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `load_reference_standard` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `load_complex_styled_standard` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `load_invalid_example` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `validate_mermaid_syntax` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `validate_mermaid_directive` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `validate_content_not_empty` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `validate_no_raw_angle_brackets` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `validate_node_definitions` [complexity: 20] [cognitive: 28] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 47%]
- **Function**: `test_reference_standards_are_valid` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_invalid_example_is_correctly_identified` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generated_output_matches_reference_syntax` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_all_node_types` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complex_labels` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_sanitize_id` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_all_edge_types` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_graph` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_no_complexity_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_without_complexity_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_edge_with_missing_node` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_default_implementation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_numeric_id_sanitization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_options_configuration` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mermaid_output_format` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_regression_empty_nodes_bug` [complexity: 12] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `test_escape_mermaid_label` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_angle_brackets_and_pipes_compatibility` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_edge_arrow_syntax` [complexity: 6] [cognitive: 8] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_realistic_dependency_graph` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/src/services/mermaid_property_tests.rs

**File Metrics**: Complexity: 2, Functions: 5

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `node_id_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `label_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `node_type_strategy` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `edge_type_strategy` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `node_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/mod.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Function**: `test_mod_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/old_cache.rs

**File Metrics**: Complexity: 2, Functions: 5

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `get_metadata` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `put_metadata` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `get_content` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `put_content` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_old_cache_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/parallel_git.rs

**File Metrics**: Complexity: 70, Functions: 33

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ParallelGitConfig`
- **Struct**: `CacheEntry`
- **Struct**: `ParallelGitExecutor`
- **Struct**: `CommitInfo`
- **Struct**: `DiffStats`
- **Use**: statement
- **Use**: statement
- **Function**: `test_parallel_git_executor` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_key_generation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_config_defaults` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_custom_config` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_batch` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_command_failure` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_clear_cache` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_commit_log` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_commit_log_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_commit_log_invalid` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_diff_stats` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_diff_stats_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_file_histories_no_git` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_file_blames_no_git` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_diff_stats_no_git` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_clone_executor` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_ttl` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_concurrent_execution` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_execute_batch_owned` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/src/services/parsed_file_cache.rs

**File Metrics**: Complexity: 38, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ParsedFileCacheKey`
- **Enum**: `CacheType`
- **Enum**: `CachedData`
- **Struct**: `CachedEntry`
- **Struct**: `ParsedFileCache`
- **Struct**: `CacheStats`
- **Use**: statement
- **Function**: `test_parsed_file_cache_context` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/src/services/progress.rs

**File Metrics**: Complexity: 26, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ProgressTracker`
- **Struct**: `FileClassificationReporter`
- **Use**: statement

### ./server/src/services/project_meta_detector.rs

**File Metrics**: Complexity: 31, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ProjectMetaDetector`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_detect_metadata_files` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_various_makefile_variants` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_various_readme_variants` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_max_depth_limitation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_read_timeout` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/proof_annotator.rs

**File Metrics**: Complexity: 59, Functions: 19

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `ProofCollectionError`
- **Struct**: `ProofCollectionResult`
- **Struct**: `CollectionMetrics`
- **Trait**: `ProofSource`
- **Struct**: `ProofCache`
- **Struct**: `ProofAnnotator`
- **Struct**: `CacheStats`
- **Struct**: `MockProofSource`
- **Use**: statement
- **Use**: statement
- **Function**: `test_proof_annotator_basic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_proof_annotator_parallel_sources` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_proof_cache` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/quality_gates.rs

**File Metrics**: Complexity: 94, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `QAVerificationResult`
- **Struct**: `DeadCodeVerification`
- **Struct**: `ComplexityVerification`
- **Struct**: `ProvabilityVerification`
- **Enum**: `VerificationStatus`
- **Struct**: `QAVerification`
- **Function**: `calculate_complexity_entropy` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_deep_context_result` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_qa_verification_dead_code` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_qa_verification_complexity` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_qa_report_generation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/ranking.rs

**File Metrics**: Complexity: 85, Functions: 63

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `FileRanker`
- **Struct**: `RankingEngine`
- **Struct**: `CompositeComplexityScore`
- **Struct**: `ChurnScore`
- **Struct**: `DuplicationScore`
- **Function**: `rank_files_vectorized` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `ComplexityRanker`
- **Function**: `rank_files_by_complexity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `MockRanker`
- **Function**: `create_test_file_metrics` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_file_list` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_limit_exceeds_files` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vectorized_ranking` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_composite_complexity_score_ordering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_composite_complexity_score_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_composite_complexity_score_equality` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_churn_score_default_and_ordering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_duplication_score_default_and_ordering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vectorized_ranking_small_dataset` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vectorized_ranking_large_dataset` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vectorized_ranking_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_calculate_composite_score` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_calculate_composite_score_empty` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ranking_engine_with_temp_files` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_ranking_engine_zero_limit` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ranking_engine_cache` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ranking_engine_format_rankings_table_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ranking_engine_format_rankings_table` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ranking_engine_format_rankings_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_compute_score_rust_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_compute_score_javascript_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_compute_score_python_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_compute_score_unknown_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_compute_score_nonexistent_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_ranker_format_ranking_entry` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rank_files_by_complexity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rank_files_by_complexity_with_limit` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rank_files_by_complexity_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ranking_engine_with_nonexistent_files` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ranking_engine_mixed_existing_nonexistent` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Struct**: `TestRanker`
- **Function**: `test_custom_ranker` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_all_score_types_partial_ord` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/readme_compressor.rs

**File Metrics**: Complexity: 169, Functions: 21

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ReadmeCompressor`
- **Struct**: `Section`
- **Struct**: `List`
- **Use**: statement
- **Function**: `test_compress_basic_readme` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_section_scoring` [complexity: 16] [cognitive: 16] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `test_truncate_intelligently` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_project_description` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_markdown_parsing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_feature_extraction` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compress_section_with_budget` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/src/services/refactor_engine.rs

**File Metrics**: Complexity: 224, Functions: 23

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `UnifiedEngine`
- **Enum**: `EngineMode`
- **Enum**: `ExplainLevel`
- **Struct**: `RingBuffer`
- **Struct**: `EngineMetrics`
- **Enum**: `Command`
- **Struct**: `InteractiveState`
- **Struct**: `StateInfo`
- **Struct**: `MetricsInfo`
- **Struct**: `ComplexityInfo`
- **Struct**: `SuggestionInfo`
- **Struct**: `OperationInfo`
- **Struct**: `StepResult`
- **Enum**: `EngineError`
- **Function**: `test_refactor_engine_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/renderer.rs

**File Metrics**: Complexity: 4, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `TemplateRenderer`
- **Function**: `render_template` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_template_renderer_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_template_simple` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_template_with_current_timestamp` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_template_with_helpers` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_template_missing_variable` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_template_error` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_render_template_with_conditionals` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_template_preserves_original_context` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/rust_borrow_checker.rs

**File Metrics**: Complexity: 118, Functions: 32

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `RustBorrowChecker`
- **Struct**: `CollectionState`
- **Use**: statement
- **Use**: statement
- **Function**: `test_rust_borrow_checker_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_rust_borrow_checker_collect` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_safety_annotation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_thread_safety_annotation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/satd_detector.rs

**File Metrics**: Complexity: 326, Functions: 57

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SATDDetector`
- **Struct**: `TechnicalDebt`
- **Struct**: `SATDAnalysisResult`
- **Struct**: `SATDSummary`
- **Enum**: `DebtCategory`
- **Enum**: `Severity`
- **Struct**: `AstContext`
- **Enum**: `AstNodeType`
- **Struct**: `DebtClassifier`
- **Struct**: `DebtPattern`
- **Struct**: `DebtEvolution`
- **Struct**: `SATDMetrics`
- **Struct**: `CategoryMetrics`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_debt` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_debt_category_as_str` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_debt_category_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_severity_escalate` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_severity_reduce` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_debt_classifier_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_debt_classifier_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_pattern_classification` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_adjust_severity` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 18] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_satd_detector_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_satd_detector_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_comment_content` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_comment_column` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_hash_stability` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_from_content` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_from_content_skips_test_blocks` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_technical_debt_equality` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_satd_summary_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_satd_analysis_result_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_category_metrics` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_satd_metrics` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_debt_evolution` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_node_type_equality` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_test_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_source_files_excludes_common_dirs` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_source_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_directory` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_project` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_large_file_handling` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_extract_from_line_error_handling` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_metrics` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 18] [provability: 75%] [coverage: 65%]

### ./server/src/services/semantic_naming.rs

**File Metrics**: Complexity: 46, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SemanticNamer`
- **Use**: statement
- **Function**: `test_path_to_module_rust` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_path_to_module_python` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_semantic_name_priority` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_clean_id` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/simple_deep_context.rs

**File Metrics**: Complexity: 74, Functions: 11

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SimpleDeepContext`
- **Struct**: `SimpleAnalysisConfig`
- **Struct**: `SimpleAnalysisReport`
- **Struct**: `ComplexityMetrics`
- **Use**: statement
- **Struct**: `FileComplexityMetrics`
- **Function**: `test_simple_deep_context_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/symbol_table.rs

**File Metrics**: Complexity: 54, Functions: 25

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SymbolTable`
- **Struct**: `SymbolTableBuilder`
- **Use**: statement
- **Use**: statement
- **Function**: `test_symbol_table_insertion_and_lookup` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_relative_location_resolution` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_qualified_name_parsing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_symbol_table_builder` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/tdg_calculator.rs

**File Metrics**: Complexity: 209, Functions: 35

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ComplexityVariance`
- **Struct**: `CouplingMetrics`
- **Struct**: `TDGCalculator`
- **Use**: statement
- **Use**: statement
- **Function**: `test_tdg_calculation` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tdg_distribution` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tdg_variance` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/src/services/template_service.rs

**File Metrics**: Complexity: 78, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `get_template_content` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_template` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `generate_context` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `list_templates` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `list_all_resources` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `parse_template_uri` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `build_template_prefix` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `extract_filename` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `validate_parameters` [complexity: 10] [cognitive: 23] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `scaffold_project` [complexity: 14] [cognitive: 23] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `search_templates` [complexity: 11] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `validate_template` [complexity: 14] [cognitive: 30] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Struct**: `ScaffoldResult`
- **Struct**: `GeneratedFile`
- **Struct**: `ScaffoldError`
- **Struct**: `SearchResult`
- **Struct**: `ValidationResult`
- **Struct**: `ValidationError`
- **Function**: `test_template_service_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/unified_ast_engine.rs

**File Metrics**: Complexity: 340, Functions: 47

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `FileType`
- **Struct**: `UnifiedAstEngine`
- **Struct**: `LanguageParsers`
- **Struct**: `AstForest`
- **Enum**: `FileAst`
- **Struct**: `ModuleNode`
- **Struct**: `ModuleMetrics`
- **Struct**: `ProjectMetrics`
- **Struct**: `ArtifactTree`
- **Struct**: `MermaidArtifacts`
- **Struct**: `Template`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_deterministic_artifact_generation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_path_to_module_name` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_source_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/unified_ast_parser.rs

**File Metrics**: Complexity: 15, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ParseResult`
- **Struct**: `ParserCapabilities`
- **Struct**: `ParserConfig`
- **Trait**: `UnifiedAstParser`
- **Struct**: `AstParserRegistry`
- **Function**: `create_default_registry` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_parser_config_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_registry_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_default_registry` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/unified_refactor_analyzer.rs

**File Metrics**: Complexity: 52, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Trait**: `UnifiedAnalyzer`
- **Struct**: `RefactorPlan`
- **Struct**: `EstimatedImprovement`
- **Enum**: `RiskLevel`
- **Struct**: `AstDelta`
- **Struct**: `NodeModification`
- **Struct**: `MetricDelta`
- **Struct**: `NodeId`
- **Enum**: `Language`
- **Enum**: `AnalyzerError`
- **Struct**: `AnalyzerPool`
- **Struct**: `RustAnalyzer`
- **Function**: `test_unified_refactor_analyzer_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/verified_complexity.rs

**File Metrics**: Complexity: 69, Functions: 18

- **Use**: statement
- **Use**: statement
- **Struct**: `VerifiedComplexityAnalyzer`
- **Struct**: `ComplexityMetrics`
- **Struct**: `HalsteadMetrics`
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_simple_function_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cognitive_bounds` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/assemblyscript.rs

**File Metrics**: Complexity: 5, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `AssemblyScriptParser`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_assemblyscript_parser` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/assemblyscript_property_tests.rs

**File Metrics**: Complexity: 2, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `empty_file_handling` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `decorators_handled` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/binary.rs

**File Metrics**: Complexity: 40, Functions: 7

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `WasmAnalysis`
- **Struct**: `WasmSection`
- **Struct**: `WasmBinaryAnalyzer`
- **Function**: `count_occurrences` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_wasm_binary_analyzer` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_count_occurrences` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/binary_property_tests.rs

**File Metrics**: Complexity: 6, Functions: 5

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `wasm_magic_strategy` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `wasm_version_strategy` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `wasm_section_strategy` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `wasm_binary_strategy` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `invalid_wasm_strategy` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/complexity.rs

**File Metrics**: Complexity: 0, Functions: 7

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `MemoryCostModel`
- **Struct**: `WasmComplexityAnalyzer`
- **Use**: statement
- **Function**: `test_complexity_analyzer` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/complexity_property_tests.rs

**File Metrics**: Complexity: 0, Functions: 3

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_edge_cases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_simple_function_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_cost_model_values` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/coverage_tests.rs

**File Metrics**: Complexity: 23, Functions: 47

- **Use**: statement
- **Use**: statement
- **Function**: `test_wasm_error_parse` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_error_format` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_error_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_error_from_io_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_error_from_anyhow` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_result_type` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_memory_pool_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_pool_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_pool_custom_sizes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_security_validator_new` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_security_validator_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_valid_wasm` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_invalid_magic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_too_small` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_large_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_ast` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_text` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_security_issue_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_security_validation_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_security_categories` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_parsed_ast_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_analysis_capabilities_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_analysis_capabilities_custom` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_webassembly_variant` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_metrics_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_op_stats` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_analysis` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_allocation_pattern` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_optimization_hint` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_alignment_issue` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_source_location` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_severity_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_severity_ordering` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_difficulty_enum` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_optimization_type_enum` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_opcode_from_u8` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_opcode_all_variants` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_metrics_with_histogram` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_memory_cost_model_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_cost_model_custom` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_analyzer_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_ast` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_text_simple` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_text_with_functions` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_text_large` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/error.rs

**File Metrics**: Complexity: 0, Functions: 4

- **Use**: statement
- **Enum**: `WasmError`

### ./server/src/services/wasm/integration_tests.rs

**File Metrics**: Complexity: 9, Functions: 6

- **Use**: statement
- **Function**: `test_wasm_error_integration` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_memory_pool_integration` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_types_integration` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_traits_integration` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_security_integration` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_complexity_integration` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement

### ./server/src/services/wasm/language_detection.rs

**File Metrics**: Complexity: 20, Functions: 8

- **Struct**: `WasmLanguageDetector`
- **Use**: statement
- **Function**: `test_assemblyscript_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wat_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_binary_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/memory_pool.rs

**File Metrics**: Complexity: 0, Functions: 3

- **Struct**: `MemoryPool`

### ./server/src/services/wasm/mod.rs

**File Metrics**: Complexity: 0, Functions: 0

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement

### ./server/src/services/wasm/parallel.rs

**File Metrics**: Complexity: 16, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ParallelConfig`
- **Struct**: `FileAnalysisResult`
- **Struct**: `AggregatedAnalysis`
- **Struct**: `ParallelWasmAnalyzer`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_parallel_analyzer` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_relevance` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/security.rs

**File Metrics**: Complexity: 6, Functions: 5

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SecurityValidation`
- **Struct**: `SecurityIssue`
- **Enum**: `SecurityCategory`
- **Struct**: `WasmSecurityValidator`

### ./server/src/services/wasm/security_property_tests.rs

**File Metrics**: Complexity: 6, Functions: 2

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `edge_case_file_sizes` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `multiple_issues_reported` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/tests.rs

**File Metrics**: Complexity: 1, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_assemblyscript_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wat_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_binary_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_assemblyscript_parser` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wat_parser` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wasm_binary_analyzer` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_analyzer` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_security_validator` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parallel_analyzer` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_opcode_conversion` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_memory_cost_model` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_webassembly_variant_display` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_severity_ordering` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_difficulty_levels` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_optimization_types` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_security_category` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_wasm_analysis_capabilities` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/traits.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `ParsedAst`
- **Trait**: `LanguageParser`
- **Trait**: `WasmAwareParser`
- **Struct**: `WasmAnalysisCapabilities`

### ./server/src/services/wasm/types.rs

**File Metrics**: Complexity: 86, Functions: 2

- **Use**: statement
- **Use**: statement
- **Enum**: `WebAssemblyVariant`
- **Struct**: `WasmMetrics`
- **Struct**: `MemoryOpStats`
- **Struct**: `WasmComplexity`
- **Struct**: `MemoryAnalysis`
- **Struct**: `AllocationPattern`
- **Struct**: `MemoryOptimizationHint`
- **Struct**: `AlignmentIssue`
- **Struct**: `SourceLocation`
- **Enum**: `Severity`
- **Enum**: `Difficulty`
- **Enum**: `OptimizationType`
- **Enum**: `WasmOpcode`

### ./server/src/services/wasm/wat.rs

**File Metrics**: Complexity: 8, Functions: 5

- **Use**: statement
- **Use**: statement
- **Struct**: `WatParser`
- **Use**: statement
- **Function**: `test_wat_parser` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_wat_parser_invalid` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/services/wasm/wat_property_tests.rs

**File Metrics**: Complexity: 2, Functions: 1

- **Use**: statement
- **Use**: statement
- **Function**: `empty_module_handling` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/stateless_server.rs

**File Metrics**: Complexity: 16, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `StatelessTemplateServer`
- **Function**: `test_stateless_server_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/additional_coverage.rs

**File Metrics**: Complexity: 8, Functions: 3

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_churn_output_format` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_validate_params` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_additional_model_coverage` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement

### ./server/src/tests/analyze_cli_tests.rs

**File Metrics**: Complexity: 80, Functions: 7

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_analyze_churn_command_parsing` [complexity: 28] [cognitive: 49] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 65%]
- **Function**: `test_analyze_churn_with_all_options` [complexity: 28] [cognitive: 49] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 65%]
- **Function**: `test_analyze_churn_format_options` [complexity: 19] [cognitive: 43] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 44%]
- **Function**: `test_analyze_churn_invalid_format` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_churn_short_flags` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_analyze_subcommand_help` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_churn_help` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/ast_e2e.rs

**File Metrics**: Complexity: 10, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_analyze_python_file_comprehensive` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 4] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `test_python_class_field_count` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_python_import_parsing` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_python_file_not_found` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_python_invalid_syntax` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_analyze_typescript_file_comprehensive` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_javascript_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_typescript_class_field_count` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_tsx_file_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_jsx_file_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_typescript_file_not_found` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_typescript_invalid_syntax` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_mixed_language_project_context` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]

### ./server/src/tests/ast_regression_test.rs

**File Metrics**: Complexity: 7, Functions: 2

- **Use**: statement
- **Function**: `test_ast_analysis_not_empty_regression` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `test_deep_context_includes_ast_analysis` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement

### ./server/src/tests/binary_integration.rs

**File Metrics**: Complexity: 2, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `get_binary_path` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_help_flag` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_version_flag` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_invalid_command` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_list_templates` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_analyze_complexity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_mcp_mode` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_environment_variables` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_json_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_analyze_dag` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_generate_template` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_context_command` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_diagnose_command` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_error_handling` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_concurrent_execution` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/src/tests/binary_size.rs

**File Metrics**: Complexity: 13, Functions: 5

- **Use**: statement
- **Function**: `binary_size_regression` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `feature_size_impact` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `template_compression_works` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `startup_time_regression` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Use**: statement
- **Function**: `memory_usage_baseline` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/build_naming_validation.rs

**File Metrics**: Complexity: 17, Functions: 8

- **Use**: statement
- **Function**: `test_cargo_build_has_single_correct_binary` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_no_old_package_references` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_no_old_binary_references_in_workflows` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_correct_binary_name_in_workflows` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_no_wrong_repo_urls_in_workflows` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_workspace_aware_cargo_commands_in_makefile` [complexity: 11] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `test_cargo_lock_only_in_root` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_build_script_workspace_aware` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/cache.rs

**File Metrics**: Complexity: 29, Functions: 30

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_session_cache_manager` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_cache` [provability: 75%] [coverage: 65%]
- **Struct**: `TestAstCacheStrategy`
- **Function**: `test_template_cache` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_dag_cache` [provability: 75%] [coverage: 65%]
- **Struct**: `TestDagCacheStrategy`
- **Function**: `test_churn_cache` [provability: 75%] [coverage: 65%]
- **Struct**: `TestChurnCacheStrategy`
- **Function**: `test_git_stats_cache` [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Struct**: `TestGitStatsCacheStrategy`
- **Function**: `test_cache_eviction` [provability: 75%] [coverage: 65%]
- **Struct**: `SmallCacheStrategy`
- **Function**: `test_cache_clear` [provability: 75%] [coverage: 65%]
- **Struct**: `TestClearStrategy`
- **Function**: `test_cache_ttl` [provability: 75%] [coverage: 65%]
- **Struct**: `ShortTtlStrategy`

### ./server/src/tests/cache_comprehensive_tests.rs

**File Metrics**: Complexity: 10, Functions: 16

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_cache_config_default_values` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_config_custom_values` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_config_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_stats_snapshot_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_stats_snapshot_zero_requests` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_stats_snapshot_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_effectiveness_structure` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_effectiveness_serialization` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_diagnostics_structure` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_stats_hit_rate_calculation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_config_ttl_values` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_config_memory_settings` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_config_git_settings` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_config_warmup_patterns` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_effectiveness_empty_caches` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_diagnostics_empty_collections` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/churn.rs

**File Metrics**: Complexity: 5, Functions: 11

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_analysis` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_churn_score_calculation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_output_format_parsing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_churn_summary` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_format_churn_markdown` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_format_churn_csv` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_no_git_repository_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_commit_line` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_multiple_commits_and_files` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_churn_score_edge_cases` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_repository` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/clap_argument_parsing_tests.rs

**File Metrics**: Complexity: 87, Functions: 28

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_numeric_argument_coercion` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_path_argument_coercion` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_enum_argument_coercion` [complexity: 11] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `test_boolean_flag_coercion` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_optional_argument_coercion` [complexity: 11] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `test_vec_argument_coercion` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_numeric_range_validation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_enum_validation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_path_validation` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_mutually_exclusive_flags` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_required_argument_validation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_string_validation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_custom_type_parsing` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_default_value_application` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_value_delimiter_parsing` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_case_sensitivity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_unicode_arguments` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_argument_with_equals_sign` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_quoted_arguments` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_special_characters_in_arguments` [complexity: 8] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_overflow_values` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_argument_order_flexibility` [complexity: 8] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `test_unknown_argument_handling` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_typo_suggestions` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_help_flag_parsing` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_version_flag_parsing` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_subcommand_help` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_double_dash_separator` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/src/tests/clap_command_structure_tests.rs

**File Metrics**: Complexity: 21, Functions: 18

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_derive_parser_propagation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_name_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_global_args_accessible` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_subcommand_hierarchy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_propagate_version` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_help_generation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_support` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_command_aliases` [complexity: 8] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_required_args_validation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_global_flags_precedence` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_subcommand_specific_args` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_value_enum_parsing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_command_error_suggestions` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_all_commands_have_help` [provability: 75%] [coverage: 65%]
- **Function**: `check_command_help` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_all_args_have_help` [provability: 75%] [coverage: 65%]
- **Function**: `check_args_help` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_conflicting_args` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_help_output_format` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_output_format` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/clap_env_var_tests.rs

**File Metrics**: Complexity: 29, Functions: 21

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_rust_log_env_var` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_precedence` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_env_var` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_unset` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_with_special_characters` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_unicode` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_env_var_with_verbose_flags` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_multiple_env_vars` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_parsing_errors` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_explicit_none_vs_env_var` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_case_sensitivity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_whitespace_handling` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_with_equals_sign` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_very_long_env_var` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_with_newlines` [complexity: 6] [cognitive: 8] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_env_var_with_null_bytes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_concurrent_modification` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_env_var_help_text` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_env_var_in_error_messages` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_isolated_env_var` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_var_does_not_leak` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/claude_code_e2e.rs

**File Metrics**: Complexity: 54, Functions: 23

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `create_tool_request` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_claude_code_rust_cli_workflow` [complexity: 12] [cognitive: 17] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `test_claude_code_all_languages_scaffold` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `create_scaffold_test_cases` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_toolchain_scaffolding` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `create_scaffold_request` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `validate_scaffold_response` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `verify_generated_files` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Struct**: `GeneratedFileFlags`
- **Function**: `process_generated_file` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `verify_makefile` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `verify_makefile_toolchain_specific` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `verify_readme` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `verify_gitignore` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `verify_gitignore_patterns` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `test_claude_code_error_scenarios` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_claude_code_search_templates` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_naming_convention_critical_requirement` [complexity: 7] [cognitive: 8] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_naming_convention_in_individual_templates` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_server_info_naming_convention` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_context_generation` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./server/src/tests/cli_basic_tests.rs

**File Metrics**: Complexity: 8, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_dir_with_rust_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_generate_command_basic` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_list_command_basic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_search_command_basic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_analyze_complexity_basic` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_rust` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_empty` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_command_dispatcher_trait_bounds` [provability: 75%] [coverage: 65%]
- **Function**: `assert_send_sync` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_output_format_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_output_format_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling_invalid_generate` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_multiple_commands_sequential` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tempdir_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_server_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/cli_command_dispatcher_tests.rs

**File Metrics**: Complexity: 8, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_dir_with_rust_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_generate_command` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_scaffold_command` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_list_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_search_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_validate_command` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_context_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_analyze_complexity_command` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_analyze_dag_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_demo_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_refactor_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execute_diagnose_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_command_handler_trait_bounds` [provability: 75%] [coverage: 65%]
- **Function**: `assert_send_sync` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_command_handler_trait_bounds` [provability: 75%] [coverage: 65%]
- **Function**: `assert_send_sync` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling_invalid_path` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_concurrent_command_execution` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/cli_comprehensive_tests.rs

**File Metrics**: Complexity: 158, Functions: 31

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_generate_command_full_parsing` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_generate_command_aliases` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_missing_required_args` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_scaffold_command_parsing` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_scaffold_template_delimiter` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_scaffold_default_parallel` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_list_command_all_formats` [complexity: 17] [cognitive: 37] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 40%]
- **Function**: `test_list_command_filters` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_list_default_format` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_search_command_parsing` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_search_default_limit` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_validate_command_parsing` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_context_command_parsing` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_context_formats` [complexity: 8] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_context_default_values` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_analyze_churn_full_options` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_analyze_churn_all_formats` [complexity: 8] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_analyze_complexity_full_options` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_analyze_complexity_formats` [complexity: 8] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_analyze_dag_full_options` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_analyze_dag_types` [complexity: 8] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_parse_key_val_basic` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_key_val_edge_cases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_params_comprehensive` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_expand_env_vars_complex` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_error_scenarios` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_help_flags` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_version_flag` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mode_flag` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_multiple_parameter_types` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_nested_subcommand_parsing` [complexity: 15] [cognitive: 31] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]

### ./server/src/tests/cli_handlers_integration_tests.rs

**File Metrics**: Complexity: 17, Functions: 22

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_project` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_generate` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_scaffold` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_scaffold_parallel` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_list` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_list_with_toolchain` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_list_with_category` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_search` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_search_with_toolchain` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_validate` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_validate_invalid_uri` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_context` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_context_with_output` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_context_json_format` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_context_with_toolchain` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling_nonexistent_path` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_to_stdout` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_with_create_dirs` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_without_create_dirs` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_concurrent_handler_operations` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `test_error_propagation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/cli_integration_full.rs

**File Metrics**: Complexity: 6, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_cli_run_generate_to_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_generate_template_direct` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_cli_list_templates_direct` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_cli_search_templates_direct` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_cli_validate_template_direct` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_cli_scaffold_project_direct` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_cli_context_generation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_cli_churn_analysis` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement

### ./server/src/tests/cli_module_tests.rs

**File Metrics**: Complexity: 15, Functions: 24

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_project_with_languages` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_rust_heavy_project` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `create_python_heavy_project` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `create_deno_heavy_project` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_rust` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_python` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_deno` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_mixed` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_empty_directory` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_no_recognized_files` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_nonexistent_path` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_primary_language_jsx_files` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_satd_filters_no_filters` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_satd_filters_by_severity` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_satd_filters_critical_only` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_satd_filters_severity_and_critical` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_apply_satd_filters_empty_input` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_early_for_tracing_no_flags` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_early_for_tracing_verbose` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_early_for_tracing_debug` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_early_for_tracing_trace` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parse_early_for_tracing_with_filter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_satd_item_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_early_cli_args_debug_trait` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/cli_property_tests.rs

**File Metrics**: Complexity: 0, Functions: 0

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement

### ./server/src/tests/cli_simple_tests.rs

**File Metrics**: Complexity: 2, Functions: 5

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_validate_params_basic` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_expand_env_vars_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_output_format_enum` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_format_enum` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_commands_construction` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/cli_tests.rs

**File Metrics**: Complexity: 32, Functions: 25

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_validate_params_all_valid` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_params_missing_required` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_params_unknown_parameter` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_params_type_validation` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_expand_env_vars` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_expand_env_vars_no_vars` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_expand_env_vars_multiple_occurrences` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `create_test_server` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_command_to_stdout` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_command_to_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_command_json_format` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_scaffold_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_parse_key_val` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_value_type_inference` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_table_formatting` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_json_output_format` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_yaml_output_format` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_missing_required_params_error` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_multiple_validation_errors` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_parameter_parsing` [complexity: 5] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_template_uri_patterns` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_toolchain_names` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/code_smell_comprehensive_tests.rs

**File Metrics**: Complexity: 36, Functions: 26

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_cross_reference_tracking` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_entry_point_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dynamic_dispatch_resolution` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_hierarchical_bitset_optimization` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_confidence_scoring` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_coverage_integration` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_multi_language_comment_parsing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_contextual_classification` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_severity_scoring` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_complexity_integration` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_duplicate_detection_config` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_clone_type_definitions` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `test_detection_engine_instantiation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cross_language_support` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_formal_verification_components` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_state_invariant_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pure_function_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_provability_score` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `detect_state_invariants` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_function_purity` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `PurityAnalysis`
- **Use**: statement
- **Function**: `test_deep_context_config` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_analysis_component_availability` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_quality_scorecard_structure` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_large_codebase_performance` [complexity: 7] [cognitive: 8] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_memory_efficiency` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `get_memory_usage` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/complexity_distribution_verification.rs

**File Metrics**: Complexity: 21, Functions: 8

- **Use**: statement
- **Use**: statement
- **Struct**: `ComplexityDistributionConfig`
- **Function**: `verify_complexity_distribution` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `calculate_entropy` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_coefficient_of_variation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_entropy_calculation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_distribution_verification` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_coefficient_of_variation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_realistic_functions` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/src/tests/dead_code_verification.rs

**File Metrics**: Complexity: 3, Functions: 6

- **Use**: statement
- **Use**: statement
- **Function**: `verify_entry_point_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `verify_cross_language_references` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `verify_dead_code_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `verify_zero_dead_code_edge_cases` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `verify_closure_and_dynamic_dispatch` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_coverage_integration` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement

### ./server/src/tests/deep_context_simplified_tests.rs

**File Metrics**: Complexity: 20, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_deep_context_config_default_values` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_deep_context_analyzer_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_metadata_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_quality_scorecard_calculations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_defect_summary_aggregation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_prioritized_recommendations` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cross_language_references` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_provenance_tracking` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Function**: `test_analysis_type_equality` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_enum_variants_complete` [complexity: 14] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]

### ./server/src/tests/deep_context_tests.rs

**File Metrics**: Complexity: 22, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_deep_context_config_default_values` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_deep_context_analyzer_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_discovery_simple_project` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_discovery_with_excludes` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_metadata_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_quality_scorecard_calculations` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_defect_summary_aggregation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_defect_hotspot_ranking` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_prioritized_recommendations` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cross_language_references` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_provenance_tracking` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `test_analysis_type_equality` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_enum_variants_complete` [complexity: 15] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]

### ./server/src/tests/demo_comprehensive_tests.rs

**File Metrics**: Complexity: 36, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_demo_runner_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_demo_step_structure` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_demo_report_structure` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_repository_git_repo` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_repository_cargo_project` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_repository_nodejs_project` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_repository_python_project` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_repository_pyproject_toml` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_repository_with_readme` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_repository_empty_directory` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_detect_repository_nonexistent_path` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_demo_report_rendering_cli` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `test_demo_report_rendering_mcp` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Function**: `test_demo_step_error_handling` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_demo_report_with_multiple_steps` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_demo_step_serialization` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_demo_report_serialization` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/diagnose_tests.rs

**File Metrics**: Complexity: 10, Functions: 8

- **Use**: statement
- **Function**: `test_diagnostic_format_enum` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_diagnose_args_defaults` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_feature_result_variants` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_diagnostic_summary` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_build_info_serialization` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compact_error_context` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_feature_status_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_diagnose_args_with_filters` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/e2e_full_coverage.rs

**File Metrics**: Complexity: 25, Functions: 9

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_mcp_server_e2e_coverage` [complexity: 25] [cognitive: 25] [big-o: O(n²)] [SATD: 8] [provability: 75%] [coverage: 65%] [defect-prob: 58%]
- **Function**: `test_cli_main_binary_version` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 8] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_main_binary_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 8] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_subcommand_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 8] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_mode_list_templates` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 8] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_generate_validation_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 8] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_search_templates` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 8] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_invalid_command` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 8] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_analyze_churn` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 8] [provability: 75%] [coverage: 65%]

### ./server/src/tests/error.rs

**File Metrics**: Complexity: 2, Functions: 10

- **Use**: statement
- **Use**: statement
- **Function**: `test_template_not_found_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_invalid_uri_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validation_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_not_found_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_s3_error` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_invalid_utf8_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_error` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_io_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/error_handling.rs

**File Metrics**: Complexity: 7, Functions: 15

- **Use**: statement
- **Use**: statement
- **Function**: `test_template_error_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_not_found_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_not_found_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validation_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_invalid_utf8_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_to_mcp_code` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parameter_spec_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parameter_spec_with_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_debug_representation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_multiple_error_types` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_error_from_anyhow` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_error_conversion` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_io_error_conversion` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_s3_error` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/fixtures/metric_tests/complex.rs

**File Metrics**: Complexity: 33, Functions: 3

- **Function**: `complex_function` [complexity: 10] [cognitive: 25] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `recursive_fibonacci` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `deeply_nested_conditionals` [complexity: 19] [cognitive: 48] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 44%]

### ./server/src/tests/fixtures/metric_tests/ffi_export.rs

**File Metrics**: Complexity: 3, Functions: 6

- **Function**: `exported_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `process_data` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `renamed_export` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `wasm_function` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `python_export` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `internal_helper` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/fixtures/metric_tests/simple.rs

**File Metrics**: Complexity: 0, Functions: 3

- **Function**: `simple_function` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `another_simple` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `basic_arithmetic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/fixtures/sample.js

**File Metrics**: Complexity: 0, Functions: 0


### ./server/src/tests/fixtures/sample.py

**File Metrics**: Complexity: 2, Functions: 3

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `User`
- **Function**: `get_display_name` [provability: 75%] [coverage: 65%]
- **Function**: `_internal_method` [provability: 75%] [coverage: 65%]
- **Struct**: `UserService`
- **Function**: `__init__` [provability: 75%] [coverage: 65%]
- **Function**: `get_user` [provability: 75%] [coverage: 65%]
- **Function**: `create_user` [provability: 75%] [coverage: 65%]
- **Function**: `list_users` [provability: 75%] [coverage: 65%]
- **Function**: `process_data` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `fetch_remote_data` [complexity: 0] [cognitive: 0] [big-o: O(?)] [provability: 75%] [coverage: 65%]
- **Function**: `_private_helper` [complexity: 0] [cognitive: 0] [big-o: O(?)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/fixtures/sample.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./server/src/tests/gitignore_respect_tests.rs

**File Metrics**: Complexity: 0, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_file_discovery_respects_gitignore` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_discovery_without_gitignore_respect` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_c_file_discovery_respects_gitignore` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_custom_paimlignore_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/helpers.rs

**File Metrics**: Complexity: 17, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_snake_case_helper` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_kebab_case_helper` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pascal_case_helper` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_current_year_helper` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_current_date_helper` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_helper_error_handling` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_string_handling` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_helper_with_non_string_parameter` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pascal_case_preserves_existing_capitalization` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_year_and_date_helpers_consistency` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/http_adapter_tests.rs

**File Metrics**: Complexity: 13, Functions: 11

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_http_adapter_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_adapter_bind` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_output_creation` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_http_context_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_context_with_no_remote_addr` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_context_with_no_user_agent` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_context_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_adapter_trait` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_status_code_variations` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_http_response_with_json` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_http_error_responses` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/src/tests/kaizen_reliability_patterns.rs

**File Metrics**: Complexity: 41, Functions: 22

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `kaizen_retry` [complexity: 8] [cognitive: 12] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `poka_yoke_timeout` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Struct**: `JidokaTestSetup`
- **Struct**: `TestStateInspector`
- **Function**: `fast_assert_eq` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `should_skip_expensive_operation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `create_minimal_test_data` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_kaizen_retry_succeeds_first_attempt` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_kaizen_retry_succeeds_after_retries` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_poka_yoke_timeout_success` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_poka_yoke_timeout_failure` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_jidoka_test_setup_cleanup` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_test_state_inspector` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_muda_elimination_fast_assert` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_should_skip_expensive_operation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/src/tests/kaizen_test_optimizations.rs

**File Metrics**: Complexity: 45, Functions: 19

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `TestMetrics`
- **Struct**: `SlowTest`
- **Struct**: `FlakyTest`
- **Enum**: `TestCategory`
- **Struct**: `KaizenTestRunner`
- **Function**: `fast_unit_test_setup` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `fast_temp_dir` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_test_data` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `MockHeavyOperation`
- **Use**: statement
- **Function**: `fast_proptest_config` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `small_string_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `small_vec_strategy` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_kaizen_runner_tracks_metrics` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_fast_temp_dir_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mock_heavy_operation_performance` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_concurrent_test_execution` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement

### ./server/src/tests/lib.rs

**File Metrics**: Complexity: 0, Functions: 12

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_template_server_new` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_server_trait_implementation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_server_deprecated_methods` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_warm_cache` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_server_cache_initialization` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_server_cache_sizes` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_warm_cache_templates` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_server_trait_via_methods` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_type_aliases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_s3_client_instantiation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_run_mcp_server_basic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_public_exports` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/src/tests/mcp_protocol.rs

**File Metrics**: Complexity: 7, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_request` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_initialize` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_list_tools` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_list_resources` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_call_tool_generate_template` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_call_tool_invalid_tool` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_call_tool_missing_parameters` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_invalid_method` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_version_default` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/metric_accuracy_suite.rs

**File Metrics**: Complexity: 3, Functions: 5

- **Use**: statement
- **Function**: `calculate_variance` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tdg_variance` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cognitive_bounds` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ffi_not_dead` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_complexity_detection` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/models.rs

**File Metrics**: Complexity: 2, Functions: 4

- **Use**: statement
- **Function**: `test_toolchain_priority` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_toolchain_as_str` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_category_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_parameter_type_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/src/tests/project_meta_integration_test.rs

**File Metrics**: Complexity: 4, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_full_metadata_compression_pipeline` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_metadata_integration` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_project` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_minimal_project` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/prompts.rs

**File Metrics**: Complexity: 6, Functions: 9

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_request` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_prompts_list` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_prompt_get_rust_project` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_prompt_get_deno_project` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_prompt_get_python_project` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_prompt_get_missing_params` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_prompt_get_invalid_params` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_prompt_get_unknown_prompt` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/protocol_service_tests.rs

**File Metrics**: Complexity: 1, Functions: 11

- **Use**: statement
- **Use**: statement
- **Function**: `test_app_state_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_adapter_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_app_state_clone` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_adapter_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_adapters_are_send_sync` [provability: 75%] [coverage: 65%]
- **Function**: `assert_send_sync` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_adapter_trait_bounds` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_app_state_reference_counting` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_adapter_memory_footprint` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_app_state_creation_performance` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_multiple_app_states` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_adapter_creation_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/resources.rs

**File Metrics**: Complexity: 8, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_request` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_resource_list` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_resource_read_success` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_resource_read_missing_params` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_resource_read_invalid_params` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_resource_read_not_found` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_resource_read_all_templates` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./server/src/tests/template_rendering.rs

**File Metrics**: Complexity: 9, Functions: 9

- **Use**: statement
- **Use**: statement
- **Function**: `test_render_rust_cli_makefile` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_python_uv_makefile` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_deno_typescript_makefile` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_readme_template` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_gitignore_template` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_with_conditionals` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_with_missing_parameters` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_with_nested_loops` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_render_with_string_helpers` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/template_resources.rs

**File Metrics**: Complexity: 10, Functions: 10

- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_all_templates` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_filter_templates_by_prefix` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_template_metadata` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_template_content` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_invalid_template_uri` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_categories` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_template_toolchains` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_parameter_types` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_rust_template_parameters` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/tools.rs

**File Metrics**: Complexity: 20, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_server` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_request` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_tool_call_missing_params` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_tool_call_invalid_params` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_templates_all` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_templates_by_toolchain` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_templates_by_category` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_template_valid` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_template_missing_required` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_template_unknown_parameter` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_template_not_found` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_scaffold_project_rust` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_scaffold_project_deno` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_templates_by_name` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_templates_with_toolchain_filter` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_templates_no_results` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_template_invalid_arguments` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/tests/unified_protocol_tests.rs

**File Metrics**: Complexity: 16, Functions: 13

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_unified_service_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_app_state_default` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_service_metrics_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_service_metrics_increment` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_service_metrics_duration_tracking` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_service_metrics_error_tracking` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_context_http_only` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_context_mcp_only` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_app_error_types` [complexity: 16] [cognitive: 16] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `test_app_error_status_codes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_error_codes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_types` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_to_protocol_response` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/adapters/cli.rs

**File Metrics**: Complexity: 426, Functions: 63

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `CliAdapter`
- **Use**: statement
- **Struct**: `CliInput`
- **Enum**: `CliOutput`
- **Function**: `format_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `churn_format_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `complexity_format_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `dag_type_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `dead_code_format_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `satd_format_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `satd_severity_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `graph_metric_type_to_string` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `graph_metrics_format_to_string` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `name_similarity_format_to_string` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `property_type_filter_to_string` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `verification_method_filter_to_string` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `proof_annotation_format_to_string` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `incremental_coverage_format_to_string` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `symbol_type_filter_to_string` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `symbol_table_format_to_string` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `big_o_format_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Struct**: `CliRunner`
- **Function**: `deep_context_format_to_string` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `deep_context_dag_type_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `deep_context_cache_strategy_to_string` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `tdg_format_to_string` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `provability_format_to_string` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_cli_input_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_adapter_decode_generate` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_adapter_decode_list` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_adapter_decode_analyze_complexity` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_adapter_encode_success` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_cli_adapter_encode_error` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_format_conversions` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_output_methods` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/adapters/http.rs

**File Metrics**: Complexity: 56, Functions: 31

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `HttpAdapter`
- **Struct**: `HttpStreamAdapter`
- **Enum**: `HttpInput`
- **Enum**: `HttpOutput`
- **Struct**: `HttpServer`
- **Trait**: `HttpServiceHandler`
- **Function**: `handle_connection` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `process_http_request` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `convert_hyper_to_http_input` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `collect_request_body` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `decode_http_input` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_unified_request` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `encode_unified_response` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `serve_http_connection` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `HttpResponseBuilder`
- **Use**: statement
- **Use**: statement
- **Function**: `test_http_adapter_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_response_builder` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_adapter_encode` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_http_context` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/adapters/mcp.rs

**File Metrics**: Complexity: 63, Functions: 25

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `McpAdapter`
- **Enum**: `McpInput`
- **Struct**: `JsonRpcRequest`
- **Struct**: `JsonRpcResponse`
- **Struct**: `JsonRpcError`
- **Struct**: `McpReader`
- **Use**: statement
- **Use**: statement
- **Function**: `test_json_rpc_request_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_rpc_notification` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_rpc_response_success` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_rpc_response_error` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_adapter_decode` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_adapter_encode_success` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_standard_json_rpc_errors` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/adapters/mod.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_mod_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/adapters/tests.rs

**File Metrics**: Complexity: 6, Functions: 8

- **Use**: statement
- **Function**: `test_adapter_module_basics` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_cli_request_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_response_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_http_request_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_response_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_mcp_request_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_response_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_error_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/error.rs

**File Metrics**: Complexity: 98, Functions: 16

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `AppError`
- **Struct**: `McpError`
- **Struct**: `HttpErrorResponse`
- **Struct**: `CliErrorResponse`
- **Function**: `extract_protocol_from_context` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `set_protocol_context` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `clear_protocol_context` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_app_error_status_codes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_error_codes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_types` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_context` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_to_protocol_response` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/error_tests.rs

**File Metrics**: Complexity: 3, Functions: 6

- **Use**: statement
- **Function**: `test_unified_error_variants` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_error_from_io_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_error_from_serde_json_error` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_error_display` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_error_is_retryable` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_error_status_code` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/src/unified_protocol/mod.rs

**File Metrics**: Complexity: 33, Functions: 22

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `UnifiedRequest`
- **Struct**: `UnifiedResponse`
- **Enum**: `Protocol`
- **Trait**: `ProtocolAdapter`
- **Struct**: `AdapterRegistry`
- **Struct**: `AdapterWrapper`
- **Enum**: `ProtocolError`
- **Struct**: `McpContext`
- **Struct**: `CliContext`
- **Struct**: `HttpContext`
- **Use**: statement
- **Function**: `test_unified_request_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_response_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_display` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/service.rs

**File Metrics**: Complexity: 122, Functions: 35

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `UnifiedService`
- **Struct**: `AppState`
- **Struct**: `ServiceMetrics`
- **Trait**: `TemplateService`
- **Trait**: `AnalysisService`
- **Struct**: `DefaultTemplateService`
- **Struct**: `DefaultAnalysisService`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `list_templates` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `get_template` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_template` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_complexity` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_complexity_get` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_churn` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_dag` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_context` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_dead_code` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `analyze_deep_context` [complexity: 19] [cognitive: 27] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 44%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_makefile_lint` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Use**: statement
- **Function**: `analyze_provability` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Function**: `mcp_endpoint` [complexity: 21] [cognitive: 21] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Function**: `health_check` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `metrics` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `ListTemplatesQuery`
- **Struct**: `TemplateList`
- **Struct**: `TemplateInfo`
- **Struct**: `TemplateParameter`
- **Struct**: `GenerateParams`
- **Struct**: `GeneratedTemplate`
- **Struct**: `TemplateMetadata`
- **Struct**: `ComplexityParams`
- **Struct**: `ComplexityQueryParams`
- **Struct**: `ComplexityAnalysis`
- **Struct**: `ComplexitySummary`
- **Struct**: `FileComplexity`
- **Struct**: `FunctionComplexity`
- **Struct**: `ChurnParams`
- **Struct**: `ChurnAnalysis`
- **Struct**: `ChurnSummary`
- **Struct**: `ChurnHotspot`
- **Struct**: `DagParams`
- **Struct**: `DagAnalysis`
- **Struct**: `ContextParams`
- **Struct**: `ProjectContext`
- **Struct**: `ProjectStructure`
- **Struct**: `ContextMetrics`
- **Struct**: `DeadCodeParams`
- **Struct**: `DeadCodeAnalysis`
- **Struct**: `DeadCodeSummary`
- **Struct**: `FileDeadCode`
- **Struct**: `MakefileLintParams`
- **Struct**: `MakefileLintAnalysis`
- **Struct**: `MakefileLintViolation`
- **Struct**: `ProvabilityParams`
- **Struct**: `ProvabilityAnalysis`
- **Struct**: `ProvabilitySummary`
- **Use**: statement
- **Function**: `test_unified_service_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_default_template_service` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_generation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/service_tests.rs

**File Metrics**: Complexity: 20, Functions: 7

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `MockHandler`
- **Function**: `test_unified_service_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_process_json_request` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_process_error_response` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_process_invalid_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_process_with_trace_id` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_request_handler_trait` [provability: 75%] [coverage: 65%]
- **Struct**: `TestHandler`

### ./server/src/unified_protocol/test_harness.rs

**File Metrics**: Complexity: 104, Functions: 20

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `TestHarness`
- **Struct**: `TestResults`
- **Struct**: `EquivalenceFailure`
- **Struct**: `TestSuiteResults`
- **Enum**: `TestError`
- **Use**: statement
- **Function**: `test_harness_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_generation_endpoint` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_error_handling_consistency` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_protocol_equivalence` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_suite_results` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/unified_protocol/tests.rs

**File Metrics**: Complexity: 11, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_unified_error_variants` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_request_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_unified_response_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_source_variants` [complexity: 10] [cognitive: 16] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `test_request_context_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_response_metadata_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/utils/helpers.rs

**File Metrics**: Complexity: 25, Functions: 18

- **Use**: statement
- **Function**: `snake_case_helper` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `kebab_case_helper` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `pascal_case_helper` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `current_year_helper` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `current_date_helper` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `to_snake_case` [complexity: 5] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `to_kebab_case` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `to_pascal_case` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_to_snake_case_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_to_snake_case_edge_cases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_to_kebab_case_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_to_pascal_case_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_snake_case_helper_with_handlebars` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_kebab_case_helper_with_handlebars` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pascal_case_helper_with_handlebars` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_current_year_helper_with_handlebars` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_current_date_helper_with_handlebars` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_helper_error_cases` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/src/utils/helpers_tests.rs

**File Metrics**: Complexity: 2, Functions: 8

- **Use**: statement
- **Use**: statement
- **Function**: `test_ensure_directory_exists` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_bytes` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_truncate_string` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_sanitize_filename` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_is_binary_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_get_file_extension` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_normalize_path` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_timeout_future` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement

### ./server/src/utils/mod.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Function**: `test_mod_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/templates/gitignore/deno/cli.json


### ./server/templates/gitignore/python-uv/cli.json


### ./server/templates/gitignore/rust/cli.json


### ./server/templates/makefile/deno/cli.json


### ./server/templates/makefile/python-uv/cli.json


### ./server/templates/makefile/rust/cli.json


### ./server/templates/readme/deno/cli.json


### ./server/templates/readme/python-uv/cli.json


### ./server/templates/readme/rust/cli.json


### ./server/test_enum.kt

- **Enum**: `Status`

### ./server/tests/ast_dag_mermaid_pipeline.rs

**File Metrics**: Complexity: 7, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_rust_project` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_to_dag_metadata_propagation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_pipeline_determinism` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_pipeline_with_complex_project` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_edge_budget_enforcement` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_individual_file_analysis` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 5] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_mermaid_output_quality` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]
- **Function**: `test_metadata_serialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 5] [provability: 75%] [coverage: 65%]

### ./server/tests/bin/pmat_tests.rs

**File Metrics**: Complexity: 0, Functions: 13

- **Use**: statement
- **Use**: statement
- **Function**: `test_pmat_version` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_analyze_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_generate_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_demo_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_context_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_refactor_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_validate_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_mcp_protocol` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_http_protocol` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_invalid_command` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_verbose_flag` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pmat_multiple_verbose` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/bin_integration.rs

**File Metrics**: Complexity: 1, Functions: 4

- **Use**: statement
- **Use**: statement
- **Function**: `test_binary_version_flag` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_json_rpc_initialize` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_invalid_json` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_binary_multiple_requests` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/cli_comprehensive_integration.rs

**File Metrics**: Complexity: 14, Functions: 29

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_generate_makefile_e2e` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_missing_required_params` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_invalid_template_uri` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_to_stdout` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_scaffold_parallel_generation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_json_output_schema` [complexity: 4] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_table_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_yaml_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_filtered_by_toolchain` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_list_filtered_by_category` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_with_limit` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_with_toolchain_filter` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_success` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_validate_missing_required` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_generation_rust` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_markdown_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_churn_json_output` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_churn_csv_output` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_complexity_summary` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_complexity_sarif_format` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_dag_mermaid_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_propagation_and_codes` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Struct**: `ErrorCase`
- **Function**: `test_help_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_version_output` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_subcommand_help` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_analyze_subcommand_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_environment_variable_expansion` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_mode_flag_cli` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/tests/cli_context_tests.rs

**File Metrics**: Complexity: 2, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_context_skips_large_files_by_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_includes_large_files_with_flag` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_progress_bars` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_context_help_shows_include_large_files` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/cli_documentation_sync.rs

**File Metrics**: Complexity: 57, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DocumentedCommand`
- **Function**: `parse_documented_cli_commands` [complexity: 14] [cognitive: 24] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `parse_cli_help_output` [complexity: 8] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `get_binary_path` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_commands_match_documentation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_subcommands_match_documentation` [complexity: 5] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_cli_options_match_documentation` [complexity: 8] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_no_undocumented_commands` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_documentation_examples_are_valid` [complexity: 19] [cognitive: 31] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 44%]

### ./server/tests/complexity_metrics.rs

**File Metrics**: Complexity: 13, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_complexity_metrics_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_metrics_default` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_function_complexity_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_class_complexity_creation` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_complexity_metrics_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_compute_complexity_cache_key` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_empty` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_aggregate_results_with_data` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_complexity_summary` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_complexity_report` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_format_as_sarif` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_stateless_template_server_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_stateless_template_server_basic_operations` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_various_helper_functions` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_error_handling_coverage` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/tests/config_integration.rs

**File Metrics**: Complexity: 15, Functions: 5

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_config_loading_from_file` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_config_hot_reload` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `test_config_default_values` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_config_accessor_methods` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_invalid_config_file` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/demo_core_extraction.rs

**File Metrics**: Complexity: 5, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_config_manager_creation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_config_manager_custom_load` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_export_formats_discovery` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_export_service_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/tests/demo_e2e_integration.rs

**File Metrics**: Complexity: 103, Functions: 14

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DemoServer`
- **Use**: statement
- **Function**: `create_test_repository` [complexity: 12] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `test_demo_server_happy_path` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_api_contract_compliance` [complexity: 16] [cognitive: 16] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `test_concurrent_requests` [complexity: 15] [cognitive: 19] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `test_performance_assertions` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_error_handling` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_analysis_pipeline_integrity` [complexity: 11] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Use**: statement
- **Function**: `test_data_source_indicators` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_mermaid_diagram_rendering` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]

### ./server/tests/demo_integration.rs

**File Metrics**: Complexity: 30, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_demo_mode_in_test_directory` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_demo_mode_with_json_output` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_demo_mode_with_specific_path` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_demo_increases_test_coverage` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_demo_runner_execution` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_repository_detection` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/demo_web_integration.rs

**File Metrics**: Complexity: 38, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_demo_server_startup_and_shutdown` [complexity: 8] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_demo_server_api_endpoints` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `test_demo_server_static_assets` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_demo_server_concurrent_requests` [complexity: 14] [cognitive: 18] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 33%]
- **Function**: `test_demo_server_response_headers` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_demo_content_rendering` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_demo_server_starts` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_demo_content_from_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/determinism_tests.rs

**File Metrics**: Complexity: 35, Functions: 17

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_unified_ast_engine_determinism` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mermaid_generation_determinism` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dogfooding_engine_determinism` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_artifact_writer_determinism` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_pagerank_numerical_stability` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_hash_collision_resistance` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_file_ordering_stability` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_edge_case_determinism` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_concurrent_generation_determinism` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_project` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_dependency_graph` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_large_test_graph` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_single_node_graph` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `create_cyclic_graph` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_artifact_tree` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `compute_tree_hash` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `normalize_manifest` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/documentation_examples.rs

**File Metrics**: Complexity: 56, Functions: 20

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `get_binary_path` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_examples_are_valid` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `process_bash_code_block` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `should_skip_line` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `has_complex_shell_features` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `is_non_toolkit_command` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `handle_multiline_command` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `validate_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `validate_binary_path` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `validate_command_arguments` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_json_examples_are_valid` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `validate_json_block` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `validate_parsed_json` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `validate_json_rpc_object` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `validate_json_array_fallback` [complexity: 5] [cognitive: 5] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `validate_batch_request_array` [complexity: 2] [cognitive: 2] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_yaml_examples_are_valid` [complexity: 5] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_jsonc_examples_are_valid` [complexity: 9] [cognitive: 17] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `test_template_uri_examples_are_valid` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_performance_numbers_are_reasonable` [complexity: 4] [cognitive: 4] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/tests/e2e/installation.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./server/tests/e2e/mcp_protocol.test.ts

**File Metrics**: Complexity: 0, Functions: 0


### ./server/tests/e2e/system.rs

**File Metrics**: Complexity: 3, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_binary_exists` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_help_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_version_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_simple_file_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_project_structure_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_parsing` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/tests/enhanced_dag_integration.rs

**File Metrics**: Complexity: 0, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_enhanced_dag_analysis` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_enhanced_analysis_backward_compatibility` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_enhanced_flags_combinations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]
- **Function**: `test_enhanced_dag_help` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 4] [provability: 75%] [coverage: 65%]

### ./server/tests/execution_mode.rs

**File Metrics**: Complexity: 7, Functions: 11

- **Use**: statement
- **Use**: statement
- **Function**: `detect_execution_mode_test` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execution_mode_detection_with_mcp_version` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execution_mode_detection_without_mcp_version` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_env_filter_creation` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_server_creation_logic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Function**: `test_mcp_version_environment_variable` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_argument_count_behavior` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_async_runtime_setup` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tracing_initialization` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_terminal_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling_setup` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/tests/export_integration.rs

**File Metrics**: Complexity: 21, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_export_report` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_churn_analysis` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_markdown_export` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_export_pretty` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_json_export_compact` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_sarif_export` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_export_service` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_export_service_save_to_file` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_create_export_report` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_export_without_optional_data` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/fixtures/test_artifacts.rs

**File Metrics**: Complexity: 16, Functions: 4

- **Use**: statement
- **Use**: statement
- **Function**: `create_test_artifacts` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `create_rust_artifacts` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `create_python_artifacts` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `create_typescript_artifacts` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/tests/generate_mermaid_example.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `generate_example_mermaid_diagram` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/generate_mermaid_test.rs

**File Metrics**: Complexity: 0, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `generate_test_mermaid` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/git_clone_validation.rs

**File Metrics**: Complexity: 41, Functions: 10

- **Use**: statement
- **Use**: statement
- **Function**: `test_github_url_parsing_comprehensive` [complexity: 6] [cognitive: 8] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_github_url_parsing_invalid` [complexity: 7] [cognitive: 10] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_cache_key_generation` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_cache_key_uniqueness` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_url_normalization` [complexity: 9] [cognitive: 16] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `test_security_boundaries` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_fuzzer_identified_security_issues` [complexity: 3] [cognitive: 3] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]
- **Function**: `test_edge_case_handling` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_clone_timeout` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_round_trip_parsing` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/tests/integration/protocols.rs

**File Metrics**: Complexity: 9, Functions: 6

- **Use**: statement
- **Use**: statement
- **Function**: `test_json_rpc_format` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_http_request_format` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_argument_parsing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_protocol_response_formats` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_response_formats` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_parameter_normalization` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/integration/services.rs

**File Metrics**: Complexity: 43, Functions: 22

- **Use**: statement
- **Use**: statement
- **Function**: `test_file_discovery_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_discovery_with_ignore_patterns` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_project_metadata_detection` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_classifier` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_template_service_basic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_git_analysis_basic` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_test_file` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Struct**: `FileDiscovery`
- **Use**: statement
- **Enum**: `ProjectType`
- **Struct**: `ProjectMetadata`
- **Struct**: `ProjectMetaDetector`
- **Enum**: `FileType`
- **Struct**: `FileClassifier`
- **Struct**: `TemplateService`
- **Use**: statement
- **Struct**: `GitAnalyzer`

### ./server/tests/kotlin_ast_test.rs

**File Metrics**: Complexity: 4, Functions: 1

- **Use**: statement
- **Function**: `test_kotlin_parsing` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./server/tests/kotlin_support_test.rs

**File Metrics**: Complexity: 11, Functions: 4

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_kotlin_class_parsing` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_kotlin_interface_parsing` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_kotlin_data_class_parsing` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_kotlin_extension_support` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/mcp_documentation_sync.rs

**File Metrics**: Complexity: 50, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `DocumentedTool`
- **Struct**: `McpResponse`
- **Struct**: `ToolDefinition`
- **Function**: `parse_documented_mcp_tools` [complexity: 10] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `get_binary_path` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `send_mcp_request` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `test_mcp_tools_match_documentation` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_tool_schemas_match_documentation` [complexity: 10] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `test_mcp_methods_match_documentation` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_mcp_error_codes_are_complete` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_no_undocumented_mcp_tools` [complexity: 6] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]

### ./server/tests/mermaid_artifact_tests.rs

**File Metrics**: Complexity: 98, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `ArtifactCategory`
- **Struct**: `MermaidArtifactSpec`
- **Function**: `get_artifact_specs` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_simple_architecture` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_styled_workflow` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_ast_simple` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_ast_styled` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `validate_simple_diagram` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `validate_styled_diagram` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `validate_ast_diagram` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `validate_complexity_styled` [complexity: 8] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_generate_all_artifacts` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_maintain_mermaid_readme` [complexity: 39] [cognitive: 94] [big-o: O(?)] [provability: 75%] [coverage: 65%] [defect-prob: 70%]
- **Use**: statement
- **Function**: `format_category_title` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Struct**: `DiagramMetrics`
- **Function**: `analyze_diagram_metrics` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `calculate_graph_depth` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/mermaid_empty_bug_fix_test.rs

**File Metrics**: Complexity: 14, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_regression_empty_nodes_bug` [complexity: 6] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_mermaid_label_escaping` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_node_types_have_labels` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_complexity_styled_diagram_has_labels` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_empty_graph_doesnt_crash` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_special_characters_in_node_ids` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./server/tests/performance/regression.rs

**File Metrics**: Complexity: 4, Functions: 5

- **Use**: statement
- **Function**: `test_string_performance` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vector_performance` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_hashmap_performance` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_sorting_performance` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_iteration_performance` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/refactor_auto_property_integration.rs

**File Metrics**: Complexity: 0, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_project` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_refactor_auto_generates_property_tests` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_property_test_generation_in_request` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generated_property_test_template` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_coverage_improvement_tracking` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_property_test_shrinking` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/services_integration.rs

**File Metrics**: Complexity: 14, Functions: 9

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_execution_mode_detection` [complexity: 3] [cognitive: 3] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_cli_run_generate_command` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_rust_analysis` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_complexity_service` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_dag_builder` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_mcp_handlers` [complexity: 10] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_binary_main_logic` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_cli_functions` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_ast_error_handling` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./server/tests/slow_integration.rs

**File Metrics**: Complexity: 33, Functions: 6

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_demo_runner_as_library` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_export_service_as_library` [complexity: 11] [cognitive: 11] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `test_programmatic_demo_with_custom_config` [complexity: 7] [cognitive: 7] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_end_to_end_library_usage` [complexity: 13] [cognitive: 13] [big-o: O(n log n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `test_config_manager_as_library` [complexity: 6] [cognitive: 6] [big-o: O(n)] [SATD: 1] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_export_formats_discovery` [complexity: 1] [cognitive: 1] [big-o: O(1)] [SATD: 1] [provability: 75%] [coverage: 65%]

### ./server/tests/stateless_server_test.rs

**File Metrics**: Complexity: 0, Functions: 3

- **Function**: `test_basic_functionality` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_edge_cases` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./server/tests/test_kotlin_direct.rs

**File Metrics**: Complexity: 7, Functions: 1

- **Function**: `test_kotlin_parser_directly` [complexity: 9] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement

### ./server/tests/test_kotlin_minimal.rs

**File Metrics**: Complexity: 3, Functions: 1

- **Use**: statement
- **Function**: `test_minimal_kotlin_parsing` [complexity: 4] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./server/tests/unit/core.rs

**File Metrics**: Complexity: 6, Functions: 15

- **Function**: `test_basic_arithmetic` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_string_operations` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_vector_operations` [complexity: 2] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_option_handling` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_result_handling` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_hashmap_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_btreemap_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_iterator_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pattern_matching` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_closure_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_string_parsing` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_range_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tuple_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_slice_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_char_operations` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./test_context/example.py

**File Metrics**: Complexity: 0, Functions: 1

- **Function**: `hello` [complexity: 0] [cognitive: 0] [big-o: O(?)] [provability: 75%] [coverage: 65%]
- **Struct**: `Calculator`
- **Function**: `add` [provability: 75%] [coverage: 65%]

