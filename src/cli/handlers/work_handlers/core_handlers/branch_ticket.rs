//! FLOW-012 P1 (GH-1591): the branch is the ticket.
//!
//! "State is derived, never declared": with no explicit id, the active ticket
//! is the one named by the current branch, `work/PMAT-NNNN`. Nothing is
//! written and nothing has to be set up first.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// The ticket a branch name denotes, if and only if it is exactly
/// `work/PMAT-<digits>`. Anything else (suffixes, other prefixes) is no ticket.
pub fn ticket_from_branch(branch: &str) -> Option<String> {
    let digits = branch.strip_prefix("work/PMAT-")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(format!("PMAT-{digits}"))
}

/// Current branch of the repository at `dir`; `None` when detached, not a
/// repository, or git is unavailable.
pub fn current_branch(dir: &Path) -> Option<String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!b.is_empty()).then_some(b)
}

/// The ticket derived from the branch checked out at `dir`, with that branch.
pub fn branch_ticket(dir: &Path) -> Option<(String, String)> {
    let branch = current_branch(dir)?;
    ticket_from_branch(&branch).map(|t| (t, branch))
}

/// Resolve the ticket id for a verb. Priority: explicit argument, then the
/// branch. Off a `work/PMAT-NNNN` branch there is no guess: an error with the fix.
pub fn resolve_ticket_id(explicit: Option<String>, dir: &Path) -> Result<String> {
    if let Some(id) = explicit {
        if let Some((derived, branch)) = branch_ticket(dir) {
            if derived != id {
                eprintln!("note: explicit id {id} used; branch {branch} names {derived}");
            }
        }
        return Ok(id);
    }
    match branch_ticket(dir) {
        Some((t, _)) => Ok(t),
        None => {
            let shown = current_branch(dir).unwrap_or_else(|| "(detached or no git)".into());
            bail!(
                "no ticket id given and branch '{shown}' does not match work/PMAT-NNNN. \
                 Pass an id, or switch to a branch named work/PMAT-NNNN."
            )
        }
    }
}

/// `pmat work status`: with no explicit id (and not `--active`), report the
/// branch's ticket. Needs no roadmap; shows the roadmap detail when the row is there.
pub async fn handle_work_status_derived(
    id: Option<String>,
    path: Option<std::path::PathBuf>,
    active: bool,
) -> Result<()> {
    use crate::cli::colors as c;
    let dir = path
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let derived = if id.is_none() && !active {
        branch_ticket(&dir)
    } else {
        None
    };
    let Some((ticket, branch)) = derived else {
        return super::handlers::handle_work_status(id, path, active).await;
    };
    println!(
        "{} Active ticket: {} (from branch {})",
        c::subheader("🎯"),
        c::path(&ticket),
        branch
    );
    println!();
    let roadmap = crate::services::roadmap_service::RoadmapService::new(
        dir.join("docs/roadmaps/roadmap.yaml"),
    );
    let known = roadmap
        .load()
        .ok()
        .is_some_and(|r| r.find_item(&ticket).is_some());
    if !known {
        println!(
            "   {}",
            c::dim("Not in docs/roadmaps/roadmap.yaml on this branch; add it with: pmat work add")
        );
        return Ok(());
    }
    super::handlers::handle_work_status(Some(ticket), path, active).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_work_branch_is_a_ticket() {
        assert_eq!(
            ticket_from_branch("work/PMAT-123").as_deref(),
            Some("PMAT-123")
        );
    }

    #[test]
    fn near_misses_are_not() {
        for b in [
            "main",
            "work/PMAT-",
            "work/PMAT-12a",
            "work/PMAT-1-x",
            "feat/PMAT-1",
            "work/pmat-1",
            "",
        ] {
            assert_eq!(ticket_from_branch(b), None, "{b}");
        }
    }

    #[test]
    fn explicit_id_wins_without_git() {
        let d = std::env::temp_dir();
        assert_eq!(
            resolve_ticket_id(Some("PMAT-9".into()), &d).unwrap(),
            "PMAT-9"
        );
    }
}
