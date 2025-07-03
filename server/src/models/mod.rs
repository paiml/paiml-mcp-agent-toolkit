pub mod churn;
pub mod complexity_bound;
pub mod dag;
#[cfg(test)]
pub mod dag_property_tests;
pub mod dead_code;
pub mod deep_context_config;
pub mod error;
pub mod mcp;
pub mod project_meta;
pub mod refactor;
pub mod tdg;
pub mod template;
pub mod unified_ast;

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
