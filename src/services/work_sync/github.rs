use crate::services::work_sync::{
    CloseReason, GithubSnapshot, IssueSnapshot, IssueState, MilestoneSnapshot,
};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
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
    let mut snapshot = parse_snapshot(repo, Utc::now(), &issues, &milestones)?;
    let epics = snapshot.epic_numbers();
    if !epics.is_empty() {
        let counts = sub_issue_counts(repo, &epics)?;
        snapshot.fill_sub_issues(&counts);
    }
    Ok(snapshot)
}

/// The sub-issue count of each issue in `numbers`, from GitHub's GraphQL
/// `subIssuesSummary { total }` — one query, one alias per issue (`i<number>`)
/// — because `gh issue list --json` exposes no sub-issue field (measured, gh
/// 2.x, 2026-09-10). Every number asked for is answered or the call fails:
/// a count that is missing is not a count of zero (goal-mode.md doctrine 2).
pub fn sub_issue_counts(repo: &str, numbers: &[u64]) -> Result<BTreeMap<u64, u64>> {
    if numbers.is_empty() {
        return Ok(BTreeMap::new());
    }

    let (owner, name) = repo
        .split_once('/')
        .with_context(|| format!("repo is not owner/name: {repo:?}"))?;
    let mut all_counts = BTreeMap::new();

    for chunk in numbers.chunks(100) {
        let mut query = format!("query {{ repository(owner: \"{owner}\", name: \"{name}\") {{ ");
        for &n in chunk {
            use std::fmt::Write;
            write!(
                &mut query,
                "i{n}: issue(number: {n}) {{ subIssuesSummary {{ total }} }} "
            )
            .unwrap();
        }
        query.push_str("} }");

        let args = ["api", "graphql", "-f", &format!("query={query}")];
        let out = gh_json(&args)?;
        let chunk_counts = parse_sub_issue_counts(&out, chunk)?;
        all_counts.extend(chunk_counts);
    }

    Ok(all_counts)
}

/// The pure half of [`sub_issue_counts`]: read `{"data":{"repository":{"i<n>":
/// {"subIssuesSummary":{"total":k}}}}}` for every `n` in `numbers`. A missing
/// or null alias is an error naming the number, never a zero.
pub fn parse_sub_issue_counts(
    graphql: &serde_json::Value,
    numbers: &[u64],
) -> Result<BTreeMap<u64, u64>> {
    if let Some(errors) = graphql.get("errors").and_then(|e| e.as_array()) {
        if let Some(first) = errors.first() {
            if let Some(msg) = first.get("message").and_then(|m| m.as_str()) {
                bail!("{msg}");
            }
        }
    }

    let repo_data = graphql
        .get("data")
        .and_then(|d| d.get("repository"))
        .context("no data.repository in GraphQL response")?;

    let mut out = BTreeMap::new();
    for &n in numbers {
        let alias = format!("i{n}");
        let total = repo_data
            .get(&alias)
            .filter(|i| !i.is_null())
            .and_then(|i| i.get("subIssuesSummary"))
            .and_then(|s| s.get("total"))
            .and_then(|t| t.as_u64());

        if let Some(t) = total {
            out.insert(n, t);
        } else {
            bail!("issue #{n} has no sub-issue count in the GraphQL response");
        }
    }
    Ok(out)
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
            sub_issues: None,
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

    /// Mutant: a missing alias read as zero, or an extra alias ignored while a
    /// requested one is missing (PMAT-728, goal-mode.md doctrine 2).
    #[test]
    fn sub_issue_counts_are_parsed_from_the_graphql_aliases_and_a_missing_one_is_an_error() {
        let ok = serde_json::json!({"data": {"repository": {
            "i1017": {"subIssuesSummary": {"total": 3, "completed": 1}},
            "i1019": {"subIssuesSummary": {"total": 0, "completed": 0}}
        }}});
        let counts = parse_sub_issue_counts(&ok, &[1017, 1019]).expect("both aliases answered");
        assert_eq!(counts.get(&1017), Some(&3));
        assert_eq!(counts.get(&1019), Some(&0));
        assert_eq!(counts.len(), 2);

        let missing = parse_sub_issue_counts(&ok, &[1017, 1018]).expect_err("1018 is not answered");
        assert!(missing.to_string().contains("1018"), "{missing}");
        let null = serde_json::json!({"data": {"repository": {"i5": null}}});
        assert!(
            parse_sub_issue_counts(&null, &[5]).is_err(),
            "a null alias (no such issue) is not a zero"
        );
        let errors = serde_json::json!({"errors": [{"message": "bad"}], "data": null});
        assert!(parse_sub_issue_counts(&errors, &[5]).is_err());
    }

    /// Mutant: `fill_sub_issues` writing every issue (a zero onto the
    /// unmeasured), `epic_numbers` reading open epics only, or the field
    /// serialised when unmeasured.
    #[test]
    fn the_snapshot_carries_sub_issues_only_where_measured() {
        let issues = serde_json::json!([
            {"number": 1, "title": "epic one", "state": "OPEN", "labels": [{"name": "epic"}], "updatedAt": "2026-09-09T10:00:00Z"},
            {"number": 2, "title": "closed epic", "state": "CLOSED", "labels": [{"name": "epic"}], "updatedAt": "2026-09-09T10:00:00Z"},
            {"number": 3, "title": "plain", "state": "OPEN", "labels": [{"name": "bug"}], "updatedAt": "2026-09-09T10:00:00Z"}
        ]);
        let mut s = parse_snapshot("paiml/pmat", Utc::now(), &issues, &serde_json::json!([]))
            .expect("parses");
        assert_eq!(
            s.epic_numbers(),
            vec![1, 2],
            "open and closed epics alike — the epic leg judges the state"
        );
        assert!(
            s.issues.iter().all(|i| i.sub_issues.is_none()),
            "parse measures nothing"
        );
        let mut counts = BTreeMap::new();
        counts.insert(1, 4);
        s.fill_sub_issues(&counts);
        assert_eq!(s.issue(1).and_then(|i| i.sub_issues), Some(4));
        assert_eq!(
            s.issue(2).and_then(|i| i.sub_issues),
            None,
            "unmeasured stays unmeasured"
        );
        assert_eq!(s.issue(3).and_then(|i| i.sub_issues), None);

        let json = s.to_json().expect("serialises");
        assert!(json.contains("\"sub_issues\": 4"), "{json}");
        assert_eq!(
            json.matches("sub_issues").count(),
            1,
            "absent when unmeasured"
        );
        let back = GithubSnapshot::from_json(&json).expect("round-trips");
        assert_eq!(back, s);
        let old = r#"{"repo":"r","taken_at":"2026-09-09T10:00:00Z","issues":[{"number":9,"title":"t","state":"open","labels":["epic"],"updated_at":"2026-09-09T10:00:00Z"}]}"#;
        let old = GithubSnapshot::from_json(old)
            .expect("a snapshot written before the field still reads");
        assert_eq!(old.issue(9).and_then(|i| i.sub_issues), None);
        assert!(old.issue(9).is_some_and(IssueSnapshot::is_epic));
    }
}
