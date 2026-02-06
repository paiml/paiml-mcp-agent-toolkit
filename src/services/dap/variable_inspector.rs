#![cfg_attr(coverage_nightly, coverage(off))]
// Variable Inspector using AST
// Sprint 71 - TRACE-003: Variable Inspection with AST
//
// AST-based variable inspection for debugging support

use super::types::Variable;
use std::path::Path;
use tree_sitter::{Node, Parser, Tree};

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
    #[cfg(feature = "python-ast")]
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

    #[cfg(not(feature = "python-ast"))]
    pub fn inspect_python(&self, _source: &str, _line: usize) -> Result<Vec<Variable>, String> {
        Err("python-ast feature is disabled".to_string())
    }

    /// Inspect variables from a file
    pub fn inspect_file(&self, path: &Path, line: usize) -> Result<Vec<Variable>, String> {
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

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
    fn extract_variables_rust(
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

    /// Extract variables from TypeScript AST
    fn extract_variables_typescript(
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

    /// Extract variables from Python AST
    #[allow(dead_code)]
    fn extract_variables_python(
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

            match node.parent() {
                Some(parent) => node = parent,
                None => return None,
            }
        }
    }

    /// Extract Rust let bindings
    fn extract_rust_let_bindings(
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
    fn extract_rust_function_params(
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
    fn extract_ts_variable_declarations(
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
    fn extract_ts_function_params(
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

    /// Extract Python assignments
    #[allow(dead_code)]
    fn extract_python_assignments(
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
    #[allow(dead_code)]
    fn extract_python_function_params(
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

    /// Deduplicate variables by name, keeping only the last occurrence (for shadowing)
    fn deduplicate_variables(&self, variables: Vec<Variable>) -> Vec<Variable> {
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

impl Default for VariableInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_creation() {
        let _inspector = VariableInspector::new();
        assert!(true); // Just test creation
    }

    #[test]
    fn test_find_scope_basic() {
        let inspector = VariableInspector::new();
        let source = "fn main() { let x = 1; }";

        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("internal error");
        let tree = parser.parse(source, None).expect("internal error");

        let scope = inspector.find_scope_at_line(tree.root_node(), 0);
        assert!(scope.is_some());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ========================================================================
    // VariableInspector Creation Tests
    // ========================================================================

    #[test]
    fn test_variable_inspector_new() {
        let inspector = VariableInspector::new();
        // Inspector is created successfully (empty struct)
        let _ = format!("{:?}", inspector);
    }

    #[test]
    fn test_variable_inspector_default() {
        let inspector = VariableInspector::default();
        let _ = format!("{:?}", inspector);
    }

    #[test]
    fn test_variable_inspector_debug() {
        let inspector = VariableInspector::new();
        let debug_str = format!("{:?}", inspector);
        assert!(debug_str.contains("VariableInspector"));
    }

    // ========================================================================
    // Rust Inspection Tests
    // ========================================================================

    #[test]
    fn test_inspect_rust_simple_let_binding() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let x = 42;
    println!("{}", x);
}
"#;
        let result = inspector.inspect_rust(source, 4);
        assert!(result.is_ok());
        let vars = result.unwrap();
        assert!(vars.iter().any(|v| v.name == "x"));
    }

    #[test]
    fn test_inspect_rust_multiple_let_bindings() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = 3;
    println!("{} {} {}", a, b, c);
}
"#;
        let result = inspector.inspect_rust(source, 6);
        assert!(result.is_ok());
        let vars = result.unwrap();
        // Should find a, b, c
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
        assert!(names.contains(&"c"));
    }

    #[test]
    fn test_inspect_rust_function_parameters() {
        let inspector = VariableInspector::new();
        let source = r#"
fn add(x: i32, y: i32) -> i32 {
    x + y
}
"#;
        let result = inspector.inspect_rust(source, 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"x"));
        assert!(names.contains(&"y"));
    }

    #[test]
    fn test_inspect_rust_type_inference_integer() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let x = 42;
}
"#;
        let result = inspector.inspect_rust(source, 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let x_var = vars.iter().find(|v| v.name == "x");
        assert!(x_var.is_some());
        assert_eq!(x_var.unwrap().type_info, "i32");
    }

    #[test]
    fn test_inspect_rust_type_inference_float() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let x = 3.14;
}
"#;
        let result = inspector.inspect_rust(source, 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let x_var = vars.iter().find(|v| v.name == "x");
        assert!(x_var.is_some());
        assert_eq!(x_var.unwrap().type_info, "f64");
    }

    #[test]
    fn test_inspect_rust_type_inference_string() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let x = "hello";
}
"#;
        let result = inspector.inspect_rust(source, 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let x_var = vars.iter().find(|v| v.name == "x");
        assert!(x_var.is_some());
        assert_eq!(x_var.unwrap().type_info, "&str");
    }

    #[test]
    fn test_inspect_rust_type_inference_bool_true() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let x = true;
}
"#;
        let result = inspector.inspect_rust(source, 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let x_var = vars.iter().find(|v| v.name == "x");
        assert!(x_var.is_some());
        assert_eq!(x_var.unwrap().type_info, "bool");
    }

    #[test]
    fn test_inspect_rust_type_inference_bool_false() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let x = false;
}
"#;
        let result = inspector.inspect_rust(source, 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let x_var = vars.iter().find(|v| v.name == "x");
        assert!(x_var.is_some());
        assert_eq!(x_var.unwrap().type_info, "bool");
    }

    #[test]
    fn test_inspect_rust_line_out_of_bounds() {
        let inspector = VariableInspector::new();
        let source = "fn main() {}";

        let result = inspector.inspect_rust(source, 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_inspect_rust_line_zero() {
        let inspector = VariableInspector::new();
        let source = "fn main() {}";

        let result = inspector.inspect_rust(source, 0);
        // Line 0 should be treated as line 1
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_rust_empty_function() {
        let inspector = VariableInspector::new();
        let source = "fn main() {}";

        let result = inspector.inspect_rust(source, 1);
        assert!(result.is_ok());
        // Empty function has no variables
    }

    #[test]
    fn test_inspect_rust_shadowed_variables() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let x = 1;
    let x = 2;
    let x = 3;
    println!("{}", x);
}
"#;
        let result = inspector.inspect_rust(source, 6);
        assert!(result.is_ok());
        let vars = result.unwrap();
        // Should deduplicate, keeping only one x
        let x_count = vars.iter().filter(|v| v.name == "x").count();
        assert_eq!(x_count, 1);
    }

    // ========================================================================
    // TypeScript Inspection Tests
    // ========================================================================

    #[test]
    fn test_inspect_typescript_const_declaration() {
        let inspector = VariableInspector::new();
        let source = r#"
function main() {
    const x = 42;
    console.log(x);
}
"#;
        let result = inspector.inspect_typescript(source, 4);
        assert!(result.is_ok());
        let vars = result.unwrap();
        assert!(vars.iter().any(|v| v.name == "x"));
    }

    #[test]
    fn test_inspect_typescript_let_declaration() {
        let inspector = VariableInspector::new();
        let source = r#"
function main() {
    let x = 42;
    console.log(x);
}
"#;
        let result = inspector.inspect_typescript(source, 4);
        assert!(result.is_ok());
        let vars = result.unwrap();
        assert!(vars.iter().any(|v| v.name == "x"));
    }

    #[test]
    fn test_inspect_typescript_multiple_declarations() {
        let inspector = VariableInspector::new();
        let source = r#"
function main() {
    const a = 1;
    let b = 2;
    var c = 3;
    console.log(a, b, c);
}
"#;
        let result = inspector.inspect_typescript(source, 6);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
        assert!(names.contains(&"c"));
    }

    #[test]
    fn test_inspect_typescript_function_parameters() {
        let inspector = VariableInspector::new();
        let source = r#"
function add(x: number, y: number): number {
    return x + y;
}
"#;
        let result = inspector.inspect_typescript(source, 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"x"));
        assert!(names.contains(&"y"));
    }

    #[test]
    fn test_inspect_typescript_arrow_function() {
        let inspector = VariableInspector::new();
        let source = r#"
const add = (x: number, y: number): number => {
    return x + y;
};
"#;
        let result = inspector.inspect_typescript(source, 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"x") || names.contains(&"y") || names.contains(&"add"));
    }

    #[test]
    fn test_inspect_typescript_line_out_of_bounds() {
        let inspector = VariableInspector::new();
        let source = "function main() {}";

        let result = inspector.inspect_typescript(source, 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_inspect_typescript_empty_function() {
        let inspector = VariableInspector::new();
        let source = "function main() {}";

        let result = inspector.inspect_typescript(source, 1);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Python Inspection Tests
    // ========================================================================

    #[test]
    fn test_inspect_python_disabled_feature() {
        let inspector = VariableInspector::new();
        let source = "x = 42";

        let result = inspector.inspect_python(source, 1);
        // Without python-ast feature, should return error
        #[cfg(not(feature = "python-ast"))]
        assert!(result.is_err());
        #[cfg(feature = "python-ast")]
        assert!(result.is_ok());
    }

    // ========================================================================
    // File Inspection Tests
    // ========================================================================

    #[test]
    fn test_inspect_file_rust() {
        let inspector = VariableInspector::new();

        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            temp_file,
            r#"
fn main() {{
    let x = 42;
}}
"#
        )
        .unwrap();

        let result = inspector.inspect_file(temp_file.path(), 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        assert!(vars.iter().any(|v| v.name == "x"));
    }

    #[test]
    fn test_inspect_file_typescript() {
        let inspector = VariableInspector::new();

        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        writeln!(
            temp_file,
            r#"
function main() {{
    const x = 42;
}}
"#
        )
        .unwrap();

        let result = inspector.inspect_file(temp_file.path(), 3);
        assert!(result.is_ok());
        let vars = result.unwrap();
        assert!(vars.iter().any(|v| v.name == "x"));
    }

    #[test]
    fn test_inspect_file_javascript() {
        let inspector = VariableInspector::new();

        let mut temp_file = NamedTempFile::with_suffix(".js").unwrap();
        writeln!(
            temp_file,
            r#"
function main() {{
    const x = 42;
}}
"#
        )
        .unwrap();

        let result = inspector.inspect_file(temp_file.path(), 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_file_tsx() {
        let inspector = VariableInspector::new();

        let mut temp_file = NamedTempFile::with_suffix(".tsx").unwrap();
        writeln!(
            temp_file,
            r#"
function Component() {{
    const x = 42;
    return <div>{{x}}</div>;
}}
"#
        )
        .unwrap();

        let result = inspector.inspect_file(temp_file.path(), 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_file_jsx() {
        let inspector = VariableInspector::new();

        let mut temp_file = NamedTempFile::with_suffix(".jsx").unwrap();
        writeln!(
            temp_file,
            r#"
function Component() {{
    const x = 42;
    return <div>{{x}}</div>;
}}
"#
        )
        .unwrap();

        let result = inspector.inspect_file(temp_file.path(), 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_file_python() {
        let inspector = VariableInspector::new();

        let mut temp_file = NamedTempFile::with_suffix(".py").unwrap();
        writeln!(temp_file, "x = 42").unwrap();

        let result = inspector.inspect_file(temp_file.path(), 1);
        // Result depends on python-ast feature
        #[cfg(not(feature = "python-ast"))]
        assert!(result.is_err());
        #[cfg(feature = "python-ast")]
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_file_unsupported_extension() {
        let inspector = VariableInspector::new();

        let mut temp_file = NamedTempFile::with_suffix(".xyz").unwrap();
        writeln!(temp_file, "some content").unwrap();

        let result = inspector.inspect_file(temp_file.path(), 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported file extension"));
    }

    #[test]
    fn test_inspect_file_no_extension() {
        let inspector = VariableInspector::new();

        let temp_file = NamedTempFile::new().unwrap();

        let result = inspector.inspect_file(temp_file.path(), 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No file extension"));
    }

    #[test]
    fn test_inspect_file_nonexistent() {
        let inspector = VariableInspector::new();

        let result = inspector.inspect_file(Path::new("/nonexistent/file.rs"), 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }

    // ========================================================================
    // Scope Finding Tests
    // ========================================================================

    #[test]
    fn test_find_scope_nested_blocks() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    {
        {
            let x = 1;
        }
    }
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();

        let scope = inspector.find_scope_at_line(tree.root_node(), 4);
        assert!(scope.is_some());
    }

    #[test]
    fn test_find_scope_target_line_outside_node() {
        let inspector = VariableInspector::new();
        let source = "fn main() {}";

        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();

        // Target line way outside the file
        let scope = inspector.find_scope_at_line(tree.root_node(), 100);
        assert!(scope.is_none());
    }

    // ========================================================================
    // Parent Function Finding Tests
    // ========================================================================

    #[test]
    fn test_find_parent_function_rust() {
        let inspector = VariableInspector::new();
        let source = r#"
fn outer() {
    fn inner() {
        let x = 1;
    }
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();

        // Find the innermost scope at line 4 (let x = 1)
        let scope = inspector.find_scope_at_line(tree.root_node(), 3);
        assert!(scope.is_some());

        // Find parent function
        let parent_func = inspector.find_parent_function(scope.unwrap());
        assert!(parent_func.is_some());
        assert_eq!(parent_func.unwrap().kind(), "function_item");
    }

    #[test]
    fn test_find_parent_function_none() {
        let inspector = VariableInspector::new();
        let source = "const X: i32 = 1;"; // No function

        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();

        let scope = inspector.find_scope_at_line(tree.root_node(), 0);
        assert!(scope.is_some());

        let parent_func = inspector.find_parent_function(scope.unwrap());
        assert!(parent_func.is_none());
    }

    // ========================================================================
    // Variable Deduplication Tests
    // ========================================================================

    #[test]
    fn test_deduplicate_variables_empty() {
        let inspector = VariableInspector::new();
        let vars: Vec<Variable> = vec![];

        let deduped = inspector.deduplicate_variables(vars);
        assert!(deduped.is_empty());
    }

    #[test]
    fn test_deduplicate_variables_no_duplicates() {
        let inspector = VariableInspector::new();
        let vars = vec![
            Variable {
                name: "a".to_string(),
                value: "1".to_string(),
                type_info: "i32".to_string(),
                variables_reference: None,
            },
            Variable {
                name: "b".to_string(),
                value: "2".to_string(),
                type_info: "i32".to_string(),
                variables_reference: None,
            },
        ];

        let deduped = inspector.deduplicate_variables(vars);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_deduplicate_variables_with_duplicates() {
        let inspector = VariableInspector::new();
        let vars = vec![
            Variable {
                name: "x".to_string(),
                value: "1".to_string(),
                type_info: "i32".to_string(),
                variables_reference: None,
            },
            Variable {
                name: "x".to_string(),
                value: "2".to_string(),
                type_info: "i32".to_string(),
                variables_reference: None,
            },
            Variable {
                name: "x".to_string(),
                value: "3".to_string(),
                type_info: "i32".to_string(),
                variables_reference: None,
            },
        ];

        let deduped = inspector.deduplicate_variables(vars);
        // Should keep only one x (the last one due to HashMap behavior)
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].name, "x");
    }

    // ========================================================================
    // Type Inference Tests
    // ========================================================================

    #[test]
    fn test_infer_rust_type_integer_literal() {
        let inspector = VariableInspector::new();
        let source = "let x = 42;";

        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();

        // Find the integer_literal node
        let root = tree.root_node();
        let mut cursor = root.walk();
        let mut found_type = None;

        for child in root.children(&mut cursor) {
            if child.kind() == "let_declaration" {
                let mut decl_cursor = child.walk();
                for decl_child in child.children(&mut decl_cursor) {
                    if decl_child.kind() == "integer_literal" {
                        found_type = Some(inspector.infer_rust_type(decl_child, source.as_bytes()));
                    }
                }
            }
        }

        assert_eq!(found_type, Some("i32".to_string()));
    }

    #[test]
    fn test_infer_rust_type_unknown() {
        let inspector = VariableInspector::new();
        let source = "let x = some_function();";

        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let _tree = parser.parse(source, None).unwrap();

        // The function call node type would be unknown
        let result = inspector.inspect_rust(source, 1);
        assert!(result.is_ok());
        let vars = result.unwrap();
        if let Some(x_var) = vars.iter().find(|v| v.name == "x") {
            // Type is unknown for function calls
            assert_eq!(x_var.type_info, "unknown");
        }
    }

    // ========================================================================
    // Edge Cases and Error Handling
    // ========================================================================

    #[test]
    fn test_empty_source_rust() {
        let inspector = VariableInspector::new();
        let result = inspector.inspect_rust("", 1);
        // Empty source might still parse, but line 1 is out of bounds
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    #[test]
    fn test_empty_source_typescript() {
        let inspector = VariableInspector::new();
        let result = inspector.inspect_typescript("", 1);
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    #[test]
    fn test_whitespace_only_source() {
        let inspector = VariableInspector::new();
        let result = inspector.inspect_rust("   \n\n  \t  ", 1);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_comment_only_source() {
        let inspector = VariableInspector::new();
        let source = "// This is a comment\n// Another comment";
        let result = inspector.inspect_rust(source, 1);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_multiline_function() {
        let inspector = VariableInspector::new();
        let source = r#"
fn complex_function(
    param1: i32,
    param2: String,
    param3: bool,
) -> Result<(), Error> {
    let local1 = param1 + 1;
    let local2 = param2.clone();
    let local3 = !param3;

    Ok(())
}
"#;
        let result = inspector.inspect_rust(source, 10);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        // Should find parameters and locals
        assert!(names.contains(&"param1") || names.contains(&"local1"));
    }

    #[test]
    fn test_nested_scope_variables() {
        let inspector = VariableInspector::new();
        let source = r#"
fn main() {
    let outer = 1;
    {
        let inner = 2;
        println!("{} {}", outer, inner);
    }
}
"#;
        let result = inspector.inspect_rust(source, 6);
        assert!(result.is_ok());
        let vars = result.unwrap();
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"inner") || names.contains(&"outer"));
    }
}
