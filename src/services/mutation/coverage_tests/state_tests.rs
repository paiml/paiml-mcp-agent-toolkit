#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::path::PathBuf;

fn create_test_mutant(id: &str) -> Mutant {
    Mutant {
        id: id.to_string(),
        original_file: PathBuf::from("test.rs"),
        mutated_source: "fn test() {}".to_string(),
        location: SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 10,
        },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: format!("hash_{}", id),
        status: MutantStatus::Pending,
    }
}

#[test]
fn test_mutation_state_creation() {
    let mutants = vec![create_test_mutant("m1"), create_test_mutant("m2")];
    let state =
        MutationState::new(std::path::Path::new("/project"), mutants, 60, true, Some(4));

    assert_eq!(state.pending_mutants.len(), 2);
    assert!(state.completed_mutants.is_empty());
    assert!(!state.is_complete());
    assert_eq!(state.total_mutants(), 2);
    assert_eq!(state.completed_count(), 0);
    assert!((state.completion_percentage() - 0.0).abs() < 0.01);
}

#[test]
fn test_mutation_state_add_result() {
    let mutants = vec![create_test_mutant("m1"), create_test_mutant("m2")];
    let mut state =
        MutationState::new(std::path::Path::new("/project"), mutants, 60, false, None);

    let result = MutationResult {
        mutant: create_test_mutant("m1"),
        status: MutantStatus::Killed,
        test_failures: vec!["test".to_string()],
        execution_time_ms: 100,
        error_message: None,
    };

    state.add_result(result);

    assert_eq!(state.pending_mutants.len(), 1);
    assert_eq!(state.completed_mutants.len(), 1);
    assert!(!state.is_complete());
    assert!((state.completion_percentage() - 50.0).abs() < 0.01);
}

#[test]
fn test_mutation_state_completion() {
    let mutants = vec![create_test_mutant("m1")];
    let mut state =
        MutationState::new(std::path::Path::new("/project"), mutants, 60, false, None);

    let result = MutationResult {
        mutant: create_test_mutant("m1"),
        status: MutantStatus::Survived,
        test_failures: vec![],
        execution_time_ms: 50,
        error_message: None,
    };

    state.add_result(result);

    assert!(state.is_complete());
    assert!((state.completion_percentage() - 100.0).abs() < 0.01);
}

#[test]
fn test_mutation_state_empty() {
    let state =
        MutationState::new(std::path::Path::new("/project"), vec![], 60, false, None);

    assert!(state.is_complete());
    assert_eq!(state.total_mutants(), 0);
    assert!((state.completion_percentage() - 100.0).abs() < 0.01);
}

#[test]
fn test_mutation_state_config() {
    let state =
        MutationState::new(std::path::Path::new("/project"), vec![], 120, true, Some(8));

    assert_eq!(state.config.timeout_secs, 120);
    assert!(state.config.parallel);
    assert_eq!(state.config.worker_count, Some(8));
}

#[test]
fn test_default_state_path() {
    let project_path = std::path::Path::new("/my/project");
    let state_path = MutationState::default_state_path(project_path);

    assert!(state_path.ends_with("mutation_state.json"));
    assert!(state_path.to_str().unwrap().contains(".pmat"));
}
