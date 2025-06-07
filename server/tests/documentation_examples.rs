use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;

fn get_binary_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap();

    let release_binary = workspace_root.join("target/release/paiml-mcp-agent-toolkit");
    let debug_binary = workspace_root.join("target/debug/paiml-mcp-agent-toolkit");

    if release_binary.exists() {
        release_binary.to_string_lossy().to_string()
    } else if debug_binary.exists() {
        debug_binary.to_string_lossy().to_string()
    } else {
        "paiml-mcp-agent-toolkit".to_string()
    }
}

#[test]
fn test_cli_examples_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/todo/active/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");
    let code_block_regex = Regex::new(r"```bash\n((?:[^`]|`[^`]|``[^`])+)\n```").unwrap();
    let binary_path = get_binary_path();

    for cap in code_block_regex.captures_iter(&content) {
        process_bash_code_block(&cap[1], &binary_path);
    }
}

fn process_bash_code_block(code_block: &str, binary_path: &str) {
    for line in code_block.lines() {
        let line = line.trim();

        if should_skip_line(line) {
            continue;
        }

        let full_command = handle_multiline_command(line, code_block);
        let test_command = full_command.replace("paiml-mcp-agent-toolkit", binary_path);

        validate_command(&test_command, binary_path, line);
    }
}

fn should_skip_line(line: &str) -> bool {
    // Skip comments, empty lines, and non-command lines
    if line.starts_with('#') || line.is_empty() || !line.contains("paiml-mcp-agent-toolkit") {
        return true;
    }

    // Check if paiml-mcp-agent-toolkit is actually the command
    let first_word = line.split_whitespace().next().unwrap_or("");
    if !first_word.contains("paiml-mcp-agent-toolkit") && !first_word.contains("=") {
        return true;
    }

    // Skip complex examples
    if has_complex_shell_features(line) {
        return true;
    }

    // Skip non-toolkit commands
    if is_non_toolkit_command(line) {
        return true;
    }

    // Skip environment variable settings
    line.contains("=") && line.split_whitespace().next().unwrap_or("").contains("=")
}

fn has_complex_shell_features(line: &str) -> bool {
    line.contains("|") || line.contains(">") || line.contains("$") || line.contains("curl")
}

fn is_non_toolkit_command(line: &str) -> bool {
    line.starts_with("git ")
        || line.starts_with("cd ")
        || line.starts_with("make ")
        || line.starts_with("claude ")
}

fn handle_multiline_command(line: &str, code_block: &str) -> String {
    if !line.ends_with('\\') {
        return line.to_string();
    }

    let mut cmd = line.trim_end_matches('\\').to_string();
    let mut lines_iter = code_block.lines().skip_while(|l| !l.contains(line));
    lines_iter.next(); // Skip current line

    for next_line in lines_iter {
        let next_line = next_line.trim();
        cmd.push(' ');

        if next_line.ends_with('\\') {
            cmd.push_str(next_line.trim_end_matches('\\'));
        } else {
            cmd.push_str(next_line);
            break;
        }
    }

    cmd
}

fn validate_command(test_command: &str, binary_path: &str, original_line: &str) {
    let parts: Vec<&str> = test_command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    validate_binary_path(parts[0], binary_path);
    validate_command_arguments(&parts, original_line);
}

fn validate_binary_path(command: &str, expected_binary_path: &str) {
    let is_valid_binary =
        command == expected_binary_path || command.ends_with("paiml-mcp-agent-toolkit");
    assert!(
        is_valid_binary,
        "Example command doesn't use the expected binary: {command} (expected {expected_binary_path} or ending with paiml-mcp-agent-toolkit)"
    );
}

fn validate_command_arguments(parts: &[&str], original_line: &str) {
    if parts.len() <= 1 {
        return;
    }

    let valid_commands = [
        "generate",
        "scaffold",
        "list",
        "search",
        "validate",
        "context",
        "analyze",
        "demo",
        "serve",
        "--help",
        "--version",
        "--mode",
    ];

    let first_arg = parts[1];
    assert!(
        valid_commands.contains(&first_arg),
        "Example uses unknown command: {first_arg} in line: {original_line}"
    );
}

#[test]
fn test_mcp_json_examples_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/todo/active/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");
    let json_block_regex = Regex::new(r"```json\n((?:[^`]|`[^`]|``[^`])+)\n```").unwrap();

    for cap in json_block_regex.captures_iter(&content) {
        validate_json_block(&cap[1]);
    }
}

fn validate_json_block(json_block: &str) {
    match serde_json::from_str::<Value>(json_block) {
        Ok(json) => validate_parsed_json(&json),
        Err(_) => validate_json_array_fallback(json_block),
    }
}

fn validate_parsed_json(json: &Value) {
    if let Some(obj) = json.as_object() {
        validate_json_rpc_object(obj);
    }
}

fn validate_json_rpc_object(obj: &serde_json::Map<String, Value>) {
    if !obj.contains_key("jsonrpc") {
        return;
    }

    assert_eq!(
        obj["jsonrpc"].as_str(),
        Some("2.0"),
        "JSON-RPC version should be 2.0"
    );

    assert!(
        obj.contains_key("method"),
        "JSON-RPC request should have a method"
    );

    assert!(obj.contains_key("id"), "JSON-RPC request should have an id");
}

fn validate_json_array_fallback(json_block: &str) {
    if !json_block.trim().starts_with('[') {
        panic!("Invalid JSON example in documentation: {json_block}");
    }

    match serde_json::from_str::<Vec<Value>>(json_block) {
        Ok(array) => validate_batch_request_array(&array),
        Err(e) => panic!("Invalid JSON example in documentation: {json_block}\nError: {e}"),
    }
}

fn validate_batch_request_array(array: &[Value]) {
    assert!(!array.is_empty(), "JSON array example should not be empty");

    for item in array {
        assert!(item.is_object(), "Batch request items should be objects");
        let obj = item.as_object().unwrap();
        assert_eq!(
            obj.get("jsonrpc").and_then(|v| v.as_str()),
            Some("2.0"),
            "Each batch item should have jsonrpc: 2.0"
        );
    }
}

#[test]
fn test_yaml_examples_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/todo/active/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    // Extract YAML code blocks (like GitHub Actions examples)
    let yaml_block_regex = Regex::new(r"```yaml\n((?:[^`]|`[^`]|``[^`])+)\n```").unwrap();

    for cap in yaml_block_regex.captures_iter(&content) {
        let yaml_block = &cap[1];

        // Basic validation - ensure it's not empty and has proper structure
        assert!(
            !yaml_block.trim().is_empty(),
            "YAML example should not be empty"
        );

        // Check for common YAML patterns
        if yaml_block.contains("name:") && yaml_block.contains("run:") {
            // This looks like a GitHub Actions snippet
            assert!(
                yaml_block.contains("paiml-mcp-agent-toolkit") || yaml_block.contains("cargo test"),
                "GitHub Actions example should reference the tool or tests"
            );
        }
    }
}

#[test]
fn test_jsonc_examples_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/todo/active/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    // Extract JSONC code blocks (JSON with comments, like VS Code config)
    let jsonc_block_regex = Regex::new(r"```jsonc\n((?:[^`]|`[^`]|``[^`])+)\n```").unwrap();

    for cap in jsonc_block_regex.captures_iter(&content) {
        let jsonc_block = &cap[1];

        // Remove comments for parsing
        let without_comments = jsonc_block
            .lines()
            .filter(|line| !line.trim().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        // Try to parse as JSON after removing comments
        match serde_json::from_str::<Value>(&without_comments) {
            Ok(json) => {
                // Verify it's a valid VS Code task or similar config
                if let Some(obj) = json.as_object() {
                    if obj.contains_key("label") && obj.contains_key("command") {
                        assert!(
                            obj["command"].as_str() == Some("paiml-mcp-agent-toolkit"),
                            "VS Code task should use paiml-mcp-agent-toolkit command"
                        );
                    }
                }
            }
            Err(e) => {
                // JSONC might have trailing commas or other relaxed syntax
                // Just ensure it's not completely broken
                assert!(
                    jsonc_block.contains("paiml-mcp-agent-toolkit"),
                    "JSONC example should reference the tool. Parse error: {e}"
                );
            }
        }
    }
}

#[test]
fn test_template_uri_examples_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/todo/active/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    // Extract template URIs
    let uri_regex = Regex::new(r"template://([a-z-]+)/([a-z-]+)/([a-z-]+)").unwrap();

    let valid_categories = ["makefile", "readme", "gitignore"];
    let valid_toolchains = ["rust", "deno", "python-uv"];
    let valid_variants = ["cli"];

    for cap in uri_regex.captures_iter(&content) {
        let category = &cap[1];
        let toolchain = &cap[2];
        let variant = &cap[3];

        assert!(
            valid_categories.contains(&category),
            "Invalid category '{category}' in template URI"
        );

        assert!(
            valid_toolchains.contains(&toolchain),
            "Invalid toolchain '{toolchain}' in template URI"
        );

        assert!(
            valid_variants.contains(&variant),
            "Invalid variant '{variant}' in template URI"
        );
    }
}

#[test]
fn test_performance_numbers_are_reasonable() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/todo/active/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    // Check that documented performance numbers are reasonable
    let perf_regex = Regex::new(r"<(\d+)ms").unwrap();

    for cap in perf_regex.captures_iter(&content) {
        let ms = cap[1].parse::<u32>().unwrap();

        // Sanity check - nothing should claim to be faster than 1ms
        // or slower than 1000ms for basic operations
        assert!(
            (1..=1000).contains(&ms),
            "Unrealistic performance claim: {ms}ms"
        );
    }

    // Check cache hit rates
    let cache_regex = Regex::new(r">(\d+)%").unwrap();

    for cap in cache_regex.captures_iter(&content) {
        let percentage = cap[1].parse::<u32>().unwrap();

        // Cache hit rates should be between 0 and 100
        assert!(percentage <= 100, "Invalid cache hit rate: {percentage}%");
    }
}
