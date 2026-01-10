//! Unified test modules - all 192 integration tests in one binary
//! This reduces coverage report time from 15+ min to <5 min
//! by compiling all tests into a single binary instead of 192 separate ones.

mod agent_integration_tests;
mod agent_mcp_server_tests;
mod analysis_timeout_test;
mod analysis_utilities_tests;
#[cfg(feature = "cli-integration")]
mod analyze_exit_status;
mod architecture_tests;
mod ast_dag_mermaid_pipeline;
mod bin_integration;
mod breakpoint_manager_tests;
mod bug_001_002_003_embed_tests;
mod bug_004_dead_code_multi_language_tests;
mod bug_005_progress_output_tests;
mod bug_006_parallel_count_tests;
mod bug_007_function_count_tests;
mod bug_008_placeholder_text_tests;
mod bug_009_copyright_tests;
mod bug_010_warning_display_tests;
mod bug_011_language_detection_tests;
mod bug_012_multi_language_cli_tests;
mod bug_064_file_safety_tests;
mod cargo_mutants_wrapper_tests;
mod claude_code_agent_mcp_server_test;
mod claude_skills_validation_tests;
#[cfg(feature = "cli-integration")]
mod cli_comprehensive_integration;
#[cfg(feature = "cli-integration")]
mod cli_context_tests;
#[cfg(feature = "cli-integration")]
mod cli_docs_enforcement;
mod cli_documentation_sync;
mod cli_functional_harness;
#[cfg(feature = "cli-integration")]
mod cli_semantic_integration;
#[cfg(feature = "cli-integration")]
mod cli_similarity_tests;
mod cli_smoke_test;
mod clippy_fix_engine_test;
mod code_quality_scorer_tests;
mod command_discoverability_test;
mod comparison_view_tests;
mod complexity_analyzer_accuracy_test;
mod complexity_analyzer_tests;
mod complexity_analyzer_validation_test;
mod complexity_bug_test;
mod complexity_metrics;
mod complexity_tests;
#[cfg(feature = "cli-integration")]
mod complexity_threshold_filtering;
#[cfg(feature = "cli-integration")]
mod comprehensive_assert_cmd_coverage;
mod config_command_test;
mod config_integration;
mod dap_integration_tests;
mod dap_recording_capture_tests;
mod dap_server_tests;
mod dead_code_timeout_test;
mod debug_command_tests;
mod debug_replay_tests;
mod debug_serve_tests;
#[cfg(feature = "cli-integration")]
mod deep_context_cli_integration;
mod deep_context_tests;
mod deep_wasm_cli_tests;
mod defect_aware_prompts_integration;
mod defect_aware_prompts_real_world;
mod demo_core_extraction;
mod demo_e2e_integration;
#[cfg(feature = "cli-integration")]
mod demo_integration;
mod demo_web_integration;
mod dependency_duplicates_test;
mod dependency_scorer_tests;
mod determinism_tests;
mod docs_enforcement_quality_gate_test;
mod docs_enforcement_unit_test;
mod documentation_examples;
mod documentation_scorer_tests;
#[cfg(feature = "cli-integration")]
mod enhanced_dag_integration;
mod enhanced_reporting_refactor_test;
mod entropy_duplicate_test;
mod evidence_gatherer_tests;
mod execution_mode;
mod execution_recorder_integration_tests;
mod execution_recorder_tests;
mod export_integration;
mod feature_052_filtering_tests;
mod format_defect_markdown_test;
mod format_detailed_report_refactor_test;
mod format_output_symbol_table_test;
mod generate_mermaid_example;
mod generate_mermaid_test;
mod git_clone_validation;
mod hooks_command_test;
#[cfg(feature = "cli-integration")]
mod include_pattern_integration;
mod intent_classifier_tests;
mod is_excluded_filename_refactor_test;
mod issue_053_mcp_tool_placeholders;
mod issue_67_integration_test;
mod json_parsing_tests;
mod kotlin_ast_test;
mod kotlin_support_test;
mod mcp_docs_enforcement;
mod mcp_documentation_sync;
mod mcp_semantic_integration;
mod mcp_server_integration;
mod mcp_server_tests;
#[cfg(feature = "cli-integration")]
mod mcp_tool_composition;
mod mermaid_artifact_tests;
mod mermaid_empty_bug_fix_test;
mod mutate_command_tests;
mod mutation_cleanup_tests;
mod mutation_compilation_test;
mod mutation_generation_integration;
mod mutation_handler_unit_tests;
mod mutation_integration_tests;
mod mutation_property_tests;
mod name_similarity_refactor_test;
mod parallel_mutation_execution;
mod parse_sprint_section_refactor_test;
mod pdmt_integration_test;
mod performance_scorer_tests;
mod polyglot_integration;
mod polyglot_tools_tests;
mod predict_quality_integration_test;
mod progress_reporting_tests;
#[cfg(feature = "cli-integration")]
mod prompt_integration_tests;
mod provability_handler_refactor_test;
mod quality_gate_complexity_test;
#[cfg(feature = "cli-integration")]
mod quality_gate_integration;
mod quality_gate_integration_test;
mod quality_gate_tests;
mod quality_proxy_integration;
mod recording_format_tests;
mod recording_workflow_e2e_tests;
mod red_team;
mod red_team_cli_tests;
mod red_team_handler_tests;
mod red_team_integration_tests;
mod red_team_repository_context_tests;
#[cfg(feature = "cli-integration")]
mod refactor_auto_property_integration;
mod replay_engine_tests;
mod replay_integration_tests;
#[cfg(feature = "cli-integration")]
mod repo_score_cli_integration_tests;
mod roadmap_yaml_validator;
mod ruchy_entropy_integration_test;
mod ruchy_integration_standalone;
mod ruchy_integration_test;
mod ruchy_tdg_integration_test;
mod run_clippy_analysis_refactor_test;
mod run_mcp_server_refactor_test;
mod rust_project_score_orchestrator_tests;
mod rust_project_score_tests;
mod rust_tooling_scorer_tests;
mod satd_detector_tests;
mod scala_tools_tests;
mod services_integration;
mod slow_integration;
mod smart_test_filtering;
mod snapshot_manager_tests;
mod snapshot_serialization_tests;
mod sprint85_entropy_reduction_test;
mod stateless_server_test;
mod storage_backend_tests;
mod tdg_auto_fail_integration_test;
mod tdg_ci_integration_tests;
mod tdg_explain_mode_tests;
mod tdg_handlers_refactor_test;
mod tdg_hooks_tests;
mod tdg_score_storage_test;
mod tdg_storage_simple_test;
mod test_kotlin_direct;
mod test_kotlin_minimal;
mod testing_scorer_tests;
mod timeline_cli_integration_tests;
mod timeline_player_tests;
mod timeline_tui_cli_integration_tests;
mod timeline_tui_event_loop_tests;
mod timeline_tui_keyboard_tests;
mod timeline_tui_stack_frame_tests;
mod timeline_tui_variable_inspector_tests;
mod timeline_tui_visualization_tests;
mod timeline_ui_playback_tests;
mod timeline_ui_tests;
mod tool_functions_tests;
mod tools_coverage;
mod typescript_javascript_source_parsing;
mod unit_code_chunker;
mod unit_hybrid_search;
mod unit_kmeans_clustering;
mod unit_mcp_semantic_tools;
mod unit_semantic_search_engine;
mod unit_topic_modeling;
mod unit_turso_vector_db;
mod universal_demo_integration;
mod universal_demo_performance;
mod universal_demo_simple;
mod universal_demo_unit;
mod variable_diff_tests;
mod variable_inspector_tests;
mod wasm_cli_tests;
mod wasm_handlers_refactor_test;
mod webassembly_handler_refactor_test;
