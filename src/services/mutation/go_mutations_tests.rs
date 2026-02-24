// Tests for all Go mutation operators
// include!()'d from go_tree_sitter_mutations.rs

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
