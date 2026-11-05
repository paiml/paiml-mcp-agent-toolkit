//! GHC-001 falsification tests.
//!
//! The spec (`Sovereign-AI-Stack claude/GHC-001-github-comply.md`) was not
//! available when these were written; OUT-204 is the whole statement. The five
//! FALSIFY ids are therefore assigned by this file, one per behaviour OUT-204
//! names, each with a planted violation AND the boundary that must pass:
//!
//! - FALSIFY-GHC-001  R1 — 101 open issues violate, 100 pass; target median 50 is not a gate.
//! - FALSIFY-GHC-002  R2 — 5 open non-draft PRs violate, 4 pass; train/* and fix-set/expedite and drafts do not count.
//! - FALSIFY-GHC-003  R3 + R6 — a ticket with neither milestone nor epic, and a PR body with no
//!   `owner/repo#N` reference, are each caught; the linked forms pass.
//! - FALSIFY-GHC-004  R4 — a non-draft PR older than 4 h without a PASS violates; PASS, young, draft do not.
//! - FALSIFY-GHC-005  R5 — `work add` refuses a title an open ticket fits and a repo at the R1 cap,
//!   and admits a distinct title under the cap; the ticket never matches itself.
//!
//! Each `falsify_ghc_00N_*` test is the planted-violation arm; `*_control` is the arm that must pass.

use super::*;
use chrono::{Duration, TimeZone};

fn t0() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 5, 12, 0, 0)
        .single()
        .expect("fixture")
}

fn issue(n: u64, title: &str) -> IssueRec {
    IssueRec {
        number: n,
        title: title.into(),
        milestone: Some("0.71".into()),
        parent: None,
        labels: vec![],
        sub_issues: 0,
    }
}

fn pr(n: u64, age_h: i64, checks: CheckState) -> PrRec {
    PrRec {
        number: n,
        title: format!("pr {n}"),
        is_draft: false,
        created_at: t0() - Duration::hours(age_h),
        head_ref: format!("feat/x{n}"),
        labels: vec![],
        body: "Part of paiml/x#1".into(),
        checks,
    }
}

fn snap(issues: Vec<IssueRec>, prs: Vec<PrRec>) -> RepoSnapshot {
    RepoSnapshot {
        repo: "paiml/x".into(),
        taken_at: t0(),
        open_issue_total: issues.len(),
        open_issues: issues,
        open_prs: prs,
    }
}

fn n_issues(n: usize) -> Vec<IssueRec> {
    (1..=n as u64)
        .map(|i| issue(i, &format!("distinct topic {i:04} alpha{i}")))
        .collect()
}

fn rules(report: &Report) -> Vec<Rule> {
    report.violations.iter().map(|v| v.rule).collect()
}

fn run(s: &RepoSnapshot, rs: &[Rule]) -> Report {
    evaluate(s, &Policy::default(), rs)
}

// ---- policy -------------------------------------------------------------

#[test]
fn policy_defaults_are_the_out_204_numbers() {
    let p = Policy::default();
    assert_eq!(p.max_open_issues, 100);
    assert_eq!(p.target_open_issues, 50);
    assert_eq!(p.max_open_prs, 4);
    assert_eq!(p.pr_max_age_hours, 4);
    assert_eq!(p.pr_exempt, ["train/*", "fix-set/expedite"]);
}

#[test]
fn policy_is_per_repo_and_layers_over_defaults() {
    let text = r#"
        [defaults]
        max_open_prs = 6
        [repo."paiml/x"]
        max_open_issues = 30
    "#;
    let x = Policy::from_toml(text, "paiml/x").expect("fixture");
    assert_eq!((x.max_open_issues, x.max_open_prs), (30, 6));
    let other = Policy::from_toml(text, "paiml/y").expect("fixture");
    assert_eq!((other.max_open_issues, other.max_open_prs), (100, 6));
}

#[test]
fn policy_unknown_key_is_an_error_not_a_default() {
    let err = Policy::from_toml("[defaults]\nmax_open_issue = 5\n", "a/b").unwrap_err();
    assert!(format!("{err:#}").contains("max_open_issue"), "{err:#}");
    assert!(Policy::from_toml("[defualts]\n", "a/b").is_err());
}

#[test]
fn policy_rejects_out_of_range_values() {
    assert!(Policy::from_toml("[defaults]\nfit_similarity = 1.5\n", "a/b").is_err());
    assert!(Policy::from_toml("[defaults]\npr_max_age_hours = -1\n", "a/b").is_err());
}

#[test]
fn policy_load_reads_the_file_and_absent_means_defaults() {
    let dir = tempfile::tempdir().expect("fixture");
    assert_eq!(
        Policy::load(dir.path(), "a/b").expect("fixture"),
        Policy::default()
    );
    std::fs::create_dir_all(dir.path().join(".pmat")).expect("fixture");
    std::fs::write(
        dir.path().join(POLICY_PATH),
        "[repo.\"a/b\"]\nmax_open_prs = 9\n",
    )
    .expect("fixture");
    assert_eq!(
        Policy::load(dir.path(), "a/b")
            .expect("fixture")
            .max_open_prs,
        9
    );
    std::fs::write(dir.path().join(POLICY_PATH), "max_open_prs = ").expect("fixture");
    assert!(Policy::load(dir.path(), "a/b").is_err());
}

// ---- FALSIFY-GHC-001: R1 ------------------------------------------------

#[test]
fn falsify_ghc_001_101_open_issues_violate_r1() {
    let r = run(&snap(n_issues(101), vec![]), &[Rule::R1]);
    assert_eq!(rules(&r), [Rule::R1]);
    assert!(!r.ok());
    assert!(
        r.violations[0].message.contains("101"),
        "{:?}",
        r.violations
    );
}

#[test]
fn falsify_ghc_001_control_100_open_issues_pass_r1() {
    let r = run(&snap(n_issues(100), vec![]), &[Rule::R1]);
    assert!(r.ok(), "{:?}", r.violations);
}

#[test]
fn r1_reads_the_authoritative_total_not_the_page() {
    let mut s = snap(n_issues(10), vec![]);
    s.open_issue_total = 250;
    assert_eq!(rules(&run(&s, &[Rule::R1])), [Rule::R1]);
}

#[test]
fn r1_cap_is_per_repo_policy() {
    let p = Policy {
        max_open_issues: 5,
        ..Policy::default()
    };
    let r = evaluate(&snap(n_issues(6), vec![]), &p, &[Rule::R1]);
    assert_eq!(rules(&r), [Rule::R1]);
}

// ---- FALSIFY-GHC-002: R2 ------------------------------------------------

fn prs(n: u64) -> Vec<PrRec> {
    (1..=n).map(|i| pr(i, 1, CheckState::Pass)).collect()
}

#[test]
fn falsify_ghc_002_5_open_non_draft_prs_violate_r2() {
    let r = run(&snap(vec![], prs(5)), &[Rule::R2]);
    assert_eq!(rules(&r), [Rule::R2]);
}

#[test]
fn falsify_ghc_002_control_4_open_non_draft_prs_pass_r2() {
    assert!(run(&snap(vec![], prs(4)), &[Rule::R2]).ok());
}

#[test]
fn r2_exempts_train_branches_the_expedite_set_and_drafts() {
    let mut v = prs(4);
    let mut train = pr(10, 1, CheckState::Pass);
    train.head_ref = "train/0.71".into();
    let mut expedite = pr(11, 1, CheckState::Pass);
    expedite.head_ref = "fix-set/expedite".into();
    let mut labelled = pr(12, 1, CheckState::Pass);
    labelled.labels = vec!["fix-set/expedite".into()];
    let mut draft = pr(13, 1, CheckState::Pass);
    draft.is_draft = true;
    v.extend([train, expedite, labelled, draft]);
    assert!(run(&snap(vec![], v), &[Rule::R2]).ok());
}

#[test]
fn r2_exemption_is_not_a_substring_match() {
    let mut v = prs(4);
    let mut near = pr(10, 1, CheckState::Pass);
    near.head_ref = "feat/train/x".into(); // does not START with train/
    v.push(near);
    assert_eq!(rules(&run(&snap(vec![], v), &[Rule::R2])), [Rule::R2]);
}

// ---- FALSIFY-GHC-003: R3 + R6 -------------------------------------------

fn orphan(n: u64) -> IssueRec {
    IssueRec {
        milestone: None,
        parent: None,
        ..issue(n, "orphan")
    }
}

#[test]
fn falsify_ghc_003_ticket_with_no_milestone_and_no_epic_violates_r3() {
    let r = run(&snap(vec![orphan(7)], vec![]), &[Rule::R3]);
    assert_eq!(rules(&r), [Rule::R3]);
    assert_eq!(r.violations[0].subject, Some(7));
}

#[test]
fn falsify_ghc_003_control_milestone_or_epic_parent_passes_r3() {
    let mut sub = orphan(8);
    sub.parent = Some(1);
    let with_ms = issue(9, "has milestone");
    assert!(run(&snap(vec![sub, with_ms], vec![]), &[Rule::R3]).ok());
}

#[test]
fn r3_exempts_epics_by_label_by_sub_issue_count_and_by_having_an_open_child() {
    let mut labelled = orphan(1);
    labelled.labels = vec!["Epic".into()];
    let mut counted = orphan(2);
    counted.sub_issues = 3;
    let parent_of_open_child = orphan(3);
    let mut child = orphan(4);
    child.parent = Some(3);
    let s = snap(vec![labelled, counted, parent_of_open_child, child], vec![]);
    let r = run(&s, &[Rule::R3]);
    assert!(r.ok(), "{:?}", r.violations);
    // control: an issue that is none of those is still a violation
    let s = snap(vec![orphan(1), orphan(2)], vec![]);
    assert_eq!(run(&s, &[Rule::R3]).violations.len(), 2);
}

#[test]
fn pr_check_reads_a_bare_hash_reference_as_this_repo_for_r3() {
    // the ONLY reference is `Closes #7` (no owner/repo): R3 must still judge #7
    let mut p = pr(5, 1, CheckState::Pass);
    p.body = "Closes #7".into();
    let r = evaluate_pr(
        &snap(vec![orphan(7)], vec![p.clone()]),
        &Policy::default(),
        5,
    );
    assert!(
        r.violations
            .iter()
            .any(|v| v.rule == Rule::R3 && v.subject == Some(7)),
        "{:?}",
        r.violations
    );
    // control: the same bare reference to a ticket with a milestone is not an R3 violation
    let r = evaluate_pr(
        &snap(vec![issue(7, "has milestone")], vec![p.clone()]),
        &Policy::default(),
        5,
    );
    assert!(
        r.violations.iter().all(|v| v.rule != Rule::R3),
        "{:?}",
        r.violations
    );
    // `Part of #7` reads the same way, and `#77` is not `#7`
    p.body = "Part of #77".into();
    let r = evaluate_pr(&snap(vec![orphan(7)], vec![p]), &Policy::default(), 5);
    assert!(
        r.violations.iter().all(|v| v.rule != Rule::R3),
        "{:?}",
        r.violations
    );
}

#[test]
fn pr_check_is_not_red_on_a_part_of_reference_to_an_epic() {
    let mut p = pr(5, 1, CheckState::Pass);
    p.body = "Part of paiml/x#1604".into();
    let mut epic = orphan(1604);
    epic.sub_issues = 2;
    let r = evaluate_pr(&snap(vec![epic], vec![p]), &Policy::default(), 5);
    assert!(r.ok(), "{:?} {:?}", r.violations, r.not_measured);
}

#[test]
fn falsify_ghc_003_pr_body_without_an_owner_repo_reference_violates_r6() {
    for body in [
        "",
        "just prose",
        "Closes #12",           // bare number: owner/repo is required
        "Part of #12",          // ditto
        "closes-ish paiml/x#1", // no keyword boundary
        "see paiml/x#1",        // a mention is not a link
    ] {
        let mut p = pr(1, 1, CheckState::Pass);
        p.body = body.into();
        let r = run(&snap(vec![], vec![p]), &[Rule::R6]);
        assert_eq!(rules(&r), [Rule::R6], "body {body:?} should violate R6");
    }
}

#[test]
fn falsify_ghc_003_control_closing_and_part_of_forms_pass_r6() {
    for body in [
        "Closes paiml/x#12",
        "fixes paiml/pmat.rs#3",
        "RESOLVES a-b/c_d#99",
        "Resolved: paiml/x#4",
        "Part of paiml/x#12",
        "summary\n\npart of  paiml/x#12\n",
        "- Closes paiml/x#12",
    ] {
        assert!(has_issue_reference(body), "{body:?}");
    }
}

#[test]
fn r6_in_the_repo_run_covers_every_open_pr() {
    let mut bad = pr(2, 1, CheckState::Pass);
    bad.body = "nothing".into();
    let r = run(
        &snap(vec![], vec![pr(1, 1, CheckState::Pass), bad]),
        &[Rule::R6],
    );
    assert_eq!(r.violations.len(), 1);
    assert_eq!(r.violations[0].subject, Some(2));
}

#[test]
fn pr_check_judges_r6_on_the_body_and_r3_on_the_referenced_ticket() {
    let mut p = pr(5, 1, CheckState::Pass);
    p.body = "Closes paiml/x#7".into();
    let s = snap(vec![orphan(7)], vec![p]);
    let r = evaluate_pr(&s, &Policy::default(), 5);
    assert_eq!(rules(&r), [Rule::R3]);
    assert_eq!(r.violations[0].subject, Some(7));
    assert_eq!(r.rules_run, [Rule::R3, Rule::R6]);
}

#[test]
fn pr_check_does_not_go_red_on_a_foreign_or_closed_issue_reference() {
    // R6 accepts any owner/repo#N, and the read itself succeeded (a failed read is an
    // error before this point), so a reference into another repo or to an issue that is
    // not open (closed) is nothing to judge, not a permanent NOT MEASURED.
    let mut p = pr(5, 1, CheckState::Pass);
    p.body = "Closes other/repo#7 and Part of paiml/x#99".into();
    let r = evaluate_pr(&snap(vec![], vec![p]), &Policy::default(), 5);
    assert!(r.violations.is_empty());
    assert!(r.not_measured.is_empty(), "{:?}", r.not_measured);
    assert!(r.ok());
    // control: a PR that is not among the open PRs is genuinely not measured
    let r = evaluate_pr(&snap(vec![], vec![]), &Policy::default(), 5);
    assert_eq!(r.not_measured.len(), 1);
    assert!(!r.ok(), "an unmeasured rule must not read as a pass");
}

#[test]
fn pr_check_on_an_unknown_pr_is_not_measured() {
    let r = evaluate_pr(&snap(vec![], vec![]), &Policy::default(), 404);
    assert!(!r.ok());
    assert!(r.not_measured[0].contains("404"));
}

// ---- FALSIFY-GHC-004: R4 ------------------------------------------------

#[test]
fn falsify_ghc_004_old_non_draft_pr_without_a_pass_violates_r4() {
    for state in [CheckState::Pending, CheckState::Fail, CheckState::None] {
        let r = run(&snap(vec![], vec![pr(1, 5, state)]), &[Rule::R4]);
        assert_eq!(rules(&r), [Rule::R4], "{state:?}");
    }
}

#[test]
fn falsify_ghc_004_control_pass_young_exact_boundary_and_draft_do_not_violate() {
    let mut draft = pr(4, 9, CheckState::Pending);
    draft.is_draft = true;
    let v = vec![
        pr(1, 9, CheckState::Pass),    // old but PASS
        pr(2, 1, CheckState::Pending), // no PASS but young
        pr(3, 4, CheckState::Pending), // exactly 4 h is not OVER 4 h
        draft,
    ];
    let r = run(&snap(vec![], v), &[Rule::R4]);
    assert!(r.ok(), "{:?}", r.violations);
}

#[test]
fn r4_age_is_measured_from_the_snapshot_not_the_wall_clock() {
    let mut p = pr(1, 0, CheckState::Pending);
    p.created_at = t0() - Duration::hours(4) - Duration::minutes(1);
    assert_eq!(rules(&run(&snap(vec![], vec![p]), &[Rule::R4])), [Rule::R4]);
}

// ---- FALSIFY-GHC-005: R5 ------------------------------------------------

fn open_tickets() -> Vec<IssueRec> {
    vec![
        issue(1, "Fix flaky tdg grade cache test on Windows"),
        issue(2, "Add pmat comply github command"),
    ]
}

#[test]
fn falsify_ghc_005_work_add_refuses_a_title_an_open_ticket_fits() {
    let s = snap(open_tickets(), vec![]);
    let err = guard_new_ticket(
        &Policy::default(),
        &s,
        "Add pmat comply github command",
        Some(50),
        Some(3),
    )
    .unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("R5") && msg.contains("#2"), "{msg}");
}

#[test]
fn falsify_ghc_005_work_add_refuses_at_the_r1_cap() {
    let s = snap(n_issues(100), vec![]);
    let err = guard_new_ticket(
        &Policy::default(),
        &s,
        "A wholly new idea",
        Some(500),
        Some(3),
    )
    .unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("R1") && msg.contains("100"), "{msg}");
}

#[test]
fn falsify_ghc_005_control_a_distinct_title_under_the_cap_is_admitted() {
    let s = snap(open_tickets(), vec![]);
    guard_new_ticket(
        &Policy::default(),
        &s,
        "Rework the roadmap fragment merger",
        Some(50),
        Some(3),
    )
    .expect("fixture");
}

#[test]
fn r5_a_ticket_never_matches_itself() {
    // --github-issue 2 is the issue being filed: it is already open and has this very title.
    let s = snap(open_tickets(), vec![]);
    guard_new_ticket(
        &Policy::default(),
        &s,
        "Add pmat comply github command",
        Some(2),
        Some(3),
    )
    .expect("fixture");
}

#[test]
fn r1_at_creation_counts_the_others_so_the_ticket_itself_is_not_the_101st() {
    // 100 open including the ticket being filed (#100): the repo holds 99 others.
    let s = snap(n_issues(100), vec![]);
    guard_new_ticket(
        &Policy::default(),
        &s,
        "brand new unrelated work",
        Some(100),
        Some(3),
    )
    .expect("fixture");
}

#[test]
fn r5_similarity_is_token_jaccard_and_threshold_is_policy() {
    let issues = vec![issue(1, "tdg grade cache flaky")];
    let hit = find_fitting_issue("flaky tdg grade cache", &issues, None, 0.6);
    assert!(hit.is_some_and(|(i, s)| i.number == 1 && (s - 1.0).abs() < 1e-9));
    assert!(find_fitting_issue("flaky tdg grade cache", &issues, None, 1.01).is_none());
    assert!(find_fitting_issue("something else entirely", &issues, None, 0.6).is_none());
    assert!(find_fitting_issue("flaky tdg grade cache", &issues, Some(1), 0.6).is_none());
}

#[test]
fn r3_at_creation_needs_an_epic_or_a_milestone_on_the_ticket() {
    let s = snap(vec![orphan(50)], vec![]);
    let err =
        guard_new_ticket(&Policy::default(), &s, "fresh work item", Some(50), None).unwrap_err();
    assert!(format!("{err:#}").contains("R3"));
    // with an epic to be linked under, the ticket will be a sub-issue
    guard_new_ticket(&Policy::default(), &s, "fresh work item", Some(50), Some(3))
        .expect("fixture");
}

// ---- report -------------------------------------------------------------

#[test]
fn report_text_names_each_rule_and_the_verdict() {
    let r = run(&snap(n_issues(101), prs(5)), &Rule::RECONCILER);
    let text = r.render_text();
    assert!(text.contains("paiml/x"), "{text}");
    assert!(text.contains("R1") && text.contains("R2"), "{text}");
    assert!(text.contains("VIOLATION"), "{text}");
    let clean = run(&snap(vec![], vec![]), &Rule::RECONCILER).render_text();
    assert!(clean.contains("COMPLIANT"), "{clean}");
}

#[test]
fn report_round_trips_through_json() {
    let r = run(&snap(n_issues(101), vec![]), &[Rule::R1]);
    let back: Report =
        serde_json::from_str(&serde_json::to_string(&r).expect("fixture")).expect("fixture");
    assert_eq!(back, r);
}

#[test]
fn rule_parse_accepts_the_snapshot_rules_only() {
    assert_eq!(Rule::parse("r3").expect("fixture"), Rule::R3);
    assert!(Rule::parse("R5").is_err());
    assert!(Rule::parse("R9").is_err());
}

#[test]
fn snapshot_json_round_trips() {
    let s = snap(vec![issue(1, "a")], vec![pr(2, 1, CheckState::Fail)]);
    let back: RepoSnapshot =
        serde_json::from_str(&serde_json::to_string(&s).expect("fixture")).expect("fixture");
    assert_eq!(back, s);
}
