// Python Mutation Generator using tree-sitter AST
// PMAT-7011: Python Mutation Testing
// Status: RED Phase - Stub implementation

use super::python_tree_sitter_mutations::*;
use super::tree_sitter_operators::TreeSitterMutationOperator;
use super::types::{Mutant, MutantStatus, MutationOperatorType};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tree_sitter::{Node, Parser, Tree};

/// Python mutation generator using tree-sitter AST
pub struct PythonMutationGenerator {
    operators: Vec<Box<dyn TreeSitterMutationOperator>>,
}

impl PythonMutationGenerator {
    /// Create generator with all default Python mutation operators
    pub fn with_default_operators() -> Self {
        Self {
            operators: vec![
                Box::new(PythonBinaryOpMutation),
                Box::new(PythonRelationalOpMutation),
                Box::new(PythonLogicalOpMutation),
                Box::new(PythonIdentityOpMutation),
                Box::new(PythonMembershipOpMutation),
            ],
        }
    }

    /// Generate all mutants from Python source code
    pub fn generate_mutants(&self, source: &str, file_path: &str) -> Result<Vec<Mutant>> {
        let tree = self.parse_python(source)?;
        let mut mutants = Vec::new();

        self.visit_node(&tree.root_node(), source.as_bytes(), &mut mutants, file_path);

        Ok(mutants)
    }

    /// Parse Python source using tree-sitter
    fn parse_python(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Python language: {}", e))?;

        parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Python source"))
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
        "PythonBinaryOp" => MutationOperatorType::ArithmeticReplacement,
        "PythonRelationalOp" => MutationOperatorType::RelationalReplacement,
        "PythonLogicalOp" => MutationOperatorType::ConditionalReplacement,
        "PythonIdentityOp" => MutationOperatorType::RelationalReplacement, // Identity is a type of comparison
        "PythonMembershipOp" => MutationOperatorType::RelationalReplacement, // Membership is a type of comparison
        _ => MutationOperatorType::ArithmeticReplacement, // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn test_python_generator_basic() {
        // RED: Test should fail because generate_mutants returns empty vec
        let source = r#"
def add(a, b):
    return a + b
"#;

        let generator = PythonMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.py")
            .expect("Should generate mutants");

        assert!(!mutants.is_empty(), "Should generate at least one mutant for '+' operator");

        // Verify mutant structure
        let mutant = &mutants[0];
        assert!(mutant.id.starts_with("PythonBinaryOp"));
        assert_eq!(mutant.original_file, PathBuf::from("test.py"));
        assert_ne!(mutant.mutated_source, source);
        assert_eq!(mutant.status, MutantStatus::Pending);
    }

    #[test]

    fn test_python_generator_multiple_operators() {
        // RED: Test should fail
        let source = r#"
def compare(a, b):
    return a > b and a > 0
"#;

        let generator = PythonMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.py")
            .expect("Should generate mutants");

        // Should generate mutants for:
        // 1. '>' (relational) - multiple replacements
        // 2. 'and' (logical) - replacement with 'or'
        assert!(mutants.len() >= 6, "Should generate multiple mutants");

        // Check operator types
        let has_relational = mutants.iter().any(|m| m.id.contains("RelationalOp"));
        let has_logical = mutants.iter().any(|m| m.id.contains("LogicalOp"));

        assert!(has_relational, "Should have relational operator mutants");
        assert!(has_logical, "Should have logical operator mutants");
    }
}
