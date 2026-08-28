#![cfg_attr(coverage_nightly, coverage(off))]
//! Scope finding, parent function resolution, and variable deduplication

use super::{Variable, VariableInspector};
use tree_sitter::Node;

impl VariableInspector {
    /// Find the innermost scope containing the target line
    pub(super) fn find_scope_at_line<'a>(
        &self,
        node: Node<'a>,
        target_line: usize,
    ) -> Option<Node<'a>> {
        let node_start_line = node.start_position().row;
        let node_end_line = node.end_position().row;

        // Check if target line is within this node
        if target_line < node_start_line || target_line > node_end_line {
            return None;
        }

        // Try to find a more specific child scope
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(child_scope) = self.find_scope_at_line(child, target_line) {
                return Some(child_scope);
            }
        }

        // Return this node as the scope
        Some(node)
    }

    /// Find parent function node
    pub(super) fn find_parent_function<'a>(&self, mut node: Node<'a>) -> Option<Node<'a>> {
        loop {
            let kind = node.kind();
            if kind == "function_item"
                || kind == "function_declaration"
                || kind == "function_definition"
                || kind == "arrow_function"
                || kind == "method_definition"
            {
                return Some(node);
            }

            // For lexical_declaration, check if it contains an arrow_function
            if kind == "lexical_declaration" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "variable_declarator" {
                        let mut decl_cursor = child.walk();
                        for decl_child in child.children(&mut decl_cursor) {
                            if decl_child.kind() == "arrow_function" {
                                return Some(decl_child);
                            }
                        }
                    }
                }
            }

            {
                let parent = node.parent()?;
                node = parent
            }
        }
    }

    /// Deduplicate variables by name, keeping only the last occurrence (for shadowing)
    pub(super) fn deduplicate_variables(&self, variables: Vec<Variable>) -> Vec<Variable> {
        use std::collections::HashMap;

        // Use HashMap to track the last occurrence of each variable name
        let mut unique_vars: HashMap<String, Variable> = HashMap::new();

        for var in variables {
            unique_vars.insert(var.name.clone(), var);
        }

        // Convert back to Vec, preserving original order where possible
        unique_vars.into_values().collect()
    }
}
