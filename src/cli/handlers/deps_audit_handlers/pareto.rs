#![cfg_attr(coverage_nightly, coverage(off))]

use std::collections::HashSet;
use std::path::Path;

use super::types::{DepAnalysis, DepCategory, ParetoEffort, ParetoEntry};

/// Calculate effort to remove a dependency based on its usage
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn estimate_effort(name: &str, category: DepCategory) -> ParetoEffort {
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

#[cfg(test)]
mod pareto_tests {
    //! Covers estimate_effort + run_pareto_analysis filtering/sorting in
    //! deps_audit_handlers/pareto.rs (9 uncov on broad, 0% cov).
    //! Skips get_transitive_count (spawns `cargo tree`).
    use super::*;

    fn dep(name: &str, category: DepCategory, reason: &str) -> DepAnalysis {
        DepAnalysis {
            name: name.to_string(),
            version: "1.0".to_string(),
            category,
            replacement: None,
            reason: reason.to_string(),
            transitive_count: 0,
            estimated_size_kb: 0,
            pagerank_score: 0.0,
            in_degree: 0,
            out_degree: 0,
            is_bridge: false,
            is_orphan: false,
        }
    }

    // ── estimate_effort: 4 arms ──

    #[test]
    fn test_estimate_effort_high_effort_deps_return_high() {
        for name in ["tokio", "serde", "clap", "anyhow", "thiserror", "tracing"] {
            assert!(
                matches!(estimate_effort(name, DepCategory::Core), ParetoEffort::High),
                "{name} must be High"
            );
        }
    }

    #[test]
    fn test_estimate_effort_medium_effort_deps_return_medium() {
        for name in [
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
        ] {
            assert!(
                matches!(
                    estimate_effort(name, DepCategory::Core),
                    ParetoEffort::Medium
                ),
                "{name} must be Medium"
            );
        }
    }

    #[test]
    fn test_estimate_effort_removable_or_devonly_low() {
        // Unknown name + Removable/DevOnly category → Low.
        assert!(matches!(
            estimate_effort("unknown_lib", DepCategory::Removable),
            ParetoEffort::Low
        ));
        assert!(matches!(
            estimate_effort("unknown_lib", DepCategory::DevOnly),
            ParetoEffort::Low
        ));
    }

    #[test]
    fn test_estimate_effort_heavy_or_replaceable_unknown_name_medium() {
        // Unknown name + Heavy/Replaceable → Medium (default arm).
        assert!(matches!(
            estimate_effort("foo_bar", DepCategory::Heavy),
            ParetoEffort::Medium
        ));
        assert!(matches!(
            estimate_effort("foo_bar", DepCategory::Replaceable),
            ParetoEffort::Medium
        ));
    }

    #[test]
    fn test_estimate_effort_unknown_core_or_sovereign_high() {
        // Unknown name + Core/Sovereign → High (catch-all default).
        assert!(matches!(
            estimate_effort("unknown_lib", DepCategory::Core),
            ParetoEffort::High
        ));
        assert!(matches!(
            estimate_effort("unknown_lib", DepCategory::Sovereign),
            ParetoEffort::High
        ));
    }

    // ── run_pareto_analysis: filtering + sorting ──

    #[test]
    fn test_run_pareto_analysis_filters_to_removable_heavy_replaceable() {
        let deps = vec![
            dep("a", DepCategory::Core, "core"),     // filtered out
            dep("b", DepCategory::Sovereign, "sov"), // filtered out
            dep("c", DepCategory::DevOnly, "dev"),   // filtered out
            dep("d", DepCategory::Removable, "rem"),
            dep("e", DepCategory::Heavy, "heavy"),
            dep("f", DepCategory::Replaceable, "rep"),
        ];
        let entries = run_pareto_analysis(&deps, std::path::Path::new("/tmp"));
        assert_eq!(
            entries.len(),
            3,
            "must filter to Removable/Heavy/Replaceable only"
        );
        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"d"));
        assert!(names.contains(&"e"));
        assert!(names.contains(&"f"));
    }

    #[test]
    fn test_run_pareto_analysis_empty_deps_returns_empty() {
        let entries = run_pareto_analysis(&[], std::path::Path::new("/tmp"));
        assert!(entries.is_empty());
    }

    #[test]
    fn test_run_pareto_analysis_only_filtered_deps_returns_empty() {
        let deps = vec![
            dep("core", DepCategory::Core, "x"),
            dep("sov", DepCategory::Sovereign, "x"),
        ];
        let entries = run_pareto_analysis(&deps, std::path::Path::new("/tmp"));
        assert!(entries.is_empty());
    }

    // ── get_transitive_count: only the cargo-failure arm (no project at /tmp) ──

    #[test]
    fn test_get_transitive_count_cargo_failure_returns_zero() {
        // /tmp has no Cargo project → cargo tree fails → unwrap_or(0).
        let count = get_transitive_count("nonexistent_dep_xyz", std::path::Path::new("/tmp"));
        assert_eq!(count, 0);
    }
}
