//! Falsification suite for the work sync engine (PMAT-720, goal-mode.md §5).
//!
//! Every case plants one defect in a fixture and asserts the exact finding —
//! class, id, number, reason — so a mutant that misclassifies is named by the
//! test that dies. Nothing here touches the network: a [`GithubSnapshot`] is
//! built by hand, and the clock is a value.

use super::*;
use chrono::TimeZone;

fn t0() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 9, 12, 0, 0)
        .single()
        .expect("a valid instant")
}

fn at(minutes: i64) -> DateTime<Utc> {
    t0() + Duration::minutes(minutes)
}

fn item(id: &str, title: &str, status: ItemStatus, issue: Option<u64>) -> RoadmapItem {
    let mut it = RoadmapItem::new(id.to_string(), title.to_string());
    it.status = status;
    it.github_issue = issue;
    it.updated = t0().to_rfc3339();
    it.created = it.updated.clone();
    it
}

fn open(id: &str, title: &str, issue: Option<u64>) -> RoadmapItem {
    item(id, title, ItemStatus::Planned, issue)
}

fn issue(number: u64, title: &str, state: IssueState) -> IssueSnapshot {
    IssueSnapshot {
        number,
        title: title.to_string(),
        state,
        state_reason: None,
        labels: Vec::new(),
        milestone: None,
        updated_at: t0(),
        sub_issues: None,
    }
}

fn roadmap(items: Vec<RoadmapItem>) -> Roadmap {
    let mut r = Roadmap::new(Some("paiml/pmat".to_string()));
    r.roadmap = items;
    r
}

fn snapshot(issues: Vec<IssueSnapshot>) -> GithubSnapshot {
    GithubSnapshot {
        repo: "paiml/pmat".to_string(),
        taken_at: t0(),
        issues,
        milestones: Vec::new(),
    }
}

fn now() -> Settings {
    Settings::new(t0(), DEFAULT_GRACE_MINUTES)
}

fn later(minutes: i64) -> Settings {
    Settings::new(at(minutes), DEFAULT_GRACE_MINUTES)
}

fn classes(report: &SyncReport) -> Vec<&'static str> {
    report.findings.iter().map(Finding::class).collect()
}

// ── §5.1 set predicate ───────────────────────────────────────────────────────

#[test]
fn a_bijection_with_agreeing_fields_is_coherent() {
    let r = roadmap(vec![
        open("A", "alpha", Some(1)),
        open("B", "beta", Some(2)),
    ]);
    let s = snapshot(vec![
        issue(1, "alpha", IssueState::Open),
        issue(2, "beta", IssueState::Open),
    ]);
    let report = check(&r, &s, &now());
    assert!(report.is_coherent(), "{report:?}");
    assert_eq!(
        (
            report.open_items,
            report.open_issues,
            report.matched,
            report.tolerated
        ),
        (2, 2, 2, 0)
    );
}

#[test]
fn an_open_item_with_no_issue_is_an_orphan_roadmap() {
    let r = roadmap(vec![open("A", "alpha", None)]);
    let report = check(&r, &snapshot(vec![]), &now());
    assert_eq!(
        report.findings,
        vec![Finding::OrphanRoadmap {
            id: "A".to_string(),
            title: "alpha".to_string(),
            github_issue: None,
            reason: OrphanReason::NoIssue,
        }]
    );
}

#[test]
fn an_open_item_naming_a_closed_issue_is_an_orphan_roadmap_with_the_reason() {
    let r = roadmap(vec![open("A", "alpha", Some(7))]);
    let s = snapshot(vec![issue(7, "alpha", IssueState::Closed)]);
    let report = check(&r, &s, &now());
    assert_eq!(
        report.findings,
        vec![Finding::OrphanRoadmap {
            id: "A".to_string(),
            title: "alpha".to_string(),
            github_issue: Some(7),
            reason: OrphanReason::IssueClosed,
        }]
    );
    assert_eq!(report.matched, 0);
}

#[test]
fn an_open_item_naming_an_absent_issue_is_an_orphan_roadmap_with_the_reason() {
    let r = roadmap(vec![open("A", "alpha", Some(77))]);
    let report = check(&r, &snapshot(vec![]), &now());
    assert_eq!(
        report.findings,
        vec![Finding::OrphanRoadmap {
            id: "A".to_string(),
            title: "alpha".to_string(),
            github_issue: Some(77),
            reason: OrphanReason::IssueAbsent,
        }]
    );
}

#[test]
fn a_terminal_item_is_outside_the_universe() {
    let r = roadmap(vec![
        item("C", "done", ItemStatus::Completed, None),
        item("X", "dropped", ItemStatus::Cancelled, Some(7)),
    ]);
    let s = snapshot(vec![issue(7, "dropped", IssueState::Closed)]);
    let report = check(&r, &s, &now());
    assert!(report.is_coherent(), "{report:?}");
    assert_eq!(report.open_items, 0);
}

#[test]
fn blocked_and_review_items_are_open() {
    let r = roadmap(vec![
        item("B", "blocked", ItemStatus::Blocked, None),
        item("R", "review", ItemStatus::Review, None),
    ]);
    let report = check(&r, &snapshot(vec![]), &now());
    assert_eq!(report.open_items, 2);
    assert_eq!(report.count("ORPHAN-ROADMAP"), 2, "{report:?}");
}

#[test]
fn an_open_issue_no_item_names_is_an_orphan_github() {
    let mut nine = issue(9, "nine", IssueState::Open);
    nine.milestone = Some("3.42.0".to_string());
    let report = check(&roadmap(vec![]), &snapshot(vec![nine]), &now());
    assert_eq!(
        report.findings,
        vec![Finding::OrphanGithub {
            number: 9,
            title: "nine".to_string(),
            milestone: Some("3.42.0".to_string()),
        }]
    );
    assert_eq!(report.open_issues, 1);
}

#[test]
fn an_open_item_naming_a_no_roadmap_issue_is_an_orphan_roadmap() {
    // Quorum finding (PMAT-720): the issue is open, so it used to read as a
    // matched pair; but it is outside G, so the item points out of the universe.
    let mut bot = issue(9, "bump deps", IssueState::Open);
    bot.labels.push(NO_ROADMAP_LABEL.to_string());
    let r = roadmap(vec![open("A", "bump deps", Some(9))]);
    let s = snapshot(vec![bot]);
    let report = check(&r, &s, &now());
    assert_eq!(
        report.findings,
        vec![Finding::OrphanRoadmap {
            id: "A".to_string(),
            title: "bump deps".to_string(),
            github_issue: Some(9),
            reason: OrphanReason::IssueExcluded,
        }]
    );
    assert_eq!((report.matched, report.open_issues), (0, 0));
    for d in [
        Direction::YamlToGithub,
        Direction::GithubToYaml,
        Direction::Full,
    ] {
        let actions = plan(&r, &s, &report, d);
        assert_eq!(actions.len(), 1, "{d:?}: {actions:?}");
        let Action::Skip { id, reason } = &actions[0] else {
            unreachable!(
                "{d:?}: an excluded issue is a human's call, never written: {:?}",
                actions[0]
            )
        };
        assert_eq!(id, "A");
        assert!(
            reason.contains("no-roadmap") && reason.contains("#9"),
            "{reason}"
        );
    }
}

#[test]
fn a_no_roadmap_label_takes_an_issue_out_of_the_universe() {
    let mut bot = issue(9, "bump deps", IssueState::Open);
    bot.labels.push(NO_ROADMAP_LABEL.to_string());
    let report = check(&roadmap(vec![]), &snapshot(vec![bot]), &now());
    assert!(report.is_coherent(), "{report:?}");
    assert_eq!(report.open_issues, 0);
}

#[test]
fn a_closed_issue_no_item_names_is_not_an_orphan() {
    let report = check(
        &roadmap(vec![]),
        &snapshot(vec![issue(9, "old", IssueState::Closed)]),
        &now(),
    );
    assert!(report.is_coherent(), "{report:?}");
}

#[test]
fn an_open_issue_named_only_by_a_completed_item_is_an_orphan_github() {
    let r = roadmap(vec![item("C", "five", ItemStatus::Completed, Some(5))]);
    let s = snapshot(vec![issue(5, "five", IssueState::Open)]);
    let report = check(&r, &s, &now());
    assert_eq!(classes(&report), vec!["ORPHAN-GITHUB"], "{report:?}");
}

#[test]
fn two_open_items_naming_one_issue_are_a_collision_and_not_orphans() {
    let r = roadmap(vec![
        open("MACS-001", "one", Some(612)),
        open("MACS-000", "zero", Some(612)),
    ]);
    let s = snapshot(vec![issue(612, "macs", IssueState::Closed)]);
    let report = check(&r, &s, &now());
    assert_eq!(
        report.findings,
        vec![Finding::Collision {
            number: 612,
            ids: vec!["MACS-000".to_string(), "MACS-001".to_string()],
        }]
    );
    assert_eq!(report.count("ORPHAN-ROADMAP"), 0);
}

#[test]
fn a_completed_item_does_not_join_a_collision() {
    let r = roadmap(vec![
        open("A", "alpha", Some(612)),
        item("C", "old", ItemStatus::Completed, Some(612)),
    ]);
    let s = snapshot(vec![issue(612, "alpha", IssueState::Open)]);
    let report = check(&r, &s, &now());
    assert!(report.is_coherent(), "{report:?}");
    assert_eq!(report.matched, 1);
}

// ── §5.2 field predicate ─────────────────────────────────────────────────────

#[test]
fn a_title_disagreement_inside_the_grace_window_is_tolerated() {
    let r = roadmap(vec![open("A", "alpha", Some(1))]);
    let s = snapshot(vec![issue(1, "alpha (renamed)", IssueState::Open)]);
    let report = check(&r, &s, &later(30));
    assert!(report.is_coherent(), "{report:?}");
    assert_eq!(report.tolerated, 1);
}

#[test]
fn a_title_disagreement_past_the_grace_window_is_drift() {
    let r = roadmap(vec![open("A", "alpha", Some(1))]);
    let s = snapshot(vec![issue(1, "alpha (renamed)", IssueState::Open)]);
    let report = check(&r, &s, &later(61));
    assert_eq!(
        report.findings,
        vec![Finding::Drift {
            id: "A".to_string(),
            number: 1,
            field: DriftField::Title,
            roadmap: "alpha".to_string(),
            github: "alpha (renamed)".to_string(),
            age_minutes: 61,
        }]
    );
    assert_eq!(report.tolerated, 0);
}

#[test]
fn the_newer_side_sets_the_clock() {
    let r = roadmap(vec![open("A", "alpha", Some(1))]);
    let mut renamed = issue(1, "alpha (renamed)", IssueState::Open);
    renamed.updated_at = at(50);
    let report = check(&r, &snapshot(vec![renamed]), &later(100));
    assert!(
        report.is_coherent(),
        "age is 50, inside the window: {report:?}"
    );
    assert_eq!(report.tolerated, 1);
}

#[test]
fn an_unparseable_item_timestamp_never_hides_drift() {
    let mut a = open("A", "alpha", Some(1));
    a.updated = "yesterday".to_string();
    let s = snapshot(vec![issue(1, "alpha (renamed)", IssueState::Open)]);
    let report = check(&roadmap(vec![a]), &s, &now());
    assert_eq!(classes(&report), vec!["DRIFT"], "{report:?}");
}

#[test]
fn a_stale_release_projection_is_drift() {
    let mut a = open("A", "alpha", Some(1));
    a.release = Some("3.41.0".to_string());
    let mut one = issue(1, "alpha", IssueState::Open);
    one.milestone = Some("3.42.0".to_string());
    let report = check(&roadmap(vec![a]), &snapshot(vec![one]), &later(61));
    assert_eq!(
        report.findings,
        vec![Finding::Drift {
            id: "A".to_string(),
            number: 1,
            field: DriftField::Release,
            roadmap: "3.41.0".to_string(),
            github: "3.42.0".to_string(),
            age_minutes: 61,
        }]
    );
}

#[test]
fn a_missing_release_projection_is_drift_when_the_issue_has_a_milestone() {
    let mut one = issue(1, "alpha", IssueState::Open);
    one.milestone = Some("3.42.0".to_string());
    let report = check(
        &roadmap(vec![open("A", "alpha", Some(1))]),
        &snapshot(vec![one]),
        &later(61),
    );
    assert_eq!(
        report.findings,
        vec![Finding::Drift {
            id: "A".to_string(),
            number: 1,
            field: DriftField::Release,
            roadmap: String::new(),
            github: "3.42.0".to_string(),
            age_minutes: 61,
        }]
    );
}

#[test]
fn no_milestone_and_no_release_agree() {
    let report = check(
        &roadmap(vec![open("A", "alpha", Some(1))]),
        &snapshot(vec![issue(1, "alpha", IssueState::Open)]),
        &later(500),
    );
    assert!(report.is_coherent(), "{report:?}");
}

#[test]
fn findings_are_ordered_by_class_then_key() {
    let mut z = open("Z", "zed", Some(3));
    z.release = Some("1.0.0".to_string());
    let r = roadmap(vec![
        z,
        open("B", "no issue b", None),
        open("A", "no issue a", None),
        open("M1", "m", Some(612)),
        open("M0", "m", Some(612)),
    ]);
    let mut three = issue(3, "zed", IssueState::Open);
    three.milestone = Some("2.0.0".to_string());
    let s = snapshot(vec![
        three,
        issue(612, "macs", IssueState::Closed),
        issue(20, "orphan twenty", IssueState::Open),
        issue(10, "orphan ten", IssueState::Open),
    ]);
    let report = check(&r, &s, &later(61));
    assert_eq!(
        classes(&report),
        vec![
            "COLLISION",
            "ORPHAN-ROADMAP",
            "ORPHAN-ROADMAP",
            "ORPHAN-GITHUB",
            "ORPHAN-GITHUB",
            "DRIFT"
        ],
        "{report:?}"
    );
    let ids: Vec<String> = report
        .findings
        .iter()
        .filter_map(|f| match f {
            Finding::OrphanRoadmap { id, .. } => Some(id.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(ids, vec!["A", "B"]);
    let numbers: Vec<u64> = report
        .findings
        .iter()
        .filter_map(|f| match f {
            Finding::OrphanGithub { number, .. } => Some(*number),
            _ => None,
        })
        .collect();
    assert_eq!(numbers, vec![10, 20]);
}

#[test]
fn a_snapshot_round_trips_through_json() {
    let mut one = issue(1, "alpha", IssueState::Closed);
    one.state_reason = Some(CloseReason::NotPlanned);
    one.labels.push("bug".to_string());
    one.milestone = Some("3.42.0".to_string());
    let mut s = snapshot(vec![one]);
    s.milestones.push(MilestoneSnapshot {
        title: "3.42.0".to_string(),
        state: IssueState::Open,
    });
    let text = s.to_json().expect("serializes");
    let back = GithubSnapshot::from_json(&text).expect("parses");
    assert_eq!(back, s);
    assert!(
        GithubSnapshot::from_json("{not json").is_err(),
        "garbage must be refused, not read as an empty snapshot"
    );
}

// ── plan ─────────────────────────────────────────────────────────────────────

fn plan_for(r: &Roadmap, s: &GithubSnapshot, d: Direction) -> Vec<Action> {
    let report = check(r, s, &later(61));
    plan(r, s, &report, d)
}

#[test]
fn yaml_to_github_opens_an_issue_for_an_item_with_none() {
    let r = roadmap(vec![open("A", "alpha", None)]);
    let actions = plan_for(&r, &snapshot(vec![]), Direction::YamlToGithub);
    assert_eq!(
        actions,
        vec![Action::CreateIssue {
            id: "A".to_string(),
            title: "alpha".to_string(),
        }]
    );
}

#[test]
fn yaml_to_github_leaves_a_closed_issue_to_the_other_direction() {
    let r = roadmap(vec![open("A", "alpha", Some(7))]);
    let s = snapshot(vec![issue(7, "alpha", IssueState::Closed)]);
    let actions = plan_for(&r, &s, Direction::YamlToGithub);
    assert_eq!(actions.len(), 1, "{actions:?}");
    let Action::Skip { id, reason } = &actions[0] else {
        unreachable!("expected a Skip, got {:?}", actions[0])
    };
    assert_eq!(id, "A");
    assert!(reason.contains("github-to-yaml"), "{reason}");
}

#[test]
fn a_collided_item_is_skipped_by_every_direction() {
    let r = roadmap(vec![
        open("MACS-000", "zero", Some(612)),
        open("MACS-001", "one", Some(612)),
    ]);
    let s = snapshot(vec![issue(612, "macs", IssueState::Closed)]);
    for d in [
        Direction::YamlToGithub,
        Direction::GithubToYaml,
        Direction::Full,
    ] {
        let actions = plan_for(&r, &s, d);
        let skips: Vec<&Action> = actions
            .iter()
            .filter(|a| matches!(a, Action::Skip { .. }))
            .collect();
        assert_eq!(skips.len(), 2, "{d:?}: {actions:?}");
        for a in &actions {
            let Action::Skip { reason, .. } = a else {
                unreachable!("{d:?} must not write for a collided item: {:?}", a)
            };
            assert!(
                reason.contains("COLLISION") && reason.contains("#612"),
                "{reason}"
            );
        }
    }
}

#[test]
fn github_to_yaml_creates_an_item_for_an_orphan_issue_with_its_milestone() {
    let mut nine = issue(9, "nine", IssueState::Open);
    nine.milestone = Some("3.42.0".to_string());
    let actions = plan_for(
        &roadmap(vec![]),
        &snapshot(vec![nine]),
        Direction::GithubToYaml,
    );
    assert_eq!(
        actions,
        vec![Action::CreateItem {
            number: 9,
            title: "nine".to_string(),
            release: Some("3.42.0".to_string()),
        }]
    );
}

#[test]
fn github_to_yaml_closes_the_item_of_a_closed_issue_by_reason() {
    let mut done = issue(7, "alpha", IssueState::Closed);
    done.state_reason = Some(CloseReason::Completed);
    let mut dropped = issue(8, "beta", IssueState::Closed);
    dropped.state_reason = Some(CloseReason::NotPlanned);
    let r = roadmap(vec![
        open("A", "alpha", Some(7)),
        open("B", "beta", Some(8)),
    ]);
    let actions = plan_for(&r, &snapshot(vec![done, dropped]), Direction::GithubToYaml);
    assert_eq!(
        actions,
        vec![
            Action::CloseItem {
                id: "A".to_string(),
                number: 7,
                status: ItemStatus::Completed,
            },
            Action::CloseItem {
                id: "B".to_string(),
                number: 8,
                status: ItemStatus::Cancelled,
            },
        ]
    );
}

#[test]
fn github_to_yaml_projects_the_milestone_into_release() {
    let mut a = open("A", "alpha", Some(1));
    a.release = Some("3.41.0".to_string());
    let mut one = issue(1, "alpha", IssueState::Open);
    one.milestone = Some("3.42.0".to_string());
    let actions = plan_for(
        &roadmap(vec![a]),
        &snapshot(vec![one]),
        Direction::GithubToYaml,
    );
    assert_eq!(
        actions,
        vec![Action::SetRelease {
            id: "A".to_string(),
            number: 1,
            release: Some("3.42.0".to_string()),
        }]
    );
}

#[test]
fn a_title_drift_is_reported_and_never_written() {
    let r = roadmap(vec![open("A", "alpha", Some(1))]);
    let s = snapshot(vec![issue(1, "alpha (renamed)", IssueState::Open)]);
    for d in [Direction::YamlToGithub, Direction::GithubToYaml] {
        let actions = plan_for(&r, &s, d);
        assert_eq!(actions.len(), 1, "{d:?}: {actions:?}");
        let Action::Skip { reason, .. } = &actions[0] else {
            unreachable!(
                "{d:?}: title has no assigned authority (§5.3): {:?}",
                actions[0]
            )
        };
        assert!(reason.contains("title"), "{reason}");
    }
}

#[test]
fn an_absent_issue_is_a_human_decision() {
    let r = roadmap(vec![open("A", "alpha", Some(77))]);
    for d in [Direction::YamlToGithub, Direction::GithubToYaml] {
        let actions = plan_for(&r, &snapshot(vec![]), d);
        assert_eq!(actions.len(), 1, "{d:?}: {actions:?}");
        let Action::Skip { reason, .. } = &actions[0] else {
            unreachable!("{d:?}: an absent issue is not fixable: {:?}", actions[0])
        };
        assert!(reason.contains("#77"), "{reason}");
    }
}

#[test]
fn full_is_the_union_of_both_directions() {
    // Computed, not assumed (quorum finding): Full's writes are exactly the
    // writes of the two directions put together, and Full skips nothing that
    // either direction would have written.
    let mut stale = open("Z", "zed", Some(3));
    stale.release = Some("1.0.0".to_string());
    let r = roadmap(vec![open("A", "alpha", None), stale]);
    let mut three = issue(3, "zed", IssueState::Open);
    three.milestone = Some("2.0.0".to_string());
    let s = snapshot(vec![three, issue(9, "nine", IssueState::Open)]);
    let writes = |d: Direction| -> Vec<String> {
        let mut w: Vec<String> = plan_for(&r, &s, d)
            .into_iter()
            .filter(|a| !matches!(a, Action::Skip { .. }))
            .map(|a| format!("{a:?}"))
            .collect();
        w.sort();
        w
    };
    let mut union = writes(Direction::YamlToGithub);
    union.extend(writes(Direction::GithubToYaml));
    union.sort();
    let full = writes(Direction::Full);
    assert_eq!(full, union, "Full must be the union of the two directions");
    assert_eq!(
        full.len(),
        3,
        "CreateIssue A, CreateItem 9, SetRelease Z: {full:?}"
    );
    assert!(
        plan_for(&r, &s, Direction::Full)
            .iter()
            .all(|a| !matches!(a, Action::Skip { .. })),
        "nothing here is a collision, a title drift or an absent issue, so Full skips nothing"
    );
}

#[test]
fn yaml_to_github_leaves_a_stale_release_to_the_other_direction() {
    let mut a = open("A", "alpha", Some(1));
    a.release = Some("3.41.0".to_string());
    let mut one = issue(1, "alpha", IssueState::Open);
    one.milestone = Some("3.42.0".to_string());
    let actions = plan_for(
        &roadmap(vec![a]),
        &snapshot(vec![one]),
        Direction::YamlToGithub,
    );
    assert_eq!(actions.len(), 1, "{actions:?}");
    let Action::Skip { id, reason } = &actions[0] else {
        unreachable!(
            "release is projected from the milestone, never pushed to it (§4.1): {:?}",
            actions[0]
        )
    };
    assert_eq!(id, "A");
    assert!(reason.contains("github-to-yaml"), "{reason}");
}

#[test]
fn a_coherent_report_plans_nothing() {
    let r = roadmap(vec![open("A", "alpha", Some(1))]);
    let s = snapshot(vec![issue(1, "alpha", IssueState::Open)]);
    for d in [
        Direction::YamlToGithub,
        Direction::GithubToYaml,
        Direction::Full,
    ] {
        assert!(plan_for(&r, &s, d).is_empty(), "{d:?}");
    }
}

// ── apply ────────────────────────────────────────────────────────────────────

#[test]
fn apply_creates_the_gh_item_and_links_it() {
    let mut r = roadmap(vec![]);
    let n = apply_to_roadmap(
        &mut r,
        &[Action::CreateItem {
            number: 9,
            title: "nine".to_string(),
            release: Some("3.42.0".to_string()),
        }],
        at(5),
    );
    assert_eq!(n, 1);
    let it = r.find_item("GH-9").expect("the item was created");
    assert_eq!(it.github_issue, Some(9));
    assert_eq!(it.release.as_deref(), Some("3.42.0"));
    assert_eq!(it.status, ItemStatus::Planned);
    assert_eq!(it.updated, at(5).to_rfc3339());
}

#[test]
fn apply_closes_and_projects() {
    let mut r = roadmap(vec![
        open("A", "alpha", Some(7)),
        open("B", "beta", Some(1)),
    ]);
    let n = apply_to_roadmap(
        &mut r,
        &[
            Action::CloseItem {
                id: "A".to_string(),
                number: 7,
                status: ItemStatus::Cancelled,
            },
            Action::SetRelease {
                id: "B".to_string(),
                number: 1,
                release: Some("3.42.0".to_string()),
            },
        ],
        at(5),
    );
    assert_eq!(n, 2);
    let a = r.find_item("A").expect("A");
    assert_eq!(a.status, ItemStatus::Cancelled);
    assert_eq!(a.updated, at(5).to_rfc3339());
    let b = r.find_item("B").expect("B");
    assert_eq!(b.release.as_deref(), Some("3.42.0"));
    assert_eq!(b.updated, at(5).to_rfc3339());
}

#[test]
fn apply_links_a_created_issue() {
    let mut r = roadmap(vec![open("A", "alpha", None)]);
    let n = apply_to_roadmap(
        &mut r,
        &[Action::LinkIssue {
            id: "A".to_string(),
            number: 42,
        }],
        at(5),
    );
    assert_eq!(n, 1);
    assert_eq!(r.find_item("A").expect("A").github_issue, Some(42));
}

#[test]
fn apply_ignores_create_issue_and_skip() {
    let mut r = roadmap(vec![open("A", "alpha", None)]);
    let before = r.clone();
    let n = apply_to_roadmap(
        &mut r,
        &[
            Action::CreateIssue {
                id: "A".to_string(),
                title: "alpha".to_string(),
            },
            Action::Skip {
                id: "A".to_string(),
                reason: "x".to_string(),
            },
        ],
        at(5),
    );
    assert_eq!(n, 0);
    assert_eq!(r, before);
}

#[test]
fn apply_on_an_unknown_id_changes_nothing() {
    let mut r = roadmap(vec![open("A", "alpha", Some(1))]);
    let before = r.clone();
    let n = apply_to_roadmap(
        &mut r,
        &[Action::SetRelease {
            id: "NOPE".to_string(),
            number: 1,
            release: Some("1.0.0".to_string()),
        }],
        at(5),
    );
    assert_eq!(n, 0);
    assert_eq!(r, before);
}

// ── §5.2 the window covers the ORPHAN legs too (PMAT-1309) ───────────────────
//
// An issue opened seconds ago, and an item added seconds ago, are not evidence
// that the roadmap and GitHub disagree — they are evidence that the sync that
// would have joined them has not run yet. CB-2115 turned master red at
// 5af9a0f0d for exactly that, with no commit to blame. Inside the window each
// is TOLERATED (the rule passes and counts it); past it each is a finding.

#[test]
fn a_freshly_opened_orphan_issue_is_tolerated() {
    // #5 was opened at t0 and the clock is 30 minutes later: inside the default
    // 60-minute window, so the bijection is not yet expected to hold.
    let report = check(
        &roadmap(vec![]),
        &snapshot(vec![issue(5, "opened by hand", IssueState::Open)]),
        &later(30),
    );
    assert!(
        report.is_coherent(),
        "an issue opened 30 minutes ago is inside the window: {report:?}"
    );
    assert_eq!(report.tolerated, 1, "{report:?}");
    assert_eq!(
        report.open_issues, 1,
        "it is still counted in G: {report:?}"
    );
}

#[test]
fn an_orphan_issue_past_the_window_is_a_finding() {
    // The control for the test above: the same issue, one minute past the
    // window, is the ORPHAN-GITHUB it always was.
    let report = check(
        &roadmap(vec![]),
        &snapshot(vec![issue(5, "opened by hand", IssueState::Open)]),
        &later(61),
    );
    assert_eq!(
        report.findings,
        vec![Finding::OrphanGithub {
            number: 5,
            title: "opened by hand".to_string(),
            milestone: None,
        }]
    );
    assert_eq!(report.tolerated, 0, "{report:?}");
}

#[test]
fn a_roadmap_item_with_no_issue_is_tolerated_inside_the_window() {
    // The mirror case: the item was written 30 minutes ago and its issue has
    // not been minted yet.
    let report = check(
        &roadmap(vec![open("A", "alpha", None)]),
        &snapshot(vec![]),
        &later(30),
    );
    assert!(report.is_coherent(), "{report:?}");
    assert_eq!(report.tolerated, 1, "{report:?}");
}

#[test]
fn a_roadmap_item_with_no_issue_past_the_window_is_a_finding() {
    let report = check(
        &roadmap(vec![open("A", "alpha", None)]),
        &snapshot(vec![]),
        &later(61),
    );
    assert_eq!(
        report.findings,
        vec![Finding::OrphanRoadmap {
            id: "A".to_string(),
            title: "alpha".to_string(),
            github_issue: None,
            reason: OrphanReason::NoIssue,
        }]
    );
    assert_eq!(report.tolerated, 0, "{report:?}");
}

#[test]
fn a_closed_issue_is_a_finding_however_recent() {
    // The window is a freshness tolerance, and only NoIssue and ORPHAN-GITHUB
    // are freshness problems. A closed issue does not become un-closed by being
    // recent, an absent one does not appear, and a no-roadmap label does not
    // fall off — all three stay findings at age zero. This test dies if someone
    // later puts them behind the window.
    let mut excluded = issue(9, "bump deps", IssueState::Open);
    excluded.labels.push(NO_ROADMAP_LABEL.to_string());
    let r = roadmap(vec![
        open("A", "alpha", Some(7)),
        open("B", "beta", Some(77)),
        open("C", "gamma", Some(9)),
    ]);
    let s = snapshot(vec![issue(7, "alpha", IssueState::Closed), excluded]);
    let report = check(&r, &s, &now());
    assert_eq!(
        classes(&report),
        vec!["ORPHAN-ROADMAP", "ORPHAN-ROADMAP", "ORPHAN-ROADMAP"],
        "{report:?}"
    );
    let reasons: Vec<OrphanReason> = report
        .findings
        .iter()
        .filter_map(|f| match f {
            Finding::OrphanRoadmap { reason, .. } => Some(*reason),
            _ => None,
        })
        .collect();
    assert_eq!(
        reasons,
        vec![
            OrphanReason::IssueClosed,
            OrphanReason::IssueAbsent,
            OrphanReason::IssueExcluded
        ],
        "{report:?}"
    );
    assert_eq!(report.tolerated, 0, "none of these is a freshness problem");
}

#[test]
fn the_tolerated_count_includes_a_tolerated_orphan() {
    // A fresh orphan on each side plus a field disagreement: the count the rule
    // prints is every toleration, not only the field ones. A PASS that swallowed
    // an orphan without counting it is the failure this rule exists to prevent.
    let r = roadmap(vec![open("A", "alpha", None), open("B", "beta", Some(1))]);
    let s = snapshot(vec![
        issue(1, "beta (renamed)", IssueState::Open),
        issue(5, "opened by hand", IssueState::Open),
    ]);
    let report = check(&r, &s, &later(30));
    assert!(report.is_coherent(), "{report:?}");
    assert_eq!(
        report.tolerated, 3,
        "one item with no issue, one orphan issue, one title drift: {report:?}"
    );
}

#[test]
fn a_tolerated_orphan_is_still_the_fixers_work() {
    // The window moves the GATE's verdict, never the fixer's work list: `pmat
    // work sync` must still mint the issue for an item added a minute ago, or
    // no new item could ever be minted until it had aged an hour.
    let r = roadmap(vec![open("A", "alpha", None)]);
    let s = snapshot(vec![issue(5, "opened by hand", IssueState::Open)]);
    let report = check(&r, &s, &later(30));
    assert!(
        report.is_coherent(),
        "tolerated, so the gate passes: {report:?}"
    );
    let actions = plan(&r, &s, &report, Direction::Full);
    assert_eq!(
        actions,
        vec![
            Action::CreateIssue {
                id: "A".to_string(),
                title: "alpha".to_string(),
            },
            Action::CreateItem {
                number: 5,
                title: "opened by hand".to_string(),
                release: None,
            },
        ],
        "a tolerated orphan is still an orphan the fixer closes"
    );
}
