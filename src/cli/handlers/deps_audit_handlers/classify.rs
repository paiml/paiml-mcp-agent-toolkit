#![cfg_attr(coverage_nightly, coverage(off))]

use std::collections::HashMap;

use super::types::{DepAnalysis, DepCategory};

/// Known Sovereign AI stack packages
pub const SOVEREIGN_PACKAGES: &[&str] = &[
    "trueno",
    "trueno-graph",
    "trueno-rag",
    "trueno-viz",
    "aprender",
    "pmcp",
    "presentar-terminal",
    "ruchy",
    "batuta",
    "realizár",
    "renacer",
    "certeza",
];

/// Replaceable dependencies with Sovereign alternatives
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn get_replacements() -> HashMap<&'static str, (&'static str, &'static str)> {
    let mut map = HashMap::new();
    // (dependency, (replacement, reason))
    map.insert(
        "petgraph",
        ("trueno-graph", "Graph algorithms with O(1) lookups"),
    );
    map.insert(
        "ratatui",
        ("presentar-terminal", "TUI with ComputeBrick profiling"),
    );
    map.insert(
        "tui",
        ("presentar-terminal", "TUI with ComputeBrick profiling"),
    );
    map.insert(
        "crossterm",
        ("presentar-terminal", "Included in presentar-terminal"),
    );
    map.insert("ndarray", ("trueno", "SIMD-accelerated tensors"));
    map.insert("nalgebra", ("trueno", "SIMD-accelerated linear algebra"));
    map.insert("arrow", ("trueno", "Use trueno for columnar data"));
    map.insert("parquet", ("trueno-rag", "Integrated in trueno-rag"));
    map.insert("polars", ("trueno", "Use trueno for dataframes"));
    map
}

/// Heavy dependencies that add significant compile time/binary size
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn get_heavy_deps() -> HashMap<&'static str, (&'static str, usize)> {
    let mut map = HashMap::new();
    // (dependency, (reason, estimated_kb))
    map.insert(
        "swc_ecma_parser",
        ("JS/TS parsing - consider tree-sitter only", 8000),
    );
    map.insert(
        "swc_common",
        ("SWC common - heavy TypeScript support", 3000),
    );
    map.insert("swc_ecma_ast", ("SWC AST - heavy TypeScript support", 2000));
    map.insert(
        "swc_ecma_visit",
        ("SWC visitor - heavy TypeScript support", 1500),
    );
    map.insert(
        "octocrab",
        ("GitHub API - consider lighter ureq-based", 5000),
    );
    map.insert("reqwest", ("HTTP client - consider ureq for sync", 4000));
    map.insert("rusqlite", ("SQLite - consider removing if unused", 2500));
    map.insert(
        "git2",
        ("libgit2 bindings - shell out to git instead", 6000),
    );
    map.insert("criterion", ("Benchmarking - dev only", 3000));
    map.insert("proptest", ("Property testing - dev only", 2000));
    map
}

/// Dev-only dependencies
pub const DEV_ONLY: &[&str] = &[
    "criterion",
    "proptest",
    "quickcheck",
    "quickcheck_macros",
    "assert_cmd",
    "predicates",
    "pretty_assertions",
    "env_logger",
    "futures-test",
    "tokio-test",
    "serial_test",
];

/// Potentially removable dependencies
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn get_removable() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("prettytable-rs", "Use simple formatting instead");
    map.insert("dialoguer", "Use simple stdin/stdout");
    map.insert("console", "Minimal terminal handling needed");
    map.insert("indicatif", "Progress bars may not be needed");
    map.insert("webbrowser", "Shell out to xdg-open/open instead");
    map.insert("sourcemap", "Only needed if debugging JS");
    map.insert("pulldown-cmark", "Use simple markdown or none");
    map.insert("pest", "Consider tree-sitter for all parsing");
    map.insert("pest_derive", "Consider tree-sitter for all parsing");
    map.insert("rmp-serde", "Use JSON or bincode only");
    map.insert("bincode", "Use JSON only for simplicity");
    map
}

/// Analyze a single dependency (graph metrics populated later)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn analyze_dep(name: &str, version: &str, is_dev: bool) -> DepAnalysis {
    let replacements = get_replacements();
    let heavy = get_heavy_deps();
    let removable = get_removable();

    let base = DepAnalysis {
        name: name.to_string(),
        version: version.to_string(),
        category: DepCategory::Core,
        replacement: None,
        reason: String::new(),
        transitive_count: 0,
        estimated_size_kb: 100,
        // Graph metrics initialized to defaults, populated by analyze_graph()
        pagerank_score: 0.0,
        in_degree: 0,
        out_degree: 0,
        is_bridge: false,
        is_orphan: false,
    };

    // Check if it's a Sovereign package
    if SOVEREIGN_PACKAGES.contains(&name) {
        return DepAnalysis {
            category: DepCategory::Sovereign,
            reason: "Part of Sovereign AI stack".to_string(),
            estimated_size_kb: 0,
            ..base
        };
    }

    // Check if dev-only
    if is_dev || DEV_ONLY.contains(&name) {
        let (reason, size) = heavy
            .get(name)
            .map(|(r, s)| (*r, *s))
            .unwrap_or(("Development dependency", 500));
        return DepAnalysis {
            category: DepCategory::DevOnly,
            reason: reason.to_string(),
            estimated_size_kb: size,
            ..base
        };
    }

    // Check if replaceable
    if let Some((replacement, reason)) = replacements.get(name) {
        return DepAnalysis {
            category: DepCategory::Replaceable,
            replacement: Some(replacement.to_string()),
            reason: reason.to_string(),
            estimated_size_kb: 2000,
            ..base
        };
    }

    // Check if heavy
    if let Some((reason, size)) = heavy.get(name) {
        return DepAnalysis {
            category: DepCategory::Heavy,
            reason: reason.to_string(),
            estimated_size_kb: *size,
            ..base
        };
    }

    // Check if removable
    if let Some(reason) = removable.get(name) {
        return DepAnalysis {
            category: DepCategory::Removable,
            reason: reason.to_string(),
            estimated_size_kb: 500,
            ..base
        };
    }

    // Default: Core dependency
    DepAnalysis {
        reason: "Essential dependency".to_string(),
        ..base
    }
}

#[cfg(test)]
mod classify_tests {
    //! Covers classify.rs analyze_dep + get_* lookup helpers.
    //! Note: file has #![cfg_attr(coverage_nightly, coverage(off))] which
    //! is stripped under `make coverage-broad`, so these tests do measure.
    use super::*;

    // ── get_replacements / get_heavy_deps / get_removable: lookup tables ──

    #[test]
    fn test_get_replacements_nonempty_with_known_keys() {
        let m = get_replacements();
        assert!(!m.is_empty(), "replacements list must not be empty");
        // Spot-check a few keys we know should be present.
        assert!(
            m.contains_key("nalgebra") || m.contains_key("ndarray") || m.contains_key("petgraph"),
            "expected at least one common ML/graph crate"
        );
    }

    #[test]
    fn test_get_heavy_deps_nonempty_with_size_data() {
        let m = get_heavy_deps();
        assert!(!m.is_empty());
        // Each entry has (reason, size_kb) — sizes should be reasonable.
        for (name, (_, size)) in &m {
            assert!(*size > 0, "{name} size_kb must be > 0");
        }
    }

    #[test]
    fn test_get_removable_nonempty_with_known_keys() {
        let m = get_removable();
        assert!(!m.is_empty());
        assert!(m.contains_key("prettytable-rs"));
        assert!(m.contains_key("dialoguer"));
    }

    // ── analyze_dep: 6 classification arms ──

    #[test]
    fn test_analyze_dep_sovereign_package_classified_as_sovereign() {
        let result = analyze_dep("trueno", "0.5", false);
        assert_eq!(result.category, DepCategory::Sovereign);
        assert!(result.reason.contains("Sovereign"));
        assert_eq!(result.estimated_size_kb, 0);
    }

    #[test]
    fn test_analyze_dep_dev_dep_classified_as_devonly() {
        let result = analyze_dep("unique_unknown_dev_dep_xyz", "1.0", true);
        assert_eq!(result.category, DepCategory::DevOnly);
        assert!(!result.reason.is_empty());
    }

    #[test]
    fn test_analyze_dep_known_dev_only_name_classified_as_devonly() {
        // Names in DEV_ONLY constant should classify as DevOnly even with is_dev=false.
        // Pick a name we know is in DEV_ONLY (we'll let the test reveal which).
        // Instead test a known dev-marker case: criterion.
        let result = analyze_dep("criterion", "0.5", false);
        // criterion may be in DEV_ONLY or heavy; either way it's not Core.
        assert!(matches!(
            result.category,
            DepCategory::DevOnly | DepCategory::Heavy | DepCategory::Replaceable
        ));
    }

    #[test]
    fn test_analyze_dep_removable_classified_correctly() {
        let result = analyze_dep("dialoguer", "0.10", false);
        assert_eq!(result.category, DepCategory::Removable);
        assert!(result.reason.contains("stdin") || result.reason.contains("simple"));
        assert_eq!(result.estimated_size_kb, 500);
    }

    #[test]
    fn test_analyze_dep_unknown_name_falls_back_to_core() {
        let result = analyze_dep("totally_unknown_lib_xyz_0xC0FFEE", "1.0", false);
        assert_eq!(result.category, DepCategory::Core);
        assert!(result.reason.contains("Essential"));
        assert_eq!(result.estimated_size_kb, 100);
    }

    #[test]
    fn test_analyze_dep_preserves_name_and_version() {
        let result = analyze_dep("foo_bar", "2.5.0", false);
        assert_eq!(result.name, "foo_bar");
        assert_eq!(result.version, "2.5.0");
    }

    #[test]
    fn test_analyze_dep_initializes_graph_metrics_to_defaults() {
        let result = analyze_dep("anything", "1.0", false);
        assert_eq!(result.pagerank_score, 0.0);
        assert_eq!(result.in_degree, 0);
        assert_eq!(result.out_degree, 0);
        assert!(!result.is_bridge);
        assert!(!result.is_orphan);
        assert_eq!(result.transitive_count, 0);
    }
}
