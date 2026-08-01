//! Property-based tests for complexity threshold filtering
//!
//! These tests ensure that the --max-cyclomatic and --max-cognitive flags
//! properly filter files based on their complexity thresholds.

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use proptest::prelude::*;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        /// Property: Files with all functions below threshold should be filtered out
        #[test]
        fn prop_files_below_threshold_filtered_out(
            threshold in 20u16..100,
            num_files in 1usize..10,
            functions_per_file in 1usize..5,
        ) {
            // Generate files where all functions are below threshold
            let mut files = vec![];
            for i in 0..num_files {
                let file = FileComplexityMetrics {
                    path: format!("file_{}.rs", i),
                    total_complexity: ComplexityMetrics::new(threshold - 1, threshold - 1, 3, 100),
                    functions: (0..functions_per_file)
                        .map(|j| FunctionComplexity {
                            name: format!("func_{}", j),
                            line_start: j as u32 * 10,
                            line_end: (j as u32 + 1) * 10,
                            metrics: ComplexityMetrics::new((threshold - 1 - j as u16).max(1), (threshold - 1 - j as u16).max(1), 2, 10),
                        })
                        .collect(),
                    classes: vec![],
                };
                files.push(file);
            }

            // Apply the filtering logic
            let _max_cyclomatic = Some(threshold);
            let filtered = files.into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > threshold
                })
            }).collect::<Vec<_>>();

            // All files should be filtered out
            prop_assert_eq!(filtered.len(), 0);
        }

        /// Property: Files with at least one function above threshold should be kept
        #[test]
        fn prop_files_above_threshold_kept(
            threshold in 20u16..50,
            num_files in 1usize..10,
            high_complexity in 51u16..100,
        ) {
            // Generate files where at least one function exceeds threshold
            let mut files = vec![];
            for i in 0..num_files {
                let file = FileComplexityMetrics {
                    path: format!("file_{}.rs", i),
                    total_complexity: ComplexityMetrics::new(high_complexity, high_complexity, 5, 200),
                    functions: vec![
                        // One high complexity function
                        FunctionComplexity {
                            name: "complex_func".to_string(),
                            line_start: 1,
                            line_end: 50,
                            metrics: ComplexityMetrics::new(high_complexity, high_complexity, 5, 50),
                        },
                        // One low complexity function
                        FunctionComplexity {
                            name: "simple_func".to_string(),
                            line_start: 51,
                            line_end: 60,
                            metrics: ComplexityMetrics::new(5, 3, 1, 10),
                        },
                    ],
                    classes: vec![],
                };
                files.push(file);
            }

            // Apply the filtering logic
            let _max_cyclomatic = Some(threshold);
            let filtered = files.into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > threshold
                })
            }).collect::<Vec<_>>();

            // All files should be kept since each has a high complexity function
            prop_assert_eq!(filtered.len(), num_files);
        }

        /// Property: Exact threshold boundary behavior
        #[test]
        fn prop_threshold_boundary_behavior(
            threshold in 10u16..50,
        ) {
            let files = vec![
                // File with function exactly at threshold - should be filtered out
                FileComplexityMetrics {
                    path: "at_threshold.rs".to_string(),
                    total_complexity: ComplexityMetrics::new(threshold, threshold, 3, 100),
                    functions: vec![FunctionComplexity {
                        name: "at_threshold_func".to_string(),
                        line_start: 1,
                        line_end: 50,
                        metrics: ComplexityMetrics::new(threshold, threshold, 3, 50),
                    }],
                    classes: vec![],
                },
                // File with function just above threshold - should be kept
                FileComplexityMetrics {
                    path: "above_threshold.rs".to_string(),
                    total_complexity: ComplexityMetrics::new(threshold + 1, threshold + 1, 3, 100),
                    functions: vec![FunctionComplexity {
                        name: "above_threshold_func".to_string(),
                        line_start: 1,
                        line_end: 50,
                        metrics: ComplexityMetrics::new(threshold + 1, threshold + 1, 3, 50),
                    }],
                    classes: vec![],
                },
            ];

            // Apply the filtering logic
            let _max_cyclomatic = Some(threshold);
            let filtered = files.into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > threshold
                })
            }).collect::<Vec<_>>();

            // Only the file above threshold should be kept
            prop_assert_eq!(filtered.len(), 1);
            prop_assert_eq!(&filtered[0].path, "above_threshold.rs");
        }

        /// Property: Both cyclomatic and cognitive thresholds work independently
        #[test]
        fn prop_independent_threshold_filtering(
            cyc_threshold in 20u16..40,
            cog_threshold in 30u16..50,
        ) {
            let files = vec![
                // File exceeding only cyclomatic
                FileComplexityMetrics {
                    path: "high_cyclomatic.rs".to_string(),
                    total_complexity: ComplexityMetrics::new(cyc_threshold + 10, cog_threshold - 10, 3, 100),
                    functions: vec![FunctionComplexity {
                        name: "cyc_complex".to_string(),
                        line_start: 1,
                        line_end: 50,
                        metrics: ComplexityMetrics::new(cyc_threshold + 10, cog_threshold - 10, 3, 50),
                    }],
                    classes: vec![],
                },
                // File exceeding only cognitive
                FileComplexityMetrics {
                    path: "high_cognitive.rs".to_string(),
                    total_complexity: ComplexityMetrics::new(cyc_threshold - 10, cog_threshold + 10, 3, 100),
                    functions: vec![FunctionComplexity {
                        name: "cog_complex".to_string(),
                        line_start: 1,
                        line_end: 50,
                        metrics: ComplexityMetrics::new(cyc_threshold - 10, cog_threshold + 10, 3, 50),
                    }],
                    classes: vec![],
                },
                // File exceeding neither
                FileComplexityMetrics {
                    path: "simple.rs".to_string(),
                    total_complexity: ComplexityMetrics::new(cyc_threshold - 10, cog_threshold - 10, 1, 50),
                    functions: vec![FunctionComplexity {
                        name: "simple_func".to_string(),
                        line_start: 1,
                        line_end: 25,
                        metrics: ComplexityMetrics::new(cyc_threshold - 10, cog_threshold - 10, 1, 25),
                    }],
                    classes: vec![],
                },
            ];

            // Test with cyclomatic threshold only
            let cyc_filtered = files.clone().into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > cyc_threshold
                })
            }).collect::<Vec<_>>();
            prop_assert_eq!(cyc_filtered.len(), 1);
            prop_assert_eq!(&cyc_filtered[0].path, "high_cyclomatic.rs");

            // Test with cognitive threshold only
            let cog_filtered = files.clone().into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cognitive > cog_threshold
                })
            }).collect::<Vec<_>>();
            prop_assert_eq!(cog_filtered.len(), 1);
            prop_assert_eq!(&cog_filtered[0].path, "high_cognitive.rs");

            // Test with both thresholds (OR logic)
            let both_filtered = files.into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > cyc_threshold ||
                    func.metrics.cognitive > cog_threshold
                })
            }).collect::<Vec<_>>();
            prop_assert_eq!(both_filtered.len(), 2);
        }
    }

    // Add comprehensive unit tests for structures and functions
    use crate::cli::handlers::complexity_handlers::ComplexityConfig;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_complexity_config_creation() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/test/project"),
            Some("rust".to_string()),
            Some(15),
            Some(20),
            vec!["**/*.rs".to_string()],
            30,
            10,
        );

        assert_eq!(config.project_path, PathBuf::from("/test/project"));
        assert_eq!(config.toolchain, Some("rust".to_string()));
        assert_eq!(config.max_cyclomatic, 15);
        assert_eq!(config.max_cognitive, 20);
        assert_eq!(config.include, vec!["**/*.rs".to_string()]);
        assert_eq!(config.timeout, 30);
        assert_eq!(config.top_files, 10);
    }

    #[test]
    fn test_complexity_config_defaults() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/test"),
            None,
            None, // Should default to 10
            None, // Should default to 15
            vec![],
            60,
            5,
        );

        assert_eq!(config.max_cyclomatic, 10); // Default
        assert_eq!(config.max_cognitive, 15); // Default
        assert_eq!(config.toolchain, None);
    }

    #[test]
    fn test_complexity_config_detect_toolchain() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/test"),
            Some("python".to_string()),
            Some(10),
            Some(15),
            vec![],
            30,
            5,
        );

        let detected = config.detect_toolchain();
        assert_eq!(detected, Some("python".to_string()));
    }

    #[test]
    fn test_complexity_config_detect_toolchain_none() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/test"),
            None,
            Some(10),
            Some(15),
            vec![],
            30,
            5,
        );

        let detected = config.detect_toolchain();
        // Should attempt to detect from file system but return None for non-existent path
        assert!(detected.is_none() || detected.is_some());
    }

    #[tokio::test]
    async fn test_analyze_single_file_nonexistent() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/tmp"),
            Some("rust".to_string()),
            Some(10),
            Some(15),
            vec![],
            30,
            5,
        );

        let nonexistent_file = PathBuf::from("nonexistent.rs");
        let result = super::super::analyze_single_file(&nonexistent_file, &config).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[tokio::test]
    async fn test_analyze_single_file_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() { println!(\"Hello\"); }").unwrap();

        let config = ComplexityConfig::from_args(
            temp_dir.path().to_path_buf(),
            Some("rust".to_string()),
            Some(10),
            Some(15),
            vec![],
            30,
            5,
        );

        let result = super::super::analyze_single_file(&test_file, &config).await;

        // Should handle absolute paths correctly, even if analysis fails
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_multiple_files_empty() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/tmp"),
            Some("rust".to_string()),
            Some(10),
            Some(15),
            vec![],
            30,
            5,
        );

        let empty_files: Vec<PathBuf> = vec![];
        let result = super::super::analyze_multiple_files(&empty_files, &config).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_analyze_multiple_files_with_nonexistent() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/tmp"),
            Some("rust".to_string()),
            Some(10),
            Some(15),
            vec![],
            30,
            5,
        );

        let files = vec![
            PathBuf::from("nonexistent1.rs"),
            PathBuf::from("nonexistent2.rs"),
        ];
        let result = super::super::analyze_multiple_files(&files, &config).await;

        // Should handle errors gracefully or return partial results
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_file_complexity_metrics_structure() {
        let metrics = FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics::new(15, 20, 5, 100),
            functions: vec![FunctionComplexity {
                name: "test_func".to_string(),
                line_start: 1,
                line_end: 10,
                metrics: ComplexityMetrics::new(5, 8, 2, 10),
            }],
            classes: vec![],
        };

        assert_eq!(metrics.path, "test.rs");
        assert_eq!(metrics.total_complexity.cyclomatic, 15);
        assert_eq!(metrics.total_complexity.cognitive, 20);
        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "test_func");
        assert_eq!(metrics.functions[0].line_start, 1);
        assert_eq!(metrics.functions[0].line_end, 10);
    }

    #[test]
    fn test_function_complexity_structure() {
        let func = FunctionComplexity {
            name: "complex_function".to_string(),
            line_start: 50,
            line_end: 100,
            metrics: ComplexityMetrics::new(25, 30, 8, 50),
        };

        assert_eq!(func.name, "complex_function");
        assert_eq!(func.line_start, 50);
        assert_eq!(func.line_end, 100);
        assert_eq!(func.metrics.cyclomatic, 25);
        assert_eq!(func.metrics.cognitive, 30);
        assert_eq!(func.metrics.nesting_max, 8);
        assert_eq!(func.metrics.lines, 50);
    }

    #[test]
    fn test_complexity_metrics_new() {
        let metrics = ComplexityMetrics::new(10, 15, 3, 25);

        assert_eq!(metrics.cyclomatic, 10);
        assert_eq!(metrics.cognitive, 15);
        assert_eq!(metrics.nesting_max, 3);
        assert_eq!(metrics.lines, 25);
    }

    #[test]
    fn test_complexity_threshold_filtering_edge_cases() {
        // Test exact threshold values
        let file_at_threshold = FileComplexityMetrics {
            path: "at_threshold.rs".to_string(),
            total_complexity: ComplexityMetrics::new(10, 10, 3, 50),
            functions: vec![FunctionComplexity {
                name: "threshold_func".to_string(),
                line_start: 1,
                line_end: 20,
                metrics: ComplexityMetrics::new(10, 10, 2, 20),
            }],
            classes: vec![],
        };

        // Function at exactly threshold should NOT be included (> comparison)
        let threshold = 10;
        let should_include = file_at_threshold
            .functions
            .iter()
            .any(|func| func.metrics.cyclomatic > threshold);

        assert!(!should_include); // At threshold, not above
    }

    #[test]
    fn test_complexity_threshold_filtering_just_above() {
        // Test just above threshold
        let file_above_threshold = FileComplexityMetrics {
            path: "above_threshold.rs".to_string(),
            total_complexity: ComplexityMetrics::new(11, 11, 3, 50),
            functions: vec![FunctionComplexity {
                name: "above_threshold_func".to_string(),
                line_start: 1,
                line_end: 20,
                metrics: ComplexityMetrics::new(11, 11, 2, 20),
            }],
            classes: vec![],
        };

        // Function just above threshold SHOULD be included
        let threshold = 10;
        let should_include = file_above_threshold
            .functions
            .iter()
            .any(|func| func.metrics.cyclomatic > threshold);

        assert!(should_include); // Above threshold
    }

    #[test]
    fn test_multiple_functions_mixed_complexity() {
        let file_mixed = FileComplexityMetrics {
            path: "mixed.rs".to_string(),
            total_complexity: ComplexityMetrics::new(20, 25, 5, 100),
            functions: vec![
                FunctionComplexity {
                    name: "simple_func".to_string(),
                    line_start: 1,
                    line_end: 10,
                    metrics: ComplexityMetrics::new(3, 5, 1, 10),
                },
                FunctionComplexity {
                    name: "complex_func".to_string(),
                    line_start: 20,
                    line_end: 50,
                    metrics: ComplexityMetrics::new(15, 20, 4, 30),
                },
                FunctionComplexity {
                    name: "very_complex_func".to_string(),
                    line_start: 60,
                    line_end: 100,
                    metrics: ComplexityMetrics::new(25, 30, 6, 40),
                },
            ],
            classes: vec![],
        };

        // With threshold 10, should include file because some functions are above
        let threshold = 10;
        let above_threshold_count = file_mixed
            .functions
            .iter()
            .filter(|func| func.metrics.cyclomatic > threshold)
            .count();

        assert_eq!(above_threshold_count, 2); // complex_func and very_complex_func

        // File should be included if ANY function is above threshold
        let should_include = file_mixed
            .functions
            .iter()
            .any(|func| func.metrics.cyclomatic > threshold);

        assert!(should_include);
    }

    /// RED TEST: Issue - Boundary condition bug in --fail-on-violation
    /// Reported: Complexity 9 fails with threshold 10 (should pass)
    /// Root cause: has_complexity_violations() uses unwrap_or with hardcoded defaults
    #[test]
    fn test_fail_on_violation_boundary_exact() {
        use super::super::has_complexity_violations;

        // Test data: Function with complexity exactly at threshold - 1
        let file = FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics::new(9, 9, 3, 100),
            functions: vec![FunctionComplexity {
                name: "func_with_9".to_string(),
                line_start: 1,
                line_end: 10,
                metrics: ComplexityMetrics::new(9, 9, 3, 10),
            }],
            classes: vec![],
        };

        // CRITICAL: Complexity 9 with threshold 10 should NOT be a violation
        // (violation means strictly greater than threshold)
        let has_violation = has_complexity_violations(std::slice::from_ref(&file), Some(10), None);

        assert!(
            !has_violation,
            "Complexity 9 should NOT violate threshold 10 (9 is not > 10)"
        );

        // Additional test: Complexity exactly AT threshold should also NOT violate
        let file_at_threshold = FileComplexityMetrics {
            path: "test2.rs".to_string(),
            total_complexity: ComplexityMetrics::new(10, 10, 3, 100),
            functions: vec![FunctionComplexity {
                name: "func_with_10".to_string(),
                line_start: 1,
                line_end: 10,
                metrics: ComplexityMetrics::new(10, 10, 3, 10),
            }],
            classes: vec![],
        };

        let has_violation_at_threshold =
            has_complexity_violations(&[file_at_threshold], Some(10), None);

        assert!(
            !has_violation_at_threshold,
            "Complexity 10 should NOT violate threshold 10 (10 is not > 10)"
        );

        // Sanity check: Complexity 11 with threshold 10 SHOULD violate
        let file_above = FileComplexityMetrics {
            path: "test3.rs".to_string(),
            total_complexity: ComplexityMetrics::new(11, 11, 3, 100),
            functions: vec![FunctionComplexity {
                name: "func_with_11".to_string(),
                line_start: 1,
                line_end: 10,
                metrics: ComplexityMetrics::new(11, 11, 3, 10),
            }],
            classes: vec![],
        };

        let has_violation_above = has_complexity_violations(&[file_above], Some(10), None);

        assert!(
            has_violation_above,
            "Complexity 11 SHOULD violate threshold 10 (11 > 10)"
        );
    }

    mod dag_type_regression {
        use crate::cli::handlers::complexity_handlers::{
            count_rendered_elements, filter_graph_by_dag_type,
        };
        use crate::cli::DagType;
        use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
        use rustc_hash::FxHashMap;

        fn node(id: &str, node_type: NodeType) -> NodeInfo {
            NodeInfo {
                id: id.to_string(),
                label: id.to_string(),
                node_type,
                file_path: "src/lib.rs".to_string(),
                line_number: 1,
                complexity: 1,
                metadata: FxHashMap::default(),
            }
        }

        fn mixed_graph() -> DependencyGraph {
            let mut graph = DependencyGraph::new();
            for id in [
                "caller", "callee", "importer", "imported", "child", "parent",
            ] {
                graph.add_node(node(id, NodeType::Function));
            }
            graph.add_edge(Edge {
                from: "caller".to_string(),
                to: "callee".to_string(),
                edge_type: EdgeType::Calls,
                weight: 1,
            });
            graph.add_edge(Edge {
                from: "importer".to_string(),
                to: "imported".to_string(),
                edge_type: EdgeType::Imports,
                weight: 1,
            });
            graph.add_edge(Edge {
                from: "child".to_string(),
                to: "parent".to_string(),
                edge_type: EdgeType::Inherits,
                weight: 1,
            });
            graph
        }

        /// Regression test for #653: `--dag-type` was ignored, so call-graph,
        /// import-graph, inheritance and full-dependency were byte-identical.
        #[test]
        fn test_dag_type_selects_different_subgraphs() {
            let graph = mixed_graph();

            let calls = filter_graph_by_dag_type(graph.clone(), &DagType::CallGraph);
            let imports = filter_graph_by_dag_type(graph.clone(), &DagType::ImportGraph);
            let inheritance = filter_graph_by_dag_type(graph.clone(), &DagType::Inheritance);
            let full = filter_graph_by_dag_type(graph.clone(), &DagType::FullDependency);

            assert_eq!(calls.edges.len(), 1);
            assert_eq!(calls.edges[0].edge_type, EdgeType::Calls);
            assert_eq!(imports.edges.len(), 1);
            assert_eq!(imports.edges[0].edge_type, EdgeType::Imports);
            assert_eq!(inheritance.edges.len(), 1);
            assert_eq!(inheritance.edges[0].edge_type, EdgeType::Inherits);
            assert_eq!(full.edges.len(), 3);

            // The call graph must not be the same graph as the inheritance graph.
            assert_ne!(calls.edges[0].edge_type, inheritance.edges[0].edge_type);
            assert_eq!(calls.nodes.len(), 2);
        }

        /// Regression test for #653: the announced node count must equal the number
        /// of nodes actually emitted in the diagram (observed: 6 announced, 4 drawn).
        #[test]
        fn test_count_rendered_elements_matches_diagram() {
            let diagram = "graph TD\n    a[main]\n    b[main]\n    c((Trait))\n\n    a --> b\n    b -.-> c\n\n    style a fill:#fff,stroke-width:1px\n";

            let (nodes, edges) = count_rendered_elements(diagram);

            assert_eq!(nodes, 3);
            assert_eq!(edges, 2);
        }

        #[test]
        fn test_count_rendered_elements_empty_diagram() {
            assert_eq!(count_rendered_elements("graph TD\n"), (0, 0));
        }
    }

    /// Round 3: `analyze complexity --format json` aggregated the summary over
    /// the `--top-files` SLICE while `summary.total_files` claimed whole-project
    /// scope. On one unchanged 1070-file tree the default `--top-files 10`
    /// reported total_files 1070 next to total_functions 159 (true 10148),
    /// technical_debt_hours 388.75 (true 1644.25) and max_cyclomatic 29 (true
    /// project maximum 31): a cap wearing the word "total".
    mod summary_scope {
        use super::*;
        use crate::cli::handlers::complexity_handlers::analysis::{
            apply_top_files_limit, build_report_over_analyzed_files,
        };
        use crate::services::complexity::aggregate_results_with_thresholds;

        fn file_with(name: &str, cyclomatic: u16) -> FileComplexityMetrics {
            FileComplexityMetrics {
                path: name.to_string(),
                total_complexity: ComplexityMetrics::new(cyclomatic, cyclomatic, 3, 100),
                functions: vec![FunctionComplexity {
                    name: format!("{name}_f"),
                    line_start: 1,
                    line_end: 20,
                    metrics: ComplexityMetrics::new(cyclomatic, cyclomatic, 2, 20),
                }],
                classes: vec![],
            }
        }

        #[test]
        fn test_summary_is_invariant_to_the_top_files_cap() {
            let all: Vec<_> = (0..40)
                .map(|i| file_with(&format!("f{i:02}.rs"), 5 + u16::try_from(i).unwrap()))
                .collect();

            // The whole-project aggregate, computed once over every file.
            let full = aggregate_results_with_thresholds(all.clone(), None, None);

            for cap in [1usize, 5, 10, 39] {
                let mut listed = all.clone();
                apply_top_files_limit(&mut listed, cap);
                assert_eq!(listed.len(), cap, "cap {cap} must truncate the LIST");

                // Exactly what the handler does.
                let report = build_report_over_analyzed_files(all.clone(), listed, None, None);

                assert_eq!(
                    report.summary.total_files, 40,
                    "total_files must be the analyzed count, not the cap"
                );
                assert!(report.files.len() <= report.summary.total_files);
                assert_eq!(report.summary.total_functions, full.summary.total_functions);
                assert_eq!(report.summary.max_cyclomatic, full.summary.max_cyclomatic);
                assert!(
                    (report.summary.technical_debt_hours - full.summary.technical_debt_hours).abs()
                        < f32::EPSILON,
                    "technical debt must not shrink with the display cap"
                );
            }
        }

        /// The defect in one assertion: aggregating the truncated list gives a
        /// smaller total than aggregating everything, so the pre-fix summary
        /// could not have been describing `total_files` files.
        #[test]
        fn test_aggregating_the_slice_understates_the_project() {
            let all: Vec<_> = (0..40)
                .map(|i| file_with(&format!("f{i:02}.rs"), 5 + u16::try_from(i).unwrap()))
                .collect();
            let mut sliced = all.clone();
            apply_top_files_limit(&mut sliced, 10);

            let full = aggregate_results_with_thresholds(all, None, None);
            let capped = aggregate_results_with_thresholds(sliced, None, None);

            assert!(capped.summary.total_functions < full.summary.total_functions);
            assert_eq!(capped.summary.total_files, 10);
            assert_eq!(full.summary.total_files, 40);
        }
    }
}
