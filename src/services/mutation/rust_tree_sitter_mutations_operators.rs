// ============================================================
// 1. BINARY OPERATOR REPLACEMENT (AOR)
// ============================================================

impl TreeSitterMutationOperator for RustBinaryOpMutation {
    fn name(&self) -> &str {
        "RustBinaryOp"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() != "binary_expression" {
            return false;
        }

        // Check if this is an arithmetic operator
        if let Some(operator_node) = node.child_by_field_name("operator") {
            let op = &source[operator_node.byte_range()];
            matches!(op, b"+" | b"-" | b"*" | b"/" | b"%")
        } else {
            false
        }
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        let mut mutations = Vec::new();

        if let Some(operator_node) = node.child_by_field_name("operator") {
            let original_op = &source[operator_node.byte_range()];

            // Define mutation mappings for arithmetic operators
            let replacements: &[&[u8]] = match original_op {
                b"+" => &[b"-", b"*", b"/", b"%"],
                b"-" => &[b"+", b"*", b"/", b"%"],
                b"*" => &[b"+", b"-", b"/", b"%"],
                b"/" => &[b"+", b"-", b"*", b"%"],
                b"%" => &[b"+", b"-", b"*", b"/"],
                _ => &[],
            };

            // Generate mutations by splicing
            for replacement in replacements {
                let mut mutated = Vec::new();
                mutated.extend_from_slice(&source[..operator_node.start_byte()]);
                mutated.extend_from_slice(replacement);
                mutated.extend_from_slice(&source[operator_node.end_byte()..]);

                let description = format!(
                    "{} → {}",
                    String::from_utf8_lossy(original_op),
                    String::from_utf8_lossy(replacement)
                );

                mutations.push(MutatedSource {
                    source: String::from_utf8_lossy(&mutated).into_owned(),
                    description,
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                });
            }
        }

        mutations
    }
}

// ============================================================
// 2. RELATIONAL OPERATOR REPLACEMENT (ROR)
// ============================================================

impl TreeSitterMutationOperator for RustRelationalOpMutation {
    fn name(&self) -> &str {
        "RustRelationalOp"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() != "binary_expression" {
            return false;
        }

        // Check if this is a relational operator
        if let Some(operator_node) = node.child_by_field_name("operator") {
            let op = &source[operator_node.byte_range()];
            matches!(op, b">" | b"<" | b">=" | b"<=" | b"==" | b"!=")
        } else {
            false
        }
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        let mut mutations = Vec::new();

        if let Some(operator_node) = node.child_by_field_name("operator") {
            let original_op = &source[operator_node.byte_range()];

            // Define mutation mappings for relational operators
            let replacements: &[&[u8]] = match original_op {
                b">" => &[b"<", b">=", b"<=", b"==", b"!="],
                b"<" => &[b">", b">=", b"<=", b"==", b"!="],
                b">=" => &[b">", b"<", b"<=", b"==", b"!="],
                b"<=" => &[b">", b"<", b">=", b"==", b"!="],
                b"==" => &[b"!=", b">", b"<", b">=", b"<="],
                b"!=" => &[b"==", b">", b"<", b">=", b"<="],
                _ => &[],
            };

            // Generate mutations by splicing
            for replacement in replacements {
                let mut mutated = Vec::new();
                mutated.extend_from_slice(&source[..operator_node.start_byte()]);
                mutated.extend_from_slice(replacement);
                mutated.extend_from_slice(&source[operator_node.end_byte()..]);

                let description = format!(
                    "{} → {}",
                    String::from_utf8_lossy(original_op),
                    String::from_utf8_lossy(replacement)
                );

                mutations.push(MutatedSource {
                    source: String::from_utf8_lossy(&mutated).into_owned(),
                    description,
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                });
            }
        }

        mutations
    }
}

// ============================================================
// 3. LOGICAL OPERATOR REPLACEMENT (LOR)
// ============================================================

impl TreeSitterMutationOperator for RustLogicalOpMutation {
    fn name(&self) -> &str {
        "RustLogicalOp"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() != "binary_expression" {
            return false;
        }

        // Check if this is a logical operator
        if let Some(operator_node) = node.child_by_field_name("operator") {
            let op = &source[operator_node.byte_range()];
            matches!(op, b"&&" | b"||")
        } else {
            false
        }
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        let mut mutations = Vec::new();

        if let Some(operator_node) = node.child_by_field_name("operator") {
            let original_op = &source[operator_node.byte_range()];

            // Define mutation mappings for logical operators
            let replacements: &[&[u8]] = match original_op {
                b"&&" => &[b"||"],
                b"||" => &[b"&&"],
                _ => &[],
            };

            // Generate mutations by splicing
            for replacement in replacements {
                let mut mutated = Vec::new();
                mutated.extend_from_slice(&source[..operator_node.start_byte()]);
                mutated.extend_from_slice(replacement);
                mutated.extend_from_slice(&source[operator_node.end_byte()..]);

                let description = format!(
                    "{} → {}",
                    String::from_utf8_lossy(original_op),
                    String::from_utf8_lossy(replacement)
                );

                mutations.push(MutatedSource {
                    source: String::from_utf8_lossy(&mutated).into_owned(),
                    description,
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                });
            }
        }

        mutations
    }
}

// ============================================================
// 4. BITWISE OPERATOR REPLACEMENT (BOR)
// ============================================================

impl TreeSitterMutationOperator for RustBitwiseOpMutation {
    fn name(&self) -> &str {
        "RustBitwiseOp"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() != "binary_expression" {
            return false;
        }

        // Check if this is a bitwise operator
        if let Some(operator_node) = node.child_by_field_name("operator") {
            let op = &source[operator_node.byte_range()];
            matches!(op, b"&" | b"|" | b"^" | b"<<" | b">>")
        } else {
            false
        }
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        let mut mutations = Vec::new();

        if let Some(operator_node) = node.child_by_field_name("operator") {
            let original_op = &source[operator_node.byte_range()];

            // Define mutation mappings for bitwise operators
            let replacements: &[&[u8]] = match original_op {
                b"&" => &[b"|", b"^"],
                b"|" => &[b"&", b"^"],
                b"^" => &[b"&", b"|"],
                b"<<" => &[b">>"],
                b">>" => &[b"<<"],
                _ => &[],
            };

            // Generate mutations by splicing
            for replacement in replacements {
                let mut mutated = Vec::new();
                mutated.extend_from_slice(&source[..operator_node.start_byte()]);
                mutated.extend_from_slice(replacement);
                mutated.extend_from_slice(&source[operator_node.end_byte()..]);

                let description = format!(
                    "{} → {}",
                    String::from_utf8_lossy(original_op),
                    String::from_utf8_lossy(replacement)
                );

                mutations.push(MutatedSource {
                    source: String::from_utf8_lossy(&mutated).into_owned(),
                    description,
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                });
            }
        }

        mutations
    }
}

// ============================================================
// 5. RANGE OPERATOR REPLACEMENT (RANGEOR) - RUST-SPECIFIC
// ============================================================

impl TreeSitterMutationOperator for RustRangeOpMutation {
    fn name(&self) -> &str {
        "RustRangeOp"
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        // Check for range expressions
        match node.kind() {
            "range_expression" | "inclusive_range_expression" => {
                // Verify it has an operator
                node.child_by_field_name("operator").is_some()
            }
            _ => false,
        }
    }

    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        let mut mutations = Vec::new();

        if let Some(operator_node) = node.child_by_field_name("operator") {
            let original_op = &source[operator_node.byte_range()];

            // Define mutation mappings for range operators
            let replacements: &[&[u8]] = match original_op {
                b".." => &[b"..="], // Exclusive to inclusive
                b"..=" => &[b".."], // Inclusive to exclusive
                _ => &[],
            };

            // Generate mutations by splicing
            for replacement in replacements {
                let mut mutated = Vec::new();
                mutated.extend_from_slice(&source[..operator_node.start_byte()]);
                mutated.extend_from_slice(replacement);
                mutated.extend_from_slice(&source[operator_node.end_byte()..]);

                let description = format!(
                    "{} → {}",
                    String::from_utf8_lossy(original_op),
                    String::from_utf8_lossy(replacement)
                );

                mutations.push(MutatedSource {
                    source: String::from_utf8_lossy(&mutated).into_owned(),
                    description,
                    location: SourceLocation {
                        line: operator_node.start_position().row + 1,
                        column: operator_node.start_position().column + 1,
                        end_line: operator_node.end_position().row + 1,
                        end_column: operator_node.end_position().column + 1,
                    },
                });
            }
        }

        mutations
    }
}

// ============================================================
// 6. PATTERN MATCH REPLACEMENT (PMR) - RUST-SPECIFIC
// ============================================================

impl TreeSitterMutationOperator for RustPatternMutation {
    fn name(&self) -> &str {
        "RustPattern"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        // Detection-only: Identify pattern matching constructs
        // Actual mutation would require type inference
        match node.kind() {
            "match_expression" => true,
            "match_arm" => {
                // Check for Option/Result patterns
                let text = &source[node.byte_range()];
                let text_str = std::str::from_utf8(text).unwrap_or("");
                text_str.contains("Some")
                    || text_str.contains("None")
                    || text_str.contains("Ok")
                    || text_str.contains("Err")
            }
            _ => false,
        }
    }

    fn mutate(&self, _node: &Node, _source: &[u8]) -> Vec<MutatedSource> {
        // Detection-only: Pattern matching mutations would require type inference
        // to ensure mutants are semantically valid (Some -> None requires compatible types)
        // Return empty mutations for now
        Vec::new()
    }
}

// ============================================================
// 7. METHOD CHAIN REPLACEMENT (MCR) - RUST-SPECIFIC
// ============================================================

impl TreeSitterMutationOperator for RustMethodChainMutation {
    fn name(&self) -> &str {
        "RustMethodChain"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        // Detection-only: Identify method call chains
        if node.kind() != "call_expression" {
            return false;
        }

        // Check for common iterator methods
        if let Some(function_node) = node.child_by_field_name("function") {
            if function_node.kind() == "field_expression" {
                if let Some(field_node) = function_node.child_by_field_name("field") {
                    let field_text = &source[field_node.byte_range()];
                    let field_str = std::str::from_utf8(field_text).unwrap_or("");
                    matches!(
                        field_str,
                        "map" | "filter" | "collect" | "fold" | "for_each" | "find"
                    )
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn mutate(&self, _node: &Node, _source: &[u8]) -> Vec<MutatedSource> {
        // Detection-only: Method chain mutations would require type inference
        // to ensure the replacement method has compatible signatures
        // Return empty mutations for now
        Vec::new()
    }
}

// ============================================================
// 8. BORROW/REFERENCE MUTATION (LBM) - RUST-SPECIFIC
// ============================================================

impl TreeSitterMutationOperator for RustBorrowMutation {
    fn name(&self) -> &str {
        "RustBorrow"
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        // Detection-only: Identify borrow/reference operations
        match node.kind() {
            "reference_expression" => true,
            "parameter" => {
                // Check if parameter type contains & or &mut
                let text = &source[node.byte_range()];
                let text_str = std::str::from_utf8(text).unwrap_or("");
                text_str.contains("&mut") || text_str.contains('&')
            }
            "unary_expression" => {
                // Check for dereference operator
                if let Some(operator_node) = node.child_by_field_name("operator") {
                    let op = &source[operator_node.byte_range()];
                    op == b"*"
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn mutate(&self, _node: &Node, _source: &[u8]) -> Vec<MutatedSource> {
        // Detection-only: Borrow mutations would violate Rust's borrow checker
        // Changing & to &mut or vice versa would likely cause compilation errors
        // Return empty mutations for now
        Vec::new()
    }
}
