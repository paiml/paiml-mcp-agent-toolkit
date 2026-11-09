//! Where a [`RepoSnapshot`] comes from: GitHub GraphQL (reads only — GHC-001 / fleet
//! rule GH-2) or a snapshot file for controls and offline runs.
//!
//! A read that fails is an error, never an empty snapshot: an unmeasured repo must not
//! read as a compliant one. The budget is checked first, and a low budget is a refusal.

use super::{CheckState, IssueRec, PrRec, RepoSnapshot};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// Below this many remaining GraphQL points the run refuses to read (fleet rule GH-1).
pub const MIN_POINTS: u64 = 500;
/// A hard stop on pagination: 50 pages of 100.
pub const MAX_PAGES: usize = 50;

const RATE_QUERY: &str = "query { rateLimit { remaining resetAt } }";

const ISSUES_QUERY: &str = "query($owner:String!,$name:String!,$after:String){\
repository(owner:$owner,name:$name){issues(states:OPEN,first:100,after:$after){\
totalCount pageInfo{hasNextPage endCursor} \
nodes{number title milestone{title} parent{number} subIssues{totalCount} subIssuesSummary{completed} \
labels(first:20){nodes{name}}}}}}";

const PRS_QUERY: &str = "query($owner:String!,$name:String!,$after:String){\
repository(owner:$owner,name:$name){pullRequests(states:OPEN,first:100,after:$after){\
pageInfo{hasNextPage endCursor} \
nodes{number title isDraft createdAt headRefName body labels(first:20){nodes{name}} \
commits(last:1){nodes{commit{statusCheckRollup{state}}}}}}}}";

const MERGED_QUERY: &str = "query($owner:String!,$name:String!,$after:String){\
repository(owner:$owner,name:$name){pullRequests(states:MERGED,first:100,after:$after){\
pageInfo{hasNextPage endCursor} \
nodes{number title body closingIssuesReferences(first:20){nodes{number repository{nameWithOwner}}}}}}}";

/// One GraphQL round trip, so tests can stand in for GitHub.
pub trait Graphql {
    fn query(&self, query: &str, vars: &[(&str, &str)]) -> Result<Value>;
}

/// `gh api graphql`. Read-only: only `query` documents are ever passed.
pub struct GhGraphql;

impl Graphql for GhGraphql {
    fn query(&self, query: &str, vars: &[(&str, &str)]) -> Result<Value> {
        if !query.trim_start().starts_with("query") {
            bail!("refused: GHC-001 reads GitHub through GraphQL queries only");
        }
        let mut cmd = Command::new("gh");
        cmd.args(["api", "graphql", "-H", "GraphQL-Features: sub_issues"]);
        cmd.arg("-f").arg(format!("query={query}"));
        for (k, v) in vars {
            cmd.arg("-f").arg(format!("{k}={v}"));
        }
        let out = cmd.output().context("could not run `gh`")?;
        if !out.status.success() {
            bail!(
                "gh api graphql failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        let v: Value = serde_json::from_slice(&out.stdout).context("gh api graphql: not JSON")?;
        if let Some(errors) = v.get("errors").filter(|e| !e.is_null()) {
            bail!("gh api graphql: {errors}");
        }
        Ok(v)
    }
}

/// `owner/name` of the repository `dir` is a checkout of.
pub fn current_repo(dir: &Path) -> Result<String> {
    let out = Command::new("gh")
        .args([
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "-q",
            ".nameWithOwner",
        ])
        .current_dir(dir)
        .output()
        .context("could not run `gh`")?;
    if !out.status.success() {
        bail!(
            "could not name the repository (pass --repo owner/name): {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.split('/').count() != 2 {
        bail!("unexpected repository name {name:?}");
    }
    Ok(name)
}

/// Read a snapshot written by [`RepoSnapshot`]'s serializer. Controls and offline runs only.
pub fn load_snapshot_file(path: &Path) -> Result<RepoSnapshot> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading snapshot {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing snapshot {}", path.display()))
}

/// Remaining points from a `rateLimit` response, wherever it sits.
pub fn remaining_points(v: &Value) -> Option<u64> {
    v["data"]["rateLimit"]["remaining"].as_u64()
}

/// Refuse when the GraphQL budget is below [`MIN_POINTS`] or unreadable.
pub fn check_budget(g: &dyn Graphql) -> Result<u64> {
    let v = g.query(RATE_QUERY, &[])?;
    let left = remaining_points(&v).context("rateLimit.remaining missing: budget not measured")?;
    if left < MIN_POINTS {
        bail!(
            "GraphQL budget is {left} points (< {MIN_POINTS}); not reading — retry after the reset"
        );
    }
    Ok(left)
}

/// Read every open issue and open PR of `repo` (`owner/name`).
pub fn fetch_snapshot(g: &dyn Graphql, repo: &str, now: DateTime<Utc>) -> Result<RepoSnapshot> {
    let (owner, name) = repo
        .split_once('/')
        .filter(|(o, n)| !o.is_empty() && !n.is_empty() && !n.contains('/'))
        .with_context(|| format!("repo must be owner/name, got {repo:?}"))?;
    check_budget(g)?;
    let mut total = 0usize;
    let mut open_issues = Vec::new();
    for page in pages(g, ISSUES_QUERY, owner, name, "issues")? {
        total = page["totalCount"]
            .as_u64()
            .context("issues.totalCount missing")? as usize;
        for node in nodes(&page)? {
            open_issues.push(parse_issue(node)?);
        }
    }
    let mut open_prs = Vec::new();
    for page in pages(g, PRS_QUERY, owner, name, "pullRequests")? {
        for node in nodes(&page)? {
            open_prs.push(parse_pr(node)?);
        }
    }
    Ok(RepoSnapshot {
        repo: repo.to_string(),
        taken_at: now,
        open_issue_total: total,
        open_issues,
        open_prs,
    })
}

/// Every merged PR of `repo` (`owner/name`): number, title, body. Budget checked first;
/// a PR node without a number is an error, never skipped.
pub fn fetch_merged_prs(g: &dyn Graphql, repo: &str) -> Result<Vec<super::backfill::MergedPr>> {
    let (owner, name) = repo
        .split_once('/')
        .filter(|(o, n)| !o.is_empty() && !n.is_empty() && !n.contains('/'))
        .with_context(|| format!("repo must be owner/name, got {repo:?}"))?;
    check_budget(g)?;
    let mut out = Vec::new();
    for page in pages(g, MERGED_QUERY, owner, name, "pullRequests")? {
        for node in nodes(&page)? {
            let number = node["number"].as_u64().context("merged PR has no number")?;
            let refs = node["closingIssuesReferences"]["nodes"]
                .as_array()
                .with_context(|| format!("merged PR #{number}: no closingIssuesReferences"))?;
            let mut closes = Vec::new();
            for r in refs {
                let n = r["number"]
                    .as_u64()
                    .context("closing reference has no number")?;
                let repo = r["repository"]["nameWithOwner"]
                    .as_str()
                    .context("closing reference has no repository")?;
                closes.push((repo.to_string(), n));
            }
            out.push(super::backfill::MergedPr {
                number,
                title: node["title"].as_str().unwrap_or_default().to_string(),
                body: node["body"].as_str().unwrap_or_default().to_string(),
                closes,
            });
        }
    }
    Ok(out)
}

fn nodes(page: &Value) -> Result<&Vec<Value>> {
    page["nodes"].as_array().context("connection has no nodes")
}

/// Follow `pageInfo` to the end of one connection, at most [`MAX_PAGES`] pages.
fn pages(g: &dyn Graphql, query: &str, owner: &str, name: &str, field: &str) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    let mut after: Option<String> = None;
    for _ in 0..MAX_PAGES {
        let mut vars = vec![("owner", owner), ("name", name)];
        if let Some(a) = after.as_deref() {
            vars.push(("after", a));
        }
        let v = g.query(query, &vars)?;
        let conn = v["data"]["repository"][field].clone();
        if conn.is_null() {
            bail!("{owner}/{name}: no `{field}` in the reply (repository not readable?)");
        }
        let more = conn["pageInfo"]["hasNextPage"].as_bool().with_context(|| {
            format!("{owner}/{name}: `{field}` reply has no pageInfo.hasNextPage; not measured")
        })?;
        let cursor = conn["pageInfo"]["endCursor"].as_str().map(str::to_string);
        out.push(conn);
        if !more {
            return Ok(out);
        }
        after = Some(cursor.context("hasNextPage without endCursor")?);
    }
    bail!("{owner}/{name}: more than {MAX_PAGES} pages of {field}; not measured")
}

fn label_names(node: &Value) -> Vec<String> {
    node["labels"]["nodes"]
        .as_array()
        .map(|ls| {
            ls.iter()
                .filter_map(|l| l["name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_issue(node: &Value) -> Result<IssueRec> {
    let number = node["number"].as_u64().context("issue has no number")?;
    Ok(IssueRec {
        number,
        title: node["title"].as_str().unwrap_or_default().to_string(),
        milestone: node["milestone"]["title"].as_str().map(str::to_string),
        parent: node["parent"]["number"].as_u64(),
        labels: label_names(node),
        sub_issues: node["subIssues"]["totalCount"].as_u64().with_context(|| {
            format!("issue #{number}: no subIssues.totalCount in the reply; not measured")
        })?,
        sub_issues_closed: node["subIssuesSummary"]["completed"].as_u64(),
    })
}

/// GitHub's rollup states, folded: SUCCESS is the only PASS.
pub fn check_state(rollup: Option<&str>) -> CheckState {
    match rollup {
        Some("SUCCESS") => CheckState::Pass,
        Some("PENDING") | Some("EXPECTED") => CheckState::Pending,
        Some(_) => CheckState::Fail,
        None => CheckState::None,
    }
}

pub fn parse_pr(node: &Value) -> Result<PrRec> {
    let number = node["number"].as_u64().context("PR has no number")?;
    let created = node["createdAt"]
        .as_str()
        .with_context(|| format!("PR #{number}: no createdAt"))?;
    let rollup = node["commits"]["nodes"][0]["commit"]["statusCheckRollup"]["state"].as_str();
    Ok(PrRec {
        number,
        title: node["title"].as_str().unwrap_or_default().to_string(),
        is_draft: node["isDraft"].as_bool().unwrap_or(false),
        created_at: created
            .parse::<DateTime<Utc>>()
            .with_context(|| format!("PR #{number}: bad createdAt {created:?}"))?,
        head_ref: node["headRefName"].as_str().unwrap_or_default().to_string(),
        labels: label_names(node),
        body: node["body"].as_str().unwrap_or_default().to_string(),
        checks: check_state(rollup),
    })
}

/// `pmat work add`'s guard end to end: read the repo, then judge R1, R3 and R5.
/// A read that fails refuses the add: an unmeasured repo is not a compliant one.
pub fn guard_filing(
    g: &dyn Graphql,
    repo: &str,
    policy: &super::Policy,
    title: &str,
    child: u64,
    epic: u64,
) -> Result<Option<String>> {
    let snap = fetch_snapshot(g, repo, Utc::now())
        .with_context(|| format!("refused: could not read {repo}'s open tickets (GHC-001 R1/R5) — nothing was linked or written"))?;
    // R3: the epic the ticket will be linked under must exist and be open, otherwise the
    // link is no home. (Otherwise the ticket itself must already carry a milestone.)
    let epic_is_open = snap.open_issues.iter().any(|i| i.number == epic);
    super::guard_new_ticket(
        policy,
        &snap,
        title,
        Some(child),
        epic_is_open.then_some(epic),
    )
}
