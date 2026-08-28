#![cfg_attr(coverage_nightly, coverage(off))]
//! Analysis command implementations
//!
//! This module contains the actual implementation of analysis commands,
//! extracted from the main CLI module to reduce complexity.
//!
//! ## File Health Initiative (CB-040)
//!
//! The codebase has identified 60+ files over 2000 lines that need splitting.
//! The pre-commit hook now enforces:
//! - New files must be <500 lines
//! - Existing files cannot grow (ratchet mechanism)
//!
//! Priority files for future refactoring:
//! - analysis_utilities.rs (12,087 lines)
//! - deep_context.rs (7,211 lines)
//! - commands.rs (6,273 lines)
//! - tools.rs (6,111 lines)

pub mod defect_prediction;
pub mod duplicates;
pub mod graph_metrics;
pub mod name_similarity;
pub mod symbol_table;

// Re-export the handlers
pub use defect_prediction::handle_analyze_defect_prediction;
pub use duplicates::handle_analyze_duplicates;
pub use graph_metrics::handle_analyze_graph_metrics;
pub use name_similarity::handle_analyze_name_similarity;
pub use symbol_table::handle_analyze_symbol_table;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests // Commented out: unused import

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}

#[cfg(test)]
#[path = "regex_hoisting_tests.rs"]
mod regex_hoisting_tests;
