// C++ Pointer and Member Access Mutations: POR, MAR
// Included from cpp_tree_sitter_mutations.rs

// ============================================================
// 6. POINTER OPERATOR REPLACEMENT (POR) - C++ SPECIFIC
// ============================================================

/// C++ Pointer Operator Mutation (POR - Pointer Operator Replacement)
/// Handles pointer-specific operators: *, &, ->
pub struct CppPointerOpMutation;

impl TreeSitterMutationOperator for CppPointerOpMutation {
    fn name(&self) -> &str {
        "CppPointerOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        match node.kind() {
            "pointer_expression" => true, // * or & in expression context
            "field_expression" => {
                // Check if it's an arrow operator
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "->" {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        if node.kind() == "pointer_expression" {
            // Find the operator (* or &)
            let mut cursor = node.walk();
            let mut operator_node = None;

            for child in node.children(&mut cursor) {
                let kind = child.kind();
                if matches!(kind, "*" | "&") {
                    operator_node = Some(child);
                    break;
                }
            }

            let operator_node = match operator_node {
                Some(n) => n,
                None => return vec![],
            };

            let op_bytes = &source[operator_node.byte_range()];
            let _op_text = std::str::from_utf8(op_bytes).unwrap_or("");

            // For pointer operators, mutation is tricky
            // * (dereference) and & (address-of) can't be simply swapped
            // We'll skip mutation for now to avoid semantic errors
            return vec![];
        }

        if node.kind() == "field_expression" {
            // Find the -> operator
            let mut cursor = node.walk();
            let mut operator_node = None;

            for child in node.children(&mut cursor) {
                if child.kind() == "->" {
                    operator_node = Some(child);
                    break;
                }
            }

            let _operator_node = match operator_node {
                Some(n) => n,
                None => return vec![],
            };

            // Mutating -> to . would require (*ptr).member
            // This is semantically equivalent but syntactically complex
            // Skip mutation for now
            return vec![];
        }

        vec![]
    }
}

// ============================================================
// 7. MEMBER ACCESS REPLACEMENT (MAR) - C++ SPECIFIC
// ============================================================

/// C++ Member Access Mutation (MAR - Member Access Replacement)
/// Handles member access operators: ., ::
pub struct CppMemberAccessMutation;

impl TreeSitterMutationOperator for CppMemberAccessMutation {
    fn name(&self) -> &str {
        "CppMemberAccess"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        match node.kind() {
            "field_expression" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "." {
                        return true;
                    }
                }
                false
            }
            "qualified_identifier" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "::" {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn mutate(&self, _node: &Node, _source: &[u8]) -> Vec<MutatedSource> {
        // Member access mutations are semantically complex
        // . and :: have different meanings (instance vs static/namespace)
        // Can't mutate without type information
        // Skip mutation for now
        vec![]
    }
}
