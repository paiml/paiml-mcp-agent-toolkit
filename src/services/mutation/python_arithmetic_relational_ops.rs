// Python Arithmetic Operator Replacement (AOR) and Relational Operator Replacement (ROR)
// include!()'d from python_tree_sitter_mutations.rs

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
        let replacements = python_binary_op_replacements(op_text);
        if replacements.is_empty() {
            return vec![];
        }

        build_operator_mutations(source, &operator_node, op_text, &replacements)
    }

    fn kill_probability(&self) -> f64 {
        0.85 // Arithmetic mutations are usually caught by tests
    }
}

/// Returns the set of replacement operators for a given Python binary operator.
fn python_binary_op_replacements(op_text: &str) -> Vec<&'static str> {
    match op_text {
        "+" => vec!["-", "*", "/", "//", "%", "**"],
        "-" => vec!["+", "*", "/", "//", "%", "**"],
        "*" => vec!["+", "-", "/", "//", "%", "**"],
        "/" => vec!["+", "-", "*", "//", "%", "**"],
        "//" => vec!["+", "-", "*", "/", "%", "**"],
        "%" => vec!["+", "-", "*", "/", "//", "**"],
        "**" => vec!["+", "-", "*", "/", "//", "%"],
        _ => vec![],
    }
}

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
        let replacements = python_relational_op_replacements(op_text);
        if replacements.is_empty() {
            return vec![];
        }

        build_operator_mutations(source, &operator_node, op_text, &replacements)
    }

    fn kill_probability(&self) -> f64 {
        0.75 // Relational mutations sometimes survive
    }
}

/// Returns the set of replacement operators for a given Python relational operator.
fn python_relational_op_replacements(op_text: &str) -> Vec<&'static str> {
    match op_text {
        "<" => vec![">", "<=", ">=", "==", "!="],
        ">" => vec!["<", "<=", ">=", "==", "!="],
        "<=" => vec!["<", ">", ">=", "==", "!="],
        ">=" => vec!["<", ">", "<=", "==", "!="],
        "==" => vec!["!=", "<", ">", "<=", ">="],
        "!=" => vec!["==", "<", ">", "<=", ">="],
        _ => vec![],
    }
}

/// Build MutatedSource entries by splicing each replacement operator into the source.
fn build_operator_mutations(
    source: &[u8],
    operator_node: &Node,
    op_text: &str,
    replacements: &[&str],
) -> Vec<MutatedSource> {
    replacements
        .iter()
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
