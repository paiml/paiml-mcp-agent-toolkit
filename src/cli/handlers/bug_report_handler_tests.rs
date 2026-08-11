// Bug report handler tests
// Included from bug_report_handler.rs — shares parent module scope (no use imports here)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // Tests below that call `capture_command_error*`, `clear_error`, or
    // `handle_bug_report` all touch the same on-disk file at
    // ~/.pmat/last_error.json. Under cargo's threaded runner they race —
    // a capture test writing the file between `clear_error()` and
    // `handle_bug_report()` flips the latter's expected Err into Ok.
    // `#[serial(bug_report_error_file)]` pins them to one thread.

    #[tokio::test]
    #[serial(bug_report_error_file)]
    async fn test_handle_bug_report_no_error() {
        // Clear any existing error first
        let _ = clear_error();

        let result = handle_bug_report(None, true, false, false).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No captured error found"));
    }

    #[tokio::test]
    #[serial(bug_report_error_file)]
    #[ignore = "Requires HOME directory to be set in test environment"]
    async fn test_handle_bug_report_clear() {
        // Clear should always succeed (no-op if file doesn't exist)
        let result = handle_bug_report(None, false, false, true).await;
        assert!(result.is_ok());
    }

    /// The capture helpers had no production caller at all, so the store
    /// `handle_bug_report` reads was never written and `maintain bug-report`
    /// answered "No captured error found" after every failure. `cli::run` must
    /// write it; `capture_cli_failure` is the one that does.
    #[test]
    #[serial(bug_report_error_file)]
    fn a_failing_invocation_is_written_to_the_store_bug_report_reads() {
        let _ = clear_error();

        capture_cli_failure(
            &[
                "pmat".to_string(),
                "tdg".to_string(),
                "/does/not/exist".to_string(),
            ],
            &anyhow::anyhow!("path does not exist: /does/not/exist"),
        );

        let captured = load_error()
            .expect("load")
            .expect("a failing command must leave a captured error behind");
        assert_eq!(captured.command, "pmat");
        assert_eq!(captured.args, vec!["tdg", "/does/not/exist"]);
        assert!(
            captured.error_message.contains("/does/not/exist"),
            "the captured message must be the real one: {}",
            captured.error_message
        );

        let _ = clear_error();
    }

    /// …and the wiring itself: `cli::run` is the only place a CLI failure
    /// surfaces, so if this call disappears the store goes back to being
    /// written by nothing.
    #[test]
    fn cli_run_captures_dispatch_failures() {
        let code = include_str!("../cli_run_command.rs")
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            code.contains("capture_cli_failure("),
            "cli::run no longer captures failing commands, so \
             `pmat maintain bug-report` has nothing to report"
        );
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error() {
        // This should not panic
        capture_command_error("pmat", &["test".to_string()], "test error");
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_empty_args() {
        // Should handle empty args without panic
        capture_command_error("pmat", &[], "error with no args");
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_multiple_args() {
        // Should handle multiple args
        capture_command_error(
            "pmat",
            &[
                "work".to_string(),
                "status".to_string(),
                "--verbose".to_string(),
            ],
            "complex error",
        );
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_long_error_message() {
        // Should handle long error messages
        let long_error = "A".repeat(10000);
        capture_command_error("pmat", &["test".to_string()], &long_error);
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_unicode() {
        // Should handle unicode in error messages
        capture_command_error("pmat", &["test".to_string()], "Error: 日本語 эррор 🚫");
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_code() {
        // Test capturing error with exit code
        capture_command_error_with_code("pmat", &["work".to_string()], "exit error", 1);
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_code_zero() {
        // Edge case: zero exit code
        capture_command_error_with_code("pmat", &["work".to_string()], "success but captured", 0);
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_code_negative() {
        // Edge case: negative exit code (signal-based termination)
        capture_command_error_with_code("pmat", &["work".to_string()], "signal error", -9);
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_code_large() {
        // Edge case: large exit code
        capture_command_error_with_code("pmat", &["work".to_string()], "error", 255);
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_special_characters() {
        // Error message with special characters that might break markdown
        capture_command_error(
            "pmat",
            &["test".to_string()],
            "Error: `backticks` and **bold** and <html>tags</html>",
        );
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_newlines() {
        // Error message with newlines (multi-line errors)
        capture_command_error(
            "pmat",
            &["test".to_string()],
            "Line 1: Error\nLine 2: Details\nLine 3: Stack trace",
        );
    }

    #[test]
    #[serial(bug_report_error_file)]
    fn test_capture_command_error_with_tabs_and_whitespace() {
        // Error message with various whitespace
        capture_command_error(
            "pmat",
            &["test".to_string()],
            "Error:\t\tTabbed\n    Spaces    \r\nCRLF",
        );
    }

    /// Test parsing of issue content format - title extraction
    #[test]
    fn test_issue_content_parsing_with_title() {
        // Simulate the format returned by generate_issue_markdown
        let content = "TITLE: My Bug Title\n---\n## Summary\n\nBody content here.";
        let parts: Vec<&str> = content.splitn(2, "\n---\n").collect();

        let title = parts
            .first()
            .unwrap_or(&"Bug report")
            .strip_prefix("TITLE: ")
            .unwrap_or("Bug report");
        let body = parts.get(1).unwrap_or(&"");

        assert_eq!(title, "My Bug Title");
        assert_eq!(*body, "## Summary\n\nBody content here.");
    }

    /// Test parsing with malformed content (no separator)
    #[test]
    fn test_issue_content_parsing_no_separator() {
        let content = "Just some content without proper format";
        let parts: Vec<&str> = content.splitn(2, "\n---\n").collect();

        let title = parts
            .first()
            .unwrap_or(&"Bug report")
            .strip_prefix("TITLE: ")
            .unwrap_or("Bug report");
        let body = parts.get(1).unwrap_or(&"");

        // Should fall back to the whole content as title (without TITLE: prefix)
        assert_eq!(title, "Bug report");
        // Body should be empty since there's no separator
        assert_eq!(*body, "");
    }

    /// Test parsing with empty content
    #[test]
    fn test_issue_content_parsing_empty() {
        let content = "";
        let parts: Vec<&str> = content.splitn(2, "\n---\n").collect();

        let title = parts
            .first()
            .unwrap_or(&"Bug report")
            .strip_prefix("TITLE: ")
            .unwrap_or("Bug report");
        let body = parts.get(1).unwrap_or(&"");

        assert_eq!(title, "Bug report");
        assert_eq!(*body, "");
    }

    /// Test parsing with only separator
    #[test]
    fn test_issue_content_parsing_only_separator() {
        let content = "\n---\n";
        let parts: Vec<&str> = content.splitn(2, "\n---\n").collect();

        let title = parts
            .first()
            .unwrap_or(&"Bug report")
            .strip_prefix("TITLE: ")
            .unwrap_or("Bug report");
        let body = parts.get(1).unwrap_or(&"");

        // First part is empty string, which doesn't have TITLE: prefix
        assert_eq!(title, "Bug report");
        // Body is also empty
        assert_eq!(*body, "");
    }

    /// Test parsing with multiple separators
    #[test]
    fn test_issue_content_parsing_multiple_separators() {
        let content = "TITLE: Title\n---\nFirst section\n---\nSecond section";
        let parts: Vec<&str> = content.splitn(2, "\n---\n").collect();

        let title = parts
            .first()
            .unwrap_or(&"Bug report")
            .strip_prefix("TITLE: ")
            .unwrap_or("Bug report");
        let body = parts.get(1).unwrap_or(&"");

        assert_eq!(title, "Title");
        // Body should include everything after first separator (including second separator)
        assert_eq!(*body, "First section\n---\nSecond section");
    }

    /// Test that CapturedError can be created and used for bug reports
    #[test]
    fn test_captured_error_creation_for_bug_report() {
        let error = CapturedError::new(
            "pmat work",
            &[
                "status".to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            "Failed to connect to database",
        );

        assert_eq!(error.command, "pmat work");
        assert_eq!(error.args.len(), 3);
        assert_eq!(error.error_message, "Failed to connect to database");
        assert!(error.exit_code.is_none());
    }

    /// Test CapturedError with exit code for bug reports
    #[test]
    fn test_captured_error_with_exit_code_for_bug_report() {
        let error = CapturedError::new("pmat", &["analyze".to_string()], "Analysis failed")
            .with_exit_code(42);

        assert_eq!(error.exit_code, Some(42));
        assert_eq!(error.command, "pmat");
    }

    /// Test CapturedError with backtrace for bug reports
    #[test]
    fn test_captured_error_with_backtrace_for_bug_report() {
        let backtrace = "   0: pmat::main\n   1: std::rt::lang_start";
        let error =
            CapturedError::new("pmat", &["work".to_string()], "Panic").with_backtrace(backtrace);

        assert_eq!(error.backtrace, Some(backtrace.to_string()));
    }

    /// Test CapturedError chaining with_backtrace and with_exit_code
    #[test]
    fn test_captured_error_chaining() {
        let error = CapturedError::new("pmat", &[], "error")
            .with_exit_code(1)
            .with_backtrace("trace");

        assert_eq!(error.exit_code, Some(1));
        assert_eq!(error.backtrace, Some("trace".to_string()));
    }

    /// Test that version and OS are populated in CapturedError
    #[test]
    fn test_captured_error_metadata() {
        let error = CapturedError::new("pmat", &[], "test");

        // Version should match CARGO_PKG_VERSION
        assert!(!error.version.is_empty());
        // OS should be populated
        assert!(!error.os.is_empty());
        // Timestamp should be set
        assert!(!error.timestamp.is_empty());
        // Timestamp should be RFC3339 format (contains T and Z or timezone)
        assert!(error.timestamp.contains('T'));
    }

    /// Test redact_paths functionality
    #[test]
    fn test_captured_error_redact_paths_in_message() {
        // Set up environment for test
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/testuser".to_string());

        let mut error =
            CapturedError::new("pmat", &[], &format!("Error at {}/project/file.rs", home));

        error.redact_paths();

        // Home should be redacted
        assert!(
            error.error_message.contains("~"),
            "Error message should contain ~ after redaction: {}",
            error.error_message
        );
        assert!(
            !error.error_message.contains(&home),
            "Error message should not contain home path after redaction"
        );
    }

    /// Test redact_paths with backtrace
    #[test]
    fn test_captured_error_redact_paths_in_backtrace() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/testuser".to_string());

        let mut error = CapturedError::new("pmat", &[], "error")
            .with_backtrace(&format!("  at {}/project/src/main.rs:42", home));

        error.redact_paths();

        if let Some(bt) = &error.backtrace {
            assert!(bt.contains("~"), "Backtrace should contain ~");
            assert!(!bt.contains(&home), "Backtrace should not contain home");
        }
    }

    /// Test redact_paths with project_path
    #[test]
    fn test_captured_error_redact_project_path() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/testuser".to_string());

        let mut error = CapturedError::new("pmat", &[], "error");
        // Manually set project_path that includes home
        error.project_path = Some(format!("{}/project", home));

        error.redact_paths();

        if let Some(path) = &error.project_path {
            assert!(path.contains("~"), "Project path should contain ~");
            assert!(
                !path.contains(&home),
                "Project path should not contain home"
            );
        }
    }

    /// Test generate_issue_markdown with empty args
    #[test]
    fn test_generate_issue_markdown_empty_args() {
        let error = CapturedError::new("pmat analyze", &[], "Analysis failed");

        let md = generate_issue_markdown(&error, Some("Empty Args Bug"));

        assert!(md.contains("TITLE: Empty Args Bug"));
        assert!(md.contains("pmat analyze"));
        // Command section should just show the command without trailing space
        assert!(md.contains("```bash\npmat analyze\n```"));
    }

    /// Test generate_issue_markdown with args
    #[test]
    fn test_generate_issue_markdown_with_args() {
        let error =
            CapturedError::new("pmat", &["work".to_string(), "status".to_string()], "Error");

        let md = generate_issue_markdown(&error, Some("Test"));

        assert!(md.contains("pmat work status"));
    }

    /// Test generate_issue_markdown includes all sections
    #[test]
    fn test_generate_issue_markdown_all_sections() {
        let error = CapturedError::new("pmat", &["test".to_string()], "Test error")
            .with_exit_code(1)
            .with_backtrace("backtrace content");

        let md = generate_issue_markdown(&error, None);

        // Check all required sections exist
        assert!(md.contains("## Summary"));
        assert!(md.contains("## Environment"));
        assert!(md.contains("## Command Executed"));
        assert!(md.contains("## Error Output"));
        assert!(md.contains("<details>"));
        assert!(md.contains("Backtrace"));
        assert!(md.contains("**Exit Code**: 1"));
        assert!(md.contains("## Steps to Reproduce"));
        assert!(md.contains("## Expected Behavior"));
        assert!(md.contains("Generated automatically by `pmat bug-report`"));
    }

    /// Test generate_issue_markdown default title
    #[test]
    fn test_generate_issue_markdown_default_title_multiword_command() {
        let error = CapturedError::new("pmat work status", &[], "Error");

        let md = generate_issue_markdown(&error, None);

        // Default title should take first two words of command
        assert!(md.contains("TITLE: Bug: pmat work fails with error"));
    }

    /// Test generate_issue_markdown without optional fields
    #[test]
    fn test_generate_issue_markdown_no_optional_fields() {
        let mut error = CapturedError::new("pmat", &[], "Error");
        error.project_path = None; // Clear project path

        let md = generate_issue_markdown(&error, Some("Simple Bug"));

        // Should not contain backtrace section
        assert!(!md.contains("<details>"));
        // Should not contain exit code
        assert!(!md.contains("Exit Code"));
    }

    /// Test issue markdown with project path included
    #[test]
    fn test_generate_issue_markdown_with_project_path() {
        let mut error = CapturedError::new("pmat", &[], "Error");
        error.project_path = Some("/path/to/project".to_string());

        let md = generate_issue_markdown(&error, Some("Bug"));

        assert!(md.contains("**Project Path**: `/path/to/project`"));
    }

    /// Test markdown escaping concerns
    #[test]
    fn test_generate_issue_markdown_special_chars() {
        let error = CapturedError::new(
            "pmat",
            &["--option=value".to_string()],
            "Error: `special` <chars> *markdown* _underscore_",
        );

        let md = generate_issue_markdown(&error, Some("Special Characters"));

        // Content should be preserved (inside code blocks)
        assert!(md.contains("`special`"));
        assert!(md.contains("<chars>"));
    }
}
