//! What did `.pmat-ratchet.toml` say before this change?
//!
//! `FALSIFY-2102-3` — "raising a baseline requires a justification" — is only
//! checkable against a previous version of the file. Reading it from git is
//! what makes the ratchet non-loosenable: the number on disk is compared with
//! the number the repository last agreed to.

use std::path::Path;
use std::process::Command;

/// The previous committed content of a tracked file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prior {
    /// The most recent committed version that differs from what is on disk.
    Content(String),
    /// The file has no committed history: nothing to have been raised FROM.
    /// This is not a hole — a brand-new baseline file cannot have loosened
    /// anything — but it is also not evidence, and the report says so.
    NoHistory,
    /// git could not answer. A hole: fail closed.
    Unavailable(String),
}

/// How far back to look for a differing version. A baseline file edited more
/// than this many times without the gate running is not a case worth guessing
/// at, and an unbounded `git log` on a large repository is not free.
const MAX_REVISIONS: usize = 50;

/// The newest committed version of `rel` whose content differs from `current`.
///
/// Comparing against "the newest committed version that differs" rather than
/// "HEAD" is what makes the check work in both places it has to: on a dirty
/// working tree (the edit is not committed yet) and in CI after the commit
/// landed (the edit IS the newest version, so the answer must be the one
/// before it).
pub fn prior_version(project_path: &Path, rel: &str, current: Option<&str>) -> Prior {
    if let Err(e) = git(project_path, &["rev-parse", "--git-dir"]) {
        return Prior::Unavailable(e);
    }
    let log = match git(
        project_path,
        &["log", "--format=%H", "-n", "200", "--", rel],
    ) {
        Ok(out) => out,
        Err(e) => return Prior::Unavailable(e),
    };
    let mut seen = 0usize;
    for hash in log.lines().filter(|l| !l.trim().is_empty()) {
        seen += 1;
        if seen > MAX_REVISIONS {
            break;
        }
        let blob = match git(project_path, &["show", &format!("{}:{}", hash.trim(), rel)]) {
            Ok(b) => b,
            // A revision where the file did not exist (it was added later on
            // this path) is not an error; keep walking.
            Err(_) => continue,
        };
        match current {
            Some(cur) if blob == cur => continue,
            _ => return Prior::Content(blob),
        }
    }
    Prior::NoHistory
}

/// Was `rel` ever committed? Used to tell "this project has no ratchet" from
/// "somebody deleted the ratchet", which are the same file-not-found to
/// everything except git.
pub fn was_ever_committed(project_path: &Path, rel: &str) -> Result<bool, String> {
    git(project_path, &["rev-parse", "--git-dir"])?;
    let log = git(project_path, &["log", "--format=%H", "-n", "1", "--", rel])?;
    Ok(!log.trim().is_empty())
}

fn git(project_path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_path)
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
