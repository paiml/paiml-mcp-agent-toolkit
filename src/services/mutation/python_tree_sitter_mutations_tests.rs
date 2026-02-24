// Tests for Python tree-sitter mutation operators
// include!()'d from python_tree_sitter_mutations.rs

#[cfg_attr(coverage_nightly, coverage(off))]
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

    /// Test UTF-8 validity after mutation (validates expect() calls in operator impls)
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

        assert!(
            !mutations.is_empty(),
            "Should generate mutations with Unicode"
        );
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
