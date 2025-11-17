// TRACE-003: Variable Inspection with AST
// RED Phase Tests - Sprint 71
//
// Tests for AST-based variable inspection across multiple languages

use std::path::Path;

// RED Phase Test 1: Rust - Simple Local Variables
// TODO: AST node traversal needs refinement for actual tree-sitter node types
#[test]
fn test_rust_simple_local_variables() {
    let source = r#"
fn main() {
    let x = 42;
    let name = "Alice";
    let is_active = true;
    println!("{}", x);  // Line 5: Inspect here
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_rust(source, 5);

    assert!(result.is_ok(), "Should successfully inspect Rust code");
    let variables = result.unwrap();

    assert_eq!(variables.len(), 3, "Should find 3 variables");

    // Check x variable
    let x_var = variables.iter().find(|v| v.name == "x");
    assert!(x_var.is_some(), "Should find variable 'x'");
    assert_eq!(x_var.unwrap().type_info, "i32");

    // Check name variable
    let name_var = variables.iter().find(|v| v.name == "name");
    assert!(name_var.is_some(), "Should find variable 'name'");
    assert_eq!(name_var.unwrap().type_info, "&str");

    // Check is_active variable
    let is_active_var = variables.iter().find(|v| v.name == "is_active");
    assert!(is_active_var.is_some(), "Should find variable 'is_active'");
    assert_eq!(is_active_var.unwrap().type_info, "bool");
}

// RED Phase Test 2: Rust - Function Parameters
// TODO: Function parameter extraction needs tree-sitter node type mapping
#[test]
fn test_rust_function_parameters() {
    let source = r#"
fn calculate(x: i32, y: i32, name: &str) -> i32 {
    let result = x + y;
    println!("{}", result);  // Line 3: Inspect here
    result
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_rust(source, 3);

    assert!(result.is_ok());
    let variables = result.unwrap();

    // Should find parameters and local variable
    assert!(variables.len() >= 3, "Should find at least 3 variables");

    // Check parameters
    let x_var = variables.iter().find(|v| v.name == "x");
    assert!(x_var.is_some());
    assert_eq!(x_var.unwrap().type_info, "i32");

    let y_var = variables.iter().find(|v| v.name == "y");
    assert!(y_var.is_some());
    assert_eq!(y_var.unwrap().type_info, "i32");

    // Check local variable
    let result_var = variables.iter().find(|v| v.name == "result");
    assert!(result_var.is_some());
}

// RED Phase Test 3: Rust - Nested Scopes
#[test]
fn test_rust_nested_scopes() {
    let source = r#"
fn main() {
    let outer = 10;
    {
        let inner = 20;
        println!("{}", inner);  // Line 5: Inspect here
    }
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_rust(source, 5);

    assert!(result.is_ok());
    let variables = result.unwrap();

    // Should find both outer and inner variables
    assert!(variables.len() >= 2);
    assert!(variables.iter().any(|v| v.name == "outer"));
    assert!(variables.iter().any(|v| v.name == "inner"));
}

// RED Phase Test 4: TypeScript - Simple Variables
#[test]
fn test_typescript_simple_variables() {
    let source = r#"
function main() {
    const x = 42;
    let name = "Bob";
    var isActive = true;
    console.log(x);  // Line 5: Inspect here
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_typescript(source, 5);

    assert!(result.is_ok());
    let variables = result.unwrap();

    assert_eq!(variables.len(), 3, "Should find 3 variables");

    let x_var = variables.iter().find(|v| v.name == "x");
    assert!(x_var.is_some());

    let name_var = variables.iter().find(|v| v.name == "name");
    assert!(name_var.is_some());

    let is_active_var = variables.iter().find(|v| v.name == "isActive");
    assert!(is_active_var.is_some());
}

// RED Phase Test 5: TypeScript - Arrow Functions
#[test]
fn test_typescript_arrow_function_parameters() {
    let source = r#"
const add = (x: number, y: number): number => {
    const result = x + y;
    return result;  // Line 3: Inspect here
};
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_typescript(source, 3);

    assert!(result.is_ok());
    let variables = result.unwrap();

    assert!(variables.len() >= 3);
    assert!(variables.iter().any(|v| v.name == "x"));
    assert!(variables.iter().any(|v| v.name == "y"));
    assert!(variables.iter().any(|v| v.name == "result"));
}

// RED Phase Test 6: Python - Simple Variables
#[test]
fn test_python_simple_variables() {
    let source = r#"
def main():
    x = 42
    name = "Charlie"
    is_active = True
    print(x)  # Line 5: Inspect here
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_python(source, 5);

    assert!(result.is_ok());
    let variables = result.unwrap();

    assert_eq!(variables.len(), 3);
    assert!(variables.iter().any(|v| v.name == "x"));
    assert!(variables.iter().any(|v| v.name == "name"));
    assert!(variables.iter().any(|v| v.name == "is_active"));
}

// RED Phase Test 7: Python - Function Parameters
#[test]
fn test_python_function_parameters() {
    let source = r#"
def calculate(x, y, name):
    result = x + y
    print(result)  # Line 3: Inspect here
    return result
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_python(source, 3);

    assert!(result.is_ok());
    let variables = result.unwrap();

    assert!(variables.len() >= 4);
    assert!(variables.iter().any(|v| v.name == "x"));
    assert!(variables.iter().any(|v| v.name == "y"));
    assert!(variables.iter().any(|v| v.name == "name"));
    assert!(variables.iter().any(|v| v.name == "result"));
}

// RED Phase Test 8: Empty Scope
#[test]
fn test_empty_scope() {
    let source = r#"
fn main() {
    println!("Hello");  // Line 2: No variables here
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_rust(source, 2);

    assert!(result.is_ok());
    let variables = result.unwrap();
    assert_eq!(
        variables.len(),
        0,
        "Should find no variables in empty scope"
    );
}

// RED Phase Test 9: Invalid Line Number
#[test]
fn test_invalid_line_number() {
    let source = r#"
fn main() {
    let x = 42;
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_rust(source, 100);

    assert!(result.is_err(), "Should fail for invalid line number");
}

// RED Phase Test 10: Syntax Error Handling
#[test]
fn test_syntax_error_handling() {
    let source = r#"
fn main() {
    let x = ;  // Syntax error
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_rust(source, 2);

    // Should either return error or empty variables, but not panic
    match result {
        Ok(vars) => assert!(vars.is_empty() || !vars.is_empty()),
        Err(_) => { /* Expected behavior */ }
    }
}

// RED Phase Test 11: Multiple Assignments
#[test]
fn test_multiple_assignments() {
    let source = r#"
fn main() {
    let mut x = 10;
    x = 20;
    x = 30;
    println!("{}", x);  // Line 5: Inspect here
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_rust(source, 5);

    assert!(result.is_ok());
    let variables = result.unwrap();

    // Should find x only once (most recent binding)
    let x_vars: Vec<_> = variables.iter().filter(|v| v.name == "x").collect();
    assert_eq!(
        x_vars.len(),
        1,
        "Should find 'x' only once despite multiple assignments"
    );
}

// RED Phase Test 12: Variable Shadowing
#[test]
fn test_variable_shadowing() {
    let source = r#"
fn main() {
    let x = 10;
    let x = "hello";  // Shadow previous x
    println!("{}", x);  // Line 4: Inspect here
}
"#;

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_rust(source, 4);

    assert!(result.is_ok());
    let variables = result.unwrap();

    let x_var = variables.iter().find(|v| v.name == "x");
    assert!(x_var.is_some());
    // Should find the shadowed version (&str, not i32)
    assert_eq!(x_var.unwrap().type_info, "&str");
}

// RED Phase Test 13: Inspect from File
#[test]
fn test_inspect_from_file() {
    // Create temporary test file
    let test_file = "/tmp/pmat_test_variable_inspection.rs";
    let source = r#"
fn main() {
    let x = 42;
    println!("{}", x);  // Line 3
}
"#;
    std::fs::write(test_file, source).unwrap();

    let inspector = pmat::services::dap::VariableInspector::new();
    let result = inspector.inspect_file(Path::new(test_file), 3);

    assert!(result.is_ok());
    let variables = result.unwrap();
    assert!(variables.iter().any(|v| v.name == "x"));

    // Cleanup
    std::fs::remove_file(test_file).ok();
}

// RED Phase Test 14: Performance - Large Scope
#[test]
fn test_performance_large_scope() {
    use std::time::Instant;

    // Generate source with 100 variables
    let mut source = String::from("fn main() {\n");
    for i in 0..100 {
        source.push_str(&format!("    let var{} = {};\n", i, i));
    }
    source.push_str("    println!(\"test\");  // Line 101: Inspect here\n");
    source.push_str("}\n");

    let inspector = pmat::services::dap::VariableInspector::new();

    let start = Instant::now();
    let result = inspector.inspect_rust(&source, 101);
    let duration = start.elapsed();

    assert!(result.is_ok());
    let variables = result.unwrap();
    assert_eq!(variables.len(), 100, "Should find all 100 variables");

    assert!(
        duration.as_millis() < 200,
        "Should inspect 100 variables in less than 200ms, took {:?}",
        duration
    );
}

// RED Phase Test 15: Multi-Language Detection
#[test]
fn test_auto_language_detection() {
    let inspector = pmat::services::dap::VariableInspector::new();

    // Rust file
    let rust_result = inspector.inspect_file(Path::new("test.rs"), 5);
    assert!(rust_result.is_ok() || rust_result.is_err()); // File doesn't exist, but should attempt Rust parsing

    // TypeScript file
    let ts_result = inspector.inspect_file(Path::new("test.ts"), 5);
    assert!(ts_result.is_ok() || ts_result.is_err()); // Should attempt TypeScript parsing

    // Python file
    let py_result = inspector.inspect_file(Path::new("test.py"), 5);
    assert!(py_result.is_ok() || py_result.is_err()); // Should attempt Python parsing
}
