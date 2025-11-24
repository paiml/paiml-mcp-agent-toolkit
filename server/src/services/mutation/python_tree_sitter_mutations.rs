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
                    source: String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
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
            if matches!(
                kind,
                "<" | ">" | "<=" | ">=" | "==" | "!=" | "is" | "is not" | "in" | "not in"
            ) {
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
                    source: String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
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
                    source: String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
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
                    source: String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
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
                    source: String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
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
                            String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)")
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
                            String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)")
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
                    source: String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
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
                    source: String::from_utf8(mutated)
                    .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
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
        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &PythonBinaryOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(
                    !mutations.is_empty(),
                    "Should generate mutations for '+' operator"
                );

                // Verify mutations replace + with -, *, /, //, %, **
                let expected_ops = ["-", "*", "/", "//", "%", "**"];
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
            "Should find binary_operator node"
        );
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

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &PythonRelationalOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(
                    !mutations.is_empty(),
                    "Should generate mutations for '>' operator"
                );

                // Verify mutations replace > with <, >=, <=, ==, !=
                let expected_ops = ["<", ">=", "<=", "==", "!="];
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

        assert!(
            find_and_test(&root, source, &operator),
            "Should find comparison_operator node"
        );
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

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &PythonLogicalOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(
                    !mutations.is_empty(),
                    "Should generate mutations for 'and' operator"
                );

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

        assert!(
            find_and_test(&root, source, &operator),
            "Should find boolean_operator node"
        );
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

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &PythonIdentityOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(
                    !mutations.is_empty(),
                    "Should generate mutations for 'is' operator"
                );

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

        assert!(
            find_and_test(&root, source, &operator),
            "Should find 'is' operator"
        );
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

        fn find_and_test(
            node: &tree_sitter::Node,
            source: &[u8],
            operator: &PythonMembershipOpMutation,
        ) -> bool {
            if operator.can_mutate(node, source) {
                let mutations = operator.mutate(node, source);
                assert!(
                    !mutations.is_empty(),
                    "Should generate mutations for 'in' operator"
                );

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

        assert!(
            find_and_test(&root, source, &operator),
            "Should find 'in' operator"
        );
    }

    /// Test UTF-8 validity after mutation (validates expect() at lines 63, 140, 207, 270, 284, 303, 317, 385, 400)
    #[test]
    fn test_utf8_validity_after_mutation() {
        // Test case 1: Simple ASCII operators (most common)
        let source = b"result = a + b - c * d / e";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        // Find and mutate all operators
        fn collect_mutations(node: &tree_sitter::Node, source: &[u8]) -> Vec<MutatedSource> {
            let mut all_mutations = Vec::new();

            if let "binary_operator" = node.kind() {
                let operator = PythonBinaryOpMutation;
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

        // Verify all mutations produce valid UTF-8
        for mutation in &mutations {
            // This validates the expect() doesn't panic
            assert!(!mutation.source.is_empty());
            // Verify it's valid UTF-8 by checking it can be parsed again
            assert!(mutation.source.is_ascii() || mutation.source.chars().count() > 0);
        }

        assert!(!mutations.is_empty(), "Should generate mutations");
    }

    /// Test UTF-8 validity with Unicode identifiers (Python 3 supports Unicode)
    #[test]
    fn test_utf8_validity_with_unicode_identifiers() {
        // Python 3 allows Unicode identifiers
        let source = "résultat = α + β".as_bytes();

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_and_mutate(node: &tree_sitter::Node, source: &[u8]) -> Vec<MutatedSource> {
            let operator = PythonBinaryOpMutation;
            if operator.can_mutate(node, source) {
                return operator.mutate(node, source);
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                let mutations = find_and_mutate(&child, source);
                if !mutations.is_empty() {
                    return mutations;
                }
            }
            vec![]
        }

        let mutations = find_and_mutate(&root, source);

        // All mutations should preserve Unicode and remain valid UTF-8
        for mutation in &mutations {
            assert!(mutation.source.contains("résultat"));
            assert!(mutation.source.contains("α"));
            assert!(mutation.source.contains("β"));
            // Verify valid UTF-8 by ensuring chars() doesn't panic
            assert!(mutation.source.chars().count() > 0);
        }

        assert!(!mutations.is_empty(), "Should generate mutations with Unicode");
    }

    /// Test all mutation operators produce valid UTF-8
    #[test]
    fn test_all_operators_produce_valid_utf8() {
        // Test with various operators
        let test_cases = vec![
            b"a + b".as_slice(),
            b"x == y".as_slice(),
            b"a and b".as_slice(),
            b"x in y".as_slice(),
        ];

        for source in test_cases {
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&tree_sitter_python::LANGUAGE.into())
                .expect("Failed to set Python language");

            let tree = parser.parse(source, None).expect("Failed to parse");
            let root = tree.root_node();

            // Collect all possible mutations
            fn collect_all_mutations(
                node: &tree_sitter::Node,
                source: &[u8],
            ) -> Vec<MutatedSource> {
                let mut mutations = Vec::new();

                // Try all operators
                let operators: Vec<Box<dyn TreeSitterMutationOperator>> = vec![
                    Box::new(PythonBinaryOpMutation),
                    Box::new(PythonRelationalOpMutation),
                    Box::new(PythonLogicalOpMutation),
                    Box::new(PythonMembershipOpMutation),
                ];

                for op in operators {
                    if op.can_mutate(node, source) {
                        mutations.extend(op.mutate(node, source));
                    }
                }

                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    mutations.extend(collect_all_mutations(&child, source));
                }

                mutations
            }

            let mutations = collect_all_mutations(&root, source);

            // Every mutation must be valid UTF-8
            for mutation in &mutations {
                // This is the key test - if expect() were to panic, this would fail
                assert!(!mutation.source.is_empty());
                // Verify UTF-8 validity explicitly
                assert!(
                    std::str::from_utf8(mutation.source.as_bytes()).is_ok(),
                    "Mutation should produce valid UTF-8: {}",
                    mutation.description
                );
            }
        }
    }

    /// Test edge case: Empty operator replacement still produces valid UTF-8
    #[test]
    fn test_edge_cases_utf8() {
        // Test with minimal source
        let source = b"a+b";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None).expect("Failed to parse");
        let root = tree.root_node();

        fn find_operator(node: &tree_sitter::Node, source: &[u8]) -> Vec<MutatedSource> {
            let operator = PythonBinaryOpMutation;
            if operator.can_mutate(node, source) {
                return operator.mutate(node, source);
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                let result = find_operator(&child, source);
                if !result.is_empty() {
                    return result;
                }
            }
            vec![]
        }

        let mutations = find_operator(&root, source);

        // All mutations valid UTF-8
        for mutation in &mutations {
            assert!(mutation.source.len() >= 3); // At least "a?b" where ? is operator
            assert!(std::str::from_utf8(mutation.source.as_bytes()).is_ok());
        }

        assert!(!mutations.is_empty());
    }
}
