//! The reader against a fake GitHub: what it asks, what it refuses.

use super::findings::PrState;
use super::source::*;
use super::test_support::{node, page, rate, rec, Fake};
use serde_json::{json, Value};

fn ok(v: Value) -> Result<Value, String> {
    Ok(json!({"data":{"repository": v}}))
}

#[test]
fn ticket_quality_source_reads_every_open_ticket_in_one_paged_read() {
    let g = Fake::new(vec![rate(5000), page(2, vec![node(5), node(6)])]);
    let (total, tickets) = fetch_tickets(&g, "o/r").unwrap();
    assert_eq!((total, tickets.len()), (2, 2));
    let t = &tickets[0];
    assert_eq!((t.rec.number, t.parent_open), (5, Some(true)));
    assert_eq!(t.linked_prs, [(7, PrState::Merged)]);
    assert_eq!(t.updated_at.to_rfc3339(), "2026-11-26T10:00:00+00:00");
    let seen = g.seen.borrow();
    assert_eq!(seen.len(), 2, "the budget, then one page");
    assert!(seen[1].contains("first:100") && seen[1].starts_with("query"));
}

#[test]
fn ticket_quality_source_refuses_a_bad_repo_a_low_budget_and_a_broken_page() {
    assert!(fetch_tickets(&Fake::new(vec![]), "nope").is_err());
    assert!(fetch_ticket(&Fake::new(vec![]), "o/r/x", 1).is_err());
    assert!(gone_paths(&Fake::new(vec![]), "/r", &[]).is_err());
    assert!(fetch_tickets(&Fake::new(vec![rate(10)]), "o/r").is_err());
    assert!(fetch_tickets(&Fake::new(vec![rate(5000), Err("boom".into())]), "o/r").is_err());
    let no_total = Ok(json!({"data":{"repository":{"issues":{
        "pageInfo":{"hasNextPage":false},"nodes":[]}}}}));
    assert!(fetch_tickets(&Fake::new(vec![rate(5000), no_total]), "o/r").is_err());
    let no_nodes = Ok(json!({"data":{"repository":{"issues":{
        "totalCount":0,"pageInfo":{"hasNextPage":false}}}}}));
    assert!(fetch_tickets(&Fake::new(vec![rate(5000), no_nodes]), "o/r").is_err());
}

#[test]
fn ticket_quality_source_a_malformed_ticket_is_an_error_never_skipped() {
    let broken = |edit: fn(&mut Value)| {
        let mut n = node(5);
        edit(&mut n);
        parse_ticket(&n).unwrap_err().to_string()
    };
    assert!(broken(|n| n["updatedAt"] = json!(null)).contains("no updatedAt"));
    assert!(broken(|n| n["updatedAt"] = json!("yesterday")).contains("bad updatedAt"));
    assert!(broken(|n| n["closedByPullRequestsReferences"] = json!(null)).contains("no closedBy"));
    assert!(
        broken(|n| n["closedByPullRequestsReferences"]["nodes"][0]["number"] = json!(null))
            .contains("no number")
    );
    assert!(
        broken(|n| n["closedByPullRequestsReferences"]["nodes"][0]["state"] = json!("DRAFT"))
            .contains("unknown pull request state")
    );
    assert!(broken(|n| n["number"] = json!(null)).contains("no number"));
    let mut n = node(5);
    n["parent"] = json!(null);
    n["closedByPullRequestsReferences"]["nodes"] =
        json!([{"number":1,"state":"OPEN"},{"number":2,"state":"CLOSED"}]);
    let t = parse_ticket(&n).unwrap();
    assert_eq!(t.parent_open, None);
    assert_eq!(t.linked_prs, [(1, PrState::Open), (2, PrState::Closed)]);
}

#[test]
fn ticket_quality_source_reads_one_ticket_for_the_rubric() {
    let reply = ok(
        json!({"issue":{"title":"Add x","body":"b","milestone":{"title":"m"},
        "parent":{"number":1,"state":"OPEN"},"labels":{"nodes":[{"name":"kind:code"}]}}}),
    );
    let g = Fake::new(vec![rate(5000), reply]);
    let t = fetch_ticket(&g, "o/r", 12).unwrap();
    assert_eq!((t.title.as_str(), t.body.as_str()), ("Add x", "b"));
    assert_eq!(t.labels, ["kind:code"]);
    assert!(t.parent_open && t.has_milestone);
    assert!(g.seen.borrow()[1].contains("issue(number:12)"));

    let bare = ok(
        json!({"issue":{"title":"Add x","body":null,"milestone":null,
        "parent":{"number":1,"state":"CLOSED"},"labels":null}}),
    );
    let t = fetch_ticket(&Fake::new(vec![rate(5000), bare]), "o/r", 12).unwrap();
    assert_eq!((t.body.as_str(), t.labels.len()), ("", 0));
    assert!(!t.parent_open && !t.has_milestone);

    let none = Fake::new(vec![rate(5000), ok(json!({"issue":null}))]);
    let err = fetch_ticket(&none, "o/r", 12).unwrap_err().to_string();
    assert!(err.contains("o/r#12: no such issue"), "{err}");
    assert!(fetch_ticket(&Fake::new(vec![rate(5000)]), "o/r", 12).is_err());
    assert!(fetch_ticket(&Fake::new(vec![rate(1)]), "o/r", 12).is_err());
}

#[test]
fn ticket_quality_source_gone_is_absent_at_head_and_present_in_history() {
    let mut t = rec(5, "Add x");
    t.body = "src/kept.rs src/gone.rs src/new.rs".to_string();
    // sorted: gone, kept, new -> p0, p1, p2
    let g = Fake::new(vec![
        ok(json!({"p0":null,"p1":{"__typename":"Blob"},"p2":null})),
        ok(json!({"defaultBranchRef":{"target":{
            "p0":{"nodes":[{"oid":"abc"}]},"p1":{"nodes":[]}}}})),
    ]);
    let (gone, notes) = gone_paths(&g, "o/r", &[t.clone()]).unwrap();
    assert_eq!(gone.into_iter().collect::<Vec<_>>(), ["src/gone.rs"]);
    assert!(notes.is_empty());
    let seen = g.seen.borrow();
    assert!(
        seen[0].contains("p0:object(expression:\"HEAD:src/gone.rs\"){__typename}"),
        "{}",
        seen[0]
    );
    assert!(seen[1].contains("defaultBranchRef{target{... on Commit{p0:history(first:1,path:\"src/gone.rs\"){nodes{oid}} p1:history(first:1,path:\"src/new.rs\")"), "{}", seen[1]);

    let none = Fake::new(vec![]);
    let (gone, _) = gone_paths(&none, "o/r", &[rec(5, "Add x")])
        .map(|(g, n)| (g, n))
        .unwrap_or_default();
    assert!(
        gone.is_empty(),
        "a ticket whose files exist asks one question and gets none gone"
    );
}

#[test]
fn ticket_quality_source_path_reads_that_fail_are_errors() {
    let mut t = rec(5, "Add x");
    t.body = "src/gone.rs".to_string();
    let tickets = [t];
    assert!(gone_paths(&Fake::new(vec![Err("boom".into())]), "o/r", &tickets).is_err());
    assert!(gone_paths(&Fake::new(vec![ok(json!(null))]), "o/r", &tickets).is_err());
    let absent = || ok(json!({"p0":null}));
    assert!(gone_paths(
        &Fake::new(vec![absent(), Err("boom".into())]),
        "o/r",
        &tickets
    )
    .is_err());
    let no_history = ok(json!({"defaultBranchRef":null}));
    let err = gone_paths(&Fake::new(vec![absent(), no_history]), "o/r", &tickets).unwrap_err();
    assert!(
        err.to_string().contains("no history for src/gone.rs"),
        "{err}"
    );
}

#[test]
fn ticket_quality_source_caps_the_paths_it_asks_about_and_says_so() {
    let mut t = rec(5, "Add x");
    t.body = (0..MAX_PATHS + 5)
        .map(|i| format!("src/f{i:04}.rs "))
        .collect();
    let present = |n: usize| {
        let fields: serde_json::Map<String, Value> = (0..n)
            .map(|i| (format!("p{i}"), json!({"__typename":"Blob"})))
            .collect();
        ok(Value::Object(fields))
    };
    let replies = (0..MAX_PATHS / PATHS_PER_QUERY)
        .map(|_| present(PATHS_PER_QUERY))
        .collect();
    let g = Fake::new(replies);
    let (gone, notes) = gone_paths(&g, "o/r", &[t]).unwrap();
    assert!(gone.is_empty());
    assert_eq!(
        notes,
        [format!(
            "V3: {} distinct paths named, only the first {MAX_PATHS} were checked",
            MAX_PATHS + 5
        )]
    );
    assert_eq!(g.seen.borrow().len(), MAX_PATHS / PATHS_PER_QUERY);
}
