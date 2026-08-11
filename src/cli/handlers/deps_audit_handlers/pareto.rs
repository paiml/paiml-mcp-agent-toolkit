#![cfg_attr(coverage_nightly, coverage(off))]

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

/// Run Pareto analysis over the transitive counts already on `deps`.
///
/// These counts used to be re-derived here by spawning `cargo tree -p <dep>`
/// per candidate, which contradicted the same command's `-f json` output: that
/// path reports [`DepAnalysis::transitive_count`], computed by BFS over the
/// Cargo.lock graph. `cargo tree -p` fails for any name that is not a unique
/// package spec in the workspace (ambiguous or renamed packages), and the
/// failure arm returned **0** rather than an error — so `--pareto` printed
/// "octocrab 0 transitive deps, ROI 0.0" while `-f json` printed 250 for the
/// same dependency in the same run, and the ROI ranking that decides what to
/// remove first was inverted. One graph, one count.
///
/// `_path` is retained so the call site does not change; nothing here shells
/// out any more.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn run_pareto_analysis(deps: &[DepAnalysis], _path: &Path) -> Vec<ParetoEntry> {
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
        // Same number the JSON report prints for this dependency.
        let transitive = dep.transitive_count;

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

// `get_transitive_count` (a `cargo tree -p <dep>` spawn whose every failure mode
// returned 0) is gone: it was the second, disagreeing source of transitive
// counts. See [`run_pareto_analysis`].

#[cfg(test)]
mod pareto_tests {
    //! Covers estimate_effort + run_pareto_analysis filtering/sorting in
    //! deps_audit_handlers/pareto.rs (9 uncov on broad, 0% cov).
    use super::*;

    fn dep_with_transitive(
        name: &str,
        category: DepCategory,
        reason: &str,
        transitive_count: usize,
    ) -> DepAnalysis {
        DepAnalysis {
            transitive_count,
            ..dep(name, category, reason)
        }
    }

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

    // ── transitive counts: one graph, one number ──

    /// This replaces `test_get_transitive_count_cargo_failure_returns_zero`,
    /// which pinned the defect: it asserted that a failed `cargo tree` spawn
    /// silently yields 0 transitive deps. That zero reached the report, so
    /// `--pareto` printed "octocrab 0 / ROI 0.0" against `-f json`'s 250 for the
    /// same dependency in the same run. The Pareto table must report the counts
    /// already computed from the Cargo.lock graph, never re-derive them.
    #[test]
    fn test_run_pareto_analysis_reports_the_analysed_transitive_counts() {
        let deps = vec![
            dep_with_transitive("octocrab", DepCategory::Heavy, "GitHub API", 250),
            dep_with_transitive("reqwest", DepCategory::Heavy, "HTTP client", 192),
            dep_with_transitive("sourcemap", DepCategory::Removable, "debug maps", 91),
        ];
        let entries = run_pareto_analysis(&deps, std::path::Path::new("/nonexistent-project"));

        let by_name = |n: &str| {
            entries
                .iter()
                .find(|e| e.name == n)
                .unwrap_or_else(|| panic!("{n} missing from the Pareto table"))
        };
        assert_eq!(by_name("octocrab").transitive_deps, 250);
        assert_eq!(by_name("reqwest").transitive_deps, 192);
        assert_eq!(by_name("sourcemap").transitive_deps, 91);

        // ROI = transitive / effort multiplier, so a real count must also
        // produce a nonzero ranking signal.
        assert!(by_name("octocrab").roi > 0.0);
    }
}
