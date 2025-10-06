//! Git integration utilities for TICKET-PMAT-5013
//!
//! Provides git operations for commit analysis and ticket tracking.

use std::process::Command;

/// Git commit information
#[derive(Debug, Clone, PartialEq)]
pub struct CommitInfo {
    /// Commit hash
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Changed files
    pub files: Vec<String>,
}

/// Git-related errors
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git command failed: {0}")]
    CommandFailed(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, GitError>;

/// Extract ticket IDs from commit message
///
/// # Example
/// ```
/// use paiml_mcp_agent_toolkit::maintenance::git::extract_ticket_ids;
///
/// let message = "feat: TICKET-PMAT-5013 - Auto-update hooks (GREEN)";
/// let ids = extract_ticket_ids(message);
/// assert_eq!(ids, vec!["TICKET-PMAT-5013"]);
/// ```
///
/// # Complexity
/// - Time: O(n) where n is message length
/// - Cyclomatic: 3
pub fn extract_ticket_ids(commit_message: &str) -> Vec<String> {
    use regex::Regex;

    let re = Regex::new(r"TICKET-PMAT-\d{4}").unwrap();
    re.find_iter(commit_message)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Get current commit info
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
pub fn get_current_commit() -> Result<CommitInfo> {
    let hash_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()?;

    if !hash_output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&hash_output.stderr).to_string(),
        ));
    }

    let message_output = Command::new("git")
        .args(["log", "-1", "--pretty=%B"])
        .output()?;

    if !message_output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&message_output.stderr).to_string(),
        ));
    }

    let files_output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
        .output()?;

    if !files_output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&files_output.stderr).to_string(),
        ));
    }

    Ok(CommitInfo {
        hash: String::from_utf8(hash_output.stdout)?.trim().to_string(),
        message: String::from_utf8(message_output.stdout)?.trim().to_string(),
        files: String::from_utf8(files_output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .collect(),
    })
}

/// Check if ticket file was updated in commit
///
/// # Complexity
/// - Time: O(n) where n is number of files
/// - Cyclomatic: 2
pub fn ticket_file_updated(commit: &CommitInfo, ticket_id: &str) -> bool {
    let ticket_file = format!("docs/tickets/{}.md", ticket_id);
    commit.files.iter().any(|f| f.contains(&ticket_file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ticket_ids_single() {
        let message = "feat: TICKET-PMAT-5013 - Auto-update hooks (GREEN)";
        let ids = extract_ticket_ids(message);

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "TICKET-PMAT-5013");
    }

    #[test]
    fn test_extract_ticket_ids_multiple() {
        let message = "fix: TICKET-PMAT-5001 and TICKET-PMAT-5002 issues";
        let ids = extract_ticket_ids(message);

        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "TICKET-PMAT-5001");
        assert_eq!(ids[1], "TICKET-PMAT-5002");
    }

    #[test]
    fn test_extract_ticket_ids_none() {
        let message = "chore: Update documentation";
        let ids = extract_ticket_ids(message);

        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn test_extract_ticket_ids_middle() {
        let message = "TICKET-PMAT-1234 is at the start";
        let ids = extract_ticket_ids(message);

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "TICKET-PMAT-1234");
    }

    #[test]
    fn test_ticket_file_updated_true() {
        let commit = CommitInfo {
            hash: "abc123".into(),
            message: "test".into(),
            files: vec![
                "docs/tickets/TICKET-PMAT-5013.md".into(),
                "server/src/main.rs".into(),
            ],
        };

        assert!(ticket_file_updated(&commit, "TICKET-PMAT-5013"));
    }

    #[test]
    fn test_ticket_file_updated_false() {
        let commit = CommitInfo {
            hash: "abc123".into(),
            message: "test".into(),
            files: vec![
                "docs/tickets/TICKET-PMAT-5013.md".into(),
                "server/src/main.rs".into(),
            ],
        };

        assert!(!ticket_file_updated(&commit, "TICKET-PMAT-9999"));
    }

    #[test]
    fn test_ticket_file_updated_empty_files() {
        let commit = CommitInfo {
            hash: "abc123".into(),
            message: "test".into(),
            files: vec![],
        };

        assert!(!ticket_file_updated(&commit, "TICKET-PMAT-5013"));
    }

    #[test]
    fn test_commit_info_equality() {
        let commit1 = CommitInfo {
            hash: "abc123".into(),
            message: "test".into(),
            files: vec!["file1.rs".into()],
        };

        let commit2 = CommitInfo {
            hash: "abc123".into(),
            message: "test".into(),
            files: vec!["file1.rs".into()],
        };

        assert_eq!(commit1, commit2);
    }

    // Integration test - only runs if we're in a git repo
    #[test]
    #[ignore] // Ignore by default, can be run manually
    fn integration_get_current_commit() {
        // This test requires being in a git repo with at least one commit
        match get_current_commit() {
            Ok(commit) => {
                assert!(!commit.hash.is_empty(), "Commit hash should not be empty");
                assert!(!commit.message.is_empty(), "Commit message should not be empty");
                // Files can be empty if this is the first commit or no files changed
            }
            Err(e) => {
                eprintln!("Git command failed (this is expected if not in a git repo): {}", e);
            }
        }
    }
}
