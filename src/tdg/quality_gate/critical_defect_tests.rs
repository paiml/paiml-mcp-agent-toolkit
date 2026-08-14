//! The gate must stay hard while the score is soft.
//!
//! These pin the property that motivated splitting them: a file carrying an
//! unsuppressed critical defect fails the gate *whatever its score is*. If a
//! future change to the penalty in `TdgScore::calculate_total` lifts an
//! offending file back above the F band, enforcement must not move with it.

use super::CriticalDefectGate;
use crate::tdg::quality_gate::types::{QualityGate, Severity};
use crate::tdg::{BaselineEntry, ComponentScores, Grade, Language, TdgBaseline, TdgScore};
use std::path::PathBuf;

fn baseline_with(files: Vec<(&str, usize, Option<&str>, f32, Grade)>) -> TdgBaseline {
    let mut baseline = TdgBaseline::new(None);
    for (path, count, suppressed, total, grade) in files {
        let p = PathBuf::from(path);
        let entry = BaselineEntry {
            content_hash: blake3::hash(b"test"),
            score: TdgScore {
                total,
                grade,
                critical_defects_count: count,
                has_critical_defects: count > 0,
                critical_defects_suppressed: suppressed.map(str::to_string),
                file_path: Some(p.clone()),
                language: Language::Rust,
                ..Default::default()
            },
            components: ComponentScores::default(),
            git_context: None,
        };
        baseline.add_entry(p, entry);
    }
    baseline
}

#[test]
fn a_clean_baseline_passes() {
    let current = baseline_with(vec![("src/good.rs", 0, None, 96.0, Grade::APlus)]);
    let result = CriticalDefectGate::with_defaults()
        .check(&TdgBaseline::new(None), &current)
        .expect("gate runs");

    assert!(result.passed);
    assert!(result.violations.is_empty());
    assert_eq!(result.message, "No critical defects");
}

/// The property the split exists to guarantee.
#[test]
fn a_defective_file_fails_the_gate_even_at_a_passing_score() {
    // 69.9 / C+ is what the graduated penalty produces for a single defect in
    // an otherwise excellent file. It is nowhere near F.
    let current = baseline_with(vec![("src/bad.rs", 1, None, 69.9, Grade::CPlus)]);
    let result = CriticalDefectGate::with_defaults()
        .check(&TdgBaseline::new(None), &current)
        .expect("gate runs");

    assert!(
        !result.passed,
        "a critical defect must fail the gate regardless of score"
    );
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].severity, Severity::Critical);
}

/// #279's waiver is honoured, and disclosed rather than hidden.
#[test]
fn a_waived_file_passes_but_the_waiver_is_reported() {
    let current = baseline_with(vec![(
        "src/new.rs",
        2,
        Some("no commits yet (#279)"),
        96.0,
        Grade::APlus,
    )]);
    let result = CriticalDefectGate::with_defaults()
        .check(&TdgBaseline::new(None), &current)
        .expect("gate runs");

    assert!(result.passed);
    assert!(result.violations.is_empty());
    assert!(
        result.message.contains("waived"),
        "a pass that skipped files must say so: {}",
        result.message
    );
}

#[test]
fn waived_and_unwaived_files_are_counted_separately() {
    let current = baseline_with(vec![
        ("src/bad.rs", 1, None, 40.0, Grade::F),
        (
            "src/new.rs",
            1,
            Some("no commits yet (#279)"),
            96.0,
            Grade::APlus,
        ),
        ("src/good.rs", 0, None, 96.0, Grade::APlus),
    ]);
    let result = CriticalDefectGate::with_defaults()
        .check(&TdgBaseline::new(None), &current)
        .expect("gate runs");

    assert!(!result.passed);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].path, PathBuf::from("src/bad.rs"));
    assert!(result.message.contains("1 waived"), "{}", result.message);
}

#[test]
fn a_migration_budget_tolerates_that_many_files() {
    let current = baseline_with(vec![
        ("src/a.rs", 1, None, 40.0, Grade::F),
        ("src/b.rs", 1, None, 40.0, Grade::F),
    ]);

    assert!(
        !CriticalDefectGate::new(1)
            .check(&TdgBaseline::new(None), &current)
            .expect("gate runs")
            .passed
    );
    assert!(
        CriticalDefectGate::new(2)
            .check(&TdgBaseline::new(None), &current)
            .expect("gate runs")
            .passed
    );
}
