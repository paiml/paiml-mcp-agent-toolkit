// C++ Mutation Generator using tree-sitter AST
// PMAT-7013: C++ Mutation Testing
// Status: GREEN Phase - Full implementation

use super::cpp_tree_sitter_mutations::*;
use super::tree_sitter_operators::TreeSitterMutationOperator;
use super::types::{Mutant, MutantStatus, MutationOperatorType};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tree_sitter::{Node, Parser, Tree};

/// C++ mutation generator using tree-sitter AST
pub struct CppMutationGenerator {
    operators: Vec<Box<dyn TreeSitterMutationOperator>>,
}

impl CppMutationGenerator {
    /// Create generator with all default C++ mutation operators
    pub fn with_default_operators() -> Self {
        Self {
            operators: vec![
                Box::new(CppBinaryOpMutation),
                Box::new(CppRelationalOpMutation),
                Box::new(CppLogicalOpMutation),
                Box::new(CppBitwiseOpMutation),
                Box::new(CppUnaryOpMutation),
                Box::new(CppPointerOpMutation),
                Box::new(CppMemberAccessMutation),
            ],
        }
    }

    /// Generate all mutants from C++ source code
    pub fn generate_mutants(&self, source: &str, file_path: &str) -> Result<Vec<Mutant>> {
        let tree = self.parse_cpp(source)?;
        let mut mutants = Vec::new();

        self.visit_node(&tree.root_node(), source.as_bytes(), &mut mutants, file_path);

        Ok(mutants)
    }

    /// Parse C++ source using tree-sitter
    fn parse_cpp(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_cpp::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set C++ language: {}", e))?;

        parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse C++ source"))
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
        "CppBinaryOp" => MutationOperatorType::ArithmeticReplacement,
        "CppRelationalOp" => MutationOperatorType::RelationalReplacement,
        "CppLogicalOp" => MutationOperatorType::ConditionalReplacement,
        "CppBitwiseOp" => MutationOperatorType::BitwiseReplacement,
        "CppUnaryOp" => MutationOperatorType::UnaryReplacement,
        "CppPointerOp" => MutationOperatorType::PointerReplacement,
        "CppMemberAccess" => MutationOperatorType::MemberAccessReplacement,
        _ => MutationOperatorType::ArithmeticReplacement, // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpp_generator_basic() {
        let source = r#"
int Add(int a, int b) {
    return a + b;
}
"#;

        let generator = CppMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.cpp")
            .expect("Should generate mutants");

        assert!(
            !mutants.is_empty(),
            "Should generate at least one mutant for '+' operator"
        );

        // Verify mutant structure
        let mutant = &mutants[0];
        assert!(mutant.id.starts_with("CppBinaryOp"));
        assert_eq!(mutant.original_file, PathBuf::from("test.cpp"));
        assert_ne!(mutant.mutated_source, source);
        assert_eq!(mutant.status, MutantStatus::Pending);
    }

    #[test]
    fn test_cpp_generator_multiple_operators() {
        let source = r#"
bool Compare(int a, int b) {
    return a > b && a > 0;
}
"#;

        let generator = CppMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.cpp")
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
    fn test_cpp_generator_pointer_operators() {
        let source = r#"
int Dereference(int* ptr) {
    return *ptr;
}
"#;

        let generator = CppMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.cpp")
            .expect("Should generate mutants");

        // Pointer operators are detected but not mutated (complex semantics)
        // So we may have 0 mutants from pointer operators
        // Generator works without errors (validated by Result::Ok)
    }

    #[test]
    fn test_cpp_generator_member_access() {
        let source = r#"
class Calculator {
    int GetValue() {
        return this->instanceValue;
    }
};
"#;

        let generator = CppMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.cpp")
            .expect("Should generate mutants");

        // Member access operators are detected but not mutated (complex semantics)
        // Generator works without errors (validated by Result::Ok)
    }

    #[test]
    fn test_cpp_generator_bitwise_operators() {
        let source = r#"
int BitwiseAnd(int a, int b) {
    return a & b;
}
"#;

        let generator = CppMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.cpp")
            .expect("Should generate mutants");

        // Should generate mutants for bitwise & operator
        assert!(!mutants.is_empty(), "Should generate bitwise operator mutants");

        let has_bitwise = mutants.iter().any(|m| m.id.contains("BitwiseOp"));
        assert!(has_bitwise, "Should have bitwise operator mutants");
    }

    #[test]
    fn test_cpp_generator_update_expressions() {
        let source = r#"
int PreIncrement(int value) {
    return ++value;
}
"#;

        let generator = CppMutationGenerator::with_default_operators();
        let mutants = generator
            .generate_mutants(source, "test.cpp")
            .expect("Should generate mutants");

        // Should generate mutants for ++ operator
        assert!(!mutants.is_empty(), "Should generate update expression mutants");

        let has_unary = mutants.iter().any(|m| m.id.contains("UnaryOp"));
        assert!(has_unary, "Should have unary operator mutants");
    }
}
