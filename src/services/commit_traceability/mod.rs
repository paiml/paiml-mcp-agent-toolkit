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
use std::process::Command;

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

/// PMAT-1356 — is this author a GitHub app account?
///
/// GitHub mints `<numeric id>+<app name>[bot]@users.noreply.github.com` for
/// app accounts and for nothing else: the `[bot]` suffix is reserved in the
/// account namespace, and the host is GitHub's own noreply domain. Both halves
/// are required here, because either alone is an address a person can choose.
///
/// This is not a signature check and is not offered as one — a commit claiming
/// the address is still just a commit. It is why [`Measurement::bot_exempt`]
/// is COUNTED and named in the verdict message: an exemption nobody can see is
/// indistinguishable from a rule that stopped running (goal-mode.md doctrine 2).
pub fn is_github_bot_account(author_email: &str) -> bool {
    let email = author_email.trim().to_ascii_lowercase();
    let Some(local) = email.strip_suffix("@users.noreply.github.com") else {
        return false;
    };
    local.ends_with("[bot]")
}

/// Whether the two inputs exist at all. A project with neither a repository
/// nor a roadmap has nothing for this rule to read, which is a structural
/// absence (Skip), not a measurement failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inputs {
    /// `project_path` is not inside a git work tree.
    NotGit,
    /// A repository that has never committed `docs/roadmaps/roadmap.yaml`: a
    /// structural absence, not a measurement failure.
    NoRoadmap,
    /// `docs/roadmaps/roadmap.yaml` was committed and is now gone — from the
    /// working tree, or removed in history. Deleting a gate's input is not a
    /// way of passing it (the line CB-2102 draws for its own input), so the
    /// check FAILS on this rather than skipping.
    RoadmapDeleted,
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
    /// Of those, how many were authored by a GitHub app account and so are not
    /// required to carry a trailer (PMAT-1356). Counted, never silent: a bot
    /// composes its own message and cannot name a roadmap item, so a rule that
    /// demanded one of it was inoperative for that whole class, not strict.
    pub bot_exempt: usize,
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
pub fn inputs(project_path: &Path) -> Inputs {
    let out = Command::new("git")
        .arg("-C")
        .arg(project_path)
        .args(["rev-parse", "--is-inside-work-tree"])
        .env("LC_ALL", "C")
        .output();
    if match out {
        Ok(o) => !o.status.success(),
        Err(_) => true,
    } {
        return Inputs::NotGit;
    }
    if !project_path.join(ROADMAP_PATH).exists() {
        // Never committed ⇒ absent by design. Committed at any point ⇒ deleted.
        // An unreadable history is treated as "never": the fail-closed arm
        // belongs to the deletion, and a repository with no history at all has
        // nothing to have deleted.
        return match crate::services::metrics_ratchet::history::was_ever_committed(
            project_path,
            ROADMAP_PATH,
        ) {
            Ok(true) => Inputs::RoadmapDeleted,
            Ok(false) | Err(_) => Inputs::NoRoadmap,
        };
    }
    Inputs::Ready
}

fn run_git(project_path: &Path, args: &[&str]) -> Result<String, String> {
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
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
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
pub fn measure_against(project_path: &Path, base: Option<&str>) -> Result<Measurement, String> {
    let resolved_base = if let Some(b) = base {
        if run_git(
            project_path,
            &["rev-parse", "--verify", "-q", &format!("{b}^{{commit}}")],
        )
        .is_ok()
        {
            b.to_string()
        } else if run_git(
            project_path,
            &[
                "rev-parse",
                "--verify",
                "-q",
                &format!("origin/{b}^{{commit}}"),
            ],
        )
        .is_ok()
        {
            format!("origin/{b}")
        } else {
            return Err(format!("base {b} not found"));
        }
    } else {
        let mut candidates = vec![];
        if let Ok(sym_ref) = run_git(
            project_path,
            &["symbolic-ref", "-q", "refs/remotes/origin/HEAD"],
        ) {
            if let Some(stripped) = sym_ref.strip_prefix("refs/remotes/") {
                candidates.push(stripped.to_string());
            }
        }
        candidates.extend_from_slice(&[
            "origin/master".to_string(),
            "origin/main".to_string(),
            "master".to_string(),
            "main".to_string(),
        ]);

        let mut found = None;
        for c in candidates {
            if run_git(
                project_path,
                &["rev-parse", "--verify", "-q", &format!("{c}^{{commit}}")],
            )
            .is_ok()
            {
                found = Some(c);
                break;
            }
        }
        found.ok_or_else(|| "no base to measure against".to_string())?
    };

    let head_commit = run_git(project_path, &["rev-parse", "HEAD"])?;
    let base_commit = run_git(
        project_path,
        &["rev-parse", &format!("{resolved_base}^{{commit}}")],
    )?;
    let current_branch =
        run_git(project_path, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
    let base_short = resolved_base
        .strip_prefix("origin/")
        .unwrap_or(&resolved_base);

    let is_default_branch = head_commit == base_commit || current_branch == base_short;

    if is_default_branch {
        default_branch_mode(project_path)
    } else {
        pr_mode(project_path, resolved_base)
    }
}

fn default_branch_mode(project_path: &Path) -> Result<Measurement, String> {
    let since = run_git(
        project_path,
        &["describe", "--tags", "--abbrev=0", "--match", "v*", "HEAD"],
    )
    .ok();

    let range_arg = if let Some(ref s) = since {
        format!("{s}..HEAD")
    } else {
        "HEAD".to_string()
    };

    let commits_out = run_git(
        project_path,
        &[
            "log",
            "--no-merges",
            "--format=%H%x1f%s%x1f%(trailers:key=Pmat-Ticket,valueonly,separator=%x2c)%x1f%ae",
            &range_arg,
        ],
    )?;
    let mut commits = 0;
    let mut trailered = 0;
    let mut bot_exempt = 0;
    if !commits_out.is_empty() {
        for line in commits_out.split('\n') {
            if line.trim().is_empty() {
                continue;
            }
            commits += 1;
            let parts: Vec<&str> = line.split('\x1f').collect();
            let author = parts.get(3).map(|s| s.trim()).unwrap_or("");
            let bot = is_github_bot_account(author);
            let has_trailer = parts.len() >= 3 && {
                let tr = parts[2].trim();
                !tr.is_empty() && tr.split(',').any(|s| !s.trim().is_empty())
            };
            if has_trailer {
                trailered += 1;
            } else if bot {
                bot_exempt += 1;
            }
        }
    }

    Ok(Measurement {
        range: Range::DefaultBranch { since },
        commits,
        trailered,
        bot_exempt,
        findings: vec![],
    })
}

fn pr_mode(project_path: &Path, resolved_base: String) -> Result<Measurement, String> {
    use crate::models::roadmap::ItemStatus;
    let merge_base = run_git(project_path, &["merge-base", &resolved_base, "HEAD"])?;
    let commits_out = run_git(
        project_path,
        &[
            "log",
            "--no-merges",
            "--format=%H%x1f%s%x1f%(trailers:key=Pmat-Ticket,valueonly,separator=%x2c)%x1f%ae",
            &format!("{merge_base}..HEAD"),
        ],
    )?;

    let rm_path = project_path.join(ROADMAP_PATH);
    let roadmap = crate::services::roadmap_service::RoadmapService::new(rm_path)
        .load()
        .map_err(|e| format!("roadmap parse error: {}", e))?;

    let mut roadmap_map = std::collections::HashMap::new();
    for item in roadmap.roadmap {
        roadmap_map.insert(item.id, item.status);
    }

    let mut commits = 0;
    let mut trailered = 0;
    let mut bot_exempt = 0;
    let mut findings = vec![];

    if !commits_out.is_empty() {
        for line in commits_out.split('\n') {
            if line.trim().is_empty() {
                continue;
            }
            commits += 1;
            let parts: Vec<&str> = line.split('\x1f').collect();
            if parts.len() < 2 {
                continue;
            }
            let hash = parts[0].to_string();
            let subject = parts[1].to_string();
            let tr = if parts.len() >= 3 {
                parts[2].trim()
            } else {
                ""
            };

            let ids: Vec<&str> = tr
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if ids.is_empty() {
                // PMAT-1356: a GitHub app account composes its own message and
                // has no roadmap item to name — the work was published
                // upstream, not chosen here. Exempt from the REQUIREMENT, and
                // counted so the exemption is visible in the verdict. Only the
                // absence is excused: a trailer that is present is judged
                // below whoever wrote it.
                if is_github_bot_account(parts.get(3).map(|s| s.trim()).unwrap_or("")) {
                    bot_exempt += 1;
                } else {
                    findings.push(Finding {
                        hash: hash.clone(),
                        subject: subject.clone(),
                        violation: Violation::NoTrailer,
                    });
                }
            } else {
                let mut commit_trailered = false;
                for id in ids {
                    commit_trailered = true;
                    if let Some(status) = roadmap_map.get(id) {
                        // Terminal states, spelled as the roadmap serialises
                        // them (`ItemStatus` is `rename_all = "lowercase"`).
                        let terminal = match status {
                            ItemStatus::Completed => Some("completed"),
                            ItemStatus::Cancelled => Some("cancelled"),
                            _ => None,
                        };
                        if let Some(status) = terminal {
                            findings.push(Finding {
                                hash: hash.clone(),
                                subject: subject.clone(),
                                violation: Violation::TerminalTicket {
                                    id: id.to_string(),
                                    status: status.to_string(),
                                },
                            });
                        }
                    } else {
                        findings.push(Finding {
                            hash: hash.clone(),
                            subject: subject.clone(),
                            violation: Violation::UnknownTicket(id.to_string()),
                        });
                    }
                }
                if commit_trailered {
                    trailered += 1;
                }
            }
        }
    }

    Ok(Measurement {
        range: Range::PullRequest {
            base: resolved_base,
            merge_base,
        },
        commits,
        trailered,
        bot_exempt,
        findings,
    })
}
