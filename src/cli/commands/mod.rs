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
pub mod mcp_commands;
pub mod org_prompt;
pub use mcp_commands::*;
pub use org_prompt::*;

// Work commands
pub mod work_commands;
pub use work_commands::*;
