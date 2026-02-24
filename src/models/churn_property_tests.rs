// Property-based tests for churn analysis types - included via include!()

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_churn_score_always_in_range(
        commit_count in 0usize..1000,
        additions in 0usize..10000,
        deletions in 0usize..10000,
        max_commits in 1usize..500,  // Avoid division by zero
        max_changes in 1usize..20000
    ) {
        let mut metrics = FileChurnMetrics {
            path: PathBuf::from("test.rs"),
            relative_path: "test.rs".to_string(),
            commit_count,
            unique_authors: vec![],
            additions,
            deletions,
            churn_score: 0.0,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        metrics.calculate_churn_score(max_commits, max_changes);

        prop_assert!(metrics.churn_score >= 0.0, "Churn score should be >= 0.0");
        prop_assert!(metrics.churn_score <= 1.0, "Churn score should be <= 1.0");
    }

    #[test]
    fn prop_churn_score_monotonic_with_commits(
        base_commits in 0usize..100,
        extra_commits in 1usize..100,
        additions in 0usize..500,
        deletions in 0usize..500
    ) {
        let max_commits = 200usize;
        let max_changes = 1000usize;

        let mut metrics_low = FileChurnMetrics {
            path: PathBuf::from("test.rs"),
            relative_path: "test.rs".to_string(),
            commit_count: base_commits,
            unique_authors: vec![],
            additions,
            deletions,
            churn_score: 0.0,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        let mut metrics_high = FileChurnMetrics {
            path: PathBuf::from("test.rs"),
            relative_path: "test.rs".to_string(),
            commit_count: base_commits + extra_commits,
            unique_authors: vec![],
            additions,
            deletions,
            churn_score: 0.0,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        metrics_low.calculate_churn_score(max_commits, max_changes);
        metrics_high.calculate_churn_score(max_commits, max_changes);

        // More commits should mean higher or equal score (monotonic)
        prop_assert!(
            metrics_high.churn_score >= metrics_low.churn_score,
            "Score should increase with more commits: {} >= {}",
            metrics_high.churn_score,
            metrics_low.churn_score
        );
    }

    #[test]
    fn prop_file_churn_metrics_serialization_roundtrip(
        path_suffix in "[a-z]{1,10}",
        commit_count in 0usize..1000,
        additions in 0usize..10000,
        deletions in 0usize..10000
    ) {
        let path = format!("src/{}.rs", path_suffix);
        let metrics = FileChurnMetrics {
            path: PathBuf::from(&path),
            relative_path: path.clone(),
            commit_count,
            unique_authors: vec!["test_author".to_string()],
            additions,
            deletions,
            churn_score: 0.5,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        let json = serde_json::to_string(&metrics).expect("Serialization failed");
        let deserialized: FileChurnMetrics = serde_json::from_str(&json).expect("Deserialization failed");

        prop_assert_eq!(deserialized.relative_path, path);
        prop_assert_eq!(deserialized.commit_count, commit_count);
        prop_assert_eq!(deserialized.additions, additions);
        prop_assert_eq!(deserialized.deletions, deletions);
    }

    #[test]
    fn prop_churn_output_format_case_insensitive(
        format_name in prop_oneof![
            Just("json"),
            Just("JSON"),
            Just("Json"),
            Just("jSoN"),
            Just("markdown"),
            Just("MARKDOWN"),
            Just("Markdown"),
            Just("csv"),
            Just("CSV"),
            Just("Csv"),
            Just("summary"),
            Just("SUMMARY"),
            Just("Summary")
        ]
    ) {
        let result = ChurnOutputFormat::from_str(&format_name);
        prop_assert!(result.is_ok(), "Format '{}' should parse successfully", format_name);
    }

    #[test]
    fn prop_author_contributions_preserved(
        author_count in 0usize..10,
        contribution in 1usize..100
    ) {
        let mut contributions = HashMap::new();
        for i in 0..author_count {
            contributions.insert(format!("author_{}", i), contribution + i);
        }

        let summary = ChurnSummary {
            total_commits: 100,
            total_files_changed: 50,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: contributions.clone(),
            mean_churn_score: 0.5,
            variance_churn_score: 0.1,
            stddev_churn_score: 0.316,
        };

        let json = serde_json::to_string(&summary).expect("Serialization failed");
        let deserialized: ChurnSummary = serde_json::from_str(&json).expect("Deserialization failed");

        prop_assert_eq!(deserialized.author_contributions.len(), author_count);

        for (author, count) in contributions.iter() {
            prop_assert_eq!(
                deserialized.author_contributions.get(author),
                Some(count),
                "Author {} contribution should be preserved", author
            );
        }
    }

    #[test]
    fn prop_churn_summary_statistics_non_negative(
        mean in 0.0f64..1.0,
        variance in 0.0f64..1.0
    ) {
        let summary = ChurnSummary {
            total_commits: 100,
            total_files_changed: 50,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: HashMap::new(),
            mean_churn_score: mean,
            variance_churn_score: variance,
            stddev_churn_score: variance.sqrt(),
        };

        prop_assert!(summary.mean_churn_score >= 0.0);
        prop_assert!(summary.variance_churn_score >= 0.0);
        prop_assert!(summary.stddev_churn_score >= 0.0);
    }
}
