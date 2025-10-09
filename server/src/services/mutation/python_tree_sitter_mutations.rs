// Python AST mutation operators using tree-sitter
// PMAT-7011: Python Mutation Testing
// Status: RED Phase - Stub implementations

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

/// Python Binary Operator Mutation (AOR - Arithmetic Operator Replacement)
/// Replaces +, -, *, /, //, %, ** with other arithmetic operators
pub struct PythonBinaryOpMutation;

impl TreeSitterMutationOperator for PythonBinaryOpMutation {
    fn name(&self) -> &str {
        "PythonBinaryOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "binary_operator"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node (middle child in binary_operator)
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            // Python operators are represented as their literal text
            if matches!(kind, "+" | "-" | "*" | "/" | "//" | "%" | "**") {
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

        // Generate replacement operators
        let replacements = match op_text {
            "+" => vec!["-", "*", "/", "//", "%", "**"],
            "-" => vec!["+", "*", "/", "//", "%", "**"],
            "*" => vec!["+", "-", "/", "//", "%", "**"],
            "/" => vec!["+", "-", "*", "//", "%", "**"],
            "//" => vec!["+", "-", "*", "/", "%", "**"],
            "%" => vec!["+", "-", "*", "/", "//", "**"],
            "**" => vec!["+", "-", "*", "/", "//", "%"],
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
        0.85 // Arithmetic mutations are usually caught by tests
    }
}

/// Python Relational Operator Mutation (ROR)
/// Replaces <, >, <=, >=, ==, != with other relational operators
pub struct PythonRelationalOpMutation;

impl TreeSitterMutationOperator for PythonRelationalOpMutation {
    fn name(&self) -> &str {
        "PythonRelationalOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "comparison_operator"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find comparison operator child node
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            // Python comparison operators
            if matches!(kind, "<" | ">" | "<=" | ">=" | "==" | "!=" | "is" | "is not" | "in" | "not in") {
                // Only handle relational operators here (not identity/membership)
                if matches!(kind, "<" | ">" | "<=" | ">=" | "==" | "!=") {
                    operator_node = Some(child);
                    break;
                }
            }
        }

        let operator_node = match operator_node {
            Some(n) => n,
            None => return vec![],
        };

        let op_bytes = &source[operator_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        // Generate replacement operators
        let replacements = match op_text {
            "<" => vec![">", "<=", ">=", "==", "!="],
            ">" => vec!["<", "<=", ">=", "==", "!="],
            "<=" => vec!["<", ">", ">=", "==", "!="],
            ">=" => vec!["<", ">", "<=", "==", "!="],
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
        0.75 // Relational mutations sometimes survive
    }
}

/// Python Logical Operator Mutation (LOR)
/// Replaces 'and', 'or' with each other or removes them
pub struct PythonLogicalOpMutation;

impl TreeSitterMutationOperator for PythonLogicalOpMutation {
    fn name(&self) -> &str {
        "PythonLogicalOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        node.kind() == "boolean_operator"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find logical operator child node
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            // Python logical operators
            if matches!(kind, "and" | "or") {
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

        // Generate replacement operators
        let replacements = match op_text {
            "and" => vec!["or"],
            "or" => vec!["and"],
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
        0.80 // Logical mutations are usually caught
    }
}

/// Python Identity Operator Mutation
/// Replaces 'is' with 'is not' and '=='
pub struct PythonIdentityOpMutation;

impl TreeSitterMutationOperator for PythonIdentityOpMutation {
    fn name(&self) -> &str {
        "PythonIdentityOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        if node.kind() != "comparison_operator" {
            return false;
        }

        // Check if operator is "is" or "is not"
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "is" | "not") {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // In Python, "is not" is represented as two separate nodes: "is" and "not"
        // We need to handle both "is" and "is not" cases
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        // Check if we have "is not" (two nodes) or just "is" (one node)
        let is_node = children.iter().find(|c| c.kind() == "is");
        let not_node = children.iter().find(|c| c.kind() == "not");

        match (is_node, not_node) {
            (Some(is_n), Some(not_n)) => {
                // "is not" → "is" and "is not" → "=="
                let mut mutations = Vec::new();

                // Mutation 1: Remove "not" to get just "is"
                let start = is_n.start_byte();
                let end = not_n.end_byte();
                let mut mutated = source.to_vec();
                mutated.splice(start..end, b"is".iter().copied());
                mutations.push(MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: "is not → is".to_string(),
                    location: SourceLocation {
                        line: is_n.start_position().row + 1,
                        column: is_n.start_position().column + 1,
                        end_line: not_n.end_position().row + 1,
                        end_column: not_n.end_position().column + 1,
                    },
                });

                // Mutation 2: Replace with ==
                let mut mutated = source.to_vec();
                mutated.splice(start..end, b"==".iter().copied());
                mutations.push(MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: "is not → ==".to_string(),
                    location: SourceLocation {
                        line: is_n.start_position().row + 1,
                        column: is_n.start_position().column + 1,
                        end_line: not_n.end_position().row + 1,
                        end_column: not_n.end_position().column + 1,
                    },
                });

                mutations
            }
            (Some(is_n), None) => {
                // "is" → "is not" and "is" → "=="
                vec![
                    MutatedSource {
                        source: {
                            let mut mutated = source.to_vec();
                            mutated.splice(is_n.byte_range(), b"is not".iter().copied());
                            String::from_utf8(mutated).unwrap()
                        },
                        description: "is → is not".to_string(),
                        location: SourceLocation {
                            line: is_n.start_position().row + 1,
                            column: is_n.start_position().column + 1,
                            end_line: is_n.end_position().row + 1,
                            end_column: is_n.end_position().column + 1,
                        },
                    },
                    MutatedSource {
                        source: {
                            let mut mutated = source.to_vec();
                            mutated.splice(is_n.byte_range(), b"==".iter().copied());
                            String::from_utf8(mutated).unwrap()
                        },
                        description: "is → ==".to_string(),
                        location: SourceLocation {
                            line: is_n.start_position().row + 1,
                            column: is_n.start_position().column + 1,
                            end_line: is_n.end_position().row + 1,
                            end_column: is_n.end_position().column + 1,
                        },
                    },
                ]
            }
            _ => vec![],
        }
    }

    fn kill_probability(&self) -> f64 {
        0.70 // Identity mutations can survive
    }
}

/// Python Membership Operator Mutation
/// Replaces 'in' with 'not in'
pub struct PythonMembershipOpMutation;

impl TreeSitterMutationOperator for PythonMembershipOpMutation {
    fn name(&self) -> &str {
        "PythonMembershipOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        if node.kind() != "comparison_operator" {
            return false;
        }

        // Check if operator is "in" (not "is")
        let mut cursor = node.walk();
        let mut has_in = false;
        let mut has_is = false;
        for child in node.children(&mut cursor) {
            if child.kind() == "in" {
                has_in = true;
            }
            if child.kind() == "is" {
                has_is = true;
            }
        }
        has_in && !has_is // Only "in", not "is"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // In Python, "not in" is represented as two separate nodes: "not" and "in"
        // We need to handle both "in" and "not in" cases
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        // Check if we have "not in" (two nodes) or just "in" (one node)
        let in_node = children.iter().find(|c| c.kind() == "in");
        let not_node = children.iter().find(|c| c.kind() == "not");

        match (in_node, not_node) {
            (Some(in_n), Some(not_n)) => {
                // "not in" → "in"
                let start = not_n.start_byte();
                let end = in_n.end_byte();
                let mut mutated = source.to_vec();
                mutated.splice(start..end, b"in".iter().copied());
                vec![MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: "not in → in".to_string(),
                    location: SourceLocation {
                        line: not_n.start_position().row + 1,
                        column: not_n.start_position().column + 1,
                        end_line: in_n.end_position().row + 1,
                        end_column: in_n.end_position().column + 1,
                    },
                }]
            }
            (Some(in_n), None) => {
                // "in" → "not in"
                let mut mutated = source.to_vec();
                mutated.splice(in_n.byte_range(), b"not in".iter().copied());
                vec![MutatedSource {
                    source: String::from_utf8(mutated).unwrap(),
                    description: "in → not in".to_string(),
                    location: SourceLocation {
                        line: in_n.start_position().row + 1,
                        column: in_n.start_position().column + 1,
                        end_line: in_n.end_position().row + 1,
                        end_column: in_n.end_position().column + 1,
                    },
                }]
            }
            _ => vec![],
        }
    }

    fn kill_probability(&self) -> f64 {
        0.75 // Membership mutations are usually caught
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_arithmetic_mutation() {
        let source = b"result = a + b";
        let operator = PythonBinaryOpMutation;

        // Parse with tree-sitter-python
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        // Recursively search for binary_operator node
        fn find_and_test(node: &tree_sitter::Node, source: &[u8], operator: &PythonBinaryOpMutation) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty(), "Should generate mutations for '+' operator");

                // Verify mutations replace + with -, *, /, //, %, **
                let expected_ops = vec!["-", "*", "/", "//", "%", "**"];
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

        assert!(find_and_test(&root, source, &operator), "Should find binary_operator node");
    }

    #[test]
    fn test_python_relational_mutation() {
        let source = b"return a > b";
        let operator = PythonRelationalOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(node: &tree_sitter::Node, source: &[u8], operator: &PythonRelationalOpMutation) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty(), "Should generate mutations for '>' operator");

                // Verify mutations replace > with <, >=, <=, ==, !=
                let expected_ops = vec!["<", ">=", "<=", "==", "!="];
                assert_eq!(mutations.len(), expected_ops.len());
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

        assert!(find_and_test(&root, source, &operator), "Should find comparison_operator node");
    }

    #[test]
    fn test_python_logical_mutation() {
        let source = b"return a and b";
        let operator = PythonLogicalOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(node: &tree_sitter::Node, source: &[u8], operator: &PythonLogicalOpMutation) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty(), "Should generate mutations for 'and' operator");

                // Verify mutations replace 'and' with 'or'
                assert!(mutations.iter().any(|m| m.source.contains("or")));
                assert!(mutations.iter().any(|m| m.description.contains("and →")));
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

        assert!(find_and_test(&root, source, &operator), "Should find boolean_operator node");
    }

    #[test]
    fn test_python_identity_mutation() {
        let source = b"return value is None";
        let operator = PythonIdentityOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(node: &tree_sitter::Node, source: &[u8], operator: &PythonIdentityOpMutation) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty(), "Should generate mutations for 'is' operator");

                // Verify mutations replace 'is' with 'is not' and '=='
                assert!(mutations.iter().any(|m| m.source.contains("is not")));
                assert!(mutations.iter().any(|m| m.source.contains("==")));
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

        assert!(find_and_test(&root, source, &operator), "Should find 'is' operator");
    }

    #[test]
    fn test_python_membership_mutation() {
        let source = b"return item in collection";
        let operator = PythonMembershipOpMutation;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_test(node: &tree_sitter::Node, source: &[u8], operator: &PythonMembershipOpMutation) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(!mutations.is_empty(), "Should generate mutations for 'in' operator");

                // Verify mutation replaces 'in' with 'not in'
                assert!(mutations.iter().any(|m| m.source.contains("not in")));
                assert!(mutations.iter().any(|m| m.description.contains("in →")));
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

        assert!(find_and_test(&root, source, &operator), "Should find 'in' operator");
    }
}
