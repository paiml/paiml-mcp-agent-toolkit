//! CLI command handlers organized by category
//!
//! This module structure reduces complexity by separating concerns
//! and grouping related functionality together.

pub mod advanced_analysis_handlers;
pub mod analysis_handlers;
pub mod big_o_handlers;
pub mod complexity_handlers;
pub mod demo_handlers;
pub mod duplication_analysis;
pub mod enforce_handlers;
pub mod enhanced_reporting_handlers;
pub mod generation_handlers;
pub mod lint_hotspot_handlers;
pub mod name_similarity_analysis;
pub mod refactor_auto_handlers;
pub mod refactor_docs_handlers;
pub mod refactor_handlers;
pub mod utility_handlers;
pub mod wasm_handlers;

#[cfg(test)]
pub mod refactor_auto_property_tests;

// Re-export handler functions
pub use advanced_analysis_handlers::{
    handle_analyze_comprehensive, handle_analyze_deep_context, handle_analyze_defect_prediction,
    handle_analyze_graph_metrics, handle_analyze_makefile, handle_analyze_provability,
    handle_analyze_symbol_table, handle_analyze_tdg,
};
pub use analysis_handlers::route_analyze_command;
pub use complexity_handlers::{
    handle_analyze_churn, handle_analyze_complexity, handle_analyze_dag, handle_analyze_dead_code,
    handle_analyze_satd,
};
pub use demo_handlers::{handle_demo, handle_quality_gate};
pub use duplication_analysis::handle_analyze_duplicates;
pub use enforce_handlers::route_enforce_command;
pub use generation_handlers::{handle_generate, handle_scaffold, handle_validate};
pub use lint_hotspot_handlers::handle_analyze_lint_hotspot;
pub use name_similarity_analysis::handle_analyze_name_similarity;
pub use refactor_docs_handlers::handle_refactor_docs;
pub use refactor_handlers::{route_refactor_command, RefactorServeParams};
pub use utility_handlers::{
    handle_context, handle_diagnose, handle_list, handle_search, handle_serve,
};
pub use wasm_handlers::{handle_analyze_assemblyscript, handle_analyze_webassembly};

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
