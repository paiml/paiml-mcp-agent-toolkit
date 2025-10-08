//! Tree-sitter based mutation operators for language-agnostic mutation testing
//!
//! EXTREME TDD: RED PHASE - Stub implementation, tests will fail
//!
//! This module provides a language-agnostic mutation operator trait that works
//! with tree-sitter ASTs instead of language-specific parsers like syn.

use super::types::SourceLocation;
use tree_sitter::Node;

/// A mutated source code variant
#[derive(Debug, Clone)]
pub struct MutatedSource {
    pub source: String,
    pub description: String,
    pub location: SourceLocation,
}

/// Trait for tree-sitter based mutation operators
///
/// Unlike the syn-based operators, these work on tree-sitter AST nodes
/// which are language-agnostic.
pub trait TreeSitterMutationOperator: Send + Sync {
    /// Name of this operator (e.g., "AOR", "ROR")
    fn name(&self) -> &str;

    /// Can this operator mutate the given AST node?
    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool;

    /// Generate mutants for the given node
    ///
    /// # Arguments
    /// * `node` - The AST node to mutate
    /// * `source` - The original source code as bytes
    ///
    /// # Returns
    /// Vector of mutated source variants
    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource>;

    /// Estimated kill probability (0.0 - 1.0)
    fn kill_probability(&self) -> f64 {
        0.5 // Default 50%
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: These tests will fail until implementation

    #[test]
    #[ignore] // RED: Will fail - not implemented yet
    fn red_test_source_location_has_line_and_column() {
        let loc = SourceLocation { line: 10, column: 5 };
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
    }

    #[test]
    #[ignore] // RED: Will fail - not implemented yet
    fn red_test_mutated_source_has_description() {
        let mutant = MutatedSource {
            source: "return a - b;".to_string(),
            description: "+ → -".to_string(),
            location: SourceLocation { line: 1, column: 10 },
        };

        assert_eq!(mutant.description, "+ → -");
        assert!(mutant.source.contains("a - b"));
    }
}
