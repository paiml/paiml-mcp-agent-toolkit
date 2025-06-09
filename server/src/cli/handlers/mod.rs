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
pub mod enhanced_reporting_handlers;
pub mod generation_handlers;
pub mod name_similarity_analysis;
pub mod refactor_handlers;
pub mod utility_handlers;

// Re-export handler functions
pub use advanced_analysis_handlers::{
    handle_analyze_comprehensive, handle_analyze_deep_context, handle_analyze_defect_prediction,
    handle_analyze_makefile, handle_analyze_provability, handle_analyze_tdg,
};
pub use analysis_handlers::route_analyze_command;
pub use complexity_handlers::{
    handle_analyze_churn, handle_analyze_complexity, handle_analyze_dag, handle_analyze_dead_code,
    handle_analyze_satd,
};
pub use demo_handlers::{handle_demo, handle_quality_gate};
pub use duplication_analysis::handle_analyze_duplicates;
pub use generation_handlers::{handle_generate, handle_scaffold, handle_validate};
pub use name_similarity_analysis::handle_analyze_name_similarity;
pub use refactor_handlers::route_refactor_command;
pub use utility_handlers::{
    handle_context, handle_diagnose, handle_list, handle_search, handle_serve,
};
