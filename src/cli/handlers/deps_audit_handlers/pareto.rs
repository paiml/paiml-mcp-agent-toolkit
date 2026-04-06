#![cfg_attr(coverage_nightly, coverage(off))]

use std::collections::HashSet;
use std::path::Path;

use super::types::{DepAnalysis, DepCategory, ParetoEffort, ParetoEntry};

/// Calculate effort to remove a dependency based on its usage
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn estimate_effort(name: &str, category: DepCategory) -> ParetoEffort {
    debug_assert!(!name.is_empty(), "name must not be empty");
    // High effort: deeply integrated deps
    let high_effort = ["tokio", "serde", "clap", "anyhow", "thiserror", "tracing"];
    if high_effort.contains(&name) {
        return ParetoEffort::High;
    }

    // Medium effort: used in multiple places but replaceable
    let medium_effort = [
        "git2",
        "octocrab",
        "reqwest",
        "swc_ecma_parser",
        "swc_common",
        "swc_ecma_ast",
        "swc_ecma_visit",
        "rusqlite",
        "pest",
        "pest_derive",
    ];
    if medium_effort.contains(&name) {
        return ParetoEffort::Medium;
    }

    // Low effort if removable category or simple utility
    if matches!(category, DepCategory::Removable | DepCategory::DevOnly) {
        return ParetoEffort::Low;
    }

    // Default based on category
    match category {
        DepCategory::Heavy => ParetoEffort::Medium,
        DepCategory::Replaceable => ParetoEffort::Medium,
        _ => ParetoEffort::High,
    }
}

/// Run Pareto analysis using cargo tree for accurate transitive counts
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn run_pareto_analysis(deps: &[DepAnalysis], path: &Path) -> Vec<ParetoEntry> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let mut entries = Vec::new();

    // Only analyze removable, heavy, and replaceable deps
    let candidates: Vec<_> = deps
        .iter()
        .filter(|d| {
            matches!(
                d.category,
                DepCategory::Removable | DepCategory::Heavy | DepCategory::Replaceable
            )
        })
        .collect();

    for dep in candidates {
        // Get actual transitive count from cargo tree
        let transitive = get_transitive_count(&dep.name, path);

        let effort = estimate_effort(&dep.name, dep.category);
        let roi = transitive as f32 / effort.multiplier();

        entries.push(ParetoEntry {
            name: dep.name.clone(),
            transitive_deps: transitive,
            effort,
            roi,
            reason: dep.reason.clone(),
            category: dep.category,
        });
    }

    // Sort by ROI (highest first)
    entries.sort_by(|a, b| {
        b.roi
            .partial_cmp(&a.roi)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    entries
}

/// Get transitive dependency count using cargo tree
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn get_transitive_count(dep_name: &str, path: &Path) -> usize {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use std::process::Command;

    let output = Command::new("cargo")
        .args(["tree", "-p", dep_name, "--prefix", "none", "-e", "no-dev"])
        .current_dir(path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Count unique lines (each is a transitive dep)
            let count: HashSet<_> = stdout.lines().collect();
            count.len().saturating_sub(1) // Don't count self
        }
        _ => 0,
    }
}
