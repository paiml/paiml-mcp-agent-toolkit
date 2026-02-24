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

impl CleanupTarget {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" => Some(Self::Rust),
            "docker" => Some(Self::Docker),
            "node" => Some(Self::Node),
            "git" => Some(Self::Git),
            "logs" => Some(Self::Logs),
            "caches" => Some(Self::Caches),
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
pub async fn handle_cleanup_resources(
    project_dir: &Path,
    targets: &[String],
    execute: bool,
    exclude: &[String],
    min_age_days: u32,
    format: OutputFormat,
) -> Result<()> {
    // Parse targets
    let parsed_targets: Vec<CleanupTarget> = targets
        .iter()
        .filter_map(|t| CleanupTarget::parse(t))
        .collect();

    if parsed_targets.is_empty() {
        println!("⚠️  No valid cleanup targets specified");
        println!("   Valid targets: rust, docker, node, git, logs, caches, all");
        return Ok(());
    }

    let has_all = parsed_targets.contains(&CleanupTarget::All);

    println!("🧹 PMAT Resource Cleanup");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📁 Scanning: {}", project_dir.display());
    println!("🎯 Targets: {:?}", targets);
    println!("⚡ Mode: {}", if execute { "EXECUTE" } else { "DRY-RUN" });
    println!();

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
        println!();
        println!("🔥 Executing cleanup...");
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
