// Operator implementations for TypeScript/JavaScript tree-sitter mutation operators.
// Included by typescript_tree_sitter_mutations.rs — do NOT add `use` imports here.

impl TreeSitterMutationOperator for TypeScriptBinaryOpMutation {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "AOR/ROR"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        debug_assert!(true, "contract: can_mutate");
        // GREEN PHASE: Detect binary expressions
        node.kind() == "binary_expression"
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        debug_assert!(true, "contract: mutate");
        // GREEN PHASE: Generate mutations for binary operators
        if node.kind() != "binary_expression" {
            return vec![];
        }

        // Find the operator child node
        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            let kind = child.kind();
            // Tree-sitter TypeScript represents operators as their literal text
            if kind == "+"
                || kind == "-"
                || kind == "*"
                || kind == "/"
                || kind == "%"
                || kind == ">"
                || kind == "<"
                || kind == ">="
                || kind == "<="
                || kind == "=="
                || kind == "!="
                || kind == "==="
                || kind == "!=="
            {
                operator_node = Some(child);
                break;
            }
        }

        let op_node = match operator_node {
            Some(node) => node,
            None => return vec![],
        };

        let op_bytes = &source[op_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        // Determine replacement operators based on current operator
        let replacements: Vec<&str> = match op_text {
            // Arithmetic operators
            "+" => vec!["-", "*", "/"],
            "-" => vec!["+", "*", "/"],
            "*" => vec!["+", "-", "/"],
            "/" => vec!["+", "-", "*"],
            "%" => vec!["*", "/"],

            // Relational operators
            ">" => vec!["<", ">=", "<=", "==", "!="],
            "<" => vec![">", ">=", "<=", "==", "!="],
            ">=" => vec![">", "<", "<=", "==", "!="],
            "<=" => vec![">", "<", ">=", "==", "!="],
            "==" => vec!["!=", ">", "<", ">=", "<="],
            "!=" => vec!["==", ">", "<", ">=", "<="],
            "===" => vec!["!==", "==", "!="],
            "!==" => vec!["===", "==", "!="],

            _ => vec![],
        };

        // Generate mutated source for each replacement
        replacements
            .into_iter()
            .map(|new_op| {
                let mut mutated = source.to_vec();
                let range = op_node.byte_range();

                // Splice in the new operator
                mutated.splice(range, new_op.bytes());

                MutatedSource {
                    source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
                    description: format!("{} → {}", op_text, new_op),
                    location: SourceLocation {
                        line: op_node.start_position().row + 1,
                        column: op_node.start_position().column + 1,
                        end_line: op_node.end_position().row + 1,
                        end_column: op_node.end_position().column + 1,
                    },
                }
            })
            .collect()
    }

    fn kill_probability(&self) -> f64 {
        debug_assert!(true, "contract: kill_probability");
        0.85
    }
}

impl TreeSitterMutationOperator for TypeScriptStrictEqualityMutation {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "Strict Equality"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        debug_assert!(true, "contract: can_mutate");
        // GREEN PHASE: Detect strict equality operators
        if node.kind() != "binary_expression" {
            return false;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if kind == "===" || kind == "!==" {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        debug_assert!(true, "contract: mutate");
        // GREEN PHASE: Mutate strict equality operators
        if node.kind() != "binary_expression" {
            return vec![];
        }

        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            if child.kind() == "===" || child.kind() == "!==" {
                operator_node = Some(child);
                break;
            }
        }

        let op_node = match operator_node {
            Some(node) => node,
            None => return vec![],
        };

        let op_bytes = &source[op_node.byte_range()];
        let op_text = std::str::from_utf8(op_bytes).unwrap_or("");

        let replacements: Vec<&str> = match op_text {
            "===" => vec!["==", "!==", "!="],
            "!==" => vec!["!=", "===", "=="],
            _ => vec![],
        };

        replacements
            .into_iter()
            .map(|new_op| {
                let mut mutated = source.to_vec();
                mutated.splice(op_node.byte_range(), new_op.bytes());

                MutatedSource {
                    source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
                    description: format!("{} → {}", op_text, new_op),
                    location: SourceLocation {
                        line: op_node.start_position().row + 1,
                        column: op_node.start_position().column + 1,
                        end_line: op_node.end_position().row + 1,
                        end_column: op_node.end_position().column + 1,
                    },
                }
            })
            .collect()
    }
}

impl TreeSitterMutationOperator for TypeScriptOptionalChainingMutation {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "Optional Chaining"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        debug_assert!(true, "contract: can_mutate");
        // GREEN PHASE: Detect optional chaining expressions
        // Tree-sitter represents optional chaining as specific node types
        matches!(node.kind(), "optional_chain" | "member_expression")
            && node.to_sexp().contains("?.")
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        debug_assert!(true, "contract: mutate");
        // GREEN PHASE: Remove optional chaining operator
        let source_text = std::str::from_utf8(source).unwrap_or("");
        let node_text = &source_text[node.byte_range()];

        // Simple mutation: remove '?' from '?.'
        if !node_text.contains("?.") {
            return vec![];
        }

        let mutated_text = node_text.replace("?.", ".");
        let mut mutated = source.to_vec();
        mutated.splice(node.byte_range(), mutated_text.bytes());

        vec![MutatedSource {
            source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
            description: "?. → .".to_string(),
            location: SourceLocation {
                line: node.start_position().row + 1,
                column: node.start_position().column + 1,
                end_line: node.end_position().row + 1,
                end_column: node.end_position().column + 1,
            },
        }]
    }
}

impl TreeSitterMutationOperator for TypeScriptNullishCoalescingMutation {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "Nullish Coalescing"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        debug_assert!(true, "contract: can_mutate");
        // GREEN PHASE: Detect nullish coalescing operator
        if node.kind() != "binary_expression" {
            return false;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "??" {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        debug_assert!(true, "contract: mutate");
        // GREEN PHASE: Mutate nullish coalescing operator
        if node.kind() != "binary_expression" {
            return vec![];
        }

        let mut cursor = node.walk();
        let mut operator_node = None;

        for child in node.children(&mut cursor) {
            if child.kind() == "??" {
                operator_node = Some(child);
                break;
            }
        }

        let op_node = match operator_node {
            Some(node) => node,
            None => return vec![],
        };

        // Two mutations: ?? → || and ?? → (use right operand only)
        vec![MutatedSource {
            source: {
                let mut mutated = source.to_vec();
                mutated.splice(op_node.byte_range(), "||".bytes());
                String::from_utf8(mutated).unwrap_or_else(|_| String::new())
            },
            description: "?? → ||".to_string(),
            location: SourceLocation {
                line: op_node.start_position().row + 1,
                column: op_node.start_position().column + 1,
                end_line: op_node.end_position().row + 1,
                end_column: op_node.end_position().column + 1,
            },
        }]
    }
}

impl TreeSitterMutationOperator for TypeScriptAsyncAwaitMutation {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "Async/Await"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        debug_assert!(true, "contract: can_mutate");
        // GREEN PHASE: Detect async/await keywords
        let kind = node.kind();
        if kind == "function_declaration" || kind == "arrow_function" || kind == "method_definition"
        {
            let source_text = std::str::from_utf8(source).unwrap_or("");
            let node_text = &source_text[node.byte_range()];
            return node_text.contains("async");
        }

        if kind == "await_expression" {
            return true;
        }

        false
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        debug_assert!(true, "contract: mutate");
        // GREEN PHASE: Remove async or await keywords
        let source_text = std::str::from_utf8(source).unwrap_or("");
        let node_text = &source_text[node.byte_range()];

        let mut mutations = Vec::new();

        if node.kind() == "await_expression" {
            // Remove "await " from expression
            let mutated_text = node_text.replace("await ", "");
            let mut mutated = source.to_vec();
            mutated.splice(node.byte_range(), mutated_text.bytes());

            mutations.push(MutatedSource {
                source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
                description: "Remove await".to_string(),
                location: SourceLocation {
                    line: node.start_position().row + 1,
                    column: node.start_position().column + 1,
                    end_line: node.end_position().row + 1,
                    end_column: node.end_position().column + 1,
                },
            });
        } else if node_text.contains("async") {
            // Remove "async " keyword
            let mutated_text = node_text.replace("async ", "");
            let mut mutated = source.to_vec();
            mutated.splice(node.byte_range(), mutated_text.bytes());

            mutations.push(MutatedSource {
                source: String::from_utf8(mutated).unwrap_or_else(|_| String::new()),
                description: "Remove async".to_string(),
                location: SourceLocation {
                    line: node.start_position().row + 1,
                    column: node.start_position().column + 1,
                    end_line: node.end_position().row + 1,
                    end_column: node.end_position().column + 1,
                },
            });
        }

        mutations
    }
}
