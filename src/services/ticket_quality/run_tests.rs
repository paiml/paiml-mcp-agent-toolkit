//! The two verbs end to end: what is printed and the exit code.

use super::findings::Settings;
use super::run::*;
use super::test_support::{node, now, page, rate, Fake};
use serde_json::{json, Value};

fn issue(body: &str) -> Result<Value, String> {
    Ok(
        json!({"data":{"repository":{"issue":{"title":"Add x","body":body,
        "milestone":{"title":"m"},"parent":{"number":1,"state":"OPEN"},
        "labels":{"nodes":[{"name":"kind:code"}]}}}}}),
    )
}

const GOOD: &str =
    "## Observed\na\n## Expected\nb\nin src/a.rs\n## Acceptance\n- [ ] `make gate` is green\n";

#[test]
fn ticket_quality_run_score_exits_0_at_the_mark_1_under_it_and_2_unread() {
    let (text, code) = run_score(
        &Fake::new(vec![rate(5000), issue(GOOD)]),
        "o/r",
        3,
        70,
        false,
    );
    assert_eq!(code, 0, "{text}");
    assert!(text.starts_with("o/r#3 ticket score: 100/100"), "{text}");

    let (text, code) = run_score(
        &Fake::new(vec![rate(5000), issue("nothing")]),
        "o/r",
        3,
        70,
        true,
    );
    assert_eq!(code, 1, "{text}");
    let v: Value = serde_json::from_str(&text).expect("run fixture JSON parses");
    assert_eq!(
        (v["total"].as_u64(), v["passed"].as_bool()),
        (Some(30), Some(false))
    );
    assert_eq!(v["items"].as_array().expect("items is an array").len(), 7);
    assert_eq!(
        v["missing"].as_array().expect("missing is an array").len(),
        4
    );

    let (text, code) = run_score(&Fake::new(vec![Err("offline".into())]), "o/r", 3, 70, true);
    assert_eq!(code, EXIT_NOT_MEASURED);
    assert!(
        text.starts_with("not_measured: o/r#3:") && text.contains("offline"),
        "{text}"
    );
}

#[test]
fn ticket_quality_run_findings_reads_once_and_writes_nothing() {
    let paths = Ok(json!({"data":{"repository":{"p0":{"__typename":"Blob"},"p1":null}}}));
    let history = Ok(json!({"data":{"repository":{"defaultBranchRef":{"target":{
        "p0":{"nodes":[{"oid":"abc"}]}}}}}}));
    let g = Fake::new(vec![rate(5000), page(1, vec![node(5)]), paths, history]);
    let (text, code) = run_findings(&g, "o/r", &Settings::default(), true, now());
    assert_eq!(code, 0, "{text}");
    let v: Value = serde_json::from_str(&text).expect("run fixture JSON parses");
    assert_eq!(v["schema"], "pmat-work-findings/v1");
    assert_eq!(
        (
            v["counts"]["Q1"].as_u64(),
            v["counts"]["V1"].as_u64(),
            v["counts"]["V3"].as_u64()
        ),
        (Some(1), Some(1), Some(1))
    );
    assert!(
        g.seen.borrow().iter().all(|q| q.starts_with("query")),
        "reads only"
    );

    let g = Fake::new(vec![rate(5000), page(0, vec![])]);
    let (text, code) = run_findings(&g, "o/r", &Settings::default(), false, now());
    assert_eq!(code, 0);
    assert!(
        text.starts_with("ticket findings — o/r (0 open, pass mark 70)"),
        "{text}"
    );
}

#[test]
fn ticket_quality_run_findings_is_not_measured_when_a_read_fails() {
    for replies in [
        vec![rate(3)],
        vec![rate(5000), page(1, vec![node(5)]), Err("boom".into())],
    ] {
        let (text, code) = run_findings(
            &Fake::new(replies),
            "o/r",
            &Settings::default(),
            true,
            now(),
        );
        assert_eq!(code, EXIT_NOT_MEASURED);
        assert!(text.starts_with("not_measured: o/r:"), "{text}");
    }
}
