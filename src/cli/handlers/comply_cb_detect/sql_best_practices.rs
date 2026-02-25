#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-700 Series: SQL Best Practices Detection
//!
//! Pattern-based SQL defect detection for `pmat comply check`.
//! Based on: Chamberlin & Boyce (1974), SQL anti-pattern literature,
//! OWASP SQL injection guidelines.
//!
//! ## Module Layout
//!
//! - `sql_file_walking.rs` — directory walking, test-file detection, SQL line parsing
//! - `sql_violation_detectors.rs` — CB-700 through CB-705 violation detectors

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip when walking for SQL files.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".claude",
    "node_modules",
    "target",
    ".pmat",
    "vendor",
    "build",
    "dist",
    "migrations_backup",
    "__pycache__",
];

/// SQL keywords that start statements (not assignments).
const SQL_KEYWORDS: &[&str] = &[
    "select", "insert", "update", "delete", "create", "alter", "drop", "grant", "revoke", "begin",
    "commit", "rollback", "explain", "with", "merge", "truncate", "set", "declare", "exec",
    "execute",
];

// =============================================================================
// Included submodules
// =============================================================================

include!("sql_file_walking.rs");
include!("sql_violation_detectors.rs");
