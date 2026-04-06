#![cfg_attr(coverage_nightly, coverage(off))]
//! Clippy integration, lint parsing, and diagnostic processing

use super::metrics::build_lint_hotspot_result;
use super::types::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

// --- Extracted function bodies ---
include!("clippy_parsing.rs");
include!("clippy_file_analysis.rs");

/// Run clippy and analyze the JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn run_clippy_analysis(
    project_path: &Path,
    clippy_flags: &str,
) -> Result<LintHotspotResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let flags: Vec<&str> = clippy_flags.split_whitespace().collect();
    let output = execute_clippy_command(project_path, &flags).await?;

    check_clippy_output(&output)?;

    let mut file_metrics = parse_clippy_json_output(&output)?;

    let workspace_root = find_workspace_root(project_path)?;
    calculate_sloc_for_files(&mut file_metrics, project_path, workspace_root.as_ref()).await?;

    build_lint_hotspot_result(file_metrics)
}

/// Run clippy on a single file and analyze the JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn run_clippy_analysis_single_file(
    project_path: &Path,
    file_path: &Path,
    clippy_flags: &str,
) -> Result<LintHotspotResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    let output = run_clippy_command(project_path, clippy_flags).await?;
    let abs_file_path = resolve_absolute_path(project_path, file_path);

    let (file_violations, all_violations, severity_dist) =
        parse_clippy_output(&output.stdout, &abs_file_path, file_path)?;

    let sloc = count_source_lines(project_path, file_path)
        .await
        .unwrap_or(100);

    create_single_file_result(
        file_path,
        file_violations,
        all_violations,
        severity_dist,
        sloc,
    )
}
