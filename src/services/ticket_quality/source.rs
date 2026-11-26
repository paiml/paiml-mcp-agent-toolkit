//! Where the ticket evidence comes from: GitHub GraphQL, reads only (fleet rule GH-2).
//!
//! A read that fails is an error, never an empty list: an unmeasured repository must
//! not read as one with no findings.

use super::findings::{PrState, TicketRec};
use super::rubric::{self, Ticket};
use crate::services::github_comply::source::{check_budget, pages, parse_issue, Graphql};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::BTreeSet;

/// Paths asked about per GraphQL document.
pub const PATHS_PER_QUERY: usize = 50;
/// The most distinct paths one run asks about; the rest are reported as not measured.
pub const MAX_PATHS: usize = 300;

const TICKETS_QUERY: &str = "query($owner:String!,$name:String!,$after:String){\
repository(owner:$owner,name:$name){issues(states:OPEN,first:100,after:$after){\
totalCount pageInfo{hasNextPage endCursor} \
nodes{number title body updatedAt milestone{title} parent{number state} \
subIssues{totalCount} subIssuesSummary{completed} labels(first:100){totalCount nodes{name}} \
closedByPullRequestsReferences(first:100,includeClosedPrs:true){totalCount nodes{number state}}}}}}";

fn split_repo(repo: &str) -> Result<(&str, &str)> {
    repo.split_once('/')
        .filter(|(o, n)| !o.is_empty() && !n.is_empty() && !n.contains('/'))
        .with_context(|| format!("repo must be owner/name, got {repo:?}"))
}

fn pr_state(s: &str) -> Result<PrState> {
    Ok(match s {
        "OPEN" => PrState::Open,
        "MERGED" => PrState::Merged,
        "CLOSED" => PrState::Closed,
        other => bail!("unknown pull request state {other:?}"),
    })
}

/// Every node of the connection `field` of issue `n`: a list GitHub cut off, or one
/// whose size was not read, is an error, never a shorter list.
fn whole<'a>(node: &'a Value, field: &str, n: u64) -> Result<&'a [Value]> {
    let list = &node[field];
    let nodes = list["nodes"]
        .as_array()
        .with_context(|| format!("issue #{n}: no {field}"))?;
    let total = list["totalCount"]
        .as_u64()
        .with_context(|| format!("issue #{n}: no totalCount on {field}"))?;
    if total > nodes.len() as u64 {
        bail!(
            "issue #{n}: read {} of {total} {field}; not measured",
            nodes.len()
        );
    }
    Ok(nodes)
}

/// One ticket from an issue node of [`TICKETS_QUERY`].
pub fn parse_ticket(node: &Value) -> Result<TicketRec> {
    let rec = parse_issue(node)?;
    let n = rec.number;
    let updated = node["updatedAt"]
        .as_str()
        .with_context(|| format!("issue #{n}: no updatedAt"))?;
    whole(node, "labels", n)?;
    let prs = whole(node, "closedByPullRequestsReferences", n)?;
    let mut linked_prs = Vec::new();
    for p in prs {
        let number = p["number"].as_u64().context("linked PR has no number")?;
        linked_prs.push((number, pr_state(p["state"].as_str().unwrap_or_default())?));
    }
    Ok(TicketRec {
        rec,
        body: node["body"].as_str().unwrap_or_default().to_string(),
        updated_at: updated
            .parse::<DateTime<Utc>>()
            .with_context(|| format!("issue #{n}: bad updatedAt {updated:?}"))?,
        parent_open: node["parent"]["state"].as_str().map(|s| s == "OPEN"),
        linked_prs,
    })
}

/// Every open ticket of `repo`, and GitHub's count of them. One paged read, 100 a page.
pub fn fetch_tickets(g: &dyn Graphql, repo: &str) -> Result<(usize, Vec<TicketRec>)> {
    let (owner, name) = split_repo(repo)?;
    check_budget(g)?;
    let mut total = 0usize;
    let mut tickets = Vec::new();
    for page in pages(g, TICKETS_QUERY, owner, name, "issues")? {
        total = page["totalCount"]
            .as_u64()
            .context("issues.totalCount missing")? as usize;
        let nodes = page["nodes"].as_array().context("issues has no nodes")?;
        for node in nodes {
            tickets.push(parse_ticket(node)?);
        }
    }
    Ok((total, tickets))
}

/// One ticket, open or not, as the rubric reads it.
pub fn fetch_ticket(g: &dyn Graphql, repo: &str, number: u64) -> Result<Ticket> {
    let (owner, name) = split_repo(repo)?;
    check_budget(g)?;
    let query = format!(
        "query($owner:String!,$name:String!){{repository(owner:$owner,name:$name){{\
issue(number:{number}){{title body milestone{{title}} parent{{number state}} \
labels(first:100){{totalCount nodes{{name}}}}}}}}}}"
    );
    let v = g.query(&query, &[("owner", owner), ("name", name)])?;
    let node = &v["data"]["repository"]["issue"];
    let title = node["title"]
        .as_str()
        .with_context(|| format!("{repo}#{number}: no such issue in the reply; not measured"))?;
    // A null connection is an issue without labels; anything else must be whole.
    let labels: &[Value] = match node.get("labels") {
        Some(Value::Null) => &[],
        _ => whole(node, "labels", number)?,
    };
    Ok(Ticket {
        title: title.to_string(),
        body: node["body"].as_str().unwrap_or_default().to_string(),
        labels: labels
            .iter()
            .filter_map(|l| l["name"].as_str().map(str::to_string))
            .collect(),
        parent_open: node["parent"]["state"].as_str() == Some("OPEN"),
        has_milestone: node["milestone"]["title"].is_string(),
    })
}

/// Only characters a path may have reach a query document: nothing here can end a
/// string literal.
fn is_query_safe(path: &str) -> bool {
    path.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'))
}

/// One aliased field per path: `p0: <field>` … Returns the reply's `inner` object.
fn ask_paths(
    g: &dyn Graphql,
    owner: &str,
    name: &str,
    paths: &[&String],
    wrap: (&str, &str),
    field: impl Fn(&str) -> String,
) -> Result<Value> {
    let fields: Vec<String> = paths
        .iter()
        .enumerate()
        .map(|(i, p)| format!("p{i}:{}", field(p)))
        .collect();
    let query = format!(
        "query($owner:String!,$name:String!){{repository(owner:$owner,name:$name){{{}{}{}}}}}",
        wrap.0,
        fields.join(" "),
        wrap.1
    );
    let v = g.query(&query, &[("owner", owner), ("name", name)])?;
    Ok(v["data"]["repository"].clone())
}

/// The files among `paths` that are absent on the default branch.
fn absent(g: &dyn Graphql, owner: &str, name: &str, paths: &[&String]) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for chunk in paths.chunks(PATHS_PER_QUERY) {
        let repo = ask_paths(g, owner, name, chunk, ("", ""), |p| {
            format!("object(expression:\"HEAD:{p}\"){{__typename}}")
        })?;
        if !repo.is_object() {
            bail!("{owner}/{name}: no repository in the path reply; not measured");
        }
        for (i, p) in chunk.iter().enumerate() {
            // `null` is GitHub saying the file is absent; no key is no answer.
            let answer = repo
                .get(format!("p{i}"))
                .with_context(|| format!("{owner}/{name}: no answer for {p}; not measured"))?;
            if answer.is_null() {
                out.push((*p).clone());
            }
        }
    }
    Ok(out)
}

/// The files among `paths` that some commit on the default branch touched.
fn with_history(
    g: &dyn Graphql,
    owner: &str,
    name: &str,
    paths: &[&String],
) -> Result<BTreeSet<String>> {
    let mut out = BTreeSet::new();
    let wrap = ("defaultBranchRef{target{... on Commit{", "}}}");
    for chunk in paths.chunks(PATHS_PER_QUERY) {
        let repo = ask_paths(g, owner, name, chunk, wrap, |p| {
            format!("history(first:1,path:\"{p}\"){{nodes{{oid}}}}")
        })?;
        let commit = &repo["defaultBranchRef"]["target"];
        for (i, p) in chunk.iter().enumerate() {
            let nodes = commit[format!("p{i}")]["nodes"]
                .as_array()
                .with_context(|| format!("{owner}/{name}: no history for {p}; not measured"))?;
            if !nodes.is_empty() {
                out.insert((*p).clone());
            }
        }
    }
    Ok(out)
}

/// The files the tickets name that were on the default branch and are gone: absent at
/// `HEAD`, with history. A file that never existed is a file the ticket asks for, not a
/// stale pointer. The second value lists what was not asked.
pub fn gone_paths(
    g: &dyn Graphql,
    repo: &str,
    tickets: &[TicketRec],
) -> Result<(BTreeSet<String>, Vec<String>)> {
    let (owner, name) = split_repo(repo)?;
    let named: BTreeSet<String> = tickets
        .iter()
        .flat_map(|t| rubric::named_files(&t.body))
        .filter(|p| is_query_safe(p))
        .collect();
    let mut not_measured = Vec::new();
    if named.len() > MAX_PATHS {
        not_measured.push(format!(
            "V3: {} distinct paths named, only the first {MAX_PATHS} were checked",
            named.len()
        ));
    }
    let asked: Vec<&String> = named.iter().take(MAX_PATHS).collect();
    let missing = absent(g, owner, name, &asked)?;
    let missing_refs: Vec<&String> = missing.iter().collect();
    let gone = with_history(g, owner, name, &missing_refs)?;
    Ok((gone, not_measured))
}
