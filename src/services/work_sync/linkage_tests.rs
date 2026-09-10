//! PMAT-724: the pure predicates behind CB-2112 and CB-2114. Each test names
//! the mutant it dies under.
use super::linkage::{numeric_tail, release_binding, ticket_linkage};
use super::linkage::{LinkFinding, ReleaseFinding};
use super::*;

fn item(id: &str, status: ItemStatus, issue: Option<u64>, release: Option<&str>) -> RoadmapItem {
    let mut it = RoadmapItem::new(id.to_string(), "t".to_string());
    it.status = status;
    it.github_issue = issue;
    it.release = release.map(str::to_string);
    it
}
fn roadmap(items: Vec<RoadmapItem>) -> Roadmap {
    let mut r = Roadmap::new(Some("paiml/fixture".to_string()));
    r.roadmap = items;
    r
}
fn issue(
    number: u64,
    state: IssueState,
    labels: &[&str],
    milestone: Option<&str>,
) -> IssueSnapshot {
    IssueSnapshot {
        number,
        title: "t".into(),
        state,
        state_reason: None,
        labels: labels.iter().map(|s| s.to_string()).collect(),
        milestone: milestone.map(str::to_string),
        updated_at: Utc::now(),
    }
}
fn snapshot(issues: Vec<IssueSnapshot>, milestones: &[&str]) -> GithubSnapshot {
    GithubSnapshot {
        repo: "paiml/fixture".into(),
        taken_at: Utc::now(),
        issues,
        milestones: milestones
            .iter()
            .map(|t| MilestoneSnapshot {
                title: t.to_string(),
                state: IssueState::Open,
            })
            .collect(),
    }
}

/// Mutant: `rsplit('-')` — `PERF-001` → "001" is fine, but `EPIC` → Some(0)
/// or `PMAT-7a` → a parse of "7a".
#[test]
fn the_numeric_tail_is_the_trailing_digit_run_or_nothing() {
    assert_eq!(numeric_tail("PMAT-724"), Some(724));
    assert_eq!(numeric_tail("GH-75"), Some(75));
    assert_eq!(numeric_tail("PERF-001"), Some(1));
    assert_eq!(numeric_tail("EPIC"), None);
    assert_eq!(numeric_tail("PMAT-7a"), None);
    assert_eq!(numeric_tail(""), None);
}

/// Mutant: findings emitted in HashMap order — two runs, two messages.
#[test]
fn linkage_findings_are_ordered_by_item_id() {
    let r = roadmap(vec![
        item("PMAT-003", ItemStatus::Planned, None, None),
        item("PMAT-001", ItemStatus::Planned, Some(9), None),
        item("PMAT-002", ItemStatus::Completed, None, None),
    ]);
    let s = snapshot(vec![issue(9, IssueState::Open, &[], None)], &[]);
    let report = ticket_linkage(&r, &s);
    assert_eq!(report.open_items, 2);
    assert_eq!(report.linked, 0);
    let ids: Vec<&str> = report.findings.iter().map(LinkFinding::id).collect();
    assert_eq!(ids, vec!["PMAT-001", "PMAT-003"]);
    assert_eq!(report.findings[0].class(), "TAIL-MISMATCH");
    assert_eq!(report.findings[1].class(), "NO-ISSUE");
}

/// Mutant: the milestone clause and the membership clause collapsed into
/// one — a NOT-ON-MILESTONE reported as NO-MILESTONE.
#[test]
fn release_findings_distinguish_a_missing_milestone_from_a_missing_membership() {
    let r = roadmap(vec![
        item("PMAT-001", ItemStatus::Planned, Some(1), Some("3.41.0")),
        item("PMAT-002", ItemStatus::Planned, Some(2), Some("9.9.9")),
        item("PMAT-003", ItemStatus::InProgress, Some(3), Some("3.41.0")),
    ]);
    let s = snapshot(
        vec![
            issue(1, IssueState::Open, &[], Some("3.42.0")),
            issue(2, IssueState::Open, &[], Some("9.9.9")),
            issue(3, IssueState::Open, &[], Some("3.41.0")),
        ],
        &["3.41.0", "3.42.0"],
    );
    let report = release_binding(&r, &s);
    assert_eq!(report.open_items, 3);
    assert_eq!(report.bound, 1);
    let classes: Vec<(&str, &str)> = report
        .findings
        .iter()
        .map(|f| (f.id(), f.class()))
        .collect();
    assert_eq!(
        classes,
        vec![
            ("PMAT-001", "NOT-ON-MILESTONE"),
            ("PMAT-002", "NO-MILESTONE")
        ]
    );
    assert!(
        matches!(&report.findings[0], ReleaseFinding::NotOnMilestone { actual: Some(m), .. } if m == "3.42.0")
    );
}

/// Mutant: `milestone.state` ignored — an open item bound to a shipped
/// release (quorum on PMAT-724, all three lanes).
#[test]
fn a_closed_milestone_is_a_finding_of_its_own_class() {
    let r = roadmap(vec![item(
        "PMAT-001",
        ItemStatus::Planned,
        Some(1),
        Some("3.40.0"),
    )]);
    let mut s = snapshot(
        vec![issue(1, IssueState::Open, &[], Some("3.40.0"))],
        &["3.40.0"],
    );
    s.milestones[0].state = IssueState::Closed;
    let report = release_binding(&r, &s);
    assert_eq!(report.bound, 0);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].class(), "MILESTONE-CLOSED");
    assert!(report.findings[0].render().contains("3.40.0"));
}
