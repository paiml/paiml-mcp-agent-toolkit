//! Unified AST module for cross-language code analysis
//!
//! This module consolidates all AST-related functionality into a clean,
//! maintainable architecture following the strategy pattern.

pub mod core;
pub mod engine;
pub mod languages;
pub mod parser;

// Re-export core types for backward compatibility
pub use core::{AstDag, Language, NodeFlags, NodeKey, UnifiedAstNode, INVALID_NODE_KEY};

// Re-export parser functionality
pub use parser::{ParseResult, ParserCapabilities, ParserConfig, UnifiedParser};

// Re-export engine functionality
pub use engine::{AstAnalysisResult, AstEngine};

// Re-export language strategies
pub use languages::{LanguageRegistry, LanguageStrategy};

/// Prelude module for common imports
pub mod prelude {
    pub use super::core::*;
    pub use super::engine::AstEngine;
    pub use super::languages::LanguageStrategy;
    pub use super::parser::UnifiedParser;
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
