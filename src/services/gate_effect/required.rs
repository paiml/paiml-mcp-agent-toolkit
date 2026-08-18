//! Where the list of REQUIRED STATUS CHECK CONTEXT STRINGS comes from.
//!
//! This is the input the whole rule is anchored to, so it fails closed: an
//! unfetchable API, an absent manifest, or an empty list is an error, never a
//! pass. "We could not find out what gates this repo" and "nothing gates this
//! repo" are both failures, and they are reported as different failures.
//!
//! Sources, in order:
//!
//! 1. `PMAT_REQUIRED_STATUS_CHECKS` — comma-separated, explicit override. Used
//!    by fixtures and by CI runners that already know the answer.
//! 2. `.github/required-status-checks.txt` — one context per line, committed.
//! 3. the GitHub branch-protection API via `gh`.
//!
//! When 2 and 3 both answer they must agree. A committed manifest that has
//! drifted from live branch protection is worse than no manifest: it is a
//! confident wrong answer, so disagreement is an error rather than a merge.

use std::path::Path;

/// Where a context list was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextSource {
    Env,
    Manifest,
    GitHubApi,
}

impl ContextSource {
    pub fn label(&self) -> &'static str {
        match self {
            ContextSource::Env => "PMAT_REQUIRED_STATUS_CHECKS",
            ContextSource::Manifest => ".github/required-status-checks.txt",
            ContextSource::GitHubApi => "GitHub branch-protection API",
        }
    }
}

/// The required contexts and the source that produced them.
#[derive(Debug, Clone)]
pub struct RequiredContexts {
    pub contexts: Vec<String>,
    pub source: ContextSource,
}

pub const ENV_VAR: &str = "PMAT_REQUIRED_STATUS_CHECKS";
pub const MANIFEST: &str = ".github/required-status-checks.txt";

/// Resolve the required contexts, or explain why they could not be resolved.
pub fn resolve(project_path: &Path) -> Result<RequiredContexts, String> {
    let raw = std::env::var_os(ENV_VAR).map(|v| v.to_string_lossy().into_owned());
    resolve_with_override(raw.as_deref(), project_path)
}

/// [`resolve`], with the environment read for it.
///
/// The env lookup lives in the caller so that the fail-closed cases can be
/// tested without mutating process state: `set_var` is `unsafe` and racy, and a
/// test that has to disarm the environment to reach its assertion tends to get
/// its assertion deleted the first time it flakes.
pub fn resolve_with_override(
    env_override: Option<&str>,
    project_path: &Path,
) -> Result<RequiredContexts, String> {
    if let Some(raw) = env_override {
        return non_empty(split_list(raw), ContextSource::Env);
    }
    let manifest = read_manifest(project_path);
    let live = fetch_live(project_path);
    match (manifest, live) {
        (Some(m), Some(l)) if drifted(&m, &l) => Err(format!(
            "required-check drift: {MANIFEST} says [{}] but branch protection says [{}] — \
             one of them is a confident wrong answer",
            m.join(", "),
            l.join(", ")
        )),
        (Some(m), _) => non_empty(m, ContextSource::Manifest),
        (None, Some(l)) => non_empty(l, ContextSource::GitHubApi),
        (None, None) => Err(format!(
            "no required status check contexts could be resolved: {ENV_VAR} unset, \
             {MANIFEST} absent, and the GitHub branch-protection API was unreachable \
             (needs `gh auth login` and admin read on the repo)"
        )),
    }
}

fn non_empty(contexts: Vec<String>, source: ContextSource) -> Result<RequiredContexts, String> {
    if contexts.is_empty() {
        return Err(format!(
            "{} produced an empty required-check list: nothing gates this repository",
            source.label()
        ));
    }
    Ok(RequiredContexts { contexts, source })
}

/// Drift is a difference in the *set* of contexts. Branch protection does not
/// order its list meaningfully, so a manifest that lists the same four checks
/// in a different order has not drifted — and reporting that as drift would
/// train people to ignore the one error that means something.
fn drifted(manifest: &[String], live: &[String]) -> bool {
    let norm = |v: &[String]| {
        let mut c: Vec<String> = v.to_vec();
        c.sort();
        c.dedup();
        c
    };
    norm(manifest) != norm(live)
}

fn split_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn read_manifest(project_path: &Path) -> Option<Vec<String>> {
    let text = std::fs::read_to_string(project_path.join(MANIFEST)).ok()?;
    Some(
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect(),
    )
}

/// Ask GitHub. `None` on any failure — the caller turns that into an error when
/// no other source answered.
fn fetch_live(project_path: &Path) -> Option<Vec<String>> {
    let nwo = gh_json(project_path, &["repo", "view", "--json", "nameWithOwner"])?;
    let repo = json_string(&nwo, "nameWithOwner")?;
    let branch = default_branch(project_path)?;
    let out = run_gh(
        project_path,
        &[
            "api",
            &format!("repos/{repo}/branches/{branch}/protection"),
            "--jq",
            ".required_status_checks.contexts[]?",
        ],
    )?;
    Some(
        out.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

fn default_branch(project_path: &Path) -> Option<String> {
    let v = gh_json(
        project_path,
        &["repo", "view", "--json", "defaultBranchRef"],
    )?;
    v.get("defaultBranchRef")?
        .get("name")?
        .as_str()
        .map(str::to_string)
}

fn gh_json(project_path: &Path, args: &[&str]) -> Option<serde_json::Value> {
    serde_json::from_str(&run_gh(project_path, args)?).ok()
}

fn json_string(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(str::to_string)
}

fn run_gh(project_path: &Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("gh")
        .args(args)
        .current_dir(project_path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}
