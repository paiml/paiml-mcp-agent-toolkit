//! The adapter must keep getting its wire names from the shared table.
//!
//! `CliInput::from_commands` used to own a private copy of the command-name
//! match. That copy lived behind `#[cfg(feature = "unified-protocol")]`, so no
//! shipped build type-checked its exhaustiveness and no gate ran the test that
//! guarded it. The table now lives in [`crate::cli::command_wire_names`], which
//! every build compiles, and this file pins the delegation: if someone
//! re-inlines a second copy here, the two can disagree and this fails.
//!
//! These assertions are the reason the delegation is safe to rely on. They run
//! only under `--features unified-protocol` — which, unlike before, is a
//! feature the `feature-tests` job in `.github/workflows/feature-matrix.yml`
//! actually executes.

use crate::cli::command_wire_names::command_name;
use crate::cli::commands::on_big_stack;
use crate::cli::Cli;
use crate::unified_protocol::adapters::cli::CliInput;

fn parse(argv: &[&str]) -> crate::cli::Commands {
    let owned: Vec<String> = std::iter::once("pmat".to_string())
        .chain(argv.iter().map(std::string::ToString::to_string))
        .collect();
    on_big_stack(move || {
        <Cli as clap::Parser>::try_parse_from(&owned)
            .map(|cli| cli.command)
            .unwrap_or_else(|e| panic!("clap must accept `pmat {}`: {e}", owned.join(" ")))
    })
}

/// A representative slice across every `CommandCategory`, plus the two
/// delegating families (`analyze`, `qdd`) whose names come from inner enums.
const SAMPLE: &[&[&str]] = &[
    &["analyze", "churn"],
    &["analyze", "reachability"],
    &["analyze", "deep-context"],
    &["analyze", "dag"],
    &["analyze", "makefile", "Makefile"],
    &["generate", "makefile", "rust/cli"],
    &["quality-gate"],
    &["verify"],
    &["serve"],
    &["refactor", "status"],
    &["list"],
    &["mcp", "manifest"],
    &["config"],
    &["demo"],
    &["enforce", "extreme"],
    &["comply", "check"],
    &["stack", "status"],
];

/// The defect this guards: two copies of the same decision that can drift.
#[test]
fn the_adapter_agrees_with_the_shared_wire_name_table() {
    for argv in SAMPLE {
        let command = parse(argv);
        let shared = command_name(&command).to_string();
        let adapter = CliInput::from_commands(command).command_name;
        assert_eq!(
            adapter,
            shared,
            "`pmat {}`: the adapter must take its wire name from \
             crate::cli::command_wire_names, not a private copy",
            argv.join(" ")
        );
    }
}

/// COUNTER-TEST. Green before the move and after it: the adapter still
/// produces real, specific names. A delegation that returned the empty string
/// for everything would satisfy the equality test above and fail here.
#[test]
fn the_adapter_still_produces_the_documented_names() {
    for (argv, expected) in &[
        (&["analyze", "churn"][..], "analyze-churn"),
        (&["quality-gate"][..], "quality-gate"),
        (&["serve"][..], "serve"),
        (&["enforce", "extreme"][..], "enforce"),
    ] {
        assert_eq!(
            CliInput::from_commands(parse(argv)).command_name,
            *expected,
            "`pmat {}` must keep its wire name",
            argv.join(" ")
        );
    }
}
