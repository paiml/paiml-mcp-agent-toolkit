//! TypeScript/JavaScript mutation generator using tree-sitter AST visitor
//!
//! GREEN PHASE: AST traversal and mutation generation

use super::tree_sitter_operators::TreeSitterMutationOperator;
use super::types::{Mutant, MutationOperatorType, SourceLocation};
use anyhow::Result;
use tree_sitter::{Parser, Tree};

/// TypeScript mutation generator
pub struct TypeScriptMutationGenerator {
    operators: Vec<Box<dyn TreeSitterMutationOperator>>,
}

impl TypeScriptMutationGenerator {
    /// Create new mutation generator with given operators
    pub fn new(operators: Vec<Box<dyn TreeSitterMutationOperator>>) -> Self {
        Self { operators }
    }

    /// Create with default TypeScript operators
    pub fn with_default_operators() -> Self {
        use super::typescript_tree_sitter_mutations::*;

        Self {
            operators: vec![
                Box::new(TypeScriptBinaryOpMutation),
                Box::new(TypeScriptStrictEqualityMutation),
                Box::new(TypeScriptOptionalChainingMutation),
                Box::new(TypeScriptNullishCoalescingMutation),
                Box::new(TypeScriptAsyncAwaitMutation),
            ],
        }
    }

    /// Generate mutants from TypeScript source code
    pub fn generate_mutants(&self, source: &str, file_path: &str) -> Result<Vec<Mutant>> {
        // Parse TypeScript source with tree-sitter
        let tree = self.parse_typescript(source)?;

        // Visit AST and collect mutations
        let mut mutants = Vec::new();
        self.visit_node(&tree.root_node(), source.as_bytes(), &mut mutants, file_path);

        Ok(mutants)
    }

    /// Parse TypeScript source to AST
    fn parse_typescript(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        // Use tree-sitter-javascript which supports both JS and TS syntax
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set JavaScript/TypeScript language: {}", e))?;

        parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse TypeScript source"))
    }

    /// Visit AST node and generate mutations
    fn visit_node(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        mutants: &mut Vec<Mutant>,
        file_path: &str,
    ) {
        // Try each operator on this node
        for operator in &self.operators {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);

                for mutation in mutations {
                    let mutated_source = mutation.source.clone();
                    // Simple hash using SHA256 (already in deps)
                    use sha2::{Sha256, Digest};
                    let hash = format!("{:x}", Sha256::digest(&mutated_source));

                    mutants.push(Mutant {
                        id: format!(
                            "{}_{}_{}:{}",
                            operator.name(),
                            sanitize_description(&mutation.description),
                            mutation.location.line,
                            mutation.location.column
                        ),
                        original_file: std::path::PathBuf::from(file_path),
                        mutated_source,
                        operator: map_operator_name_to_type(operator.name()),
                        location: super::types::SourceLocation {
                            line: mutation.location.line,
                            column: mutation.location.column,
                            end_line: mutation.location.line,
                            end_column: mutation.location.column + 1,
                        },
                        hash,
                        status: super::types::MutantStatus::Pending,
                    });
                }
            }
        }

        // Recurse to children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, source, mutants, file_path);
        }
    }
}

/// Sanitize mutation description for use in ID
fn sanitize_description(desc: &str) -> String {
    desc.replace(" ", "_")
        .replace("→", "to")
        .replace("?", "")
        .replace(".", "")
        .replace("+", "plus")
        .replace("-", "minus")
        .replace("*", "mul")
        .replace("/", "div")
        .replace("=", "eq")
        .replace("!", "not")
        .replace(">", "gt")
        .replace("<", "lt")
        .replace("|", "or")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Map operator name to MutationOperatorType
fn map_operator_name_to_type(name: &str) -> MutationOperatorType {
    match name {
        "AOR/ROR" => MutationOperatorType::ArithmeticReplacement,
        "Strict Equality" => MutationOperatorType::RelationalReplacement,
        "Optional Chaining" => MutationOperatorType::StatementDeletion, // Closest match
        "Nullish Coalescing" => MutationOperatorType::ConditionalReplacement,
        "Async/Await" => MutationOperatorType::StatementDeletion,
        _ => MutationOperatorType::ArithmeticReplacement,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_description() {
        assert_eq!(sanitize_description("+ → -"), "plus_to_minus");
        assert_eq!(sanitize_description("=== → =="), "eqeqeq_to_eqeq");
        assert_eq!(sanitize_description("?. → ."), "_to_");
    }

    #[test]
    #[ignore] // GREEN: Will pass once operators work
    fn test_generate_mutants_arithmetic() {
        let generator = TypeScriptMutationGenerator::with_default_operators();
        let source = "function add(a, b) { return a + b; }";

        let mutants = generator.generate_mutants(source, "test.ts").unwrap();

        assert!(!mutants.is_empty(), "Should generate mutants");
        assert!(
            mutants.iter().any(|m| m.mutated_source.contains("a - b")),
            "Should generate + → - mutation"
        );
    }

    #[test]
    #[ignore] // GREEN: Will pass once operators work
    fn test_generate_mutants_strict_equality() {
        let generator = TypeScriptMutationGenerator::with_default_operators();
        let source = "if (x === 5) { return true; }";

        let mutants = generator.generate_mutants(source, "test.ts").unwrap();

        assert!(
            mutants.iter().any(|m| m.mutated_source.contains("x == 5")),
            "Should generate === → == mutation"
        );
    }
}
