use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DocumentedTool {
    name: String,
    description: String,
    required_params: Vec<String>,
    optional_params: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct McpResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ToolDefinition {
    name: String,
    description: Option<String>,
    #[serde(rename = "inputSchema")]
    input_schema: Option<Value>,
}

/// The documentation this suite grades, or `None` when it was never shipped.
///
/// `None` has exactly ONE cause: the published crate excludes `/docs/`
/// (Cargo.toml:26) while shipping `tests/`, so a consumer running `cargo test` on
/// the crates.io tarball has the tests but not the document. That is not drift and
/// not something this suite can measure, so it says so and stops.
///
/// Everything else PANICS. If `docs/` is present — which it is in every source
/// checkout and every CI job — then a missing or renamed `mcp-methods.md` is drift, and
/// #1228 is the record of what happens when that reads as "ok": five green tests,
/// 0.00s, asserting nothing, while the document they guard drifted 60 commands.
fn mcp_methods_doc() -> Option<String> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if !repo_root.join("docs").is_dir() {
        eprintln!(
            "NOT-SHIPPED: docs/ is absent, so this is the packaged crate rather \
             than a source checkout; mcp-methods.md was never included. Nothing to compare."
        );
        return None;
    }
    let doc_path = repo_root.join("docs/mcp-methods.md");
    Some(fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "mcp-methods.md not found at {} ({e}), but docs/ exists — so this is a \
             source checkout and the document has been moved, renamed or deleted. \
             That is drift; it must fail rather than skip (#1228).",
            doc_path.display()
        )
    }))
}

fn parse_documented_mcp_tools() -> Vec<DocumentedTool> {
    let Some(content) = mcp_methods_doc() else {
        return Default::default();
    };

    let mut tools = Vec::new();

    // Look for MCP tool documentation patterns
    // Tools are typically mentioned in the MCP sections
    let tool_patterns = vec![
        (
            "generate_template",
            "Generate templates with parameter substitution",
        ),
        ("get_server_info", "Get information about the server"),
        ("list_templates", "List available templates"),
        ("scaffold_project", "Scaffold a complete project"),
        ("search_templates", "Search for templates"),
        ("validate_template", "Validate template parameters"),
        ("analyze_complexity", "Analyze code complexity"),
        ("analyze_code_churn", "Analyze code change patterns"),
        ("analyze_dag", "Generate dependency graphs"),
        ("generate_context", "Generate project context"),
        ("analyze_dead_code", "Analyze dead code"),
        ("analyze_deep_context", "Analyze deep context"),
        ("analyze_satd", "Analyze self-admitted technical debt"),
        ("analyze_tdg", "Calculate technical debt gradient"),
        ("analyze_lint_hotspot", "Analyze lint violation hotspots"),
        // Vectorized tools
        (
            "analyze_duplicates_vectorized",
            "Analyze code duplicates using SIMD",
        ),
        (
            "analyze_graph_metrics_vectorized",
            "Analyze graph metrics using vectorization",
        ),
        (
            "analyze_name_similarity_vectorized",
            "Analyze name similarity using SIMD",
        ),
        (
            "analyze_symbol_table_vectorized",
            "Analyze symbol tables with vectorization",
        ),
        (
            "analyze_incremental_coverage_vectorized",
            "Analyze incremental coverage with SIMD",
        ),
        (
            "analyze_big_o_vectorized",
            "Analyze Big O complexity using vectorization",
        ),
        (
            "generate_enhanced_report",
            "Generate enhanced analysis report",
        ),
    ];

    // Pre-compile the parameter extraction regex outside the loop
    let param_regex = Regex::new(r#""([^"]+)":\s*[^,}]+"#).unwrap();

    for (name, description) in tool_patterns {
        // Check if the tool is mentioned in the documentation
        if content.contains(name) {
            // Extract parameters from MCP examples
            let mut required_params = Vec::new();
            let mut optional_params = Vec::new();

            // Look for JSON examples containing this tool
            let json_regex = Regex::new(&format!(
                r#""name":\s*"{name}".*?"arguments":\s*\{{([^}}]+)\}}"#
            ))
            .unwrap();
            if let Some(cap) = json_regex.captures(&content) {
                let args_content = &cap[1];

                // Extract parameter names from the JSON
                for param_cap in param_regex.captures_iter(args_content) {
                    let param_name = param_cap[1].to_string();

                    // Determine if required or optional based on common patterns
                    if param_name == "resource_uri" || param_name == "project_path" {
                        required_params.push(param_name);
                    } else {
                        optional_params.push(param_name);
                    }
                }
            }

            tools.push(DocumentedTool {
                name: name.to_string(),
                description: description.to_string(),
                required_params,
                optional_params,
            });
        }
    }

    tools
}

/// The binary this test grades.
///
/// #1228: this hand-built `<repo>/target/{release,debug}/pmat`. Two things were
/// wrong with that. The path was rooted at `CARGO_MANIFEST_DIR.parent()`, one level
/// ABOVE the repository, so neither candidate existed and it fell through to the
/// literal "pmat" — whatever was on PATH, typically a `cargo install`ed copy from an
/// older release. And even rooted correctly it ignores `CARGO_TARGET_DIR`: on a
/// machine that redirects the target directory (this repo's own does) the hand-built
/// path finds a STALE `./target/release/pmat` while the real build is elsewhere —
/// the same "grade the wrong binary" failure, wearing a different hat.
///
/// `CARGO_BIN_EXE_pmat` is set by Cargo for integration test targets, points at the
/// binary Cargo just built for THIS test run, and honours the target directory. An
/// earlier revision of this function claimed it was unavailable here; that was
/// simply false, and measuring it took one `println!`.
fn get_binary_path() -> String {
    env!("CARGO_BIN_EXE_pmat").to_string()
}

fn send_mcp_request(request: Value) -> Result<McpResponse, String> {
    use std::io::{BufRead, BufReader};
    use std::time::{Duration, Instant};

    let binary_path = get_binary_path();

    // Start the MCP server in MCP mode by setting MCP_VERSION environment variable
    let mut child = Command::new(&binary_path)
        .env("MCP_VERSION", "1.0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start MCP server: {e}"))?;

    let mut stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

    // Send request
    let request_str = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    stdin
        .write_all(request_str.as_bytes())
        .map_err(|e| e.to_string())?;
    stdin.write_all(b"\n").map_err(|e| e.to_string())?;
    stdin.flush().map_err(|e| e.to_string())?;
    drop(stdin);

    // Read response with timeout
    let reader = BufReader::new(stdout);
    let start = Instant::now();
    let timeout = Duration::from_secs(10); // 10 second timeout

    for line in reader.lines() {
        // Check timeout
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Timeout waiting for MCP response".to_string());
        }

        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(response) = serde_json::from_str::<McpResponse>(&line) {
            // Kill the child process since we got our response
            let _ = child.kill();
            let _ = child.wait();
            return Ok(response);
        }
    }

    // Kill the child process if we didn't find a response
    let _ = child.kill();
    let _ = child.wait();

    Err("No valid JSON response found in output".to_string())
}

#[test]
#[ignore = "Slow test - requires binary and MCP server interaction"]
fn test_mcp_tools_match_documentation() {
    // Skip in CI or when SKIP_SLOW_TESTS is set
    if std::env::var("CI").is_ok() || std::env::var("SKIP_SLOW_TESTS").unwrap_or_default() == "true"
    {
        eprintln!("Skipping MCP server test in CI environment");
        return;
    }

    // First, initialize the MCP connection
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {}
        }
    });

    let init_response =
        send_mcp_request(init_request).expect("Failed to initialize MCP connection");

    assert!(
        init_response.error.is_none(),
        "MCP initialization failed: {:?}",
        init_response.error
    );

    // Get list of available tools
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });

    let tools_response = send_mcp_request(tools_request).expect("Failed to get tools list");

    assert!(
        tools_response.error.is_none(),
        "Failed to list tools: {:?}",
        tools_response.error
    );

    let tools_result = tools_response.result.expect("No result in tools response");
    let tools_array = tools_result["tools"]
        .as_array()
        .expect("Tools result should contain a tools array");

    // Parse actual tools from response
    let actual_tools: Vec<ToolDefinition> = tools_array
        .iter()
        .filter_map(|t| serde_json::from_value(t.clone()).ok())
        .collect();

    let actual_tool_names: Vec<String> = actual_tools.iter().map(|t| t.name.clone()).collect();

    // Parse documented tools
    let documented_tools = parse_documented_mcp_tools();

    // Verify all documented tools exist
    for doc_tool in &documented_tools {
        assert!(
            actual_tool_names.contains(&doc_tool.name),
            "Documented MCP tool '{}' not found in actual tools. Available tools: {:?}",
            doc_tool.name,
            actual_tool_names
        );
    }
}

#[test]
#[ignore = "Slow test - requires binary and MCP server interaction"]
fn test_mcp_tool_schemas_match_documentation() {
    // Skip in CI or when SKIP_SLOW_TESTS is set
    if std::env::var("CI").is_ok() || std::env::var("SKIP_SLOW_TESTS").unwrap_or_default() == "true"
    {
        eprintln!("Skipping MCP server test in CI environment");
        return;
    }

    // Initialize connection
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {}
        }
    });

    send_mcp_request(init_request).expect("Failed to initialize");

    // Get tools
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });

    let tools_response = send_mcp_request(tools_request).expect("Failed to get tools list");

    let tools_result = tools_response.result.expect("No result in tools response");
    let tools_array = tools_result["tools"]
        .as_array()
        .expect("Tools result should contain a tools array");

    let documented_tools = parse_documented_mcp_tools();

    // Check schemas for each documented tool
    for doc_tool in &documented_tools {
        // Find the actual tool definition
        let actual_tool = tools_array
            .iter()
            .find(|t| t["name"].as_str() == Some(&doc_tool.name));

        if let Some(tool) = actual_tool {
            // Check input schema if present
            if let Some(schema) = tool.get("inputSchema") {
                if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                    // Check documented required params exist
                    for req_param in &doc_tool.required_params {
                        assert!(
                            properties.contains_key(req_param),
                            "Documented required parameter '{}' not found in schema for tool '{}'",
                            req_param,
                            doc_tool.name
                        );
                    }
                }

                // Check required array matches
                if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
                    let actual_required: Vec<String> = required
                        .iter()
                        .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                        .collect();

                    for doc_req in &doc_tool.required_params {
                        assert!(
                            actual_required.contains(doc_req),
                            "Documented parameter '{}' should be required for tool '{}' but isn't",
                            doc_req,
                            doc_tool.name
                        );
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "Documentation structure test - may fail during development"]
fn test_mcp_methods_match_documentation() {
    let Some(content) = mcp_methods_doc() else {
        return Default::default();
    };

    // Extract documented MCP methods from the "Available MCP Methods" section
    let methods_section = content
        .split("### Available MCP Methods")
        .nth(1)
        .and_then(|s| s.split("###").next())
        .expect("Could not find Available MCP Methods section");

    let mut documented_methods = Vec::new();
    let method_regex = Regex::new(r"`([a-z/]+)`\s*-").unwrap();

    for cap in method_regex.captures_iter(methods_section) {
        documented_methods.push(cap[1].to_string());
    }

    // These are the standard MCP methods that should be supported
    let expected_methods = vec![
        "initialize",
        "tools/list",
        "tools/call",
        "resources/list",
        "resources/read",
        "prompts/list",
    ];

    for method in &expected_methods {
        assert!(
            documented_methods.contains(&(*method).to_string()),
            "Expected MCP method '{method}' not documented"
        );
    }
}

#[test]
#[ignore = "Documentation structure test - may fail during development"]
fn test_mcp_error_codes_are_complete() {
    let Some(content) = mcp_methods_doc() else {
        return Default::default();
    };

    // Extract error codes from documentation
    let error_section = content
        .split("### Error Codes")
        .nth(1)
        .and_then(|s| s.split("###").next())
        .expect("Could not find Error Codes section");

    let mut documented_errors = Vec::new();
    let error_regex = Regex::new(r"\|\s*(-?\d+)\s*\|").unwrap();

    for cap in error_regex.captures_iter(error_section) {
        if let Ok(code) = cap[1].parse::<i32>() {
            documented_errors.push(code);
        }
    }

    // Standard JSON-RPC error codes that should be documented
    let standard_errors = vec![-32700, -32600, -32601, -32602];

    for error_code in &standard_errors {
        assert!(
            documented_errors.contains(error_code),
            "Standard JSON-RPC error code {error_code} not documented"
        );
    }
}

#[test]
#[ignore = "Slow test - requires binary and MCP server interaction"]
fn test_no_undocumented_mcp_tools() {
    // Skip in CI or when SKIP_SLOW_TESTS is set
    if std::env::var("CI").is_ok() || std::env::var("SKIP_SLOW_TESTS").unwrap_or_default() == "true"
    {
        eprintln!("Skipping MCP server test in CI environment");
        return;
    }

    // Initialize and get tools list
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {}
        }
    });

    send_mcp_request(init_request).expect("Failed to initialize");

    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });

    let tools_response = send_mcp_request(tools_request).expect("Failed to get tools list");

    let tools_result = tools_response.result.expect("No result");
    let tools_array = tools_result["tools"].as_array().expect("No tools array");

    let documented_tools = parse_documented_mcp_tools();
    if documented_tools.is_empty() {
        eprintln!("No documented tools found, skipping test");
        return;
    }
    let documented_names: Vec<String> = documented_tools.iter().map(|t| t.name.clone()).collect();

    // Check for undocumented tools
    for tool in tools_array {
        if let Some(name) = tool["name"].as_str() {
            // Skip internal tools
            if name.starts_with('_') {
                continue;
            }

            assert!(
                documented_names.iter().any(|doc_name| doc_name == name),
                "MCP tool '{name}' exists but is not documented in mcp-methods.md"
            );
        }
    }
}
