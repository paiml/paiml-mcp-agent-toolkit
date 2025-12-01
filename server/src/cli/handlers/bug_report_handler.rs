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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Flaky test due to shared ~/.pmat/last_error.json in parallel test execution"]
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
    #[ignore = "Flaky test due to file system race conditions in parallel test execution"]
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
}
