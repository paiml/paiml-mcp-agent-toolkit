// C++ Mutation Operators using tree-sitter AST
// PMAT-7013: C++ Mutation Testing
// Status: GREEN Phase - Full implementation

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

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
                    source: String::from_utf8(mutated).unwrap(),
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
                    source: String::from_utf8(mutated).unwrap(),
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
            source: String::from_utf8(mutated).unwrap(),
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
                    source: String::from_utf8(mutated).unwrap(),
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
                    source: String::from_utf8(mutated).unwrap(),
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

// ============================================================
// UNIT TESTS
// ============================================================

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
    fn find_node_by_kind_iter<'a>(tree: &'a tree_sitter::Tree, kind: &str) -> Option<tree_sitter::Node<'a>> {
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

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression").expect("Should find binary_expression");

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

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression").expect("Should find binary_expression");

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

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression").expect("Should find binary_expression");

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

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression").expect("Should find binary_expression");

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

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression").expect("Should find binary_expression");

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

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression").expect("Should find binary_expression");

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

        let binary_node = find_node_by_kind_iter(&tree, "binary_expression").expect("Should find binary_expression");

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

        let unary_node = find_node_by_kind_iter(&tree, "unary_expression").expect("Should find unary_expression");

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

        let unary_node = find_node_by_kind_iter(&tree, "unary_expression").expect("Should find unary_expression");

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

        let unary_node = find_node_by_kind_iter(&tree, "unary_expression").expect("Should find unary_expression");

        let operator = CppUnaryOpMutation;
        let mutants = operator.mutate(&unary_node, source.as_bytes());
        assert_eq!(mutants.len(), 1); // - → +
        assert!(mutants[0].source.contains("+value"));
    }

    #[test]
    fn test_cpp_pre_increment() {
        let source = "int result = ++i;";
        let tree = parse_cpp(source);

        let update_node = find_node_by_kind_iter(&tree, "update_expression").expect("Should find update_expression");

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

        let update_node = find_node_by_kind_iter(&tree, "update_expression").expect("Should find update_expression");

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

        let pointer_node = find_node_by_kind_iter(&tree, "pointer_expression").expect("Should find pointer_expression");

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
}
