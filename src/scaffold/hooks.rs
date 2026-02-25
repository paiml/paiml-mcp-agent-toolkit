#![cfg_attr(coverage_nightly, coverage(off))]
//! Pre-commit and post-commit hook generation and installation for scaffolded projects.
//!
//! # TICKET-PMAT-5005: Git Initialization and Pre-commit Hooks
//! # TICKET-PMAT-5013: Auto-update Hooks (Post-commit)
//!
//! This module generates and installs hooks that enforce quality gates
//! and automatically update roadmaps on every commit. Supports both pforge and WASM project types.
//!
//! # Module Organization
//! - `hooks_generation.rs`: Hook script generation (pre-commit, post-commit)
//! - `hooks_installation.rs`: Hook installation to .git/hooks/
//! - `hooks_gates.rs`: Quality gate executor integration (TICKET-PMAT-5021)
//! - `hooks_tests.rs`: Unit tests and property tests

use super::config::{QualityGateConfig, TemplateType};
use super::errors::{Result, ScaffoldError};
use std::path::Path;

/// Pre-commit hook configuration
#[derive(Debug, Clone)]
pub struct HookConfig {
    /// Project type (affects which quality gates run)
    pub project_type: TemplateType,
    /// Quality gates to enforce
    pub quality_gates: QualityGateConfig,
}

// Hook script generation: pre-commit (pforge, WASM), post-commit
include!("hooks_generation.rs");

// Hook installation: pre-commit, post-commit to .git/hooks/
include!("hooks_installation.rs");

// Quality gate executor integration (TICKET-PMAT-5021)
include!("hooks_gates.rs");

// Unit tests and property tests
include!("hooks_tests.rs");
