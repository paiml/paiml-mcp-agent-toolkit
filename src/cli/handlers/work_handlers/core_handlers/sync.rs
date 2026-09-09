#![cfg_attr(coverage_nightly, coverage(off))]
//! `pmat work sync` (goal-mode.md §5, PMAT-720).
//!
//! Judge the roadmap/GitHub bijection from a snapshot, then fix the orphans in
//! the chosen direction. The engine (`services::work_sync`) is pure; this file
//! is the only place that reads the network (`gh`), the roadmap file, or writes
//! either. `--check-only` is the gate-shaped mode: it reports every finding and
//! exits 1 when any exists. A `COLLISION` is reported and never fixed (§5.4).

use super::github::{create_github_issue_from_item, detect_github_repo};
use crate::cli::colors as c;
use crate::cli::commands::SyncDirection;
use crate::services::roadmap_service::RoadmapService;
use crate::services::work_sync::{
    self as engine, Action, CloseReason, Direction, Finding, GithubSnapshot, IssueSnapshot,
    IssueState, MilestoneSnapshot, Settings, SyncReport,
};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::process::Command;

/// Everything `pmat work sync` was asked to do.
#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub direction: SyncDirection,
    pub path: Option<PathBuf>,
    /// Print the plan, write nothing.
    pub dry_run: bool,
    /// Report only; exit 1 on any finding; `direction` is ignored.
    pub check_only: bool,
    /// Read the GitHub snapshot from this file instead of `gh`.
    pub snapshot: Option<PathBuf>,
    /// Write the snapshot that was judged to this file.
    pub write_snapshot: Option<PathBuf>,
    /// §5.2 grace window.
    pub grace_minutes: i64,
    /// One JSON document on stdout instead of the text report.
    pub json: bool,
}

/// Handle `pmat work sync`.
pub async fn handle_work_sync(opts: SyncOptions) -> Result<()> {
    let project_path = opts.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);
    let mut roadmap = service.load()?;
    let repo = match roadmap.github_repo.clone() {
        Some(r) => r,
        None => detect_github_repo(&project_path)?.context(
            "no GitHub repository: set github_repo in docs/roadmaps/roadmap.yaml or add an origin remote",
        )?,
    };

    let snapshot = match &opts.snapshot {
        Some(p) => {
            let text = std::fs::read_to_string(p)
                .with_context(|| format!("cannot read snapshot {}", p.display()))?;
            GithubSnapshot::from_json(&text)?
        }
        None => fetch_snapshot(&repo)?,
    };
    if let Some(p) = &opts.write_snapshot {
        std::fs::write(p, snapshot.to_json()?)
            .with_context(|| format!("cannot write snapshot {}", p.display()))?;
    }

    let settings = Settings::new(Utc::now(), opts.grace_minutes);
    let report = engine::check(&roadmap, &snapshot, &settings);

    if opts.check_only {
        if opts.json {
            println!("{}", report_json(&report, &snapshot, opts.grace_minutes)?);
        } else {
            print_report(&report, &snapshot, opts.grace_minutes);
        }
        return verdict(&report);
    }

    let direction = match opts.direction {
        SyncDirection::YamlToGithub => Direction::YamlToGithub,
        SyncDirection::GithubToYaml => Direction::GithubToYaml,
        SyncDirection::Full => Direction::Full,
    };
    let actions = engine::plan(&roadmap, &snapshot, &report, direction);
    if !opts.json {
        print_report(&report, &snapshot, opts.grace_minutes);
        print_plan(&actions, opts.direction, opts.dry_run);
    }

    let mut applied = 0;
    let mut opened: Vec<(String, u64)> = Vec::new();
    if !opts.dry_run {
        let mut to_apply = Vec::with_capacity(actions.len());
        for action in &actions {
            if let Action::CreateIssue { id, .. } = action {
                let item = roadmap
                    .find_item(id)
                    .cloned()
                    .with_context(|| format!("{id} vanished from the roadmap mid-sync"))?;
                let info = create_github_issue_from_item(&repo, &item).await?;
                if !opts.json {
                    println!(
                        "   {} #{} opened for {}",
                        c::pass(""),
                        info.number,
                        c::path(id)
                    );
                }
                opened.push((id.clone(), info.number));
                to_apply.push(Action::LinkIssue {
                    id: id.clone(),
                    number: info.number,
                });
            } else {
                to_apply.push(action.clone());
            }
        }
        applied = engine::apply_to_roadmap(&mut roadmap, &to_apply, Utc::now());
        if applied > 0 {
            service.save(&roadmap)?;
        }
        if !opts.json {
            println!(
                "   {} roadmap change(s) written to {}",
                c::number(&applied.to_string()),
                c::path(&roadmap_path.display().to_string())
            );
        }
    }

    if opts.json {
        let doc = serde_json::json!({
            "repo": snapshot.repo,
            "taken_at": snapshot.taken_at,
            "direction": direction_name(opts.direction),
            "dry_run": opts.dry_run,
            "report": report,
            "actions": actions,
            "opened": opened.iter().map(|(id, n)| serde_json::json!({"id": id, "number": n})).collect::<Vec<_>>(),
            "applied": applied,
        });
        println!("{}", serde_json::to_string_pretty(&doc)?);
    }
    Ok(())
}

fn direction_name(d: SyncDirection) -> &'static str {
    match d {
        SyncDirection::YamlToGithub => "yaml-to-github",
        SyncDirection::GithubToYaml => "github-to-yaml",
        SyncDirection::Full => "full",
    }
}

/// `--check-only`'s exit: coherent is 0, anything else is an error naming the
/// counts, so a pipeline sees the failure without parsing the report.
fn verdict(report: &SyncReport) -> Result<()> {
    if report.is_coherent() {
        return Ok(());
    }
    bail!(
        "work sync --check-only: {} finding(s) — {} COLLISION, {} ORPHAN-ROADMAP, {} ORPHAN-GITHUB, {} DRIFT; the roadmap and GitHub are not in bijection (goal-mode.md §5.1)",
        report.findings.len(),
        report.count("COLLISION"),
        report.count("ORPHAN-ROADMAP"),
        report.count("ORPHAN-GITHUB"),
        report.count("DRIFT"),
    )
}

fn report_json(report: &SyncReport, snapshot: &GithubSnapshot, grace: i64) -> Result<String> {
    let doc = serde_json::json!({
        "repo": snapshot.repo,
        "taken_at": snapshot.taken_at,
        "grace_minutes": grace,
        "open_items": report.open_items,
        "open_issues": report.open_issues,
        "matched": report.matched,
        "tolerated": report.tolerated,
        "coherent": report.is_coherent(),
        "counts": {
            "COLLISION": report.count("COLLISION"),
            "ORPHAN-ROADMAP": report.count("ORPHAN-ROADMAP"),
            "ORPHAN-GITHUB": report.count("ORPHAN-GITHUB"),
            "DRIFT": report.count("DRIFT"),
        },
        "findings": report.findings,
    });
    Ok(serde_json::to_string_pretty(&doc)?)
}

fn print_report(report: &SyncReport, snapshot: &GithubSnapshot, grace: i64) {
    println!(
        "{}",
        c::header(&format!(
            "🔄 work sync — {} · snapshot {} · grace {} min",
            snapshot.repo,
            snapshot.taken_at.to_rfc3339(),
            grace
        ))
    );
    println!(
        "   open items {} · open issues {} · matched {} · tolerated {}",
        c::number(&report.open_items.to_string()),
        c::number(&report.open_issues.to_string()),
        c::number(&report.matched.to_string()),
        c::number(&report.tolerated.to_string()),
    );
    for f in &report.findings {
        println!("   {}", render_finding(f));
    }
    if report.is_coherent() {
        println!(
            "{}",
            c::pass("coherent: R ↔ G is a bijection and no matched pair disagrees past the grace window")
        );
    } else {
        println!(
            "{}",
            c::fail(&format!(
                "{} finding(s): {} COLLISION, {} ORPHAN-ROADMAP, {} ORPHAN-GITHUB, {} DRIFT",
                report.findings.len(),
                report.count("COLLISION"),
                report.count("ORPHAN-ROADMAP"),
                report.count("ORPHAN-GITHUB"),
                report.count("DRIFT"),
            ))
        );
    }
}

fn render_finding(f: &Finding) -> String {
    match f {
        Finding::Collision { number, ids } => format!(
            "{} #{} ← {} ({} items)",
            c::fail("COLLISION     "),
            number,
            ids.join(", "),
            ids.len()
        ),
        Finding::OrphanRoadmap {
            id,
            github_issue,
            reason,
            ..
        } => {
            let why = match reason {
                engine::OrphanReason::NoIssue => "no issue".to_string(),
                engine::OrphanReason::IssueClosed => {
                    format!("#{} is closed", github_issue.unwrap_or(0))
                }
                engine::OrphanReason::IssueAbsent => {
                    format!("#{} does not exist", github_issue.unwrap_or(0))
                }
            };
            format!("{} {:<18} {}", c::warn("ORPHAN-ROADMAP"), c::path(id), why)
        }
        Finding::OrphanGithub { number, title, .. } => format!(
            "{} {:<18} {}",
            c::warn("ORPHAN-GITHUB "),
            format!("#{number}"),
            title
        ),
        Finding::Drift {
            id,
            number,
            field,
            roadmap,
            github,
            age_minutes,
        } => format!(
            "{} {} #{} {:?}: {:?} ≠ {:?} ({} min)",
            c::warn("DRIFT         "),
            c::path(id),
            number,
            field,
            roadmap,
            github,
            age_minutes
        ),
    }
}

fn print_plan(actions: &[Action], direction: SyncDirection, dry_run: bool) {
    println!();
    println!(
        "{}",
        c::subheader(&format!(
            "📋 plan (--direction {}{}): {} action(s)",
            direction_name(direction),
            if dry_run { ", dry run" } else { "" },
            actions.len()
        ))
    );
    for a in actions {
        let line = match a {
            Action::CreateIssue { id, title } => {
                format!("{} {} {:?}", c::label("create-issue"), c::path(id), title)
            }
            Action::LinkIssue { id, number } => {
                format!("{} {} → #{}", c::label("link-issue  "), c::path(id), number)
            }
            Action::CreateItem {
                number,
                title,
                release,
            } => format!(
                "{} #{} {:?} → GH-{}{}",
                c::label("create-item "),
                number,
                title,
                number,
                release
                    .as_ref()
                    .map(|r| format!(" (release {r})"))
                    .unwrap_or_default()
            ),
            Action::CloseItem { id, number, status } => format!(
                "{} {} #{} → {:?}",
                c::label("close-item  "),
                c::path(id),
                number,
                status
            ),
            Action::SetRelease {
                id,
                number,
                release,
            } => format!(
                "{} {} #{} → {}",
                c::label("set-release "),
                c::path(id),
                number,
                release.as_deref().unwrap_or("(none)")
            ),
            Action::Skip { id, reason } => {
                format!(
                    "{} {} {}",
                    c::dim("skip        "),
                    c::path(id),
                    c::dim(reason)
                )
            }
        };
        println!("   {line}");
    }
}

// ── the network edge ─────────────────────────────────────────────────────────

fn gh_json(args: &[&str]) -> Result<serde_json::Value> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .context("failed to run gh — is it installed and authenticated? (or pass --snapshot)")?;
    if !out.status.success() {
        bail!(
            "gh {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    serde_json::from_slice(&out.stdout).context("gh printed something that is not JSON")
}

/// Take a snapshot of every issue (open and closed — a closed issue must be
/// told apart from an absent one) and every milestone, through `gh`.
pub(super) fn fetch_snapshot(repo: &str) -> Result<GithubSnapshot> {
    let issues = gh_json(&[
        "issue",
        "list",
        "--repo",
        repo,
        "--state",
        "all",
        "--limit",
        "5000",
        "--json",
        "number,title,state,stateReason,labels,milestone,updatedAt",
    ])?;
    let milestones = gh_json(&[
        "api",
        &format!("repos/{repo}/milestones?state=all&per_page=100"),
    ])?;
    parse_snapshot(repo, Utc::now(), &issues, &milestones)
}

/// Turn `gh issue list --json …` and `gh api …/milestones` output into a
/// snapshot. Pure, so a fixture can prove the field mapping.
pub(super) fn parse_snapshot(
    repo: &str,
    taken_at: DateTime<Utc>,
    issues: &serde_json::Value,
    milestones: &serde_json::Value,
) -> Result<GithubSnapshot> {
    let mut out = Vec::new();
    for v in issues
        .as_array()
        .context("gh issue list did not return an array")?
    {
        let number = v["number"].as_u64().context("an issue without a number")?;
        let state = match v["state"].as_str().unwrap_or("") {
            "OPEN" => IssueState::Open,
            "CLOSED" => IssueState::Closed,
            other => bail!("issue #{number}: unknown state {other:?}"),
        };
        let state_reason = match v["stateReason"].as_str().unwrap_or("") {
            "COMPLETED" => Some(CloseReason::Completed),
            "NOT_PLANNED" => Some(CloseReason::NotPlanned),
            "DUPLICATE" => Some(CloseReason::Duplicate),
            _ => None,
        };
        let updated_at = v["updatedAt"]
            .as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc))
            .with_context(|| format!("issue #{number}: updatedAt is not RFC 3339"))?;
        out.push(IssueSnapshot {
            number,
            title: v["title"].as_str().unwrap_or("").to_string(),
            state,
            state_reason,
            labels: v["labels"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|l| l["name"].as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            milestone: v["milestone"]["title"].as_str().map(str::to_string),
            updated_at,
        });
    }
    let mut ms = Vec::new();
    for v in milestones.as_array().unwrap_or(&Vec::new()) {
        let title = match v["title"].as_str() {
            Some(t) => t.to_string(),
            None => continue,
        };
        let state = if v["state"].as_str() == Some("closed") {
            IssueState::Closed
        } else {
            IssueState::Open
        };
        ms.push(MilestoneSnapshot { title, state });
    }
    Ok(GithubSnapshot {
        repo: repo.to_string(),
        taken_at,
        issues: out,
        milestones: ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::roadmap::ItemStatus;
    use tempfile::TempDir;

    const ROADMAP: &str =
        "roadmap_version: '1.0'\ngithub_enabled: true\ngithub_repo: paiml/pmat\nroadmap:\n";

    fn project(items: &str, snapshot: &str) -> (TempDir, PathBuf, PathBuf) {
        let dir = TempDir::new().expect("tempdir");
        let roadmaps = dir.path().join("docs/roadmaps");
        std::fs::create_dir_all(&roadmaps).expect("mkdir");
        let roadmap = roadmaps.join("roadmap.yaml");
        std::fs::write(&roadmap, format!("{ROADMAP}{items}")).expect("write roadmap");
        let snap = dir.path().join("snapshot.json");
        std::fs::write(&snap, snapshot).expect("write snapshot");
        (dir, roadmap, snap)
    }

    fn opts(dir: &TempDir, snap: &PathBuf) -> SyncOptions {
        SyncOptions {
            direction: SyncDirection::Full,
            path: Some(dir.path().to_path_buf()),
            dry_run: false,
            check_only: false,
            snapshot: Some(snap.clone()),
            write_snapshot: None,
            grace_minutes: 60,
            json: false,
        }
    }

    fn snap(issues: &str) -> String {
        format!(
            r#"{{"repo":"paiml/pmat","taken_at":"2026-09-09T12:00:00Z","issues":[{issues}],"milestones":[{{"title":"3.42.0","state":"open"}}]}}"#
        )
    }

    const ORPHAN_ITEM: &str =
        "- id: A\n  github_issue: null\n  item_type: task\n  title: alpha\n  status: planned\n";
    const MATCHED_ITEM: &str =
        "- id: B\n  github_issue: 2\n  item_type: task\n  title: beta\n  status: planned\n";
    const ISSUE_TWO: &str = r#"{"number":2,"title":"beta","state":"open","labels":[],"updated_at":"2026-09-09T12:00:00Z"}"#;

    #[tokio::test]
    async fn check_only_reports_and_fails_on_a_planted_defect() {
        let (dir, _, snap_path) = project(ORPHAN_ITEM, &snap(""));
        let mut o = opts(&dir, &snap_path);
        o.check_only = true;
        let err = handle_work_sync(o)
            .await
            .expect_err("an orphan item is a finding");
        let msg = err.to_string();
        assert!(
            msg.contains("1 finding(s)") && msg.contains("1 ORPHAN-ROADMAP"),
            "{msg}"
        );
    }

    #[tokio::test]
    async fn check_only_passes_on_a_bijection() {
        let (dir, _, snap_path) = project(MATCHED_ITEM, &snap(ISSUE_TWO));
        let mut o = opts(&dir, &snap_path);
        o.check_only = true;
        o.json = true;
        handle_work_sync(o).await.expect("a bijection is coherent");
    }

    #[tokio::test]
    async fn dry_run_writes_nothing() {
        let (dir, roadmap, snap_path) = project(ORPHAN_ITEM, &snap(""));
        let before = std::fs::read(&roadmap).expect("read");
        let mut o = opts(&dir, &snap_path);
        o.direction = SyncDirection::YamlToGithub;
        o.dry_run = true;
        handle_work_sync(o).await.expect("a dry run succeeds");
        assert_eq!(std::fs::read(&roadmap).expect("read"), before);
    }

    #[tokio::test]
    async fn github_to_yaml_creates_the_gh_item_and_projects_the_release() {
        let nine = r#"{"number":9,"title":"nine","state":"open","labels":[],"milestone":"3.42.0","updated_at":"2026-09-09T12:00:00Z"}"#;
        let (dir, roadmap, snap_path) =
            project(MATCHED_ITEM, &snap(&format!("{ISSUE_TWO},{nine}")));
        let mut o = opts(&dir, &snap_path);
        o.direction = SyncDirection::GithubToYaml;
        handle_work_sync(o).await.expect("github-to-yaml succeeds");
        let saved = RoadmapService::new(&roadmap).load().expect("reload");
        let gh9 = saved.find_item("GH-9").expect("GH-9 was created");
        assert_eq!(gh9.github_issue, Some(9));
        assert_eq!(gh9.release.as_deref(), Some("3.42.0"));
        assert_eq!(gh9.status, ItemStatus::Planned);
        let b = saved.find_item("B").expect("B survived");
        assert_eq!((b.github_issue, b.title.as_str()), (Some(2), "beta"));
    }

    #[tokio::test]
    async fn a_collided_item_is_never_written() {
        let items = "- id: M0\n  github_issue: 612\n  item_type: task\n  title: zero\n  status: planned\n- id: M1\n  github_issue: 612\n  item_type: task\n  title: one\n  status: planned\n";
        let closed = r#"{"number":612,"title":"macs","state":"closed","state_reason":"completed","labels":[],"updated_at":"2026-09-09T12:00:00Z"}"#;
        let (dir, roadmap, snap_path) = project(items, &snap(closed));
        let before = std::fs::read(&roadmap).expect("read");
        let mut o = opts(&dir, &snap_path);
        o.direction = SyncDirection::GithubToYaml;
        handle_work_sync(o).await.expect("skips are not errors");
        assert_eq!(
            std::fs::read(&roadmap).expect("read"),
            before,
            "a COLLISION is never auto-fixed (§5.4)"
        );
    }

    #[tokio::test]
    async fn a_missing_snapshot_file_is_an_error() {
        let (dir, _, _) = project(MATCHED_ITEM, "{}");
        let mut o = opts(&dir, &dir.path().join("nope.json"));
        o.check_only = true;
        let err = handle_work_sync(o).await.expect_err("missing file");
        assert!(err.to_string().contains("cannot read snapshot"), "{err}");
    }

    #[test]
    fn parse_snapshot_reads_gh_json() {
        let issues = serde_json::json!([
            {"number": 7, "title": "seven", "state": "CLOSED", "stateReason": "NOT_PLANNED",
             "labels": [{"name": "bug"}, {"name": "no-roadmap"}], "milestone": {"title": "3.42.0"},
             "updatedAt": "2026-09-09T10:00:00Z"},
            {"number": 8, "title": "eight", "state": "OPEN", "stateReason": null,
             "labels": [], "milestone": null, "updatedAt": "2026-09-09T11:00:00Z"}
        ]);
        let milestones = serde_json::json!([{"title": "3.42.0", "state": "open"}, {"title": "3.40.0", "state": "closed"}]);
        let s = parse_snapshot("paiml/pmat", Utc::now(), &issues, &milestones).expect("parses");
        let seven = s.issue(7).expect("seven");
        assert_eq!(seven.state, IssueState::Closed);
        assert_eq!(seven.state_reason, Some(CloseReason::NotPlanned));
        assert_eq!(seven.labels, vec!["bug", "no-roadmap"]);
        assert_eq!(seven.milestone.as_deref(), Some("3.42.0"));
        assert!(!seven.in_universe());
        let eight = s.issue(8).expect("eight");
        assert_eq!(eight.state, IssueState::Open);
        assert_eq!(eight.state_reason, None);
        assert!(eight.milestone.is_none());
        assert!(eight.in_universe());
        assert_eq!(s.milestones.len(), 2);
        assert_eq!(s.milestones[1].state, IssueState::Closed);

        let bad = serde_json::json!([{"number": 1, "title": "x", "state": "MAYBE", "updatedAt": "2026-09-09T11:00:00Z"}]);
        assert!(parse_snapshot("r", Utc::now(), &bad, &serde_json::json!([])).is_err());
        let bad_time = serde_json::json!([{"number": 1, "title": "x", "state": "OPEN", "updatedAt": "yesterday"}]);
        assert!(parse_snapshot("r", Utc::now(), &bad_time, &serde_json::json!([])).is_err());
    }
}
