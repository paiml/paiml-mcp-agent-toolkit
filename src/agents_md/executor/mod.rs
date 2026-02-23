#![cfg_attr(coverage_nightly, coverage(off))]
//! Command Executor for AGENTS.md
//!
//! Safely executes commands from AGENTS.md with quality gate enforcement.

mod execution;
mod quality_types;
mod safety;
mod types;

// Re-export all public types so external code sees the same API
pub use quality_types::*;
pub use types::*;

// Import parent types needed by submodules
use super::{Command, PathBuf};

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests;
