// Go Mutation Operators using tree-sitter AST
// PMAT-7012: Go Mutation Testing
// Status: RED Phase - Stub implementation

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

/// Go Binary Operator Mutation (AOR - Arithmetic Operator Replacement)
///
/// Mutates arithmetic operators in Go: +, -, *, /, %
///
/// Example:
/// ```go
/// func Add(a, b int) int {
///     return a + b  // Mutated to: a - b, a * b, a / b, a % b
/// }
/// ```
pub struct GoBinaryOpMutation;

impl TreeSitterMutationOperator for GoBinaryOpMutation {
    fn name(&self) -> &str {
        "GoBinaryOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "binary_expression"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node (middle child in binary_expression)
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "+" | "-" | "*" | "/" | "%") {
                operator_node = Some(child);
                break;
            }
        }

        let operator_node = match operator_node {
            Some(n) => n,
            None => return vec![],
        };

        let op_bytes = &source[operator_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacements = match op_text {
            "+" => vec!["-", "*", "/", "%"],
            "-" => vec!["+", "*", "/", "%"],
            "*" => vec!["+", "-", "/", "%"],
            "/" => vec!["+", "-", "*", "%"],
            "%" => vec!["+", "-", "*", "/"],
            _ => return vec![],
        };

        replacements
            .into_iter()
            .map(|new_op| {
                let mut mutated = source.to_vec();
                mutated.splice(operator_node.byte_range(), new_op.bytes());

                MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: format!("{} → {}", op_text, new_op),
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                }
            })
            .collect()
    }

    fn kill_probability(&self) -> f64 {
        0.85
    }
}

/// Go Relational Operator Mutation (ROR - Relational Operator Replacement)
///
/// Mutates comparison operators: <, >, <=, >=, ==, !=
///
/// Example:
/// ```go
/// func IsPositive(value int) bool {
///     return value > 0  // Mutated to: value < 0, value >= 0, value <= 0, value == 0, value != 0
/// }
/// ```
pub struct GoRelationalOpMutation;

impl TreeSitterMutationOperator for GoRelationalOpMutation {
    fn name(&self) -> &str {
        "GoRelationalOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "binary_expression"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "<" | ">" | "<=" | ">=" | "==" | "!=") {
                operator_node = Some(child);
                break;
            }
        }

        let operator_node = match operator_node {
            Some(n) => n,
            None => return vec![],
        };

        let op_bytes = &source[operator_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacements = match op_text {
            "<" => vec!["<=", ">", ">=", "==", "!="],
            ">" => vec!["<", ">=", "<=", "==", "!="],
            "<=" => vec!["<", ">", ">=", "==", "!="],
            ">=" => vec!["<=", "<", ">", "==", "!="],
            "==" => vec!["!=", "<", ">", "<=", ">="],
            "!=" => vec!["==", "<", ">", "<=", ">="],
            _ => return vec![],
        };

        replacements
            .into_iter()
            .map(|new_op| {
                let mut mutated = source.to_vec();
                mutated.splice(operator_node.byte_range(), new_op.bytes());

                MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: format!("{} → {}", op_text, new_op),
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                }
            })
            .collect()
    }

    fn kill_probability(&self) -> f64 {
        0.90
    }
}

/// Go Logical Operator Mutation (LOR - Logical Operator Replacement)
///
/// Mutates logical operators: &&, ||
///
/// Example:
/// ```go
/// func BothPositive(a, b int) bool {
///     return a > 0 && b > 0  // Mutated to: a > 0 || b > 0
/// }
/// ```
pub struct GoLogicalOpMutation;

impl TreeSitterMutationOperator for GoLogicalOpMutation {
    fn name(&self) -> &str {
        "GoLogicalOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "binary_expression"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "&&" | "||") {
                operator_node = Some(child);
                break;
            }
        }

        let operator_node = match operator_node {
            Some(n) => n,
            None => return vec![],
        };

        let op_bytes = &source[operator_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacement = match op_text {
            "&&" => "||",
            "||" => "&&",
            _ => return vec![],
        };

        let mut mutated = source.to_vec();
        mutated.splice(operator_node.byte_range(), replacement.bytes());

        vec![MutatedSource {
            source: String::from_utf8(mutated).unwrap(),
            description: format!("{} → {}", op_text, replacement),
            location: SourceLocation {
                line: operator_node.start_position().row + 1,
                column: operator_node.start_position().column + 1,
                end_line: operator_node.end_position().row + 1,
                end_column: operator_node.end_position().column + 1,
            },
        }]
    }

    fn kill_probability(&self) -> f64 {
        0.92
    }
}

/// Go Bitwise Operator Mutation (BOR - Bitwise Operator Replacement)
///
/// Mutates bitwise operators: &, |, ^, <<, >>
///
/// Example:
/// ```go
/// func BitwiseAnd(a, b int) int {
///     return a & b  // Mutated to: a | b, a ^ b, a << b, a >> b
/// }
/// ```
pub struct GoBitwiseOpMutation;

impl TreeSitterMutationOperator for GoBitwiseOpMutation {
    fn name(&self) -> &str {
        "GoBitwiseOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "binary_expression"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "&" | "|" | "^" | "<<" | ">>") {
                operator_node = Some(child);
                break;
            }
        }

        let operator_node = match operator_node {
            Some(n) => n,
            None => return vec![],
        };

        let op_bytes = &source[operator_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacements = match op_text {
            "&" => vec!["|", "^", "<<", ">>"],
            "|" => vec!["&", "^", "<<", ">>"],
            "^" => vec!["&", "|", "<<", ">>"],
            "<<" => vec![">>", "&", "|", "^"],
            ">>" => vec!["<<", "&", "|", "^"],
            _ => return vec![],
        };

        replacements
            .into_iter()
            .map(|new_op| {
                let mut mutated = source.to_vec();
                mutated.splice(operator_node.byte_range(), new_op.bytes());

                MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: format!("{} → {}", op_text, new_op),
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                }
            })
            .collect()
    }

    fn kill_probability(&self) -> f64 {
        0.80
    }
}

/// Go Unary Operator Mutation (UOR - Unary Operator Replacement)
///
/// Mutates unary operators: !, -, +
///
/// Example:
/// ```go
/// func Negate(value int) int {
///     return -value  // Mutated to: +value, value (remove operator)
/// }
/// ```
pub struct GoUnaryOpMutation;

impl TreeSitterMutationOperator for GoUnaryOpMutation {
    fn name(&self) -> &str {
        "GoUnaryOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "unary_expression"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node (first child in unary_expression)
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "!" | "-" | "+") {
                operator_node = Some(child);
                break;
            }
        }

        let operator_node = match operator_node {
            Some(n) => n,
            None => return vec![],
        };

        let op_bytes = &source[operator_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacements = match op_text {
            "-" => vec!["+"],
            "+" => vec!["-"],
            "!" => vec![], // Can't replace ! with - or + (type mismatch)
            _ => return vec![],
        };

        replacements
            .into_iter()
            .map(|new_op| {
                let mut mutated = source.to_vec();
                mutated.splice(operator_node.byte_range(), new_op.bytes());

                MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: format!("{} → {}", op_text, new_op),
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                }
            })
            .collect()
    }

    fn kill_probability(&self) -> f64 {
        0.88
    }
}

/// Go Assignment Operator Mutation
///
/// Mutates assignment operators: +=, -=, *=, /=
///
/// Example:
/// ```go
/// func AddAssign(value, delta int) int {
///     value += delta  // Mutated to: value -= delta, value *= delta, value /= delta
///     return value
/// }
/// ```
pub struct GoAssignmentOpMutation;

impl TreeSitterMutationOperator for GoAssignmentOpMutation {
    fn name(&self) -> &str {
        "GoAssignmentOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "assignment_statement"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=") {
                operator_node = Some(child);
                break;
            }
        }

        let operator_node = match operator_node {
            Some(n) => n,
            None => return vec![],
        };

        let op_bytes = &source[operator_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacements = match op_text {
            "+=" => vec!["-=", "*=", "/="],
            "-=" => vec!["+=", "*=", "/="],
            "*=" => vec!["+=", "-=", "/="],
            "/=" => vec!["+=", "-=", "*="],
            "%=" => vec!["+=", "-=", "*=", "/="],
            "&=" => vec!["|=", "^="],
            "|=" => vec!["&=", "^="],
            "^=" => vec!["&=", "|="],
            "<<=" => vec![">>="],
            ">>=" => vec!["<<="],
            _ => return vec![],
        };

        replacements
            .into_iter()
            .map(|new_op| {
                let mut mutated = source.to_vec();
                mutated.splice(operator_node.byte_range(), new_op.bytes());

                MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: format!("{} → {}", op_text, new_op),
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                }
            })
            .collect()
    }

    fn kill_probability(&self) -> f64 {
        0.75
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GREEN PHASE: All operators implemented

    #[test]
    fn test_go_arithmetic_mutation() {
        let source = b"result := a + b";
        let operator = GoBinaryOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Failed to set Go language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        // Recursively search for binary_expression node
        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &GoBinaryOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(
                    !mutations.is_empty(),
                    "Should generate mutations for '+' operator"
                );

                let expected_ops = vec!["-", "*", "/", "%"];
                assert_eq!(mutations.len(), expected_ops.len());

                for (i, mutation) in mutations.iter().enumerate() {
                    assert!(mutation.source.contains(expected_ops[i]));
                    assert!(mutation.description.contains("+ →"));
                }
                return true;
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if find_and_test(&child, source, operator) {
                    return true;
                }
            }
            false
        }

        assert!(
            find_and_test(&root, source, &operator),
            "Should find binary_expression node"
        );
    }

    #[test]
    fn test_go_relational_mutation() {
        let source = b"flag := value > 0";
        let operator = GoRelationalOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Failed to set Go language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &GoRelationalOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty(), "Should generate mutations");

                let expected_ops = vec!["<", ">=", "<=", "==", "!="];
                assert!(mutations.len() >= 3, "Should generate at least 3 mutants");

                return true;
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if find_and_test(&child, source, operator) {
                    return true;
                }
            }
            false
        }

        assert!(find_and_test(&root, source, &operator));
    }

    #[test]
    fn test_go_logical_mutation() {
        let source = b"result := a > 0 && b > 0";
        let operator = GoLogicalOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Failed to set Go language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &GoLogicalOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty());
                assert!(mutations.iter().any(|m| m.source.contains("||")));
                return true;
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if find_and_test(&child, source, operator) {
                    return true;
                }
            }
            false
        }

        assert!(find_and_test(&root, source, &operator));
    }

    #[test]
    fn test_go_bitwise_mutation() {
        let source = b"result := a & b";
        let operator = GoBitwiseOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Failed to set Go language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &GoBitwiseOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty());

                let expected = vec!["|", "^", "<<", ">>"];
                assert!(mutations.len() >= 2);

                return true;
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if find_and_test(&child, source, operator) {
                    return true;
                }
            }
            false
        }

        assert!(find_and_test(&root, source, &operator));
    }

    #[test]
    fn test_go_unary_mutation() {
        let source = b"result := -value";
        let operator = GoUnaryOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Failed to set Go language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &GoUnaryOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty());
                assert!(mutations.iter().any(|m| m.source.contains("+value")));
                return true;
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if find_and_test(&child, source, operator) {
                    return true;
                }
            }
            false
        }

        assert!(find_and_test(&root, source, &operator));
    }

    #[test]
    fn test_go_assignment_mutation() {
        let source = b"value += delta";
        let operator = GoAssignmentOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Failed to set Go language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &GoAssignmentOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty());

                let expected = vec!["-=", "*=", "/="];
                assert!(mutations.len() >= 2);

                return true;
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if find_and_test(&child, source, operator) {
                    return true;
                }
            }
            false
        }

        assert!(find_and_test(&root, source, &operator));
    }
}
