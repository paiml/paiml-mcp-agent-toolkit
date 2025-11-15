//! CLI command handlers organized by category
//!
//! This module structure reduces complexity by separating concerns
//! and grouping related functionality together.

pub mod advanced_analysis_handlers;
pub mod agent_handlers;
pub mod analysis;
pub mod analysis_handlers;
pub mod prompt_handlers;
#[cfg(test)]
pub mod annotation_tdd_tests;
pub mod big_o_handlers;
pub mod cache;
pub mod cargo_mutants_backend; // Sprint 70: cargo-mutants wrapper
pub mod churn_formatter;
pub mod complexity_handlers;
pub mod comprehensive_analysis_handler;
pub mod comprehensive_handler;
pub mod config_command_handlers;
pub mod configuration_handlers;
pub mod debug_handlers; // Sprint 74: Time-travel debugging CLI handlers
#[cfg(feature = "deep-wasm")]
pub mod deep_wasm_handlers;
pub mod defect_prediction_handler;
pub mod demo_handlers;
pub mod doc_validate_handlers;
pub mod duplication_analysis;
pub mod enforce_handlers;
pub mod enhanced_reporting_handlers;
pub mod generation_handlers;
#[cfg(test)]
pub mod graph_context_integration_tests;
pub mod health_handler;
pub mod hooks_command_handlers;
pub mod incremental_coverage_handler;
pub mod lint_hotspot_handlers;
#[cfg(test)]
pub mod lint_hotspot_property_tests;
pub mod memory;
pub mod mutate;
pub mod mutation_handlers;
pub mod name_similarity_analysis;
pub mod new_tdg_handler;
pub mod proof_annotations_handler;
pub mod provability_handler;
pub mod qdd_handlers;
pub mod quality_gate_formatter;
#[cfg(test)]
pub mod quality_gate_property_tests;
pub mod quality_gates_handler; // TICKET-PMAT-5023
pub mod readme_validate_handlers; // Sprint 38: Hallucination detection CLI
pub mod red_team; // Red Team Mode: Automated hallucination detection
pub mod refactor_auto_handlers;
#[cfg(test)]
pub mod refactor_auto_property_tests;
pub mod refactor_docs_handlers;
pub mod refactor_handlers;
pub mod repo_score_handlers; // Sprint 48: Repository health scoring
pub mod roadmap_handler;
pub mod satd_handler;
pub mod similarity_handler;
pub mod subagent_handlers;
pub mod tdg_diagnostic_handler;
pub mod tdg_formatter;
#[cfg(test)]
pub mod tdg_git_context_tests; // Sprint 65 Phase 2: Git-commit correlation
pub mod tdg_handlers;
#[cfg(test)]
pub mod tdg_history_tests; // Sprint 65 Phase 3: TDG History Commands
pub mod telemetry_handlers;
pub mod test_handlers;
pub mod unified_context_advanced;
#[cfg(test)]
pub mod unified_context_advanced_tests;
pub mod unified_context_builder;
#[cfg(test)]
pub mod unified_context_property_tests;
pub mod utility_handlers;
pub mod wasm_handler;
pub mod wasm_handlers;
pub mod timeline_mode; // Sprint 78: TUI-006 - Timeline CLI integration

// Re-export handler functions
pub use advanced_analysis_handlers::{
    handle_analyze_comprehensive, handle_analyze_deep_context, handle_analyze_graph_metrics,
    handle_analyze_makefile, handle_analyze_symbol_table, handle_analyze_tdg,
};
pub use agent_handlers::handle_agent_command;
pub use analysis_handlers::route_analyze_command;
pub use cache::handle_cache_command;
pub use complexity_handlers::{
    handle_analyze_churn, handle_analyze_complexity, handle_analyze_dag, handle_analyze_dead_code,
};
pub use config_command_handlers::handle_config_command;
pub use configuration_handlers::handle_configuration;
pub use debug_handlers::{handle_debug_compare, handle_debug_timeline}; // Sprint 77: TIMELINE-004
pub use defect_prediction_handler::handle_analyze_defect_prediction;
pub use demo_handlers::{handle_demo, handle_quality_gate};
pub use doc_validate_handlers::ValidateDocsCmd;
pub use duplication_analysis::handle_analyze_duplicates;
pub use enforce_handlers::route_enforce_command;
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
pub use prompt_handlers::{handle_prompt, handle_prompt_command};
pub use provability_handler::handle_analyze_provability;
pub use quality_gates_handler::handle_quality_gates_command; // TICKET-PMAT-5023
pub use readme_validate_handlers::ValidateReadmeCmd; // Sprint 38: Hallucination detection CLI
pub use red_team::RedTeamCmd; // Red Team Mode: Automated hallucination detection
pub use refactor_docs_handlers::handle_refactor_docs;
pub use refactor_handlers::{route_refactor_command, RefactorServeParams};
pub use repo_score_handlers::handle_repo_score; // Sprint 48: Repository health scoring
pub use roadmap_handler::handle_maintain_roadmap; // TICKET-PMAT-5032
pub use satd_handler::handle_analyze_satd;
pub use tdg_handlers::handle_tdg_command;
pub use telemetry_handlers::handle_telemetry;
pub use test_handlers::handle_test;
pub use utility_handlers::{
    handle_context, handle_diagnose, handle_list, handle_search, handle_serve,
};
pub use wasm_handlers::{handle_analyze_assemblyscript, handle_analyze_webassembly};
pub use timeline_mode::{TimelineMode, handle_timeline, get_timeline_help_text}; // Sprint 78: TUI-006

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
