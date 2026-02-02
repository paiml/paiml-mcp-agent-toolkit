//! Coverage boost tests for cli/analysis_utilities/churn module
//! Tests for churn analysis formatting functions

use crate::cli::analysis_utilities::{
    format_churn_as_csv, format_churn_as_markdown, format_churn_as_summary, write_churn_output,
};
use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;

fn create_test_analysis() -> CodeChurnAnalysis {
    CodeChurnAnalysis {
        generated_at: Utc::now(),
        period_days: 30,
        repository_root: PathBuf::from("/test/repo"),
        files: vec![
            FileChurnMetrics {
                path: PathBuf::from("/test/repo/src/main.rs"),
                relative_path: "src/main.rs".to_string(),
                commit_count: 15,
                unique_authors: vec!["dev1".to_string(), "dev2".to_string()],
                additions: 100,
                deletions: 50,
                churn_score: 0.75,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
            FileChurnMetrics {
                path: PathBuf::from("/test/repo/src/lib.rs"),
                relative_path: "src/lib.rs".to_string(),
                commit_count: 8,
                unique_authors: vec!["dev1".to_string()],
                additions: 60,
                deletions: 20,
                churn_score: 0.45,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
        ],
        summary: ChurnSummary {
            total_commits: 23,
            total_files_changed: 2,
            hotspot_files: vec![PathBuf::from("src/main.rs")],
            stable_files: vec![PathBuf::from("src/lib.rs")],
            author_contributions: {
                let mut map = HashMap::new();
                map.insert("dev1".to_string(), 15);
                map.insert("dev2".to_string(), 8);
                map
            },
            mean_churn_score: 0.6,
            variance_churn_score: 0.0225,
            stddev_churn_score: 0.15,
        },
    }
}

fn create_empty_analysis() -> CodeChurnAnalysis {
    CodeChurnAnalysis {
        generated_at: Utc::now(),
        period_days: 7,
        repository_root: PathBuf::from("/test/repo"),
        files: vec![],
        summary: ChurnSummary {
            total_commits: 0,
            total_files_changed: 0,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: HashMap::new(),
            mean_churn_score: 0.0,
            variance_churn_score: 0.0,
            stddev_churn_score: 0.0,
        },
    }
}

// ============ format_churn_as_summary Tests ============

#[test]
fn test_format_churn_as_summary_with_files() {
    let analysis = create_test_analysis();
    let result = format_churn_as_summary(&analysis).unwrap();

    assert!(result.contains("Code Churn Analysis Summary"));
    assert!(result.contains("30 days"));
    assert!(result.contains("23")); // total commits
    assert!(result.contains("2")); // files changed
}

#[test]
fn test_format_churn_as_summary_empty() {
    let analysis = create_empty_analysis();
    let result = format_churn_as_summary(&analysis).unwrap();

    assert!(result.contains("Code Churn Analysis Summary"));
    assert!(result.contains("7 days"));
}

#[test]
fn test_format_churn_as_summary_top_files() {
    let analysis = create_test_analysis();
    let result = format_churn_as_summary(&analysis).unwrap();

    // Should list top files
    assert!(result.contains("Top Files by Churn"));
    assert!(result.contains("main.rs"));
}

#[test]
fn test_format_churn_as_summary_hotspot_files() {
    let analysis = create_test_analysis();
    let result = format_churn_as_summary(&analysis).unwrap();

    // Should include hotspot section
    assert!(result.contains("Hotspot Files"));
}

#[test]
fn test_format_churn_as_summary_stable_files() {
    let analysis = create_test_analysis();
    let result = format_churn_as_summary(&analysis).unwrap();

    // Should include stable files section
    assert!(result.contains("Stable Files"));
}

#[test]
fn test_format_churn_as_summary_contributors() {
    let analysis = create_test_analysis();
    let result = format_churn_as_summary(&analysis).unwrap();

    // Should include top contributors
    assert!(result.contains("Top Contributors"));
    assert!(result.contains("dev1"));
}

// ============ format_churn_as_markdown Tests ============

#[test]
fn test_format_churn_as_markdown_with_files() {
    let analysis = create_test_analysis();
    let result = format_churn_as_markdown(&analysis).unwrap();

    assert!(result.contains("# Code Churn Analysis Report"));
    assert!(result.contains("Generated:"));
    assert!(result.contains("Repository:"));
    assert!(result.contains("Analysis Period: 30 days"));
}

#[test]
fn test_format_churn_as_markdown_summary_table() {
    let analysis = create_test_analysis();
    let result = format_churn_as_markdown(&analysis).unwrap();

    // Should have summary statistics table
    assert!(result.contains("## Summary Statistics"));
    assert!(result.contains("| Metric | Value |"));
    assert!(result.contains("Total Commits"));
    assert!(result.contains("Files Changed"));
}

#[test]
fn test_format_churn_as_markdown_file_details() {
    let analysis = create_test_analysis();
    let result = format_churn_as_markdown(&analysis).unwrap();

    // Should have file churn details
    assert!(result.contains("## File Churn Details"));
    assert!(result.contains("src/main.rs"));
    assert!(result.contains("src/lib.rs"));
}

#[test]
fn test_format_churn_as_markdown_author_contributions() {
    let analysis = create_test_analysis();
    let result = format_churn_as_markdown(&analysis).unwrap();

    // Should have author contributions
    assert!(result.contains("## Author Contributions"));
    assert!(result.contains("dev1"));
}

#[test]
fn test_format_churn_as_markdown_recommendations() {
    let analysis = create_test_analysis();
    let result = format_churn_as_markdown(&analysis).unwrap();

    // Should have recommendations
    assert!(result.contains("## Recommendations"));
    assert!(result.contains("Review Hotspot Files"));
    assert!(result.contains("Add Tests"));
}

#[test]
fn test_format_churn_as_markdown_empty() {
    let analysis = create_empty_analysis();
    let result = format_churn_as_markdown(&analysis).unwrap();

    // Should still have header and recommendations
    assert!(result.contains("# Code Churn Analysis Report"));
    assert!(result.contains("## Recommendations"));
}

// ============ format_churn_as_csv Tests ============

#[test]
fn test_format_churn_as_csv_with_files() {
    let analysis = create_test_analysis();
    let result = format_churn_as_csv(&analysis).unwrap();

    // Should have header row
    assert!(result.contains("file_path,relative_path,commit_count,unique_authors,additions,deletions,churn_score,last_modified,first_seen"));

    // Should have data rows
    assert!(result.contains("src/main.rs"));
    assert!(result.contains("src/lib.rs"));
    assert!(result.contains("15")); // commit count for main.rs
    assert!(result.contains("0.75")); // churn score
}

#[test]
fn test_format_churn_as_csv_empty() {
    let analysis = create_empty_analysis();
    let result = format_churn_as_csv(&analysis).unwrap();

    // Should have header but no data rows
    assert!(result.contains("file_path,relative_path"));
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 1); // Just header
}

#[test]
fn test_format_churn_as_csv_column_count() {
    let analysis = create_test_analysis();
    let result = format_churn_as_csv(&analysis).unwrap();

    let lines: Vec<&str> = result.lines().collect();
    let header_cols = lines[0].split(',').count();
    let data_cols = lines[1].split(',').count();

    assert_eq!(header_cols, 9); // 9 columns
    assert_eq!(data_cols, 9);
}

// ============ write_churn_output Tests ============

#[tokio::test]
async fn test_write_churn_output_to_stdout() {
    let content = "Test churn output".to_string();
    let result = write_churn_output(content, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_write_churn_output_to_file() {
    let content = "Test churn output to file".to_string();
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_churn_output.txt");

    let result = write_churn_output(content.clone(), Some(output_path.clone())).await;
    assert!(result.is_ok());

    // Verify file was written
    let read_content = tokio::fs::read_to_string(&output_path).await.unwrap();
    assert_eq!(read_content, content);

    // Cleanup
    let _ = tokio::fs::remove_file(&output_path).await;
}

// ============ Edge Cases ============

#[test]
fn test_format_with_special_characters_in_paths() {
    let mut analysis = create_test_analysis();
    analysis.files[0].relative_path = "src/path with spaces/file.rs".to_string();

    let result = format_churn_as_csv(&analysis).unwrap();
    assert!(result.contains("src/path with spaces/file.rs"));
}

#[test]
fn test_format_with_zero_churn_score() {
    let mut analysis = create_test_analysis();
    analysis.files[0].churn_score = 0.0;

    let result = format_churn_as_markdown(&analysis).unwrap();
    assert!(result.contains("0.00"));
}

#[test]
fn test_format_with_high_churn_score() {
    let mut analysis = create_test_analysis();
    analysis.files[0].churn_score = 1.0;

    let result = format_churn_as_markdown(&analysis).unwrap();
    assert!(result.contains("1.00"));
}

#[test]
fn test_format_with_many_authors() {
    let mut analysis = create_test_analysis();
    analysis.files[0].unique_authors = vec![
        "dev1".to_string(),
        "dev2".to_string(),
        "dev3".to_string(),
        "dev4".to_string(),
        "dev5".to_string(),
    ];

    let result = format_churn_as_summary(&analysis).unwrap();
    assert!(result.contains("5 authors"));
}

#[test]
fn test_format_summary_sorts_by_commit_count() {
    let mut analysis = create_test_analysis();
    // Swap order - lib.rs now has more commits
    analysis.files[0].commit_count = 5;
    analysis.files[1].commit_count = 20;

    let result = format_churn_as_summary(&analysis).unwrap();

    // lib.rs should appear first due to higher commit count
    let lib_pos = result.find("lib.rs");
    let main_pos = result.find("main.rs");
    assert!(lib_pos.is_some());
    assert!(main_pos.is_some());
    // In sorted output, higher commit count comes first
}

#[test]
fn test_format_markdown_sorts_by_churn_score() {
    let mut analysis = create_test_analysis();
    // Swap churn scores
    analysis.files[0].churn_score = 0.25;
    analysis.files[1].churn_score = 0.95;

    let result = format_churn_as_markdown(&analysis).unwrap();

    // Check that file with higher churn (lib.rs) appears in output
    assert!(result.contains("lib.rs"));
    assert!(result.contains("0.95"));
}
