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
    // `--version` output contains "3.28.3" keep working. `-V` remains short.
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
    /// Force specific mode (auto-detected by default)
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
