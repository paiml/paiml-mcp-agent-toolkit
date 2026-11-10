#![cfg_attr(coverage_nightly, coverage(off))]
//! FLOW-012 P1 (pmat#1591): the branch is the ticket.
//!
//! The active ticket is derived from the checked-out branch `work/PMAT-NNNN`.
//! No declaration file and no key file is read, so a fresh worktree on
//! `work/PMAT-123` needs no setup for `pmat work status` to report PMAT-123.

use std::path::Path;
use std::process::Command;

/// `work/PMAT-123` -> `Some("PMAT-123")`. Anything else, including a bare
/// `work/PMAT-` or a suffix after the digits, is not a ticket branch.
pub(crate) fn ticket_from_branch(branch: &str) -> Option<String> {
    let id = branch.strip_prefix("work/")?;
    let digits = id.strip_prefix("PMAT-")?;
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then(|| id.to_string())
}

/// The ticket the checkout in `dir` is on, from its current branch name.
/// `None` for a detached HEAD, a non-ticket branch, or a dir that is not a repo.
pub(crate) fn active_ticket(dir: &Path) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["symbolic-ref", "--short", "-q", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    ticket_from_branch(String::from_utf8_lossy(&out.stdout).trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_names_map_to_tickets() {
        assert_eq!(
            ticket_from_branch("work/PMAT-123").as_deref(),
            Some("PMAT-123")
        );
        for bad in [
            "main",
            "work/PMAT-",
            "work/PMAT-12a",
            "work/PMAT-1/x",
            "feat/PMAT-1",
            "PMAT-1",
            "",
        ] {
            assert_eq!(ticket_from_branch(bad), None, "{bad}");
        }
    }

    fn git(dir: &Path, args: &[&str]) {
        let st = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .expect("git runs");
        assert!(st.success(), "git {args:?}");
    }

    /// The ticket's falsifier: a fresh worktree on work/PMAT-123 with no setup.
    #[test]
    fn fresh_checkout_on_a_ticket_branch_reports_it_with_no_setup() {
        let dir = tempfile::tempdir().expect("tempdir");
        git(dir.path(), &["init", "-q", "-b", "main"]);
        assert_eq!(active_ticket(dir.path()), None, "main is no ticket");
        git(dir.path(), &["checkout", "-q", "-b", "work/PMAT-123"]);
        assert_eq!(active_ticket(dir.path()).as_deref(), Some("PMAT-123"));
        assert!(
            !dir.path().join(".pmat-work").exists(),
            "no setup file was needed"
        );
    }

    #[test]
    fn a_non_repo_dir_has_no_active_ticket() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(active_ticket(dir.path()), None);
    }
}
