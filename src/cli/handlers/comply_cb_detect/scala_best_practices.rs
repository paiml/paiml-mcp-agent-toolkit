#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-800 Series: Scala Best Practices Detection
//!
//! Pattern-based Scala defect detection for `pmat comply check`.
//! Based on: Odersky et al. (2004), Scalastyle, WartRemover,
//! Scala style guide conventions.

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip when walking for Scala files.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".claude",
    "node_modules",
    "target",
    ".pmat",
    "vendor",
    "build",
    "dist",
    ".bsp",
    ".metals",
    ".bloop",
    ".idea",
    "project/target",
];

// =============================================================================
// File walking
// =============================================================================

/// Walk directory recursively for `.scala` and `.sc` files.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn walkdir_scala_files(dir: &Path) -> Vec<PathBuf> {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let mut files = Vec::new();
    walk_scala_recursive(dir, &mut files);
    files
}

fn walk_scala_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_scala_recursive(&path, files);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "scala" | "sc"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

/// Check if a Scala file is a test file.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn is_scala_test_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.ends_with("Test")
        || stem.ends_with("Spec")
        || stem.ends_with("Suite")
        || stem.starts_with("Test")
    {
        return true;
    }
    path.components().any(|c| {
        let s = c.as_os_str().to_str().unwrap_or("");
        s == "test" || s == "tests" || s == "it" || s == "spec"
    })
}

/// Compute production lines (strip Scala comments).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn compute_scala_production_lines(content: &str) -> Vec<(usize, String)> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut result = Vec::new();
    let mut in_block_comment = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }

        // Strip inline comments
        let line_content = if let Some(pos) = trimmed.find("//") {
            // Avoid stripping URLs (http://) and string literals
            if pos > 0 && &trimmed[pos - 1..pos] == ":" {
                trimmed
            } else {
                trimmed[..pos].trim()
            }
        } else {
            trimmed
        };

        if !line_content.is_empty() {
            result.push((i + 1, line_content.to_string()));
        }
    }

    result
}

// =============================================================================
// CB-800 through CB-802: Mutable collections, null usage, wildcard imports
// =============================================================================
include!("scala_bp_mutable_null_wildcard.rs");

// =============================================================================
// CB-803 through CB-805: Return statements, var declarations, blocking futures
// =============================================================================
include!("scala_bp_return_var_future.rs");
