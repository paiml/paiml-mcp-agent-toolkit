#![cfg_attr(coverage_nightly, coverage(off))]
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
                    source: String::from_utf8(mutated).expect(
                        "mutated source is valid UTF-8 (original source + ASCII operators)",
                    ),
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
                    source: String::from_utf8(mutated).expect(
                        "mutated source is valid UTF-8 (original source + ASCII operators)",
                    ),
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
            source: String::from_utf8(mutated)
                .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
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
                    source: String::from_utf8(mutated).expect(
                        "mutated source is valid UTF-8 (original source + ASCII operators)",
                    ),
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
                    source: String::from_utf8(mutated).expect(
                        "mutated source is valid UTF-8 (original source + ASCII operators)",
                    ),
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
            if matches!(
                kind,
                "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>="
            ) {
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
                    source: String::from_utf8(mutated).expect(
                        "mutated source is valid UTF-8 (original source + ASCII operators)",
                    ),
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
#[allow(unused_variables)]
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

                let expected_ops = ["-", "*", "/", "%"];
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

                let expected_ops = ["<", ">=", "<=", "==", "!="];
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

                let expected = ["|", "^", "<<", ">>"];
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

                let expected = ["-=", "*=", "/="];
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

    /// Test UTF-8 validity after Go mutation (validates expect() at lines 67, 144, 214, 288, 362, 447)
    #[test]
    fn test_utf8_validity_after_go_mutation() {
        let source = b"result := a + b - c * d / e";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Failed to set Go language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn collect_mutations(node: &tree_sitter::Node, source: &[u8]) -> Vec<MutatedSource> {
            let mut all_mutations = Vec::new();

            if let "binary_expression" = node.kind() {
                let operator = GoBinaryOpMutation;
                if operator.can_mutate(node, source) {
                    all_mutations.extend(operator.mutate(node, source));
                }
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                all_mutations.extend(collect_mutations(&child, source));
            }

            all_mutations
        }

        let mutations = collect_mutations(&root, source);

        // All mutations must be valid UTF-8
        for mutation in &mutations {
            assert!(!mutation.source.is_empty());
            assert!(std::str::from_utf8(mutation.source.as_bytes()).is_ok());
        }

        assert!(!mutations.is_empty());
    }

    /// Test all Go operators produce valid UTF-8
    #[test]
    fn test_all_go_operators_produce_valid_utf8() {
        let test_cases = vec![
            b"a + b".as_slice(),
            b"x == y".as_slice(),
            b"a && b".as_slice(),
            b"x += y".as_slice(),
        ];

        for source in test_cases {
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&tree_sitter_go::LANGUAGE.into())
                .expect("Failed to set Go language");

            let tree = parser.parse(source, None).expect("Failed to parse");
            let root = tree.root_node();

            fn collect_all(node: &tree_sitter::Node, source: &[u8]) -> Vec<MutatedSource> {
                let mut mutations = Vec::new();

                let operators: Vec<Box<dyn TreeSitterMutationOperator>> = vec![
                    Box::new(GoBinaryOpMutation),
                    Box::new(GoRelationalOpMutation),
                    Box::new(GoLogicalOpMutation),
                    Box::new(GoAssignmentOpMutation),
                ];

                for op in operators {
                    if op.can_mutate(node, source) {
                        mutations.extend(op.mutate(node, source));
                    }
                }

                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    mutations.extend(collect_all(&child, source));
                }

                mutations
            }

            let mutations = collect_all(&root, source);

            for mutation in &mutations {
                assert!(!mutation.source.is_empty());
                assert!(
                    std::str::from_utf8(mutation.source.as_bytes()).is_ok(),
                    "Mutation should produce valid UTF-8: {}",
                    mutation.description
                );
            }
        }
    }
}
