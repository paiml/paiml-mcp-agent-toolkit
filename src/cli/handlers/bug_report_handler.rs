// Bug report CLI handler (GH-81)
// Toyota Way: Kaizen - continuous improvement through feedback

use anyhow::{Context, Result};
use std::process::Command;

use crate::services::error_capture::{
    clear_error, generate_issue_markdown, load_error, CapturedError,
};

/// Handle the `pmat maintain bug-report` command
pub async fn handle_bug_report(
    title: Option<&str>,
    dry_run: bool,
    interactive: bool,
    clear: bool,
) -> Result<()> {
    // Handle clear flag
    if clear {
        clear_error()?;
        println!("✅ Cleared captured error");
        return Ok(());
    }

    // Load captured error
    let error = load_error()?.context(
        "No captured error found. Run a pmat command that fails first, \
         or the error capture may not be enabled.",
    )?;

    println!("📋 Captured error from: {}", error.command);
    println!("🔍 PMAT Version: {}", error.version);
    println!("💻 OS: {}", error.os);
    println!("📅 Timestamp: {}", error.timestamp);
    println!();

    // Generate issue markdown
    let mut redacted_error = error.clone();
    redacted_error.redact_paths();
    let issue_content = generate_issue_markdown(&redacted_error, title);

    // Parse title and body from generated content
    let parts: Vec<&str> = issue_content.splitn(2, "\n---\n").collect();
    let issue_title = parts
        .first()
        .unwrap_or(&"Bug report")
        .strip_prefix("TITLE: ")
        .unwrap_or("Bug report");
    let issue_body = parts.get(1).unwrap_or(&"");

    if dry_run {
        println!("📝 Generated Issue (dry-run):\n");
        println!("Title: {}", issue_title);
        println!("---");
        println!("{}", issue_body);
        return Ok(());
    }

    // Interactive confirmation
    if interactive {
        println!("📝 Generated Issue:\n");
        println!("Title: {}", issue_title);
        println!("---");
        println!("{}", issue_body);
        println!();

        print!("Create this issue? [Y/n] ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input == "n" || input == "no" {
            println!("❌ Cancelled");
            return Ok(());
        }
    }

    // Create GitHub issue using gh CLI
    create_github_issue(issue_title, issue_body)?;

    // Clear error after successful report
    clear_error()?;

    Ok(())
}

/// Create GitHub issue using gh CLI
fn create_github_issue(title: &str, body: &str) -> Result<()> {
    // Check if gh is available
    let gh_check = Command::new("gh").arg("--version").output();

    if gh_check.is_err() {
        return Err(anyhow::anyhow!(
            "GitHub CLI (gh) not found. Install it from: https://cli.github.com/"
        ));
    }

    println!("🔄 Creating GitHub issue...");

    // Create issue
    let output = Command::new("gh")
        .args([
            "issue",
            "create",
            "--repo",
            "paiml/paiml-mcp-agent-toolkit",
            "--title",
            title,
            "--body",
            body,
            "--label",
            "bug",
        ])
        .output()
        .context("Failed to run gh issue create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to create issue: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("✅ Created: {}", stdout.trim());

    Ok(())
}

/// Capture an error for later bug reporting
/// Called when a pmat command fails
pub fn capture_command_error(command: &str, args: &[String], error: &str) {
    let captured = CapturedError::new(command, args, error);

    if let Err(e) = crate::services::error_capture::save_error(&captured) {
        eprintln!("Warning: Failed to capture error for bug reporting: {}", e);
    }
}

/// Capture an error with exit code
pub fn capture_command_error_with_code(command: &str, args: &[String], error: &str, code: i32) {
    let captured = CapturedError::new(command, args, error).with_exit_code(code);

    if let Err(e) = crate::services::error_capture::save_error(&captured) {
        eprintln!("Warning: Failed to capture error for bug reporting: {}", e);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
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
    // Re-enabled: test passes
    async fn test_handle_bug_report_clear() {
        // Clear should always succeed (no-op if file doesn't exist)
        let result = handle_bug_report(None, false, false, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_capture_command_error() {
        // This should not panic
        capture_command_error("pmat", &["test".to_string()], "test error");
    }

    #[test]
    fn test_capture_command_error_with_empty_args() {
        // Should handle empty args without panic
        capture_command_error("pmat", &[], "error with no args");
    }

    #[test]
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
    fn test_capture_command_error_with_long_error_message() {
        // Should handle long error messages
        let long_error = "A".repeat(10000);
        capture_command_error("pmat", &["test".to_string()], &long_error);
    }

    #[test]
    fn test_capture_command_error_with_unicode() {
        // Should handle unicode in error messages
        capture_command_error("pmat", &["test".to_string()], "Error: 日本語 эррор 🚫");
    }

    #[test]
    fn test_capture_command_error_with_code() {
        // Test capturing error with exit code
        capture_command_error_with_code("pmat", &["work".to_string()], "exit error", 1);
    }

    #[test]
    fn test_capture_command_error_with_code_zero() {
        // Edge case: zero exit code
        capture_command_error_with_code("pmat", &["work".to_string()], "success but captured", 0);
    }

    #[test]
    fn test_capture_command_error_with_code_negative() {
        // Edge case: negative exit code (signal-based termination)
        capture_command_error_with_code("pmat", &["work".to_string()], "signal error", -9);
    }

    #[test]
    fn test_capture_command_error_with_code_large() {
        // Edge case: large exit code
        capture_command_error_with_code("pmat", &["work".to_string()], "error", 255);
    }

    #[test]
    fn test_capture_command_error_with_special_characters() {
        // Error message with special characters that might break markdown
        capture_command_error(
            "pmat",
            &["test".to_string()],
            "Error: `backticks` and **bold** and <html>tags</html>",
        );
    }

    #[test]
    fn test_capture_command_error_with_newlines() {
        // Error message with newlines (multi-line errors)
        capture_command_error(
            "pmat",
            &["test".to_string()],
            "Line 1: Error\nLine 2: Details\nLine 3: Stack trace",
        );
    }

    #[test]
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
