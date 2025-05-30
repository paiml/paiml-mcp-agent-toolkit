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
        .join("docs/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    // Extract bash code blocks
    let code_block_regex = Regex::new(r"```bash\n((?:[^`]|`[^`]|``[^`])+)\n```").unwrap();
    let binary_path = get_binary_path();

    for cap in code_block_regex.captures_iter(&content) {
        let code_block = &cap[1];

        // Process each line in the code block
        for line in code_block.lines() {
            let line = line.trim();

            // Skip comments, empty lines, and non-command lines
            if line.starts_with('#') || line.is_empty() || !line.contains("paiml-mcp-agent-toolkit")
            {
                continue;
            }

            // Check if paiml-mcp-agent-toolkit is actually the command (not just in a path)
            let first_word = line.split_whitespace().next().unwrap_or("");
            if !first_word.contains("paiml-mcp-agent-toolkit") && !first_word.contains("=") {
                // paiml-mcp-agent-toolkit is not the command, skip this line
                continue;
            }

            // Skip complex examples with pipes, redirects, or shell variables
            if line.contains("|")
                || line.contains(">")
                || line.contains("$")
                || line.contains("curl")
            {
                continue;
            }

            // Skip git commands and other non-toolkit commands
            if line.starts_with("git ")
                || line.starts_with("cd ")
                || line.starts_with("make ")
                || line.starts_with("claude ")
            {
                continue;
            }

            // Skip lines with environment variable settings (e.g., RUST_LOG=...)
            if line.contains("=") && line.split_whitespace().next().unwrap_or("").contains("=") {
                continue;
            }

            // Handle multi-line commands (with backslash continuation)
            let full_command = if line.ends_with('\\') {
                // Collect all continuation lines
                let mut cmd = line.trim_end_matches('\\').to_string();
                let mut lines_iter = code_block.lines().skip_while(|l| !l.contains(line));
                lines_iter.next(); // Skip current line

                for next_line in lines_iter {
                    let next_line = next_line.trim();
                    if next_line.ends_with('\\') {
                        cmd.push(' ');
                        cmd.push_str(next_line.trim_end_matches('\\'));
                    } else {
                        cmd.push(' ');
                        cmd.push_str(next_line);
                        break;
                    }
                }
                cmd
            } else {
                line.to_string()
            };

            // Replace the binary name with our test binary path
            let test_command = full_command.replace("paiml-mcp-agent-toolkit", &binary_path);

            // Parse the command
            let parts: Vec<&str> = test_command.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            // Validate the command structure by checking if it would parse
            // The command has been replaced with our test binary path,
            // so we check if it's either the actual binary path or ends with the expected name
            let is_valid_binary =
                parts[0] == binary_path || parts[0].ends_with("paiml-mcp-agent-toolkit");
            assert!(
                is_valid_binary,
                "Example command doesn't use the expected binary: {} (expected {} or ending with paiml-mcp-agent-toolkit)",
                parts[0],
                binary_path
            );

            // Verify it's a known command or starts with valid flags
            if parts.len() > 1 {
                let first_arg = parts[1];
                let valid_commands = [
                    "generate",
                    "scaffold",
                    "list",
                    "search",
                    "validate",
                    "context",
                    "analyze",
                    "demo",
                    "--help",
                    "--version",
                    "--mode",
                ];

                assert!(
                    valid_commands.contains(&first_arg),
                    "Example uses unknown command: {} in line: {}",
                    first_arg,
                    line
                );
            }
        }
    }
}

#[test]
fn test_mcp_json_examples_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    // Extract JSON code blocks
    let json_block_regex = Regex::new(r"```json\n((?:[^`]|`[^`]|``[^`])+)\n```").unwrap();

    for cap in json_block_regex.captures_iter(&content) {
        let json_block = &cap[1];

        // Try to parse as JSON
        match serde_json::from_str::<Value>(json_block) {
            Ok(json) => {
                // Verify it looks like a JSON-RPC request
                if json.is_object() {
                    let obj = json.as_object().unwrap();

                    // Check for JSON-RPC structure
                    if obj.contains_key("jsonrpc") {
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
                }
            }
            Err(_) => {
                // It might be a JSON array (batch request)
                if json_block.trim().starts_with('[') {
                    match serde_json::from_str::<Vec<Value>>(json_block) {
                        Ok(array) => {
                            assert!(!array.is_empty(), "JSON array example should not be empty");

                            // Each element should be a valid JSON-RPC request
                            for item in &array {
                                assert!(item.is_object(), "Batch request items should be objects");
                                let obj = item.as_object().unwrap();
                                assert_eq!(
                                    obj.get("jsonrpc").and_then(|v| v.as_str()),
                                    Some("2.0"),
                                    "Each batch item should have jsonrpc: 2.0"
                                );
                            }
                        }
                        Err(e) => {
                            panic!(
                                "Invalid JSON example in documentation: {}\nError: {}",
                                json_block, e
                            );
                        }
                    }
                } else {
                    panic!("Invalid JSON example in documentation: {}", json_block);
                }
            }
        }
    }
}

#[test]
fn test_yaml_examples_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/cli-mcp.md");

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
        .join("docs/cli-mcp.md");

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
                    "JSONC example should reference the tool. Parse error: {}",
                    e
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
        .join("docs/cli-mcp.md");

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
            "Invalid category '{}' in template URI",
            category
        );

        assert!(
            valid_toolchains.contains(&toolchain),
            "Invalid toolchain '{}' in template URI",
            toolchain
        );

        assert!(
            valid_variants.contains(&variant),
            "Invalid variant '{}' in template URI",
            variant
        );
    }
}

#[test]
fn test_performance_numbers_are_reasonable() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    // Check that documented performance numbers are reasonable
    let perf_regex = Regex::new(r"<(\d+)ms").unwrap();

    for cap in perf_regex.captures_iter(&content) {
        let ms = cap[1].parse::<u32>().unwrap();

        // Sanity check - nothing should claim to be faster than 1ms
        // or slower than 1000ms for basic operations
        assert!(
            (1..=1000).contains(&ms),
            "Unrealistic performance claim: {}ms",
            ms
        );
    }

    // Check cache hit rates
    let cache_regex = Regex::new(r">(\d+)%").unwrap();

    for cap in cache_regex.captures_iter(&content) {
        let percentage = cap[1].parse::<u32>().unwrap();

        // Cache hit rates should be between 0 and 100
        assert!(percentage <= 100, "Invalid cache hit rate: {}%", percentage);
    }
}
