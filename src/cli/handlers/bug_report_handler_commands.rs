// Bug report handler commands - handle_bug_report, create_github_issue, capture helpers
// Included from bug_report_handler.rs — shares parent module scope (no use imports here)

/// Handle the `pmat maintain bug-report` command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_bug_report(
    title: Option<&str>,
    dry_run: bool,
    interactive: bool,
    clear: bool,
) -> Result<()> {
    use crate::cli::colors as c;

    // Handle clear flag
    if clear {
        clear_error()?;
        println!("{}", c::pass("Cleared captured error"));
        return Ok(());
    }

    // Load captured error
    let error = load_error()?.context(
        "No captured error found. Run a pmat command that fails first, \
         or the error capture may not be enabled.",
    )?;

    println!("{} Captured error from: {}", c::label(""), c::path(&error.command));
    println!("  {}: {}", c::dim("PMAT Version"), error.version);
    println!("  {}: {}", c::dim("OS"), error.os);
    println!("  {}: {}", c::dim("Timestamp"), error.timestamp);
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
        println!("{} Generated Issue (dry-run):\n", c::label(""));
        println!("{}: {}", c::dim("Title"), issue_title);
        println!("{}", c::separator());
        println!("{}", issue_body);
        return Ok(());
    }

    // Interactive confirmation
    if interactive {
        println!("{} Generated Issue:\n", c::label(""));
        println!("{}: {}", c::dim("Title"), issue_title);
        println!("{}", c::separator());
        println!("{}", issue_body);
        println!();

        print!("Create this issue? [Y/n] ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input == "n" || input == "no" {
            println!("{}", c::fail("Cancelled"));
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
    debug_assert!(!body.is_empty(), "body must not be empty");
    use crate::cli::colors as c;

    // Check if gh is available
    let gh_check = Command::new("gh").arg("--version").output();

    if gh_check.is_err() {
        return Err(anyhow::anyhow!(
            "GitHub CLI (gh) not found. Install it from: https://cli.github.com/"
        ));
    }

    println!("{}", c::label("Creating GitHub issue..."));

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
    println!("{} Created: {}", c::pass(""), c::path(stdout.trim()));

    Ok(())
}

/// Capture an error for later bug reporting
/// Called when a pmat command fails
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn capture_command_error(command: &str, args: &[String], error: &str) {
    debug_assert!(!command.is_empty(), "command must not be empty");
    let captured = CapturedError::new(command, args, error);

    if let Err(e) = crate::services::error_capture::save_error(&captured) {
        eprintln!("Warning: Failed to capture error for bug reporting: {}", e);
    }
}

/// Capture an error with exit code
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn capture_command_error_with_code(command: &str, args: &[String], error: &str, code: i32) {
    debug_assert!(!command.is_empty(), "command must not be empty");
    let captured = CapturedError::new(command, args, error).with_exit_code(code);

    if let Err(e) = crate::services::error_capture::save_error(&captured) {
        eprintln!("Warning: Failed to capture error for bug reporting: {}", e);
    }
}
