use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DocumentedCommand {
    name: String,
    description: String,
    subcommands: Vec<String>,
    arguments: Vec<String>,
    options: Vec<String>,
}

/// Where the CLI reference lives.
///
/// #1228: this was `Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()`.
/// `CARGO_MANIFEST_DIR` is ALREADY the repository root, so `.parent()` pointed at
/// `<repo>/../rust-docs/cli-reference.md`, which does not exist. Every test in this
/// file then took its "file not found" branch, returned empty, and reported ok:
/// five green tests asserting nothing, in 0.00s.
fn cli_reference_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("rust-docs/cli-reference.md")
}

/// Read the CLI reference, or fail loudly.
///
/// A missing input is RED, never green. The whole point of this suite is to notice
/// when the CLI and its reference drift apart; if it cannot read the reference it
/// has measured nothing, and saying so is the only honest outcome.
fn read_cli_reference(doc_path: &Path) -> String {
    fs::read_to_string(doc_path).unwrap_or_else(|e| {
        panic!(
            "cli-reference.md not found at {} ({e}).\n\
             This suite compares the CLI against that document; without it nothing \
             is measured. It must not be skipped — see #1228, where the same read \
             failed silently and five tests reported ok.",
            doc_path.display()
        )
    })
}

/// Parse `rust-docs/cli-reference.md` into the commands it documents.
///
/// #1228 defect 3: this used to split on `"### Command: `"`, a heading shape that
/// occurs ZERO times in the document. The parser therefore returned an empty Vec
/// on a perfectly readable file, and all four callers took an
/// `if documented_commands.is_empty() { return; }` branch and reported ok. That is
/// a third, independent way this suite was vacuous — fixing only the two `.parent()`
/// defects left all five tests still green in 0.00s, which is how it was found.
///
/// The document's real shape, measured:
///   `### \`demo\``              — a top-level command, under `## Commands`
///   `##### \`analyze churn\``   — a subcommand
///   `**Arguments:**`           — an argument block (not `#### Arguments`)
fn parse_documented_cli_commands() -> Vec<DocumentedCommand> {
    let doc_path = cli_reference_path();
    let content = read_cli_reference(&doc_path);

    let top_level = Regex::new(r"(?m)^### `([a-z][a-z0-9-]*)`").expect("static regex must compile");
    let sub_level = Regex::new(r"(?m)^##### `([a-z][a-z0-9-]* [a-z][a-z0-9-]*)`")
        .expect("static regex must compile");
    let arg_regex = Regex::new(r"`<([^>]+)>`").expect("static regex must compile");
    let opt_regex =
        Regex::new(r"`(?:-[a-zA-Z], )?--([a-z][a-z0-9-]*)`").expect("static regex must compile");

    let subcommands_by_parent: Vec<String> = sub_level
        .captures_iter(&content)
        .map(|c| c[1].to_string())
        .collect();

    let mut commands = Vec::new();
    let heads: Vec<(usize, String)> = top_level
        .captures_iter(&content)
        .map(|c| {
            let m = c.get(0).expect("group 0 always exists");
            (m.start(), c[1].to_string())
        })
        .collect();

    for (i, (offset, name)) in heads.iter().enumerate() {
        let section_end = heads.get(i + 1).map_or(content.len(), |(next, _)| *next);
        let section = &content[*offset..section_end];

        let description = section
            .lines()
            .skip(1)
            .find(|line| !line.trim().is_empty())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let arguments = section
            .split("**Arguments:**")
            .nth(1)
            .and_then(|rest| rest.split("**").next())
            .map(|block| {
                arg_regex
                    .captures_iter(block)
                    .map(|c| c[1].to_string())
                    .collect()
            })
            .unwrap_or_default();

        let options = section
            .split("**Options:**")
            .nth(1)
            .and_then(|rest| rest.split("**").next())
            .map(|block| {
                opt_regex
                    .captures_iter(block)
                    .map(|c| format!("--{}", &c[1]))
                    .collect()
            })
            .unwrap_or_default();

        let subcommands = subcommands_by_parent
            .iter()
            .filter_map(|full| full.strip_prefix(&format!("{name} ")))
            .map(str::to_string)
            .collect();

        commands.push(DocumentedCommand {
            name: name.clone(),
            description,
            subcommands,
            arguments,
            options,
        });
    }

    assert!(
        !commands.is_empty(),
        "parsed 0 commands out of {} ({} bytes). The document is readable, so either \
         its heading shape changed or this parser is wrong — either way this suite \
         measures nothing and must not report ok (#1228).",
        doc_path.display(),
        content.len()
    );

    commands
}

fn parse_cli_help_output(output: &[u8]) -> Vec<String> {
    let output_str = String::from_utf8_lossy(output);
    let mut commands = Vec::new();

    // #1228 defect 4: this was `^\s{2,}(\w+)\s+`. `\w` is [A-Za-z0-9_] and does
    // NOT match `-`, and the trailing `\s+` then required whitespace immediately
    // after the captured word — so every hyphenated command failed the match
    // outright and was dropped. 23 of pmat's 71 top-level commands are hyphenated
    // (quality-gates, dead-code, deep-context, five-whys, ...), so the CLI side of
    // this comparison was missing a third of its input.
    let command_regex =
        Regex::new(r"^\s{2,}([a-z][a-z0-9-]*)\s{2,}").expect("static regex must compile");
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

/// The binary this test grades.
///
/// #1228: this used to be `Path::new(manifest_dir).parent().unwrap()`, which is
/// one level ABOVE the repository, so neither candidate could ever exist and the
/// function fell through to the string `"pmat"` — whatever happened to be on
/// `PATH`, typically a `cargo install`ed copy from some earlier release. A doc
/// test that silently grades a different binary than the one just built is worse
/// than no test, so there is no fallback now: if the build is missing, say so.
fn get_binary_path() -> String {
    // CARGO_MANIFEST_DIR is the repository root; the previous `.parent()` climbed
    // out of it. Cargo's own `CARGO_BIN_EXE_pmat` is not available here because
    // this file is compiled into the `all` integration target, not a bin target.
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let release_binary = repo_root.join("target/release/pmat");
    let debug_binary = repo_root.join("target/debug/pmat");

    if release_binary.exists() {
        release_binary.to_string_lossy().to_string()
    } else if debug_binary.exists() {
        debug_binary.to_string_lossy().to_string()
    } else {
        panic!(
            "no pmat binary to grade the documentation against.\n  tried: {}\n  tried: {}\n\
             Build one first: cargo build --release --bin pmat\n\
             (this used to fall back to `pmat` on PATH, which graded a stale \
             install against current docs — #1228)",
            release_binary.display(),
            debug_binary.display()
        )
    }
}

#[test]
fn test_cli_commands_match_documentation() {
    // Parse documented commands from docs/cli-mcp.md
    let documented_commands = parse_documented_cli_commands();
    assert!(
        !documented_commands.is_empty(),
        "no documented commands parsed — this test would otherwise report ok \
         while comparing nothing (#1228)"
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
    assert!(
        !documented_commands.is_empty(),
        "no documented commands parsed — this test would otherwise report ok \
         while comparing nothing (#1228)"
    );
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
    assert!(
        !documented_commands.is_empty(),
        "no documented commands parsed — this test would otherwise report ok \
         while comparing nothing (#1228)"
    );
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

/// Top-level commands that `rust-docs/cli-reference.md` does not document yet.
///
/// A RATCHET, not an allow-list: this may only ever get SHORTER. It exists because
/// the honest measurement is 60 undocumented commands out of 71, and a test that
/// demanded all 60 be written today would simply be disabled tomorrow — which is
/// how this suite came to assert nothing in the first place (#1228).
///
/// Adding a command without documenting it fails the test below. Documenting one
/// without deleting its line here ALSO fails, so the list cannot rot into a
/// permanent excuse. Emptying it is the goal; #1228 Step 3 (generate the reference
/// from the command registry) is how that is meant to happen.
const UNDOCUMENTED_AT_BASELINE: &[&str] = &[
    "agent",
    "agy",
    "brick-score",
    "cache",
    "ci-local",
    "comply",
    "config",
    "cuda-tdg",
    "debug",
    "demo-score",
    "deps-audit",
    "diagnose",
    "embed",
    "enforce",
    "explain",
    "extract",
    "falsify",
    "five-whys",
    "hooks",
    "infra-score",
    "init",
    "kaizen",
    "localize",
    "maintain",
    "mcp",
    "memory",
    "oracle",
    "org",
    "perfection-score",
    "popper-score",
    "predict-quality",
    "project-diag",
    "prompt",
    "qa-work",
    "qdd",
    "quality-gates",
    "query",
    "record-metric",
    "red-team",
    "report",
    "repo-score",
    "roadmap",
    "rust-project-score",
    "score",
    "semantic",
    "serve",
    "show-metrics",
    "spec",
    "split",
    "sql",
    "stack",
    "tdg",
    "telemetry",
    "test",
    "test-discovery",
    "test-stability",
    "validate-docs",
    "validate-readme",
    "verify",
    "work",
];

#[test]
fn test_no_undocumented_commands() {
    let documented_commands = parse_documented_cli_commands();
    assert!(
        !documented_commands.is_empty(),
        "no documented commands parsed — this test would otherwise report ok \
         while comparing nothing (#1228)"
    );

    let binary_path = get_binary_path();
    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "CLI --help command failed");

    let actual_commands = parse_cli_help_output(&output.stdout);
    assert!(
        !actual_commands.is_empty(),
        "no commands parsed from `--help` — the CLI side of this comparison is empty"
    );

    let documented_names: Vec<&str> = documented_commands
        .iter()
        .filter(|cmd| !cmd.name.contains(' '))
        .map(|cmd| cmd.name.as_str())
        .collect();

    // `help` is clap's own; it is not part of this repo's surface.
    let undocumented_now: Vec<&str> = actual_commands
        .iter()
        .map(String::as_str)
        .filter(|c| *c != "help" && !documented_names.contains(c))
        .collect();

    let newly_undocumented: Vec<&&str> = undocumented_now
        .iter()
        .filter(|c| !UNDOCUMENTED_AT_BASELINE.contains(c))
        .collect();
    assert!(
        newly_undocumented.is_empty(),
        "{} new undocumented command(s): {:?}\n\
         Document them in rust-docs/cli-reference.md as `### `<name>``, or, if that \
         is genuinely out of scope, add them to UNDOCUMENTED_AT_BASELINE and say why \
         in the commit message. The list may only shrink.",
        newly_undocumented.len(),
        newly_undocumented
    );

    let stale: Vec<&&str> = UNDOCUMENTED_AT_BASELINE
        .iter()
        .filter(|c| !undocumented_now.contains(c))
        .collect();
    assert!(
        stale.is_empty(),
        "{} entr(y/ies) in UNDOCUMENTED_AT_BASELINE {:?} are now documented or gone \
         from the CLI. Delete them from the list — a ratchet that keeps satisfied \
         entries stops measuring anything.",
        stale.len(),
        stale
    );
}

#[test]
fn test_documentation_examples_are_valid() {
    let doc_path = cli_reference_path();
    let content = read_cli_reference(&doc_path);

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
