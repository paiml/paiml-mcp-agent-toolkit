#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI command structures
//!
//! This module contains all the command structures used by the CLI for parsing
//! and executing commands. It's separated from the main CLI module to reduce complexity.

// CLI top-level struct, Mode, ColorMode
pub mod cli_struct;
pub use cli_struct::*;

// Main Commands enum
pub mod commands_enum;
pub use commands_enum::*;

// Misc command types
pub mod misc_commands;
pub use misc_commands::*;

// Analyze commands
pub mod analyze_commands;
pub use analyze_commands::*;

// Quality commands (QDD, Enforce)
pub mod quality_commands;
pub use quality_commands::*;

// Refactor and Scaffold commands
pub mod refactor_scaffold;
pub use refactor_scaffold::*;

// Roadmap and Agent commands
pub mod roadmap_agent;
pub use roadmap_agent::*;

// Config and Hooks commands
pub mod config_hooks;
pub use config_hooks::*;

// Semantic search commands
pub mod semantic_search;
pub use semantic_search::*;

// Org and Prompt commands
pub mod agy_commands;
pub mod mcp_commands;
pub mod org_prompt;
pub use agy_commands::*;
pub use mcp_commands::*;
pub use org_prompt::*;

// Work commands
pub mod work_commands;
pub use work_commands::*;

/// Run `f` on a thread with an 8MB stack, for tests that build or parse the
/// clap command tree.
///
/// Clap's generated builder recurses deeply enough over this crate's command
/// enums to overflow the default 2MB test stack: the test binary dies with
/// `fatal runtime error: stack overflow` / SIGABRT, taking every other test
/// with it. That is invisible under `RUST_MIN_STACK=8388608`, which is how
/// `pmat verify` and the Makefile run the suite — but CI's coverage job runs a
/// bare `cargo test --lib`, so a single unwrapped `augment_subcommands` or
/// `try_parse_from` in a test turns `ci / coverage` red with an abort that
/// names no test.
///
/// Several older tests spawn this thread by hand; new ones should call this.
#[cfg(test)]
pub(crate) fn on_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(f)
        .expect("spawn 8MB-stack test thread")
        .join()
        .expect("8MB-stack test thread panicked")
}
