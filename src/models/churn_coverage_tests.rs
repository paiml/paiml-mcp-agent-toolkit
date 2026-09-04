// Coverage tests for FileChurnMetrics and ChurnSummary - included via include!()
//
// These tests exercise all edge cases, boundary conditions, and
// ensure comprehensive coverage of the churn analysis data structures.

// ============================================================================
// FileChurnMetrics Tests
// ============================================================================

mod file_churn_metrics_tests {
    use super::*;

    fn create_default_metrics() -> FileChurnMetrics {
        FileChurnMetrics {
            path: PathBuf::from("src/lib.rs"),
            relative_path: "src/lib.rs".to_string(),
            commit_count: 0,
            unique_authors: vec![],
            additions: 0,
            deletions: 0,
            churn_score: 0.0,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        }
    }

    #[test]
    fn test_calculate_churn_score_zero_everything() {
        let mut metrics = create_default_metrics();
        metrics.calculate_churn_score(0, 0);
        assert_eq!(metrics.churn_score, 0.0);
    }

    #[test]
    fn test_calculate_churn_score_zero_max_commits_only() {
        let mut metrics = create_default_metrics();
        metrics.additions = 100;
        metrics.deletions = 50;
        metrics.calculate_churn_score(0, 300);
        // With zero max_commits, commit_factor is 0.0
        // change_factor = 150/300 = 0.5
        // score = 0.0 * 0.6 + 0.5 * 0.4 = 0.2
        assert!((metrics.churn_score - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_calculate_churn_score_zero_max_changes_only() {
        let mut metrics = create_default_metrics();
        metrics.commit_count = 10;
        metrics.calculate_churn_score(20, 0);
        // commit_factor = 10/20 = 0.5
        // change_factor = 0.0 (max_changes is 0)
        // score = 0.5 * 0.6 + 0.0 * 0.4 = 0.3
        assert!((metrics.churn_score - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_calculate_churn_score_exact_max() {
        let mut metrics = create_default_metrics();
        metrics.commit_count = 100;
        metrics.additions = 500;
        metrics.deletions = 500;
        metrics.calculate_churn_score(100, 1000);
        // commit_factor = 1.0, change_factor = 1.0
        // score = 1.0 * 0.6 + 1.0 * 0.4 = 1.0
        assert_eq!(metrics.churn_score, 1.0);
    }

    #[test]
    fn test_calculate_churn_score_exceeds_max_capped() {
        let mut metrics = create_default_metrics();
        metrics.commit_count = 200; // Exceeds max
        metrics.additions = 2000;
        metrics.deletions = 2000;
        metrics.calculate_churn_score(100, 1000);
        // commit_factor = 200/100 = 2.0
        // change_factor = 4000/1000 = 4.0
        // raw_score = 2.0 * 0.6 + 4.0 * 0.4 = 1.2 + 1.6 = 2.8
        // capped to 1.0
        assert_eq!(metrics.churn_score, 1.0);
    }

    #[test]
    fn test_calculate_churn_score_weighting_60_40() {
        let mut metrics = create_default_metrics();
        // Set up so commit_factor = 1.0 and change_factor = 0.0
        metrics.commit_count = 50;
        metrics.additions = 0;
        metrics.deletions = 0;
        metrics.calculate_churn_score(50, 100);
        // score = 1.0 * 0.6 + 0.0 * 0.4 = 0.6
        assert!((metrics.churn_score - 0.6).abs() < 0.001);

        // Now opposite: commit_factor = 0.0, change_factor = 1.0
        metrics.commit_count = 0;
        metrics.additions = 50;
        metrics.deletions = 50;
        metrics.calculate_churn_score(50, 100);
        // score = 0.0 * 0.6 + 1.0 * 0.4 = 0.4
        assert!((metrics.churn_score - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_calculate_churn_score_fractional_values() {
        let mut metrics = create_default_metrics();
        metrics.commit_count = 7;
        metrics.additions = 33;
        metrics.deletions = 17;
        metrics.calculate_churn_score(14, 100);
        // commit_factor = 7/14 = 0.5
        // change_factor = 50/100 = 0.5
        // score = 0.5 * 0.6 + 0.5 * 0.4 = 0.3 + 0.2 = 0.5
        assert!((metrics.churn_score - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_file_churn_metrics_with_multiple_authors() {
        let metrics = FileChurnMetrics {
            path: PathBuf::from("src/important.rs"),
            relative_path: "src/important.rs".to_string(),
            commit_count: 50,
            unique_authors: vec![
                "alice".to_string(),
                "bob".to_string(),
                "charlie".to_string(),
            ],
            additions: 1000,
            deletions: 200,
            churn_score: 0.8,
            last_modified: Utc::now(),
            first_seen: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        };

        assert_eq!(metrics.unique_authors.len(), 3);
        assert!(metrics.unique_authors.contains(&"alice".to_string()));
    }

    #[test]
    fn test_file_churn_metrics_serialization_full() {
        let now = Utc::now();
        let metrics = FileChurnMetrics {
            path: PathBuf::from("complex/path/to/file.rs"),
            relative_path: "complex/path/to/file.rs".to_string(),
            commit_count: 42,
            unique_authors: vec!["dev1".to_string(), "dev2".to_string()],
            additions: 500,
            deletions: 250,
            churn_score: 0.75,
            last_modified: now,
            first_seen: now,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: FileChurnMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.path, metrics.path);
        assert_eq!(deserialized.relative_path, metrics.relative_path);
        assert_eq!(deserialized.commit_count, metrics.commit_count);
        assert_eq!(deserialized.unique_authors.len(), 2);
        assert_eq!(deserialized.additions, metrics.additions);
        assert_eq!(deserialized.deletions, metrics.deletions);
        assert!((deserialized.churn_score - metrics.churn_score).abs() < 0.0001);
    }

    #[test]
    fn test_file_churn_metrics_clone() {
        let metrics = create_default_metrics();
        let cloned = metrics.clone();

        assert_eq!(cloned.path, metrics.path);
        assert_eq!(cloned.relative_path, metrics.relative_path);
        assert_eq!(cloned.churn_score, metrics.churn_score);
    }
}

// ============================================================================
// ChurnSummary Tests
// ============================================================================

mod churn_summary_tests {
    use super::*;

    #[test]
    fn test_churn_summary_empty() {
        let summary = ChurnSummary {
            total_commits: 0,
            total_files_changed: 0,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: std::collections::BTreeMap::new(),
            mean_churn_score: 0.0,
            variance_churn_score: 0.0,
            stddev_churn_score: 0.0,
        };

        assert_eq!(summary.total_commits, 0);
        assert!(summary.hotspot_files.is_empty());
        assert!(summary.author_contributions.is_empty());
    }

    #[test]
    fn test_churn_summary_with_hotspots() {
        let summary = ChurnSummary {
            total_commits: 150,
            total_files_changed: 45,
            hotspot_files: vec![
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/lib.rs"),
                PathBuf::from("src/core.rs"),
            ],
            stable_files: vec![PathBuf::from("src/utils.rs")],
            author_contributions: std::collections::BTreeMap::new(),
            mean_churn_score: 0.65,
            variance_churn_score: 0.04,
            stddev_churn_score: 0.2,
        };

        assert_eq!(summary.hotspot_files.len(), 3);
        assert_eq!(summary.stable_files.len(), 1);
        assert!((summary.mean_churn_score - 0.65).abs() < 0.001);
    }

    #[test]
    fn test_churn_summary_author_contributions() {
        let mut contributions = std::collections::BTreeMap::new();
        contributions.insert("lead_dev".to_string(), 100);
        contributions.insert("junior_dev".to_string(), 25);
        contributions.insert("reviewer".to_string(), 10);

        let summary = ChurnSummary {
            total_commits: 135,
            total_files_changed: 80,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: contributions,
            mean_churn_score: 0.5,
            variance_churn_score: 0.1,
            stddev_churn_score: 0.316,
        };

        assert_eq!(summary.author_contributions.len(), 3);
        assert_eq!(summary.author_contributions.get("lead_dev"), Some(&100));
        assert_eq!(summary.author_contributions.get("junior_dev"), Some(&25));
    }

    #[test]
    fn test_churn_summary_statistics() {
        let summary = ChurnSummary {
            total_commits: 200,
            total_files_changed: 100,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: std::collections::BTreeMap::new(),
            mean_churn_score: 0.45,
            variance_churn_score: 0.0225, // 0.15^2
            stddev_churn_score: 0.15,
        };

        // Verify statistical relationship
        let calculated_stddev = summary.variance_churn_score.sqrt();
        assert!((calculated_stddev - summary.stddev_churn_score).abs() < 0.001);
    }

    #[test]
    fn test_churn_summary_serialization() {
        let mut contributions = std::collections::BTreeMap::new();
        contributions.insert("dev".to_string(), 50);

        let summary = ChurnSummary {
            total_commits: 100,
            total_files_changed: 30,
            hotspot_files: vec![PathBuf::from("hot.rs")],
            stable_files: vec![PathBuf::from("stable.rs")],
            author_contributions: contributions,
            mean_churn_score: 0.6,
            variance_churn_score: 0.04,
            stddev_churn_score: 0.2,
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: ChurnSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_commits, summary.total_commits);
        assert_eq!(deserialized.hotspot_files.len(), 1);
        assert_eq!(deserialized.author_contributions.get("dev"), Some(&50));
    }
}
