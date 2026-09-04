//! Hooks command handlers for pre-commit hook management
//!
//! Following TDD approach for Sprint 80: Pre-commit Hook Management as Core Feature
//! Implements dynamic hook management as specified in:
//! docs/specifications/pre-commit-hooks-spec.md

#![cfg_attr(coverage_nightly, coverage(off))]

mod cache_handlers;
mod command_dispatch;
mod commit_msg_hook;
mod hook_generation;
mod hooks_command;
mod interactive_setup;
pub(crate) mod tdg_hooks;
mod types;

pub use command_dispatch::handle_hooks_command;
#[allow(unused_imports)]
pub(crate) use command_dispatch::{
    print_installed_status, print_verification_fixes, print_verification_issues,
};
pub use hooks_command::HooksCommand;
#[allow(unused_imports)]
pub(crate) use tdg_hooks::{
    install_tdg_hooks, install_tdg_post_commit_hook, install_tdg_pre_commit_hook,
};
pub use types::{
    HookInstallResult, HookRefreshResult, HookRunResult, HookStatus, HookUninstallResult,
    HookVerificationResult,
};

// Tests extracted to hooks_command_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "../hooks_command_handlers_tests.rs"]
mod tests;

// GH-301 regression tests live in a dedicated small file so future growth of
// hooks_command_handlers_tests.rs (already near the CB-040 ratchet) does not
// block the fix from landing.
#[cfg(test)]
#[path = "../hooks_tests_gh301_makefile_scan.rs"]
mod gh301_makefile_scan_tests;

#[cfg(test)]
mod commit_enforcement_tests;
