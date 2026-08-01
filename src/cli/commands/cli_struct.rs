#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI top-level struct, Mode and ColorMode enums

use clap::Parser;

use super::commands_enum::Commands;

/// Main CLI structure
#[derive(Parser)]
#[command(
    name = "pmat",
    about = "PMAT - Professional Multi-language Analysis Toolkit for code quality, complexity, and technical debt",
    version,
    // `--version` reports the revision the binary was actually built from.
    //
    // A version number repeats across builds, so it cannot distinguish a fresh
    // binary from a stale one. v3.28.2 shipped a headline fix that did not work
    // because three measurements were taken against binaries that did not match
    // the tree under test. This makes that mismatch checkable.
    //
    // The plain version stays the first token, so existing assertions of the form
    // "`--version` output contains <the version number>" keep working, and `-V`
    // remains a single line for packagers and scripts that parse it.
    long_version = concat!(
        env!("CARGO_PKG_VERSION"),
        "\ncommit: ",
        env!("PMAT_GIT_SHA"),
        "\nworktree: ",
        env!("PMAT_GIT_DIRTY"),
    ),
    long_about = None,
    after_help = "EXAMPLES:
# Analyze code complexity
pmat analyze complexity --path .

# Calculate Technical Debt Grade (TDG)
pmat tdg .

# Find self-admitted technical debt markers
pmat analyze satd --path .

# Find dead code
pmat analyze dead-code --path .

# Generate AI-ready project context
pmat context --format llm-optimized

# Run quality gates
pmat quality-gate

# Calculate repository health score
pmat repo-score

# Start background agent daemon
pmat agent start"
)]
#[cfg_attr(test, derive(Debug))]
/// Cli.
pub struct Cli {
    /// Force cli or mcp mode instead of auto-detecting (`--mode mcp` starts the
    /// MCP stdio server, the same one `MCP_VERSION=1 pmat` starts)
    //
    // Read from raw argv by `cli::forced_mode_from_args` *before* clap parses,
    // because `pmat --mode mcp` carries no subcommand and clap requires one.
    // Until then nothing read this flag at all: it was advertised here, and the
    // only code that looked at it started a different, legacy MCP server.
    #[arg(long, value_enum, global = true)]
    pub mode: Option<Mode>,

    /// Enable verbose output (info level)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable quiet mode (errors only)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Enable debug output (debug level)
    #[arg(long, global = true)]
    pub debug: bool,

    /// Enable trace output (trace level)
    #[arg(long, global = true)]
    pub trace: bool,

    /// Custom trace filter (overrides other flags)
    /// Example: --trace-filter="paiml=debug,cache=trace"
    #[arg(long, global = true, env = "RUST_LOG")]
    pub trace_filter: Option<String>,

    /// Control color output
    #[arg(long, global = true, value_enum, default_value = "auto")]
    pub color: ColorMode,

    #[command(subcommand)]
    pub command: Commands,
}

/// CLI execution mode
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum Mode {
    Cli,
    Mcp,
}

/// Color output mode
#[derive(Clone, Debug, clap::ValueEnum, PartialEq, Default)]
pub enum ColorMode {
    /// Auto-detect based on TTY and environment
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

/// Guards against documentation drifting away from behaviour.
///
/// These live here, not in `src/tests/clap_command_structure_tests.rs`, because
/// that file is reachable only through the `tests/all.rs` integration target —
/// CI runs `cargo test --lib`, so a guard placed there would never run. Both
/// build pmat's clap tree on an 8 MiB thread; the 2 MiB default test stack is
/// not enough for it.
#[cfg(test)]
mod help_text_guards {
    /// Run `body` on a stack big enough for `Cli::command()`.
    fn on_a_big_stack<F: FnOnce() + Send + 'static>(body: F) {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(body)
            .expect("spawn")
            .join()
            .expect("the clap tree must build");
    }

    /// `--help` advertised `--high-risk-only  Show only high-risk files
    /// (probability > 0.7)` while the band it selects begins at 0.6, so the flag
    /// returned files at 0.6069833 and 0.657 — both below the documented cut.
    /// The help text must quote the constant the code actually uses.
    #[test]
    fn high_risk_only_help_names_the_threshold_the_code_uses() {
        on_a_big_stack(|| {
            use crate::services::facades::defect_prediction_facade::HIGH_RISK_PROBABILITY;
            use clap::CommandFactory;

            let cmd = crate::cli::Cli::command();
            let analyze = cmd.find_subcommand("analyze").expect("`analyze`");
            let defect_prediction = analyze
                .find_subcommand("defect-prediction")
                .expect("`analyze defect-prediction`");
            let arg = defect_prediction
                .get_arguments()
                .find(|a| a.get_id() == "high_risk_only")
                .expect("--high-risk-only");
            let help = arg
                .get_help()
                .expect("--high-risk-only must be documented")
                .to_string();

            assert!(
                help.contains(&HIGH_RISK_PROBABILITY.to_string()),
                "help says {help:?} but the band starts at {HIGH_RISK_PROBABILITY}"
            );
            assert!(
                !help.contains("0.7"),
                "0.7 is not a boundary of any band this flag selects: {help:?}"
            );
        });
    }

    /// `--mode` is advertised as a global flag but was read nowhere;
    /// `cli::forced_mode_from_args` now reads it from raw argv before clap runs,
    /// so the two must agree on the spelling and the accepted values.
    #[test]
    fn the_mode_flag_clap_accepts_is_the_one_argv_scanning_recognises() {
        on_a_big_stack(|| {
            use crate::cli::commands::Mode;
            use crate::cli::forced_mode_from_args;
            use clap::Parser;

            for (args, expected) in [
                (vec!["pmat", "--mode", "mcp", "list"], Some(Mode::Mcp)),
                (vec!["pmat", "--mode=cli", "list"], Some(Mode::Cli)),
                (vec!["pmat", "list"], None),
            ] {
                let parsed = crate::cli::Cli::try_parse_from(&args).expect("clap accepts these");
                let owned: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
                assert_eq!(
                    parsed.mode, expected,
                    "clap and forced_mode_from_args must agree on {args:?}"
                );
                assert_eq!(forced_mode_from_args(&owned), expected, "{args:?}");
            }
        });
    }
}
