//! Each finding: when it fires, when it does not, and what the loop is told to do.

use super::findings::*;
use super::test_support::{now, rec, ticket_from};
use chrono::Duration;
use std::collections::BTreeSet;

fn run(tickets: &[TicketRec]) -> FindingsReport {
    run_with(
        tickets,
        &BTreeSet::new(),
        tickets.len(),
        &Settings::default(),
    )
}

fn run_with(
    tickets: &[TicketRec],
    gone: &BTreeSet<String>,
    open_total: usize,
    s: &Settings,
) -> FindingsReport {
    evaluate(
        Evidence {
            repo: "o/r",
            now: now(),
            open_total,
            tickets,
            gone_paths: gone,
            not_measured: vec!["V3: a note".to_string()],
        },
        s,
    )
}

fn only<'a>(r: &'a FindingsReport, id: &str) -> &'a Finding {
    let found: Vec<&Finding> = r.findings.iter().filter(|f| f.id == id).collect();
    assert_eq!(found.len(), 1, "{id}: {:?}", r.findings);
    found[0]
}

#[test]
fn ticket_quality_findings_clean_ticket_has_none_and_every_id_is_counted() {
    let r = run(&[rec(5, "Add the arm rule")]);
    assert_eq!(r.findings, []);
    assert_eq!(
        r.counts.keys().copied().collect::<Vec<_>>().len(),
        IDS.len()
    );
    assert!(r.counts.values().all(|n| *n == 0));
    assert_eq!(
        (r.schema, r.repo.as_str(), r.open_issues, r.pass_mark),
        (SCHEMA, "o/r", 1, 70)
    );
    assert_eq!(r.not_measured, ["V3: a note"]);
}

#[test]
fn ticket_quality_findings_q1_labels_and_comments_a_ticket_under_the_mark() {
    let mut t = rec(5, "Add the arm rule");
    t.body = ticket_from(0b000_0001).body;
    let r = run(&[t]);
    let f = only(&r, "Q1");
    assert_eq!(
        (f.issue, f.action, f.label),
        (Some(5), Action::LabelComment, Some("needs-rewrite"))
    );
    assert_eq!(f.score, Some(55));
    assert_eq!(f.missing.len(), 3);
    assert_eq!(r.counts["Q1"], 1);
}

#[test]
fn ticket_quality_findings_v1_proposes_a_close_only_when_every_linked_pr_is_merged() {
    let mut merged = rec(5, "Add the arm rule");
    merged.linked_prs = vec![(7, PrState::Merged), (8, PrState::Merged)];
    let mut mixed = rec(6, "Port the jq cases");
    mixed.linked_prs = vec![(9, PrState::Merged), (10, PrState::Closed)];
    let r = run(&[merged, mixed]);
    let f = only(&r, "V1");
    assert_eq!((f.issue, f.action), (Some(5), Action::CloseProposal));
    assert!(f.message.contains("#7 #8"), "{}", f.message);
}

#[test]
fn ticket_quality_findings_v2_names_the_older_ticket_as_survivor() {
    let r = run(&[
        rec(9, "Add the arm rule to arbiter"),
        rec(5, "Add the arm rule to arbiter"),
        rec(7, "Write the stale PR rule"),
    ]);
    let f = only(&r, "V2");
    assert_eq!(
        (f.issue, f.target, f.action),
        (Some(9), Some(5), Action::CloseProposal)
    );
    assert!(f.message.contains("survivor #5"), "{}", f.message);
}

#[test]
fn ticket_quality_findings_v3_fires_per_gone_path_only() {
    let mut t = rec(5, "Add the arm rule");
    t.body
        .push_str("\nsee src/gone.rs and src/kept.rs and src/also_gone.rs\n");
    let gone: BTreeSet<String> = ["src/gone.rs", "src/also_gone.rs", "src/other.rs"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let r = run_with(&[t], &gone, 1, &Settings::default());
    let v3: Vec<&Finding> = r.findings.iter().filter(|f| f.id == "V3").collect();
    assert_eq!(v3.len(), 2);
    assert_eq!(v3[0].path.as_deref(), Some("src/gone.rs"));
    assert_eq!(
        (v3[0].action, v3[0].label),
        (Action::LabelComment, Some("needs-revalidate"))
    );
}

#[test]
fn ticket_quality_findings_v4_needs_idle_past_the_limit_and_no_open_pr() {
    let idle = |days: i64, prs: Vec<(u64, PrState)>| {
        let mut t = rec(5, "Add the arm rule");
        t.updated_at = now() - Duration::days(days);
        t.linked_prs = prs;
        run(&[t])
    };
    let r = idle(31, vec![(3, PrState::Closed)]);
    let f = only(&r, "V4");
    assert_eq!(
        (f.action, f.label),
        (Action::Label, Some("needs-revalidate"))
    );
    assert!(f.message.contains("31 days (limit 30)"), "{}", f.message);
    assert_eq!(
        idle(30, vec![]).counts["V4"],
        0,
        "the limit itself is not past it"
    );
    assert_eq!(idle(90, vec![(3, PrState::Open)]).counts["V4"], 0);
}

#[test]
fn ticket_quality_findings_p1_links_to_a_stated_open_epic_else_labels() {
    let orphan = |n: u64, body: &str| {
        let mut t = rec(n, &format!("Write rule number {n}"));
        t.rec.parent = None;
        t.parent_open = None;
        t.body.push_str(body);
        t
    };
    let mut epic = orphan(2, "");
    epic.rec.labels.push("Epic".to_string());
    let mut master = orphan(3, "");
    master.rec.sub_issues = 2;
    let r = run(&[
        epic,
        master,
        orphan(10, "\n**Parent:** #2\n"),
        orphan(11, "\nParent: #99\n"),
        orphan(12, "\nParent: #12\n"),
        orphan(13, "\nParent: #10\n"),
        orphan(14, ""),
    ]);
    let p1: Vec<(Option<u64>, Action, Option<u64>)> = r
        .findings
        .iter()
        .filter(|f| f.id == "P1")
        .map(|f| (f.issue, f.action, f.target))
        .collect();
    assert_eq!(
        p1,
        [
            (Some(10), Action::Link, Some(2)),
            (Some(11), Action::Label, None),
            (Some(12), Action::Label, None),
            (Some(13), Action::Label, None),
            (Some(14), Action::Label, None),
        ],
        "epics and masters are exempt; a parent must be an open epic or master"
    );
    assert_eq!(
        r.findings
            .iter()
            .find(|f| f.issue == Some(14) && f.id == "P1")
            .unwrap()
            .label,
        Some("needs-parent")
    );
    assert_eq!(stated_parent("x\n- parent: #41 (the train)"), Some(41));
    assert_eq!(stated_parent("the parent is #41"), None);
}

#[test]
fn ticket_quality_findings_p2_proposes_closing_a_finished_epic_and_reports_unmeasured() {
    let epic = |n: u64, subs: u64, closed: Option<u64>| {
        let mut t = rec(n, &format!("Train number {n}"));
        t.rec.sub_issues = subs;
        t.rec.sub_issues_closed = closed;
        t
    };
    let r = run(&[epic(2, 3, Some(3)), epic(3, 3, Some(1)), epic(4, 3, None)]);
    let f = only(&r, "P2");
    assert_eq!((f.issue, f.action), (Some(2), Action::CloseProposal));
    assert!(
        r.not_measured.iter().any(|m| m.contains("#4")),
        "{:?}",
        r.not_measured
    );
}

#[test]
fn ticket_quality_findings_p3_counts_open_children_over_eight() {
    let master = |n: u64, subs: u64, closed: u64| {
        let mut t = rec(n, &format!("Train number {n}"));
        t.rec.sub_issues = subs;
        t.rec.sub_issues_closed = Some(closed);
        t
    };
    let r = run(&[master(2, 12, 3), master(3, 12, 4)]);
    let f = only(&r, "P3");
    assert_eq!((f.issue, f.action, f.label), (Some(2), Action::None, None));
    assert!(f.message.contains("9 open children"), "{}", f.message);
}

#[test]
fn ticket_quality_findings_p3_without_a_closed_count_is_not_measured_over_the_limit() {
    let unread = |n: u64, subs: u64| {
        let mut t = rec(n, &format!("Train number {n}"));
        t.rec.sub_issues = subs;
        t.rec.sub_issues_closed = None;
        t
    };
    let r = run(&[unread(2, 9), unread(3, 8)]);
    assert_eq!(r.counts["P3"], 0);
    let notes: Vec<&String> = r
        .not_measured
        .iter()
        .filter(|m| m.starts_with("P3:"))
        .collect();
    assert_eq!(
        notes,
        ["P3: #2 has 9 sub-issues but the read gave no closed count"],
        "8 children cannot be over the limit, so only #2 is unjudged"
    );
}

#[test]
fn ticket_quality_findings_p4_fires_over_the_cap_on_githubs_count() {
    let s = Settings {
        ticket_cap: 2,
        ..Settings::default()
    };
    let t = [rec(5, "Add the arm rule")];
    let over = run_with(&t, &BTreeSet::new(), 3, &s);
    let f = only(&over, "P4");
    assert_eq!((f.issue, f.action), (None, Action::None));
    assert!(
        f.message.contains("3 open tickets, over the cap of 2"),
        "{}",
        f.message
    );
    assert_eq!(over.open_issues, 3);
    assert_eq!(run_with(&t, &BTreeSet::new(), 2, &s).counts["P4"], 0);
}

#[test]
fn ticket_quality_findings_are_ordered_by_id_then_issue_and_render() {
    let mut a = rec(9, "Add the arm rule");
    a.body = String::new();
    a.linked_prs = vec![(1, PrState::Merged)];
    let mut b = rec(4, "Write the stale rule");
    b.body = String::new();
    let r = run_with(&[a, b], &BTreeSet::new(), 60, &Settings::default());
    let order: Vec<(&str, Option<u64>)> = r.findings.iter().map(|f| (f.id, f.issue)).collect();
    assert_eq!(
        order,
        [
            ("Q1", Some(4)),
            ("Q1", Some(9)),
            ("V1", Some(9)),
            ("P4", None)
        ]
    );
    let text = r.render_text();
    assert!(
        text.contains("ticket findings — o/r (60 open, pass mark 70)"),
        "{text}"
    );
    assert!(text.contains("  Q1: 2\n"), "{text}");
    assert!(
        text.contains("  V1 #9: every linked PR is merged"),
        "{text}"
    );
    assert!(text.contains("  P4: 60 open tickets"), "{text}");
    assert!(text.contains("NOT MEASURED: V3: a note"), "{text}");
    let json = serde_json::to_value(&r).unwrap();
    assert_eq!(json["findings"][0]["action"], "label-comment");
    assert!(json["findings"][3].get("issue").is_none());
}

#[test]
fn ticket_quality_findings_ticket_view_carries_parent_and_milestone() {
    let mut t = rec(5, "Add the arm rule");
    assert!(t.ticket().parent_open && t.ticket().has_milestone && !t.is_container());
    t.parent_open = Some(false);
    t.rec.milestone = None;
    t.rec.labels.push("master".to_string());
    assert!(!t.ticket().parent_open && !t.ticket().has_milestone && t.is_container());
}
