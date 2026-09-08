use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

/// Documented examples that name a flag or subcommand the CLI does not have.
///
/// A RATCHET, like `UNDOCUMENTED_AT_BASELINE`: it may only ever get SHORTER. The
/// honest measurement the moment this test could see anything at all was **16 of
/// 79** examples drifted — the extractor had been reading one line per block and
/// skipping any block that opened with a comment, which is nearly all of them.
///
/// Recorded rather than fixed here because the two are different jobs: deciding
/// what `demo --web` or `analyze defect-prediction --explain` was MEANT to say
/// needs someone who knows whether the flag was renamed, dropped, or never
/// shipped, and guessing would replace drift with fiction. A new drifted example
/// fails the test immediately; a line here that starts working ALSO fails it, so
/// the list cannot rot. Emptying it is the goal.
const DRIFTED_EXAMPLES_AT_BASELINE: &[&str] = &[
    r#"paiml-mcp-agent-toolkit demo --web --port 8080"#,
    r#"paiml-mcp-agent-toolkit demo --export markdown -o analysis.md"#,
    r#"paiml-mcp-agent-toolkit demo --export sarif -o results.sarif"#,
    r#"paiml-mcp-agent-toolkit scaffold rust \"#,
    r#"paiml-mcp-agent-toolkit context rust"#,
    r#"paiml-mcp-agent-toolkit context deno \"#,
    r#"pmat analyze duplicates --gpu --perf --format json"#,
    r#"pmat analyze defect-prediction --min-confidence 0.8"#,
    r#"pmat analyze defect-prediction --explain --format detailed"#,
    r#"pmat analyze defect-prediction --sarif -o defects.sarif"#,
    r#"pmat analyze big-o --min-complexity "O(n^2)" --format json"#,
    r#"pmat analyze makefile --min-severity error --format sarif"#,
    r#"pmat analyze incremental-coverage --min-coverage 80.0 --fail-on-decrease"#,
    r#"pmat analyze symbol-table --format ctags --include-private"#,
    r#"pmat refactor serve --resume --auto-commit "refactor: {file}""#,
    r#"pmat analyze web-assembly --include-binary --no-include-text"#,
];

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

    // #1228: this loop used to take `code_block.lines().next()` — the FIRST line
    // of each block — and `continue` on any block starting with `#`. Nearly every
    // block in the document opens with a comment, so nearly every block was
    // skipped whole, and of the survivors only one line was ever examined.
    // Measured: planting `demo-that-does-not-exist` into a real example left the
    // test green. Every line of every block is examined now.
    let mut examined = 0usize;
    let mut drifted: Vec<String> = Vec::new();
    for code_block in bash_blocks {
        for raw_line in code_block.lines() {
            let command_line = raw_line.trim();

            // Per LINE, not per block: a comment above an example must not take
            // the example down with it.
            if command_line.is_empty() || command_line.starts_with('#') {
                continue;
            }
            // Shell composition is out of scope — this checks that documented
            // pmat commands exist, not that a pipeline runs.
            if command_line.contains('|')
                || command_line.contains('$')
                || command_line.contains('>')
                || command_line.contains("&&")
            {
                continue;
            }
            if !command_line.starts_with("paiml-mcp-agent-toolkit")
                && !command_line.starts_with("pmat ")
            {
                continue;
            }

            let test_command = command_line
                .replace("paiml-mcp-agent-toolkit", &binary_path)
                .replace("pmat ", &format!("{binary_path} "));

            // A continuation line documents one command split across lines; take
            // the head and ask the CLI whether that much is a real command.
            let head = test_command.split('\\').next().unwrap_or("").trim();
            let parts: Vec<&str> = head.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let mut cmd_args: Vec<&str> = parts[1..].to_vec();

            // `--help` goes at the END. Inserting it after the first token is
            // what made this unable to fail: `pmat analyze --help <anything>`
            // prints the `analyze` help and exits 0, so a documented subcommand
            // that does not exist was never parsed. Measured:
            //   pmat analyze --help complexity-that-does-not-exist -> exit 0
            //   pmat analyze complexity-that-does-not-exist --help -> exit 2,
            //     "error: unrecognized subcommand"
            if !cmd_args.contains(&"--help") {
                cmd_args.push("--help");
            }

            let output = pmat_command()
                .args(&cmd_args)
                .output()
                .unwrap_or_else(|e| panic!("could not spawn pmat for {command_line:?}: {e}"));

            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("unrecognized subcommand") || stderr.contains("unexpected argument")
            {
                // Collected, not asserted here: a document that has drifted has
                // usually drifted in several places, and failing on the first
                // turns one fix into N runs.
                drifted.push(format!(
                    "  {command_line}\n    {}",
                    stderr.lines().next().unwrap_or("").trim()
                ));
            }
            examined += 1;
        }
    }

    // The count is the point. `0 examined` is what this test reported for its
    // whole life while printing ok, so a run that examines nothing must fail.
    assert!(
        examined > 0,
        "examined 0 documented examples — the extractor matched nothing, which is \
         how this test passed while measuring nothing (#1228)"
    );
    let new_drift: Vec<&String> = drifted
        .iter()
        .filter(|d| {
            let command = d.lines().next().unwrap_or("").trim();
            !DRIFTED_EXAMPLES_AT_BASELINE.contains(&command)
        })
        .collect();
    assert!(
        new_drift.is_empty(),
        "{} NEW drifted example(s) out of {examined} examined:\n{}\n\nFix the \
         document, or, if the example is right and the CLI is wrong, fix the CLI. \
         Adding a line to DRIFTED_EXAMPLES_AT_BASELINE needs a reason in the \
         commit message — the list may only shrink.",
        new_drift.len(),
        new_drift
            .iter()
            .map(|d| d.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );

    let fixed: Vec<&&str> = DRIFTED_EXAMPLES_AT_BASELINE
        .iter()
        .filter(|baseline| {
            !drifted
                .iter()
                .any(|d| d.lines().next().unwrap_or("").trim() == **baseline)
        })
        .collect();
    assert!(
        fixed.is_empty(),
        "{} entr(y/ies) in DRIFTED_EXAMPLES_AT_BASELINE now work: {fixed:?}\nDelete \
         them from the list — a ratchet that keeps satisfied entries stops \
         measuring anything.",
        fixed.len()
    );
}
