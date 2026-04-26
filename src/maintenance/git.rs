//! Git integration utilities for TICKET-PMAT-5013
//!
//! Provides git operations for commit analysis and ticket tracking.

#![cfg_attr(coverage_nightly, coverage(off))]
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
/// use pmat::maintenance::git::extract_ticket_ids;
///
/// let message = "feat: TICKET-PMAT-5013 - Auto-update hooks (GREEN)";
/// let ids = extract_ticket_ids(message);
/// assert_eq!(ids, vec!["TICKET-PMAT-5013"]);
/// ```
///
/// # Complexity
/// - Time: O(n) where n is message length
/// - Cyclomatic: 3
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_ticket_ids(commit_message: &str) -> Vec<String> {
    debug_assert!(
        !commit_message.is_empty(),
        "commit_message must not be empty"
    );
    use regex::Regex;

    let re = Regex::new(r"TICKET-PMAT-\d{4}").expect("internal error");
    re.find_iter(commit_message)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Pure-compute parser extracted for R5 testability (per spec §4.7).
///
/// Builds a `CommitInfo` from the three captured stdouts produced by the
/// subprocess calls in `get_current_commit`. Extracted so the parsing logic
/// can be tested without a live git invocation.
///
/// - `hash_stdout`: bytes from `git rev-parse HEAD`
/// - `message_stdout`: bytes from `git log -1 --pretty=%B`
/// - `files_stdout`: bytes from `git diff-tree --no-commit-id --name-only -r HEAD`
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn parse_commit_info_from_outputs(
    hash_stdout: &[u8],
    message_stdout: &[u8],
    files_stdout: &[u8],
) -> Result<CommitInfo> {
    Ok(CommitInfo {
        hash: String::from_utf8(hash_stdout.to_vec())?.trim().to_string(),
        message: String::from_utf8(message_stdout.to_vec())?
            .trim()
            .to_string(),
        files: String::from_utf8(files_stdout.to_vec())?
            .lines()
            .map(|s| s.to_string())
            .collect(),
    })
}

/// Get current commit info
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn get_current_commit() -> Result<CommitInfo> {
    let hash_output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

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

    parse_commit_info_from_outputs(
        &hash_output.stdout,
        &message_output.stdout,
        &files_output.stdout,
    )
}

/// Check if ticket file was updated in commit
///
/// # Complexity
/// - Time: O(n) where n is number of files
/// - Cyclomatic: 2
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn ticket_file_updated(commit: &CommitInfo, ticket_id: &str) -> bool {
    let ticket_file = format!("docs/tickets/{}.md", ticket_id);
    commit.files.iter().any(|f| f.contains(&ticket_file))
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn integration_get_current_commit() {
        // This test requires being in a git repo with at least one commit
        match get_current_commit() {
            Ok(commit) => {
                assert!(!commit.hash.is_empty(), "Commit hash should not be empty");
                assert!(
                    !commit.message.is_empty(),
                    "Commit message should not be empty"
                );
                // Files can be empty if this is the first commit or no files changed
            }
            Err(e) => {
                eprintln!(
                    "Git command failed (this is expected if not in a git repo): {}",
                    e
                );
            }
        }
    }

    // ── parse_commit_info_from_outputs (R5 extraction tests) ────────────────

    #[test]
    fn test_parse_commit_info_basic() {
        let info = parse_commit_info_from_outputs(
            b"abc1234\n",
            b"feat: add thing\n",
            b"src/lib.rs\nsrc/main.rs\n",
        )
        .unwrap();
        assert_eq!(info.hash, "abc1234");
        assert_eq!(info.message, "feat: add thing");
        assert_eq!(info.files, vec!["src/lib.rs", "src/main.rs"]);
    }

    #[test]
    fn test_parse_commit_info_strips_whitespace_from_hash_and_message() {
        // git outputs trailing newlines; the parser must trim
        let info = parse_commit_info_from_outputs(
            b"  abc1234  \n",
            b"\nmessage with surrounding ws\n\n",
            b"",
        )
        .unwrap();
        assert_eq!(info.hash, "abc1234");
        assert_eq!(info.message, "message with surrounding ws");
    }

    #[test]
    fn test_parse_commit_info_empty_files_yields_empty_vec() {
        let info = parse_commit_info_from_outputs(b"abc\n", b"msg\n", b"").unwrap();
        assert!(info.files.is_empty());
    }

    #[test]
    fn test_parse_commit_info_multiline_message_trimmed() {
        // git log --pretty=%B produces multi-line bodies; only outer ws trimmed
        let info = parse_commit_info_from_outputs(
            b"abc\n",
            b"feat: subject\n\nbody line 1\nbody line 2\n",
            b"",
        )
        .unwrap();
        assert_eq!(info.message, "feat: subject\n\nbody line 1\nbody line 2");
    }

    #[test]
    fn test_parse_commit_info_invalid_utf8_in_hash_errors() {
        // Invalid UTF-8 sequence in any of the three buffers → Utf8Error
        let bad_utf8 = vec![0xFFu8, 0xFEu8];
        let result = parse_commit_info_from_outputs(&bad_utf8, b"msg", b"");
        assert!(matches!(result, Err(GitError::Utf8Error(_))));
    }

    #[test]
    fn test_parse_commit_info_invalid_utf8_in_message_errors() {
        let bad_utf8 = vec![0xFFu8, 0xFEu8];
        let result = parse_commit_info_from_outputs(b"abc", &bad_utf8, b"");
        assert!(matches!(result, Err(GitError::Utf8Error(_))));
    }

    #[test]
    fn test_parse_commit_info_invalid_utf8_in_files_errors() {
        let bad_utf8 = vec![0xFFu8, 0xFEu8];
        let result = parse_commit_info_from_outputs(b"abc", b"msg", &bad_utf8);
        assert!(matches!(result, Err(GitError::Utf8Error(_))));
    }

    #[test]
    fn test_parse_commit_info_files_with_blank_line_keeps_empty_entry() {
        // .lines() on "a\n\nb\n" yields ["a", "", "b"] — pinned behavior
        let info = parse_commit_info_from_outputs(b"abc", b"msg", b"a\n\nb\n").unwrap();
        assert_eq!(info.files, vec!["a", "", "b"]);
    }
}
