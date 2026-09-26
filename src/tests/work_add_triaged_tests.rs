//! FLOW-03 (#1440): `pmat work add` refuses an untriaged ticket and links it
//! under its epic before the row is written. Contract:
//! `contracts/pmat-work-add-triaged-v1.yaml`.
//!
//! GitHub is stood in for by [`FakeGithub`], which records every call, so a
//! refusal can be shown to have made no write. The binary-level arms, with a
//! stub `gh` on PATH, are `scripts/work-add-triaged-audit.sh`.

use super::work_add_triage::{
    ensure_linked, gate, parse_issue, resolve_kind, tags_with_kind, EpicLinks, IssueFacts, Triaged,
    WorkAddTriage,
};
use crate::cli::commands::{WorkKind, WorkPriority};
use std::cell::RefCell;
use std::collections::HashMap;

const EPIC: u64 = 10;
const CHILD: u64 = 20;

struct FakeGithub {
    issues: HashMap<u64, IssueFacts>,
    subs: RefCell<Vec<(u64, u64)>>,
    calls: RefCell<Vec<String>>,
    link_fails: bool,
}

fn facts(id: u64, open: bool, labels: &[&str]) -> IssueFacts {
    IssueFacts {
        id,
        open,
        is_pull_request: false,
        labels: labels.iter().map(|l| l.to_string()).collect(),
    }
}

impl FakeGithub {
    fn new() -> Self {
        let mut issues = HashMap::new();
        issues.insert(EPIC, facts(9_010, true, &["epic"]));
        issues.insert(CHILD, facts(9_020, true, &[]));
        Self {
            issues,
            subs: RefCell::new(Vec::new()),
            calls: RefCell::new(Vec::new()),
            link_fails: false,
        }
    }

    fn writes(&self) -> usize {
        self.calls
            .borrow()
            .iter()
            .filter(|c| c.starts_with("link"))
            .count()
    }
}

impl EpicLinks for FakeGithub {
    fn issue(&self, number: u64) -> anyhow::Result<IssueFacts> {
        self.calls.borrow_mut().push(format!("issue {number}"));
        self.issues
            .get(&number)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no issue #{number}"))
    }

    fn sub_issues(&self, epic: u64) -> anyhow::Result<Vec<u64>> {
        self.calls.borrow_mut().push(format!("subs {epic}"));
        let by_id: HashMap<u64, u64> = self.issues.iter().map(|(n, f)| (f.id, *n)).collect();
        Ok(self
            .subs
            .borrow()
            .iter()
            .filter(|(e, _)| *e == epic)
            .filter_map(|(_, id)| by_id.get(id).copied())
            .collect())
    }

    fn link(&self, epic: u64, child_id: u64) -> anyhow::Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("link {epic} {child_id}"));
        if self.link_fails {
            anyhow::bail!("422 Unprocessable Entity");
        }
        self.subs.borrow_mut().push((epic, child_id));
        Ok(())
    }
}

fn full() -> WorkAddTriage {
    WorkAddTriage {
        epic: Some(EPIC),
        priority: Some(WorkPriority::High),
        kind: Some(WorkKind::Code),
    }
}

fn refusal(
    triage: &WorkAddTriage,
    tags: Option<&str>,
    issue: Option<u64>,
    gh: &FakeGithub,
) -> String {
    let err = gate(triage, tags, issue, gh).expect_err("an untriaged add must be refused");
    assert_eq!(gh.writes(), 0, "a refusal must make no GitHub write");
    format!("{err:#}")
}

#[test]
fn work_add_triaged_refuses_without_priority() {
    let gh = FakeGithub::new();
    let t = WorkAddTriage {
        priority: None,
        ..full()
    };
    let msg = refusal(&t, None, Some(CHILD), &gh);
    assert!(msg.contains("--priority P0|P1|P2|P3"), "{msg}");
    assert!(
        gh.calls.borrow().is_empty(),
        "no GitHub read before the local checks"
    );
}

#[test]
fn work_add_triaged_refuses_without_kind() {
    let gh = FakeGithub::new();
    let t = WorkAddTriage {
        kind: None,
        ..full()
    };
    let msg = refusal(&t, Some("area:cli"), Some(CHILD), &gh);
    assert!(msg.contains("--kind"), "{msg}");
}

#[test]
fn work_add_triaged_refuses_without_epic() {
    let gh = FakeGithub::new();
    let t = WorkAddTriage {
        epic: None,
        ..full()
    };
    let msg = refusal(&t, None, Some(CHILD), &gh);
    assert!(msg.contains("--epic <issue#>"), "{msg}");
}

#[test]
fn work_add_triaged_refuses_an_epic_without_an_issue_to_link() {
    let gh = FakeGithub::new();
    let msg = refusal(&full(), None, None, &gh);
    assert!(msg.contains("--github-issue N"), "{msg}");
}

#[test]
fn work_add_triaged_refuses_the_ticket_as_its_own_epic() {
    let gh = FakeGithub::new();
    let t = WorkAddTriage {
        epic: Some(CHILD),
        ..full()
    };
    let msg = refusal(&t, None, Some(CHILD), &gh);
    assert!(msg.contains("its own epic"), "{msg}");
}

#[test]
fn work_add_triaged_refuses_a_closed_epic() {
    let mut gh = FakeGithub::new();
    gh.issues.insert(EPIC, facts(9_010, false, &["epic"]));
    let msg = refusal(&full(), None, Some(CHILD), &gh);
    assert!(msg.contains("is closed"), "{msg}");
}

#[test]
fn work_add_triaged_refuses_an_unlabelled_epic() {
    let mut gh = FakeGithub::new();
    gh.issues.insert(EPIC, facts(9_010, true, &["kind:code"]));
    let msg = refusal(&full(), None, Some(CHILD), &gh);
    assert!(msg.contains("not labelled `epic`"), "{msg}");
}

#[test]
fn work_add_triaged_refuses_a_pull_request_epic() {
    let mut gh = FakeGithub::new();
    let mut pr = facts(9_010, true, &["epic"]);
    pr.is_pull_request = true;
    gh.issues.insert(EPIC, pr);
    let msg = refusal(&full(), None, Some(CHILD), &gh);
    assert!(msg.contains("pull request"), "{msg}");
}

#[test]
fn work_add_triaged_accepts_a_triaged_add_and_reads_only() {
    let gh = FakeGithub::new();
    let t = gate(&full(), None, Some(CHILD), &gh).expect("a triaged add passes the gate");
    assert_eq!(
        t,
        Triaged {
            epic: EPIC,
            child: CHILD,
            priority: WorkPriority::High,
            kind: WorkKind::Code
        }
    );
    assert_eq!(
        gh.writes(),
        0,
        "the gate reads; the link is ensure_linked's job"
    );
}

#[test]
fn work_add_triaged_epic_label_is_case_insensitive() {
    let mut gh = FakeGithub::new();
    gh.issues.insert(EPIC, facts(9_010, true, &["Epic"]));
    gate(&full(), None, Some(CHILD), &gh).expect("`Epic` is the epic label");
}

#[test]
fn work_add_triaged_links_the_child_by_rest_id_once() {
    let gh = FakeGithub::new();
    let t = gate(&full(), None, Some(CHILD), &gh).expect("gate");
    assert!(ensure_linked(&t, &gh).expect("link"), "first add links");
    assert_eq!(
        gh.calls.borrow().last().map(String::as_str),
        Some("link 10 9020"),
        "the sub-issue API takes the child's REST id, not its number"
    );
    assert!(
        !ensure_linked(&t, &gh).expect("relink"),
        "already linked: no second link"
    );
    assert_eq!(gh.writes(), 1);
}

#[test]
fn work_add_triaged_a_failed_link_is_an_error() {
    let mut gh = FakeGithub::new();
    gh.link_fails = true;
    let t = gate(&full(), None, Some(CHILD), &gh).expect("gate");
    let err = ensure_linked(&t, &gh).expect_err("a failed link must not read as linked");
    assert!(
        format!("{err:#}").contains("nothing was written"),
        "{err:#}"
    );
}

#[test]
fn work_add_triaged_kind_comes_from_flag_or_tag_and_must_agree() {
    assert_eq!(
        resolve_kind(None, Some("kind:triage")).expect("tag"),
        WorkKind::Triage
    );
    assert_eq!(
        resolve_kind(Some(WorkKind::Docs), None).expect("flag"),
        WorkKind::Docs
    );
    assert_eq!(
        resolve_kind(Some(WorkKind::Code), Some("a, kind:code")).expect("agree"),
        WorkKind::Code
    );
    assert!(resolve_kind(Some(WorkKind::Code), Some("kind:docs")).is_err());
    assert!(resolve_kind(None, Some("kind:code,kind:docs")).is_err());
    assert!(resolve_kind(None, Some("kind:bogus")).is_err());
    assert!(resolve_kind(None, None).is_err());
}

#[test]
fn work_add_triaged_kind_label_is_added_once() {
    assert_eq!(tags_with_kind(None, WorkKind::Code), "kind:code");
    assert_eq!(
        tags_with_kind(Some("a, b"), WorkKind::Docs),
        "a,b,kind:docs"
    );
    assert_eq!(
        tags_with_kind(Some("kind:docs"), WorkKind::Docs),
        "kind:docs"
    );
}

#[test]
fn work_add_triaged_parses_the_rest_issue_object() {
    let open = parse_issue(r#"{"id":7,"number":3,"state":"open","labels":[{"name":"epic"}]}"#)
        .expect("issue");
    assert_eq!(open, facts(7, true, &["epic"]));
    let pr = parse_issue(r#"{"id":8,"state":"closed","labels":[],"pull_request":{"url":"x"}}"#)
        .expect("pr");
    assert!(pr.is_pull_request && !pr.open);
    let plain = parse_issue(r#"{"id":9,"state":"open","labels":[],"pull_request":null}"#)
        .expect("null pull_request");
    assert!(!plain.is_pull_request);
    assert!(
        parse_issue(r#"{"state":"open"}"#).is_err(),
        "no id is an error, not id 0"
    );
}

#[test]
fn work_add_triaged_priority_accepts_p0_to_p3() {
    use clap::ValueEnum;
    for (spelling, want) in [
        ("P0", WorkPriority::Critical),
        ("p1", WorkPriority::High),
        ("P2", WorkPriority::Medium),
        ("P3", WorkPriority::Low),
        ("high", WorkPriority::High),
    ] {
        assert_eq!(
            WorkPriority::from_str(spelling, false).ok(),
            Some(want),
            "{spelling}"
        );
    }
    assert!(WorkPriority::from_str("P4", false).is_err());
}
