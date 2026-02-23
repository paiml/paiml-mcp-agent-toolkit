#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for VariableInspector

use super::*;
use tree_sitter::Parser;

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
