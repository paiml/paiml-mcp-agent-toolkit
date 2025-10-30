// Variable Inspector using AST
// Sprint 71 - TRACE-003: Variable Inspection with AST
//
// AST-based variable inspection for debugging support

use super::types::Variable;
use std::path::Path;
use tree_sitter::{Parser, Node, Tree};

/// Variable Inspector for extracting variables from source code
#[derive(Debug)]
pub struct VariableInspector {
    // Parser state (could be cached per language)
}

impl VariableInspector {
    /// Create a new variable inspector
    pub fn new() -> Self {
        Self {}
    }

    /// Inspect variables in Rust source at the given line
    pub fn inspect_rust(&self, source: &str, line: usize) -> Result<Vec<Variable>, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| format!("Failed to set Rust language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse Rust source".to_string())?;

        self.extract_variables_rust(&tree, source, line)
    }

    /// Inspect variables in TypeScript source at the given line
    pub fn inspect_typescript(&self, source: &str, line: usize) -> Result<Vec<Variable>, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .map_err(|e| format!("Failed to set TypeScript language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse TypeScript source".to_string())?;

        self.extract_variables_typescript(&tree, source, line)
    }

    /// Inspect variables in Python source at the given line
    pub fn inspect_python(&self, source: &str, line: usize) -> Result<Vec<Variable>, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| format!("Failed to set Python language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse Python source".to_string())?;

        self.extract_variables_python(&tree, source, line)
    }

    /// Inspect variables from a file
    pub fn inspect_file(&self, path: &Path, line: usize) -> Result<Vec<Variable>, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Detect language from file extension
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| "No file extension".to_string())?;

        match ext {
            "rs" => self.inspect_rust(&source, line),
            "ts" | "tsx" => self.inspect_typescript(&source, line),
            "js" | "jsx" => self.inspect_typescript(&source, line), // Use TS parser for JS
            "py" => self.inspect_python(&source, line),
            _ => Err(format!("Unsupported file extension: {}", ext)),
        }
    }

    /// Extract variables from Rust AST
    fn extract_variables_rust(&self, tree: &Tree, source: &str, target_line: usize) -> Result<Vec<Variable>, String> {
        let root_node = tree.root_node();
        let bytes = source.as_bytes();

        // Convert line number to byte position (lines are 0-indexed internally)
        let target_line_idx = if target_line > 0 { target_line - 1 } else { 0 };

        // Find the scope containing the target line
        let scope_node = self.find_scope_at_line(root_node, target_line_idx);

        if scope_node.is_none() {
            return Ok(Vec::new());
        }

        let scope = scope_node.unwrap();
        let mut variables = Vec::new();

        // Extract let bindings
        self.extract_rust_let_bindings(scope, bytes, &mut variables, target_line_idx);

        // Extract function parameters if we're in a function
        if let Some(func_node) = self.find_parent_function(scope) {
            self.extract_rust_function_params(func_node, bytes, &mut variables);
        }

        Ok(variables)
    }

    /// Extract variables from TypeScript AST
    fn extract_variables_typescript(&self, tree: &Tree, source: &str, target_line: usize) -> Result<Vec<Variable>, String> {
        let root_node = tree.root_node();
        let bytes = source.as_bytes();
        let target_line_idx = if target_line > 0 { target_line - 1 } else { 0 };

        let scope_node = self.find_scope_at_line(root_node, target_line_idx);
        if scope_node.is_none() {
            return Ok(Vec::new());
        }

        let scope = scope_node.unwrap();
        let mut variables = Vec::new();

        // Extract variable declarations (const, let, var)
        self.extract_ts_variable_declarations(scope, bytes, &mut variables, target_line_idx);

        // Extract function parameters
        if let Some(func_node) = self.find_parent_function(scope) {
            self.extract_ts_function_params(func_node, bytes, &mut variables);
        }

        Ok(variables)
    }

    /// Extract variables from Python AST
    fn extract_variables_python(&self, tree: &Tree, source: &str, target_line: usize) -> Result<Vec<Variable>, String> {
        let root_node = tree.root_node();
        let bytes = source.as_bytes();
        let target_line_idx = if target_line > 0 { target_line - 1 } else { 0 };

        let scope_node = self.find_scope_at_line(root_node, target_line_idx);
        if scope_node.is_none() {
            return Ok(Vec::new());
        }

        let scope = scope_node.unwrap();
        let mut variables = Vec::new();

        // Extract assignments
        self.extract_python_assignments(scope, bytes, &mut variables, target_line_idx);

        // Extract function parameters
        if let Some(func_node) = self.find_parent_function(scope) {
            self.extract_python_function_params(func_node, bytes, &mut variables);
        }

        Ok(variables)
    }

    /// Find the innermost scope containing the target line
    fn find_scope_at_line<'a>(&self, node: Node<'a>, target_line: usize) -> Option<Node<'a>> {
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
    fn find_parent_function<'a>(&self, mut node: Node<'a>) -> Option<Node<'a>> {
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

            match node.parent() {
                Some(parent) => node = parent,
                None => return None,
            }
        }
    }

    /// Extract Rust let bindings
    fn extract_rust_let_bindings(&self, node: Node, bytes: &[u8], variables: &mut Vec<Variable>, target_line: usize) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            // Only include bindings before or at target line
            if child.start_position().row > target_line {
                continue;
            }

            if child.kind() == "let_declaration" {
                if let Some(pattern_node) = child.child_by_field_name("pattern") {
                    let var_name = pattern_node.utf8_text(bytes).unwrap_or("").to_string();

                    // Infer type
                    let type_info = if let Some(type_node) = child.child_by_field_name("type") {
                        type_node.utf8_text(bytes).unwrap_or("unknown").to_string()
                    } else if let Some(value_node) = child.child_by_field_name("value") {
                        self.infer_rust_type(value_node, bytes)
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
    fn extract_rust_function_params(&self, func_node: Node, bytes: &[u8], variables: &mut Vec<Variable>) {
        if let Some(params_node) = func_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for param in params_node.children(&mut cursor) {
                if param.kind() == "parameter" {
                    if let Some(pattern) = param.child_by_field_name("pattern") {
                        let name = pattern.utf8_text(bytes).unwrap_or("").to_string();
                        let type_info = if let Some(type_node) = param.child_by_field_name("type") {
                            type_node.utf8_text(bytes).unwrap_or("unknown").to_string()
                        } else {
                            "unknown".to_string()
                        };

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
    fn infer_rust_type(&self, node: Node, _bytes: &[u8]) -> String {
        match node.kind() {
            "integer_literal" => "i32".to_string(),
            "float_literal" => "f64".to_string(),
            "string_literal" => "&str".to_string(),
            "boolean_literal" | "true" | "false" => "bool".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Extract TypeScript variable declarations
    fn extract_ts_variable_declarations(&self, node: Node, bytes: &[u8], variables: &mut Vec<Variable>, target_line: usize) {
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
                        if let Some(name_node) = declarator.child_by_field_name("name") {
                            let name = name_node.utf8_text(bytes).unwrap_or("").to_string();
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

            self.extract_ts_variable_declarations(child, bytes, variables, target_line);
        }
    }

    /// Extract TypeScript function parameters
    fn extract_ts_function_params(&self, func_node: Node, bytes: &[u8], variables: &mut Vec<Variable>) {
        if let Some(params_node) = func_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for param in params_node.children(&mut cursor) {
                if param.kind() == "required_parameter" || param.kind() == "identifier" {
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

    /// Extract Python assignments
    fn extract_python_assignments(&self, node: Node, bytes: &[u8], variables: &mut Vec<Variable>, target_line: usize) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.start_position().row > target_line {
                continue;
            }

            if child.kind() == "assignment" {
                if let Some(left_node) = child.child_by_field_name("left") {
                    let name = left_node.utf8_text(bytes).unwrap_or("").to_string();
                    variables.push(Variable {
                        name,
                        value: String::new(),
                        type_info: "Any".to_string(),
                        variables_reference: None,
                    });
                }
            }

            self.extract_python_assignments(child, bytes, variables, target_line);
        }
    }

    /// Extract Python function parameters
    fn extract_python_function_params(&self, func_node: Node, bytes: &[u8], variables: &mut Vec<Variable>) {
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

impl Default for VariableInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_creation() {
        let inspector = VariableInspector::new();
        assert!(true); // Just test creation
    }

    #[test]
    fn test_find_scope_basic() {
        let inspector = VariableInspector::new();
        let source = "fn main() { let x = 1; }";

        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(source, None).unwrap();

        let scope = inspector.find_scope_at_line(tree.root_node(), 0);
        assert!(scope.is_some());
    }
}
