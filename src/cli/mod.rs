#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI module for the paiml-mcp-agent-toolkit
//!
//! This module implements the command-line interface using a modular architecture
//! to reduce complexity and improve testability.

pub mod analysis;
pub mod analysis_helpers;
pub mod analysis_utilities;
pub mod args;
pub mod colors;
pub mod command_dispatcher;
pub mod command_structure;
pub mod commands;
pub mod coverage_helpers;
pub mod dead_code_formatter;
pub mod defect_formatter;
pub mod defect_helpers;
pub mod defect_prediction_helpers;
pub mod diagnose;
pub mod drift_detector;
pub mod enums;
pub mod error_context;
pub mod formatting_helpers;
pub mod handlers;
pub mod help_generator;
pub mod language_analyzer;
pub mod mcp_schema_generator;
pub mod name_similarity_helpers;
pub mod output;
pub mod progress;
pub mod proof_annotation_formatter;
pub mod proof_annotation_helpers;
pub mod provability_helpers;
pub mod registry;
pub mod semantic_commands;
pub mod symbol_table_helpers;
pub mod tdg_helpers;
pub mod unified_help;
pub mod verify;

// Re-export commonly used types from submodules
pub use commands::{
    AgentCommands, AnalyzeCommands, Cli, Commands, EnforceCommands, Mode, RefactorCommands,
};
pub use enums::*;
pub use handlers::get_timeline_help_text; // Sprint 78: TUI-006
pub use help_generator::HelpGenerator; // Issue #118
pub use mcp_schema_generator::McpSchemaGenerator; // Issue #118
pub use registry::{
    ArgumentMetadata, CommandMetadata, CommandRegistry, ExampleMetadata, McpToolMetadata,
}; // Issue #118
pub use unified_help::{HelpResponse, HelpSearchResult, UnifiedHelpService}; // Issue #118

use crate::stateless_server::StatelessTemplateServer;
use command_dispatcher::CommandDispatcher;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

// Type definitions for handler compatibility
#[derive(Debug, Clone)]
/// Information about name.
pub struct NameInfo {
    pub name: String,
    pub kind: String,
    pub file_path: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
/// Result of name similarity operation.
pub struct NameSimilarityResult {
    pub name: String,
    pub kind: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub similarity: f32,
    pub phonetic_match: bool,
    pub fuzzy_match: bool,
}

#[derive(Debug, Clone)]
/// Configuration for duplicate handler.
pub struct DuplicateHandlerConfig {
    pub project_path: PathBuf,
    pub detection_type: DuplicateType,
    pub threshold: f32,
    pub min_lines: usize,
    pub max_tokens: usize,
    pub format: DuplicateOutputFormat,
    pub perf: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub output: Option<PathBuf>,
}

/// Early CLI args struct for tracing initialization
#[derive(Debug, Clone)]
pub struct EarlyCliArgs {
    pub verbose: bool,
    pub debug: bool,
    pub trace: bool,
    pub trace_filter: Option<String>,
    pub is_mcp_server: bool,
}

/// Parse CLI early to extract tracing configuration
///
/// # Examples
///
/// ```rust,ignore
/// use pmat::cli::parse_early_for_tracing;
///
/// // This function reads from std::env::args() and RUST_LOG
/// let args = parse_early_for_tracing();
///
/// // The function always returns valid EarlyCliArgs
/// // Values depend on actual command line arguments
/// ```ignore
#[must_use]
#[cfg_attr(coverage_nightly, coverage(off))]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn parse_early_for_tracing() -> EarlyCliArgs {
    let args: Vec<String> = std::env::args().collect();

    let verbose = args.iter().any(|arg| arg == "-v" || arg == "--verbose");
    let debug = args.iter().any(|arg| arg == "--debug");
    let trace = args.iter().any(|arg| arg == "--trace");

    // Check if this is an MCP server command
    let is_mcp_server = args.len() >= 3 && args[1] == "agent" && args[2] == "mcp-server";

    let trace_filter = args
        .iter()
        .position(|arg| arg == "--trace-filter")
        .and_then(|pos| args.get(pos + 1))
        .cloned()
        .or_else(|| std::env::var("RUST_LOG").ok());

    EarlyCliArgs {
        verbose,
        debug,
        trace,
        trace_filter,
        is_mcp_server,
    }
}

/// Write the process's final diagnostic for a fatal error to `w`.
///
/// Lives in the library, not in `src/bin/pmat.rs`, so that the unit tests below
/// run under the `--lib` suite CI actually executes.
///
/// # Why this is not `tracing::error!`
///
/// It used to be. But [`EarlyCliArgs::is_mcp_server`] causes the binary to
/// install `EnvFilter::new("off")`, which discards every event including the
/// fatal one. `pmat agent mcp-server` therefore exited 1 with both stdout and
/// stderr completely empty, even though the command had already produced an
/// accurate message ("Agent daemon feature not enabled. Build with --features
/// agent-daemon"). The diagnostic existed; the log filter ate it.
///
/// A process's last words must not depend on log configuration. Writing to
/// stderr is safe even under MCP, where only stdout carries JSON-RPC frames.
pub fn write_fatal_error<W: std::io::Write>(mut w: W, error: &anyhow::Error) {
    // `{:#}` renders the full anyhow context chain on one line.
    let _ = writeln!(w, "Error: {error:#}");
}

/// Fail unless `path` exists and can actually be read, before any analysis
/// reports a result for it.
///
/// Analysis subcommands walk a directory tree; a tree that is not there yields
/// zero files, which several handlers then reported as a clean bill of health —
/// `analyze satd --path /nope` printed "Found 0 SATD violations in 0 files" and
/// exited 0, and `analyze duplicates` and `analyze big-o` did the same. A CI
/// gate cannot tell that apart from a genuinely clean tree, so a typo in a path
/// silently turned the gate green.
///
/// The same hole exists one step further in: a directory that exists but cannot
/// be opened (`chmod 000`) also walks to zero files. GH-682 observed
/// `analyze complexity -p <chmod 000 dir>` exit 0 with only "⚠️  Warning: No
/// files were found or analyzed", over a tree containing a `Cargo.toml` and a
/// `src/main.rs` it analyses fine once permissions are restored — while
/// `analyze satd` on the identical directory exited 1 with "Permission denied".
/// Listing the directory is the first thing every walker does, so a failure
/// here is exactly the failure the walk would otherwise swallow.
///
/// Several handlers already carry a `path_exists` contract annotation; this is
/// the runtime check that makes the annotation true.
///
/// # Errors
///
/// - `Path not found: <path>` if `path` does not exist, matching the wording the
///   other analysis handlers already use.
/// - `Path not readable: <path>: <io error>` if `path` is a directory whose
///   entries cannot be listed.
pub fn ensure_analysis_path_exists(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }
    if path.is_dir() {
        // Carried as context over the io::Error rather than one flat string: the
        // binary's `categorize_error` keys the exit code off `Error::to_string()`
        // (the outermost message only), so an inlined "Permission denied" would
        // have turned this into exit 126 while `analyze satd` — whose io error is
        // likewise wrapped — exits 1. `{:#}` still renders the whole chain.
        if let Err(e) = std::fs::read_dir(path) {
            return Err(
                anyhow::Error::new(e).context(format!("Path not readable: {}", path.display()))
            );
        }
    }
    Ok(())
}

// Core CLI execution: run(), apply_ux_settings(), parse_with_suggestions()
include!("cli_run_command.rs");

// Language detection helpers for project primary language
include!("cli_language_detection.rs");

// Deep context config, SATD filtering, analysis type parsing, DAG/cache conversion
include!("cli_deep_context.rs");

// Handler stubs for backward-compatible analysis commands
include!("cli_handler_stubs.rs");

// Import tests in test configuration
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod language_detection_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod analysis_utilities_property_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod analysis_path_guard_tests {
    use super::ensure_analysis_path_exists;
    use std::path::Path;

    #[test]
    fn rejects_a_path_that_does_not_exist() {
        let err = ensure_analysis_path_exists(Path::new("/definitely-not-a-real-path-9f3a"))
            .expect_err("a missing path must not be reported as analysable");
        let msg = err.to_string();
        assert!(msg.contains("Path not found"), "got: {msg}");
        assert!(
            msg.contains("/definitely-not-a-real-path-9f3a"),
            "must name the offending path so a typo is obvious, got: {msg}"
        );
    }

    #[test]
    fn accepts_an_existing_directory() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        assert!(ensure_analysis_path_exists(dir.path()).is_ok());
    }

    #[test]
    fn accepts_an_existing_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let file = dir.path().join("a.rs");
        std::fs::write(&file, "fn main() {}").expect("write");
        // `analyze satd --path` accepts a single file, not only a directory.
        assert!(ensure_analysis_path_exists(&file).is_ok());
    }

    /// GH-682: a `chmod 000` directory exists, so the existence check passed and
    /// `analyze complexity` reported a clean pass over content it was denied
    /// access to (exit 0), while `analyze satd` exited 1 on the same directory.
    #[cfg(unix)]
    #[test]
    fn rejects_a_directory_it_cannot_read() {
        use std::os::unix::fs::PermissionsExt;

        let parent = tempfile::TempDir::new().expect("tempdir");
        let locked = parent.path().join("noread");
        std::fs::create_dir(&locked).expect("create dir");
        std::fs::write(locked.join("main.rs"), "fn main() {}").expect("write");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
            .expect("chmod 000");

        // Running as root defeats permission bits entirely; detect that by
        // observation rather than asserting something untrue of the environment.
        let permissions_bite = std::fs::read_dir(&locked).is_err();
        let result = ensure_analysis_path_exists(&locked);

        // Restore before asserting so the TempDir can always clean itself up.
        let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));

        if !permissions_bite {
            return;
        }

        let err = result.expect_err("an unreadable directory must not be reported as analysable");
        let msg = err.to_string();
        assert!(msg.contains("Path not readable"), "got: {msg}");
        assert!(
            format!("{err:#}").contains("Permission denied"),
            "the underlying cause must survive in the chain, got: {err:#}"
        );
        // The binary categorises the exit code from `to_string()` alone; keeping
        // "permission" out of the outermost message is what makes this exit 1
        // (matching `analyze satd`) instead of 126.
        assert!(
            !msg.to_lowercase().contains("permission"),
            "outermost message must not trip the 126 classifier, got: {msg}"
        );
        assert!(
            msg.contains("noread"),
            "must name the offending path, got: {msg}"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod fatal_error_tests {
    use super::write_fatal_error;

    fn rendered(error: &anyhow::Error) -> String {
        let mut buf = Vec::new();
        write_fatal_error(&mut buf, error);
        String::from_utf8(buf).expect("diagnostic must be UTF-8")
    }

    #[test]
    fn writes_the_error_message() {
        let out = rendered(&anyhow::anyhow!(
            "Agent daemon feature not enabled. Build with --features agent-daemon"
        ));
        assert!(
            out.contains("Agent daemon feature not enabled"),
            "fatal diagnostic must carry the message, got: {out:?}"
        );
        assert!(out.starts_with("Error: "), "got: {out:?}");
        assert!(
            out.ends_with('\n'),
            "must be newline-terminated, got: {out:?}"
        );
    }

    #[test]
    fn includes_the_full_context_chain() {
        // `{:#}` not `{}` — a bare `{}` would print only "outer" and drop the
        // root cause, which is usually the part that says what to actually do.
        let error = anyhow::anyhow!("root cause").context("outer");
        let out = rendered(&error);
        assert!(out.contains("outer"), "got: {out:?}");
        assert!(
            out.contains("root cause"),
            "must render the whole anyhow context chain, got: {out:?}"
        );
    }

    #[test]
    fn never_writes_an_empty_diagnostic() {
        // The defect this function exists to prevent: exiting non-zero having
        // printed nothing, because a log filter discarded the only message.
        for message in ["x", "", "  "] {
            let out = rendered(&anyhow::anyhow!(message.to_string()));
            assert!(
                !out.trim().is_empty(),
                "fatal path must never produce an empty diagnostic, got: {out:?}"
            );
        }
    }
}
