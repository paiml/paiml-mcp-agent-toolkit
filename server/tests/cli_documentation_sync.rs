use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DocumentedCommand {
    name: String,
    description: String,
    subcommands: Vec<String>,
    arguments: Vec<String>,
    options: Vec<String>,
}

fn parse_documented_cli_commands() -> Vec<DocumentedCommand> {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/todo/active/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    let mut commands = Vec::new();

    // Split by command sections
    let sections: Vec<&str> = content.split("### Command: `").collect();

    // Pre-compile regex patterns outside the loop
    let arg_regex = Regex::new(r"`<([^>]+)>`").unwrap();
    let opt_regex = Regex::new(r"`(-[a-z], )?--([a-z-]+)`").unwrap();
    let subcommand_regex = Regex::new(r"### Command: `analyze ([^`]+)`").unwrap();

    for (i, section) in sections.iter().enumerate() {
        if i == 0 {
            continue; // Skip the first split (before any command)
        }

        // Extract command name
        let name = if let Some(end) = section.find('`') {
            section[..end].to_string()
        } else {
            continue;
        };

        // Extract description (first non-empty line after header)
        let description = section
            .lines()
            .skip(1)
            .find(|line| !line.trim().is_empty())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // Extract arguments from #### Arguments section
        let mut arguments = Vec::new();
        if let Some(args_section) = section.split("#### Arguments").nth(1) {
            if let Some(args_content) = args_section.split("####").next() {
                for arg_cap in arg_regex.captures_iter(args_content) {
                    arguments.push(arg_cap[1].to_string());
                }
            }
        }

        // Extract options from #### Options section
        let mut options = Vec::new();
        if let Some(opts_section) = section.split("#### Options").nth(1) {
            if let Some(opts_content) = opts_section.split("####").next() {
                for opt_cap in opt_regex.captures_iter(opts_content) {
                    options.push(format!("--{}", &opt_cap[2]));
                }
            }
        }

        // Extract subcommands (for commands like "analyze")
        let mut subcommands = Vec::new();
        if name == "analyze" {
            for sub_cap in subcommand_regex.captures_iter(&content) {
                subcommands.push(sub_cap[1].to_string());
            }
        }

        commands.push(DocumentedCommand {
            name,
            description,
            subcommands,
            arguments,
            options,
        });
    }

    commands
}

fn parse_cli_help_output(output: &[u8]) -> Vec<String> {
    let output_str = String::from_utf8_lossy(output);
    let mut commands = Vec::new();

    // Look for commands in the help output
    let command_regex = Regex::new(r"^\s{2,}(\w+)\s+").unwrap();
    let mut in_commands_section = false;

    for line in output_str.lines() {
        if line.contains("Commands:") || line.contains("SUBCOMMANDS:") {
            in_commands_section = true;
            continue;
        }

        if in_commands_section && line.trim().is_empty() {
            break;
        }

        if in_commands_section {
            if let Some(cap) = command_regex.captures(line) {
                commands.push(cap[1].to_string());
            }
        }
    }

    commands
}

fn get_binary_path() -> String {
    // Try to find the binary in the target directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap();

    // Check release build first, then debug - look for pmat binary
    let release_binary = workspace_root.join("target/release/pmat");
    let debug_binary = workspace_root.join("target/debug/pmat");

    if release_binary.exists() {
        release_binary.to_string_lossy().to_string()
    } else if debug_binary.exists() {
        debug_binary.to_string_lossy().to_string()
    } else {
        // Fall back to system binary
        "pmat".to_string()
    }
}

#[test]
fn test_cli_commands_match_documentation() {
    // Parse documented commands from docs/cli-mcp.md
    let documented_commands = parse_documented_cli_commands();
    assert!(
        !documented_commands.is_empty(),
        "No commands found in documentation"
    );

    // Get actual commands from CLI
    let binary_path = get_binary_path();
    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success(), "CLI --help command failed");

    let actual_commands = parse_cli_help_output(&output.stdout);
    assert!(
        !actual_commands.is_empty(),
        "No commands found in CLI help output"
    );

    // Compare main commands
    for doc_cmd in &documented_commands {
        if doc_cmd.name.contains(' ') {
            // Skip subcommands for now, they'll be checked separately
            continue;
        }

        assert!(
            actual_commands.contains(&doc_cmd.name),
            "Documented command '{}' not found in CLI. Available commands: {:?}",
            doc_cmd.name,
            actual_commands
        );
    }
}

#[test]
fn test_cli_subcommands_match_documentation() {
    let documented_commands = parse_documented_cli_commands();
    let binary_path = get_binary_path();

    // Check subcommands for commands that have them
    for doc_cmd in &documented_commands {
        if doc_cmd.subcommands.is_empty() {
            continue;
        }

        // Get help for the parent command
        let output = Command::new(&binary_path)
            .args([&doc_cmd.name, "--help"])
            .output()
            .expect("Failed to run CLI subcommand help");

        if output.status.success() {
            let actual_subcommands = parse_cli_help_output(&output.stdout);

            for subcmd in &doc_cmd.subcommands {
                assert!(
                    actual_subcommands.contains(subcmd),
                    "Documented subcommand '{} {}' not found in CLI",
                    doc_cmd.name,
                    subcmd
                );
            }
        }
    }
}

#[test]
fn test_cli_options_match_documentation() {
    let documented_commands = parse_documented_cli_commands();
    let binary_path = get_binary_path();

    for doc_cmd in &documented_commands {
        // Get help for each command
        let args = if doc_cmd.name.contains(' ') {
            // Handle subcommands like "analyze complexity"
            let parts: Vec<&str> = doc_cmd.name.split(' ').collect();
            vec![parts[0], parts[1], "--help"]
        } else {
            vec![&doc_cmd.name[..], "--help"]
        };

        let output = Command::new(&binary_path).args(&args).output();

        if let Ok(output) = output {
            if output.status.success() {
                let help_text = String::from_utf8_lossy(&output.stdout);

                // Check that documented options exist in help text
                for option in &doc_cmd.options {
                    assert!(
                        help_text.contains(option),
                        "Documented option '{}' for command '{}' not found in help text",
                        option,
                        doc_cmd.name
                    );
                }
            }
        }
    }
}

#[test]
fn test_no_undocumented_commands() {
    let documented_commands = parse_documented_cli_commands();
    let binary_path = get_binary_path();

    // Get actual commands from CLI
    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .expect("Failed to run CLI");

    let actual_commands = parse_cli_help_output(&output.stdout);
    let documented_names: Vec<String> = documented_commands
        .iter()
        .filter(|cmd| !cmd.name.contains(' '))
        .map(|cmd| cmd.name.clone())
        .collect();

    // Check for undocumented commands
    for actual_cmd in &actual_commands {
        // Special case: "analyze" is documented as subcommands, not standalone
        if actual_cmd == "analyze" {
            let has_analyze_subcommands = documented_commands
                .iter()
                .any(|cmd| cmd.name.starts_with("analyze "));
            assert!(
                has_analyze_subcommands,
                "Command 'analyze' exists in CLI but has no subcommands documented"
            );
            continue;
        }

        // Special case: "help" is a standard CLI command that doesn't need documentation
        if actual_cmd == "help" {
            continue;
        }

        assert!(
            documented_names.contains(actual_cmd),
            "Command '{actual_cmd}' exists in CLI but is not documented in cli-mcp.md"
        );
    }
}

#[test]
fn test_documentation_examples_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/todo/active/cli-mcp.md");

    let content = fs::read_to_string(&doc_path).expect("Failed to read cli-mcp.md");

    // Extract bash code blocks - use a simpler approach
    let mut in_bash_block = false;
    let mut current_block = String::new();
    let mut bash_blocks = Vec::new();

    for line in content.lines() {
        if line == "```bash" {
            in_bash_block = true;
            current_block.clear();
        } else if line == "```" && in_bash_block {
            in_bash_block = false;
            if !current_block.is_empty() {
                bash_blocks.push(current_block.clone());
            }
        } else if in_bash_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }
    let binary_path = get_binary_path();

    for code_block in bash_blocks {
        // Skip comments and complex examples
        if code_block.starts_with('#') || code_block.contains('|') || code_block.contains('$') {
            continue;
        }

        // Extract the command (first line if multi-line)
        let command_line = code_block.lines().next().unwrap_or("");

        // Skip if it's not a paiml-mcp-agent-toolkit or pmat command
        if !command_line.contains("paiml-mcp-agent-toolkit") && !command_line.contains("pmat") {
            continue;
        }

        // Skip commands with environment variables
        if command_line.contains("RUST_LOG=") || command_line.contains("MCP_VERSION=") {
            continue;
        }

        // Replace the binary name with our test binary path (handle both old and new names)
        let test_command = command_line
            .replace("paiml-mcp-agent-toolkit", &binary_path)
            .replace("pmat", &binary_path);

        // For commands with line continuations, just test the first line with --help
        let test_args: Vec<&str> = if test_command.contains('\\') {
            let base_cmd = test_command.split('\\').next().unwrap().trim();
            let mut parts: Vec<&str> = base_cmd.split_whitespace().collect();
            parts.push("--help");
            parts
        } else {
            test_command.split_whitespace().collect()
        };

        if test_args.len() > 1 {
            // Test that the command structure is valid by running with --help
            let mut cmd_args = test_args[1..].to_vec();

            // If the command doesn't already have --help, add it
            if !cmd_args.contains(&"--help") {
                // Find the subcommand position to insert --help
                let subcommand_pos = cmd_args
                    .iter()
                    .position(|arg| !arg.starts_with('-'))
                    .map_or(cmd_args.len(), |pos| pos + 1);

                cmd_args.insert(subcommand_pos.min(cmd_args.len()), "--help");
            }

            let output = Command::new(test_args[0]).args(&cmd_args).output();

            // We expect the command to at least be recognized (even if it shows help)
            assert!(
                output.is_ok(),
                "Example command failed to execute: {command_line}"
            );
        }
    }
}
