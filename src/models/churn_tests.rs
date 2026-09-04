// Unit tests for churn analysis types - included via include!()

#[test]
fn test_file_churn_metrics_calculate_score() {
    let mut metrics = FileChurnMetrics {
        path: PathBuf::from("test.rs"),
        relative_path: "test.rs".to_string(),
        commit_count: 10,
        unique_authors: vec!["author1".to_string()],
        additions: 100,
        deletions: 50,
        churn_score: 0.0,
        last_modified: Utc::now(),
        first_seen: Utc::now(),
    };

    metrics.calculate_churn_score(20, 300);
    assert!(metrics.churn_score > 0.0);
    assert!(metrics.churn_score <= 1.0);

    // Test with max values
    metrics.commit_count = 20;
    metrics.additions = 150;
    metrics.deletions = 150;
    metrics.calculate_churn_score(20, 300);
    assert_eq!(metrics.churn_score, 1.0);
}

#[test]
fn test_file_churn_metrics_zero_max() {
    let mut metrics = FileChurnMetrics {
        path: PathBuf::from("test.rs"),
        relative_path: "test.rs".to_string(),
        commit_count: 10,
        unique_authors: vec![],
        additions: 100,
        deletions: 50,
        churn_score: 0.0,
        last_modified: Utc::now(),
        first_seen: Utc::now(),
    };

    metrics.calculate_churn_score(0, 0);
    assert_eq!(metrics.churn_score, 0.0);
}

#[test]
fn test_churn_output_format_from_str() {
    assert_eq!(
        ChurnOutputFormat::from_str("json").unwrap(),
        ChurnOutputFormat::Json
    );
    assert_eq!(
        ChurnOutputFormat::from_str("JSON").unwrap(),
        ChurnOutputFormat::Json
    );
    assert_eq!(
        ChurnOutputFormat::from_str("markdown").unwrap(),
        ChurnOutputFormat::Markdown
    );
    assert_eq!(
        ChurnOutputFormat::from_str("csv").unwrap(),
        ChurnOutputFormat::Csv
    );
    assert_eq!(
        ChurnOutputFormat::from_str("summary").unwrap(),
        ChurnOutputFormat::Summary
    );

    assert!(ChurnOutputFormat::from_str("invalid").is_err());
}

#[test]
fn test_code_churn_analysis_creation() {
    let analysis = CodeChurnAnalysis {
        generated_at: Utc::now(),
        period_days: 30,
        repository_root: PathBuf::from("/test/repo"),
        files: vec![],
        summary: ChurnSummary {
            total_commits: 100,
            total_files_changed: 50,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: std::collections::BTreeMap::new(),
            mean_churn_score: 0.0,
            variance_churn_score: 0.0,
            stddev_churn_score: 0.0,
        },
    };

    assert_eq!(analysis.period_days, 30);
    assert_eq!(analysis.summary.total_commits, 100);
    assert_eq!(analysis.summary.total_files_changed, 50);
}

#[test]
fn test_churn_summary_with_data() {
    let mut author_contributions = std::collections::BTreeMap::new();
    author_contributions.insert("author1".to_string(), 50);
    author_contributions.insert("author2".to_string(), 30);

    let summary = ChurnSummary {
        total_commits: 80,
        total_files_changed: 25,
        hotspot_files: vec![PathBuf::from("hot1.rs"), PathBuf::from("hot2.rs")],
        stable_files: vec![PathBuf::from("stable1.rs")],
        author_contributions,
        mean_churn_score: 0.0,
        variance_churn_score: 0.0,
        stddev_churn_score: 0.0,
    };

    assert_eq!(summary.total_commits, 80);
    assert_eq!(summary.hotspot_files.len(), 2);
    assert_eq!(summary.stable_files.len(), 1);
    assert_eq!(summary.author_contributions.get("author1"), Some(&50));
}

#[test]
fn test_serialization() {
    let metrics = FileChurnMetrics {
        path: PathBuf::from("test.rs"),
        relative_path: "test.rs".to_string(),
        commit_count: 5,
        unique_authors: vec!["dev".to_string()],
        additions: 50,
        deletions: 20,
        churn_score: 0.5,
        last_modified: Utc::now(),
        first_seen: Utc::now(),
    };

    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: FileChurnMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.commit_count, metrics.commit_count);
    assert_eq!(deserialized.churn_score, metrics.churn_score);
}

/// CRUX-07 leg c: `author_contributions` reaches the JSON document in author
/// order, so two runs over an unchanged repository produce the same bytes.
/// A `HashMap` here randomised key order per process.
#[test]
fn author_contributions_serialize_in_author_order() {
    let mut contributions = BTreeMap::new();
    for author in ["zoe", "ann", "mel", "bob"] {
        contributions.insert(author.to_string(), 1);
    }
    let summary = ChurnSummary {
        total_commits: 4,
        total_files_changed: 1,
        hotspot_files: vec![],
        stable_files: vec![],
        author_contributions: contributions,
        mean_churn_score: 0.0,
        variance_churn_score: 0.0,
        stddev_churn_score: 0.0,
    };
    let json = serde_json::to_string(&summary.author_contributions)
        .expect("a BTreeMap of counts must serialize");
    assert_eq!(json, r#"{"ann":1,"bob":1,"mel":1,"zoe":1}"#);
}
