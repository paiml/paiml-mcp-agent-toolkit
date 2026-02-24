#![cfg_attr(coverage_nightly, coverage(off))]
//! GitHub Actions integration for unified quality system
//!
//! Provides quality gates and automation through GitHub Actions workflows
//!
//! ## Module structure
//!
//! This module is split into include files for maintainability:
//! - `github_actions_types.rs` — Config structs, result types, enums, and Default impls
//! - `github_actions_integration.rs` — `impl GitHubActionsIntegration` methods
//! - `github_actions_tests.rs` — Unit tests

use crate::unified_quality::enforcement::{Decision, DiffAnalysis, ErrorBudgetEnforcer};
use crate::unified_quality::foundation::QualityMonitor;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// GitHub Actions integration for quality enforcement
pub struct GitHubActionsIntegration {
    /// Quality monitor
    monitor: QualityMonitor,

    /// Error budget enforcer
    enforcer: ErrorBudgetEnforcer,

    /// Integration configuration
    config: GitHubConfig,
}

// --- Type definitions (configs, results, enums) ---
include!("github_actions_types.rs");

// --- impl GitHubActionsIntegration ---
include!("github_actions_integration.rs");

// --- Tests ---
include!("github_actions_tests.rs");
