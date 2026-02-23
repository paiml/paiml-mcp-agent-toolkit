//! Hooks command handlers for pre-commit hook management
//!
//! Following TDD approach for Sprint 80: Pre-commit Hook Management as Core Feature
//! Implements dynamic hook management as specified in:
//! docs/specifications/pre-commit-hooks-spec.md

#![cfg_attr(coverage_nightly, coverage(off))]

mod cache_handlers;
mod command_dispatch;
mod hook_generation;
mod hooks_command;
mod interactive_setup;
pub(crate) mod tdg_hooks;
mod types;

pub use command_dispatch::handle_hooks_command;
pub use hooks_command::HooksCommand;
pub use types::{
    HookInstallResult, HookRefreshResult, HookRunResult, HookStatus, HookUninstallResult,
    HookVerificationResult,
};

// Tests extracted to hooks_command_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "../hooks_command_handlers_tests.rs"]
mod tests;
