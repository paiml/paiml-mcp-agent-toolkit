// Python Logical (LOR), Identity, and Membership operator mutations
// include!()'d from python_tree_sitter_mutations.rs

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
        0.80 // Logical mutations are usually caught
    }
}

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
                mutate_is_not_to_alternatives(source, is_n, not_n)
            }
            (Some(is_n), None) => {
                mutate_is_to_alternatives(source, is_n)
            }
            _ => vec![],
        }
    }

    fn kill_probability(&self) -> f64 {
        0.70 // Identity mutations can survive
    }
}

/// Mutate "is not" to "is" and "=="
fn mutate_is_not_to_alternatives(
    source: &[u8],
    is_n: &Node,
    not_n: &Node,
) -> Vec<MutatedSource> {
    let mut mutations = Vec::new();
    let start = is_n.start_byte();
    let end = not_n.end_byte();

    // Mutation 1: Remove "not" to get just "is"
    let mut mutated = source.to_vec();
    mutated.splice(start..end, b"is".iter().copied());
    mutations.push(MutatedSource {
        source: String::from_utf8(mutated).expect(
            "mutated source is valid UTF-8 (original source + ASCII operators)",
        ),
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
        source: String::from_utf8(mutated).expect(
            "mutated source is valid UTF-8 (original source + ASCII operators)",
        ),
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

/// Mutate "is" to "is not" and "=="
fn mutate_is_to_alternatives(source: &[u8], is_n: &Node) -> Vec<MutatedSource> {
    vec![
        MutatedSource {
            source: {
                let mut mutated = source.to_vec();
                mutated.splice(is_n.byte_range(), b"is not".iter().copied());
                String::from_utf8(mutated).expect(
                    "mutated source is valid UTF-8 (original source + ASCII operators)",
                )
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
                String::from_utf8(mutated).expect(
                    "mutated source is valid UTF-8 (original source + ASCII operators)",
                )
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
                mutate_not_in_to_in(source, in_n, not_n)
            }
            (Some(in_n), None) => {
                mutate_in_to_not_in(source, in_n)
            }
            _ => vec![],
        }
    }

    fn kill_probability(&self) -> f64 {
        0.75 // Membership mutations are usually caught
    }
}

/// Mutate "not in" to "in"
fn mutate_not_in_to_in(
    source: &[u8],
    in_n: &Node,
    not_n: &Node,
) -> Vec<MutatedSource> {
    let start = not_n.start_byte();
    let end = in_n.end_byte();
    let mut mutated = source.to_vec();
    mutated.splice(start..end, b"in".iter().copied());
    vec![MutatedSource {
        source: String::from_utf8(mutated).expect(
            "mutated source is valid UTF-8 (original source + ASCII operators)",
        ),
        description: "not in → in".to_string(),
        location: SourceLocation {
            line: not_n.start_position().row + 1,
            column: not_n.start_position().column + 1,
            end_line: in_n.end_position().row + 1,
            end_column: in_n.end_position().column + 1,
        },
    }]
}

/// Mutate "in" to "not in"
fn mutate_in_to_not_in(source: &[u8], in_n: &Node) -> Vec<MutatedSource> {
    let mut mutated = source.to_vec();
    mutated.splice(in_n.byte_range(), b"not in".iter().copied());
    vec![MutatedSource {
        source: String::from_utf8(mutated).expect(
            "mutated source is valid UTF-8 (original source + ASCII operators)",
        ),
        description: "in → not in".to_string(),
        location: SourceLocation {
            line: in_n.start_position().row + 1,
            column: in_n.start_position().column + 1,
            end_line: in_n.end_position().row + 1,
            end_column: in_n.end_position().column + 1,
        },
    }]
}
