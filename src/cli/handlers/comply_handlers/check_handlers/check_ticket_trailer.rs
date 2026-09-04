// CB-1340: Ticket Trailer — every commit on a branch carries a `Pmat-Ticket:`
// trailer naming a ticket that is in progress (AD-07, spec §8).
//
// The trailer is the machine-checkable ticket↔work record: it is git-native,
// survives rebases and squashes that keep the message, and needs no network.
// A branch name is deleted on merge and a PR body is not git, so neither is
// the record. Read with git's own trailer parser (`%(trailers:key=…)`), never
// a regex over the subject — subjects are rewritten on squash.
//
// Included by check.rs (this directory is one module stitched from include!()
// fragments), so it carries no `//!` docs and inherits check.rs's imports.

/// Run a git command in `project_path`, returning trimmed stdout on success.
fn git_stdout(project_path: &Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(project_path)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The repository's default branch: `origin/HEAD` when it is set, else the
/// first of `master` / `main` that exists locally.
/// The ref to judge against, as a revision git can resolve HERE: `origin/HEAD`'s
/// target when the remote is known, else `origin/master` / `origin/main`, else a
/// local `master` / `main`. A CI checkout usually has the remote-tracking ref and
/// no local default branch — the first cut returned the bare name and the
/// merge-base lookup failed, which read as "nothing to judge" (the AD-04 quorum
/// caught it).
fn default_branch(project_path: &Path) -> Option<String> {
    let resolves = |rev: &str| {
        git_stdout(project_path, &["rev-parse", "--verify", "--quiet", rev]).is_some()
    };
    if let Some(sym) = git_stdout(project_path, &["symbolic-ref", "refs/remotes/origin/HEAD"]) {
        if let Some(name) = sym.rsplit('/').next() {
            if !name.is_empty() {
                let remote = format!("origin/{name}");
                if resolves(&remote) {
                    return Some(remote);
                }
                if resolves(name) {
                    return Some(name.to_string());
                }
            }
        }
    }
    for candidate in ["origin/master", "origin/main", "master", "main"] {
        if resolves(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

/// One `(sha, trailer values)` pair from `git log`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrailerCommit {
    pub sha: String,
    pub tickets: Vec<String>,
}

impl TrailerCommit {
    fn short(&self) -> String {
        self.sha.chars().take(7).collect()
    }
}

/// Parse `git log --format=%H%x1f%(trailers:…)%x1e` output.
///
/// Records are RS-separated (`\x1e`), fields US-separated (`\x1f`), and the
/// trailer field is a comma-separated list (a commit may carry more than one
/// `Pmat-Ticket:` line).
pub(crate) fn parse_trailer_log(raw: &str) -> Vec<TrailerCommit> {
    let mut out = Vec::new();
    for record in raw.split('\u{1e}') {
        let record = record.trim_matches(|c: char| c == '\n' || c == '\r');
        if record.trim().is_empty() {
            continue;
        }
        let mut fields = record.splitn(2, '\u{1f}');
        let sha = fields.next().unwrap_or("").trim().to_string();
        if sha.is_empty() {
            continue;
        }
        let tickets = fields
            .next()
            .unwrap_or("")
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        out.push(TrailerCommit { sha, tickets });
    }
    out
}

/// Why a trailer does not satisfy CB-1340, or `None` when it does.
fn ticket_defect(ticket: &str, roadmap: &crate::models::roadmap::Roadmap) -> Option<String> {
    let item = if let Some(number) = ticket.strip_prefix('#') {
        number
            .parse::<u64>()
            .ok()
            .and_then(|n| roadmap.roadmap.iter().find(|i| i.github_issue == Some(n)))
    } else {
        roadmap
            .roadmap
            .iter()
            .find(|i| i.id.eq_ignore_ascii_case(ticket))
    };
    match item {
        None => Some(format!("{ticket} is not in the roadmap")),
        Some(item) if item.status == crate::models::roadmap::ItemStatus::InProgress => None,
        Some(item) => Some(format!(
            "{ticket} is {}, not in progress",
            item.status.display_name()
        )),
    }
}

/// Judge one commit: `None` when it carries a trailer naming an in-progress
/// ticket, else the reason it fails.
fn commit_defect(
    commit: &TrailerCommit,
    roadmap: &crate::models::roadmap::Roadmap,
) -> Option<String> {
    if commit.tickets.is_empty() {
        return Some(format!("{} (no Pmat-Ticket trailer)", commit.short()));
    }
    // Every ticket the commit names must be in progress: a commit that names
    // one live ticket and one unknown or completed one is still mislinked (the
    // AD-04 quorum caught the first cut accepting it on the live one alone).
    let reasons: Vec<String> = commit
        .tickets
        .iter()
        .filter_map(|ticket| ticket_defect(ticket, roadmap))
        .collect();
    if reasons.is_empty() {
        None
    } else {
        Some(format!("{} ({})", commit.short(), reasons.join("; ")))
    }
}

/// The verdict over a whole branch, given the commits and the roadmap.
/// Separated from the git/IO plumbing so it is directly unit-testable.
pub(crate) fn judge_trailers(
    commits: &[TrailerCommit],
    roadmap: &crate::models::roadmap::Roadmap,
) -> ComplianceCheck {
    let defects: Vec<String> = commits.iter().filter_map(|c| commit_defect(c, roadmap)).collect();
    if defects.is_empty() {
        let tickets: std::collections::BTreeSet<&str> = commits
            .iter()
            .flat_map(|c| c.tickets.iter().map(|t| t.as_str()))
            .collect();
        return ComplianceCheck {
            name: "CB-1340: Ticket Trailer".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} commit(s) on this branch carry a Pmat-Ticket trailer naming {} in-progress ticket(s)",
                commits.len(),
                tickets.len()
            ),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: "CB-1340: Ticket Trailer".into(),
        status: CheckStatus::Fail,
        message: format!(
            "{} of {} commit(s) lack a Pmat-Ticket trailer naming an in-progress ticket: {}",
            defects.len(),
            commits.len(),
            defects.join("; ")
        ),
        severity: Severity::Error,
    }
}

/// A PASS that says what could not be judged, rather than a silent green.
fn trailer_check_nothing_to_judge(reason: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: "CB-1340: Ticket Trailer".into(),
        status: CheckStatus::Pass,
        message: format!("Nothing to judge: {reason}"),
        severity: Severity::Info,
    }
}

/// CB-1340: every non-merge commit on the current branch carries a
/// `Pmat-Ticket:` trailer naming a roadmap ticket that is in progress.
///
/// Passes with an explicit "nothing to judge" outside a git repository, on the
/// default branch (its history is already merged and judged), on a detached
/// HEAD, and where there is no roadmap to name tickets from.
pub(crate) fn check_ticket_trailers(project_path: &Path) -> ComplianceCheck {
    if git_stdout(project_path, &["rev-parse", "--git-dir"]).is_none() {
        return trailer_check_nothing_to_judge("not a git repository");
    }
    let branch = match git_stdout(project_path, &["symbolic-ref", "--short", "HEAD"]) {
        Some(b) => b,
        None => return trailer_check_nothing_to_judge("detached HEAD"),
    };
    let default = match default_branch(project_path) {
        Some(d) => d,
        None => {
            return ComplianceCheck {
                name: "CB-1340: Ticket Trailer".into(),
                status: CheckStatus::Warn,
                message: "Cannot judge: no default branch ref resolves (origin/HEAD, origin/master, origin/main, master, main) — fetch the default branch so the merge base can be found".into(),
                severity: Severity::Warning,
            };
        }
    };
    if branch == default || default.strip_prefix("origin/") == Some(branch.as_str()) {
        return trailer_check_nothing_to_judge(format!(
            "on the default branch ({default}); its history is judged before merge"
        )
        .as_str());
    }

    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    if !roadmap_path.exists() {
        return trailer_check_nothing_to_judge("no docs/roadmaps/roadmap.yaml");
    }
    let service = crate::services::roadmap_service::RoadmapService::new(&roadmap_path);
    let roadmap = match service.load() {
        Ok(r) => r,
        Err(e) => {
            return ComplianceCheck {
                name: "CB-1340: Ticket Trailer".into(),
                status: CheckStatus::Warn,
                message: format!(
                    "Could not read {}: {}",
                    roadmap_path.display(),
                    e.to_string().lines().next().unwrap_or("parse error")
                ),
                severity: Severity::Warning,
            };
        }
    };

    let base = match git_stdout(project_path, &["merge-base", &default, "HEAD"]) {
        Some(b) => b,
        None => return trailer_check_nothing_to_judge(format!("no merge base with {default}").as_str()),
    };
    let range = format!("{base}..HEAD");
    let raw = match git_stdout(
        project_path,
        &[
            "log",
            "--no-merges",
            "--format=%H%x1f%(trailers:key=Pmat-Ticket,valueonly,separator=%x2C)%x1e",
            &range,
        ],
    ) {
        Some(r) => r,
        None => return trailer_check_nothing_to_judge(format!("git log {range} failed").as_str()),
    };
    let commits = parse_trailer_log(&raw);
    if commits.is_empty() {
        return trailer_check_nothing_to_judge(format!("no commits in {range}").as_str());
    }
    judge_trailers(&commits, &roadmap)
}

#[cfg(test)]
mod ticket_trailer_tests {
    use super::*;
    use crate::models::roadmap::{ItemStatus, Roadmap, RoadmapItem};

    pub(super) fn roadmap_with(statuses: &[(&str, ItemStatus)]) -> Roadmap {
        let items = statuses
            .iter()
            .map(|(id, status)| {
                let mut item = RoadmapItem::new((*id).to_string(), (*id).to_string());
                item.status = *status;
                item
            })
            .collect();
        Roadmap {
            roadmap: items,
            ..Default::default()
        }
    }

    #[test]
    fn parses_sha_and_multiple_trailers() {
        let raw = "aaaa\u{1f}PMAT-1\u{1e}\nbbbb\u{1f}PMAT-1,PMAT-2\u{1e}\ncccc\u{1f}\u{1e}";
        let parsed = parse_trailer_log(raw);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].tickets, vec!["PMAT-1".to_string()]);
        assert_eq!(parsed[1].tickets.len(), 2);
        assert!(parsed[2].tickets.is_empty(), "untrailered commit has no tickets");
    }

    #[test]
    fn in_progress_trailer_passes() {
        let roadmap = roadmap_with(&[("PMAT-1", ItemStatus::InProgress)]);
        let commits = vec![TrailerCommit {
            sha: "abcdef1234567890".into(),
            tickets: vec!["PMAT-1".into()],
        }];
        let check = judge_trailers(&commits, &roadmap);
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains('1'), "{}", check.message);
    }

    #[test]
    fn missing_trailer_fails_naming_the_sha() {
        let roadmap = roadmap_with(&[("PMAT-1", ItemStatus::InProgress)]);
        let commits = vec![TrailerCommit {
            sha: "abcdef1234567890".into(),
            tickets: vec![],
        }];
        let check = judge_trailers(&commits, &roadmap);
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("abcdef1"), "{}", check.message);
        assert!(check.message.contains("no Pmat-Ticket trailer"), "{}", check.message);
    }

    #[test]
    fn completed_ticket_fails() {
        let roadmap = roadmap_with(&[("PMAT-2", ItemStatus::Completed)]);
        let commits = vec![TrailerCommit {
            sha: "0123456789abcdef".into(),
            tickets: vec!["PMAT-2".into()],
        }];
        let check = judge_trailers(&commits, &roadmap);
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("0123456"), "{}", check.message);
        assert!(check.message.contains("not in progress"), "{}", check.message);
    }

    #[test]
    fn planned_ticket_and_unknown_ticket_both_fail() {
        let roadmap = roadmap_with(&[("PMAT-3", ItemStatus::Planned)]);
        let planned = judge_trailers(
            &[TrailerCommit {
                sha: "1111111111".into(),
                tickets: vec!["PMAT-3".into()],
            }],
            &roadmap,
        );
        assert_eq!(planned.status, CheckStatus::Fail);
        let unknown = judge_trailers(
            &[TrailerCommit {
                sha: "2222222222".into(),
                tickets: vec!["PMAT-9".into()],
            }],
            &roadmap,
        );
        assert_eq!(unknown.status, CheckStatus::Fail);
        assert!(unknown.message.contains("not in the roadmap"), "{}", unknown.message);
    }

    #[test]
    fn issue_number_trailer_resolves_through_github_issue() {
        let mut item = RoadmapItem::new("PMAT-4".to_string(), "t".to_string());
        item.status = ItemStatus::InProgress;
        item.github_issue = Some(1234);
        let roadmap = Roadmap {
            roadmap: vec![item],
            ..Default::default()
        };
        let ok = judge_trailers(
            &[TrailerCommit {
                sha: "3333333333".into(),
                tickets: vec!["#1234".into()],
            }],
            &roadmap,
        );
        assert_eq!(ok.status, CheckStatus::Pass, "{}", ok.message);
        let bad = judge_trailers(
            &[TrailerCommit {
                sha: "4444444444".into(),
                tickets: vec!["#9999".into()],
            }],
            &roadmap,
        );
        assert_eq!(bad.status, CheckStatus::Fail);
    }

    // ---- end-to-end over a real temporary repository ----

    fn git(dir: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    fn fixture_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join("docs/roadmaps")).expect("mkdir");
        let mut one = RoadmapItem::new("PMAT-1".to_string(), "in progress".to_string());
        one.status = ItemStatus::InProgress;
        let mut two = RoadmapItem::new("PMAT-2".to_string(), "completed".to_string());
        two.status = ItemStatus::Completed;
        let roadmap = Roadmap {
            roadmap: vec![one, two],
            ..Default::default()
        };
        crate::services::roadmap_service::RoadmapService::new(dir.join("docs/roadmaps/roadmap.yaml"))
            .save(&roadmap)
            .expect("save roadmap");
        git(dir, &["init", "-q", "--template=", "-b", "master"]);
        git(dir, &["config", "user.email", "t@t"]);
        git(dir, &["config", "user.name", "t"]);
        git(dir, &["config", "commit.gpgsign", "false"]);
        std::fs::write(dir.join("a.txt"), "a").expect("write");
        git(dir, &["add", "."]);
        git(dir, &["commit", "-qm", "init"]);
    }

    fn commit_with(dir: &Path, body: &str) {
        let file = dir.join("a.txt");
        let prev = std::fs::read_to_string(&file).unwrap_or_default();
        std::fs::write(&file, format!("{prev}\n{body}")).expect("write");
        git(dir, &["commit", "-qam", body]);
    }

    #[test]
    fn end_to_end_branch_and_default_branch() {
        let tmp = tempfile::tempdir().expect("tmp");
        let dir = tmp.path();
        fixture_repo(dir);

        // The control: on the default branch the check does not judge history.
        let on_master = check_ticket_trailers(dir);
        assert_eq!(on_master.status, CheckStatus::Pass, "{}", on_master.message);
        assert!(on_master.message.contains("default branch"), "{}", on_master.message);

        git(dir, &["switch", "-q", "-c", "PMAT-1-work"]);
        commit_with(dir, "one\n\nPmat-Ticket: PMAT-1");
        let green = check_ticket_trailers(dir);
        assert_eq!(green.status, CheckStatus::Pass, "{}", green.message);

        commit_with(dir, "two with no trailer");
        let sha = git_stdout(dir, &["rev-parse", "--short", "HEAD"]).expect("sha");
        let red = check_ticket_trailers(dir);
        assert_eq!(red.status, CheckStatus::Fail, "{}", red.message);
        assert!(red.message.contains(&sha), "message names {sha}: {}", red.message);
    }

    #[test]
    fn outside_a_git_repository_there_is_nothing_to_judge() {
        let tmp = tempfile::tempdir().expect("tmp");
        let check = check_ticket_trailers(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("not a git repository"), "{}", check.message);
    }
}

#[cfg(test)]
mod quorum_findings_tests {
    //! Two findings of the AD-04 quorum on the PR that introduced CB-1340.
    use super::*;
    use crate::models::roadmap::ItemStatus;

    fn run(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[test]
    fn a_commit_naming_a_live_and_a_completed_ticket_is_still_mislinked() {
        let roadmap = super::ticket_trailer_tests::roadmap_with(&[("PMAT-1", ItemStatus::InProgress), ("PMAT-2", ItemStatus::Completed)]);
        let commit = TrailerCommit {
            sha: "abcdef1234567890".into(),
            tickets: vec!["PMAT-1".into(), "PMAT-2".into()],
        };
        let defect = commit_defect(&commit, &roadmap).expect("the completed ticket must fail the commit");
        assert!(defect.contains("PMAT-2") && defect.contains("not in progress"), "{defect}");
        let clean = TrailerCommit { sha: "abcdef1234567890".into(), tickets: vec!["PMAT-1".into()] };
        assert!(commit_defect(&clean, &roadmap).is_none(), "the control: one live ticket passes");
    }

    #[test]
    fn a_checkout_with_only_remote_refs_is_judged_not_passed() {
        // origin: master with one commit; clone: only refs/remotes/origin/*, a feature branch, no local master
        let origin = tempfile::tempdir().expect("tempdir");
        let o = origin.path();
        run(o, &["init", "-q", "--template=", "-b", "master"]);
        run(o, &["config", "user.email", "t@t"]);
        run(o, &["config", "user.name", "t"]);
        std::fs::write(o.join("a.txt"), "a\n").expect("write");
        run(o, &["add", "."]);
        run(o, &["commit", "-q", "-m", "init"]);
        let clone = tempfile::tempdir().expect("tempdir");
        let c = clone.path().join("c");
        run(o, &["clone", "-q", "--template=", o.to_str().expect("utf-8 path"), c.to_str().expect("utf-8 path")]);
        run(&c, &["config", "user.email", "t@t"]);
        run(&c, &["config", "user.name", "t"]);
        run(&c, &["switch", "-q", "-c", "PMAT-1-work"]);
        run(&c, &["branch", "-D", "master"]); // the CI shape: the default branch exists only as origin/master
        assert!(git_stdout(&c, &["rev-parse", "--verify", "--quiet", "master"]).is_none(), "the control: no local master");
        let d = default_branch(&c).expect("origin/master must resolve");
        assert_eq!(d, "origin/master");
        std::fs::write(c.join("a.txt"), "b\n").expect("write");
        run(&c, &["commit", "-q", "-am", "no trailer"]);
        std::fs::create_dir_all(c.join("docs/roadmaps")).expect("mkdir");
        std::fs::write(
            c.join("docs/roadmaps/roadmap.yaml"),
            "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: fx/fx\nroadmap: []\n",
        )
        .expect("write");
        let check = check_ticket_trailers(&c);
        assert!(
            matches!(check.status, CheckStatus::Fail),
            "an untrailered commit on a remote-only checkout must FAIL, not read as nothing to judge: {} — {}",
            check.name,
            check.message
        );
    }
}

