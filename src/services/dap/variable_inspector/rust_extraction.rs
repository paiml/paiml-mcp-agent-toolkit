#![cfg_attr(coverage_nightly, coverage(off))]
//! Rust-specific variable extraction from AST nodes

use super::{Variable, VariableInspector};
use tree_sitter::{Node, Tree};

impl VariableInspector {
    /// Extract variables from Rust AST
    pub(super) fn extract_variables_rust(
        &self,
        tree: &Tree,
        source: &str,
        target_line: usize,
    ) -> Result<Vec<Variable>, String> {
        let root_node = tree.root_node();
        let bytes = source.as_bytes();

        // Convert line number to byte position (lines are 0-indexed internally)
        let target_line_idx = if target_line > 0 { target_line - 1 } else { 0 };

        // Validate line number is within bounds
        let max_line = root_node.end_position().row;
        if target_line_idx > max_line {
            return Err(format!(
                "Line {} is out of bounds (file has {} lines)",
                target_line,
                max_line + 1
            ));
        }

        // Find the scope containing the target line
        let scope_node = self.find_scope_at_line(root_node, target_line_idx);

        if scope_node.is_none() {
            return Ok(Vec::new());
        }

        let scope = scope_node.expect("internal error");
        let mut variables = Vec::new();

        // Find the parent function to get the scope we should search from
        let search_scope = if let Some(func_node) = self.find_parent_function(scope) {
            // Extract function parameters
            self.extract_rust_function_params(func_node, bytes, &mut variables);
            func_node
        } else {
            scope
        };

        // Extract let bindings from the function or scope
        self.extract_rust_let_bindings(search_scope, bytes, &mut variables, target_line_idx);

        // Deduplicate variables by name (keep last occurrence for shadowing)
        Ok(self.deduplicate_variables(variables))
    }

    /// Extract Rust let bindings
    pub(super) fn extract_rust_let_bindings(
        &self,
        node: Node,
        bytes: &[u8],
        variables: &mut Vec<Variable>,
        target_line: usize,
    ) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            // Only include bindings before or at target line
            if child.start_position().row > target_line {
                continue;
            }

            if child.kind() == "let_declaration" {
                // Find the identifier node (variable name)
                let mut decl_cursor = child.walk();
                let mut var_name = String::new();
                let mut value_node = None;

                for decl_child in child.children(&mut decl_cursor) {
                    match decl_child.kind() {
                        "identifier" if var_name.is_empty() => {
                            var_name = decl_child.utf8_text(bytes).unwrap_or("").to_string();
                        }
                        "integer_literal" | "float_literal" | "string_literal"
                        | "boolean_literal" | "true" | "false" => {
                            value_node = Some(decl_child);
                        }
                        _ => {}
                    }
                }

                if !var_name.is_empty() {
                    let type_info = if let Some(val_node) = value_node {
                        self.infer_rust_type(val_node, bytes)
                    } else {
                        "unknown".to_string()
                    };

                    variables.push(Variable {
                        name: var_name,
                        value: String::new(),
                        type_info,
                        variables_reference: None,
                    });
                }
            }

            // Recurse into child scopes
            self.extract_rust_let_bindings(child, bytes, variables, target_line);
        }
    }

    /// Extract Rust function parameters
    pub(super) fn extract_rust_function_params(
        &self,
        func_node: Node,
        bytes: &[u8],
        variables: &mut Vec<Variable>,
    ) {
        if let Some(params_node) = func_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for param in params_node.children(&mut cursor) {
                if param.kind() == "parameter" {
                    // Extract identifier and type from parameter children
                    let mut param_cursor = param.walk();
                    let mut name = String::new();
                    let mut type_info = String::from("unknown");

                    for param_child in param.children(&mut param_cursor) {
                        match param_child.kind() {
                            "identifier" if name.is_empty() => {
                                name = param_child.utf8_text(bytes).unwrap_or("").to_string();
                            }
                            "primitive_type" | "type_identifier" => {
                                type_info = param_child
                                    .utf8_text(bytes)
                                    .unwrap_or("unknown")
                                    .to_string();
                            }
                            _ => {}
                        }
                    }

                    if !name.is_empty() {
                        variables.push(Variable {
                            name,
                            value: String::new(),
                            type_info,
                            variables_reference: None,
                        });
                    }
                }
            }
        }
    }

    /// Infer Rust type from literal
    pub(super) fn infer_rust_type(&self, node: Node, _bytes: &[u8]) -> String {
        match node.kind() {
            "integer_literal" => "i32".to_string(),
            "float_literal" => "f64".to_string(),
            "string_literal" => "&str".to_string(),
            "boolean_literal" | "true" | "false" => "bool".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
