//! Analysis command implementations
//!
//! This module contains the actual implementation of analysis commands,
//! extracted from the main CLI module to reduce complexity.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
