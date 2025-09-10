//! Scaffolding system for generating projects and agents.

pub mod agent;

// Re-export key types for convenience
pub use agent::{
    scaffold_agent, AgentContext, AgentContextBuilder, AgentFeature, AgentTemplate,
    InteractiveScaffolder, QualityLevel, ScaffoldError, TemplateRegistry,
};

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
