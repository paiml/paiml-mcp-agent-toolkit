#![cfg_attr(coverage_nightly, coverage(off))]
//! Help Generator - Dynamic --help text from CommandRegistry
//!
//! This module generates accurate help text from the single source of truth
//! (CommandRegistry), ensuring documentation is never out of sync with implementation.
//!
//! # Architecture (Toyota Way - Genchi Genbutsu)
//!
//! ```text
//! CommandRegistry -> HelpGenerator -> Formatted Help Text
//!                                      |-- Terminal output
//!                                      |-- Man pages
//!                                      +-- Markdown docs
//! ```
//!
//! # Module Layout
//!
//! - `help_generator_formatting.rs` - HelpGenerator impl (constructor, generation, formatting)
//! - `help_generator_utils.rs` - Free utility functions (levenshtein, truncate_str)
//! - `help_generator_tests.rs` - Unit tests
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118

// `std::io::IsTerminal` was imported for a private `--color auto` that ignored
// `--color never`/`--color always`; `crate::cli::colors::colors_enabled()` owns
// that decision now, is_terminal fallback included.
use crate::cli::registry::{
    ArgumentMetadata, CommandMetadata, CommandRegistry, ExecutionTime, ValueType,
};

/// Generates formatted help text from CommandRegistry.
pub struct HelpGenerator {
    registry: CommandRegistry,
    color: bool,
    width: usize,
}

// --- Include files ---

include!("help_generator_formatting.rs");
include!("help_generator_utils.rs");
include!("help_generator_tests.rs");
