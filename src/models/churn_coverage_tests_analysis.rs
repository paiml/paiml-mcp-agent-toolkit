// Coverage tests for CodeChurnAnalysis and ChurnOutputFormat - included via include!()

// ============================================================================
// CodeChurnAnalysis Tests
// ============================================================================

mod code_churn_analysis_tests {
    use super::*;

    fn create_test_analysis() -> CodeChurnAnalysis {
        CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("/project"),
            files: vec![],
            summary: ChurnSummary {
                total_commits: 0,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: std::collections::BTreeMap::new(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        }
    }

    #[test]
    fn test_code_churn_analysis_period_days() {
        let mut analysis = create_test_analysis();

        // Test various period days
        analysis.period_days = 7;
        assert_eq!(analysis.period_days, 7);

        analysis.period_days = 90;
        assert_eq!(analysis.period_days, 90);

        analysis.period_days = 365;
        assert_eq!(analysis.period_days, 365);
    }

    #[test]
    fn test_code_churn_analysis_with_files() {
        let now = Utc::now();
        let files = vec![
            FileChurnMetrics {
                path: PathBuf::from("src/main.rs"),
                relative_path: "src/main.rs".to_string(),
                commit_count: 10,
                unique_authors: vec!["dev".to_string()],
                additions: 100,
                deletions: 50,
                churn_score: 0.7,
                last_modified: now,
                first_seen: now,
            },
            FileChurnMetrics {
                path: PathBuf::from("src/lib.rs"),
                relative_path: "src/lib.rs".to_string(),
                commit_count: 5,
                unique_authors: vec!["dev".to_string()],
                additions: 50,
                deletions: 20,
                churn_score: 0.4,
                last_modified: now,
                first_seen: now,
            },
        ];

        let analysis = CodeChurnAnalysis {
            generated_at: now,
            period_days: 30,
            repository_root: PathBuf::from("/project"),
            files,
            summary: ChurnSummary {
                total_commits: 15,
                total_files_changed: 2,
                hotspot_files: vec![PathBuf::from("src/main.rs")],
                stable_files: vec![],
                author_contributions: std::collections::BTreeMap::new(),
                mean_churn_score: 0.55,
                variance_churn_score: 0.0225,
                stddev_churn_score: 0.15,
            },
        };

        assert_eq!(analysis.files.len(), 2);
        assert_eq!(analysis.summary.total_commits, 15);
    }

    #[test]
    fn test_code_churn_analysis_serialization() {
        let now = Utc::now();
        let mut contributions = std::collections::BTreeMap::new();
        contributions.insert("alice".to_string(), 30);

        let analysis = CodeChurnAnalysis {
            generated_at: now,
            period_days: 14,
            repository_root: PathBuf::from("/test/project"),
            files: vec![FileChurnMetrics {
                path: PathBuf::from("test.rs"),
                relative_path: "test.rs".to_string(),
                commit_count: 5,
                unique_authors: vec!["alice".to_string()],
                additions: 25,
                deletions: 10,
                churn_score: 0.35,
                last_modified: now,
                first_seen: now,
            }],
            summary: ChurnSummary {
                total_commits: 5,
                total_files_changed: 1,
                hotspot_files: vec![],
                stable_files: vec![PathBuf::from("test.rs")],
                author_contributions: contributions,
                mean_churn_score: 0.35,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: CodeChurnAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.period_days, 14);
        assert_eq!(deserialized.files.len(), 1);
        assert_eq!(deserialized.summary.total_commits, 5);
    }
}

// ============================================================================
// ChurnOutputFormat Tests
// ============================================================================

mod churn_output_format_tests {
    use super::*;

    #[test]
    fn test_from_str_lowercase() {
        assert_eq!(
            ChurnOutputFormat::from_str("json").unwrap(),
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
    }

    #[test]
    fn test_from_str_uppercase() {
        assert_eq!(
            ChurnOutputFormat::from_str("JSON").unwrap(),
            ChurnOutputFormat::Json
        );
        assert_eq!(
            ChurnOutputFormat::from_str("MARKDOWN").unwrap(),
            ChurnOutputFormat::Markdown
        );
        assert_eq!(
            ChurnOutputFormat::from_str("CSV").unwrap(),
            ChurnOutputFormat::Csv
        );
        assert_eq!(
            ChurnOutputFormat::from_str("SUMMARY").unwrap(),
            ChurnOutputFormat::Summary
        );
    }

    #[test]
    fn test_from_str_mixed_case() {
        assert_eq!(
            ChurnOutputFormat::from_str("Json").unwrap(),
            ChurnOutputFormat::Json
        );
        assert_eq!(
            ChurnOutputFormat::from_str("MarkDown").unwrap(),
            ChurnOutputFormat::Markdown
        );
        assert_eq!(
            ChurnOutputFormat::from_str("CsV").unwrap(),
            ChurnOutputFormat::Csv
        );
        assert_eq!(
            ChurnOutputFormat::from_str("SuMmArY").unwrap(),
            ChurnOutputFormat::Summary
        );
    }

    #[test]
    fn test_from_str_invalid() {
        let result = ChurnOutputFormat::from_str("invalid");
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert!(err.contains("Invalid output format"));
        assert!(err.contains("invalid"));
    }

    #[test]
    fn test_from_str_empty() {
        let result = ChurnOutputFormat::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_whitespace() {
        // Should fail because we don't trim whitespace
        let result = ChurnOutputFormat::from_str(" json ");
        assert!(result.is_err());
    }

    #[test]
    fn test_churn_output_format_equality() {
        assert_eq!(ChurnOutputFormat::Json, ChurnOutputFormat::Json);
        assert_ne!(ChurnOutputFormat::Json, ChurnOutputFormat::Csv);
        assert_ne!(ChurnOutputFormat::Markdown, ChurnOutputFormat::Summary);
    }

    #[test]
    fn test_churn_output_format_clone() {
        let format = ChurnOutputFormat::Markdown;
        let cloned = format.clone();
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_churn_output_format_debug() {
        let format = ChurnOutputFormat::Json;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Json"));
    }
}
