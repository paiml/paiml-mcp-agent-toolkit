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
fn parse_documented_cli_commands() -> Option<Vec<DocumentedCommand>> {
    // `None` only when the document was never shipped — see cli_reference().
    let content = cli_reference()?;

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
        "parsed 0 commands out of {} bytes of cli-reference.md. The document is \
         readable, so either its heading shape changed or this parser is wrong — \
         either way this suite measures nothing and must not report ok (#1228).",
        content.len()
    );

    Some(commands)
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

/// A pmat command that does not inherit the ambient environment.
///
/// #1228 got this twice wrong before landing here. It first hand-built
/// `<repo>/target/{release,debug}/pmat`, rooted one level ABOVE the repository,
/// so neither candidate existed and it fell through to the literal "pmat" on
/// PATH — a `cargo install`ed copy from some older release, graded against
/// current docs. Rooting it correctly still ignored `CARGO_TARGET_DIR`, which
/// this repo redirects, so it then found a STALE `./target/release/pmat`.
///
/// `pmat_cmd::pmat()` settles both: `CARGO_BIN_EXE_pmat` is the binary Cargo
/// built for THIS run, and the environment is scrubbed. That second half is not
/// incidental here — `MCP_VERSION` makes the binary ignore argv and start an MCP
/// server, and `PMAT_QUIET`/`NO_COLOR` change the bytes these tests assert on.
/// Enforced by `src/services/test_env_hygiene.rs`, which could not see these
/// three files until they started naming the binary the way Cargo does.
fn pmat_command() -> std::process::Command {
    crate::modules::pmat_cmd::pmat()
}
/// The same binary as [`pmat_command`], as a string, for the doc examples that
/// substitute it into a command line before parsing it.
fn pmat_binary_path() -> String {
    crate::modules::pmat_cmd::pmat_bin()
        .to_string_lossy()
        .to_string()
}

#[test]
fn test_cli_commands_match_documentation() {
    // Parse documented commands from docs/cli-mcp.md
    // `None` means the document was never shipped (packaged crate); there is
    // nothing to compare and saying so is the honest answer. An EMPTY parse from a
    // document that IS present panics inside parse_documented_cli_commands instead.
    let Some(documented_commands) = parse_documented_cli_commands() else {
        return;
    };

    // Get actual commands from CLI
    let output = pmat_command()
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
    // `None` means the document was never shipped (packaged crate); there is
    // nothing to compare and saying so is the honest answer. An EMPTY parse from a
    // document that IS present panics inside parse_documented_cli_commands instead.
    let Some(documented_commands) = parse_documented_cli_commands() else {
        return;
    };

    // Check subcommands for commands that have them
    for doc_cmd in &documented_commands {
        if doc_cmd.subcommands.is_empty() {
            continue;
        }

        // Get help for the parent command
        let output = pmat_command()
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
    // `None` means the document was never shipped (packaged crate); there is
    // nothing to compare and saying so is the honest answer. An EMPTY parse from a
    // document that IS present panics inside parse_documented_cli_commands instead.
    let Some(documented_commands) = parse_documented_cli_commands() else {
        return;
    };

    for doc_cmd in &documented_commands {
        // Get help for each command
        let args = if doc_cmd.name.contains(' ') {
            // Handle subcommands like "analyze complexity"
            let parts: Vec<&str> = doc_cmd.name.split(' ').collect();
            vec![parts[0], parts[1], "--help"]
        } else {
            vec![&doc_cmd.name[..], "--help"]
        };

        let output = pmat_command().args(&args).output();

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
    // `None` means the document was never shipped (packaged crate); there is
    // nothing to compare and saying so is the honest answer. An EMPTY parse from a
    // document that IS present panics inside parse_documented_cli_commands instead.
    let Some(documented_commands) = parse_documented_cli_commands() else {
        return;
    };

    let output = pmat_command()
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
    // `None` only when the document was never shipped — see cli_reference().
    let Some(content) = cli_reference() else {
        return;
    };

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
    let binary_path = pmat_binary_path();

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
