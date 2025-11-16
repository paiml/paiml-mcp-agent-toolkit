// TRACE-004: DAP-PMAT Integration Tests
// Sprint 71 - RED Phase
//
// Integration tests for DAP server with PMAT's existing infrastructure:
// - Language detection using LanguageAnalyzer
// - Tree-sitter AST caching
// - Variable inspection integration

use pmat::cli::language_analyzer::Language;
use serde_json::json;

// RED Test 1: Language Detection - Rust Files
#[test]
fn test_dap_detects_rust_language() {
    let server = pmat::services::dap::DapServer::new();

    // Initialize server
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {
            "clientID": "test",
            "adapterID": "pmat"
        }
    });
    server.handle_request(init_request);

    // Launch with Rust file
    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/sample.rs"
        }
    });
    server.handle_request(launch_request);

    // Should detect Rust language
    assert_eq!(
        server.current_language(),
        Some(Language::Rust),
        "Should detect Rust from .rs file"
    );
}

// RED Test 2: Language Detection - Python Files
#[test]
fn test_dap_detects_python_language() {
    let server = pmat::services::dap::DapServer::new();

    // Initialize and launch
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/sample.py"
        }
    });
    server.handle_request(launch_request);

    // Should detect Python language
    assert_eq!(
        server.current_language(),
        Some(Language::Python),
        "Should detect Python from .py file"
    );
}

// RED Test 3: Language Detection - No Language Before Launch
#[test]
fn test_dap_no_language_before_launch() {
    let server = pmat::services::dap::DapServer::new();

    // Before launch, no language should be detected
    assert_eq!(
        server.current_language(),
        None,
        "Should have no language before program is launched"
    );
}

// RED Test 4: AST Caching - Stores AST After Launch
#[test]
fn test_dap_caches_ast_after_launch() {
    let server = pmat::services::dap::DapServer::new();

    // Initialize and launch
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/sample.rs"
        }
    });
    server.handle_request(launch_request);

    // Should have cached AST for the file
    assert!(
        server.has_ast_for("tests/fixtures/sample.rs"),
        "Should cache AST for launched program"
    );
}

// RED Test 5: AST Caching - No AST Before Launch
#[test]
fn test_dap_no_ast_before_launch() {
    let server = pmat::services::dap::DapServer::new();

    // Before launch, no AST should be cached
    assert!(
        !server.has_ast_for("tests/fixtures/sample.rs"),
        "Should not have AST before launch"
    );
}

// RED Test 6: AST Caching - Multiple Files
#[test]
fn test_dap_caches_ast_for_multiple_files() {
    let server = pmat::services::dap::DapServer::new();

    // Initialize
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    // Launch first file
    let launch_request1 = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/sample.rs"
        }
    });
    server.handle_request(launch_request1);

    // Set breakpoints in another file (should trigger AST caching)
    let bp_request = json!({
        "seq": 3,
        "type": "request",
        "command": "setBreakpoints",
        "arguments": {
            "source": {"path": "tests/fixtures/complex.rs"},
            "breakpoints": [{"line": 10}]
        }
    });
    server.handle_request(bp_request);

    // Should have AST for both files
    assert!(
        server.has_ast_for("tests/fixtures/sample.rs"),
        "Should cache AST for launched program"
    );
    assert!(
        server.has_ast_for("tests/fixtures/complex.rs"),
        "Should cache AST when breakpoints are set"
    );
}

// RED Test 7: Variable Inspection - Extract Variables at Breakpoint
#[test]
fn test_dap_extracts_variables_at_breakpoint() {
    let server = pmat::services::dap::DapServer::new();

    // Initialize and launch
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/complex.rs"
        }
    });
    server.handle_request(launch_request);

    // Set breakpoint at line 11 (deeply_nested scope)
    let bp_request = json!({
        "seq": 3,
        "type": "request",
        "command": "setBreakpoints",
        "arguments": {
            "source": {"path": "tests/fixtures/complex.rs"},
            "breakpoints": [{"line": 11}]
        }
    });
    server.handle_request(bp_request);

    // Simulate stopping at breakpoint
    let variables = server.get_variables_at_line("tests/fixtures/complex.rs", 11);

    assert!(
        variables.is_ok(),
        "Should successfully extract variables: {:?}",
        variables
    );

    let vars = variables.unwrap();
    assert!(
        vars.len() > 0,
        "Should find variables at line 11 (deeply_nested scope)"
    );

    // Should find deeply_nested, combined, outer_var, param1, param2
    let var_names: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
    assert!(
        var_names.contains(&"deeply_nested".to_string()),
        "Should find deeply_nested variable. Found: {:?}",
        var_names
    );
}

// RED Test 8: Variable Inspection - Returns Empty for No Variables
#[test]
fn test_dap_returns_empty_variables_when_none_exist() {
    let server = pmat::services::dap::DapServer::new();

    // Initialize and launch
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/sample.rs"
        }
    });
    server.handle_request(launch_request);

    // Line 1 is just a comment, no variables
    let variables = server.get_variables_at_line("tests/fixtures/sample.rs", 1);

    assert!(variables.is_ok(), "Should succeed even with no variables");
    assert_eq!(
        variables.unwrap().len(),
        0,
        "Should return empty vector for comment line"
    );
}

// RED Test 9: Variable Inspection - Handles Shadowing
#[test]
fn test_dap_handles_variable_shadowing() {
    let server = pmat::services::dap::DapServer::new();

    // Initialize and launch
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/complex.rs"
        }
    });
    server.handle_request(launch_request);

    // Line 25 is after shadowing: let x = "shadowed"
    let variables = server.get_variables_at_line("tests/fixtures/complex.rs", 25);

    assert!(variables.is_ok(), "Should handle shadowed variables");

    let vars = variables.unwrap();
    let x_var = vars.iter().find(|v| v.name == "x");

    assert!(x_var.is_some(), "Should find shadowed variable 'x'");
    assert_eq!(
        x_var.unwrap().type_info,
        "&str",
        "Should return &str type for shadowed x (not i32)"
    );
}

// RED Test 10: Variable Inspection - Python Support
#[test]
fn test_dap_extracts_python_variables() {
    let server = pmat::services::dap::DapServer::new();

    // Initialize and launch Python file
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/sample.py"
        }
    });
    server.handle_request(launch_request);

    // Line 5 has variables: x, y, result, doubled
    let variables = server.get_variables_at_line("tests/fixtures/sample.py", 5);

    assert!(variables.is_ok(), "Should extract Python variables");

    let vars = variables.unwrap();
    assert!(
        vars.len() >= 3,
        "Should find at least 3 variables (x, y, result, doubled)"
    );

    let var_names: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
    assert!(
        var_names.contains(&"result".to_string()),
        "Should find 'result' variable"
    );
}

// RED Test 11: Integration - Scopes Request Uses Variable Inspector
#[test]
fn test_dap_scopes_request_integrates_with_variable_inspector() {
    let mut server = pmat::services::dap::DapServer::new();

    // Initialize and launch
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/complex.rs"
        }
    });
    server.handle_request(launch_request);

    // Simulate stopped at line 11
    server.simulate_stop_at_line("tests/fixtures/complex.rs", 11);

    // Request scopes
    let scopes_request = json!({
        "seq": 3,
        "type": "request",
        "command": "scopes",
        "arguments": {
            "frameId": 0
        }
    });
    let response = server.handle_request(scopes_request);

    // Should return scopes with variable reference
    let scopes = response["body"]["scopes"].as_array();
    assert!(scopes.is_some(), "Should return scopes array");
    assert!(
        scopes.unwrap().len() > 0,
        "Should have at least one scope (locals)"
    );

    let local_scope = &scopes.unwrap()[0];
    assert_eq!(
        local_scope["name"].as_str().unwrap(),
        "Locals",
        "First scope should be 'Locals'"
    );
    assert!(
        local_scope["variablesReference"].as_i64().unwrap() > 0,
        "Should have non-zero variable reference"
    );
}

// RED Test 12: Integration - Variables Request Returns Actual Variables
#[test]
fn test_dap_variables_request_returns_inspected_variables() {
    let mut server = pmat::services::dap::DapServer::new();

    // Initialize and launch
    let init_request = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    server.handle_request(init_request);

    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": "tests/fixtures/complex.rs"
        }
    });
    server.handle_request(launch_request);

    // Simulate stopped at line 11
    server.simulate_stop_at_line("tests/fixtures/complex.rs", 11);

    // Get scopes to get variablesReference
    let scopes_request = json!({
        "seq": 3,
        "type": "request",
        "command": "scopes",
        "arguments": {"frameId": 0}
    });
    let scopes_response = server.handle_request(scopes_request);
    let var_ref = scopes_response["body"]["scopes"][0]["variablesReference"]
        .as_i64()
        .unwrap();

    // Request variables
    let vars_request = json!({
        "seq": 4,
        "type": "request",
        "command": "variables",
        "arguments": {
            "variablesReference": var_ref
        }
    });
    let response = server.handle_request(vars_request);

    // Should return actual variables from VariableInspector
    let variables = response["body"]["variables"].as_array();
    assert!(variables.is_some(), "Should return variables array");
    assert!(
        variables.unwrap().len() > 0,
        "Should have variables at line 11"
    );

    // Check for specific variable
    let vars_array = variables.unwrap();
    let has_deeply_nested = vars_array
        .iter()
        .any(|v| v["name"].as_str() == Some("deeply_nested"));
    assert!(
        has_deeply_nested,
        "Should include 'deeply_nested' variable. Got: {:?}",
        vars_array
    );
}
