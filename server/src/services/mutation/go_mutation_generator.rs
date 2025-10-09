// Go Mutation Generator using tree-sitter AST
// PMAT-7012: Go Mutation Testing
// Status: RED Phase - Stub implementation

use super::go_tree_sitter_mutations::*;
use super::tree_sitter_operators::TreeSitterMutationOperator;
use super::types::{Mutant, MutantStatus, MutationOperatorType};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tree_sitter::{Node, Parser, Tree};

/// Go mutation generator using tree-sitter AST
pub struct GoMutationGenerator {
    operators: Vec<Box<dyn TreeSitterMutationOperator>>,
}

impl GoMutationGenerator {
    /// Create generator with all default Go mutation operators
    pub fn with_default_operators() -> Self {
        Self {
            operators: vec![
                Box::new(GoBinaryOpMutation),
                Box::new(GoRelationalOpMutation),
                Box::new(GoLogicalOpMutation),
                Box::new(GoBitwiseOpMutation),
                Box::new(GoUnaryOpMutation),
                Box::new(GoAssignmentOpMutation),
            ],
        }
    }

    /// Generate all mutants from Go source code
    pub fn generate_mutants(&self, source: &str, file_path: &str) -> Result<Vec<Mutant>> {
        let tree = self.parse_go(source)?;
        let mut mutants = Vec::new();

        self.visit_node(&tree.root_node(), source.as_bytes(), &mut mutants, file_path);

        Ok(mutants)
    }

    /// Parse Go source using tree-sitter
    fn parse_go(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Go language: {}", e))?;

        parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Go source"))
    }

    /// Recursively visit AST nodes and apply mutation operators
    fn visit_node(
        &self,
        node: &Node,
        source: &[u8],
        mutants: &mut Vec<Mutant>,
        file_path: &str,
    ) {
        // Apply all operators to current node
        for operator in &self.operators {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                for mutation in mutations {
                    let hash = format!("{:x}", Sha256::digest(&mutation.source));
                    mutants.push(Mutant {
                        id: format!(
                            "{}_{}_{}:{}",
                            operator.name(),
                            file_path,
                            mutation.location.line,
                            mutation.location.column
                        ),
                        original_file: PathBuf::from(file_path),
                        mutated_source: mutation.source,
                        operator: map_operator_name_to_type(operator.name()),
                        location: mutation.location,
                        hash,
                        status: MutantStatus::Pending,
                    });
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
        "GoBinaryOp" => MutationOperatorType::ArithmeticReplacement,
        "GoRelationalOp" => MutationOperatorType::RelationalReplacement,
        "GoLogicalOp" => MutationOperatorType::ConditionalReplacement,
        "GoBitwiseOp" => MutationOperatorType::BitwiseReplacement,
        "GoUnaryOp" => MutationOperatorType::UnaryReplacement,
        "GoAssignmentOp" => MutationOperatorType::AssignmentReplacement,
        _ => MutationOperatorType::ArithmeticReplacement, // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_generator_basic() {
        let source = r#"
package main

func Add(a, b int) int {
    return a + b
}
"#;

        let generator = GoMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.go")
            .expect("Should generate mutants");

        assert!(
            !mutants.is_empty(),
            "Should generate at least one mutant for '+' operator"
        );

        // Verify mutant structure
        let mutant = &mutants[0];
        assert!(mutant.id.starts_with("GoBinaryOp"));
        assert_eq!(mutant.original_file, PathBuf::from("test.go"));
        assert_ne!(mutant.mutated_source, source);
        assert_eq!(mutant.status, MutantStatus::Pending);
    }

    #[test]
    fn test_go_generator_multiple_operators() {
        let source = r#"
package main

func Compare(a, b int) bool {
    return a > b && a > 0
}
"#;

        let generator = GoMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.go")
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
}
