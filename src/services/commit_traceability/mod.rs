//! CB-2113 — commit traceability: every non-merge commit in the range under
//! judgement carries a `Pmat-Ticket:` trailer naming a real, non-terminal
//! roadmap item (`docs/specifications/goal-mode.md` §7, §8.4, §9).
//!
//! The `commit-msg` hook refuses an untrailered commit at the point of action,
//! but a hook only exists where it was installed. This module is the CI
//! backstop for the uninstalled hook: it reads git, not the hook.
//!
//! Which commits are judged is decided by where HEAD sits (§8.4). On a branch
//! that is not the default branch — every pull request, every merge-queue
//! entry — the range is `merge-base(base, HEAD)..HEAD`, the commits the pull
//! request would add. On the default branch itself the commits since the
//! latest `v*` tag are COUNTED and reported but not judged: until merges are
//! restricted to merge commits a squash rewrites the trailers, so master
//! cannot be held to a property its merge method does not preserve. That
//! verdict is `not_applicable`, printed with the count, never a silent pass.
//!
//! Offline by construction (goal-mode.md doctrine 4): `git` and the roadmap
//! file, nothing else.
//!
//! Engine only. The comply check that reports it is
//! `src/cli/handlers/comply_handlers/check_handlers/check_traceability.rs`.

use std::path::Path;

#[cfg(test)]
mod tests;

/// The roadmap the trailer ids are resolved against.
pub const ROADMAP_PATH: &str = "docs/roadmaps/roadmap.yaml";

/// The trailer key, exactly as `git interpret-trailers` spells it.
pub const TRAILER_KEY: &str = "Pmat-Ticket";

/// Environment variable GitHub Actions sets on `pull_request` runs, naming
/// the base branch. Read by [`measure`]; [`measure_against`] takes it as an
/// argument so tests never touch the process environment.
pub const BASE_REF_ENV: &str = "GITHUB_BASE_REF";

/// Whether the two inputs exist at all. A project with neither a repository
/// nor a roadmap has nothing for this rule to read, which is a structural
/// absence (Skip), not a measurement failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inputs {
    /// `project_path` is not inside a git work tree.
    NotGit,
    /// A repository, but no `docs/roadmaps/roadmap.yaml`.
    NoRoadmap,
    /// Both present; [`measure`] can run.
    Ready,
}

/// Which commits were judged, and against what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range {
    /// HEAD is not the default branch: the commits `merge_base..HEAD` would add
    /// to `base`.
    PullRequest {
        /// The ref the base was resolved to (`origin/master`, `master`, …).
        base: String,
        /// Full hash of `merge-base(base, HEAD)`.
        merge_base: String,
    },
    /// HEAD is the default branch: `since..HEAD` is counted, not judged (§8.4).
    DefaultBranch {
        /// The latest `v*` tag, or `None` when the repository has none.
        since: Option<String>,
    },
}

/// Why one commit fails the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    /// No `Pmat-Ticket:` trailer at all.
    NoTrailer,
    /// A trailer names an id the roadmap does not contain.
    UnknownTicket(String),
    /// A trailer names an item that is already `completed` or `cancelled`.
    TerminalTicket { id: String, status: String },
}

/// One commit, one violation. A commit with two bad trailers yields two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Full hash.
    pub hash: String,
    /// First line of the message.
    pub subject: String,
    pub violation: Violation,
}

/// The measurement. `findings` is empty in [`Range::DefaultBranch`] mode by
/// construction — those commits are counted, not judged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    pub range: Range,
    /// Non-merge commits in the range.
    pub commits: usize,
    /// Of those, how many carry at least one `Pmat-Ticket:` trailer.
    pub trailered: usize,
    pub findings: Vec<Finding>,
}

impl Measurement {
    /// True when the range was judged and nothing in it violates the rule. A
    /// default-branch measurement is never `clean` — it was not judged.
    pub fn clean(&self) -> bool {
        matches!(self.range, Range::PullRequest { .. }) && self.findings.is_empty()
    }
}

/// Are the inputs there?
pub fn inputs(_project_path: &Path) -> Inputs {
    unimplemented_stub("inputs")
}

/// Measure, resolving the base from [`BASE_REF_ENV`] when it is set and from
/// the default branch otherwise.
///
/// `Err` is *not measured* — git failed, no base ref resolves, or the roadmap
/// does not parse — and the caller must FAIL on it (goal-mode.md doctrine 2).
pub fn measure(project_path: &Path) -> Result<Measurement, String> {
    let env_base = std::env::var(BASE_REF_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty());
    measure_against(project_path, env_base.as_deref())
}

/// [`measure`] with the base branch named explicitly (`None` = discover it:
/// `origin/HEAD`, then `origin/master`, `origin/main`, `master`, `main`).
pub fn measure_against(_project_path: &Path, _base: Option<&str>) -> Result<Measurement, String> {
    unimplemented_stub("measure_against")
}

/// RED scaffold (PMAT-719): the API is fixed so the falsification suite
/// compiles and fails on its assertions rather than on the build. Replaced by
/// the implementation in the same ticket; nothing may ship while this exists.
fn unimplemented_stub<T>(what: &str) -> T {
    panic!("commit_traceability::{what} is not implemented yet (PMAT-719 RED)")
}
