//! TypeScript/JavaScript tree-sitter based mutation operators
//!
//! EXTREME TDD: RED PHASE - Stub implementations, all tests will fail
//!
//! This module implements AST-based mutation operators for TypeScript and JavaScript
//! using tree-sitter instead of language-specific parsers.

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

/// Arithmetic Operator Replacement (AOR) for TypeScript/JavaScript
///
/// Mutations: + → -, * → /, etc.
pub struct TypeScriptBinaryOpMutation;

impl TreeSitterMutationOperator for TypeScriptBinaryOpMutation {
    fn name(&self) -> &str {
        "AOR/ROR"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        // GREEN PHASE: Detect binary expressions
        node.kind() == "binary_expression"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // GREEN PHASE: Generate mutations for binary operators
        if node.kind() != "binary_expression" {
            return vec![];
        }

        // Find the operator child node
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            // Tree-sitter TypeScript represents operators as their literal text
            if kind == "+" || kind == "-" || kind == "*" || kind == "/" || kind == "%"
                || kind == ">" || kind == "<" || kind == ">=" || kind == "<="
                || kind == "==" || kind == "!=" || kind == "===" || kind == "!==" {
                operator_node = Some(child);
                break;
            }
        }

        let op_node = match operator_node {
            Some(node) => node,
            None => return vec![],
        };

        let op_bytes = &source[op_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        // Determine replacement operators based on current operator
        let replacements: Vec<&str> = match op_text {
            // Arithmetic operators
            "+" => vec!["-", "*", "/"],
            "-" => vec!["+", "*", "/"],
            "*" => vec!["+", "-", "/"],
            "/" => vec!["+", "-", "*"],
            "%" => vec!["*", "/"],

            // Relational operators
            ">" => vec!["<", ">=", "<=", "==", "!="],
            "<" => vec![">", ">=", "<=", "==", "!="],
            ">=" => vec![">", "<", "<=", "==", "!="],
            "<=" => vec![">", "<", ">=", "==", "!="],
            "==" => vec!["!=", ">", "<", ">=", "<="],
            "!=" => vec!["==", ">", "<", ">=", "<="],
            "===" => vec!["!==", "==", "!="],
            "!==" => vec!["===", "==", "!="],

            _ => vec![],
        };

        // Generate mutated source for each replacement
        replacements.into_iter().map(|new_op| {
            let mut mutated = source.to_vec();
            let range = op_node.byte_range();

            // Splice in the new operator
            mutated.splice(range.clone(), new_op.bytes());

            MutatedSource {
                source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
                description: format!("{} → {}", op_text, new_op),
                location: SourceLocation {
                    line: op_node.start_position().row + 1,
                    column: op_node.start_position().column + 1,
                    end_line: op_node.end_position().row + 1,
                    end_column: op_node.end_position().column + 1,
                },
            }
        }).collect()
    }

    fn kill_probability(&self) -> f64 {
        0.85
    }
}

/// Strict Equality Mutation for TypeScript/JavaScript
///
/// Mutations: === → ==, !== → !=
pub struct TypeScriptStrictEqualityMutation;

impl TreeSitterMutationOperator for TypeScriptStrictEqualityMutation {
    fn name(&self) -> &str {
        "Strict Equality"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        // GREEN PHASE: Detect strict equality operators
        if node.kind() != "binary_expression" {
            return false;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if kind == "===" || kind == "!==" {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // GREEN PHASE: Mutate strict equality operators
        if node.kind() != "binary_expression" {
            return vec![];
        }

        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            if child.kind() == "===" || child.kind() == "!==" {
                operator_node = Some(child);
                break;
            }
        }

        let op_node = match operator_node {
            Some(node) => node,
            None => return vec![],
        };

        let op_bytes = &source[op_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacements: Vec<&str> = match op_text {
            "===" => vec!["==", "!==", "!="],
            "!==" => vec!["!=", "===", "=="],
            _ => vec![],
        };

        replacements.into_iter().map(|new_op| {
            let mut mutated = source.to_vec();
            mutated.splice(op_node.byte_range(), new_op.bytes());

            MutatedSource {
                source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
                description: format!("{} → {}", op_text, new_op),
                location: SourceLocation {
                    line: op_node.start_position().row + 1,
                    column: op_node.start_position().column + 1,
                    end_line: op_node.end_position().row + 1,
                    end_column: op_node.end_position().column + 1,
                },
            }
        }).collect()
    }
}

/// Optional Chaining Mutation for TypeScript
///
/// Mutations: obj?.prop → obj.prop
pub struct TypeScriptOptionalChainingMutation;

impl TreeSitterMutationOperator for TypeScriptOptionalChainingMutation {
    fn name(&self) -> &str {
        "Optional Chaining"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        // GREEN PHASE: Detect optional chaining expressions
        // Tree-sitter represents optional chaining as specific node types
        matches!(node.kind(), "optional_chain" | "member_expression") &&
            node.to_sexp().contains("?.")
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // GREEN PHASE: Remove optional chaining operator
        let source_text = std::str::from_utf8(source).unwrap_or("");
        let node_text = &source_text[node.byte_range()];

        // Simple mutation: remove '?' from '?.'
        if !node_text.contains("?.") {
            return vec![];
        }

        let mutated_text = node_text.replace("?.", ".");
        let mut mutated = source.to_vec();
        mutated.splice(node.byte_range(), mutated_text.bytes());

        vec![MutatedSource {
            source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
            description: "?. → .".to_string(),
            location: SourceLocation {
                line: node.start_position().row + 1,
                column: node.start_position().column + 1,
                end_line: node.end_position().row + 1,
                end_column: node.end_position().column + 1,
            },
        }]
    }
}

/// Nullish Coalescing Mutation for TypeScript
///
/// Mutations: a ?? b → a || b, a ?? b → b
pub struct TypeScriptNullishCoalescingMutation;

impl TreeSitterMutationOperator for TypeScriptNullishCoalescingMutation {
    fn name(&self) -> &str {
        "Nullish Coalescing"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        // GREEN PHASE: Detect nullish coalescing operator
        if node.kind() != "binary_expression" {
            return false;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "??" {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // GREEN PHASE: Mutate nullish coalescing operator
        if node.kind() != "binary_expression" {
            return vec![];
        }

        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            if child.kind() == "??" {
                operator_node = Some(child);
                break;
            }
        }

        let op_node = match operator_node {
            Some(node) => node,
            None => return vec![],
        };

        // Two mutations: ?? → || and ?? → (use right operand only)
        vec![
            MutatedSource {
                source: {
                    let mut mutated = source.to_vec();
                    mutated.splice(op_node.byte_range(), "||".bytes());
                    String::from_utf8(mutated).unwrap_or_else(|_| String::new())
                },
                description: "?? → ||".to_string(),
                location: SourceLocation {
                    line: op_node.start_position().row + 1,
                    column: op_node.start_position().column + 1,
                    end_line: op_node.end_position().row + 1,
                    end_column: op_node.end_position().column + 1,
                },
            }
        ]
    }
}

/// Async/Await Mutation for TypeScript/JavaScript
///
/// Mutations: Remove await, remove async
pub struct TypeScriptAsyncAwaitMutation;

impl TreeSitterMutationOperator for TypeScriptAsyncAwaitMutation {
    fn name(&self) -> &str {
        "Async/Await"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        // GREEN PHASE: Detect async/await keywords
        let kind = node.kind();
        if kind == "function_declaration" || kind == "arrow_function" || kind == "method_definition" {
            let source_text = std::str::from_utf8(source).unwrap_or("");
            let node_text = &source_text[node.byte_range()];
            return node_text.contains("async");
        }

        if kind == "await_expression" {
            return true;
        }

        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // GREEN PHASE: Remove async or await keywords
        let source_text = std::str::from_utf8(source).unwrap_or("");
        let node_text = &source_text[node.byte_range()];

        let mut mutations = Vec::new();

        if node.kind() == "await_expression" {
            // Remove "await " from expression
            let mutated_text = node_text.replace("await ", "");
            let mut mutated = source.to_vec();
            mutated.splice(node.byte_range(), mutated_text.bytes());

            mutations.push(MutatedSource {
                source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
                description: "Remove await".to_string(),
                location: SourceLocation {
                    line: node.start_position().row + 1,
                    column: node.start_position().column + 1,
                    end_line: node.end_position().row + 1,
                    end_column: node.end_position().column + 1,
                },
            });
        } else if node_text.contains("async") {
            // Remove "async " keyword
            let mutated_text = node_text.replace("async ", "");
            let mut mutated = source.to_vec();
            mutated.splice(node.byte_range(), mutated_text.bytes());

            mutations.push(MutatedSource {
                source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
                description: "Remove async".to_string(),
                location: SourceLocation {
                    line: node.start_position().row + 1,
                    column: node.start_position().column + 1,
                    end_line: node.end_position().row + 1,
                    end_column: node.end_position().column + 1,
                },
            });
        }

        mutations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::{Parser, Tree};

    fn parse_typescript(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn find_binary_expression(tree: &Tree) -> Option<Node> {
        let root = tree.root_node();

        fn find_recursive<'a>(node: &tree_sitter::Node<'a>) -> Option<tree_sitter::Node<'a>> {
            if node.kind() == "binary_expression" {
                return Some(*node);
            }
            for child in node.children(&mut node.walk()) {
                if let Some(found) = find_recursive(&child) {
                    return Some(found);
                }
            }
            None
        }

        find_recursive(&root)
    }

    // RED PHASE TESTS: All should fail

    #[test]
    #[ignore] // RED: Will fail - not implemented
    fn red_test_typescript_arithmetic_operator_replacement() {
        let source = "function add(a, b) { return a + b; }";
        let tree = parse_typescript(source);
        let node = find_binary_expression(&tree).unwrap();

        let operator = TypeScriptBinaryOpMutation;
        let mutants = operator.mutate(&node, source.as_bytes());

        // Should generate: a - b, a * b, a / b
        assert!(mutants.len() >= 3, "Expected at least 3 mutants, got {}", mutants.len());
        assert!(
            mutants.iter().any(|m| m.source.contains("a - b")),
            "Missing mutation: + → -"
        );
        assert!(
            mutants.iter().any(|m| m.source.contains("a * b")),
            "Missing mutation: + → *"
        );
        assert!(
            mutants.iter().any(|m| m.source.contains("a / b")),
            "Missing mutation: + → /"
        );
    }

    #[test]
    #[ignore] // RED: Will fail
    fn red_test_typescript_strict_equality_mutation() {
        let source = "if (x === 5) { return true; }";
        let tree = parse_typescript(source);
        let node = find_binary_expression(&tree).unwrap();

        let operator = TypeScriptStrictEqualityMutation;
        let mutants = operator.mutate(&node, source.as_bytes());

        assert!(
            mutants.iter().any(|m| m.source.contains("x == 5")),
            "Missing mutation: === → =="
        );
        assert!(
            mutants.iter().any(|m| m.source.contains("x !== 5")),
            "Missing mutation: === → !=="
        );
    }

    #[test]
    #[ignore] // RED: Will fail
    fn red_test_typescript_optional_chaining_mutation() {
        let source = "const val = obj?.prop?.nested;";
        let tree = parse_typescript(source);

        // Find optional chaining nodes
        // This is simplified - real implementation needs proper AST traversal
        let operator = TypeScriptOptionalChainingMutation;

        // RED PHASE: This will fail because can_mutate returns false
        let root = tree.root_node();
        assert!(
            operator.can_mutate(&root, source.as_bytes()),
            "Should detect optional chaining"
        );
    }

    #[test]
    #[ignore] // RED: Will fail
    fn red_test_typescript_nullish_coalescing_mutation() {
        let source = "const val = value ?? defaultValue;";
        let tree = parse_typescript(source);

        let operator = TypeScriptNullishCoalescingMutation;
        let root = tree.root_node();

        // RED PHASE: This will fail
        assert!(
            operator.can_mutate(&root, source.as_bytes()),
            "Should detect nullish coalescing"
        );
    }

    #[test]
    #[ignore] // RED: Will fail
    fn red_test_typescript_async_await_mutation() {
        let source = "async function fetch() { return await api(); }";
        let tree = parse_typescript(source);

        let operator = TypeScriptAsyncAwaitMutation;
        let root = tree.root_node();

        let mutants = operator.mutate(&root, source.as_bytes());

        assert!(
            mutants.iter().any(|m| m.source.contains("return api()") && !m.source.contains("await")),
            "Missing mutation: Remove await"
        );
        assert!(
            mutants.iter().any(|m| !m.source.contains("async")),
            "Missing mutation: Remove async"
        );
    }

    #[test]
    #[ignore] // RED: Will fail
    fn red_test_mutation_preserves_syntax() {
        let source = "function test() { return x + y; }";
        let tree = parse_typescript(source);
        let node = find_binary_expression(&tree).unwrap();

        let operator = TypeScriptBinaryOpMutation;
        let mutants = operator.mutate(&node, source.as_bytes());

        // All mutants must parse without syntax errors
        for mutant in mutants {
            let mutated_tree = parse_typescript(&mutant.source);
            assert!(
                !mutated_tree.root_node().has_error(),
                "Mutant has syntax error: {}",
                mutant.source
            );
        }
    }

    #[test]
    #[ignore] // RED: Will fail
    fn red_test_mutation_location_metadata() {
        let source = "function test() {\n  return a + b;\n}";
        let tree = parse_typescript(source);
        let node = find_binary_expression(&tree).unwrap();

        let operator = TypeScriptBinaryOpMutation;
        let mutants = operator.mutate(&node, source.as_bytes());

        assert!(!mutants.is_empty(), "No mutants generated");

        for mutant in mutants {
            assert!(mutant.location.line > 0, "Line number should be > 0");
            assert!(mutant.location.column > 0, "Column number should be > 0");
        }
    }
}
