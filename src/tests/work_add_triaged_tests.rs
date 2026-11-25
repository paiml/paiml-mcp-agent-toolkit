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
        body: String::new(),
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
        msg.contains("no P0|P1|P2|P3 label"),
        "the refusal names the label fallback: {msg}"
    );
}

/// FLOW-03 / #1414: with no `--priority`, the issue's P-label carries it.
/// Falsifier: a P0-labelled issue with a `done_when` section yields the
/// contract's P0 level (`critical`) and a non-empty acceptance list.
#[test]
fn work_add_triaged_priority_is_carried_from_the_issue_label() {
    let body = "Measured on the host.\n\n## done_when\n\n- the receipt is committed\n- decide refuses when it is missing\n";
    let mut gh = FakeGithub::new();
    gh.issues.insert(
        CHILD,
        IssueFacts {
            body: body.to_string(),
            ..facts(9_020, true, &["P0", "triaged"])
        },
    );
    let t = WorkAddTriage {
        priority: None,
        ..full()
    };
    let triaged =
        gate(&t, None, Some(CHILD), &gh).expect("a P0-labelled issue carries its priority");
    assert_eq!(triaged.priority, WorkPriority::Critical);
    assert!(!derive_from_body(body).acceptance.is_empty());
}

/// An explicit `--priority` wins, and the issue's label is not read for it.
#[test]
fn work_add_triaged_explicit_priority_is_not_overridden_by_the_label() {
    let mut gh = FakeGithub::new();
    gh.issues.insert(CHILD, facts(9_020, true, &["P0"]));
    let t = gate(&full(), None, Some(CHILD), &gh).expect("gate");
    assert_eq!(t.priority, WorkPriority::High);
    assert!(
        !gh.calls.borrow().iter().any(|c| c == "issue 20"),
        "no label read when --priority is given"
    );
}

/// Two different P-labels are refused rather than guessed at.
#[test]
fn work_add_triaged_conflicting_priority_labels_are_refused() {
    let mut gh = FakeGithub::new();
    gh.issues.insert(CHILD, facts(9_020, true, &["P0", "P2"]));
    let t = WorkAddTriage {
        priority: None,
        ..full()
    };
    let err = gate(&t, None, Some(CHILD), &gh).expect_err("two P-labels must be refused");
    assert!(
        format!("{err:#}").contains("two different P-labels"),
        "{err:#}"
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

// ── #1414: spec and acceptance criteria come from the issue body ───────────

use super::work_add_triage::{derive_from_body, Derived};

const P0_BODY: &str = "Measured.\n\n## done_when\n\n- the receipt is committed\n- `decide` refuses `apr cell unreceipted`\n\n## Notes\nunrelated\n\nSpec row: ARB-APR-001 row 3.\n";

#[test]
fn work_add_derives_acceptance_and_spec_from_the_body() {
    let d = derive_from_body(P0_BODY);
    assert_eq!(d.spec.as_deref(), Some("ARB-APR-001 row 3"));
    assert_eq!(
        d.acceptance,
        vec![
            "the receipt is committed".to_string(),
            "`decide` refuses `apr cell unreceipted`".to_string()
        ]
    );
}

#[test]
fn work_add_derives_an_inline_done_when_key() {
    let d = derive_from_body("done_when: the row is green\n");
    assert_eq!(d.acceptance, vec!["the row is green".to_string()]);
    assert_eq!(d.spec, None);
}

#[test]
fn work_add_leaves_fields_empty_when_the_body_states_none() {
    assert_eq!(
        derive_from_body("just prose, no section"),
        Derived::default()
    );
}

#[tokio::test]
async fn work_add_writes_the_derived_fields_into_the_row() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("docs/roadmaps/roadmap.yaml");
    std::fs::create_dir_all(path.parent().ok_or_else(|| anyhow::anyhow!("no parent"))?)?;
    std::fs::write(
        &path,
        "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: null\nroadmap: []\n",
    )?;
    super::add_ticket_row_derived(
        "t".to_string(),
        None,
        WorkPriority::Critical,
        None,
        Some(dir.path().to_path_buf()),
        false,
        None,
        None,
        Some(CHILD),
        false,
        None,
        derive_from_body(P0_BODY),
    )
    .await?;
    let rm: crate::models::roadmap::Roadmap =
        serde_yaml_ng::from_str(&std::fs::read_to_string(&path)?)?;
    let item = rm
        .roadmap
        .first()
        .ok_or_else(|| anyhow::anyhow!("no row"))?;
    assert_eq!(
        item.spec,
        Some(std::path::PathBuf::from("ARB-APR-001 row 3"))
    );
    assert_eq!(item.acceptance_criteria.len(), 2);
    assert_eq!(item.acceptance_criteria[0], "the receipt is committed");
    Ok(())
}

/// Write one row from `body` and `description`; return its criteria.
async fn criteria_written(body: &str, description: Option<&str>) -> anyhow::Result<Vec<String>> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("docs/roadmaps/roadmap.yaml");
    std::fs::create_dir_all(path.parent().ok_or_else(|| anyhow::anyhow!("no parent"))?)?;
    std::fs::write(
        &path,
        "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: null\nroadmap: []\n",
    )?;
    let d = description.map(str::to_string);
    let p = Some(dir.path().to_path_buf());
    let derived = derive_from_body(body);
    super::add_ticket_row_derived(
        "t".into(),
        d,
        WorkPriority::Critical,
        None,
        p,
        false,
        None,
        None,
        Some(CHILD),
        false,
        None,
        derived,
    )
    .await?;
    let rm: crate::models::roadmap::Roadmap =
        serde_yaml_ng::from_str(&std::fs::read_to_string(&path)?)?;
    Ok(rm
        .roadmap
        .into_iter()
        .next()
        .map(|i| i.acceptance_criteria)
        .unwrap_or_default())
}

#[tokio::test]
async fn work_add_prose_starting_with_acceptance_keeps_the_description() -> anyhow::Result<()> {
    for body in [
        "Acceptance tests run nightly.\nThe owner is the infra team.\nSee the runbook.\n",
        "Acceptance criteria are tracked in #1414.\n\nNo list here.\n",
        "done_whenever it ships.\nLater.\n",
    ] {
        assert_eq!(
            derive_from_body(body).acceptance,
            Vec::<String>::new(),
            "{body}"
        );
        assert_eq!(
            criteria_written(body, Some("d")).await?,
            vec!["d".to_string()],
            "{body}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn work_add_description_is_the_fallback_and_is_dropped_when_derived() -> anyhow::Result<()> {
    assert_eq!(
        criteria_written("prose", Some("d")).await?,
        vec!["d".to_string()]
    );
    assert!(criteria_written("prose", None).await?.is_empty());
    let got = criteria_written(P0_BODY, Some("d")).await?;
    assert_eq!(got.len(), 2);
    assert!(!got.contains(&"d".to_string()));
    Ok(())
}

#[test]
fn work_add_opens_a_section_only_on_a_heading_or_a_key() {
    assert_eq!(
        derive_from_body("Acceptance: a\n").acceptance,
        vec!["a".to_string()]
    );
    assert_eq!(
        derive_from_body("acceptance_criteria:\n- b\n").acceptance,
        vec!["b".to_string()]
    );
    assert_eq!(
        derive_from_body("## Acceptance criteria\n- c\n").acceptance,
        vec!["c".to_string()]
    );
    assert!(derive_from_body("Acceptance tests run.\nx\n")
        .acceptance
        .is_empty());
}
