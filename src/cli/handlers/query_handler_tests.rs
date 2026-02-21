#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handle_query_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create empty project
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/main.rs"), "").unwrap();

        let result = handle_query(
            "test".to_string(),
            10,
            None,
            None,
            None,
            None,
            project_path,
            QueryOutputFormat::Text,
            false,
            false,
            false,
            None,   // rank_by
            None,   // min_pagerank
            vec![], // include_project
            false,  // churn
            false,  // duplicates
            false,  // entropy
            false,  // faults
            false,  // coverage
            false,  // uncovered_only
            None,   // coverage_diff
            None,   // coverage_file
            false,  // coverage_gaps
            false,  // include_excluded
            None,   // definition_type
            false,  // code
            false,  // git_history
            false,  // regex
            false,  // literal
            false,  // raw
            false,  // case_sensitive
            false,  // ignore_case
            None,   // exclude
            None,   // exclude_file
            false,  // files_with_matches
            false,  // count
            None,   // after_context
            None,   // before_context
            None,   // context_lines
            false,  // ptx_flow
            false,  // ptx_diagnostics
            false,  // suggest_rename
            false,  // apply
            false,  // docs
            false,  // docs_only
        )
        .await;

        // Should not error, just find nothing
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_query_with_functions() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create project with a function
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/main.rs"),
            r#"
/// Handle errors in the API layer
fn handle_api_error(err: String) -> String {
    format!("Error: {}", err)
}

fn main() {
    println!("Hello");
}
"#,
        )
        .unwrap();

        let result = handle_query(
            "error handling".to_string(),
            10,
            None,
            None,
            None,
            None,
            project_path,
            QueryOutputFormat::Json,
            false,
            true, // Force rebuild
            false,
            None,   // rank_by
            None,   // min_pagerank
            vec![], // include_project
            false,  // churn
            false,  // duplicates
            false,  // entropy
            false,  // faults
            false,  // coverage
            false,  // uncovered_only
            None,   // coverage_diff
            None,   // coverage_file
            false,  // coverage_gaps
            false,  // include_excluded
            None,   // definition_type
            false,  // code
            false,  // git_history
            false,  // regex
            false,  // literal
            false,  // raw
            false,  // case_sensitive
            false,  // ignore_case
            None,   // exclude
            None,   // exclude_file
            false,  // files_with_matches
            false,  // count
            None,   // after_context
            None,   // before_context
            None,   // context_lines
            false,  // ptx_flow
            false,  // ptx_diagnostics
            false,  // suggest_rename
            false,  // apply
            false,  // docs
            false,  // docs_only
        )
        .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_classify_commit_type() {
        assert_eq!(classify_commit_type("fix: null pointer").1, "[fix]");
        assert_eq!(classify_commit_type("feat: add auth").1, "[feat]");
        assert_eq!(
            classify_commit_type("refactor: simplify parser").1,
            "[refactor]"
        );
        assert_eq!(classify_commit_type("docs: update README").1, "[docs]");
        assert_eq!(classify_commit_type("chore: bump deps").1, "[chore]");
        assert_eq!(classify_commit_type("random commit").1, "");
        assert_eq!(classify_commit_type("Merge branch main").1, "[merge]");
    }

    #[test]
    fn test_format_timestamp() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let ts = 1704067200_i64;
        let formatted = format_timestamp(ts);
        // Should produce something reasonable (approximate date)
        assert!(formatted.starts_with("2024"));
    }

    #[test]
    fn test_compute_decay_score() {
        let mut hotspot = FileHotspot::default();
        hotspot.commit_count = 10;
        hotspot.fix_count = 5;
        hotspot.annotation.tdg_grade = Some("D".to_string());
        hotspot.annotation.dead_code_pct = 10.0;

        let decay = compute_decay_score(&hotspot, 100);
        assert!(decay > 0.0);
        assert!(decay <= 1.0);

        // Grade A with no fixes should have low decay
        let mut healthy = FileHotspot::default();
        healthy.commit_count = 5;
        healthy.fix_count = 0;
        healthy.annotation.tdg_grade = Some("A".to_string());
        let healthy_decay = compute_decay_score(&healthy, 100);
        assert!(
            healthy_decay < decay,
            "Healthy file should have lower decay"
        );
    }

    #[test]
    fn test_compute_impact_risk() {
        let mut hotspot = FileHotspot::default();
        hotspot.commit_count = 50;
        hotspot.annotation.max_pagerank = Some(0.01);
        hotspot.annotation.fault_count = 3;

        let risk = compute_impact_risk(&hotspot, 100);
        assert!(risk > 0.0);

        // Zero pagerank = zero risk
        let mut low_risk = FileHotspot::default();
        low_risk.commit_count = 50;
        low_risk.annotation.max_pagerank = Some(0.0);
        assert_eq!(compute_impact_risk(&low_risk, 100), 0.0);
    }

    #[test]
    fn test_parse_git_log_with_issue_refs() {
        let log = "PMAT_START\nH:abc1234567890123456789012345678901234567\nS:feat: add auth (PMAT-472)\nN:noah\nE:noah@test.com\nT:1704067200\nPMAT_FILES\nM\tsrc/main.rs";
        let commits = parse_git_log(log);
        assert_eq!(commits.len(), 1);
        assert!(
            commits[0].issue_refs.contains(&"PMAT-472".to_string())
                || commits[0].issue_refs.contains(&"(PMAT-472)".to_string())
        );
        assert!(commits[0].is_feat);
    }

    #[test]
    fn test_file_annotation_default() {
        let annot = FileAnnotation::default();
        assert_eq!(annot.tdg_grade, None);
        assert_eq!(annot.function_count, 0);
        assert_eq!(annot.dead_code_count, 0);
        assert_eq!(annot.fault_count, 0);
    }
}
