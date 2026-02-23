#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::super::classify::analyze_dep;
    use super::super::types::DepCategory;

    #[test]
    fn test_analyze_sovereign_dep() {
        let dep = analyze_dep("trueno-graph", "0.1.10", false);
        assert_eq!(dep.category, DepCategory::Sovereign);
    }

    #[test]
    fn test_analyze_replaceable_dep() {
        let dep = analyze_dep("petgraph", "0.6.0", false);
        assert_eq!(dep.category, DepCategory::Replaceable);
        assert_eq!(dep.replacement, Some("trueno-graph".to_string()));
    }

    #[test]
    fn test_analyze_heavy_dep() {
        let dep = analyze_dep("swc_ecma_parser", "24.0.0", false);
        assert_eq!(dep.category, DepCategory::Heavy);
        assert!(dep.estimated_size_kb > 5000);
    }

    #[test]
    fn test_analyze_dev_dep() {
        let dep = analyze_dep("criterion", "0.5.0", true);
        assert_eq!(dep.category, DepCategory::DevOnly);
    }

    #[test]
    fn test_analyze_removable_dep() {
        let dep = analyze_dep("prettytable-rs", "0.10.0", false);
        assert_eq!(dep.category, DepCategory::Removable);
    }
}

#[cfg(test)]
mod coverage_instrumented_tests {
    use std::collections::{HashMap, HashSet};

    use super::super::classify::{analyze_dep, get_heavy_deps, get_removable, get_replacements};
    use super::super::graph::{analyze_dependency_graph, apply_graph_analysis, count_transitive_deps};
    use super::super::pareto::estimate_effort;
    use super::super::types::{
        DepAnalysis, DepCategory, GraphAnalysis, ParetoEffort, SortMode,
    };
    use trueno_graph::NodeId;

    // ====================================================================
    // analyze_dep: all classification branches
    // ====================================================================

    #[test]
    fn test_ci_analyze_dep_sovereign_trueno() {
        let dep = analyze_dep("trueno", "0.5.0", false);
        assert_eq!(dep.category, DepCategory::Sovereign);
        assert_eq!(dep.reason, "Part of Sovereign AI stack");
        assert_eq!(dep.estimated_size_kb, 0);
    }

    #[test]
    fn test_ci_analyze_dep_sovereign_aprender() {
        let dep = analyze_dep("aprender", "1.0.0", false);
        assert_eq!(dep.category, DepCategory::Sovereign);
    }

    #[test]
    fn test_ci_analyze_dep_dev_only_explicit_flag() {
        // is_dev=true forces DevOnly even if name isn't in DEV_ONLY list
        let dep = analyze_dep("some-unknown-dep", "0.1.0", true);
        assert_eq!(dep.category, DepCategory::DevOnly);
    }

    #[test]
    fn test_ci_analyze_dep_dev_only_by_name() {
        // DEV_ONLY list match, even with is_dev=false
        let dep = analyze_dep("pretty_assertions", "1.0.0", false);
        assert_eq!(dep.category, DepCategory::DevOnly);
    }

    #[test]
    fn test_ci_analyze_dep_dev_and_heavy_overlap() {
        // criterion is both in DEV_ONLY and get_heavy_deps
        let dep = analyze_dep("criterion", "0.5.0", false);
        assert_eq!(dep.category, DepCategory::DevOnly);
        // Heavy size should be used (3000) not default 500
        assert_eq!(dep.estimated_size_kb, 3000);
    }

    #[test]
    fn test_ci_analyze_dep_replaceable_ndarray() {
        let dep = analyze_dep("ndarray", "0.15.0", false);
        assert_eq!(dep.category, DepCategory::Replaceable);
        assert_eq!(dep.replacement, Some("trueno".to_string()));
        assert_eq!(dep.estimated_size_kb, 2000);
    }

    #[test]
    fn test_ci_analyze_dep_heavy_git2() {
        let dep = analyze_dep("git2", "0.18.0", false);
        assert_eq!(dep.category, DepCategory::Heavy);
        assert_eq!(dep.estimated_size_kb, 6000);
    }

    #[test]
    fn test_ci_analyze_dep_removable_bincode() {
        let dep = analyze_dep("bincode", "1.3.0", false);
        assert_eq!(dep.category, DepCategory::Removable);
        assert_eq!(dep.estimated_size_kb, 500);
    }

    #[test]
    fn test_ci_analyze_dep_core_unknown() {
        // Falls through to Core (default)
        let dep = analyze_dep("serde", "1.0.0", false);
        assert_eq!(dep.category, DepCategory::Core);
        assert_eq!(dep.reason, "Essential dependency");
        assert_eq!(dep.estimated_size_kb, 100);
    }

    // ====================================================================
    // get_replacements / get_heavy_deps / get_removable
    // ====================================================================

    #[test]
    fn test_ci_get_replacements_contains_expected() {
        let r = get_replacements();
        assert!(r.contains_key("petgraph"));
        assert!(r.contains_key("nalgebra"));
        assert!(r.contains_key("polars"));
        assert_eq!(r.get("petgraph").unwrap().0, "trueno-graph");
    }

    #[test]
    fn test_ci_get_heavy_deps_contains_expected() {
        let h = get_heavy_deps();
        assert!(h.contains_key("reqwest"));
        assert!(h.contains_key("octocrab"));
        let (reason, size) = h.get("reqwest").unwrap();
        assert!(reason.contains("HTTP"));
        assert_eq!(*size, 4000);
    }

    #[test]
    fn test_ci_get_removable_contains_expected() {
        let r = get_removable();
        assert!(r.contains_key("dialoguer"));
        assert!(r.contains_key("pest"));
        assert!(r.get("pest").unwrap().contains("tree-sitter"));
    }

    // ====================================================================
    // SortMode::from_str
    // ====================================================================

    #[test]
    fn test_ci_sort_mode_size_variants() {
        assert_eq!(SortMode::from_str("size"), SortMode::Size);
        assert_eq!(SortMode::from_str("binary"), SortMode::Size);
        assert_eq!(SortMode::from_str("kb"), SortMode::Size);
    }

    #[test]
    fn test_ci_sort_mode_pagerank_variants() {
        assert_eq!(SortMode::from_str("pagerank"), SortMode::PageRank);
        assert_eq!(SortMode::from_str("rank"), SortMode::PageRank);
        assert_eq!(SortMode::from_str("critical"), SortMode::PageRank);
    }

    #[test]
    fn test_ci_sort_mode_name_variants() {
        assert_eq!(SortMode::from_str("name"), SortMode::Name);
        assert_eq!(SortMode::from_str("alpha"), SortMode::Name);
        assert_eq!(SortMode::from_str("alphabetical"), SortMode::Name);
    }

    #[test]
    fn test_ci_sort_mode_category_variants() {
        assert_eq!(SortMode::from_str("category"), SortMode::Category);
        assert_eq!(SortMode::from_str("cat"), SortMode::Category);
    }

    #[test]
    fn test_ci_sort_mode_default_transitive() {
        assert_eq!(SortMode::from_str("transitive"), SortMode::Transitive);
        assert_eq!(SortMode::from_str("anything_else"), SortMode::Transitive);
        assert_eq!(SortMode::from_str(""), SortMode::Transitive);
    }

    #[test]
    fn test_ci_sort_mode_case_insensitive() {
        assert_eq!(SortMode::from_str("SIZE"), SortMode::Size);
        assert_eq!(SortMode::from_str("PageRank"), SortMode::PageRank);
        assert_eq!(SortMode::from_str("NAME"), SortMode::Name);
    }

    // ====================================================================
    // ParetoEffort methods
    // ====================================================================

    #[test]
    fn test_ci_pareto_effort_multiplier() {
        assert_eq!(ParetoEffort::Low.multiplier(), 1.0);
        assert_eq!(ParetoEffort::Medium.multiplier(), 2.0);
        assert_eq!(ParetoEffort::High.multiplier(), 3.0);
    }

    #[test]
    fn test_ci_pareto_effort_label() {
        assert_eq!(ParetoEffort::Low.label(), "Low");
        assert_eq!(ParetoEffort::Medium.label(), "Medium");
        assert_eq!(ParetoEffort::High.label(), "High");
    }

    // ====================================================================
    // estimate_effort
    // ====================================================================

    #[test]
    fn test_ci_estimate_effort_high_effort_deps() {
        assert!(matches!(
            estimate_effort("tokio", DepCategory::Core),
            ParetoEffort::High
        ));
        assert!(matches!(
            estimate_effort("serde", DepCategory::Core),
            ParetoEffort::High
        ));
        assert!(matches!(
            estimate_effort("clap", DepCategory::Core),
            ParetoEffort::High
        ));
    }

    #[test]
    fn test_ci_estimate_effort_medium_effort_deps() {
        assert!(matches!(
            estimate_effort("git2", DepCategory::Heavy),
            ParetoEffort::Medium
        ));
        assert!(matches!(
            estimate_effort("reqwest", DepCategory::Heavy),
            ParetoEffort::Medium
        ));
        assert!(matches!(
            estimate_effort("pest", DepCategory::Removable),
            ParetoEffort::Medium
        ));
    }

    #[test]
    fn test_ci_estimate_effort_low_removable() {
        assert!(matches!(
            estimate_effort("prettytable-rs", DepCategory::Removable),
            ParetoEffort::Low
        ));
    }

    #[test]
    fn test_ci_estimate_effort_low_dev_only() {
        assert!(matches!(
            estimate_effort("my-test-util", DepCategory::DevOnly),
            ParetoEffort::Low
        ));
    }

    #[test]
    fn test_ci_estimate_effort_default_heavy() {
        // Unknown name, Heavy category -> Medium
        assert!(matches!(
            estimate_effort("unknown-heavy", DepCategory::Heavy),
            ParetoEffort::Medium
        ));
    }

    #[test]
    fn test_ci_estimate_effort_default_replaceable() {
        // Unknown name, Replaceable category -> Medium
        assert!(matches!(
            estimate_effort("unknown-replaceable", DepCategory::Replaceable),
            ParetoEffort::Medium
        ));
    }

    #[test]
    fn test_ci_estimate_effort_default_core() {
        // Unknown name, Core category -> High
        assert!(matches!(
            estimate_effort("unknown-core", DepCategory::Core),
            ParetoEffort::High
        ));
    }

    // ====================================================================
    // analyze_dependency_graph and count_transitive_deps
    // ====================================================================

    #[test]
    fn test_ci_analyze_dependency_graph_empty() {
        let analysis = analyze_dependency_graph(&[], &[], &[]);
        assert!(analysis.pagerank_scores.is_empty());
        assert!(analysis.bridges.is_empty());
        assert!(analysis.orphans.is_empty());
    }

    #[test]
    fn test_ci_analyze_dependency_graph_simple() {
        use super::super::types::DepEdge;

        let direct = vec!["a".to_string(), "b".to_string()];
        let all = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            DepEdge {
                from: "a".to_string(),
                to: "c".to_string(),
            },
            DepEdge {
                from: "b".to_string(),
                to: "c".to_string(),
            },
        ];
        let analysis = analyze_dependency_graph(&direct, &all, &edges);

        // "a" and "b" have no in-degree from direct deps -> orphans
        assert!(analysis.orphans.contains("a"));
        assert!(analysis.orphans.contains("b"));

        // transitive counts
        assert_eq!(*analysis.transitive_counts.get("a").unwrap_or(&0), 1);
        assert_eq!(*analysis.transitive_counts.get("b").unwrap_or(&0), 1);
    }

    #[test]
    fn test_ci_count_transitive_deps_missing_start() {
        let name_to_id: HashMap<String, NodeId> = HashMap::new();
        let id_to_name: HashMap<NodeId, String> = HashMap::new();
        let count = count_transitive_deps("nonexistent", &[], &name_to_id, &id_to_name);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_ci_count_transitive_deps_chain() {
        let mut name_to_id = HashMap::new();
        let mut id_to_name = HashMap::new();
        for (i, name) in ["a", "b", "c", "d"].iter().enumerate() {
            name_to_id.insert(name.to_string(), NodeId(i as u32));
            id_to_name.insert(NodeId(i as u32), name.to_string());
        }
        // Chain: a -> b -> c -> d
        let edges = vec![
            (NodeId(0), NodeId(1), 1.0),
            (NodeId(1), NodeId(2), 1.0),
            (NodeId(2), NodeId(3), 1.0),
        ];
        let count = count_transitive_deps("a", &edges, &name_to_id, &id_to_name);
        assert_eq!(count, 3); // b, c, d
    }

    // ====================================================================
    // apply_graph_analysis
    // ====================================================================

    #[test]
    fn test_ci_apply_graph_analysis() {
        let mut deps = vec![DepAnalysis {
            name: "test-dep".to_string(),
            version: "1.0.0".to_string(),
            category: DepCategory::Core,
            replacement: None,
            reason: "test".to_string(),
            transitive_count: 0,
            estimated_size_kb: 0,
            pagerank_score: 0.0,
            in_degree: 0,
            out_degree: 0,
            is_bridge: false,
            is_orphan: false,
        }];

        let mut pagerank_scores = HashMap::new();
        pagerank_scores.insert("test-dep".to_string(), 0.42_f32);
        let mut in_degrees = HashMap::new();
        in_degrees.insert("test-dep".to_string(), 3_usize);
        let mut out_degrees = HashMap::new();
        out_degrees.insert("test-dep".to_string(), 5_usize);
        let mut bridges = HashSet::new();
        bridges.insert("test-dep".to_string());
        let mut orphans = HashSet::new();
        orphans.insert("other-dep".to_string());
        let mut transitive_counts = HashMap::new();
        transitive_counts.insert("test-dep".to_string(), 10_usize);

        let analysis = GraphAnalysis {
            pagerank_scores,
            in_degrees,
            out_degrees,
            bridges,
            orphans,
            transitive_counts,
        };

        apply_graph_analysis(&mut deps, &analysis);

        assert!((deps[0].pagerank_score - 0.42).abs() < 0.001);
        assert_eq!(deps[0].in_degree, 3);
        assert_eq!(deps[0].out_degree, 5);
        assert!(deps[0].is_bridge);
        assert!(!deps[0].is_orphan);
        assert_eq!(deps[0].transitive_count, 10);
    }
}
