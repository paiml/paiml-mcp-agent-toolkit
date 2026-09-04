#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
    use chrono::Utc;
    use proptest::prelude::*;
    use std::path::PathBuf;

    proptest! {
        #[test]
        fn prop_is_source_file_deterministic(path in "[a-z]{1,20}\\.[a-z]{1,5}") {
            let path = Path::new(&path);
            let result1 = is_source_file(path);
            let result2 = is_source_file(path);
            prop_assert_eq!(result1, result2, "is_source_file should be deterministic");
        }

        #[test]
        fn prop_rust_files_always_source(name in "[a-z]{1,20}") {
            let path_str = format!("src/{}.rs", name);
            let path = Path::new(&path_str);
            prop_assert!(
                is_source_file(path),
                "Any .rs file in src/ should be a source file"
            );
        }

        #[test]
        fn prop_test_directories_never_source(
            dir in prop_oneof![
                Just("tests"),
                Just("test"),
                Just("examples"),
                Just("benches"),
                Just("fixtures"),
                Just("__tests__")
            ],
            _name in "[a-z]{1,10}"
        ) {
            let path_str = format!("/{}/source.rs", dir);
            let path = Path::new(&path_str);
            prop_assert!(
                !is_source_file(path),
                "Files in {} directory should not be source files",
                dir
            );
        }

        #[test]
        fn prop_format_churn_markdown_always_returns_string(
            period_days in 1u32..365,
            total_commits in 0usize..10000
        ) {
            let summary = ChurnSummary {
                total_commits,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: std::collections::BTreeMap::new(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            };

            let analysis = CodeChurnAnalysis {
                generated_at: Utc::now(),
                period_days,
                repository_root: PathBuf::from("/test"),
                files: vec![],
                summary,
            };

            let result = format_churn_markdown(&analysis);
            prop_assert!(result.is_ok(), "format_churn_markdown should always succeed");

            let output = result.unwrap();
            prop_assert!(!output.is_empty(), "Output should not be empty");
            prop_assert!(
                output.contains("# Code Churn Analysis Report"),
                "Output should contain header"
            );
        }

        #[test]
        fn prop_summary_table_contains_accurate_counts(
            commits in 0usize..10000,
            files in 0usize..1000,
            hotspots in 0usize..50,
            stable in 0usize..50,
            authors in 0usize..100
        ) {
            let mut author_contributions = std::collections::BTreeMap::new();
            for i in 0..authors {
                author_contributions.insert(format!("author_{}", i), 10);
            }

            let summary = ChurnSummary {
                total_commits: commits,
                total_files_changed: files,
                hotspot_files: (0..hotspots).map(|i| PathBuf::from(format!("hot{}.rs", i))).collect(),
                stable_files: (0..stable).map(|i| PathBuf::from(format!("stable{}.rs", i))).collect(),
                author_contributions,
                mean_churn_score: 0.5,
                variance_churn_score: 0.1,
                stddev_churn_score: 0.316,
            };

            let mut output = String::new();
            write_markdown_summary_table(&mut output, &summary).unwrap();

            prop_assert!(
                output.contains(&format!("| Total Commits | {} |", commits)),
                "Output should contain correct commit count"
            );
            prop_assert!(
                output.contains(&format!("| Files Changed | {} |", files)),
                "Output should contain correct files count"
            );
            prop_assert!(
                output.contains(&format!("| Hotspot Files | {} |", hotspots)),
                "Output should contain correct hotspot count"
            );
            prop_assert!(
                output.contains(&format!("| Stable Files | {} |", stable)),
                "Output should contain correct stable count"
            );
            prop_assert!(
                output.contains(&format!("| Contributing Authors | {} |", authors)),
                "Output should contain correct author count"
            );
        }

        #[test]
        fn prop_file_extensions_preserved_in_output(
            ext in prop_oneof![
                Just("rs"),
                Just("js"),
                Just("ts"),
                Just("py"),
                Just("java"),
                Just("go")
            ]
        ) {
            let file_path = format!("src/file.{}", ext);
            let metrics = FileChurnMetrics {
                path: PathBuf::from(&file_path),
                relative_path: file_path.clone(),
                commit_count: 10,
                unique_authors: vec!["dev".to_string()],
                additions: 100,
                deletions: 50,
                churn_score: 0.5,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            };

            let summary = ChurnSummary {
                total_commits: 10,
                total_files_changed: 1,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: std::collections::BTreeMap::new(),
                mean_churn_score: 0.5,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            };

            let analysis = CodeChurnAnalysis {
                generated_at: Utc::now(),
                period_days: 30,
                repository_root: PathBuf::from("/test"),
                files: vec![metrics],
                summary,
            };

            let result = format_churn_markdown(&analysis).unwrap();
            prop_assert!(
                result.contains(&format!(".{}", ext)),
                "Output should contain the file extension .{}",
                ext
            );
        }
    }
}
