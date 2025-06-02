# Project Context: rust Project

Generated: 2025-06-02 03:13:41 UTC

## Summary

- Files analyzed: 123
- Functions: 738
- Structs: 337
- Enums: 88
- Traits: 15
- Implementations: 219

## Dependencies

- anyhow
- async-trait
- atty
- axum
- blake3
- bytes
- chrono
- clap
- dashmap
- dirs
- flate2
- futures
- handlebars
- hex
- http
- http-body-util
- httparse
- hyper
- hyper-util
- ignore
- lru
- num_cpus
- once_cell
- parking_lot
- petgraph
- proc-macro2
- quote
- rand
- rayon
- regex
- roaring
- rustpython-parser
- semver
- serde
- serde_json
- serde_yaml
- sha2
- swc_common
- swc_ecma_ast
- swc_ecma_parser
- swc_ecma_visit
- syn
- thiserror
- tokio
- toml
- tower
- tower-http
- tracing
- tracing-subscriber
- uuid
- walkdir
- webbrowser

## Files

### ./src/bin/paiml-mcp-agent-toolkit.rs

**Enums:**
- `private enum ExecutionMode` (2 variants) (line 1)

**Functions:**
- `private fn detect_execution_mode` (line 1)
- `private fn init_tracing` (line 1)
- `private async fn main` (line 1)

### ./src/demo/mod.rs

**Modules:**
- `pub mod assets` (line 1)
- `pub mod runner` (line 1)
- `pub mod server` (line 1)
- `pub mod templates` (line 1)

**Structs:**
- `pub struct DemoArgs` (9 fields) (line 1)

**Functions:**
- `pub async fn run_demo` (line 1)
- `private fn extract_analysis_from_demo_report` (line 1)
- `private fn parse_complexity_summary` (line 1)
- `private fn parse_dag_data` (line 1)
- `private async fn run_web_demo` (line 1)
- `private async fn analyze_context` (line 1)
- `private async fn analyze_complexity` (line 1)
- `private async fn analyze_dag` (line 1)
- `private async fn analyze_churn` (line 1)
- `private async fn analyze_system_architecture` (line 1)
- `private async fn analyze_defect_probability` (line 1)

### ./src/demo/server.rs

**Structs:**
- `pub struct DemoContent` (10 fields) (line 1)
- `pub struct Hotspot` (3 fields) (line 1)
- `pub struct DemoState` (4 fields) (line 1)
- `pub struct AnalysisResults` (6 fields) (line 1)
- `pub struct LocalDemoServer` (2 fields) (line 1)
- `private struct MinimalRequest` (1 fields) (line 1)
- `private struct HotspotEntry` (5 fields) (line 1)

**Functions:**
- `private async fn handle_connection` (line 1)
- `private fn parse_minimal_request` (line 1)
- `private fn serialize_response` (line 1)
- `private fn serve_dashboard` (line 1)
- `private fn serve_static_asset` (line 1)
- `private fn serve_static_asset` (line 1)
- `private fn serve_architecture_analysis` (line 1)
- `private fn serve_defect_analysis` (line 1)
- `private fn serve_statistics_analysis` (line 1)
- `private fn serve_system_diagram` (line 1)
- `private fn serve_analysis_stream` (line 1)
- `private fn calculate_graph_density` (line 1)
- `private fn calculate_avg_degree` (line 1)
- `private fn serve_summary_json` (line 1)
- `private fn serve_metrics_json` (line 1)
- `private fn serve_hotspots_table` (line 1)
- `private fn serve_dag_mermaid` (line 1)
- `private fn serve_system_diagram_mermaid` (line 1)
- `private fn serve_architecture_analysis` (line 1)
- `private fn serve_defect_analysis` (line 1)
- `private fn serve_statistics_analysis` (line 1)
- `private fn serve_system_diagram` (line 1)
- `private fn serve_analysis_stream` (line 1)
- `private fn calculate_graph_density` (line 1)
- `private fn calculate_avg_degree` (line 1)
- `pub fn spawn_sync` (line 1)

**Implementations:**
- `impl LocalDemoServer` (line 1)
- `impl DemoContent` (line 1)
- `impl Default for ComplexityReport` (line 1)
- `impl Default for CodeChurnAnalysis` (line 1)

### ./src/demo/runner.rs

**Structs:**
- `pub struct DemoRunner` (2 fields) (line 1)
- `pub struct DemoStep` (7 fields) (line 1)
- `pub struct DemoReport` (4 fields) (line 1)
- `private struct Component` (4 fields) (line 1)

**Functions:**
- `pub fn detect_repository` (line 1)

**Implementations:**
- `impl DemoRunner` (line 1)
- `impl DemoReport` (line 1)

### ./src/demo/assets.rs

**Structs:**
- `pub struct EmbeddedAsset` (3 fields) (line 1)

**Enums:**
- `pub enum AssetEncoding` (2 variants) (line 1)

**Functions:**
- `pub fn get_asset` (line 1)
- `pub fn get_asset` (line 1)
- `pub fn decompress_asset` (line 1)
- `pub fn get_asset_hash` (line 1)

### ./src/demo/templates.rs

### ./src/tests/prompts.rs

**Functions:**
- `private fn create_test_server` (line 1)
- `private fn create_request` (line 1)
- `private async fn test_handle_prompts_list` (line 1)
- `private async fn test_handle_prompt_get_rust_project` (line 1)
- `private async fn test_handle_prompt_get_deno_project` (line 1)
- `private async fn test_handle_prompt_get_python_project` (line 1)
- `private async fn test_handle_prompt_get_missing_params` (line 1)
- `private async fn test_handle_prompt_get_invalid_params` (line 1)
- `private async fn test_handle_prompt_get_unknown_prompt` (line 1)

### ./src/tests/cli_tests.rs

**Modules:**
- `private mod cli_args_tests` (line 1)
- `private mod cli_integration_tests` (line 1)
- `private mod cli_parsing_tests` (line 1)
- `private mod cli_output_tests` (line 1)
- `private mod cli_error_handling_tests` (line 1)
- `private mod cli_mode_detection_tests` (line 1)

### ./src/tests/binary_size.rs

**Modules:**
- `private mod benchmarks` (line 1)

**Functions:**
- `private fn binary_size_regression` (line 1)
- `private fn feature_size_impact` (line 1)
- `private fn template_compression_works` (line 1)

### ./src/tests/ast_e2e.rs

**Modules:**
- `private mod ast_python_tests` (line 1)
- `private mod ast_typescript_tests` (line 1)
- `private mod ast_integration_tests` (line 1)

### ./src/tests/cli_integration_full.rs

**Modules:**
- `private mod tests` (line 1)

### ./src/tests/analyze_cli_tests.rs

**Modules:**
- `private mod tests` (line 1)

### ./src/tests/template_resources.rs

**Functions:**
- `private fn create_test_server` (line 1)
- `private async fn test_list_all_templates` (line 1)
- `private async fn test_filter_templates_by_prefix` (line 1)
- `private async fn test_get_template_metadata` (line 1)
- `private async fn test_get_template_content` (line 1)
- `private async fn test_invalid_template_uri` (line 1)
- `private async fn test_template_categories` (line 1)
- `private async fn test_template_toolchains` (line 1)
- `private async fn test_template_parameter_types` (line 1)
- `private async fn test_rust_template_parameters` (line 1)

### ./src/tests/additional_coverage.rs

**Functions:**
- `private fn test_churn_output_format` (line 1)
- `private fn test_cli_validate_params` (line 1)
- `private fn test_additional_model_coverage` (line 1)

### ./src/tests/cache_comprehensive_tests.rs

**Functions:**
- `private fn test_cache_config_default_values` (line 1)
- `private fn test_cache_config_custom_values` (line 1)
- `private fn test_cache_config_serialization` (line 1)
- `private fn test_cache_stats_snapshot_creation` (line 1)
- `private fn test_cache_stats_snapshot_zero_requests` (line 1)
- `private fn test_cache_stats_snapshot_serialization` (line 1)
- `private fn test_cache_effectiveness_structure` (line 1)
- `private fn test_cache_effectiveness_serialization` (line 1)
- `private fn test_cache_diagnostics_structure` (line 1)
- `private fn test_cache_stats_hit_rate_calculation` (line 1)
- `private fn test_cache_config_ttl_values` (line 1)
- `private fn test_cache_config_memory_settings` (line 1)
- `private fn test_cache_config_git_settings` (line 1)
- `private fn test_cache_config_warmup_patterns` (line 1)
- `private fn test_cache_effectiveness_empty_caches` (line 1)
- `private fn test_cache_diagnostics_empty_collections` (line 1)

### ./src/tests/deep_context_tests.rs

**Functions:**
- `private async fn test_deep_context_config_default_values` (line 1)
- `private async fn test_deep_context_analyzer_creation` (line 1)
- `private async fn test_discovery_simple_project` (line 1)
- `private async fn test_discovery_with_excludes` (line 1)
- `private async fn test_metadata_creation` (line 1)
- `private async fn test_quality_scorecard_calculations` (line 1)
- `private async fn test_defect_summary_aggregation` (line 1)
- `private async fn test_defect_hotspot_ranking` (line 1)
- `private async fn test_prioritized_recommendations` (line 1)
- `private async fn test_cross_language_references` (line 1)
- `private async fn test_template_provenance_tracking` (line 1)
- `private async fn test_analysis_type_equality` (line 1)
- `private async fn test_enum_variants_complete` (line 1)

### ./src/tests/cli_comprehensive_tests.rs

**Functions:**
- `private fn test_generate_command_full_parsing` (line 1)
- `private fn test_generate_command_aliases` (line 1)
- `private fn test_generate_missing_required_args` (line 1)
- `private fn test_scaffold_command_parsing` (line 1)
- `private fn test_scaffold_template_delimiter` (line 1)
- `private fn test_scaffold_default_parallel` (line 1)
- `private fn test_list_command_all_formats` (line 1)
- `private fn test_list_command_filters` (line 1)
- `private fn test_list_default_format` (line 1)
- `private fn test_search_command_parsing` (line 1)
- `private fn test_search_default_limit` (line 1)
- `private fn test_validate_command_parsing` (line 1)
- `private fn test_context_command_parsing` (line 1)
- `private fn test_context_formats` (line 1)
- `private fn test_context_default_values` (line 1)
- `private fn test_analyze_churn_full_options` (line 1)
- `private fn test_analyze_churn_all_formats` (line 1)
- `private fn test_analyze_complexity_full_options` (line 1)
- `private fn test_analyze_complexity_formats` (line 1)
- `private fn test_analyze_dag_full_options` (line 1)
- `private fn test_analyze_dag_types` (line 1)
- `private fn test_parse_key_val_basic` (line 1)
- `private fn test_parse_key_val_edge_cases` (line 1)
- `private fn test_validate_params_comprehensive` (line 1)
- `private fn test_expand_env_vars_complex` (line 1)
- `private fn test_cli_error_scenarios` (line 1)
- `private fn test_help_flags` (line 1)
- `private fn test_version_flag` (line 1)
- `private fn test_mode_flag` (line 1)
- `private fn test_multiple_parameter_types` (line 1)
- `private fn test_nested_subcommand_parsing` (line 1)

### ./src/tests/cache.rs

**Functions:**
- `private async fn test_session_cache_manager` (line 1)
- `private async fn test_ast_cache` (line 1)
- `private async fn test_template_cache` (line 1)
- `private async fn test_dag_cache` (line 1)
- `private async fn test_churn_cache` (line 1)
- `private async fn test_git_stats_cache` (line 1)
- `private async fn test_cache_eviction` (line 1)
- `private async fn test_cache_clear` (line 1)
- `private async fn test_cache_ttl` (line 1)

### ./src/tests/cli_property_tests.rs

### ./src/tests/churn.rs

**Modules:**
- `private mod tests` (line 1)

### ./src/tests/helpers.rs

**Modules:**
- `private mod helpers_tests` (line 1)

### ./src/tests/template_rendering.rs

**Functions:**
- `private fn test_render_rust_cli_makefile` (line 1)
- `private fn test_render_python_uv_makefile` (line 1)
- `private fn test_render_deno_typescript_makefile` (line 1)
- `private fn test_render_readme_template` (line 1)
- `private fn test_render_gitignore_template` (line 1)
- `private fn test_render_with_conditionals` (line 1)
- `private fn test_render_with_missing_parameters` (line 1)
- `private fn test_render_with_nested_loops` (line 1)
- `private fn test_render_with_string_helpers` (line 1)

### ./src/tests/http_adapter_tests.rs

**Functions:**
- `private async fn test_http_adapter_creation` (line 1)
- `private async fn test_http_adapter_bind` (line 1)
- `private async fn test_http_output_creation` (line 1)
- `private async fn test_http_context_creation` (line 1)
- `private async fn test_http_context_with_no_remote_addr` (line 1)
- `private async fn test_http_context_with_no_user_agent` (line 1)
- `private async fn test_http_context_empty` (line 1)
- `private fn test_protocol_adapter_trait` (line 1)
- `private async fn test_http_status_code_variations` (line 1)
- `private async fn test_http_response_with_json` (line 1)
- `private async fn test_http_error_responses` (line 1)

### ./src/tests/cli_simple_tests.rs

**Modules:**
- `private mod tests` (line 1)
- `private mod cli_command_enums` (line 1)

### ./src/tests/error_handling.rs

**Modules:**
- `private mod error_handling_tests` (line 1)

### ./src/tests/mcp_protocol.rs

**Functions:**
- `private fn create_test_server` (line 1)
- `private fn create_request` (line 1)
- `private async fn test_handle_initialize` (line 1)
- `private async fn test_handle_list_tools` (line 1)
- `private async fn test_handle_list_resources` (line 1)
- `private async fn test_handle_call_tool_generate_template` (line 1)
- `private async fn test_handle_call_tool_invalid_tool` (line 1)
- `private async fn test_handle_call_tool_missing_parameters` (line 1)
- `private async fn test_handle_invalid_method` (line 1)
- `private async fn test_protocol_version_default` (line 1)

### ./src/tests/deep_context_simplified_tests.rs

**Functions:**
- `private fn test_deep_context_config_default_values` (line 1)
- `private fn test_deep_context_analyzer_creation` (line 1)
- `private fn test_metadata_creation` (line 1)
- `private fn test_quality_scorecard_calculations` (line 1)
- `private fn test_defect_summary_aggregation` (line 1)
- `private fn test_prioritized_recommendations` (line 1)
- `private fn test_cross_language_references` (line 1)
- `private fn test_template_provenance_tracking` (line 1)
- `private fn test_analysis_type_equality` (line 1)
- `private fn test_enum_variants_complete` (line 1)

### ./src/tests/demo_comprehensive_tests.rs

**Functions:**
- `private async fn test_demo_runner_creation` (line 1)
- `private async fn test_demo_step_structure` (line 1)
- `private async fn test_demo_report_structure` (line 1)
- `private async fn test_detect_repository_git_repo` (line 1)
- `private async fn test_detect_repository_cargo_project` (line 1)
- `private async fn test_detect_repository_nodejs_project` (line 1)
- `private async fn test_detect_repository_python_project` (line 1)
- `private async fn test_detect_repository_pyproject_toml` (line 1)
- `private async fn test_detect_repository_with_readme` (line 1)
- `private async fn test_detect_repository_empty_directory` (line 1)
- `private async fn test_detect_repository_nonexistent_path` (line 1)
- `private async fn test_demo_report_rendering_cli` (line 1)
- `private async fn test_demo_report_rendering_mcp` (line 1)
- `private async fn test_demo_step_error_handling` (line 1)
- `private async fn test_demo_report_with_multiple_steps` (line 1)
- `private fn test_demo_step_serialization` (line 1)
- `private fn test_demo_report_serialization` (line 1)

### ./src/tests/resources.rs

**Functions:**
- `private fn create_test_server` (line 1)
- `private fn create_request` (line 1)
- `private async fn test_handle_resource_list` (line 1)
- `private async fn test_handle_resource_read_success` (line 1)
- `private async fn test_handle_resource_read_missing_params` (line 1)
- `private async fn test_handle_resource_read_invalid_params` (line 1)
- `private async fn test_handle_resource_read_not_found` (line 1)
- `private async fn test_handle_resource_read_all_templates` (line 1)

### ./src/tests/error.rs

**Functions:**
- `private fn test_template_not_found_error` (line 1)
- `private fn test_invalid_uri_error` (line 1)
- `private fn test_validation_error` (line 1)
- `private fn test_render_error` (line 1)
- `private fn test_not_found_error` (line 1)
- `private fn test_s3_error` (line 1)
- `private fn test_invalid_utf8_error` (line 1)
- `private fn test_cache_error` (line 1)
- `private fn test_json_error` (line 1)
- `private fn test_io_error` (line 1)

### ./src/tests/build_naming_validation.rs

**Modules:**
- `private mod tests` (line 1)

### ./src/tests/tools.rs

**Functions:**
- `private fn create_test_server` (line 1)
- `private fn create_request` (line 1)
- `private async fn test_handle_tool_call_missing_params` (line 1)
- `private async fn test_handle_tool_call_invalid_params` (line 1)
- `private async fn test_list_templates_all` (line 1)
- `private async fn test_list_templates_by_toolchain` (line 1)
- `private async fn test_list_templates_by_category` (line 1)
- `private async fn test_validate_template_valid` (line 1)
- `private async fn test_validate_template_missing_required` (line 1)
- `private async fn test_validate_template_unknown_parameter` (line 1)
- `private async fn test_validate_template_not_found` (line 1)
- `private async fn test_scaffold_project_rust` (line 1)
- `private async fn test_scaffold_project_deno` (line 1)
- `private async fn test_search_templates_by_name` (line 1)
- `private async fn test_search_templates_with_toolchain_filter` (line 1)
- `private async fn test_search_templates_no_results` (line 1)
- `private async fn test_generate_template_invalid_arguments` (line 1)

### ./src/tests/lib.rs

**Functions:**
- `private async fn test_template_server_new` (line 1)
- `private async fn test_template_server_trait_implementation` (line 1)
- `private async fn test_template_server_deprecated_methods` (line 1)
- `private async fn test_warm_cache` (line 1)
- `private async fn test_template_server_cache_initialization` (line 1)
- `private async fn test_template_server_cache_sizes` (line 1)
- `private async fn test_warm_cache_templates` (line 1)
- `private async fn test_template_server_trait_via_methods` (line 1)
- `private fn test_type_aliases` (line 1)
- `private fn test_s3_client_instantiation` (line 1)
- `private async fn test_run_mcp_server_basic` (line 1)
- `private fn test_public_exports` (line 1)

### ./src/tests/claude_code_e2e.rs

**Structs:**
- `private struct GeneratedFileFlags` (3 fields) (line 1)

**Functions:**
- `private fn create_test_server` (line 1)
- `private fn create_tool_request` (line 1)
- `private async fn test_claude_code_rust_cli_workflow` (line 1)
- `private async fn test_claude_code_all_languages_scaffold` (line 1)
- `private fn create_scaffold_test_cases` (line 1)
- `private async fn test_toolchain_scaffolding` (line 1)
- `private fn create_scaffold_request` (line 1)
- `private fn validate_scaffold_response` (line 1)
- `private fn verify_generated_files` (line 1)
- `private fn process_generated_file` (line 1)
- `private fn verify_makefile` (line 1)
- `private fn verify_makefile_toolchain_specific` (line 1)
- `private fn verify_readme` (line 1)
- `private fn verify_gitignore` (line 1)
- `private fn verify_gitignore_patterns` (line 1)
- `private async fn test_claude_code_error_scenarios` (line 1)
- `private async fn test_claude_code_search_templates` (line 1)
- `private async fn test_naming_convention_critical_requirement` (line 1)
- `private async fn test_naming_convention_in_individual_templates` (line 1)
- `private async fn test_server_info_naming_convention` (line 1)
- `private async fn test_ast_context_generation` (line 1)

**Implementations:**
- `impl GeneratedFileFlags` (line 1)

### ./src/tests/e2e_full_coverage.rs

**Functions:**
- `private async fn test_mcp_server_e2e_coverage` (line 1)
- `private fn test_cli_main_binary_version` (line 1)
- `private fn test_cli_main_binary_help` (line 1)
- `private fn test_cli_subcommand_help` (line 1)
- `private fn test_cli_mode_list_templates` (line 1)
- `private fn test_cli_generate_validation_error` (line 1)
- `private fn test_cli_search_templates` (line 1)
- `private fn test_cli_invalid_command` (line 1)
- `private fn test_cli_analyze_churn` (line 1)

### ./src/tests/models.rs

**Functions:**
- `private fn test_toolchain_priority` (line 1)
- `private fn test_toolchain_as_str` (line 1)
- `private fn test_template_category_serialization` (line 1)
- `private fn test_parameter_type_serialization` (line 1)

### ./src/tests/unified_protocol_tests.rs

**Functions:**
- `private async fn test_unified_service_creation` (line 1)
- `private async fn test_app_state_default` (line 1)
- `private async fn test_service_metrics_creation` (line 1)
- `private async fn test_service_metrics_increment` (line 1)
- `private async fn test_service_metrics_duration_tracking` (line 1)
- `private async fn test_service_metrics_error_tracking` (line 1)
- `private fn test_protocol_context_http_only` (line 1)
- `private fn test_protocol_context_mcp_only` (line 1)
- `private fn test_app_error_types` (line 1)
- `private fn test_app_error_status_codes` (line 1)
- `private fn test_mcp_error_codes` (line 1)
- `private fn test_error_types` (line 1)
- `private async fn test_error_to_protocol_response` (line 1)

### ./src/cli/mod.rs

**Modules:**
- `pub mod args` (line 1)

**Structs:**
- `pub(crate) struct Cli` (6 fields) (line 1)
- `pub struct EarlyCliArgs` (4 fields) (line 1)
- `private struct DeepContextConfigParams` (10 fields) (line 1)

**Enums:**
- `pub(crate) enum Mode` (2 variants) (line 1)
- `pub enum ExecutionMode` (2 variants) (line 1)
- `pub enum Commands` (9 variants) (line 1)
- `pub enum AnalyzeCommands` (6 variants) (line 1)
- `pub enum ContextFormat` (2 variants) (line 1)
- `pub enum OutputFormat` (3 variants) (line 1)
- `pub enum ComplexityOutputFormat` (4 variants) (line 1)
- `pub enum DeadCodeOutputFormat` (4 variants) (line 1)
- `pub enum SatdOutputFormat` (4 variants) (line 1)
- `pub enum SatdSeverity` (4 variants) (line 1)
- `pub enum DagType` (4 variants) (line 1)
- `pub enum DeepContextOutputFormat` (3 variants) (line 1)
- `pub enum DeepContextDagType` (4 variants) (line 1)
- `pub enum DeepContextCacheStrategy` (3 variants) (line 1)

**Functions:**
- `pub fn parse_early_for_tracing` (line 1)
- `pub async fn run` (line 1)
- `private async fn execute_command` (line 1)
- `private async fn execute_analyze_command` (line 1)
- `private async fn execute_demo_command` (line 1)
- `private async fn handle_generate` (line 1)
- `private async fn handle_scaffold` (line 1)
- `private async fn handle_list` (line 1)
- `private async fn handle_search` (line 1)
- `private async fn handle_validate` (line 1)
- `private async fn handle_context` (line 1)
- `private async fn handle_analyze_churn` (line 1)
- `private async fn handle_analyze_dag` (line 1)
- `private async fn handle_analyze_complexity` (line 1)
- `private async fn handle_analyze_dead_code` (line 1)
- `private async fn handle_analyze_satd` (line 1)
- `private async fn handle_analyze_deep_context` (line 1)
- `private fn build_deep_context_config` (line 1)
- `private fn convert_dag_type` (line 1)
- `private fn convert_cache_strategy` (line 1)
- `private fn parse_analysis_filters` (line 1)
- `private fn parse_analysis_type` (line 1)
- `private fn print_analysis_summary` (line 1)
- `private async fn write_deep_context_output` (line 1)
- `private fn format_deep_context_as_markdown` (line 1)
- `private fn format_deep_context_comprehensive` (line 1)
- `private fn format_annotated_tree` (line 1)
- `private fn format_tree_node` (line 1)
- `private fn format_complexity_hotspots` (line 1)
- `private fn format_churn_analysis` (line 1)
- `private fn format_technical_debt` (line 1)
- `private fn format_dead_code_analysis` (line 1)
- `private fn format_defect_predictions` (line 1)
- `private fn format_prioritized_recommendations` (line 1)
- `private fn format_deep_context_terse` (line 1)
- `private fn format_terse_header` (line 1)
- `private fn format_terse_executive_summary` (line 1)
- `private fn get_terse_satd_breakdown` (line 1)
- `private fn format_terse_key_metrics` (line 1)
- `private fn format_terse_complexity_metrics` (line 1)
- `private fn format_terse_churn_metrics` (line 1)
- `private fn calculate_terse_median_changes` (line 1)
- `private fn format_terse_satd_metrics` (line 1)
- `private fn format_terse_duplicates_metrics` (line 1)
- `private fn format_terse_dead_code_metrics` (line 1)
- `private fn format_terse_ast_network_analysis` (line 1)
- `private fn format_terse_predicted_defect_files` (line 1)
- `private fn calculate_terse_file_risks` (line 1)
- `private fn format_deep_context_full` (line 1)
- `private fn format_full_report_header` (line 1)
- `private fn format_full_executive_summary` (line 1)
- `private fn get_satd_breakdown` (line 1)
- `private fn format_full_complexity_analysis` (line 1)
- `private fn format_full_churn_analysis` (line 1)
- `private fn calculate_median_changes` (line 1)
- `private fn format_full_satd_analysis` (line 1)
- `private fn format_full_dead_code_analysis` (line 1)
- `private fn format_full_risk_prediction` (line 1)
- `private fn calculate_file_risks` (line 1)
- `private fn format_full_recommendations` (line 1)
- `private fn format_deep_context_as_sarif` (line 1)
- `private fn format_satd_output` (line 1)
- `private fn format_satd_summary` (line 1)
- `private fn format_satd_as_sarif` (line 1)
- `private fn format_satd_as_markdown` (line 1)
- `private fn format_dead_code_output` (line 1)
- `private fn format_dead_code_summary` (line 1)
- `private fn format_dead_code_as_sarif` (line 1)
- `private fn format_dead_code_as_markdown` (line 1)
- `private fn detect_toolchain` (line 1)
- `private fn build_complexity_thresholds` (line 1)
- `private async fn analyze_project_files` (line 1)
- `private fn should_analyze_file` (line 1)
- `private fn matches_include_patterns` (line 1)
- `private async fn analyze_file_by_toolchain` (line 1)
- `private fn params_to_json` (line 1)
- `private fn format_top_files_ranking` (line 1)
- `private async fn handle_serve` (line 1)
- `private fn print_table` (line 1)

### ./src/cli/args.rs

**Functions:**
- `pub fn validate_params` (line 1)
- `private fn validate_type` (line 1)
- `private fn value_type_name` (line 1)
- `pub fn expand_env_vars` (line 1)
- `pub fn parse_key_val` (line 1)

### ./src/unified_protocol/adapters/cli.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct CliAdapter` (0 fields) (line 1)
- `pub struct CliInput` (3 fields) (line 1)
- `pub struct CliRunner` (1 fields) (line 1)

**Enums:**
- `pub enum CliOutput` (2 variants) (line 1)

**Functions:**
- `private fn format_to_string` (line 1)
- `private fn churn_format_to_string` (line 1)
- `private fn complexity_format_to_string` (line 1)
- `private fn dag_type_to_string` (line 1)
- `private fn dead_code_format_to_string` (line 1)
- `private fn satd_format_to_string` (line 1)
- `private fn satd_severity_to_string` (line 1)
- `private fn deep_context_format_to_string` (line 1)
- `private fn deep_context_dag_type_to_string` (line 1)
- `private fn deep_context_cache_strategy_to_string` (line 1)

**Implementations:**
- `impl CliAdapter` (line 1)
- `impl Default for CliAdapter` (line 1)
- `impl ProtocolAdapter for CliAdapter` (line 1)
- `impl CliInput` (line 1)
- `impl CliOutput` (line 1)
- `impl CliRunner` (line 1)
- `impl Default for CliRunner` (line 1)

### ./src/unified_protocol/adapters/mod.rs

**Modules:**
- `pub mod cli` (line 1)
- `pub mod http` (line 1)
- `pub mod mcp` (line 1)

### ./src/unified_protocol/adapters/http.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct HttpAdapter` (2 fields) (line 1)
- `pub struct HttpStreamAdapter` (2 fields) (line 1)
- `pub struct HttpServer` (2 fields) (line 1)
- `pub struct HttpResponseBuilder` (0 fields) (line 1)

**Enums:**
- `pub enum HttpInput` (2 variants) (line 1)
- `pub enum HttpOutput` (1 variants) (line 1)

**Traits:**
- `pub trait HttpServiceHandler` (line 1)

**Functions:**
- `private async fn handle_connection` (line 1)
- `private async fn process_http_request` (line 1)
- `private async fn convert_hyper_to_http_input` (line 1)
- `private async fn collect_request_body` (line 1)
- `private async fn decode_http_input` (line 1)
- `private async fn handle_unified_request` (line 1)
- `private async fn encode_unified_response` (line 1)
- `private async fn serve_http_connection` (line 1)

**Implementations:**
- `impl HttpAdapter` (line 1)
- `impl ProtocolAdapter for HttpAdapter` (line 1)
- `impl ProtocolAdapter for HttpStreamAdapter` (line 1)
- `impl HttpServer` (line 1)
- `impl HttpResponseBuilder` (line 1)

### ./src/unified_protocol/adapters/mcp.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct McpAdapter` (1 fields) (line 1)
- `pub struct JsonRpcRequest` (4 fields) (line 1)
- `pub struct JsonRpcResponse` (4 fields) (line 1)
- `pub struct JsonRpcError` (3 fields) (line 1)
- `pub struct McpReader` (1 fields) (line 1)

**Enums:**
- `pub enum McpInput` (2 variants) (line 1)

**Implementations:**
- `impl McpAdapter` (line 1)
- `impl Default for McpAdapter` (line 1)
- `impl ProtocolAdapter for McpAdapter` (line 1)
- `impl JsonRpcRequest` (line 1)
- `impl JsonRpcResponse` (line 1)
- `impl JsonRpcError` (line 1)
- `impl McpReader` (line 1)

### ./src/unified_protocol/mod.rs

**Modules:**
- `pub mod adapters` (line 1)
- `pub mod error` (line 1)
- `pub mod service` (line 1)
- `private mod tests` (line 1)

**Structs:**
- `pub struct UnifiedRequest` (6 fields) (line 1)
- `pub struct UnifiedResponse` (4 fields) (line 1)
- `pub struct AdapterRegistry` (1 fields) (line 1)
- `private struct AdapterWrapper` (1 fields) (line 1)
- `pub struct McpContext` (2 fields) (line 1)
- `pub struct CliContext` (2 fields) (line 1)
- `pub struct HttpContext` (2 fields) (line 1)

**Enums:**
- `pub enum Protocol` (4 variants) (line 1)
- `pub enum ProtocolError` (7 variants) (line 1)

**Traits:**
- `pub trait ProtocolAdapter` (line 1)

**Implementations:**
- `impl UnifiedRequest` (line 1)
- `impl UnifiedResponse` (line 1)
- `impl IntoResponse for UnifiedResponse` (line 1)
- `impl Display for Protocol` (line 1)
- `impl AdapterRegistry` (line 1)
- `impl AdapterWrapper` (line 1)
- `impl ProtocolAdapter for AdapterWrapper` (line 1)

### ./src/unified_protocol/test_harness.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct TestHarness` (5 fields) (line 1)
- `pub struct TestResults` (5 fields) (line 1)
- `pub struct EquivalenceFailure` (5 fields) (line 1)
- `pub struct TestSuiteResults` (3 fields) (line 1)

**Enums:**
- `pub enum TestError` (8 variants) (line 1)

**Implementations:**
- `impl TestHarness` (line 1)
- `impl TestSuiteResults` (line 1)
- `impl Default for TestHarness` (line 1)

### ./src/unified_protocol/error.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct McpError` (3 fields) (line 1)
- `pub struct HttpErrorResponse` (3 fields) (line 1)
- `pub struct CliErrorResponse` (3 fields) (line 1)

**Enums:**
- `pub enum AppError` (14 variants) (line 1)

**Functions:**
- `private fn extract_protocol_from_context` (line 1)
- `pub fn set_protocol_context` (line 1)
- `pub fn clear_protocol_context` (line 1)

**Implementations:**
- `impl AppError` (line 1)
- `impl IntoResponse for AppError` (line 1)

### ./src/unified_protocol/service.rs

**Modules:**
- `pub mod handlers` (line 1)
- `private mod tests` (line 1)

**Structs:**
- `pub struct UnifiedService` (3 fields) (line 1)
- `pub struct AppState` (3 fields) (line 1)
- `pub struct ServiceMetrics` (3 fields) (line 1)
- `pub struct DefaultTemplateService` (0 fields) (line 1)
- `pub struct DefaultAnalysisService` (0 fields) (line 1)
- `pub struct ListTemplatesQuery` (2 fields) (line 1)
- `pub struct TemplateList` (2 fields) (line 1)
- `pub struct TemplateInfo` (5 fields) (line 1)
- `pub struct TemplateParameter` (4 fields) (line 1)
- `pub struct GenerateParams` (2 fields) (line 1)
- `pub struct GeneratedTemplate` (3 fields) (line 1)
- `pub struct TemplateMetadata` (3 fields) (line 1)
- `pub struct ComplexityParams` (6 fields) (line 1)
- `pub struct ComplexityQueryParams` (6 fields) (line 1)
- `pub struct ComplexityAnalysis` (2 fields) (line 1)
- `pub struct ComplexitySummary` (4 fields) (line 1)
- `pub struct FileComplexity` (2 fields) (line 1)
- `pub struct FunctionComplexity` (4 fields) (line 1)
- `pub struct ChurnParams` (3 fields) (line 1)
- `pub struct ChurnAnalysis` (2 fields) (line 1)
- `pub struct ChurnSummary` (3 fields) (line 1)
- `pub struct ChurnHotspot` (3 fields) (line 1)
- `pub struct DagParams` (4 fields) (line 1)
- `pub struct DagAnalysis` (4 fields) (line 1)
- `pub struct ContextParams` (3 fields) (line 1)
- `pub struct ProjectContext` (4 fields) (line 1)
- `pub struct ProjectStructure` (2 fields) (line 1)
- `pub struct ContextMetrics` (3 fields) (line 1)
- `pub struct DeadCodeParams` (6 fields) (line 1)
- `pub struct DeadCodeAnalysis` (2 fields) (line 1)
- `pub struct DeadCodeSummary` (4 fields) (line 1)
- `pub struct FileDeadCode` (6 fields) (line 1)

**Traits:**
- `pub trait TemplateService` (line 1)
- `pub trait AnalysisService` (line 1)

**Implementations:**
- `impl Default for AppState` (line 1)
- `impl UnifiedService` (line 1)
- `impl Default for UnifiedService` (line 1)
- `impl TemplateService for DefaultTemplateService` (line 1)
- `impl AnalysisService for DefaultAnalysisService` (line 1)

### ./src/models/mod.rs

**Modules:**
- `pub mod churn` (line 1)
- `pub mod dag` (line 1)
- `pub mod dead_code` (line 1)
- `pub mod error` (line 1)
- `pub mod mcp` (line 1)
- `pub mod template` (line 1)
- `pub mod unified_ast` (line 1)

### ./src/models/dead_code.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct FileDeadCodeMetrics` (11 fields) (line 1)
- `pub struct DeadCodeItem` (4 fields) (line 1)
- `pub struct DeadCodeRankingResult` (4 fields) (line 1)
- `pub struct DeadCodeSummary` (8 fields) (line 1)
- `pub struct DeadCodeAnalysisConfig` (3 fields) (line 1)

**Enums:**
- `pub enum ConfidenceLevel` (3 variants) (line 1)
- `pub enum DeadCodeType` (4 variants) (line 1)

**Implementations:**
- `impl FileDeadCodeMetrics` (line 1)
- `impl DeadCodeSummary` (line 1)
- `impl Default for DeadCodeAnalysisConfig` (line 1)

### ./src/models/dag.rs

**Structs:**
- `pub struct DependencyGraph` (2 fields) (line 1)
- `pub struct NodeInfo` (6 fields) (line 1)
- `pub struct Edge` (4 fields) (line 1)

**Enums:**
- `pub enum NodeType` (5 variants) (line 1)
- `pub enum EdgeType` (5 variants) (line 1)

**Implementations:**
- `impl DependencyGraph` (line 1)
- `impl Default for DependencyGraph` (line 1)

### ./src/models/churn.rs

**Structs:**
- `pub struct CodeChurnAnalysis` (5 fields) (line 1)
- `pub struct FileChurnMetrics` (9 fields) (line 1)
- `pub struct ChurnSummary` (5 fields) (line 1)

**Enums:**
- `pub enum ChurnOutputFormat` (4 variants) (line 1)

**Implementations:**
- `impl FileChurnMetrics` (line 1)
- `impl FromStr for ChurnOutputFormat` (line 1)

### ./src/models/template.rs

**Structs:**
- `pub struct TemplateResource` (10 fields) (line 1)
- `pub struct ParameterSpec` (6 fields) (line 1)
- `pub struct GeneratedTemplate` (4 fields) (line 1)
- `pub struct TemplateResponse` (1 fields) (line 1)

**Enums:**
- `pub enum Toolchain` (3 variants) (line 1)
- `pub enum TemplateCategory` (4 variants) (line 1)
- `pub enum ParameterType` (6 variants) (line 1)

**Implementations:**
- `impl Toolchain` (line 1)

### ./src/models/unified_ast.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct NodeFlags` (1 fields) (line 1)
- `pub struct UnifiedAstNode` (11 fields) (line 1)
- `pub struct ColumnStore` (2 fields) (line 1)
- `pub struct AstDag` (4 fields) (line 1)
- `pub struct LanguageParsers` (0 fields) (line 1)

**Enums:**
- `pub enum Language` (4 variants) (line 1)
- `pub enum AstKind` (8 variants) (line 1)
- `pub enum FunctionKind` (7 variants) (line 1)
- `pub enum ClassKind` (6 variants) (line 1)
- `pub enum VarKind` (5 variants) (line 1)
- `pub enum ImportKind` (5 variants) (line 1)
- `pub enum ExprKind` (8 variants) (line 1)
- `pub enum StmtKind` (8 variants) (line 1)
- `pub enum TypeKind` (8 variants) (line 1)
- `pub enum ModuleKind` (3 variants) (line 1)

**Implementations:**
- `impl NodeFlags` (line 1)
- `impl Default for NodeMetadata` (line 1)
- `impl Clone for NodeMetadata` (line 1)
- `impl Copy for NodeMetadata` (line 1)
- `impl UnifiedAstNode` (line 1)
- `impl Debug for UnifiedAstNode` (line 1)
- `impl ColumnStore` (line 1)
- `impl Default for AstDag` (line 1)
- `impl AstDag` (line 1)

### ./src/models/mcp.rs

**Structs:**
- `pub struct McpRequest` (4 fields) (line 1)
- `pub struct McpResponse` (4 fields) (line 1)
- `pub struct McpError` (3 fields) (line 1)
- `pub struct ToolCallParams` (2 fields) (line 1)
- `pub struct GenerateTemplateArgs` (2 fields) (line 1)
- `pub struct ListTemplatesArgs` (2 fields) (line 1)
- `pub struct ResourceReadParams` (1 fields) (line 1)
- `pub struct ValidateTemplateArgs` (2 fields) (line 1)
- `pub struct ScaffoldProjectArgs` (3 fields) (line 1)
- `pub struct SearchTemplatesArgs` (2 fields) (line 1)
- `pub struct PromptGetParams` (1 fields) (line 1)
- `pub struct Prompt` (3 fields) (line 1)
- `pub struct PromptArgument` (3 fields) (line 1)

**Implementations:**
- `impl McpResponse` (line 1)

### ./src/models/error.rs

**Enums:**
- `pub enum TemplateError` (10 variants) (line 1)

**Implementations:**
- `impl TemplateError` (line 1)

### ./src/lib.rs

**Modules:**
- `pub mod cli` (line 1)
- `pub mod demo` (line 1)
- `pub mod handlers` (line 1)
- `pub mod models` (line 1)
- `pub mod services` (line 1)
- `pub mod stateless_server` (line 1)
- `pub mod unified_protocol` (line 1)
- `pub mod utils` (line 1)
- `private mod tests` (line 1)

**Structs:**
- `pub struct S3Client` (0 fields) (line 1)
- `pub struct TemplateServer` (5 fields) (line 1)

**Traits:**
- `pub trait TemplateServerTrait` (line 1)

**Functions:**
- `pub async fn run_mcp_server` (line 1)

**Implementations:**
- `impl TemplateServer` (line 1)
- `impl TemplateServerTrait for TemplateServer` (line 1)

### ./src/utils/mod.rs

**Modules:**
- `pub mod helpers` (line 1)

### ./src/utils/helpers.rs

**Modules:**
- `private mod tests` (line 1)

**Functions:**
- `pub fn snake_case_helper` (line 1)
- `pub fn kebab_case_helper` (line 1)
- `pub fn pascal_case_helper` (line 1)
- `pub fn current_year_helper` (line 1)
- `pub fn current_date_helper` (line 1)
- `private fn to_snake_case` (line 1)
- `private fn to_kebab_case` (line 1)
- `private fn to_pascal_case` (line 1)

### ./src/stateless_server.rs

**Structs:**
- `pub struct StatelessTemplateServer` (1 fields) (line 1)

**Implementations:**
- `impl StatelessTemplateServer` (line 1)
- `impl TemplateServerTrait for StatelessTemplateServer` (line 1)

### ./src/handlers/prompts.rs

**Functions:**
- `pub async fn handle_prompts_list` (line 1)
- `pub async fn handle_prompt_get` (line 1)

### ./src/handlers/mod.rs

**Modules:**
- `pub mod initialize` (line 1)
- `pub mod prompts` (line 1)
- `pub mod resources` (line 1)
- `pub mod tools` (line 1)

**Functions:**
- `pub async fn handle_request` (line 1)

### ./src/handlers/resources.rs

**Functions:**
- `pub async fn handle_resource_list` (line 1)
- `pub async fn handle_resource_read` (line 1)

### ./src/handlers/tools.rs

**Structs:**
- `private struct ValidationResult` (2 fields) (line 1)
- `private struct AnalyzeCodeChurnArgs` (3 fields) (line 1)
- `private struct AnalyzeComplexityArgs` (7 fields) (line 1)
- `private struct AnalyzeDagArgs` (5 fields) (line 1)
- `private struct GenerateContextArgs` (3 fields) (line 1)
- `private struct AnalyzeSystemArchitectureArgs` (3 fields) (line 1)
- `private struct AnalyzeDefectProbabilityArgs` (2 fields) (line 1)
- `private struct AnalyzeDeadCodeArgs` (6 fields) (line 1)
- `private struct AnalyzeDeepContextArgs` (11 fields) (line 1)

**Functions:**
- `pub async fn handle_tool_call` (line 1)
- `private fn parse_tool_call_params` (line 1)
- `private async fn dispatch_tool_call` (line 1)
- `private fn is_template_tool` (line 1)
- `private fn is_analysis_tool` (line 1)
- `private async fn handle_template_tools` (line 1)
- `private async fn handle_analysis_tools` (line 1)
- `private async fn handle_generate_template` (line 1)
- `private async fn handle_list_templates` (line 1)
- `private async fn handle_validate_template` (line 1)
- `private fn parse_validate_template_args` (line 1)
- `private fn validate_template_parameters` (line 1)
- `private fn find_missing_required_parameters` (line 1)
- `private fn validate_parameter_values` (line 1)
- `private fn validate_single_parameter` (line 1)
- `private fn create_validation_response` (line 1)
- `private async fn handle_scaffold_project` (line 1)
- `private async fn handle_search_templates` (line 1)
- `private async fn handle_get_server_info` (line 1)
- `private async fn handle_analyze_code_churn` (line 1)
- `pub fn format_churn_summary` (line 1)
- `pub fn format_churn_as_markdown` (line 1)
- `pub fn format_churn_as_csv` (line 1)
- `private fn calculate_relevance` (line 1)
- `private async fn handle_analyze_complexity` (line 1)
- `private fn resolve_project_path_complexity` (line 1)
- `private fn detect_toolchain` (line 1)
- `private fn build_complexity_thresholds` (line 1)
- `private async fn analyze_project_files` (line 1)
- `private fn should_analyze_file` (line 1)
- `private fn matches_include_filters` (line 1)
- `private fn matches_pattern` (line 1)
- `private async fn analyze_file_complexity` (line 1)
- `private fn format_complexity_output` (line 1)
- `private fn format_complexity_rankings` (line 1)
- `private async fn handle_analyze_dag` (line 1)
- `private async fn handle_generate_context` (line 1)
- `private async fn handle_analyze_system_architecture` (line 1)
- `private async fn handle_analyze_defect_probability` (line 1)
- `private async fn handle_analyze_dead_code` (line 1)
- `private fn format_dead_code_output` (line 1)
- `private fn format_dead_code_summary_mcp` (line 1)
- `private fn format_dead_code_as_sarif_mcp` (line 1)
- `private fn format_dead_code_as_markdown_mcp` (line 1)
- `private async fn handle_analyze_deep_context` (line 1)
- `private fn parse_deep_context_args` (line 1)
- `private fn resolve_project_path` (line 1)
- `private fn parse_analysis_types` (line 1)
- `private fn parse_dag_type` (line 1)
- `private fn parse_cache_strategy` (line 1)
- `private fn build_deep_context_config` (line 1)
- `private fn create_deep_context_analyzer` (line 1)
- `private fn format_deep_context_response` (line 1)
- `private fn format_deep_context_as_sarif` (line 1)
- `private fn format_deep_context_as_markdown` (line 1)

### ./src/handlers/initialize.rs

**Functions:**
- `pub async fn handle_initialize` (line 1)
- `pub async fn handle_tools_list` (line 1)

### ./src/services/code_intelligence.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct AnalysisRequest` (6 fields) (line 1)
- `pub struct AnalysisReport` (7 fields) (line 1)
- `pub struct ComplexityReport` (3 fields) (line 1)
- `pub struct ComplexityHotspot` (4 fields) (line 1)
- `pub struct DependencyGraphReport` (4 fields) (line 1)
- `pub struct DefectScore` (4 fields) (line 1)
- `pub struct GraphMetricsReport` (3 fields) (line 1)
- `pub struct CentralityScore` (5 fields) (line 1)
- `pub struct UnifiedCache` (1 fields) (line 1)
- `pub struct CodeIntelligence` (4 fields) (line 1)

**Enums:**
- `pub enum AnalysisType` (6 variants) (line 1)

**Functions:**
- `pub async fn analyze_dag_enhanced` (line 1)

**Implementations:**
- `impl AnalysisRequest` (line 1)
- `impl UnifiedCache` (line 1)
- `impl Default for CodeIntelligence` (line 1)
- `impl CodeIntelligence` (line 1)

### ./src/services/universal_output_adapter.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct UniversalOutputAdapter` (2 fields) (line 1)
- `pub struct EnhancedContext` (4 fields) (line 1)
- `pub struct FormatMetadata` (4 fields) (line 1)
- `pub struct SizeStatistics` (4 fields) (line 1)
- `private struct ExplicitFormatDetector` (0 fields) (line 1)
- `private struct ExtensionBasedDetector` (0 fields) (line 1)
- `private struct EnvironmentDetector` (0 fields) (line 1)
- `private struct PipelineDetector` (0 fields) (line 1)
- `private struct DefaultDetector` (0 fields) (line 1)
- `private struct LLMContextEnhancer` (0 fields) (line 1)
- `private struct SarifEnhancer` (0 fields) (line 1)
- `private struct MarkdownEnhancer` (0 fields) (line 1)
- `private struct JsonEnhancer` (0 fields) (line 1)

**Enums:**
- `pub enum AudienceType` (5 variants) (line 1)
- `pub enum OptimizationType` (5 variants) (line 1)

**Traits:**
- `pub trait FormatDetector` (line 1)
- `pub trait QualityEnhancer` (line 1)

**Functions:**
- `private fn estimate_context_size` (line 1)

**Implementations:**
- `impl UniversalOutputAdapter` (line 1)
- `impl FormatDetector for ExplicitFormatDetector` (line 1)
- `impl FormatDetector for ExtensionBasedDetector` (line 1)
- `impl FormatDetector for EnvironmentDetector` (line 1)
- `impl FormatDetector for PipelineDetector` (line 1)
- `impl FormatDetector for DefaultDetector` (line 1)
- `impl QualityEnhancer for LLMContextEnhancer` (line 1)
- `impl QualityEnhancer for SarifEnhancer` (line 1)
- `impl QualityEnhancer for MarkdownEnhancer` (line 1)
- `impl QualityEnhancer for JsonEnhancer` (line 1)
- `impl Default for UniversalOutputAdapter` (line 1)

### ./src/services/context.rs

**Structs:**
- `pub struct ProjectContext` (3 fields) (line 1)
- `pub struct ProjectSummary` (7 fields) (line 1)
- `pub struct FileContext` (4 fields) (line 1)
- `private struct RustVisitor` (2 fields) (line 1)
- `private struct GroupedItems` (6 fields) (line 1)

**Enums:**
- `pub enum AstItem` (7 variants) (line 1)

**Functions:**
- `pub async fn analyze_rust_file` (line 1)
- `pub async fn analyze_rust_file_with_cache` (line 1)
- `pub async fn analyze_project` (line 1)
- `pub async fn analyze_rust_file_with_persistent_cache` (line 1)
- `pub async fn analyze_project_with_cache` (line 1)
- `private fn build_gitignore` (line 1)
- `private async fn scan_and_analyze_files` (line 1)
- `private async fn analyze_file_by_toolchain` (line 1)
- `private async fn analyze_deno_file` (line 1)
- `private async fn build_project_summary` (line 1)
- `private fn calculate_item_counts` (line 1)
- `private async fn read_dependencies` (line 1)
- `private async fn read_rust_dependencies` (line 1)
- `private async fn read_deno_dependencies` (line 1)
- `private async fn read_python_dependencies` (line 1)
- `pub async fn analyze_project_with_persistent_cache` (line 1)
- `private async fn scan_and_analyze_files_persistent` (line 1)
- `private async fn analyze_file_by_toolchain_persistent` (line 1)
- `pub fn format_context_as_markdown` (line 1)
- `private fn format_header` (line 1)
- `private fn format_summary` (line 1)
- `private fn format_dependencies` (line 1)
- `private fn format_files` (line 1)
- `private fn group_items_by_type` (line 1)
- `private fn format_item_groups` (line 1)
- `private fn format_item_group` (line 1)
- `private fn format_module_item` (line 1)
- `private fn format_struct_item` (line 1)
- `private fn format_enum_item` (line 1)
- `private fn format_trait_item` (line 1)
- `private fn format_function_item` (line 1)
- `private fn format_impl_item` (line 1)
- `private fn format_footer` (line 1)
- `pub fn format_deep_context_as_markdown` (line 1)
- `private fn format_quality_scorecard` (line 1)
- `private fn format_project_summary` (line 1)
- `private fn format_analysis_results` (line 1)
- `private fn format_ast_summary` (line 1)
- `private fn count_ast_items` (line 1)
- `pub async fn analyze_project_with_progressive_analyzer` (line 1)

**Implementations:**
- `impl AstItem` (line 1)
- `impl RustVisitor` (line 1)
- `impl Visit for RustVisitor` (line 1)

### ./src/services/mod.rs

**Modules:**
- `pub mod artifact_writer` (line 1)
- `pub mod ast_python` (line 1)
- `pub mod ast_rust` (line 1)
- `pub mod ast_typescript` (line 1)
- `pub mod cache` (line 1)
- `pub mod canonical_query` (line 1)
- `pub mod code_intelligence` (line 1)
- `pub mod complexity` (line 1)
- `pub mod context` (line 1)
- `pub mod dag_builder` (line 1)
- `pub mod dead_code_analyzer` (line 1)
- `pub mod deep_context` (line 1)
- `pub mod defect_probability` (line 1)
- `pub mod deterministic_mermaid_engine` (line 1)
- `pub mod dogfooding_engine` (line 1)
- `pub mod duplicate_detector` (line 1)
- `pub mod embedded_templates` (line 1)
- `pub mod git_analysis` (line 1)
- `pub mod mermaid_generator` (line 1)
- `pub mod polyglot_detector` (line 1)
- `pub mod progressive_analyzer` (line 1)
- `pub mod ranking` (line 1)
- `pub mod relevance_scorer` (line 1)
- `pub mod renderer` (line 1)
- `pub mod satd_detector` (line 1)
- `pub mod smart_defaults` (line 1)
- `pub mod template_service` (line 1)
- `pub mod unified_ast_engine` (line 1)
- `pub mod universal_output_adapter` (line 1)

### ./src/services/mermaid_property_tests.rs

**Modules:**
- `private mod tests` (line 1)

### ./src/services/deep_context.rs

**Structs:**
- `pub struct DeepContextConfig` (9 fields) (line 1)
- `pub struct ComplexityThresholds` (2 fields) (line 1)
- `pub struct DeepContext` (8 fields) (line 1)
- `pub struct ContextMetadata` (5 fields) (line 1)
- `pub struct CacheStats` (3 fields) (line 1)
- `pub struct AnnotatedFileTree` (3 fields) (line 1)
- `pub struct AnnotatedNode` (5 fields) (line 1)
- `pub struct NodeAnnotations` (5 fields) (line 1)
- `pub struct AnalysisResults` (7 fields) (line 1)
- `pub struct EnhancedFileContext` (5 fields) (line 1)
- `pub struct FileChurnMetrics` (5 fields) (line 1)
- `pub struct DefectAnnotations` (4 fields) (line 1)
- `pub struct DeadCodeAnnotation` (3 fields) (line 1)
- `pub struct DeadCodeItem` (4 fields) (line 1)
- `pub struct TechnicalDebtItem` (5 fields) (line 1)
- `pub struct ComplexityViolation` (5 fields) (line 1)
- `pub struct CrossLangReference` (4 fields) (line 1)
- `pub struct QualityScorecard` (6 fields) (line 1)
- `pub struct TemplateProvenance` (4 fields) (line 1)
- `pub struct DriftAnalysis` (4 fields) (line 1)
- `pub struct DefectSummary` (4 fields) (line 1)
- `pub struct DefectHotspot` (4 fields) (line 1)
- `pub struct FileLocation` (3 fields) (line 1)
- `pub struct RefactoringEstimate` (4 fields) (line 1)
- `pub struct PrioritizedRecommendation` (6 fields) (line 1)
- `pub struct DeepContextAnalyzer` (2 fields) (line 1)
- `private struct ParallelAnalysisResults` (6 fields) (line 1)

**Enums:**
- `pub enum AnalysisType` (7 variants) (line 1)
- `pub enum DagType` (4 variants) (line 1)
- `pub enum CacheStrategy` (3 variants) (line 1)
- `pub enum NodeType` (2 variants) (line 1)
- `pub enum ConfidenceLevel` (3 variants) (line 1)
- `pub enum DeadCodeItemType` (4 variants) (line 1)
- `pub enum TechnicalDebtCategory` (5 variants) (line 1)
- `pub enum TechnicalDebtSeverity` (4 variants) (line 1)
- `pub enum ComplexityMetricType` (3 variants) (line 1)
- `pub enum CrossLangReferenceType` (4 variants) (line 1)
- `pub enum DefectFactor` (4 variants) (line 1)
- `pub enum Priority` (4 variants) (line 1)
- `pub enum Impact` (3 variants) (line 1)
- `private enum AnalysisResult` (5 variants) (line 1)

**Functions:**
- `private async fn analyze_ast_contexts` (line 1)
- `private async fn analyze_complexity` (line 1)
- `private async fn analyze_rust_file_complexity` (line 1)
- `private async fn analyze_churn` (line 1)
- `private async fn analyze_dead_code` (line 1)
- `private async fn analyze_satd` (line 1)

**Implementations:**
- `impl Default for DeepContextConfig` (line 1)
- `impl DeepContextAnalyzer` (line 1)

### ./src/services/ranking.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct RankingEngine` (2 fields) (line 1)
- `pub struct CompositeComplexityScore` (5 fields) (line 1)
- `pub struct ChurnScore` (5 fields) (line 1)
- `pub struct DuplicationScore` (6 fields) (line 1)
- `pub struct ComplexityRanker` (3 fields) (line 1)

**Traits:**
- `pub trait FileRanker` (line 1)

**Functions:**
- `pub fn rank_files_vectorized` (line 1)
- `pub fn rank_files_by_complexity` (line 1)

**Implementations:**
- `impl RankingEngine` (line 1)
- `impl Default for CompositeComplexityScore` (line 1)
- `impl PartialEq for CompositeComplexityScore` (line 1)
- `impl PartialOrd for CompositeComplexityScore` (line 1)
- `impl Default for ChurnScore` (line 1)
- `impl PartialEq for ChurnScore` (line 1)
- `impl PartialOrd for ChurnScore` (line 1)
- `impl Default for DuplicationScore` (line 1)
- `impl PartialEq for DuplicationScore` (line 1)
- `impl PartialOrd for DuplicationScore` (line 1)
- `impl Default for ComplexityRanker` (line 1)
- `impl ComplexityRanker` (line 1)
- `impl FileRanker for ComplexityRanker` (line 1)

### ./src/services/complexity.rs

**Structs:**
- `pub struct ComplexityMetrics` (4 fields) (line 1)
- `pub struct FileComplexityMetrics` (4 fields) (line 1)
- `pub struct FunctionComplexity` (4 fields) (line 1)
- `pub struct ClassComplexity` (5 fields) (line 1)
- `pub struct ComplexityThresholds` (6 fields) (line 1)
- `pub struct ComplexitySummary` (9 fields) (line 1)
- `pub struct ComplexityHotspot` (5 fields) (line 1)
- `pub struct ComplexityReport` (4 fields) (line 1)
- `pub struct ComplexityVisitor` (5 fields) (line 1)
- `pub struct CyclomaticComplexityRule` (2 fields) (line 1)
- `pub struct CognitiveComplexityRule` (2 fields) (line 1)

**Enums:**
- `pub enum Violation` (2 variants) (line 1)

**Traits:**
- `pub trait ComplexityRule` (line 1)

**Functions:**
- `pub fn compute_complexity_cache_key` (line 1)
- `pub fn aggregate_results` (line 1)
- `pub fn format_complexity_summary` (line 1)
- `pub fn format_complexity_report` (line 1)
- `pub fn format_as_sarif` (line 1)

**Implementations:**
- `impl Default for ComplexityThresholds` (line 1)
- `impl ComplexityVisitor` (line 1)
- `impl CyclomaticComplexityRule` (line 1)
- `impl ComplexityRule for CyclomaticComplexityRule` (line 1)
- `impl CognitiveComplexityRule` (line 1)
- `impl ComplexityRule for CognitiveComplexityRule` (line 1)

### ./src/services/deterministic_mermaid_engine.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct DeterministicMermaidEngine` (2 fields) (line 1)

**Enums:**
- `private enum ComplexityBucket` (3 variants) (line 1)

**Implementations:**
- `impl Default for DeterministicMermaidEngine` (line 1)
- `impl DeterministicMermaidEngine` (line 1)

### ./src/services/ast_typescript.rs

**Structs:**
- `private struct TypeScriptVisitor` (10 fields) (line 1)

**Functions:**
- `pub async fn analyze_typescript_file_with_complexity` (line 1)
- `pub async fn analyze_typescript_file_with_complexity_cached` (line 1)
- `pub async fn analyze_javascript_file_with_complexity` (line 1)
- `pub async fn analyze_typescript_file` (line 1)
- `pub async fn analyze_javascript_file` (line 1)

**Implementations:**
- `impl TypeScriptVisitor` (line 1)
- `impl Visit for TypeScriptVisitor` (line 1)

### ./src/services/ast_rust.rs

**Structs:**
- `private struct RustComplexityVisitor` (10 fields) (line 1)

**Functions:**
- `pub async fn analyze_rust_file_with_complexity` (line 1)
- `pub async fn analyze_rust_file` (line 1)

**Implementations:**
- `impl RustComplexityVisitor` (line 1)
- `impl Visit for RustComplexityVisitor` (line 1)

### ./src/services/duplicate_detector.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct VectorizedLSH` (4 fields) (line 1)
- `pub struct ANNIndex` (2 fields) (line 1)
- `pub struct UniversalFeatureExtractor` (3 fields) (line 1)
- `pub struct CloneReport` (5 fields) (line 1)
- `pub struct CloneGroup` (4 fields) (line 1)
- `pub struct CloneInstance` (5 fields) (line 1)
- `pub struct CloneSummary` (4 fields) (line 1)
- `pub struct DuplicateDetector` (5 fields) (line 1)

**Enums:**
- `pub enum CloneType` (4 variants) (line 1)

**Functions:**
- `private fn euclidean_distance` (line 1)
- `private fn compute_rabin_fingerprint` (line 1)
- `private fn compute_alpha_normalized_hash` (line 1)
- `private fn compute_minhash_signature` (line 1)

**Implementations:**
- `impl VectorizedLSH` (line 1)
- `impl Default for ANNIndex` (line 1)
- `impl ANNIndex` (line 1)
- `impl Default for UniversalFeatureExtractor` (line 1)
- `impl Default for DuplicateDetector` (line 1)
- `impl DuplicateDetector` (line 1)

### ./src/services/relevance_scorer.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct RelevanceScorer` (3 fields) (line 1)
- `pub struct NodeKey` (2 fields) (line 1)
- `pub struct ScoredItem` (4 fields) (line 1)
- `pub struct ScoringFactors` (6 fields) (line 1)

**Enums:**
- `pub enum ContextItem` (7 variants) (line 1)
- `pub enum EntryPointType` (5 variants) (line 1)
- `pub enum TestType` (5 variants) (line 1)
- `pub enum DocumentationType` (6 variants) (line 1)
- `pub enum ConfigurationType` (5 variants) (line 1)

**Functions:**
- `private fn extract_identifier_terms` (line 1)
- `private fn extract_doc_terms` (line 1)
- `private fn split_camel_case` (line 1)
- `private fn is_common_word` (line 1)

**Implementations:**
- `impl RelevanceScorer` (line 1)
- `impl ContextItem` (line 1)
- `impl Default for RelevanceScorer` (line 1)

### ./src/services/mermaid_generator.rs

**Modules:**
- `private mod tests` (line 1)
- `private mod property_tests` (line 1)

**Structs:**
- `pub struct MermaidGenerator` (1 fields) (line 1)
- `pub struct MermaidOptions` (4 fields) (line 1)

**Implementations:**
- `impl MermaidGenerator` (line 1)
- `impl Default for MermaidGenerator` (line 1)

### ./src/services/progressive_analyzer.rs

**Structs:**
- `pub struct AnalysisMetadata` (6 fields) (line 1)
- `pub struct AnalysisStage` (4 fields) (line 1)
- `pub struct StageContext` (10 fields) (line 1)
- `pub struct QuickProjectMetrics` (4 fields) (line 1)
- `private struct ThreadSafeSATDAnalyzer` (1 fields) (line 1)
- `pub struct ProgressiveAnalyzer` (3 fields) (line 1)
- `private struct OrderedHotspot` (1 fields) (line 1)
- `private struct LanguageDetectionAnalyzer` (0 fields) (line 1)
- `private struct ProjectStructureAnalyzer` (0 fields) (line 1)
- `private struct QuickMetricsAnalyzer` (0 fields) (line 1)
- `private struct AstAnalysisStageAnalyzer` (0 fields) (line 1)
- `private struct GitAnalysisStageAnalyzer` (0 fields) (line 1)
- `private struct FilesystemTimestampAnalyzer` (0 fields) (line 1)
- `private struct ComplexityAnalysisStageAnalyzer` (0 fields) (line 1)
- `private struct DependencyGraphAnalyzer` (0 fields) (line 1)
- `private struct DeadCodeAnalysisStageAnalyzer` (0 fields) (line 1)
- `private struct SatdAnalysisStageAnalyzer` (1 fields) (line 1)

**Enums:**
- `pub enum FailureMode` (3 variants) (line 1)
- `pub enum ProjectSize` (4 variants) (line 1)

**Traits:**
- `pub trait StageAnalyzer` (line 1)

**Functions:**
- `private fn build_file_tree_bounded` (line 1)
- `private fn should_exclude_path` (line 1)
- `private fn calculate_quick_metrics` (line 1)
- `private async fn analyze_rust_ast` (line 1)
- `private async fn analyze_typescript_ast` (line 1)
- `private async fn analyze_python_ast` (line 1)
- `private async fn analyze_rust_complexity` (line 1)

**Implementations:**
- `impl Default for AnalysisMetadata` (line 1)
- `impl Default for StageContext` (line 1)
- `impl ThreadSafeSATDAnalyzer` (line 1)
- `impl ProgressiveAnalyzer` (line 1)
- `impl PartialEq for OrderedHotspot` (line 1)
- `impl Eq for OrderedHotspot` (line 1)
- `impl PartialOrd for OrderedHotspot` (line 1)
- `impl Ord for OrderedHotspot` (line 1)
- `impl StageAnalyzer for LanguageDetectionAnalyzer` (line 1)
- `impl StageAnalyzer for ProjectStructureAnalyzer` (line 1)
- `impl StageAnalyzer for QuickMetricsAnalyzer` (line 1)
- `impl StageAnalyzer for AstAnalysisStageAnalyzer` (line 1)
- `impl StageAnalyzer for GitAnalysisStageAnalyzer` (line 1)
- `impl StageAnalyzer for FilesystemTimestampAnalyzer` (line 1)
- `impl StageAnalyzer for ComplexityAnalysisStageAnalyzer` (line 1)
- `impl StageAnalyzer for DependencyGraphAnalyzer` (line 1)
- `impl StageAnalyzer for DeadCodeAnalysisStageAnalyzer` (line 1)
- `impl SatdAnalysisStageAnalyzer` (line 1)
- `impl StageAnalyzer for SatdAnalysisStageAnalyzer` (line 1)
- `impl Default for ProgressiveAnalyzer` (line 1)

### ./src/services/satd_detector.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct SATDDetector` (2 fields) (line 1)
- `pub struct TechnicalDebt` (7 fields) (line 1)
- `pub struct SATDAnalysisResult` (4 fields) (line 1)
- `pub struct AstContext` (6 fields) (line 1)
- `pub struct DebtClassifier` (2 fields) (line 1)
- `private struct DebtPattern` (4 fields) (line 1)
- `pub struct DebtEvolution` (4 fields) (line 1)
- `pub struct SATDMetrics` (5 fields) (line 1)
- `pub struct CategoryMetrics` (3 fields) (line 1)

**Enums:**
- `pub enum DebtCategory` (6 variants) (line 1)
- `pub enum Severity` (4 variants) (line 1)
- `pub enum AstNodeType` (5 variants) (line 1)

**Implementations:**
- `impl DebtCategory` (line 1)
- `impl Display for DebtCategory` (line 1)
- `impl Severity` (line 1)
- `impl Default for DebtClassifier` (line 1)
- `impl DebtClassifier` (line 1)
- `impl Default for SATDDetector` (line 1)
- `impl SATDDetector` (line 1)

### ./src/services/dogfooding_engine.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct DogfoodingEngine` (1 fields) (line 1)
- `pub struct FileContext` (6 fields) (line 1)
- `pub struct ChurnMetrics` (5 fields) (line 1)
- `pub struct FileHotspot` (4 fields) (line 1)
- `pub struct DagMetrics` (6 fields) (line 1)

**Implementations:**
- `impl DogfoodingEngine` (line 1)
- `impl Default for DogfoodingEngine` (line 1)

### ./src/services/smart_defaults.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct DeepContextConfig` (9 fields) (line 1)
- `pub struct QualityThresholds` (4 fields) (line 1)
- `pub struct EnvironmentHints` (8 fields) (line 1)
- `pub struct SmartDefaults` (1 fields) (line 1)
- `private struct ProjectSizeHeuristic` (0 fields) (line 1)
- `private struct EnvironmentHeuristic` (0 fields) (line 1)
- `private struct PerformanceHeuristic` (0 fields) (line 1)
- `private struct OutputFormatHeuristic` (0 fields) (line 1)
- `private struct MonorepoHeuristic` (0 fields) (line 1)
- `private struct CIOptimizationHeuristic` (0 fields) (line 1)
- `private struct ResourceConstraintHeuristic` (0 fields) (line 1)

**Enums:**
- `pub enum AnalysisDepth` (4 variants) (line 1)
- `pub enum OutputFormat` (5 variants) (line 1)
- `pub enum OutputSize` (4 variants) (line 1)
- `pub enum CacheStrategy` (4 variants) (line 1)

**Traits:**
- `pub trait ConfigHeuristic` (line 1)

**Implementations:**
- `impl SmartDefaults` (line 1)
- `impl Default for DeepContextConfig` (line 1)
- `impl ConfigHeuristic for ProjectSizeHeuristic` (line 1)
- `impl ConfigHeuristic for EnvironmentHeuristic` (line 1)
- `impl ConfigHeuristic for PerformanceHeuristic` (line 1)
- `impl ConfigHeuristic for OutputFormatHeuristic` (line 1)
- `impl ConfigHeuristic for MonorepoHeuristic` (line 1)
- `impl MonorepoHeuristic` (line 1)
- `impl ConfigHeuristic for CIOptimizationHeuristic` (line 1)
- `impl ConfigHeuristic for ResourceConstraintHeuristic` (line 1)
- `impl Default for SmartDefaults` (line 1)

### ./src/services/dead_code_analyzer.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct HierarchicalBitSet` (2 fields) (line 1)
- `pub struct CrossLangReferenceGraph` (3 fields) (line 1)
- `pub struct ReferenceEdge` (4 fields) (line 1)
- `pub struct ReferenceNode` (3 fields) (line 1)
- `pub struct VTableResolver` (2 fields) (line 1)
- `private struct VTable` (2 fields) (line 1)
- `pub struct CoverageData` (2 fields) (line 1)
- `pub struct DeadCodeReport` (5 fields) (line 1)
- `pub struct DeadCodeItem` (7 fields) (line 1)
- `pub struct UnreachableBlock` (4 fields) (line 1)
- `pub struct DeadCodeSummary` (4 fields) (line 1)
- `pub struct DeadCodeAnalyzer` (5 fields) (line 1)

**Enums:**
- `pub enum ReferenceType` (6 variants) (line 1)
- `pub enum DeadCodeType` (5 variants) (line 1)

**Implementations:**
- `impl HierarchicalBitSet` (line 1)
- `impl Default for VTableResolver` (line 1)
- `impl VTableResolver` (line 1)
- `impl DeadCodeAnalyzer` (line 1)
- `impl CrossLangReferenceGraph` (line 1)

### ./src/services/template_service.rs

**Structs:**
- `pub struct ScaffoldResult` (2 fields) (line 1)
- `pub struct GeneratedFile` (3 fields) (line 1)
- `pub struct ScaffoldError` (2 fields) (line 1)
- `pub struct SearchResult` (3 fields) (line 1)
- `pub struct ValidationResult` (2 fields) (line 1)
- `pub struct ValidationError` (2 fields) (line 1)

**Functions:**
- `pub async fn get_template_content` (line 1)
- `pub async fn generate_template` (line 1)
- `private async fn generate_context` (line 1)
- `pub async fn list_templates` (line 1)
- `pub async fn list_all_resources` (line 1)
- `private fn parse_template_uri` (line 1)
- `private fn build_template_prefix` (line 1)
- `private fn extract_filename` (line 1)
- `private fn validate_parameters` (line 1)
- `pub async fn scaffold_project` (line 1)
- `pub async fn search_templates` (line 1)
- `pub async fn validate_template` (line 1)

### ./src/services/artifact_writer.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct ArtifactWriter` (2 fields) (line 1)
- `pub struct ArtifactMetadata` (5 fields) (line 1)
- `pub struct VerificationReport` (4 fields) (line 1)
- `pub struct IntegrityFailure` (3 fields) (line 1)
- `pub struct ArtifactStatistics` (5 fields) (line 1)
- `pub struct TypeStatistics` (2 fields) (line 1)
- `pub struct CleanupReport` (2 fields) (line 1)

**Enums:**
- `pub enum ArtifactType` (5 variants) (line 1)

**Implementations:**
- `impl ArtifactWriter` (line 1)

### ./src/services/polyglot_detector.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct LanguageWeight` (4 fields) (line 1)
- `pub struct PolyglotDetector` (2 fields) (line 1)
- `pub struct BuildFileDetector` (0 fields) (line 1)
- `pub struct ExtensionBasedDetector` (0 fields) (line 1)
- `pub struct ContentBasedDetector` (0 fields) (line 1)

**Enums:**
- `pub enum Language` (9 variants) (line 1)

**Traits:**
- `pub trait DetectionStrategy` (line 1)

**Implementations:**
- `impl PolyglotDetector` (line 1)
- `impl DetectionStrategy for BuildFileDetector` (line 1)
- `impl DetectionStrategy for ExtensionBasedDetector` (line 1)
- `impl DetectionStrategy for ContentBasedDetector` (line 1)
- `impl Default for PolyglotDetector` (line 1)
- `impl Display for Language` (line 1)

### ./src/services/git_analysis.rs

**Structs:**
- `pub struct GitAnalysisService` (0 fields) (line 1)
- `private struct FileStats` (6 fields) (line 1)
- `private struct CommitInfo` (3 fields) (line 1)

**Implementations:**
- `impl GitAnalysisService` (line 1)

### ./src/services/embedded_templates.rs

**Structs:**
- `private struct EmbeddedTemplateMetadata` (7 fields) (line 1)
- `private struct EmbeddedParameter` (5 fields) (line 1)

**Functions:**
- `private fn convert_to_template_resource` (line 1)
- `private fn parse_template_category` (line 1)
- `private fn parse_toolchain` (line 1)
- `private fn convert_embedded_parameters` (line 1)
- `private fn convert_embedded_parameter` (line 1)
- `private fn parse_parameter_type` (line 1)
- `private fn convert_json_value_to_string` (line 1)
- `private fn build_s3_object_key` (line 1)
- `private fn get_category_path` (line 1)
- `pub async fn list_templates` (line 1)
- `pub async fn get_template_metadata` (line 1)
- `pub async fn get_template_content` (line 1)

### ./src/services/old_cache.rs

**Functions:**
- `pub async fn get_metadata` (line 1)
- `pub async fn put_metadata` (line 1)
- `pub async fn get_content` (line 1)
- `pub async fn put_content` (line 1)

### ./src/services/cache/strategies.rs

**Structs:**
- `pub struct AstCacheStrategy` (0 fields) (line 1)
- `pub struct TemplateCacheStrategy` (0 fields) (line 1)
- `pub struct DagCacheStrategy` (0 fields) (line 1)
- `pub struct ChurnCacheStrategy` (0 fields) (line 1)
- `pub struct GitStatsCacheStrategy` (0 fields) (line 1)
- `pub struct GitStats` (4 fields) (line 1)

**Implementations:**
- `impl CacheStrategy for AstCacheStrategy` (line 1)
- `impl CacheStrategy for TemplateCacheStrategy` (line 1)
- `impl CacheStrategy for DagCacheStrategy` (line 1)
- `impl CacheStrategy for ChurnCacheStrategy` (line 1)
- `impl ChurnCacheStrategy` (line 1)
- `impl CacheStrategy for GitStatsCacheStrategy` (line 1)
- `impl GitStatsCacheStrategy` (line 1)

### ./src/services/cache/persistent_manager.rs

**Structs:**
- `pub struct PersistentCacheManager` (5 fields) (line 1)

**Implementations:**
- `impl PersistentCacheManager` (line 1)

### ./src/services/cache/manager.rs

**Structs:**
- `pub struct SessionCacheManager` (8 fields) (line 1)

**Implementations:**
- `impl SessionCacheManager` (line 1)
- `impl Send for SessionCacheManager` (line 1)
- `impl Sync for SessionCacheManager` (line 1)

### ./src/services/cache/mod.rs

**Modules:**
- `pub mod base` (line 1)
- `pub mod cache_trait` (line 1)
- `pub mod config` (line 1)
- `pub mod content_cache` (line 1)
- `pub mod diagnostics` (line 1)
- `pub mod manager` (line 1)
- `pub mod persistent` (line 1)
- `pub mod persistent_manager` (line 1)
- `pub mod strategies` (line 1)

### ./src/services/cache/content_cache.rs

**Structs:**
- `pub struct ContentCache` (4 fields) (line 1)
- `pub struct CacheMetrics` (5 fields) (line 1)

**Implementations:**
- `impl ContentCache` (line 1)
- `impl CacheMetrics` (line 1)
- `impl Clone for ContentCache` (line 1)

### ./src/services/cache/base.rs

**Structs:**
- `pub struct CacheEntry` (5 fields) (line 1)
- `pub struct CacheStats` (4 fields) (line 1)

**Traits:**
- `pub trait CacheStrategy` (line 1)

**Implementations:**
- `impl CacheEntry` (line 1)
- `impl CacheStats` (line 1)
- `impl Default for CacheStats` (line 1)

### ./src/services/cache/diagnostics.rs

**Structs:**
- `pub struct CacheDiagnostics` (7 fields) (line 1)
- `pub struct CacheStatsSnapshot` (6 fields) (line 1)
- `pub struct CacheEffectiveness` (4 fields) (line 1)
- `pub struct CacheDiagnosticReport` (3 fields) (line 1)

**Functions:**
- `pub fn format_prometheus_metrics` (line 1)

**Implementations:**
- `impl From for CacheStatsSnapshot` (line 1)
- `impl CacheDiagnosticReport` (line 1)

### ./src/services/cache/cache_trait.rs

**Traits:**
- `pub trait AstCacheManager` (line 1)

### ./src/services/cache/config.rs

**Structs:**
- `pub struct CacheConfig` (14 fields) (line 1)

**Implementations:**
- `impl Default for CacheConfig` (line 1)
- `impl CacheConfig` (line 1)

### ./src/services/cache/persistent.rs

**Structs:**
- `private struct PersistentCacheEntry` (3 fields) (line 1)
- `pub struct PersistentCache` (4 fields) (line 1)

**Implementations:**
- `impl PersistentCacheEntry` (line 1)
- `impl PersistentCache` (line 1)

### ./src/services/defect_probability.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct DefectProbabilityCalculator` (1 fields) (line 1)
- `pub struct DefectWeights` (4 fields) (line 1)
- `pub struct FileMetrics` (9 fields) (line 1)
- `pub struct DefectScore` (5 fields) (line 1)
- `pub struct ProjectDefectAnalysis` (5 fields) (line 1)

**Enums:**
- `pub enum RiskLevel` (3 variants) (line 1)

**Functions:**
- `private fn interpolate_cdf` (line 1)

**Implementations:**
- `impl Default for DefectWeights` (line 1)
- `impl DefectProbabilityCalculator` (line 1)
- `impl ProjectDefectAnalysis` (line 1)
- `impl Default for DefectProbabilityCalculator` (line 1)

### ./src/services/renderer.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct TemplateRenderer` (1 fields) (line 1)

**Functions:**
- `pub fn render_template` (line 1)

**Implementations:**
- `impl TemplateRenderer` (line 1)

### ./src/services/dag_builder.rs

**Structs:**
- `pub struct DagBuilder` (3 fields) (line 1)

**Functions:**
- `pub fn filter_call_edges` (line 1)
- `pub fn filter_import_edges` (line 1)
- `pub fn filter_inheritance_edges` (line 1)

**Implementations:**
- `impl DagBuilder` (line 1)
- `impl Default for DagBuilder` (line 1)

### ./src/services/canonical_query.rs

**Structs:**
- `pub struct AnalysisContext` (5 fields) (line 1)
- `pub struct CallGraph` (2 fields) (line 1)
- `pub struct CallNode` (4 fields) (line 1)
- `pub struct CallEdge` (4 fields) (line 1)
- `pub struct QueryResult` (2 fields) (line 1)
- `pub struct GraphMetadata` (6 fields) (line 1)
- `pub struct SystemArchitectureQuery` (0 fields) (line 1)
- `pub struct Component` (6 fields) (line 1)
- `pub struct ComponentEdge` (4 fields) (line 1)
- `pub struct ComponentMetrics` (5 fields) (line 1)

**Enums:**
- `pub enum CallNodeType` (5 variants) (line 1)
- `pub enum CallEdgeType` (5 variants) (line 1)
- `pub enum ComponentEdgeType` (4 variants) (line 1)

**Traits:**
- `pub trait CanonicalQuery` (line 1)

**Functions:**
- `private fn detect_architectural_components` (line 1)
- `private fn infer_component_relationships` (line 1)
- `private fn aggregate_component_metrics` (line 1)
- `private fn generate_styled_architecture_diagram` (line 1)
- `private fn sanitize_component_id` (line 1)
- `private fn humanize_component_name` (line 1)
- `private fn collect_component_nodes` (line 1)
- `private fn merge_coupled_components` (line 1)
- `private fn calculate_graph_diameter` (line 1)

**Implementations:**
- `impl CanonicalQuery for SystemArchitectureQuery` (line 1)
- `impl Clone for ComponentEdgeType` (line 1)
- `impl Hash for ComponentEdgeType` (line 1)
- `impl PartialEq for ComponentEdgeType` (line 1)
- `impl Eq for ComponentEdgeType` (line 1)

### ./src/services/unified_ast_engine.rs

**Modules:**
- `private mod tests` (line 1)

**Structs:**
- `pub struct UnifiedAstEngine` (2 fields) (line 1)
- `pub struct LanguageParsers` (0 fields) (line 1)
- `pub struct AstForest` (1 fields) (line 1)
- `pub struct ModuleNode` (4 fields) (line 1)
- `pub struct ModuleMetrics` (4 fields) (line 1)
- `pub struct ProjectMetrics` (4 fields) (line 1)
- `pub struct ArtifactTree` (3 fields) (line 1)
- `pub struct MermaidArtifacts` (2 fields) (line 1)
- `pub struct Template` (4 fields) (line 1)

**Enums:**
- `pub enum FileAst` (3 variants) (line 1)

**Implementations:**
- `impl Default for LanguageParsers` (line 1)
- `impl LanguageParsers` (line 1)
- `impl AstForest` (line 1)
- `impl Debug for FileAst` (line 1)
- `impl FileAst` (line 1)
- `impl UnifiedAstEngine` (line 1)
- `impl Default for UnifiedAstEngine` (line 1)

### ./src/services/ast_python.rs

**Structs:**
- `private struct PythonComplexityVisitor` (8 fields) (line 1)

**Functions:**
- `pub async fn analyze_python_file_with_complexity` (line 1)
- `pub async fn analyze_python_file` (line 1)
- `private fn extract_python_items` (line 1)

**Implementations:**
- `impl PythonComplexityVisitor` (line 1)

### ./build.rs

**Functions:**
- `private fn main` (line 1)
- `private fn verify_dependency_versions` (line 1)
- `private fn download_and_compress_assets` (line 1)
- `private fn setup_asset_directories` (line 1)
- `private fn get_asset_definitions` (line 1)
- `private fn process_assets` (line 1)
- `private fn should_skip_asset` (line 1)
- `private fn ensure_asset_downloaded` (line 1)
- `private fn download_asset` (line 1)
- `private fn handle_download_failure` (line 1)
- `private fn compress_asset` (line 1)
- `private fn create_compressed_data` (line 1)
- `private fn write_compressed_file` (line 1)
- `private fn set_asset_hash_env` (line 1)
- `private fn compress_templates` (line 1)
- `private fn collect_template_files` (line 1)
- `private fn read_template_file` (line 1)
- `private fn serde_json_to_string` (line 1)
- `private fn generate_hex_string` (line 1)
- `private fn generate_template_code` (line 1)
- `private fn minify_demo_assets` (line 1)
- `private fn minify_js_file` (line 1)
- `private fn minify_css_file` (line 1)
- `private fn copy_demo_asset` (line 1)
- `private fn simple_js_minify` (line 1)
- `private fn simple_css_minify` (line 1)
- `private fn calculate_asset_hash` (line 1)

### ./tests/demo_integration.rs

**Modules:**
- `private mod demo_tests` (line 1)

### ./tests/enhanced_dag_integration.rs

**Functions:**
- `private fn test_enhanced_dag_analysis` (line 1)
- `private fn test_enhanced_analysis_backward_compatibility` (line 1)
- `private fn test_enhanced_flags_combinations` (line 1)
- `private fn test_enhanced_dag_help` (line 1)

### ./tests/mermaid_empty_bug_fix_test.rs

**Functions:**
- `private fn test_regression_empty_nodes_bug` (line 1)
- `private fn test_mermaid_label_escaping` (line 1)
- `private fn test_node_types_have_labels` (line 1)
- `private fn test_complexity_styled_diagram_has_labels` (line 1)
- `private fn test_empty_graph_doesnt_crash` (line 1)
- `private fn test_special_characters_in_node_ids` (line 1)

### ./tests/cli_documentation_sync.rs

**Structs:**
- `private struct DocumentedCommand` (5 fields) (line 1)

**Functions:**
- `private fn parse_documented_cli_commands` (line 1)
- `private fn parse_cli_help_output` (line 1)
- `private fn get_binary_path` (line 1)
- `private fn test_cli_commands_match_documentation` (line 1)
- `private fn test_cli_subcommands_match_documentation` (line 1)
- `private fn test_cli_options_match_documentation` (line 1)
- `private fn test_no_undocumented_commands` (line 1)
- `private fn test_documentation_examples_are_valid` (line 1)

### ./tests/demo_web_integration.rs

**Modules:**
- `private mod demo_web_tests` (line 1)

**Functions:**
- `private async fn test_demo_server_startup_and_shutdown` (line 1)
- `private async fn test_demo_server_api_endpoints` (line 1)
- `private async fn test_demo_server_static_assets` (line 1)
- `private async fn test_demo_server_concurrent_requests` (line 1)
- `private async fn test_demo_server_response_headers` (line 1)
- `private async fn test_demo_content_rendering` (line 1)

### ./tests/determinism_tests.rs

**Functions:**
- `private async fn test_unified_ast_engine_determinism` (line 1)
- `private fn test_mermaid_generation_determinism` (line 1)
- `private async fn test_dogfooding_engine_determinism` (line 1)
- `private async fn test_artifact_writer_determinism` (line 1)
- `private fn test_pagerank_numerical_stability` (line 1)
- `private fn test_hash_collision_resistance` (line 1)
- `private async fn test_file_ordering_stability` (line 1)
- `private fn test_edge_case_determinism` (line 1)
- `private async fn test_concurrent_generation_determinism` (line 1)
- `private async fn create_test_project` (line 1)
- `private fn create_test_dependency_graph` (line 1)
- `private fn create_large_test_graph` (line 1)
- `private fn create_single_node_graph` (line 1)
- `private fn create_cyclic_graph` (line 1)
- `private fn create_test_artifact_tree` (line 1)
- `private fn compute_tree_hash` (line 1)
- `private fn normalize_manifest` (line 1)

### ./tests/mcp_documentation_sync.rs

**Structs:**
- `private struct DocumentedTool` (4 fields) (line 1)
- `private struct McpResponse` (4 fields) (line 1)
- `private struct ToolDefinition` (3 fields) (line 1)

**Functions:**
- `private fn parse_documented_mcp_tools` (line 1)
- `private fn get_binary_path` (line 1)
- `private fn send_mcp_request` (line 1)
- `private fn test_mcp_tools_match_documentation` (line 1)
- `private fn test_mcp_tool_schemas_match_documentation` (line 1)
- `private fn test_mcp_methods_match_documentation` (line 1)
- `private fn test_mcp_error_codes_are_complete` (line 1)
- `private fn test_no_undocumented_mcp_tools` (line 1)

### ./tests/generate_mermaid_test.rs

**Functions:**
- `private fn generate_test_mermaid` (line 1)

### ./tests/mermaid_artifact_tests.rs

**Structs:**
- `pub struct MermaidArtifactSpec` (5 fields) (line 1)
- `private struct DiagramMetrics` (4 fields) (line 1)

**Enums:**
- `pub enum ArtifactCategory` (4 variants) (line 1)

**Functions:**
- `private fn get_artifact_specs` (line 1)
- `private fn generate_simple_architecture` (line 1)
- `private fn generate_styled_workflow` (line 1)
- `private fn generate_ast_simple` (line 1)
- `private fn generate_ast_styled` (line 1)
- `private fn validate_simple_diagram` (line 1)
- `private fn validate_styled_diagram` (line 1)
- `private fn validate_ast_diagram` (line 1)
- `private fn validate_complexity_styled` (line 1)
- `private fn test_generate_all_artifacts` (line 1)
- `private fn test_maintain_mermaid_readme` (line 1)
- `private fn format_category_title` (line 1)
- `private fn analyze_diagram_metrics` (line 1)
- `private fn calculate_graph_depth` (line 1)

**Implementations:**
- `impl ArtifactCategory` (line 1)

### ./tests/execution_mode.rs

**Modules:**
- `private mod binary_main_tests` (line 1)

### ./tests/services_integration.rs

**Modules:**
- `private mod integration_coverage_tests` (line 1)

### ./tests/documentation_examples.rs

**Functions:**
- `private fn get_binary_path` (line 1)
- `private fn test_cli_examples_are_valid` (line 1)
- `private fn process_bash_code_block` (line 1)
- `private fn should_skip_line` (line 1)
- `private fn has_complex_shell_features` (line 1)
- `private fn is_non_toolkit_command` (line 1)
- `private fn handle_multiline_command` (line 1)
- `private fn validate_command` (line 1)
- `private fn validate_binary_path` (line 1)
- `private fn validate_command_arguments` (line 1)
- `private fn test_mcp_json_examples_are_valid` (line 1)
- `private fn validate_json_block` (line 1)
- `private fn validate_parsed_json` (line 1)
- `private fn validate_json_rpc_object` (line 1)
- `private fn validate_json_array_fallback` (line 1)
- `private fn validate_batch_request_array` (line 1)
- `private fn test_yaml_examples_are_valid` (line 1)
- `private fn test_jsonc_examples_are_valid` (line 1)
- `private fn test_template_uri_examples_are_valid` (line 1)
- `private fn test_performance_numbers_are_reasonable` (line 1)

### ./tests/complexity_metrics.rs

**Modules:**
- `private mod coverage_improvement` (line 1)

### ./tests/cli_comprehensive_integration.rs

**Functions:**
- `private fn test_generate_makefile_e2e` (line 1)
- `private fn test_generate_missing_required_params` (line 1)
- `private fn test_generate_invalid_template_uri` (line 1)
- `private fn test_generate_to_stdout` (line 1)
- `private fn test_scaffold_parallel_generation` (line 1)
- `private fn test_list_json_output_schema` (line 1)
- `private fn test_list_table_output` (line 1)
- `private fn test_list_yaml_output` (line 1)
- `private fn test_list_filtered_by_toolchain` (line 1)
- `private fn test_list_filtered_by_category` (line 1)
- `private fn test_search_basic` (line 1)
- `private fn test_search_with_limit` (line 1)
- `private fn test_search_with_toolchain_filter` (line 1)
- `private fn test_validate_success` (line 1)
- `private fn test_validate_missing_required` (line 1)
- `private fn test_context_generation_rust` (line 1)
- `private fn test_context_markdown_output` (line 1)
- `private fn test_analyze_churn_json_output` (line 1)
- `private fn test_analyze_churn_csv_output` (line 1)
- `private fn test_analyze_complexity_summary` (line 1)
- `private fn test_analyze_complexity_sarif_format` (line 1)
- `private fn test_analyze_dag_mermaid_output` (line 1)
- `private fn test_error_propagation_and_codes` (line 1)
- `private fn test_help_output` (line 1)
- `private fn test_version_output` (line 1)
- `private fn test_subcommand_help` (line 1)
- `private fn test_analyze_subcommand_help` (line 1)
- `private fn test_environment_variable_expansion` (line 1)
- `private fn test_mode_flag_cli` (line 1)

### ./tests/bin_integration.rs

**Functions:**
- `private fn test_binary_version_flag` (line 1)
- `private fn test_binary_json_rpc_initialize` (line 1)
- `private fn test_binary_invalid_json` (line 1)
- `private fn test_binary_multiple_requests` (line 1)

### ./tests/generate_mermaid_example.rs

**Functions:**
- `private fn generate_example_mermaid_diagram` (line 1)

### ./benches/critical_path.rs

**Functions:**
- `private fn benchmark_cli_parsing` (line 1)
- `private fn benchmark_template_generation` (line 1)
- `private fn benchmark_dag_generation` (line 1)
- `private fn benchmark_context_generation` (line 1)

---
Generated by paiml-mcp-agent-toolkit
