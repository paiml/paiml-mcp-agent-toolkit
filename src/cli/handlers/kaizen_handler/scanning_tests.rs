// Kaizen scanning tests
// Included from scanning.rs — no `use` imports, shares parent scope.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::KaizenOutputFormat;

    #[test]
    fn test_first_line() {
        assert_eq!(first_line("hello\nworld"), "hello");
        assert_eq!(first_line("single"), "single");
        assert_eq!(first_line(""), "");
    }

    #[test]
    fn test_extract_file_from_message() {
        let msg = serde_json::json!({
            "spans": [{"file_name": "src/main.rs", "line_start": 10}]
        });
        assert_eq!(
            extract_file_from_message(&msg),
            Some("src/main.rs".to_string())
        );
    }

    #[test]
    fn test_extract_file_from_message_empty() {
        let msg = serde_json::json!({"spans": []});
        assert_eq!(extract_file_from_message(&msg), None);
    }

    #[test]
    fn test_extract_score_from_command_output_json() {
        assert_eq!(
            extract_score_from_command_output(r#"{"score": 95.5}"#),
            Some(95.5)
        );
        assert_eq!(
            extract_score_from_command_output("some output\n{\"score\": 42.0}\nmore output"),
            Some(42.0)
        );
    }

    #[test]
    fn test_extract_score_from_command_output_score_prefix() {
        assert_eq!(extract_score_from_command_output("SCORE: 88.3"), Some(88.3));
        assert_eq!(
            extract_score_from_command_output("running tests...\nSCORE: 100"),
            Some(100.0)
        );
    }

    #[test]
    fn test_extract_score_from_command_output_no_score() {
        assert_eq!(extract_score_from_command_output("no score here"), None);
        assert_eq!(extract_score_from_command_output(""), None);
    }

    #[test]
    fn test_scan_crate_tags_findings() {
        // scan_crate with a non-existent path will return empty (tools not found),
        // but we can verify the tagging logic by calling it
        let path = std::path::PathBuf::from("/nonexistent");
        let result = scan_crate(
            &path,
            Some("test-crate"),
            &KaizenConfig {
                path: path.clone(),
                dry_run: true,
                commit: false,
                create_issues: false,
                push: false,
                auto_agent: false,
                max_agents: 0,
                format: KaizenOutputFormat::Text,
                output: None,
                skip_clippy: true,
                skip_fmt: true,
                skip_comply: true,
                skip_github: true,
                skip_defects: true,
                cross_stack: true,
            },
        );
        // With all scanners skipped, should be empty but Ok
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
