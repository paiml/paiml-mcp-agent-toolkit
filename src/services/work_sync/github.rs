use crate::services::work_sync::{
    CloseReason, GithubSnapshot, IssueSnapshot, IssueState, MilestoneSnapshot,
};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::process::Command;

pub enum SnapshotSource {
    File(PathBuf),
    Live { repo: String },
}

impl std::fmt::Display for SnapshotSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotSource::File(path) => write!(f, "file {}", path.display()),
            SnapshotSource::Live { repo } => write!(f, "gh {}", repo),
        }
    }
}

impl SnapshotSource {
    pub fn load(&self) -> Result<GithubSnapshot> {
        match self {
            SnapshotSource::File(path) => {
                let text = std::fs::read_to_string(path)
                    .with_context(|| format!("cannot read snapshot {}", path.display()))?;
                GithubSnapshot::from_json(&text)
            }
            SnapshotSource::Live { repo } => fetch_snapshot(repo),
        }
    }
}

pub fn gh_json(args: &[&str]) -> Result<serde_json::Value> {
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

pub fn refuse_truncation(count: usize, cap: usize, what: &str) -> Result<()> {
    if count >= cap {
        bail!(
            "the GitHub snapshot is truncated: {what} returned {count}, the request cap of {cap}; a snapshot that may be missing {what} cannot prove coherence — raise the cap"
        );
    }
    Ok(())
}

pub const ISSUE_CAP: usize = 5000;
pub const MILESTONE_CAP: usize = 100;

pub fn fetch_snapshot(repo: &str) -> Result<GithubSnapshot> {
    let issues = gh_json(&[
        "issue",
        "list",
        "--repo",
        repo,
        "--state",
        "all",
        "--limit",
        &ISSUE_CAP.to_string(),
        "--json",
        "number,title,state,stateReason,labels,milestone,updatedAt",
    ])?;
    refuse_truncation(
        issues.as_array().map(Vec::len).unwrap_or(0),
        ISSUE_CAP,
        "issues",
    )?;
    let milestones = gh_json(&[
        "api",
        &format!("repos/{repo}/milestones?state=all&per_page={MILESTONE_CAP}"),
    ])?;
    refuse_truncation(
        milestones.as_array().map(Vec::len).unwrap_or(0),
        MILESTONE_CAP,
        "milestones",
    )?;
    parse_snapshot(repo, Utc::now(), &issues, &milestones)
}

pub fn parse_snapshot(
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

    #[test]
    fn a_snapshot_at_the_page_cap_is_refused() {
        refuse_truncation(ISSUE_CAP - 1, ISSUE_CAP, "issues").expect("under the cap is fine");
        let err = refuse_truncation(ISSUE_CAP, ISSUE_CAP, "issues").expect_err("at the cap");
        assert!(err.to_string().contains("truncated"), "{err}");
        assert!(refuse_truncation(MILESTONE_CAP, MILESTONE_CAP, "milestones").is_err());
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
