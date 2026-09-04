#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for churn formatter
    //!
    //! These tests exercise the markdown formatting, path detection,
    //! and all helper functions with comprehensive edge cases.

    use super::*;
    use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
    use chrono::Utc;
    use std::path::PathBuf;

    // ============================================================================
    // Test Fixtures
    // ============================================================================

    fn create_empty_summary() -> ChurnSummary {
        ChurnSummary {
            total_commits: 0,
            total_files_changed: 0,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: std::collections::BTreeMap::new(),
            mean_churn_score: 0.0,
            variance_churn_score: 0.0,
            stddev_churn_score: 0.0,
        }
    }

    fn create_populated_summary() -> ChurnSummary {
        let mut author_contributions = std::collections::BTreeMap::new();
        author_contributions.insert("alice".to_string(), 50);
        author_contributions.insert("bob".to_string(), 30);
        author_contributions.insert("charlie".to_string(), 20);

        ChurnSummary {
            total_commits: 100,
            total_files_changed: 25,
            hotspot_files: vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
            stable_files: vec![PathBuf::from("src/utils.rs")],
            author_contributions,
            mean_churn_score: 0.45,
            variance_churn_score: 0.04,
            stddev_churn_score: 0.2,
        }
    }

    fn create_test_file_metrics(
        path: &str,
        commit_count: usize,
        churn_score: f32,
    ) -> FileChurnMetrics {
        let now = Utc::now();
        FileChurnMetrics {
            path: PathBuf::from(path),
            relative_path: path.to_string(),
            commit_count,
            unique_authors: vec!["dev1".to_string(), "dev2".to_string()],
            additions: 100,
            deletions: 50,
            churn_score,
            last_modified: now,
            first_seen: now,
        }
    }

    fn create_test_analysis(files: Vec<FileChurnMetrics>) -> CodeChurnAnalysis {
        let summary = if files.is_empty() {
            create_empty_summary()
        } else {
            create_populated_summary()
        };

        CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("/test/repo"),
            files,
            summary,
        }
    }

    // ============================================================================
    // format_churn_markdown Tests
    // ============================================================================

    mod format_churn_markdown_tests {
        use super::*;

        #[test]
        fn test_format_empty_analysis() {
            let analysis = create_test_analysis(vec![]);
            let result = format_churn_markdown(&analysis).unwrap();

            assert!(result.contains("# Code Churn Analysis Report"));
            assert!(result.contains("Analysis Period: 30 days"));
            assert!(result.contains("## Summary Statistics"));
        }

        #[test]
        fn test_format_with_files() {
            let files = vec![
                create_test_file_metrics("src/main.rs", 20, 0.85),
                create_test_file_metrics("src/lib.rs", 15, 0.65),
            ];
            let analysis = create_test_analysis(files);
            let result = format_churn_markdown(&analysis).unwrap();

            assert!(result.contains("# Code Churn Analysis Report"));
            assert!(result.contains("## File Churn Details"));
            assert!(result.contains("src/main.rs"));
            assert!(result.contains("src/lib.rs"));
        }

        #[test]
        fn test_format_includes_header() {
            let analysis = create_test_analysis(vec![]);
            let result = format_churn_markdown(&analysis).unwrap();

            assert!(result.contains("Generated:"));
            assert!(result.contains("Repository:"));
        }

        #[test]
        fn test_format_with_author_contributions() {
            let files = vec![create_test_file_metrics("src/main.rs", 20, 0.85)];
            let analysis = create_test_analysis(files);
            let result = format_churn_markdown(&analysis).unwrap();

            assert!(result.contains("## Author Contributions"));
            assert!(result.contains("alice"));
            assert!(result.contains("bob"));
            assert!(result.contains("charlie"));
        }

        #[test]
        fn test_format_sorts_files_by_churn_score() {
            let files = vec![
                create_test_file_metrics("low.rs", 5, 0.2),
                create_test_file_metrics("high.rs", 30, 0.9),
                create_test_file_metrics("medium.rs", 15, 0.5),
            ];
            let analysis = create_test_analysis(files);
            let result = format_churn_markdown(&analysis).unwrap();

            let high_pos = result.find("high.rs").unwrap();
            let medium_pos = result.find("medium.rs").unwrap();
            let low_pos = result.find("low.rs").unwrap();

            assert!(
                high_pos < medium_pos,
                "high.rs should appear before medium.rs"
            );
            assert!(
                medium_pos < low_pos,
                "medium.rs should appear before low.rs"
            );
        }

        #[test]
        fn test_format_limits_to_top_20_files() {
            let files: Vec<FileChurnMetrics> = (0..25)
                .map(|i| {
                    create_test_file_metrics(&format!("file{}.rs", i), i + 1, (i as f32) / 25.0)
                })
                .collect();

            let analysis = create_test_analysis(files);
            let result = format_churn_markdown(&analysis).unwrap();

            let file_rows: Vec<&str> = result
                .lines()
                .filter(|line| line.starts_with("| file"))
                .collect();

            assert!(file_rows.len() <= 20, "Should have at most 20 file rows");
        }
    }

    // ============================================================================
    // write_markdown_summary_table Tests
    // ============================================================================

    mod write_markdown_summary_table_tests {
        use super::*;

        #[test]
        fn test_empty_summary_table() {
            let summary = create_empty_summary();
            let mut output = String::new();

            write_markdown_summary_table(&mut output, &summary).unwrap();

            assert!(output.contains("## Summary Statistics"));
            assert!(output.contains("| Total Commits | 0 |"));
            assert!(output.contains("| Files Changed | 0 |"));
        }

        #[test]
        fn test_populated_summary_table() {
            let summary = create_populated_summary();
            let mut output = String::new();

            write_markdown_summary_table(&mut output, &summary).unwrap();

            assert!(output.contains("| Total Commits | 100 |"));
            assert!(output.contains("| Files Changed | 25 |"));
            assert!(output.contains("| Hotspot Files | 2 |"));
            assert!(output.contains("| Stable Files | 1 |"));
            assert!(output.contains("| Contributing Authors | 3 |"));
        }

        #[test]
        fn test_table_has_headers() {
            let summary = create_empty_summary();
            let mut output = String::new();

            write_markdown_summary_table(&mut output, &summary).unwrap();

            assert!(output.contains("| Metric | Value |"));
            assert!(output.contains("|--------|-------|"));
        }
    }

    // Path detection and edge case tests are in churn_formatter_tests_path.rs
}
