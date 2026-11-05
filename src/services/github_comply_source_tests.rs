//! GHC-001: the GraphQL reader, against a fake GitHub that records every query.

use super::source::*;
use super::*;
use chrono::TimeZone;
use serde_json::{json, Value};
use std::cell::RefCell;

type Seen = (String, Vec<(String, String)>);

struct Fake {
    replies: RefCell<Vec<Result<Value, String>>>,
    seen: RefCell<Vec<Seen>>,
}

impl Fake {
    fn new(replies: Vec<Result<Value, String>>) -> Self {
        Fake {
            replies: RefCell::new(replies.into_iter().rev().collect()),
            seen: RefCell::new(vec![]),
        }
    }
}

impl Graphql for Fake {
    fn query(&self, query: &str, vars: &[(&str, &str)]) -> anyhow::Result<Value> {
        self.seen.borrow_mut().push((
            query.to_string(),
            vars.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        ));
        self.replies
            .borrow_mut()
            .pop()
            .expect("the reader made a query the test did not expect")
            .map_err(|e| anyhow::anyhow!(e))
    }
}

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 5, 12, 0, 0)
        .single()
        .expect("fixture")
}

fn rate(remaining: u64) -> Result<Value, String> {
    Ok(json!({"data":{"rateLimit":{"remaining":remaining,"resetAt":"2026-11-05T13:00:00Z"}}}))
}

fn issues_page(total: u64, next: Option<&str>, numbers: &[u64]) -> Result<Value, String> {
    let nodes: Vec<Value> = numbers
        .iter()
        .map(|n| {
            json!({"number":n,"title":format!("t{n}"),"milestone":null,"parent":{"number":1},
                   "labels":{"nodes":[{"name":"epic"}]}})
        })
        .collect();
    Ok(json!({"data":{"repository":{"issues":{
        "totalCount":total,
        "pageInfo":{"hasNextPage":next.is_some(),"endCursor":next},
        "nodes":nodes}}}}))
}

fn prs_page(nodes: Value) -> Result<Value, String> {
    Ok(json!({"data":{"repository":{"pullRequests":{
        "pageInfo":{"hasNextPage":false,"endCursor":null},"nodes":nodes}}}}))
}

#[test]
fn reads_every_page_and_keeps_graphql_totalcount() {
    let g = Fake::new(vec![
        rate(4000),
        issues_page(3, Some("C1"), &[1, 2]),
        issues_page(3, None, &[3]),
        prs_page(json!([])),
    ]);
    let s = fetch_snapshot(&g, "paiml/x", now()).expect("fixture");
    assert_eq!(s.open_issue_total, 3);
    assert_eq!(
        s.open_issues.iter().map(|i| i.number).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(s.open_issues[0].parent, Some(1));
    assert_eq!(s.open_issues[0].milestone, None);
    let seen = g.seen.borrow();
    // the second issues page carries the cursor, the first does not
    assert!(!seen[1].1.iter().any(|(k, _)| k == "after"));
    assert!(seen[2].1.contains(&("after".to_string(), "C1".to_string())));
    assert!(seen
        .iter()
        .all(|(q, _)| q.trim_start().starts_with("query")));
}

#[test]
fn a_low_budget_refuses_before_any_read() {
    let g = Fake::new(vec![rate(MIN_POINTS - 1)]);
    let err = fetch_snapshot(&g, "paiml/x", now()).unwrap_err();
    assert!(format!("{err:#}").contains("budget"), "{err:#}");
    assert_eq!(
        g.seen.borrow().len(),
        1,
        "only the rate-limit query may run"
    );
}

#[test]
fn an_unreadable_budget_is_not_measured_not_ok() {
    let g = Fake::new(vec![Ok(json!({"data":{}}))]);
    assert!(fetch_snapshot(&g, "paiml/x", now()).is_err());
}

#[test]
fn a_failed_read_is_an_error_never_an_empty_snapshot() {
    let g = Fake::new(vec![rate(4000), Err("HTTP 502".into())]);
    let err = fetch_snapshot(&g, "paiml/x", now()).unwrap_err();
    assert!(format!("{err:#}").contains("502"));
    let g = Fake::new(vec![rate(4000), Ok(json!({"data":{"repository":null}}))]);
    assert!(fetch_snapshot(&g, "paiml/x", now()).is_err());
}

#[test]
fn a_bad_repo_name_is_refused_before_any_query() {
    for bad in ["x", "a/b/c", "/b", "a/"] {
        let g = Fake::new(vec![]);
        assert!(fetch_snapshot(&g, bad, now()).is_err(), "{bad}");
        assert!(g.seen.borrow().is_empty());
    }
}

#[test]
fn runaway_pagination_stops_at_the_page_cap() {
    let mut replies = vec![rate(4000)];
    for _ in 0..MAX_PAGES {
        replies.push(issues_page(9999, Some("C"), &[1]));
    }
    let g = Fake::new(replies);
    let err = fetch_snapshot(&g, "paiml/x", now()).unwrap_err();
    assert!(format!("{err:#}").contains("not measured"), "{err:#}");
}

#[test]
fn prs_carry_draft_age_branch_labels_body_and_the_rollup() {
    let node = |n: u64, rollup: Value| {
        json!({"number":n,"title":"p","isDraft":n==2,"createdAt":"2026-11-05T08:00:00Z",
               "headRefName":"train/1","body":"Part of a/b#1",
               "labels":{"nodes":[{"name":"x"}]},
               "commits":{"nodes":[{"commit":{"statusCheckRollup":rollup}}]}})
    };
    let g = Fake::new(vec![
        rate(4000),
        issues_page(0, None, &[]),
        prs_page(json!([
            node(1, json!({"state":"SUCCESS"})),
            node(2, json!({"state":"PENDING"})),
            node(3, json!({"state":"FAILURE"})),
            node(4, Value::Null),
        ])),
    ]);
    let s = fetch_snapshot(&g, "paiml/x", now()).expect("fixture");
    let checks: Vec<CheckState> = s.open_prs.iter().map(|p| p.checks).collect();
    assert_eq!(
        checks,
        [
            CheckState::Pass,
            CheckState::Pending,
            CheckState::Fail,
            CheckState::None
        ]
    );
    assert!(s.open_prs[1].is_draft && !s.open_prs[0].is_draft);
    assert_eq!(s.open_prs[0].head_ref, "train/1");
    assert_eq!(s.open_prs[0].labels, ["x"]);
    assert_eq!(s.open_prs[0].created_at, now() - chrono::Duration::hours(4));
}

#[test]
fn only_queries_reach_github() {
    let err = GhGraphql.query("mutation { addComment }", &[]).unwrap_err();
    assert!(format!("{err:#}").contains("queries only"));
}

#[test]
fn snapshot_file_round_trip_and_errors() {
    let dir = tempfile::tempdir().expect("fixture");
    let path = dir.path().join("s.json");
    let s = RepoSnapshot {
        repo: "a/b".into(),
        taken_at: now(),
        open_issue_total: 0,
        open_issues: vec![],
        open_prs: vec![],
    };
    std::fs::write(&path, serde_json::to_string(&s).expect("fixture")).expect("fixture");
    assert_eq!(load_snapshot_file(&path).expect("fixture"), s);
    assert!(load_snapshot_file(&dir.path().join("missing.json")).is_err());
    std::fs::write(&path, "{").expect("fixture");
    assert!(load_snapshot_file(&path).is_err());
}

fn filing(title: &str, replies: Vec<Result<Value, String>>) -> anyhow::Result<()> {
    let g = Fake::new(replies);
    guard_filing(&g, "paiml/x", &Policy::default(), title, 50, 3)
}

#[test]
fn filing_guard_refuses_a_fitting_title_admits_a_distinct_one_and_fails_closed() {
    let open = || {
        vec![
            rate(4000),
            issues_page(2, None, &[1, 2]),
            prs_page(json!([])),
        ]
    };
    // issues_page titles are "t1", "t2": a title of exactly "t2" fits #2
    let err = filing("t2", open()).unwrap_err();
    assert!(format!("{err:#}").contains("R5"), "{err:#}");
    filing("wholly different work item", open()).expect("fixture");
    let err = filing("anything", vec![rate(4000), Err("HTTP 502".into())]).unwrap_err();
    assert!(
        format!("{err:#}").contains("nothing was linked or written"),
        "{err:#}"
    );
}

#[test]
fn filing_guard_refuses_at_the_cap_with_the_graphql_total() {
    let err = filing(
        "wholly different work item",
        vec![
            rate(4000),
            issues_page(100, None, &[1]),
            prs_page(json!([])),
        ],
    )
    .unwrap_err();
    assert!(format!("{err:#}").contains("R1"), "{err:#}");
}
