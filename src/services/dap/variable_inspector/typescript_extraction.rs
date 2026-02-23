#![cfg_attr(coverage_nightly, coverage(off))]
//! TypeScript-specific variable extraction from AST nodes

use super::{Variable, VariableInspector};
use tree_sitter::{Node, Tree};

impl VariableInspector {
    /// Extract variables from TypeScript AST
    pub(super) fn extract_variables_typescript(
        &self,
        tree: &Tree,
        source: &str,
        target_line: usize,
    ) -> Result<Vec<Variable>, String> {
        let root_node = tree.root_node();
        let bytes = source.as_bytes();
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

        let scope_node = self.find_scope_at_line(root_node, target_line_idx);
        if scope_node.is_none() {
            return Ok(Vec::new());
        }

        let scope = scope_node.expect("internal error");
        let mut variables = Vec::new();

        // Find the parent function to get the scope we should search from
        let search_scope = if let Some(func_node) = self.find_parent_function(scope) {
            // Extract function parameters
            self.extract_ts_function_params(func_node, bytes, &mut variables);
            func_node
        } else {
            scope
        };

        // Extract variable declarations (const, let, var) from the function or scope
        self.extract_ts_variable_declarations(search_scope, bytes, &mut variables, target_line_idx);

        // Deduplicate variables by name (keep last occurrence for shadowing)
        Ok(self.deduplicate_variables(variables))
    }

    /// Extract TypeScript variable declarations
    pub(super) fn extract_ts_variable_declarations(
        &self,
        node: Node,
        bytes: &[u8],
        variables: &mut Vec<Variable>,
        target_line: usize,
    ) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.start_position().row > target_line {
                continue;
            }

            if child.kind() == "lexical_declaration" || child.kind() == "variable_declaration" {
                // Extract variable declarator - create new cursor for inner iteration
                let mut child_cursor = child.walk();
                for declarator in child.children(&mut child_cursor) {
                    if declarator.kind() == "variable_declarator" {
                        // Find the first identifier child
                        let mut decl_cursor = declarator.walk();
                        for decl_child in declarator.children(&mut decl_cursor) {
                            if decl_child.kind() == "identifier" {
                                let name = decl_child.utf8_text(bytes).unwrap_or("").to_string();
                                if !name.is_empty() {
                                    variables.push(Variable {
                                        name,
                                        value: String::new(),
                                        type_info: "any".to_string(),
                                        variables_reference: None,
                                    });
                                }
                                break; // Only get the first identifier
                            }
                        }
                    }
                }
            }

            self.extract_ts_variable_declarations(child, bytes, variables, target_line);
        }
    }

    /// Extract TypeScript function parameters
    pub(super) fn extract_ts_function_params(
        &self,
        func_node: Node,
        bytes: &[u8],
        variables: &mut Vec<Variable>,
    ) {
        // Try field-based access first (regular functions)
        let params_node_opt = if let Some(params) = func_node.child_by_field_name("parameters") {
            Some(params)
        } else {
            // For arrow functions, look for formal_parameters child manually
            let mut found = None;
            let mut params_cursor = func_node.walk();
            for child in func_node.children(&mut params_cursor) {
                if child.kind() == "formal_parameters" {
                    found = Some(child);
                    break;
                }
            }
            found
        };

        if let Some(params_node) = params_node_opt {
            let mut cursor = params_node.walk();
            for param in params_node.children(&mut cursor) {
                if param.kind() == "required_parameter" {
                    // Extract the identifier child from required_parameter
                    let mut param_cursor = param.walk();
                    for param_child in param.children(&mut param_cursor) {
                        if param_child.kind() == "identifier" {
                            let name = param_child.utf8_text(bytes).unwrap_or("").to_string();
                            if !name.is_empty() {
                                variables.push(Variable {
                                    name,
                                    value: String::new(),
                                    type_info: "any".to_string(),
                                    variables_reference: None,
                                });
                            }
                            break;
                        }
                    }
                } else if param.kind() == "identifier" {
                    // Direct identifier (uncommon but possible)
                    let name = param.utf8_text(bytes).unwrap_or("").to_string();
                    if !name.is_empty() && name != "(" && name != ")" && name != "," {
                        variables.push(Variable {
                            name,
                            value: String::new(),
                            type_info: "any".to_string(),
                            variables_reference: None,
                        });
                    }
                }
            }
        }
    }
}
