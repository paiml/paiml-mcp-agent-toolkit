use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// The documentation this suite grades, or `None` when it was never shipped.
///
/// `None` has exactly ONE cause: the published crate excludes `/rust-docs/`
/// (Cargo.toml:26) while shipping `tests/`, so a consumer running `cargo test` on
/// the crates.io tarball has the tests but not the document. That is not drift and
/// not something this suite can measure, so it says so and stops.
///
/// Everything else PANICS. If `rust-docs/` is present — which it is in every source
/// checkout and every CI job — then a missing or renamed `cli-reference.md` is drift, and
/// #1228 is the record of what happens when that reads as "ok": five green tests,
/// 0.00s, asserting nothing, while the document they guard drifted 60 commands.
fn cli_reference() -> Option<String> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if !repo_root.join("rust-docs").is_dir() {
        eprintln!(
            "NOT-SHIPPED: rust-docs/ is absent, so this is the packaged crate rather \
             than a source checkout; cli-reference.md was never included. Nothing to compare."
        );
        return None;
    }
    let doc_path = repo_root.join("rust-docs/cli-reference.md");
    Some(fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "cli-reference.md not found at {} ({e}), but rust-docs/ exists — so this is a \
             source checkout and the document has been moved, renamed or deleted. \
             That is drift; it must fail rather than skip (#1228).",
            doc_path.display()
        )
    }))
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

#[test]
fn test_cli_examples_are_valid() {
    let Some(content) = cli_reference() else {
        return Default::default();
    };
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
        let test_command = full_command.replace("pmat", binary_path);

        validate_command(&test_command, binary_path, line);
    }
}

fn should_skip_line(line: &str) -> bool {
    // Skip comments, empty lines, and non-command lines
    if line.starts_with('#') || line.is_empty() || !line.contains("pmat") {
        return true;
    }

    // Check if pmat is actually the command
    let first_word = line.split_whitespace().next().unwrap_or("");
    if !first_word.contains("pmat") && !first_word.contains('=') {
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
    line.contains('=') && line.split_whitespace().next().unwrap_or("").contains('=')
}

fn has_complex_shell_features(line: &str) -> bool {
    line.contains('|') || line.contains('>') || line.contains('$') || line.contains("curl")
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
    let is_valid_binary = command == expected_binary_path || command.ends_with("pmat");
    assert!(
        is_valid_binary,
        "Example command doesn't use the expected binary: {command} (expected {expected_binary_path} or ending with pmat)"
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
        "refactor",
        "quality-gate",
        "diagnose",
        "report",
        "enforce",
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
    let Some(content) = cli_reference() else {
        return Default::default();
    };
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
    assert!(
        json_block.trim().starts_with('['),
        "Invalid JSON example in documentation: {json_block}"
    );

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
    let Some(content) = cli_reference() else {
        return Default::default();
    };

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
                yaml_block.contains("pmat") || yaml_block.contains("cargo test"),
                "GitHub Actions example should reference the tool or tests"
            );
        }
    }
}

#[test]
fn test_jsonc_examples_are_valid() {
    let Some(content) = cli_reference() else {
        return Default::default();
    };

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
                            obj["command"].as_str() == Some("pmat"),
                            "VS Code task should use pmat command"
                        );
                    }
                }
            }
            Err(e) => {
                // JSONC might have trailing commas or other relaxed syntax
                // Just ensure it's not completely broken
                assert!(
                    jsonc_block.contains("pmat"),
                    "JSONC example should reference the tool. Parse error: {e}"
                );
            }
        }
    }
}

#[test]
fn test_template_uri_examples_are_valid() {
    let Some(content) = cli_reference() else {
        return Default::default();
    };

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
    let Some(content) = cli_reference() else {
        return Default::default();
    };

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
