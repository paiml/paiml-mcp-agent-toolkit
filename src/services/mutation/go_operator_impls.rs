// Go mutation operator trait implementations
// include!()'d from go_tree_sitter_mutations.rs

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
