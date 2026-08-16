#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::path::PathBuf;

fn create_test_result(status: MutantStatus, file: &str, line: usize) -> MutationResult {
    MutationResult {
        mutant: Mutant {
            id: format!("test_{}", line),
            original_file: PathBuf::from(file),
            mutated_source: String::new(),
            location: SourceLocation {
                line,
                column: 1,
                end_line: line,
                end_column: 10,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: format!("hash_{}", line),
            status: status.clone(),
        },
        status,
        test_failures: vec![],
        execution_time_ms: 100,
        error_message: None,
    }
}

#[test]
fn test_mutation_scorer_creation() {
    let results = vec![
        create_test_result(MutantStatus::Killed, "foo.rs", 10),
        create_test_result(MutantStatus::Survived, "foo.rs", 20),
    ];

    let scorer = MutationScorer::new(results);
    let score = scorer.calculate_score();

    assert_eq!(score.total, 2);
    assert_eq!(score.killed, 1);
    assert_eq!(score.survived, 1);
    assert!((score.score - 0.5).abs() < 0.01);
}

#[test]
fn test_mutation_scorer_with_equivalents() {
    let results = vec![
        create_test_result(MutantStatus::Killed, "foo.rs", 10),
        create_test_result(MutantStatus::Killed, "foo.rs", 20),
        create_test_result(MutantStatus::Equivalent, "foo.rs", 30),
    ];

    let scorer = MutationScorer::new(results);
    let score = scorer.calculate_score();

    // Score should be 2/2 = 1.0 (equivalent mutants excluded)
    assert_eq!(score.score, 1.0);
    assert_eq!(score.equivalent, 1);
}

#[test]
fn test_mutation_summary_generation() {
    let results = vec![
        create_test_result(MutantStatus::Killed, "foo.rs", 10),
        create_test_result(MutantStatus::Survived, "foo.rs", 20),
        create_test_result(MutantStatus::Survived, "foo.rs", 25),
        create_test_result(MutantStatus::CompileError, "bar.rs", 5),
    ];

    let scorer = MutationScorer::new(results);
    let summary = scorer.summary();

    assert_eq!(summary.total_mutants, 4);
    assert_eq!(summary.killed, 1);
    assert_eq!(summary.survived, 2);
    assert_eq!(summary.compile_errors, 1);
    assert!(!summary.weak_spots.is_empty()); // foo.rs has survivors
}

#[test]
fn test_weak_spots_grouping() {
    let results = vec![
        create_test_result(MutantStatus::Survived, "fileA.rs", 10),
        create_test_result(MutantStatus::Survived, "fileA.rs", 20),
        create_test_result(MutantStatus::Survived, "fileA.rs", 30),
        create_test_result(MutantStatus::Survived, "fileB.rs", 5),
    ];

    let scorer = MutationScorer::new(results);
    let weak_spots = scorer.weak_spots();

    assert_eq!(weak_spots.len(), 2);
    // fileA should be first (more survivors)
    assert_eq!(weak_spots[0].survived_mutants, 3);
    assert_eq!(weak_spots[1].survived_mutants, 1);
}

#[test]
fn test_weak_spots_line_range() {
    let results = vec![
        create_test_result(MutantStatus::Survived, "file.rs", 5),
        create_test_result(MutantStatus::Survived, "file.rs", 15),
        create_test_result(MutantStatus::Survived, "file.rs", 25),
    ];

    let scorer = MutationScorer::new(results);
    let weak_spots = scorer.weak_spots();

    assert_eq!(weak_spots.len(), 1);
    assert_eq!(weak_spots[0].line_range, (5, 25));
}
