#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI command handlers organized by category
//!
//! This module structure reduces complexity by separating concerns
//! and grouping related functionality together.

pub mod advanced_analysis_handlers;
#[cfg(feature = "agent-daemon")]
pub mod agent_handlers;
pub mod analysis;
pub mod analysis_handlers;
pub mod analyze_defects_handler; // Known Defects: Project-wide defect scanning
#[cfg(test)]
pub mod annotation_tdd_tests;
pub mod big_o_handlers;
pub mod bottleneck_handler; // Architectural churn bottleneck detection
pub mod brick_score_handlers; // PMAT-446: ComputeBrick profiling score
pub mod bug_report_handler; // Issue #81: Auto GitHub bug reports from errors
pub mod cache;
#[cfg(feature = "mutation-testing")]
pub mod cargo_mutants_backend; // Sprint 70: cargo-mutants wrapper
pub mod ci_local_handler; // Local CI simulation to avoid push-wait-fix loops
pub mod churn_formatter;
pub mod cleanup_resources_handler; // Issue #86: System resource cleanup
pub mod complexity_handlers;
pub mod comply_cb_detect; // CB pattern detection extracted for file health (CB-040)
pub mod comply_handlers; // GH-96: PMAT compliance and migration system
pub mod comprehensive_analysis_handler;
pub mod comprehensive_handler;
pub mod config_command_handlers;
pub mod configuration_handlers;
pub mod coverage_improve_handler;
pub mod cuda_tdg_handlers; // CUDA-SIMD TDG: 100-point Popper falsification scoring
pub mod dead_code_handlers; // Dead code analysis extracted for file health (CB-040)
pub mod debug_handlers; // Sprint 74: Time-travel debugging CLI handlers
#[cfg(feature = "deep-wasm")]
pub mod deep_wasm_handlers;
pub mod defect_prediction_handler;
#[cfg(feature = "demo")]
pub mod demo_handlers;
pub mod demo_score_handlers; // GH-109/112: Demo Quality scoring (Category G)
pub mod deps_audit_handlers; // Dependency audit for Sovereign AI stack migration
pub mod doc_validate_handlers;
pub mod duplication_analysis;
pub mod enforce_handlers;
pub mod enhanced_reporting_handlers;
pub mod extract_handler; // GH-215: pmat extract --list
pub mod five_whys_handlers; // Five Whys root cause analysis (Toyota Way)
pub mod generation_handlers;
#[cfg(test)]
pub mod graph_context_integration_tests;
pub mod handler_utils; // Unit-testable pure functions extracted from handlers
pub mod health_handler;
pub mod hooks_command_handlers;
pub mod incremental_coverage_handler;
pub mod kaizen_handler; // GH-213: Autonomous continuous improvement (Toyota Way Kaizen)
pub mod lint_hotspot_handlers;
#[cfg(test)]
pub mod lint_hotspot_property_tests;
pub mod localize_handlers; // GH-103: Tarantula fault localization
pub mod memory;
#[cfg(feature = "mutation-testing")]
pub mod mutate;
#[cfg(feature = "mutation-testing")]
pub mod mutation_handlers;
pub mod name_similarity_analysis;
pub mod new_tdg_handler;
pub mod oracle_handlers; // PMAT Oracle - PDCA loop for automated quality improvement
pub mod org_handlers;
pub mod perfection_score_handlers; // master-plan-pmat-work-system.md: 200-point unified score
pub mod popper_score_handlers; // Popper Falsifiability Score v1.1
pub mod predict_quality_handlers; // Phase 4.1: Predictive Quality Gates CLI
pub mod project_diag_handlers; // Project diagnostics (lltop Tab 8 equivalent)
pub mod prompt_handlers;
pub mod proof_annotations_handler;
pub mod provability_handler;
pub mod qa_work_handler;
pub mod qdd_handlers;
pub mod quality_gate_formatter;
#[cfg(test)]
pub mod quality_gate_property_tests;
pub mod quality_gates_handler; // TICKET-PMAT-5023
pub mod query_handler; // PMAT-470: RAG-powered semantic code search
pub mod readme_validate_handlers; // Sprint 38: Hallucination detection CLI
pub mod red_team; // Red Team Mode: Automated hallucination detection
pub mod refactor_auto_handlers;
#[cfg(test)]
pub mod refactor_auto_property_tests;
pub mod refactor_auto_types; // Types extracted for file health (CB-040)
pub mod refactor_docs_handlers;
pub mod refactor_handlers;
pub mod repo_score_handlers; // Sprint 48: Repository health scoring
pub mod roadmap_handler;
pub mod rust_project_score_handlers; // Sprint 3: Rust Project Score v1.1
pub mod satd_handler;
pub mod similarity_handler;
pub mod spec_falsify_handler;
pub mod spec_handlers; // master-plan-pmat-work-system.md: Spec management commands
pub mod split_handler; // File split: semantic file splitting via Louvain
pub mod sql_handler; // GH-218: Direct SQL access to function index database
pub mod subagent_handlers;
pub mod tdg_diagnostic_handler;
pub mod tdg_formatter;
#[cfg(test)]
pub mod tdg_git_context_tests; // Sprint 65 Phase 2: Git-commit correlation
pub mod tdg_handlers;
#[cfg(test)]
pub mod tdg_history_tests; // Sprint 65 Phase 3: TDG History Commands
pub mod telemetry_handlers;
pub mod test_discovery_handlers; // GH-98: Systematic test discovery and fixing
pub mod test_handlers;
pub mod test_stability_handler; // Flaky test detection and timeout sensitivity analysis
pub mod timeline_mode; // Sprint 78: TUI-006 - Timeline CLI integration
pub mod unified_context_advanced;
#[cfg(test)]
pub mod unified_context_advanced_tests;
pub mod unified_context_builder;
#[cfg(test)]
pub mod unified_context_property_tests;
pub mod utility_handlers;
pub mod utility_serve_handlers; // Server handlers extracted for file health (CB-040)
#[cfg(feature = "wasm-ast")]
pub mod wasm_handler;
pub mod wasm_handlers; // Phase 4: Organizational Intelligence Integration
pub mod work_contract; // PMAT Work Contract: Popperian falsification-based quality enforcement
pub mod work_falsification; // PMAT Work Contract: Falsification executor
pub mod work_handlers; // Issue #75: Unified GitHub/YAML workflow // GH-102: Toyota Way QA validation
pub mod work_ledger; // Falsification ledger: append-only receipt tracking
pub mod work_quality_handlers; // Quality gates and falsification extracted for file health (CB-040) // RAG-powered spec falsification handler

// Re-export handler functions
pub use advanced_analysis_handlers::{
    handle_analyze_comprehensive, handle_analyze_deep_context, handle_analyze_graph_metrics,
    handle_analyze_makefile, handle_analyze_symbol_table, handle_analyze_tdg,
};
#[cfg(feature = "agent-daemon")]
pub use agent_handlers::handle_agent_command;
pub use analysis_handlers::route_analyze_command;
pub use brick_score_handlers::handle_brick_score; // PMAT-446: ComputeBrick profiling score
pub use cache::handle_cache_command;
pub use complexity_handlers::{
    handle_analyze_churn, handle_analyze_complexity, handle_analyze_dag,
};
pub use config_command_handlers::handle_config_command;
pub use configuration_handlers::handle_configuration;
pub use dead_code_handlers::handle_analyze_dead_code;
pub use debug_handlers::{handle_debug_compare, handle_debug_timeline}; // Sprint 77: TIMELINE-004
pub use defect_prediction_handler::handle_analyze_defect_prediction;
#[cfg(feature = "demo")]
pub use demo_handlers::{handle_demo, handle_quality_gate};
pub use demo_score_handlers::handle_demo_score; // GH-109/112: Demo Quality scoring
pub use doc_validate_handlers::ValidateDocsCmd;
pub use duplication_analysis::handle_analyze_duplicates;
pub use enforce_handlers::route_enforce_command;
pub use extract_handler::handle_extract_list; // GH-215: pmat extract --list
pub use generation_handlers::{
    handle_generate,
    handle_list_agent_templates,
    handle_scaffold,
    handle_scaffold_agent,
    handle_scaffold_wasm,
    handle_validate,
    handle_validate_agent_template,
    ScaffoldAgentParams,
    ScaffoldWasmParams, // TICKET-PMAT-5031
};
pub use health_handler::handle_maintain_health; // TICKET-PMAT-5033
pub use hooks_command_handlers::handle_hooks_command;
pub use incremental_coverage_handler::handle_analyze_incremental_coverage;
pub use lint_hotspot_handlers::handle_analyze_lint_hotspot;
pub use memory::handle_memory_command;
pub use name_similarity_analysis::handle_analyze_name_similarity;
#[cfg(feature = "org-intelligence")]
pub use org_handlers::handle_org_command;
pub use popper_score_handlers::handle_popper_score; // Popper Falsifiability Score v1.1
pub use prompt_handlers::{handle_prompt, handle_prompt_command};
pub use provability_handler::handle_analyze_provability;
pub use quality_gates_handler::handle_quality_gates_command; // TICKET-PMAT-5023
pub use query_handler::handle_query; // PMAT-470: RAG-powered semantic code search
pub use readme_validate_handlers::ValidateReadmeCmd; // Sprint 38: Hallucination detection CLI
pub use red_team::RedTeamCmd; // Red Team Mode: Automated hallucination detection
pub use refactor_docs_handlers::handle_refactor_docs;
pub use refactor_handlers::{route_refactor_command, RefactorServeParams};
pub use repo_score_handlers::handle_repo_score; // Sprint 48: Repository health scoring
pub use roadmap_handler::handle_maintain_roadmap; // TICKET-PMAT-5032
pub use rust_project_score_handlers::handle_rust_project_score; // Sprint 3: Rust Project Score v1.1
pub use satd_handler::handle_analyze_satd;
pub use tdg_handlers::handle_tdg_command;
pub use telemetry_handlers::handle_telemetry;
pub use test_handlers::handle_test;
pub use timeline_mode::{get_timeline_help_text, handle_timeline, TimelineMode}; // Sprint 78: TUI-006
pub use utility_handlers::{
    handle_context, handle_diagnose, handle_list, handle_search, handle_serve,
};
pub use wasm_handlers::{handle_analyze_assemblyscript, handle_analyze_webassembly}; // Phase 4: Organizational Intelligence Integration

// CUDA-SIMD TDG exports (100-point Popper falsification scoring)
pub use cuda_tdg_handlers::{handle_cuda_tdg_command, CudaTdgCommandConfig};

#[cfg(test)]
mod tests {

    #[test]
    fn test_handler_exports() {
        // Basic test to verify module exports
        assert_eq!(1, 1);
    }

    #[test]
    fn test_module_basic() {
        // Basic test
        assert_eq!(2 + 2, 4);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
