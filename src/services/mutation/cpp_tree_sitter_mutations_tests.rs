// C++ Tree-Sitter Mutation Tests
// Included from cpp_tree_sitter_mutations.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_cpp(source: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_cpp::LANGUAGE.into())
            .expect("Failed to set C++ language");
        parser.parse(source, None).expect("Failed to parse C++")
    }

    // Helper to find a node of a specific kind (non-recursive for lifetime safety)
    fn find_node_by_kind_iter<'a>(
        tree: &'a tree_sitter::Tree,
        kind: &str,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut cursor = tree.walk();

        loop {
            let node = cursor.node();

            if node.kind() == kind {
                return Some(node);
            }

            // Try to go to first child
            if cursor.goto_first_child() {
                continue;
            }

            // Try to go to next sibling
            while !cursor.goto_next_sibling() {
                // Go up to parent and try next sibling
                if !cursor.goto_parent() {
                    return None; // Reached root, no match found
                }
            }
        }
    }

    // ============================================================
    // BINARY OPERATOR TESTS
    // ============================================================

    #[test]
    fn test_cpp_binary_addition() {
        let source = "int result = a + b;";
        let tree = parse_cpp(source);

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression")
            .expect("Should find binary_expression");

        let operator = CppBinaryOpMutation;
        assert!(operator.can_mutate(&binary_node, source.as_bytes()));

        let mutants = operator.mutate(&binary_node, source.as_bytes());
        assert_eq!(mutants.len(), 4); // +  → -, *, /, %

        assert!(mutants.iter().any(|m| m.source.contains("a - b")));
        assert!(mutants.iter().any(|m| m.source.contains("a * b")));
        assert!(mutants.iter().any(|m| m.source.contains("a / b")));
        assert!(mutants.iter().any(|m| m.source.contains("a % b")));
    }

    #[test]
    fn test_cpp_binary_subtraction() {
        let source = "int result = a - b;";
        let tree = parse_cpp(source);

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression")
            .expect("Should find binary_expression");

        let operator = CppBinaryOpMutation;
        let mutants = operator.mutate(&binary_node, source.as_bytes());
        assert_eq!(mutants.len(), 4); // - → +, *, /, %
    }

    // ============================================================
    // RELATIONAL OPERATOR TESTS
    // ============================================================

    #[test]
    fn test_cpp_relational_greater() {
        let source = "bool result = a > b;";
        let tree = parse_cpp(source);

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression")
            .expect("Should find binary_expression");

        let operator = CppRelationalOpMutation;
        assert!(operator.can_mutate(&binary_node, source.as_bytes()));

        let mutants = operator.mutate(&binary_node, source.as_bytes());
        assert_eq!(mutants.len(), 5); // > → <, <=, >=, ==, !=

        assert!(mutants.iter().any(|m| m.source.contains("a < b")));
        assert!(mutants.iter().any(|m| m.source.contains("a >= b")));
    }

    #[test]
    fn test_cpp_relational_less() {
        let source = "bool result = a < b;";
        let tree = parse_cpp(source);

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression")
            .expect("Should find binary_expression");

        let operator = CppRelationalOpMutation;
        let mutants = operator.mutate(&binary_node, source.as_bytes());
        assert_eq!(mutants.len(), 5); // < → >, <=, >=, ==, !=
    }

    // ============================================================
    // LOGICAL OPERATOR TESTS
    // ============================================================

    #[test]
    fn test_cpp_logical_and() {
        let source = "bool result = a && b;";
        let tree = parse_cpp(source);

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression")
            .expect("Should find binary_expression");

        let operator = CppLogicalOpMutation;
        assert!(operator.can_mutate(&binary_node, source.as_bytes()));

        let mutants = operator.mutate(&binary_node, source.as_bytes());
        assert_eq!(mutants.len(), 1); // && → ||
        assert!(mutants[0].source.contains("a || b"));
    }

    #[test]
    fn test_cpp_logical_or() {
        let source = "bool result = a || b;";
        let tree = parse_cpp(source);

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression")
            .expect("Should find binary_expression");

        let operator = CppLogicalOpMutation;
        let mutants = operator.mutate(&binary_node, source.as_bytes());
        assert_eq!(mutants.len(), 1); // || → &&
        assert!(mutants[0].source.contains("a && b"));
    }

    // ============================================================
    // BITWISE OPERATOR TESTS
    // ============================================================

    #[test]
    fn test_cpp_bitwise_and() {
        let source = "int result = a & b;";
        let tree = parse_cpp(source);

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression")
            .expect("Should find binary_expression");

        let operator = CppBitwiseOpMutation;
        assert!(operator.can_mutate(&binary_node, source.as_bytes()));

        let mutants = operator.mutate(&binary_node, source.as_bytes());
        assert_eq!(mutants.len(), 2); // & → |, ^
        assert!(mutants.iter().any(|m| m.source.contains("a | b")));
        assert!(mutants.iter().any(|m| m.source.contains("a ^ b")));
    }

    #[test]
    fn test_cpp_bitwise_not() {
        let source = "int result = ~a;";
        let tree = parse_cpp(source);

        let unary_node = find_node_by_kind_iter(&tree, "unary_expression")
            .expect("Should find unary_expression");

        let operator = CppBitwiseOpMutation;
        assert!(operator.can_mutate(&unary_node, source.as_bytes()));

        let mutants = operator.mutate(&unary_node, source.as_bytes());
        // Bitwise NOT is difficult to mutate, so we skip it
        assert_eq!(mutants.len(), 0);
    }

    // ============================================================
    // UNARY OPERATOR TESTS
    // ============================================================

    #[test]
    fn test_cpp_unary_not() {
        let source = "bool result = !flag;";
        let tree = parse_cpp(source);

        let unary_node = find_node_by_kind_iter(&tree, "unary_expression")
            .expect("Should find unary_expression");

        let operator = CppUnaryOpMutation;
        assert!(operator.can_mutate(&unary_node, source.as_bytes()));

        let mutants = operator.mutate(&unary_node, source.as_bytes());
        // Logical NOT is difficult to mutate, so we skip it
        assert_eq!(mutants.len(), 0);
    }

    #[test]
    fn test_cpp_unary_negate() {
        let source = "int result = -value;";
        let tree = parse_cpp(source);

        let unary_node = find_node_by_kind_iter(&tree, "unary_expression")
            .expect("Should find unary_expression");

        let operator = CppUnaryOpMutation;
        let mutants = operator.mutate(&unary_node, source.as_bytes());
        assert_eq!(mutants.len(), 1); // - → +
        assert!(mutants[0].source.contains("+value"));
    }

    #[test]
    fn test_cpp_pre_increment() {
        let source = "int result = ++i;";
        let tree = parse_cpp(source);

        let update_node = find_node_by_kind_iter(&tree, "update_expression")
            .expect("Should find update_expression");

        let operator = CppUnaryOpMutation;
        assert!(operator.can_mutate(&update_node, source.as_bytes()));

        let mutants = operator.mutate(&update_node, source.as_bytes());
        assert_eq!(mutants.len(), 1); // ++ → --
        assert!(mutants[0].source.contains("--i"));
    }

    #[test]
    fn test_cpp_post_increment() {
        let source = "int result = i++;";
        let tree = parse_cpp(source);

        let update_node = find_node_by_kind_iter(&tree, "update_expression")
            .expect("Should find update_expression");

        let operator = CppUnaryOpMutation;
        let mutants = operator.mutate(&update_node, source.as_bytes());
        assert_eq!(mutants.len(), 1); // ++ → --
        assert!(mutants[0].source.contains("i--"));
    }

    // ============================================================
    // POINTER OPERATOR TESTS (C++ SPECIFIC)
    // ============================================================

    #[test]
    fn test_cpp_pointer_dereference() {
        let source = "int value = *ptr;";
        let tree = parse_cpp(source);

        let pointer_node = find_node_by_kind_iter(&tree, "pointer_expression")
            .expect("Should find pointer_expression");

        let operator = CppPointerOpMutation;
        assert!(operator.can_mutate(&pointer_node, source.as_bytes()));

        let mutants = operator.mutate(&pointer_node, source.as_bytes());
        // Pointer operator mutations are complex, we skip them
        assert_eq!(mutants.len(), 0);
    }

    #[test]
    fn test_cpp_pointer_address_of() {
        let source = "int* ptr = &value;";
        let tree = parse_cpp(source);

        let pointer_node = find_node_by_kind_iter(&tree, "pointer_expression");
        if let Some(node) = pointer_node {
            let operator = CppPointerOpMutation;
            assert!(operator.can_mutate(&node, source.as_bytes()));
        }
    }

    #[test]
    fn test_cpp_pointer_arrow() {
        let source = "obj->method();";
        let tree = parse_cpp(source);

        let field_node = find_node_by_kind_iter(&tree, "field_expression");
        if let Some(node) = field_node {
            let operator = CppPointerOpMutation;
            assert!(operator.can_mutate(&node, source.as_bytes()));

            let mutants = operator.mutate(&node, source.as_bytes());
            // Arrow operator mutations are complex, we skip them
            assert_eq!(mutants.len(), 0);
        }
    }

    // ============================================================
    // MEMBER ACCESS TESTS (C++ SPECIFIC)
    // ============================================================

    #[test]
    fn test_cpp_member_dot_access() {
        let source = "obj.member = 10;";
        let tree = parse_cpp(source);

        let field_node = find_node_by_kind_iter(&tree, "field_expression");
        if let Some(node) = field_node {
            let operator = CppMemberAccessMutation;
            assert!(operator.can_mutate(&node, source.as_bytes()));

            let mutants = operator.mutate(&node, source.as_bytes());
            // Member access mutations are complex, we skip them
            assert_eq!(mutants.len(), 0);
        }
    }

    #[test]
    fn test_cpp_member_scope_resolution() {
        let source = "Class::staticMethod();";
        let tree = parse_cpp(source);

        let qualified_node = find_node_by_kind_iter(&tree, "qualified_identifier");
        if let Some(node) = qualified_node {
            let operator = CppMemberAccessMutation;
            assert!(operator.can_mutate(&node, source.as_bytes()));

            let mutants = operator.mutate(&node, source.as_bytes());
            // Scope resolution mutations are complex, we skip them
            assert_eq!(mutants.len(), 0);
        }
    }

    /// Test UTF-8 validity for C++ mutations (validates expect() calls)
    #[test]
    fn test_cpp_utf8_validity() {
        let source = "int result = a + b;";
        let tree = parse_cpp(source);

        let bin_node = find_node_by_kind_iter(&tree, "binary_expression");
        if let Some(node) = bin_node {
            let operator = CppBinaryOpMutation;
            let mutations = operator.mutate(&node, source.as_bytes());

            for mutation in &mutations {
                assert!(!mutation.source.is_empty());
                assert!(std::str::from_utf8(mutation.source.as_bytes()).is_ok());
            }

            assert!(!mutations.is_empty());
        }
    }

    /// Test all C++ operators produce valid UTF-8
    #[test]
    fn test_all_cpp_operators_utf8() {
        let test_cases = vec!["a + b", "x == y", "a && b", "a += b", "ptr->field"];

        for source in test_cases {
            let tree = parse_cpp(source);
            let root = tree.root_node();

            fn collect_all(node: &tree_sitter::Node, source: &[u8]) -> Vec<MutatedSource> {
                let mut mutations = Vec::new();

                let operators: Vec<Box<dyn TreeSitterMutationOperator>> = vec![
                    Box::new(CppBinaryOpMutation),
                    Box::new(CppRelationalOpMutation),
                    Box::new(CppLogicalOpMutation),
                    Box::new(CppBitwiseOpMutation),
                    Box::new(CppPointerOpMutation),
                    Box::new(CppMemberAccessMutation),
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

            let mutations = collect_all(&root, source.as_bytes());

            for mutation in &mutations {
                assert!(
                    std::str::from_utf8(mutation.source.as_bytes()).is_ok(),
                    "Mutation should produce valid UTF-8: {}",
                    mutation.description
                );
            }
        }
    }
}
