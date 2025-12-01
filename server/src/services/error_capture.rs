// Error capture service for bug-report command (GH-81)
// Toyota Way: Jidoka - automation with human touch for error handling

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Captured error information for bug reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedError {
    /// Command that was executed
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Error message
    pub error_message: String,
    /// Backtrace if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtrace: Option<String>,
    /// Timestamp of error
    pub timestamp: String,
    /// PMAT version
    pub version: String,
    /// Operating system
    pub os: String,
    /// Project path where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
    /// Exit code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

impl CapturedError {
    /// Create a new captured error
    pub fn new(command: &str, args: &[String], error_message: &str) -> Self {
        Self {
            command: command.to_string(),
            args: args.to_vec(),
            error_message: error_message.to_string(),
            backtrace: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            project_path: std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string()),
            exit_code: None,
        }
    }

    /// Add backtrace to error
    pub fn with_backtrace(mut self, backtrace: &str) -> Self {
        self.backtrace = Some(backtrace.to_string());
        self
    }

    /// Add exit code to error
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Redact sensitive paths from error (privacy protection)
    pub fn redact_paths(&mut self) {
        // Redact home directory
        if let Ok(home) = std::env::var("HOME") {
            self.error_message = self.error_message.replace(&home, "~");
            if let Some(ref mut bt) = self.backtrace {
                *bt = bt.replace(&home, "~");
            }
            if let Some(ref mut path) = self.project_path {
                *path = path.replace(&home, "~");
            }
        }

        // Redact username from paths
        if let Ok(user) = std::env::var("USER") {
            self.error_message = self.error_message.replace(&user, "<user>");
        }
    }
}

/// Get the path to the error capture file
pub fn get_error_capture_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let pmat_dir = home.join(".pmat");

    // Create directory if it doesn't exist
    if !pmat_dir.exists() {
        std::fs::create_dir_all(&pmat_dir).context("Failed to create ~/.pmat directory")?;
    }

    Ok(pmat_dir.join("last_error.json"))
}

/// Save captured error to disk
pub fn save_error(error: &CapturedError) -> Result<()> {
    let path = get_error_capture_path()?;
    let json = serde_json::to_string_pretty(error).context("Failed to serialize error")?;
    std::fs::write(&path, json).context("Failed to write error file")?;
    Ok(())
}

/// Load captured error from disk
pub fn load_error() -> Result<Option<CapturedError>> {
    let path = get_error_capture_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path).context("Failed to read error file")?;
    let error: CapturedError =
        serde_json::from_str(&content).context("Failed to parse error file")?;

    Ok(Some(error))
}

/// Clear captured error from disk
pub fn clear_error() -> Result<()> {
    let path = get_error_capture_path()?;

    if path.exists() {
        std::fs::remove_file(&path).context("Failed to remove error file")?;
    }

    Ok(())
}

/// Generate GitHub issue markdown from captured error
pub fn generate_issue_markdown(error: &CapturedError, title: Option<&str>) -> String {
    let default_title = format!(
        "Bug: {} fails with error",
        error.command.split_whitespace().take(2).collect::<Vec<_>>().join(" ")
    );
    let title = title.unwrap_or(&default_title);

    let mut md = String::new();

    // Summary section
    md.push_str("## Summary\n\n");
    md.push_str(&format!(
        "Command `{}` failed with an error.\n\n",
        error.command
    ));

    // Environment section
    md.push_str("## Environment\n\n");
    md.push_str(&format!("- **PMAT Version**: {}\n", error.version));
    md.push_str(&format!("- **OS**: {}\n", error.os));
    md.push_str(&format!("- **Timestamp**: {}\n", error.timestamp));
    if let Some(ref path) = error.project_path {
        md.push_str(&format!("- **Project Path**: `{}`\n", path));
    }
    md.push_str("\n");

    // Command section
    md.push_str("## Command Executed\n\n");
    md.push_str("```bash\n");
    if error.args.is_empty() {
        md.push_str(&error.command);
    } else {
        md.push_str(&format!("{} {}", error.command, error.args.join(" ")));
    }
    md.push_str("\n```\n\n");

    // Error output section
    md.push_str("## Error Output\n\n");
    md.push_str("```\n");
    md.push_str(&error.error_message);
    md.push_str("\n```\n\n");

    // Backtrace section (if available)
    if let Some(ref backtrace) = error.backtrace {
        md.push_str("<details>\n<summary>Backtrace</summary>\n\n");
        md.push_str("```\n");
        md.push_str(backtrace);
        md.push_str("\n```\n\n");
        md.push_str("</details>\n\n");
    }

    // Exit code section
    if let Some(code) = error.exit_code {
        md.push_str(&format!("**Exit Code**: {}\n\n", code));
    }

    // Steps to reproduce (placeholder)
    md.push_str("## Steps to Reproduce\n\n");
    md.push_str("1. Navigate to project directory\n");
    md.push_str(&format!(
        "2. Run: `{}`\n",
        if error.args.is_empty() {
            error.command.clone()
        } else {
            format!("{} {}", error.command, error.args.join(" "))
        }
    ));
    md.push_str("3. Observe error\n\n");

    // Expected behavior
    md.push_str("## Expected Behavior\n\n");
    md.push_str("Command should complete successfully without errors.\n\n");

    // Generated notice
    md.push_str("---\n");
    md.push_str("*Generated automatically by `pmat bug-report`*\n");

    // Return title and body for the gh command
    format!("TITLE: {}\n---\n{}", title, md)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captured_error_new() {
        let error = CapturedError::new("pmat", &["work".to_string(), "status".to_string()], "Failed to load roadmap");

        assert_eq!(error.command, "pmat");
        assert_eq!(error.args, vec!["work", "status"]);
        assert_eq!(error.error_message, "Failed to load roadmap");
        assert!(error.backtrace.is_none());
        assert!(!error.version.is_empty());
        assert!(!error.os.is_empty());
    }

    #[test]
    fn test_captured_error_with_backtrace() {
        let error = CapturedError::new("pmat", &[], "error")
            .with_backtrace("backtrace here");

        assert_eq!(error.backtrace, Some("backtrace here".to_string()));
    }

    #[test]
    fn test_captured_error_with_exit_code() {
        let error = CapturedError::new("pmat", &[], "error")
            .with_exit_code(1);

        assert_eq!(error.exit_code, Some(1));
    }

    #[test]
    fn test_redact_paths() {
        let mut error = CapturedError::new("pmat", &[], "/home/testuser/project/error");
        std::env::set_var("HOME", "/home/testuser");
        error.redact_paths();

        assert!(error.error_message.contains("~"));
        assert!(!error.error_message.contains("/home/testuser"));
    }

    #[test]
    fn test_generate_issue_markdown() {
        let error = CapturedError::new("pmat", &["work".to_string(), "status".to_string()], "Failed");
        let md = generate_issue_markdown(&error, Some("Test Title"));

        assert!(md.contains("TITLE: Test Title"));
        assert!(md.contains("## Summary"));
        assert!(md.contains("## Environment"));
        assert!(md.contains("## Command Executed"));
        assert!(md.contains("pmat work status"));
        assert!(md.contains("## Error Output"));
    }

    #[test]
    fn test_generate_issue_markdown_default_title() {
        let error = CapturedError::new("pmat work", &[], "Failed");
        let md = generate_issue_markdown(&error, None);

        assert!(md.contains("TITLE: Bug: pmat work fails with error"));
    }

    #[test]
    fn test_error_capture_path() {
        let path = get_error_capture_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains(".pmat"));
        assert!(path.to_string_lossy().contains("last_error.json"));
    }
}
