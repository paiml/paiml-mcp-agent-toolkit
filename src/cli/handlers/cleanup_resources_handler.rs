#![cfg_attr(coverage_nightly, coverage(off))]
// Cleanup resources CLI handler (GH-86)
// Toyota Way: Muda elimination - remove waste from development environments

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::cli::OutputFormat;

/// Cleanup target types
#[derive(Debug, Clone, PartialEq)]
pub enum CleanupTarget {
    Rust,
    Docker,
    Node,
    Git,
    Logs,
    Caches,
    All,
}

/// The targets `scan_targets` actually dispatches a scanner for.
///
/// `docker` and `caches` are declared in the enum and were accepted by
/// `parse`, but no scanner exists for either — not even under `all`. So
/// `--targets docker` printed a header, a blank line where the scan phase
/// belongs, "Items found: 0" and exited 0, which is indistinguishable from a
/// machine with nothing to clean. Until `cleanup_scanners.rs` grows a
/// `scan_docker_targets`/`scan_cache_targets`, the names are rejected rather
/// than silently no-op'd.
pub const SCANNABLE_TARGETS: &[&str] = &["rust", "node", "git", "logs", "all"];

impl CleanupTarget {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Parse the input.
    ///
    /// Returns `None` for `docker` and `caches`: see [`SCANNABLE_TARGETS`].
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" => Some(Self::Rust),
            "node" => Some(Self::Node),
            "git" => Some(Self::Git),
            "logs" => Some(Self::Logs),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}

/// Cleanup candidate found during scan
#[derive(Debug, Clone)]
pub struct CleanupCandidate {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub category: String,
    pub description: String,
    pub age_days: u32,
}

/// Cleanup result summary
#[derive(Debug, Default)]
pub struct CleanupResult {
    pub candidates: Vec<CleanupCandidate>,
    pub total_size_bytes: u64,
    pub items_found: usize,
    pub items_cleaned: usize,
    pub space_freed_bytes: u64,
    pub errors: Vec<String>,
}

/// Handle the `pmat maintain cleanup-resources` command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_cleanup_resources(
    project_dir: &Path,
    targets: &[String],
    execute: bool,
    exclude: &[String],
    min_age_days: u32,
    format: OutputFormat,
) -> Result<()> {
    // A typo'd --project-dir used to be indistinguishable from a genuinely
    // clean project: the walkers simply yielded nothing, so the command printed
    // "Found 0 Rust target directories" and "Items found: 0" and exited 0 —
    // for a directory that does not exist. The same flag drives the destructive
    // --execute mode, and the handler already carries a `path_exists` contract
    // that nothing enforced at runtime.
    if !project_dir.exists() {
        anyhow::bail!("Cleanup path does not exist: {}", project_dir.display());
    }
    if !project_dir.is_dir() {
        anyhow::bail!("Cleanup path is not a directory: {}", project_dir.display());
    }

    // Parse targets
    let parsed_targets: Vec<CleanupTarget> = targets
        .iter()
        .filter_map(|t| CleanupTarget::parse(t))
        .collect();

    if parsed_targets.is_empty() {
        println!("⚠️  No valid cleanup targets specified");
        println!("   Valid targets: {}", SCANNABLE_TARGETS.join(", "));
        return Ok(());
    }

    let has_all = parsed_targets.contains(&CleanupTarget::All);

    crate::status_println!("🧹 PMAT Resource Cleanup");
    crate::status_println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    crate::status_println!("📁 Scanning: {}", project_dir.display());
    crate::status_println!("🎯 Targets: {:?}", targets);
    crate::status_println!("⚡ Mode: {}", if execute { "EXECUTE" } else { "DRY-RUN" });
    crate::status_println!();

    let mut result = CleanupResult::default();

    scan_targets(
        project_dir,
        &parsed_targets,
        has_all,
        exclude,
        min_age_days,
        &mut result,
    )?;
    print_results(&result, format)?;
    finalize_cleanup(execute, &mut result)
}

fn scan_targets(
    project_dir: &Path,
    targets: &[CleanupTarget],
    has_all: bool,
    exclude: &[String],
    min_age_days: u32,
    result: &mut CleanupResult,
) -> Result<()> {
    if has_all || targets.contains(&CleanupTarget::Rust) {
        scan_rust_targets(project_dir, exclude, min_age_days, result)?;
    }
    if has_all || targets.contains(&CleanupTarget::Node) {
        scan_node_targets(project_dir, exclude, min_age_days, result)?;
    }
    if has_all || targets.contains(&CleanupTarget::Git) {
        scan_git_targets(project_dir, result)?;
    }
    if has_all || targets.contains(&CleanupTarget::Logs) {
        scan_log_targets(project_dir, exclude, min_age_days, result)?;
    }
    Ok(())
}

fn finalize_cleanup(execute: bool, result: &mut CleanupResult) -> Result<()> {
    if execute && !result.candidates.is_empty() {
        crate::status_println!();
        crate::status_println!("🔥 Executing cleanup...");
        execute_cleanup(result)?;
        println!();
        println!(
            "✅ Cleaned {} items, freed {} MB",
            result.items_cleaned,
            result.space_freed_bytes / (1024 * 1024)
        );
    } else if !execute && !result.candidates.is_empty() {
        println!();
        println!("💡 Run with --execute to perform cleanup");
    }
    Ok(())
}

// Scanner functions: scan_rust_targets, scan_node_targets, scan_git_targets,
// scan_log_targets, is_old_enough, print_results
include!("cleanup_scanners.rs");

// Execution and helper functions: execute_cleanup, cleanup_directory,
// cleanup_git, cleanup_file, is_hidden, is_excluded, calculate_dir_size,
// count_loose_objects
include!("cleanup_execution.rs");

// Tests module
include!("cleanup_tests.rs");
