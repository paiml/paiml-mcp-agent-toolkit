//! Automation Layer: Conservative Automation
//!
//! Phase 4 Implementation (Months 10-12)
//! Safe, deterministic automation for simple fixes

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use crate::unified_quality::metrics::{Violation, ViolationType};

// Type definitions: structs, enums, type aliases, Default impl
include!("automation_types.rs");

// impl ConservativeAutomator
include!("automation_automator.rs");

// impl GitSafetyNet + impl RollbackManager
include!("automation_git.rs");


#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // Core automation tests: creation, auto_fix, rollback manager
    include!("automation_tests_core.rs");

    // Serialization, Debug, Clone trait tests and batch/integration tests
    include!("automation_tests_serialization.rs");
}
