//! Git plumbing for commit traceability: run git, resolve the base ref.

use std::path::Path;
use std::process::Command;

pub(super) fn run_git(project_path: &Path, args: &[&str]) -> Result<String, String> {
    run_git_raw(project_path, args).map(|text| text.trim().to_string())
}

/// `git args` stdout exactly as printed (leading indentation and the final
/// newline of a file kept).
pub(super) fn run_git_raw(project_path: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(project_path)
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("git {:?} failed: {}", args, e))?;
    if !out.status.success() {
        return Err(format!(
            "git {:?}: {}",
            args.first().unwrap_or(&""),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Resolve the base ref: the named one (bare or `origin/`-prefixed), else the
/// first discovered candidate.
pub(super) fn resolve_base(project_path: &Path, base: Option<&str>) -> Result<String, String> {
    let rev_ok = |rev: &str| {
        run_git(
            project_path,
            &["rev-parse", "--verify", "-q", &format!("{rev}^{{commit}}")],
        )
        .is_ok()
    };
    if let Some(b) = base {
        if rev_ok(b) {
            return Ok(b.to_string());
        }
        let prefixed = format!("origin/{b}");
        if rev_ok(&prefixed) {
            return Ok(prefixed);
        }
        return Err(format!("base {b} not found"));
    }
    let mut candidates = vec![];
    if let Ok(sym_ref) = run_git(
        project_path,
        &["symbolic-ref", "-q", "refs/remotes/origin/HEAD"],
    ) {
        if let Some(stripped) = sym_ref.strip_prefix("refs/remotes/") {
            candidates.push(stripped.to_string());
        }
    }
    candidates.extend(["origin/master", "origin/main", "master", "main"].map(String::from));
    candidates
        .into_iter()
        .find(|c| rev_ok(c))
        .ok_or_else(|| "no base to measure against".to_string())
}
