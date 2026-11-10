//! Failing-first tests for the API-measured runner smells (RS-13, RS-14, RS-15).
//! Every test is offline: runs come from fixtures, never from GitHub.
use super::*;
use chrono::{Duration, TimeZone, Utc};

fn t0() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 10, 0, 0, 0).unwrap()
}

fn run(queue_secs: Option<i64>, conclusion: Option<&str>) -> RunSample {
    RunSample {
        created_at: t0(),
        started_at: queue_secs.map(|s| t0() + Duration::seconds(s)),
        conclusion: conclusion.map(str::to_string),
    }
}

fn window(runs_total: u64, merged_prs: u64, runs: Vec<RunSample>) -> Window {
    Window {
        repo: "o/r".into(),
        since: t0(),
        runs_total,
        runs,
        merged_prs,
    }
}

fn verdict(f: &Finding) -> Verdict {
    f.verdict
}

#[test]
fn runs_per_merge_passes_at_or_below_the_warn_bound() {
    let f = runs_per_merge(&window(150, 10, vec![]), &Thresholds::default());
    assert_eq!(verdict(&f), Verdict::Pass, "{}", f.message);
    assert!(f.message.contains("15.0"), "{}", f.message);
}

#[test]
fn runs_per_merge_warns_above_15_and_fails_above_30() {
    let warn = runs_per_merge(&window(151, 10, vec![]), &Thresholds::default());
    assert_eq!(verdict(&warn), Verdict::Warn, "{}", warn.message);
    let still_warn = runs_per_merge(&window(300, 10, vec![]), &Thresholds::default());
    assert_eq!(
        verdict(&still_warn),
        Verdict::Warn,
        "{}",
        still_warn.message
    );
    let fail = runs_per_merge(&window(301, 10, vec![]), &Thresholds::default());
    assert_eq!(verdict(&fail), Verdict::Fail, "{}", fail.message);
}

#[test]
fn runs_per_merge_with_no_merges_counts_the_runs_against_one_merge() {
    let f = runs_per_merge(&window(31, 0, vec![]), &Thresholds::default());
    assert_eq!(verdict(&f), Verdict::Fail, "{}", f.message);
    assert!(f.message.contains("no merged PR"), "{}", f.message);
    let quiet = runs_per_merge(&window(0, 0, vec![]), &Thresholds::default());
    assert_eq!(verdict(&quiet), Verdict::Pass, "{}", quiet.message);
}

#[test]
fn runs_per_merge_thresholds_come_from_the_options() {
    let t = Thresholds {
        runs_per_merge_warn: 2.0,
        runs_per_merge_fail: 4.0,
        ..Thresholds::default()
    };
    assert_eq!(
        verdict(&runs_per_merge(&window(21, 10, vec![]), &t)),
        Verdict::Warn
    );
    assert_eq!(
        verdict(&runs_per_merge(&window(41, 10, vec![]), &t)),
        Verdict::Fail
    );
}

#[test]
fn cancelled_share_counts_only_finished_runs() {
    // 2 cancelled of 4 finished (the in-progress run has no conclusion) = 50%.
    let runs = vec![
        run(Some(1), Some("cancelled")),
        run(Some(1), Some("cancelled")),
        run(Some(1), Some("success")),
        run(Some(1), Some("failure")),
        run(Some(1), None),
    ];
    let f = cancelled_share(&window(5, 1, runs), &Thresholds::default());
    assert_eq!(verdict(&f), Verdict::Warn, "{}", f.message);
    assert!(f.message.contains("50.0%"), "{}", f.message);
}

#[test]
fn cancelled_share_bounds_are_strict_greater_than() {
    let mk = |cancelled: usize, total: usize| {
        let mut v = vec![run(Some(1), Some("cancelled")); cancelled];
        v.extend(vec![run(Some(1), Some("success")); total - cancelled]);
        cancelled_share(&window(total as u64, 1, v), &Thresholds::default())
    };
    assert_eq!(verdict(&mk(40, 100)), Verdict::Pass);
    assert_eq!(verdict(&mk(41, 100)), Verdict::Warn);
    assert_eq!(verdict(&mk(60, 100)), Verdict::Warn);
    assert_eq!(verdict(&mk(61, 100)), Verdict::Fail);
}

#[test]
fn cancelled_share_with_no_finished_run_is_not_measured() {
    let f = cancelled_share(&window(1, 1, vec![run(None, None)]), &Thresholds::default());
    assert_eq!(verdict(&f), Verdict::NotMeasured, "{}", f.message);
    let none = cancelled_share(&window(0, 0, vec![]), &Thresholds::default());
    assert_eq!(verdict(&none), Verdict::NotMeasured, "{}", none.message);
}

#[test]
fn percentile_is_nearest_rank() {
    let v: Vec<i64> = (1..=100).collect();
    assert_eq!(percentile(&v, 95.0), 95);
    assert_eq!(percentile(&v, 100.0), 100);
    assert_eq!(percentile(&[7], 95.0), 7);
    assert_eq!(percentile(&[1, 2, 3, 4], 50.0), 2);
}

#[test]
fn queue_p95_judges_the_slow_tail_not_the_median() {
    // 19 fast runs and one 20-minute wait: the median is 10 s, the p95 is the wait.
    let mut runs: Vec<RunSample> = (0..19).map(|_| run(Some(10), Some("success"))).collect();
    runs.push(run(Some(1200), Some("success")));
    let f = queue_p95(&window(20, 2, runs), &Thresholds::default());
    assert_eq!(
        verdict(&f),
        Verdict::Pass,
        "p95 of 20 is rank 19 = 10 s: {}",
        f.message
    );
    let mut two_slow: Vec<RunSample> = (0..18).map(|_| run(Some(10), Some("success"))).collect();
    two_slow.push(run(Some(1200), Some("success")));
    two_slow.push(run(Some(1200), Some("success")));
    let g = queue_p95(&window(20, 2, two_slow), &Thresholds::default());
    assert_eq!(verdict(&g), Verdict::Fail, "{}", g.message);
}

#[test]
fn queue_p95_warns_between_the_bounds() {
    let runs: Vec<RunSample> = (0..10).map(|_| run(Some(400), Some("success"))).collect();
    let f = queue_p95(&window(10, 1, runs), &Thresholds::default());
    assert_eq!(verdict(&f), Verdict::Warn, "{}", f.message);
    assert!(f.message.contains("400"), "{}", f.message);
}

#[test]
fn queue_p95_with_no_started_run_is_not_measured() {
    let f = queue_p95(
        &window(2, 1, vec![run(None, None), run(None, Some("cancelled"))]),
        &Thresholds::default(),
    );
    assert_eq!(verdict(&f), Verdict::NotMeasured, "{}", f.message);
}

#[test]
fn a_start_before_the_create_is_clamped_to_zero_not_negative() {
    let runs = vec![run(Some(-30), Some("success"))];
    let f = queue_p95(&window(1, 1, runs), &Thresholds::default());
    assert_eq!(verdict(&f), Verdict::Pass, "{}", f.message);
    assert!(f.message.contains("0 s"), "{}", f.message);
}

#[test]
fn a_sampled_window_says_so() {
    let runs = vec![run(Some(1), Some("success")); 3];
    let f = cancelled_share(&window(300, 1, runs), &Thresholds::default());
    assert!(f.message.contains("sample of 3 of 300"), "{}", f.message);
}

struct Fixture(Result<Window, String>);
impl ActionsApi for Fixture {
    fn window(&self, _repo: &str, _since: DateTime<Utc>) -> Result<Window, String> {
        self.0.clone()
    }
}

#[test]
fn a_failed_read_is_not_measured_for_all_three_rules() {
    let api = Fixture(Err("HTTP 403: rate limit exceeded".into()));
    let out = evaluate(&api, "o/r", t0(), &Thresholds::default());
    assert_eq!(out.len(), 3);
    for f in &out {
        assert_eq!(verdict(f), Verdict::NotMeasured, "{} {}", f.id, f.message);
        assert!(f.message.contains("rate limit"), "{}", f.message);
    }
    let ids: Vec<&str> = out.iter().map(|f| f.id).collect();
    assert_eq!(ids, ["RS-13", "RS-14", "RS-15"]);
}

#[test]
fn evaluate_runs_all_three_rules_on_one_window() {
    let runs = vec![run(Some(5), Some("success")); 4];
    let api = Fixture(Ok(window(4, 2, runs)));
    let out = evaluate(&api, "o/r", t0(), &Thresholds::default());
    assert!(out.iter().all(|f| f.verdict == Verdict::Pass), "{out:?}");
}

#[test]
fn parse_runs_page_reads_total_and_timestamps() {
    let json = r#"{"total_count":3,"workflow_runs":[
        {"created_at":"2026-11-10T00:00:00Z","run_started_at":"2026-11-10T00:00:07Z","conclusion":"success"},
        {"created_at":"2026-11-10T00:01:00Z","run_started_at":"2026-11-10T00:01:00Z","conclusion":"cancelled"},
        {"created_at":"2026-11-10T00:02:00Z","run_started_at":null,"conclusion":null}]}"#;
    let (total, runs) = parse_runs_page(json).unwrap();
    assert_eq!(total, 3);
    assert_eq!(runs.len(), 3);
    assert_eq!(
        (runs[0].started_at.unwrap() - runs[0].created_at).num_seconds(),
        7
    );
    assert_eq!(runs[1].conclusion.as_deref(), Some("cancelled"));
    assert!(runs[2].started_at.is_none() && runs[2].conclusion.is_none());
}

#[test]
fn parse_runs_page_refuses_a_body_that_is_not_a_runs_page() {
    assert!(parse_runs_page("{\"message\":\"API rate limit exceeded\"}").is_err());
    assert!(parse_runs_page("not json").is_err());
    assert!(parse_runs_page(
        "{\"total_count\":1,\"workflow_runs\":[{\"created_at\":\"garbage\"}]}"
    )
    .is_err());
}

#[test]
fn parse_merged_count_reads_the_search_count_and_refuses_graphql_errors() {
    assert_eq!(
        parse_merged_count("{\"data\":{\"search\":{\"issueCount\":7}}}").unwrap(),
        7
    );
    assert!(
        parse_merged_count("{\"data\":null,\"errors\":[{\"message\":\"RATE_LIMITED\"}]}").is_err()
    );
    assert!(parse_merged_count("{\"data\":{\"search\":null}}").is_err());
}

#[test]
fn the_merged_query_is_a_graphql_search_scoped_to_the_repo_and_window() {
    let q = merged_query("paiml/x", t0());
    assert!(q.contains("repo:paiml/x"), "{q}");
    assert!(q.contains("is:merged"), "{q}");
    assert!(q.contains("merged:>=2026-11-10T00:00:00Z"), "{q}");
    assert!(q.contains("issueCount"), "{q}");
}

#[test]
fn thresholds_read_options_and_refuse_a_non_number() {
    let mut o: std::collections::HashMap<&str, Option<f64>> = std::collections::HashMap::new();
    o.insert("runs_per_merge_warn", Some(5.0));
    o.insert("queue_p95_fail_secs", Some(60.0));
    let t = Thresholds::from_lookup(&|k| o.get(k).copied()).unwrap();
    assert_eq!(t.runs_per_merge_warn, 5.0);
    assert_eq!(t.queue_p95_fail_secs, 60);
    assert_eq!(t.runs_per_merge_fail, 30.0);
    o.insert("cancelled_warn", None);
    assert!(Thresholds::from_lookup(&|k| o.get(k).copied())
        .unwrap_err()
        .contains("cancelled_warn"));
}
