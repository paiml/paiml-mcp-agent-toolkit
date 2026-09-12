//! PMAT-728 (goal-mode.md §11 step 6): the pure half of CB-2110. Every test
//! names the mutant it dies under, so a survivor is a test to delete, not keep.

use super::*;
use crate::services::work_sync::{GithubSnapshot, IssueSnapshot, IssueState};
use chrono::Utc;

fn fm(epic: &str, status: &str) -> String {
    format!("---\nepic: {epic}\nstatus: {status}\nvendors: []\n---\n\n# A title\n\nBody.\n")
}

fn spec(path: &str, text: &str) -> SpecInput {
    SpecInput {
        path: format!("docs/specifications/{path}"),
        text: text.to_string(),
    }
}

fn issue(
    number: u64,
    state: IssueState,
    labels: &[&str],
    sub_issues: Option<u64>,
) -> IssueSnapshot {
    IssueSnapshot {
        number,
        title: format!("issue {number}"),
        state,
        state_reason: None,
        labels: labels.iter().map(|l| (*l).to_string()).collect(),
        milestone: None,
        created_at: None,
        updated_at: Utc::now(),
        sub_issues,
    }
}

fn snapshot(issues: Vec<IssueSnapshot>) -> GithubSnapshot {
    GithubSnapshot {
        repo: "paiml/fixture".to_string(),
        taken_at: Utc::now(),
        issues,
        milestones: Vec::new(),
    }
}

fn parsed(text: &str) -> SpecFrontMatter {
    parse_front_matter(text)
        .expect("the fixture front-matter must parse — the failing test names it")
}

/// Mutant: `epic: null` read as a number, a missing `epic:` key read as an
/// error, quotes or a block list refused, an unknown key refused.
#[test]
fn the_front_matter_parses_its_three_keys_and_ignores_the_rest() {
    let a = parsed(&fm("1234", "active"));
    assert_eq!(
        a,
        SpecFrontMatter {
            epic: Some(1234),
            status: SpecStatus::Active,
            vendors: vec![]
        }
    );

    let b = parsed("---\nepic: null\nstatus: historical\nvendors: [cuda, rocm]\n---\n# T\n");
    assert_eq!(b.epic, None);
    assert_eq!(b.status, SpecStatus::Historical);
    assert_eq!(b.vendors, vec!["cuda", "rocm"]);

    let c = parsed("---\nstatus: superseded\n---\n# T\n");
    assert_eq!(
        c,
        SpecFrontMatter {
            epic: None,
            status: SpecStatus::Superseded,
            vendors: vec![]
        },
        "a missing epic: key reads as null"
    );

    let d = parsed("---\nepic: \"12\"\nstatus: 'active'\nvendors: []\n---\n");
    assert_eq!(
        (d.epic, d.status),
        (Some(12), SpecStatus::Active),
        "quotes are tolerated"
    );

    let e = parsed("---\nstatus: active\nvendors:\n  - cuda\n  - nvidia\nepic: 7\n---\n");
    assert_eq!(e.vendors, vec!["cuda", "nvidia"], "a block list is a list");
    assert_eq!(e.epic, Some(7), "key order is free");

    let f = parsed(
        "---\ntitle: Something\nTicket: PMAT-1\n# a comment\n\nstatus: active\nepic: 3\n---\n",
    );
    assert_eq!(
        f.epic,
        Some(3),
        "pmat spec's own keys, comments and blank lines are ignored"
    );
    assert_eq!(f.status, SpecStatus::Active);

    let g = parsed("---\r\nepic: 9\r\nstatus: active\r\n---\r\n# T\r\n");
    assert_eq!(g.epic, Some(9), "CRLF is a line ending");
}

/// Mutant: a `# Title` first line read as the opening fence, or an unclosed
/// block read as closed at end of file.
#[test]
fn a_file_that_does_not_open_with_the_fence_has_no_front_matter() {
    assert_eq!(
        parse_front_matter("# Title\n---\nstatus: active\n---\n"),
        Err(FrontMatterError::Absent)
    );
    assert_eq!(parse_front_matter(""), Err(FrontMatterError::Absent));
    assert_eq!(
        parse_front_matter("\n---\nstatus: active\n---\n"),
        Err(FrontMatterError::Absent),
        "the fence is line one, not the first non-blank line"
    );
    assert_eq!(
        parse_front_matter("---\nstatus: active\n"),
        Err(FrontMatterError::Unterminated)
    );
    assert_eq!(
        parse_front_matter("--- \nstatus: active\n---\n"),
        Err(FrontMatterError::Absent),
        "the fence is exactly ---"
    );
}

/// Mutant: a bad value read as its default (a misspelt status as `active`, a
/// non-number epic as null), or a case-folded status accepted.
#[test]
fn bad_values_are_named_not_defaulted() {
    assert_eq!(
        parse_front_matter("---\nepic: 12a\nstatus: active\n---\n"),
        Err(FrontMatterError::BadEpic("12a".to_string()))
    );
    assert_eq!(
        parse_front_matter("---\nepic: '#12'\nstatus: active\n---\n"),
        Err(FrontMatterError::BadEpic("#12".to_string())),
        "an issue number, not a reference"
    );
    assert_eq!(
        parsed("---\nepic: #12\nstatus: active\n---\n").epic,
        None,
        "bare, `#12` is a YAML comment: the value is null and an active spec is NO-EPIC — a reference is refused on one leg or the other"
    );
    assert_eq!(
        parse_front_matter("---\nepic: null\nstatus: draft\n---\n"),
        Err(FrontMatterError::BadStatus("draft".to_string()))
    );
    assert_eq!(
        parse_front_matter("---\nepic: null\nstatus: Active\n---\n"),
        Err(FrontMatterError::BadStatus("Active".to_string())),
        "the set is closed and lower-case"
    );
    assert_eq!(
        parse_front_matter("---\nepic: null\n---\n"),
        Err(FrontMatterError::MissingStatus)
    );
    assert_eq!(
        parse_front_matter("---\nstatus: active\nvendors: cuda\n---\n"),
        Err(FrontMatterError::BadVendors("cuda".to_string()))
    );
    assert_eq!(SpecStatus::parse("Active"), None);
    assert_eq!(
        SpecStatus::parse("historical"),
        Some(SpecStatus::Historical)
    );

    assert_eq!(
        FrontMatterError::BadStatus("draft".to_string()).render(),
        "status: \"draft\" is not active | superseded | historical"
    );
    assert_eq!(
        FrontMatterError::BadEpic("x".to_string()).render(),
        "epic: \"x\" is neither an issue number nor null"
    );
    assert_eq!(
        FrontMatterError::MissingStatus.render(),
        "status: is missing (active | superseded | historical)"
    );
    assert_eq!(
        FrontMatterError::Unterminated.render(),
        "the --- front-matter block is not closed"
    );
}

/// Mutant M1: the parse leg dropped for a `historical` spec (its exemption
/// leaking from the epic leg into the parse leg). Mutant M2: `NO-EPIC` only
/// when the `epic:` key is missing, so `epic: null` passes. Also: findings
/// or exemptions in input order instead of path order.
#[test]
fn the_parse_leg_finds_no_epic_on_active_specs_only_and_parse_defects_on_every_status() {
    let specs = vec![
        spec("e.md", "---\nstatus: historical\nepic: x\n---\n# E\n"),
        spec("a.md", &fm("null", "active")),
        spec("d.md", "# D — no front-matter\n"),
        spec("c.md", &fm("null", "superseded")),
        spec("b.md", &fm("null", "historical")),
        spec(
            "f.md",
            "---\nstatus: active\n---\n# F — the epic: line deleted (§7 falsifier)\n",
        ),
    ];
    let p = parse_specs(&specs);
    let paths: Vec<&str> = p.specs.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "docs/specifications/a.md",
            "docs/specifications/b.md",
            "docs/specifications/c.md",
            "docs/specifications/f.md"
        ],
        "parsed specs, path order"
    );
    assert_eq!(
        p.findings,
        vec![
            SpecFinding::NoEpic {
                spec: "docs/specifications/a.md".to_string()
            },
            SpecFinding::NoFrontMatter {
                spec: "docs/specifications/d.md".to_string()
            },
            SpecFinding::BadFrontMatter {
                spec: "docs/specifications/e.md".to_string(),
                what: "epic: \"x\" is neither an issue number nor null".to_string()
            },
            SpecFinding::NoEpic {
                spec: "docs/specifications/f.md".to_string()
            },
        ]
    );
    assert_eq!(
        p.exempt(),
        vec![
            (
                "docs/specifications/b.md".to_string(),
                SpecStatus::Historical
            ),
            (
                "docs/specifications/c.md".to_string(),
                SpecStatus::Superseded
            )
        ]
    );
    assert_eq!(
        p.render_exempt(),
        "docs/specifications/b.md (historical), docs/specifications/c.md (superseded)"
    );
    assert!(
        p.epics_named().is_empty(),
        "no active spec names an epic, so the epic leg needs no snapshot"
    );
    assert_eq!(p.active().count(), 2);
    assert_eq!(Parsed::default().render_exempt(), "none");
}

/// Mutant: `epics_named` including an exempt spec's epic — a historical
/// spec's epic fetched and judged.
#[test]
fn the_epics_named_are_the_active_specs_only() {
    let specs = vec![
        spec("a.md", &fm("1", "active")),
        spec("b.md", &fm("2", "historical")),
        spec("c.md", &fm("1", "active")),
        spec("d.md", &fm("3", "superseded")),
    ];
    let p = parse_specs(&specs);
    assert_eq!(p.epics_named().into_iter().collect::<Vec<_>>(), vec![1]);
    assert!(p.findings.is_empty());
}

/// Mutant M3: a sub-issue count the snapshot did not carry read as zero, or as
/// a pass. Mutant M4: the label clause dropped (any open issue is an epic).
/// Mutant M5: the closed clause dropped. Also: the clause order — a closed
/// unlabelled issue must be `EPIC-CLOSED`, not `NOT-AN-EPIC`; an exempt spec's
/// closed epic is nobody's finding.
#[test]
fn the_epic_leg_reports_the_first_failing_clause_in_path_order() {
    let specs = vec![
        spec("g.md", &fm("7", "historical")),
        spec("f.md", &fm("6", "active")),
        spec("e.md", &fm("5", "active")),
        spec("d.md", &fm("4", "active")),
        spec("c.md", &fm("3", "active")),
        spec("b.md", &fm("2", "active")),
        spec("a.md", &fm("1", "active")),
    ];
    let p = parse_specs(&specs);
    assert!(p.findings.is_empty(), "{:?}", p.findings);
    let snap = snapshot(vec![
        issue(2, IssueState::Closed, &[], Some(1)),
        issue(3, IssueState::Open, &["bug"], Some(1)),
        issue(4, IssueState::Open, &["epic"], None),
        issue(5, IssueState::Open, &["epic"], Some(0)),
        issue(6, IssueState::Open, &["epic", "bug"], Some(2)),
        issue(7, IssueState::Closed, &["epic"], Some(0)),
    ]);
    let found = bind_epics(&p, &snap);
    assert_eq!(
        found,
        vec![
            SpecFinding::EpicAbsent {
                spec: "docs/specifications/a.md".to_string(),
                number: 1
            },
            SpecFinding::EpicClosed {
                spec: "docs/specifications/b.md".to_string(),
                number: 2
            },
            SpecFinding::NotAnEpic {
                spec: "docs/specifications/c.md".to_string(),
                number: 3
            },
            SpecFinding::SubIssuesUnmeasured {
                spec: "docs/specifications/d.md".to_string(),
                number: 4
            },
            SpecFinding::NoSubIssues {
                spec: "docs/specifications/e.md".to_string(),
                number: 5
            },
        ]
    );
    assert_eq!(found.iter().filter(|f| f.is_unmeasured()).count(), 1);
    let classes: Vec<&str> = found.iter().map(SpecFinding::class).collect();
    assert_eq!(
        classes,
        vec![
            "EPIC-ABSENT",
            "EPIC-CLOSED",
            "NOT-AN-EPIC",
            "SUB-ISSUES-UNMEASURED",
            "NO-SUB-ISSUES"
        ]
    );
    for f in &found {
        let r = f.render();
        assert!(
            r.starts_with(&format!("{} {}:", f.class(), f.spec())),
            "{r}"
        );
    }
    assert_eq!(found[3].render(), "SUB-ISSUES-UNMEASURED docs/specifications/d.md: the snapshot does not carry #4's sub-issue count — not measured is not zero (goal-mode.md doctrine 2)");
    assert_eq!(found[4].render(), "NO-SUB-ISSUES docs/specifications/e.md: #5 has no sub-issue — a spec's tickets are its epic's sub-issues (goal-mode.md §4.3)");
    assert_eq!(SpecFinding::NoEpic { spec: "s".to_string() }.render(), "NO-EPIC s: epic is null — an active spec names an open issue labelled epic (goal-mode.md §4.3)");
    assert_eq!(SpecFinding::NoFrontMatter { spec: "s".to_string() }.render(), "NO-FRONT-MATTER s: the file does not begin with a --- front-matter block (goal-mode.md §4.3)");
}

/// Mutant M6: `strip_inline_comment` returning its input unchanged (a
/// trailing `# …` kept in the value), or a `#` inside quotes taken as a
/// comment. The header goal-mode.md §4.3 documents carries a comment on every
/// line; it is read from the spec itself so the two cannot drift.
#[test]
fn the_header_goal_mode_documents_parses_with_its_inline_comments() {
    let doc = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/specifications/goal-mode.md"
    ))
    .expect("goal-mode.md is in the tree");
    let start = doc
        .find("```yaml\n---\nepic:")
        .expect("§4.3 documents the header in a yaml fence");
    let block = &doc[start + "```yaml\n".len()..];
    let end = block.find("```").expect("the fence closes");
    let block = &block[..end];
    assert!(
        block.contains('#'),
        "the documented header carries comments: {block:?}"
    );
    let fm = parsed(block);
    assert_eq!(
        fm,
        SpecFrontMatter {
            epic: Some(1234),
            status: SpecStatus::Active,
            vendors: vec!["cuda".to_string()],
        },
        "{block:?}"
    );

    let commented = "---\nepic: 7 # seven\nstatus: active   # still active\nvendors:\n  - cuda # gpu\n  - 'rocm' # amd\n---\n";
    assert_eq!(
        parsed(commented),
        SpecFrontMatter {
            epic: Some(7),
            status: SpecStatus::Active,
            vendors: vec!["cuda".to_string(), "rocm".to_string()],
        }
    );
    assert_eq!(
        parsed("---\nepic: # filled later\nstatus: active\n---\n").epic,
        None
    );
    assert_eq!(
        parse_front_matter("---\nepic: '7 # not a comment'\nstatus: active\n---\n"),
        Err(FrontMatterError::BadEpic("7 # not a comment".to_string())),
        "a # inside quotes is part of the value"
    );
    assert_eq!(
        parse_front_matter("---\nepic: 7#8\nstatus: active\n---\n"),
        Err(FrontMatterError::BadEpic("7#8".to_string())),
        "a # that follows no whitespace is part of the value (YAML)"
    );
}

/// Mutant: the `~` or the empty arm of `parse_epic` dropped — `epic: ~` and
/// `epic:` are YAML null exactly as `epic: null` is.
#[test]
fn epic_tilde_and_empty_read_as_null() {
    for epic in ["null", "~", "", "''", "\"\""] {
        assert_eq!(parsed(&fm(epic, "active")).epic, None, "epic: {epic}");
    }
}
