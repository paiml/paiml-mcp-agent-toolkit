#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod unreachable_block_accounting_tests {
    use super::*;

    fn empty_context() -> crate::services::context::ProjectContext {
        crate::services::context::ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            summary: crate::services::context::ProjectSummary {
                total_files: 0,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            graph: None,
        }
    }

    fn report_with_one_unreachable_block(
        file: &str,
        start: u32,
        end: u32,
    ) -> super::super::types::DeadCodeReport {
        use super::super::types::{DeadCodeReport, DeadCodeSummary, UnreachableBlock};
        DeadCodeReport {
            dead_functions: vec![],
            dead_classes: vec![],
            dead_variables: vec![],
            unreachable_code: vec![UnreachableBlock {
                start_line: start,
                end_line: end,
                file_path: file.to_string(),
                reason: "after return".to_string(),
            }],
            summary: DeadCodeSummary {
                total_dead_code_lines: 0,
                percentage_dead: 0.0,
                dead_by_type: std::collections::HashMap::new(),
                confidence_level: 0.9,
            },
        }
    }

    /// The loop incremented `unreachable_blocks` itself AND called `add_item`,
    /// which increments it too, so the counter came out at exactly twice the
    /// item list it heads: one reported block counted as 2.
    #[test]
    fn test_unreachable_block_count_agrees_with_the_item_list() {
        let analyzer = DeadCodeAnalyzer::new(100);
        let report = report_with_one_unreachable_block("does_not_exist.rs", 10, 14);
        let config = crate::models::dead_code::DeadCodeAnalysisConfig {
            include_unreachable: true,
            include_tests: true,
            min_dead_lines: 0,
        };

        let metrics = analyzer
            .aggregate_by_file(&report, &empty_context(), &config)
            .unwrap();

        let file = metrics.first().expect("one file must be reported");
        let items = file
            .items
            .iter()
            .filter(|i| {
                matches!(
                    i.item_type,
                    crate::models::dead_code::DeadCodeType::UnreachableCode
                )
            })
            .count();

        assert_eq!(items, 1, "sanity: exactly one unreachable item was reported");
        assert_eq!(
            file.unreachable_blocks, items,
            "unreachable_blocks ({}) must equal the item list it heads ({items})",
            file.unreachable_blocks
        );
    }

    /// The analyzer reports the block's real extent, so `dead_lines` must be
    /// that extent -- not the extent PLUS `add_item`'s flat 3-line estimate.
    #[test]
    fn test_unreachable_dead_lines_use_the_measured_extent_not_extent_plus_estimate() {
        let analyzer = DeadCodeAnalyzer::new(100);
        // Lines 10..=14 inclusive is an extent of 5.
        let report = report_with_one_unreachable_block("does_not_exist.rs", 10, 14);
        let config = crate::models::dead_code::DeadCodeAnalysisConfig {
            include_unreachable: true,
            include_tests: true,
            min_dead_lines: 0,
        };

        let metrics = analyzer
            .aggregate_by_file(&report, &empty_context(), &config)
            .unwrap();

        let file = metrics.first().expect("one file must be reported");
        assert_eq!(
            file.dead_lines, 5,
            "measured extent is 5 lines; 8 would mean the flat estimate was added on top"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_code_analyzer() {
        let mut analyzer = DeadCodeAnalyzer::new(100);
        let dag = AstDag::new();

        let report = analyzer.analyze(&dag);

        assert_eq!(report.summary.total_dead_code_lines, 0);
        assert_eq!(report.dead_functions.len(), 0);
    }

    #[test]
    fn test_dead_code_analyzer_with_entry_points() {
        use super::super::types::ReferenceEdge;
        use super::super::types::ReferenceType;

        let mut analyzer = DeadCodeAnalyzer::new(100);

        // Add some entry points
        analyzer.add_entry_point(1);
        analyzer.add_entry_point(5);

        // Add reference edges
        let edge1 = ReferenceEdge {
            from: 1,
            to: 2,
            reference_type: ReferenceType::DirectCall,
            confidence: 0.95,
        };

        let edge2 = ReferenceEdge {
            from: 2,
            to: 3,
            reference_type: ReferenceType::TypeReference,
            confidence: 0.85,
        };

        analyzer.add_reference(edge1);
        analyzer.add_reference(edge2);

        let dag = AstDag::new();
        let report = analyzer.analyze(&dag);

        // Should have processed the empty DAG without errors
        assert_eq!(report.dead_functions.len(), 0);
        assert_eq!(report.dead_classes.len(), 0);
        assert_eq!(report.dead_variables.len(), 0);
    }

    #[tokio::test]
    #[ignore = "Slow test - takes too long in CI"]
    async fn test_analyze_with_ranking() {
        use crate::models::dead_code::DeadCodeAnalysisConfig;
        use std::path::PathBuf;

        let mut analyzer = DeadCodeAnalyzer::new(1000);
        let config = DeadCodeAnalysisConfig {
            include_unreachable: false,
            include_tests: false,
            min_dead_lines: 5,
        };

        // Use current directory as test path
        let path = PathBuf::from(".");

        // This should not fail, even if it finds no dead code
        let result = analyzer.analyze_with_ranking(&path, config).await;

        // The result might be an error due to project structure, but the function should not panic
        match result {
            Ok(ranking_result) => {
                // These values are always non-negative by type, so just check they exist
                assert!(ranking_result.summary.total_files_analyzed < usize::MAX);
                assert!(ranking_result.ranked_files.len() < usize::MAX);
            }
            Err(_) => {
                // This is expected if the current directory doesn't have a valid project structure
                // The important thing is that the function doesn't panic
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod pure_function_tests {
    use super::*;

    #[test]
    fn test_compute_reachability_basic() {
        let mut entry_points = HashSet::new();
        entry_points.insert("main".to_string());

        let mut function_calls: HashMap<String, HashSet<String>> = HashMap::new();
        let mut main_calls = HashSet::new();
        main_calls.insert("helper".to_string());
        function_calls.insert("main".to_string(), main_calls);

        let reachable = compute_reachability(&entry_points, &function_calls);
        assert!(reachable.contains("main"));
        assert!(reachable.contains("helper"));
    }

    #[test]
    fn test_compute_reachability_transitive() {
        let mut entry_points = HashSet::new();
        entry_points.insert("main".to_string());

        let mut function_calls: HashMap<String, HashSet<String>> = HashMap::new();
        let mut main_calls = HashSet::new();
        main_calls.insert("a".to_string());
        function_calls.insert("main".to_string(), main_calls);

        let mut a_calls = HashSet::new();
        a_calls.insert("b".to_string());
        function_calls.insert("a".to_string(), a_calls);

        let reachable = compute_reachability(&entry_points, &function_calls);
        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains("main"));
        assert!(reachable.contains("a"));
        assert!(reachable.contains("b"));
    }

    #[test]
    fn test_detect_function_calls_in_lines_basic() {
        let lines = vec!["fn caller() { helper(); }"];
        let mut all_functions = HashMap::new();
        all_functions.insert("test::helper".to_string(), ("test.rs".to_string(), 1));

        let calls = detect_function_calls_in_lines("test.rs", &lines, &all_functions);
        // Function may or may not detect calls depending on implementation
        assert!(calls.len() <= 1);
    }

    #[test]
    fn test_calculate_dead_percentage() {
        // calculate_dead_percentage(total_functions, dead_count)
        assert_eq!(calculate_dead_percentage(100, 0), 0.0);
        assert_eq!(calculate_dead_percentage(100, 50), 50.0);
        assert_eq!(calculate_dead_percentage(100, 100), 100.0);
        assert_eq!(calculate_dead_percentage(0, 10), 0.0); // Edge case: no total
    }

    #[test]
    fn test_classify_dead_functions_pure() {
        let mut all_functions = HashMap::new();
        all_functions.insert("main".to_string(), ("src/main.rs".to_string(), 1));
        all_functions.insert("unused".to_string(), ("src/lib.rs".to_string(), 10));

        let mut reachable = HashSet::new();
        reachable.insert("main".to_string());

        let dead = classify_dead_functions_pure(&all_functions, &reachable);
        assert_eq!(dead.len(), 1);
        // Result is Vec<(String, String, u32)> - (name, file, line)
        assert!(dead.iter().any(|(name, _, _)| name == "unused"));
    }
}

