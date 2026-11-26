//! The rubric: weights, additivity (so monotone), determinism, and each item's edge.

use super::rubric::*;
use super::test_support::ticket_from;
use crate::services::normalized_score::{Grade, NormalizedScore};

fn with_body(body: &str) -> Ticket {
    Ticket {
        body: body.to_string(),
        ..Ticket::default()
    }
}

fn with_title(title: &str) -> Ticket {
    Ticket {
        title: title.to_string(),
        ..Ticket::default()
    }
}

#[test]
fn ticket_quality_weights_sum_to_100_over_seven_items() {
    assert_eq!(Item::ALL.len(), 7);
    assert_eq!(Item::ALL.iter().map(|i| i.points()).sum::<u32>(), 100);
    let weights: Vec<u32> = Item::ALL.iter().map(|i| i.points()).collect();
    assert_eq!(weights, [25, 20, 15, 15, 10, 10, 5]);
}

#[test]
fn ticket_quality_score_is_the_sum_of_the_passed_items_for_every_subset() {
    for mask in 0u8..128 {
        let want: u32 = Item::ALL
            .iter()
            .enumerate()
            .filter(|(bit, _)| mask & (1 << bit) != 0)
            .map(|(_, i)| i.points())
            .sum();
        let got = score(&ticket_from(mask), PASS_MARK);
        assert_eq!(got.total, want, "mask {mask:07b}: {got}");
        assert_eq!(got.missing.len(), 7 - mask.count_ones() as usize);
        assert_eq!(got.passed, want >= PASS_MARK);
    }
}

#[test]
fn ticket_quality_passing_one_more_item_never_lowers_the_score() {
    for mask in 0u8..128 {
        for bit in 0..7 {
            let more = score(&ticket_from(mask | (1 << bit)), PASS_MARK).total;
            assert!(more >= score(&ticket_from(mask), PASS_MARK).total);
        }
    }
}

#[test]
fn ticket_quality_same_input_gives_the_same_score() {
    for mask in [0u8, 0b101_0101, 0b111_1111] {
        let t = ticket_from(mask);
        assert_eq!(score(&t, PASS_MARK), score(&t.clone(), PASS_MARK));
    }
}

#[test]
fn ticket_quality_full_ticket_scores_100_and_grades_a() {
    let s = score(&ticket_from(0b111_1111), PASS_MARK);
    assert_eq!((s.total, s.passed, s.missing.len()), (100, true, 0));
    assert_eq!(
        (s.raw(), s.max_raw(), s.normalized()),
        (100.0, 100.0, 100.0)
    );
    assert_eq!(s.grade(), Grade::A);
    let text = s.to_string();
    assert!(
        text.contains("ticket score: 100/100 (pass mark 70) PASS"),
        "{text}"
    );
    assert!(text.contains("25/25 acceptance"), "{text}");
}

#[test]
fn ticket_quality_empty_ticket_scores_0_and_lists_every_item() {
    let s = score(&Ticket::default(), PASS_MARK);
    assert_eq!((s.total, s.passed, s.missing.len()), (0, false, 7));
    let text = s.to_string();
    assert!(text.contains("NEEDS REWRITE"), "{text}");
    for item in Item::ALL {
        assert!(text.contains(item.id()), "{text}");
        assert!(text.contains(item.missing()), "{text}");
    }
}

#[test]
fn ticket_quality_pass_mark_is_inclusive() {
    // acceptance 25 + falsifiable 20 + parent 15 + problem 10 = 70
    let t = ticket_from(0b001_0111);
    assert_eq!(score(&t, 70).total, 70);
    assert!(score(&t, 70).passed);
    assert!(!score(&t, 71).passed);
}

#[test]
fn ticket_quality_filing_gate_refuses_under_the_mark_and_names_what_is_missing() {
    let err = refuse_low_score(&ticket_from(0b000_0001), PASS_MARK)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("scores 25/100, under the pass mark 70"),
        "{err}"
    );
    assert!(err.contains(Item::Falsifiable.missing()), "{err}");
    assert!(!err.contains(Item::Acceptance.missing()), "{err}");
    assert_eq!(
        refuse_low_score(&ticket_from(0b111_1111), PASS_MARK)
            .unwrap()
            .total,
        100
    );
}

#[test]
fn ticket_quality_acceptance_needs_a_checklist_under_the_heading() {
    let pass = |b: &str| Item::Acceptance.passes(&with_body(b));
    assert!(pass("## Acceptance\n- [ ] one\n- [x] two\n* [X] three"));
    assert!(pass("acceptance_criteria:\n- [ ] one"));
    assert!(
        !pass("## Acceptance\n- one\n- two"),
        "bullets are not a checklist"
    );
    assert!(
        !pass("## Acceptance\n- [ ]\n"),
        "an empty box is not an item"
    );
    assert!(
        !pass("- [ ] one"),
        "a checklist outside the section does not count"
    );
    assert_eq!(
        checklist("## Acceptance\n- [ ] a\n- b\n- [x] c"),
        ["[ ] a", "[x] c"]
    );
}

#[test]
fn ticket_quality_falsifiable_needs_a_code_span_or_fence_in_the_section() {
    let pass = |b: &str| Item::Falsifiable.passes(&with_body(b));
    assert!(pass("## Acceptance\n- [ ] `cargo test --lib x` passes"));
    assert!(pass("## Acceptance\n```bash\nmake gate\n```"));
    assert!(!pass("## Acceptance\n- [ ] it works"));
    assert!(!pass("## Acceptance\n- [ ] empty `` span"));
    assert!(!pass(
        "`cargo test` outside\n\n## Acceptance\n- [ ] it works"
    ));
}

#[test]
fn ticket_quality_parent_item_needs_an_open_parent_and_a_milestone() {
    let t = |parent_open, has_milestone| Ticket {
        parent_open,
        has_milestone,
        ..Ticket::default()
    };
    assert!(Item::ParentMilestone.passes(&t(true, true)));
    assert!(!Item::ParentMilestone.passes(&t(true, false)));
    assert!(!Item::ParentMilestone.passes(&t(false, true)));
}

#[test]
fn ticket_quality_scope_is_a_path_a_scoped_title_or_a_scope_line() {
    let pass = |b: &str| Item::Scope.passes(&with_body(b));
    assert!(pass("fix src/lib.rs"));
    assert!(pass("everything under machines/intel/"));
    assert!(pass("see machines/intel/Makefile"));
    assert!(pass("**Scope:** the arm rule"));
    assert!(pass("Component: arbiter"));
    assert!(!pass("Scope:"), "an empty scope line names nothing");
    assert!(!pass(
        "and/or CI/CD on paiml/infra#12 at https://github.com/paiml/infra"
    ));
    assert!(!pass("the scope of this is wide"));
    assert!(Item::Scope.passes(&with_title("fix(arm): one rule")));
    assert!(!Item::Scope.passes(&with_title("fix: one rule")));
}

#[test]
fn ticket_quality_named_paths_are_repo_relative_and_clean() {
    let text = "at src/a.rs:12:3, ./scripts/x.sh. Not /abs/p.rs, ~/h/q.rs, ../up/r.rs, \
                a//b.rs, src/*.rs, <dir>/s.rs, $X/t.rs, http://h.io/u.rs; .github/w.yml; src/a.rs";
    assert_eq!(
        named_paths(text),
        ["src/a.rs", "scripts/x.sh", ".github/w.yml"]
    );
    assert_eq!(named_files("docs/ and src/a.rs and bin/tool"), ["src/a.rs"]);
}

#[test]
fn ticket_quality_problem_needs_observed_and_expected() {
    let pass = |b: &str| Item::Problem.passes(&with_body(b));
    assert!(pass("## Observed\nx\n## Expected behaviour\ny"));
    assert!(pass("**Observed:** x\n- Expected: y"));
    assert!(pass("Observed**:** x\n> expected : y"));
    assert!(!pass("## Observed\nx"));
    assert!(
        !pass("observed nothing, expected nothing"),
        "prose is not a label"
    );
}

#[test]
fn ticket_quality_size_needs_at_most_8_items_and_a_kind_label() {
    let t = |items: usize, labels: &[&str]| Ticket {
        body: format!("## Acceptance\n{}", "- [ ] x\n".repeat(items)),
        labels: labels.iter().map(|l| l.to_string()).collect(),
        ..Ticket::default()
    };
    assert!(Item::Size.passes(&t(8, &["Kind:code"])));
    assert!(Item::Size.passes(&t(0, &["kind:docs"])));
    assert!(!Item::Size.passes(&t(9, &["kind:code"])));
    assert!(!Item::Size.passes(&t(1, &["bug", "kind:"])));
}

#[test]
fn ticket_quality_title_is_short_and_starts_with_a_verb_or_scoped_prefix() {
    let pass = |t: &str| Item::Title.passes(&with_title(t));
    assert!(pass("Add a ticket score"));
    assert!(pass("fix: the arm rule"));
    assert!(pass("arbiter(flow): one loop"));
    assert!(pass(&format!("Add {}", "x".repeat(76))));
    assert!(!pass(&format!("Add {}", "x".repeat(77))), "81 characters");
    assert!(!pass("The arm rule is in three places"));
    assert!(!pass("  "));
}
