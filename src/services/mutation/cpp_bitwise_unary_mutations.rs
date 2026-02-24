// C++ Bitwise and Unary Operator Mutations: BOR, UOR
// Included from cpp_tree_sitter_mutations.rs

// ============================================================
// 4. BITWISE OPERATOR REPLACEMENT (BOR)
// ============================================================

/// C++ Bitwise Operator Mutation (BOR - Bitwise Operator Replacement)
/// Replaces bitwise operators: &, |, ^, <<, >>
/// Also handles unary bitwise NOT: ~
pub struct CppBitwiseOpMutation;

impl TreeSitterMutationOperator for CppBitwiseOpMutation {
    fn name(&self) -> &str {
        "CppBitwiseOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        match node.kind() {
            "binary_expression" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    let kind = child.kind();
                    if matches!(kind, "&" | "|" | "^" | "<<" | ">>") {
                        return true;
                    }
                }
                false
            }
            "unary_expression" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "~" {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        if node.kind() == "unary_expression" {
            // Handle unary ~ operator
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "~" {
                    // For unary ~, we can't easily replace it with another operator
                    // Skip mutation for now (would require semantic analysis)
                    return vec![];
                }
            }
            return vec![];
        }

        // Handle binary bitwise operators
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
            "&" => vec!["|", "^"],
            "|" => vec!["&", "^"],
            "^" => vec!["&", "|"],
            "<<" => vec![">>"],
            ">>" => vec!["<<"],
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
}

// ============================================================
// 5. UNARY OPERATOR REPLACEMENT (UOR)
// ============================================================

/// C++ Unary Operator Mutation (UOR - Unary Operator Replacement)
/// Replaces unary operators: !, -, +
/// Also handles update expressions: ++, --
pub struct CppUnaryOpMutation;

impl TreeSitterMutationOperator for CppUnaryOpMutation {
    fn name(&self) -> &str {
        "CppUnaryOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        match node.kind() {
            "unary_expression" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    let kind = child.kind();
                    if matches!(kind, "!" | "-" | "+") {
                        return true;
                    }
                }
                false
            }
            "update_expression" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    let kind = child.kind();
                    if matches!(kind, "++" | "--") {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "!" | "-" | "+" | "++" | "--") {
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
            "!" => vec![], // Can't replace ! without changing semantics drastically
            "-" => vec!["+"],
            "+" => vec!["-"],
            "++" => vec!["--"],
            "--" => vec!["++"],
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
}
