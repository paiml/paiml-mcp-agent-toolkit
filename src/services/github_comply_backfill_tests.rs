//! GHC-001 backfill (OUT-204 step 4): open issues with >= 1 merged referencing PR,
//! classified DONE / PARTIAL / WRONG-LINK.

use super::backfill::*;
use super::source::*;
use super::*;
use chrono::TimeZone;
use serde_json::{json, Value};
use std::cell::RefCell;

fn issue(n: u64, title: &str) -> IssueRec {
    IssueRec {
        number: n,
        title: title.into(),
        milestone: None,
        parent: None,
        labels: vec![],
    }
}

fn merged(n: u64, title: &str, body: &str) -> MergedPr {
    MergedPr {
        number: n,
        title: title.into(),
        body: body.into(),
    }
}

fn snap(issues: Vec<IssueRec>) -> RepoSnapshot {
    RepoSnapshot {
        repo: "paiml/x".into(),
        taken_at: chrono::Utc
            .with_ymd_and_hms(2026, 11, 5, 12, 0, 0)
            .single()
            .expect("fixture"),
        open_issue_total: issues.len(),
        open_issues: issues,
        open_prs: vec![],
    }
}

fn verdict_of(report: &BackfillReport, issue: u64) -> Option<Verdict> {
    report
        .rows
        .iter()
        .find(|r| r.issue == issue)
        .map(|r| r.verdict)
}

#[test]
fn closing_ref_with_a_title_in_common_is_done() {
    let s = snap(vec![issue(7, "Add the comply github command")]);
    let m = [merged(
        70,
        "feat: comply github command",
        "Closes paiml/x#7",
    )];
    let r = backfill(&s, &m);
    assert_eq!(verdict_of(&r, 7), Some(Verdict::Done));
    assert_eq!(r.rows[0].prs, vec![70]);
}

#[test]
fn part_of_only_is_partial() {
    let s = snap(vec![issue(7, "Add the comply github command")]);
    let m = [merged(
        70,
        "feat: comply github command",
        "Part of paiml/x#7",
    )];
    assert_eq!(verdict_of(&backfill(&s, &m), 7), Some(Verdict::Partial));
}

#[test]
fn closing_ref_to_an_unrelated_pr_is_wrong_link() {
    let s = snap(vec![issue(7, "Add the comply github command")]);
    let m = [merged(70, "docs: fix typo in changelog", "Fixes paiml/x#7")];
    assert_eq!(verdict_of(&backfill(&s, &m), 7), Some(Verdict::WrongLink));
}

#[test]
fn a_fitting_closer_outweighs_an_unrelated_one_and_a_part_of() {
    let s = snap(vec![issue(7, "Add the comply github command")]);
    let m = [
        merged(70, "docs: fix typo in changelog", "Fixes paiml/x#7"),
        merged(71, "feat: comply github command", "Closes paiml/x#7"),
        merged(72, "chore: more", "Part of paiml/x#7"),
    ];
    let r = backfill(&s, &m);
    assert_eq!(verdict_of(&r, 7), Some(Verdict::Done));
    assert_eq!(r.rows[0].prs, vec![70, 71, 72]);
}

#[test]
fn issues_without_a_merged_reference_get_no_row() {
    let s = snap(vec![
        issue(7, "Add the comply github command"),
        issue(8, "Other"),
    ]);
    let m = [merged(
        70,
        "feat: comply github command",
        "Closes paiml/x#7",
    )];
    let r = backfill(&s, &m);
    assert_eq!(r.rows.len(), 1);
    assert_eq!(verdict_of(&r, 8), None);
}

#[test]
fn a_reference_into_another_repo_or_a_bare_number_is_not_a_link() {
    let s = snap(vec![issue(7, "Add the comply github command")]);
    let m = [
        merged(70, "feat: comply github command", "Closes paiml/other#7"),
        merged(71, "feat: comply github command", "Closes #7"),
    ];
    assert!(backfill(&s, &m).rows.is_empty());
}

#[test]
fn repo_name_matches_case_insensitively() {
    let s = snap(vec![issue(7, "Add the comply github command")]);
    let m = [merged(
        70,
        "feat: comply github command",
        "closes PAIML/X#7",
    )];
    assert_eq!(verdict_of(&backfill(&s, &m), 7), Some(Verdict::Done));
}

#[test]
fn text_table_names_every_verdict_and_json_round_trips() {
    let s = snap(vec![issue(7, "Add the comply github command")]);
    let r = backfill(
        &s,
        &[merged(
            70,
            "feat: comply github command",
            "Closes paiml/x#7",
        )],
    );
    let text = r.render_text();
    assert!(
        text.contains("DONE") && text.contains("#7") && text.contains("#70"),
        "{text}"
    );
    let back: BackfillReport =
        serde_json::from_str(&serde_json::to_string(&r).expect("fixture")).expect("fixture");
    assert_eq!(back, r);
    assert_eq!(Verdict::WrongLink.label(), "WRONG-LINK");
}

// ---- the reader ----

type Seen = Vec<String>;

struct Fake {
    replies: RefCell<Vec<Value>>,
    seen: RefCell<Seen>,
}

impl Graphql for Fake {
    fn query(&self, query: &str, _vars: &[(&str, &str)]) -> anyhow::Result<Value> {
        self.seen.borrow_mut().push(query.to_string());
        Ok(self.replies.borrow_mut().pop().expect("unexpected query"))
    }
}

fn fake(replies: Vec<Value>) -> Fake {
    Fake {
        replies: RefCell::new(replies.into_iter().rev().collect()),
        seen: RefCell::new(vec![]),
    }
}

fn rate(n: u64) -> Value {
    json!({"data":{"rateLimit":{"remaining":n,"resetAt":"x"}}})
}

fn merged_page(next: Option<&str>, nodes: Value) -> Value {
    json!({"data":{"repository":{"pullRequests":{
        "pageInfo":{"hasNextPage":next.is_some(),"endCursor":next},"nodes":nodes}}}})
}

#[test]
fn merged_prs_are_read_by_query_only_after_the_budget_check_and_paginated() {
    let g = fake(vec![
        rate(4300),
        merged_page(
            Some("c1"),
            json!([{"number":1,"title":"a","body":"Closes o/r#9"}]),
        ),
        merged_page(None, json!([{"number":2,"title":"b","body":null}])),
    ]);
    let got = fetch_merged_prs(&g, "o/r").expect("fixture");
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].body, "Closes o/r#9");
    assert_eq!(got[1].body, "");
    let seen = g.seen.borrow();
    assert_eq!(seen.len(), 3);
    assert!(seen[0].contains("rateLimit"));
    assert!(seen[1].contains("states:MERGED"), "{}", seen[1]);
    assert!(seen.iter().all(|q| q.trim_start().starts_with("query")));
}

#[test]
fn a_low_budget_refuses_before_any_merged_pr_read() {
    let g = fake(vec![rate(10)]);
    assert!(fetch_merged_prs(&g, "o/r").is_err());
    assert_eq!(g.seen.borrow().len(), 1);
}

#[test]
fn a_pr_without_a_number_is_an_error_not_a_skip() {
    let g = fake(vec![rate(4300), merged_page(None, json!([{"title":"a"}]))]);
    assert!(fetch_merged_prs(&g, "o/r").is_err());
}
