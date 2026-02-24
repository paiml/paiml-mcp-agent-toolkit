// C++ Binary Operator Mutations: AOR, ROR, LOR
// Included from cpp_tree_sitter_mutations.rs

// ============================================================
// 1. BINARY OPERATOR REPLACEMENT (AOR)
// ============================================================

/// C++ Binary Operator Mutation (AOR - Arithmetic Operator Replacement)
/// Replaces arithmetic operators: +, -, *, /, %
pub struct CppBinaryOpMutation;

impl TreeSitterMutationOperator for CppBinaryOpMutation {
    fn name(&self) -> &str {
        "CppBinaryOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        if node.kind() != "binary_expression" {
            return false;
        }

        // Find operator child node
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "+" | "-" | "*" | "/" | "%") {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Find operator child node
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
}

// ============================================================
// 2. RELATIONAL OPERATOR REPLACEMENT (ROR)
// ============================================================

/// C++ Relational Operator Mutation (ROR - Relational Operator Replacement)
/// Replaces comparison operators: <, >, <=, >=, ==, !=
pub struct CppRelationalOpMutation;

impl TreeSitterMutationOperator for CppRelationalOpMutation {
    fn name(&self) -> &str {
        "CppRelationalOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        if node.kind() != "binary_expression" {
            return false;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "<" | ">" | "<=" | ">=" | "==" | "!=") {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
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
            "<" => vec![">", "<=", ">=", "==", "!="],
            ">" => vec!["<", "<=", ">=", "==", "!="],
            "<=" => vec!["<", ">", ">=", "==", "!="],
            ">=" => vec!["<", ">", "<=", "==", "!="],
            "==" => vec!["<", ">", "<=", ">=", "!="],
            "!=" => vec!["<", ">", "<=", ">=", "=="],
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
// 3. LOGICAL OPERATOR REPLACEMENT (LOR)
// ============================================================

/// C++ Logical Operator Mutation (LOR - Logical Operator Replacement)
/// Replaces logical operators: &&, ||
pub struct CppLogicalOpMutation;

impl TreeSitterMutationOperator for CppLogicalOpMutation {
    fn name(&self) -> &str {
        "CppLogicalOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        if node.kind() != "binary_expression" {
            return false;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "&&" | "||") {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
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

        let new_op = match op_text {
            "&&" => "||",
            "||" => "&&",
            _ => return vec![],
        };

        let mut mutated = source.to_vec();
        mutated.splice(operator_node.byte_range(), new_op.bytes());

        vec![MutatedSource {
            source: String::from_utf8(mutated)
                .expect("mutated source is valid UTF-8 (original source + ASCII operators)"),
            description: format!("{} → {}", op_text, new_op),
            location: SourceLocation {
                line: operator_node.start_position().row + 1,
                column: operator_node.start_position().column + 1,
                end_line: operator_node.end_position().row + 1,
                end_column: operator_node.end_position().column + 1,
            },
        }]
    }
}
