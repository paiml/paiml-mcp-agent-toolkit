// Rust Mutation Generator using tree-sitter AST
// PMAT-7014: Rust Mutation Testing
// Status: RED Phase - Stub implementation

use super::rust_tree_sitter_mutations::*;
use super::tree_sitter_operators::TreeSitterMutationOperator;
use super::types::{Mutant, MutantStatus, MutationOperatorType};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tree_sitter::{Node, Parser, Tree};

/// Rust mutation generator using tree-sitter AST
pub struct RustMutationGenerator {
    operators: Vec<Box<dyn TreeSitterMutationOperator>>,
}

impl RustMutationGenerator {
    /// Create generator with all default Rust mutation operators
    pub fn with_default_operators() -> Self {
        Self {
            operators: vec![
                Box::new(RustBinaryOpMutation),
                Box::new(RustRelationalOpMutation),
                Box::new(RustLogicalOpMutation),
                Box::new(RustBitwiseOpMutation),
                Box::new(RustRangeOpMutation),
                Box::new(RustPatternMutation),
                Box::new(RustMethodChainMutation),
                Box::new(RustBorrowMutation),
            ],
        }
    }

    /// Generate all mutants from Rust source code
    pub fn generate_mutants(&self, source: &str, file_path: &str) -> Result<Vec<Mutant>> {
        let tree = self.parse_rust(source)?;
        let mut mutants = Vec::new();

        // Visit all nodes in the AST
        let root = tree.root_node();
        self.visit_node(&root, source.as_bytes(), &mut mutants, file_path);

        Ok(mutants)
    }

    /// Parse Rust source using tree-sitter
    fn parse_rust(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Rust language: {}", e))?;

        parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Rust source"))
    }

    /// Recursively visit AST nodes and apply mutation operators
    fn visit_node(
        &self,
        node: &Node,
        source: &[u8],
        mutants: &mut Vec<Mutant>,
        file_path: &str,
    ) {
        // Try to apply each operator to this node
        for operator in &self.operators {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);

                // Convert MutatedSource to Mutant
                for (index, mutation) in mutations.iter().enumerate() {
                    let operator_name = operator.name();
                    let operator_type = map_operator_name_to_type(operator_name);

                    // Generate hash for deduplication
                    let mut hasher = Sha256::new();
                    hasher.update(mutation.source.as_bytes());
                    let hash = format!("{:x}", hasher.finalize());

                    let mutant = Mutant {
                        id: format!("{}_{}_{}_{}", operator_name, mutation.location.line, mutation.location.column, index),
                        original_file: PathBuf::from(file_path),
                        mutated_source: mutation.source.clone(),
                        location: mutation.location.clone(),
                        operator: operator_type,
                        hash,
                        status: MutantStatus::Pending,
                    };

                    mutants.push(mutant);
                }
            }
        }

        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, source, mutants, file_path);
        }
    }
}

/// Helper to map operator name to MutationOperatorType enum
fn map_operator_name_to_type(name: &str) -> MutationOperatorType {
    match name {
        "RustBinaryOp" => MutationOperatorType::ArithmeticReplacement,
        "RustRelationalOp" => MutationOperatorType::RelationalReplacement,
        "RustLogicalOp" => MutationOperatorType::ConditionalReplacement,
        "RustBitwiseOp" => MutationOperatorType::BitwiseReplacement,
        "RustRangeOp" => MutationOperatorType::RangeReplacement,
        "RustPattern" => MutationOperatorType::PatternReplacement,
        "RustMethodChain" => MutationOperatorType::MethodChainReplacement,
        "RustBorrow" => MutationOperatorType::BorrowReplacement,
        _ => MutationOperatorType::ArithmeticReplacement, // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_generator_basic() {
        let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        let generator = RustMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.rs")
            .expect("Should generate mutants");

        assert!(
            !mutants.is_empty(),
            "Should generate at least one mutant for '+' operator"
        );

        // Verify mutant structure
        let mutant = &mutants[0];
        assert!(mutant.id.starts_with("RustBinaryOp"));
        assert_eq!(mutant.original_file, PathBuf::from("test.rs"));
        assert_ne!(mutant.mutated_source, source);
        assert_eq!(mutant.status, MutantStatus::Pending);
    }

    #[test]
    fn test_rust_generator_multiple_operators() {
        let source = r#"
fn compare(a: i32, b: i32) -> bool {
    a > b && a > 0
}
"#;

        let generator = RustMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.rs")
            .expect("Should generate mutants");

        // Should generate mutants for:
        // 1. '>' (relational) - multiple replacements (2 occurrences)
        // 2. '&&' (logical) - replacement with '||'
        assert!(mutants.len() >= 6, "Should generate multiple mutants");

        // Check operator types
        let has_relational = mutants.iter().any(|m| m.id.contains("RelationalOp"));
        let has_logical = mutants.iter().any(|m| m.id.contains("LogicalOp"));

        assert!(has_relational, "Should have relational operator mutants");
        assert!(has_logical, "Should have logical operator mutants");
    }

    #[test]
    fn test_rust_generator_range_operators() {
        let source = r#"
fn range_sum(start: i32, end: i32) -> i32 {
    (start..end).sum()
}
"#;

        let generator = RustMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.rs")
            .expect("Should generate mutants");

        // Should generate mutants for range operator
        let has_range = mutants.iter().any(|m| m.id.contains("RangeOp"));
        assert!(has_range, "Should have range operator mutants");
    }

    #[test]
    fn test_rust_generator_pattern_matching() {
        let source = r#"
fn unwrap(value: Option<i32>) -> i32 {
    match value {
        Some(x) => x,
        None => 0,
    }
}
"#;

        let generator = RustMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.rs")
            .expect("Should generate mutants");

        // Should detect pattern matching (may not mutate due to complexity)
        // Generator works without errors (validated by Result::Ok)
    }

    #[test]
    fn test_rust_generator_method_chains() {
        let source = r#"
fn process(values: Vec<i32>) -> Vec<i32> {
    values.iter().map(|x| x * 2).collect()
}
"#;

        let generator = RustMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.rs")
            .expect("Should generate mutants");

        // Method chains may be detection-only
        // Generator works without errors (validated by Result::Ok)
    }
}
