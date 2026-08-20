//! Every subcommand `pmat` advertises must have a wire name.
//!
//! # Why this test lives here
//!
//! It used to sit at `src/unified_protocol/adapters/cli/`, behind
//! `#[cfg(feature = "unified-protocol")]` — a feature in neither `default` nor
//! `full`, and which no CI job ran the tests of. `cargo test --lib -- --list`
//! returned **zero** matches for `command_name_totality`, so this guard against
//! a variant reaching `unreachable!()` had never once executed in a gate. It
//! now sits next to [`crate::cli::command_wire_names`], the default-features
//! module that holds the table, and runs in the ordinary test suite.
//!
//! `CliInput::from_commands` used to decide the name in two steps: an
//! exhaustive categoriser (`Commands` -> `CommandCategory`), then one name
//! extractor per category, each ending in `_ => unreachable!(..)`. Only the
//! categoriser was checked by the compiler, so a variant that was categorised
//! but never named compiled cleanly and aborted the process the first time it
//! was asked for its name.
//!
//! That is not hypothetical: `Mcp` and `Agy` were categorised `System` with no
//! entry in the `System` namer, and `Reachability`, `HardcodedPaths` and
//! `VacuousTests` were categorised `Basic` with no entry in the `Basic` namer.
//! Both were repaired by hand, one variant at a time, which is the repair that
//! does not scale — 41 of the 70 top-level subcommands and 12 of the 36
//! `analyze` subcommands were still unnamed after those two fixes.
//!
//! The fixture below is the full advertised command list, parsed through clap
//! exactly as a user's argv would be, so a subcommand cannot be in `--help` and
//! absent from this test at the same time.

use crate::cli::command_wire_names::command_name;
use crate::cli::commands::on_big_stack;
use crate::cli::Commands;

/// Parse a real argv into the `Commands` value the binary would dispatch on.
fn try_parse(argv: &[&str]) -> Result<Commands, String> {
    let owned: Vec<String> = std::iter::once("pmat".to_string())
        .chain(argv.iter().map(std::string::ToString::to_string))
        .collect();
    on_big_stack(move || {
        <crate::cli::Cli as clap::Parser>::try_parse_from(&owned)
            .map(|cli| cli.command)
            .map_err(|e| e.to_string())
    })
}

fn parse(argv: &[&str]) -> Commands {
    try_parse(argv).unwrap_or_else(|e| panic!("clap must accept `pmat {}`: {e}", argv.join(" ")))
}

/// Fixture integrity: every argv below is a real invocation. If clap stops
/// accepting one, the two tests that matter would "pass" by never reaching the
/// name lookup, so this failure has to be loud and separate.
#[test]
fn every_fixture_argv_parses() {
    let broken: Vec<String> = ALREADY_NAMED
        .iter()
        .chain(WAS_UNREACHABLE.iter())
        .filter_map(|(argv, _)| {
            try_parse(argv).err().map(|e| {
                format!(
                    "pmat {} => {}",
                    argv.join(" "),
                    e.lines().next().unwrap_or("")
                )
            })
        })
        .collect();
    assert!(
        broken.is_empty(),
        "unparseable fixture argv:\n{}",
        broken.join("\n")
    );
}

/// The wire name the crate gives an argv, or a panic if it has none.
fn wire_name(argv: &[&str]) -> String {
    command_name(&parse(argv)).to_string()
}

/// Names that the pre-fix code already produced. These must keep working
/// unchanged: a "fix" that renamed or refused the commands that used to work
/// would be caught here, not celebrated.
const ALREADY_NAMED: &[(&[&str], &str)] = &[
    (&["analyze", "churn"], "analyze-churn"),
    (&["analyze", "complexity"], "analyze-complexity"),
    (&["analyze", "dead-code"], "analyze-dead-code"),
    (&["analyze", "satd"], "analyze-satd"),
    (&["analyze", "tdg"], "analyze-tdg"),
    (&["analyze", "lint-hotspot"], "analyze-lint-hotspot"),
    (&["analyze", "reachability"], "analyze-reachability"),
    (&["analyze", "hardcoded-paths"], "analyze-hardcoded-paths"),
    (&["analyze", "vacuous-tests"], "analyze-vacuous-tests"),
    (&["analyze", "deep-context"], "analyze-deep-context"),
    (&["analyze", "comprehensive"], "analyze-comprehensive"),
    (
        &["analyze", "defect-prediction"],
        "analyze-defect-prediction",
    ),
    (&["analyze", "duplicates"], "analyze-duplicates"),
    (&["analyze", "big-o"], "analyze-big-o"),
    (&["analyze", "dag"], "analyze-dag"),
    (&["analyze", "graph-metrics"], "analyze-graph-metrics"),
    (&["analyze", "symbol-table"], "analyze-symbol-table"),
    (
        &["analyze", "name-similarity", "foo"],
        "analyze-name-similarity",
    ),
    (&["analyze", "makefile", "Makefile"], "analyze-makefile"),
    (&["analyze", "provability"], "analyze-provability"),
    (
        &["analyze", "proof-annotations"],
        "analyze-proof-annotations",
    ),
    (
        &["analyze", "incremental-coverage"],
        "analyze-incremental-coverage",
    ),
    (&["analyze", "assembly-script"], "analyze-assemblyscript"),
    (&["analyze", "web-assembly"], "analyze-webassembly"),
    (&["generate", "makefile", "rust/cli"], "generate"),
    (
        &["scaffold", "agent", "--name", "a", "--template", "x"],
        "scaffold",
    ),
    (&["quality-gate"], "quality-gate"),
    (&["report"], "report"),
    (&["five-whys", "boom"], "five-whys"),
    (&["serve"], "serve"),
    (&["cache", "stats"], "cache"),
    (&["memory", "stats"], "memory"),
    (&["telemetry"], "telemetry"),
    (&["refactor", "status"], "refactor"),
    (&["test"], "test"),
    (
        &["roadmap", "init", "--version", "1", "--title", "t"],
        "roadmap",
    ),
    (&["validate", "template://x"], "validate"),
    (&["maintain", "health"], "maintain"),
    (&["hooks", "status"], "hooks"),
    (&["work", "list"], "work"),
    (&["list"], "list"),
    (&["search", "foo"], "search"),
    (&["context"], "context"),
    (&["diagnose"], "diagnose"),
    (&["debug", "replay", "rec.pmat"], "debug"),
    (&["init"], "init"),
    (&["mcp", "manifest"], "mcp"),
    (&["agy", "sync"], "agy"),
    (&["config"], "config"),
    (&["agent", "status"], "agent"),
    (&["tdg"], "tdg"),
    (&["demo"], "demo"),
    (&["enforce", "extreme"], "enforce"),
];

/// Subcommands the split design categorised but never named. Every one of
/// these aborted the process with `unreachable!(..)` before the classifier was
/// fused into a single total match.
const WAS_UNREACHABLE: &[(&[&str], &str)] = &[
    // `analyze` family
    (&["analyze", "bottleneck"], "analyze-bottleneck"),
    (&["analyze", "defects"], "analyze-defects"),
    (&["analyze", "build-tdg"], "analyze-build-tdg"),
    (&["analyze", "clippy"], "analyze-clippy"),
    (&["analyze", "entropy"], "analyze-entropy"),
    (&["analyze", "coverage-improve"], "analyze-coverage-improve"),
    (&["analyze", "wasm", "mod.wasm"], "analyze-wasm"),
    (
        &["analyze", "cluster", "--method", "kmeans"],
        "analyze-cluster",
    ),
    (
        &["analyze", "topics", "--num-topics", "3"],
        "analyze-topics",
    ),
    (&["analyze", "models"], "analyze-models"),
    // top level
    (&["query", "cache"], "query"),
    (&["verify"], "verify"),
    (&["explain", "satd"], "explain"),
    (&["comply", "check"], "comply"),
    (&["quality-gates", "show"], "quality-gates"),
    (&["score"], "score"),
    (&["repo-score"], "repo-score"),
    (&["rust-project-score"], "rust-project-score"),
    (&["popper-score"], "popper-score"),
    (&["demo-score"], "demo-score"),
    (&["brick-score"], "brick-score"),
    (&["infra-score"], "infra-score"),
    (&["perfection-score"], "perfection-score"),
    (&["deps-audit"], "deps-audit"),
    (&["validate-docs"], "validate-docs"),
    (
        &[
            "validate-readme",
            "--targets",
            "README.md",
            "--deep-context",
            "ctx.md",
        ],
        "validate-readme",
    ),
    (&["red-team", "analyze", "--message", "m"], "red-team"),
    (
        &[
            "org",
            "localize",
            "--passed-coverage",
            "a",
            "--failed-coverage",
            "b",
            "--passed-count",
            "1",
            "--failed-count",
            "1",
        ],
        "org",
    ),
    (&["prompt", "show"], "prompt"),
    (&["embed", "status"], "embed"),
    (&["semantic", "search", "foo"], "semantic"),
    (&["show-metrics"], "show-metrics"),
    (&["predict-quality"], "predict-quality"),
    (&["record-metric", "lint", "1"], "record-metric"),
    (&["project-diag"], "project-diag"),
    (&["test-discovery", "run"], "test-discovery"),
    (&["test-stability"], "test-stability"),
    (
        &[
            "localize",
            "--passed-coverage",
            "a",
            "--failed-coverage",
            "b",
            "--passed-count",
            "1",
            "--failed-count",
            "1",
        ],
        "localize",
    ),
    (&["cuda-tdg", "score"], "cuda-tdg"),
    (&["falsify", "spec.md"], "falsify"),
    (&["sql", "select 1"], "sql"),
    (&["oracle", "status"], "oracle"),
    (&["qa-work", "validate", "T-1"], "qa-work"),
    (&["spec", "list"], "spec"),
    (&["kaizen"], "kaizen"),
    (&["extract", "--list", "a.rs"], "extract"),
    (&["split", "a.rs"], "split"),
    (&["ci-local"], "ci-local"),
    (&["stack", "status"], "stack"),
];

/// COUNTER-TEST. Green before the fix and green after it: the commands that
/// already had names keep exactly those names. A "fix" that made
/// `from_commands` return a placeholder for everything, or that renamed the
/// working commands while adding the missing ones, fails here.
#[test]
fn commands_that_already_had_names_keep_them() {
    for (argv, expected) in ALREADY_NAMED {
        assert_eq!(
            wire_name(argv),
            *expected,
            "`pmat {}` must keep its wire name",
            argv.join(" ")
        );
    }
}

/// The defect: these subcommands are advertised by `--help`, parse fine, and
/// then hit `unreachable!(..)` when asked for a wire name.
#[test]
fn every_advertised_subcommand_has_a_wire_name() {
    for (argv, expected) in WAS_UNREACHABLE {
        assert_eq!(
            wire_name(argv),
            *expected,
            "`pmat {}` must have a wire name",
            argv.join(" ")
        );
    }
}

/// Names must be unique per subcommand: the fix must not paper over the gap by
/// giving several commands the same string.
#[test]
fn wire_names_are_distinct_per_subcommand() {
    let mut seen: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (argv, _) in ALREADY_NAMED.iter().chain(WAS_UNREACHABLE.iter()) {
        seen.entry(wire_name(argv))
            .or_default()
            .push(argv.join(" "));
    }
    let collisions: Vec<_> = seen.iter().filter(|(_, v)| v.len() > 1).collect();
    assert!(
        collisions.is_empty(),
        "one wire name per subcommand; collisions: {collisions:?}"
    );
}

/// The categorisation half of the same decision, pinned for the three variants
/// whose miscategorisation started this: they are `Basic`, and the `Basic`
/// namer is now the same match arm, so the two can no longer disagree.
#[test]
fn category_and_name_come_from_the_same_arm() {
    use crate::cli::command_wire_names::{classify_analyze_command, AnalyzeCommandCategory};
    for (argv, expected_name) in &[
        (&["analyze", "reachability"][..], "analyze-reachability"),
        (
            &["analyze", "hardcoded-paths"][..],
            "analyze-hardcoded-paths",
        ),
        (&["analyze", "vacuous-tests"][..], "analyze-vacuous-tests"),
    ] {
        let Commands::Analyze(analyze_cmd) = parse(argv) else {
            panic!("`pmat {}` must parse as an analyze command", argv.join(" "))
        };
        let (category, name) = classify_analyze_command(&analyze_cmd);
        assert!(
            matches!(category, AnalyzeCommandCategory::Basic),
            "`pmat {}` is Basic, got {category:?}",
            argv.join(" ")
        );
        assert_eq!(name, *expected_name);
    }
}
