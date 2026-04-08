#![cfg_attr(coverage_nightly, coverage(off))]
//! Python-specific variable extraction from AST nodes

use super::{Variable, VariableInspector};
use tree_sitter::{Node, Tree};

impl VariableInspector {
    /// Extract variables from Python AST

    pub(super) fn extract_variables_python(
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
            self.extract_python_function_params(func_node, bytes, &mut variables);
            func_node
        } else {
            scope
        };

        // Extract assignments from the function or scope
        self.extract_python_assignments(search_scope, bytes, &mut variables, target_line_idx);

        // Deduplicate variables by name (keep last occurrence for shadowing)
        Ok(self.deduplicate_variables(variables))
    }

    /// Extract Python assignments

    pub(super) fn extract_python_assignments(
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

            if child.kind() == "assignment" {
                // Find the first identifier child (the variable being assigned to)
                let mut assign_cursor = child.walk();
                for assign_child in child.children(&mut assign_cursor) {
                    if assign_child.kind() == "identifier" {
                        let name = assign_child.utf8_text(bytes).unwrap_or("").to_string();
                        if !name.is_empty() {
                            variables.push(Variable {
                                name,
                                value: String::new(),
                                type_info: "Any".to_string(),
                                variables_reference: None,
                            });
                        }
                        break; // Only get the first identifier (left side of assignment)
                    }
                }
            }

            self.extract_python_assignments(child, bytes, variables, target_line);
        }
    }

    /// Extract Python function parameters

    pub(super) fn extract_python_function_params(
        &self,
        func_node: Node,
        bytes: &[u8],
        variables: &mut Vec<Variable>,
    ) {
        if let Some(params_node) = func_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for param in params_node.children(&mut cursor) {
                if param.kind() == "identifier" {
                    let name = param.utf8_text(bytes).unwrap_or("").to_string();
                    if !name.is_empty() && name != "(" && name != ")" && name != "," {
                        variables.push(Variable {
                            name,
                            value: String::new(),
                            type_info: "Any".to_string(),
                            variables_reference: None,
                        });
                    }
                }
            }
        }
    }
}
