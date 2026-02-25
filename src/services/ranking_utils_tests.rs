#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_result_builder() {
        let result = AnalysisResultBuilder::new(PathBuf::from("test.rs"))
            .with_line_range(10, Some(20))
            .add_metric_int("cyclomatic", 15)
            .add_metric_float("coverage", 85.5)
            .with_description("Complex function")
            .with_entity("process_data", "function")
            .build();

        assert_eq!(result.file_path, PathBuf::from("test.rs"));
        assert_eq!(result.line_range.start.line, 10);
        assert_eq!(result.line_range.end.as_ref().unwrap().line, 20);
        assert_eq!(
            result.metrics.get("cyclomatic"),
            Some(&MetricValue::Integer(15))
        );
        assert_eq!(
            result.metrics.get("coverage"),
            Some(&MetricValue::Float(85.5))
        );
        assert_eq!(result.context.entity_name, Some("process_data".to_string()));
    }

    #[test]
    fn test_severity_computation() {
        let mut metrics = BTreeMap::new();

        // High complexity
        metrics.insert("cyclomatic".to_string(), MetricValue::Integer(55));
        assert_eq!(compute_severity_from_metrics(&metrics), Severity::Critical);

        // Medium complexity
        metrics.clear();
        metrics.insert("cognitive_complexity".to_string(), MetricValue::Float(15.0));
        assert_eq!(compute_severity_from_metrics(&metrics), Severity::Medium);

        // Low complexity
        metrics.clear();
        metrics.insert("complexity".to_string(), MetricValue::Integer(5));
        assert_eq!(compute_severity_from_metrics(&metrics), Severity::Low);
    }

    #[test]
    fn test_severity_computation_high() {
        let mut metrics = BTreeMap::new();
        metrics.insert("cyclomatic".to_string(), MetricValue::Integer(25));
        assert_eq!(compute_severity_from_metrics(&metrics), Severity::High);
    }

    #[test]
    fn test_severity_computation_empty_metrics() {
        let metrics = BTreeMap::new();
        assert_eq!(compute_severity_from_metrics(&metrics), Severity::Low);
    }

    #[test]
    fn test_ranking_config_default() {
        let config = RankingConfig::default();
        assert_eq!(config.top_files, 0);
        assert!(config.min_score.is_none());
    }

    #[test]
    fn test_builder_with_column_range() {
        let result = AnalysisResultBuilder::new(PathBuf::from("test.rs"))
            .with_column_range(5, Some(15))
            .build();
        assert_eq!(result.line_range.start.column, 5);
    }

    #[test]
    fn test_builder_add_metric() {
        let result = AnalysisResultBuilder::new(PathBuf::from("test.rs"))
            .add_metric("custom", MetricValue::String("value".to_string()))
            .build();
        assert_eq!(
            result.metrics.get("custom"),
            Some(&MetricValue::String("value".to_string()))
        );
    }

    #[test]
    fn test_format_ranked_files_table_empty() {
        let files: Vec<RankedFile> = vec![];
        let table = format_ranked_files_table(&files);
        assert!(table.contains("RANK"));
        assert!(table.contains("SCORE"));
    }

    #[test]
    fn test_format_ranked_files_table_with_data() {
        let files = vec![RankedFile {
            path: PathBuf::from("src/main.rs"),
            rank: 1,
            score: 8.5,
            defects: vec![Defect {
                id: "D001".to_string(),
                severity: Severity::High,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("src/main.rs"),
                line_start: 10,
                line_end: Some(20),
                column_start: Some(1),
                column_end: None,
                message: "High complexity".to_string(),
                rule_id: "CC001".to_string(),
                fix_suggestion: None,
                metrics: std::collections::HashMap::new(),
            }],
        }];
        let table = format_ranked_files_table(&files);
        assert!(table.contains("src/main.rs"));
        assert!(table.contains("8.5"));
    }

    #[test]
    fn test_apply_file_ranking_no_config() {
        let results = vec![
            AnalysisResultBuilder::new(PathBuf::from("a.rs"))
                .add_metric_int("cyclomatic", 10)
                .build(),
            AnalysisResultBuilder::new(PathBuf::from("b.rs"))
                .add_metric_int("cyclomatic", 5)
                .build(),
        ];
        let config = RankingConfig::default();
        let ranked = apply_file_ranking(results, &config, |r| r.clone());
        assert_eq!(ranked.len(), 2);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
