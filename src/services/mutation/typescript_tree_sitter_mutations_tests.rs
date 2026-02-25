// Tests for TypeScript/JavaScript tree-sitter mutation operators.
// Included by typescript_tree_sitter_mutations.rs — do NOT add `use` imports here.

#[cfg_attr(coverage_nightly, coverage(off))]
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

    fn find_binary_expression(tree: &Tree) -> Option<Node<'_>> {
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
        assert!(
            mutants.len() >= 3,
            "Expected at least 3 mutants, got {}",
            mutants.len()
        );
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
            mutants
                .iter()
                .any(|m| m.source.contains("return api()") && !m.source.contains("await")),
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
