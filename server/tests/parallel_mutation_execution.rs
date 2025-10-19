//! RED tests for parallel mutation execution
//!
//! GOAL: Speed up mutation testing with thread pool parallelism
//! CURRENT: Sequential execution ~22-25s per mutant
//! TARGET: Parallel execution with N workers (CPU cores)
//!
//! Toyota Way: No file conflicts, safe parallel execution

use pmat::services::mutation::{
    Mutant, MutantExecutor, MutantStatus, MutationOperatorType, SourceLocation,
};
use std::path::PathBuf;
use std::time::Instant;

#[tokio::test]
#[ignore] // TDD RED phase - Feature not implemented yet (PMAT-COVERAGE-005)
          // Takes >900s per test (60+ min for 4 tests), calls non-existent execute_mutants_parallel
          // Remove #[ignore] when implementing parallel mutation execution feature
async fn red_parallel_execution_must_be_faster_than_sequential() {
    // Create test mutants (simple ones that compile)
    let mutants = create_test_mutants(4);

    let work_dir = PathBuf::from(".");
    let executor = MutantExecutor::new(work_dir.clone());

    // Sequential execution
    let start = Instant::now();
    let seq_results = executor.execute_mutants(&mutants).await.unwrap();
    let seq_time = start.elapsed();

    // Parallel execution (RED: This method doesn't exist yet!)
    let start = Instant::now();
    let par_results = executor.execute_mutants_parallel(&mutants, 4).await.unwrap();
    let par_time = start.elapsed();

    // Parallel should be faster (at least 2× with 4 workers)
    assert!(par_time < seq_time / 2,
        "Parallel ({:?}) should be at least 2× faster than sequential ({:?})",
        par_time, seq_time
    );

    // Results should be identical (order doesn't matter)
    assert_eq!(seq_results.len(), par_results.len());
}

#[tokio::test]
#[ignore] // TDD RED phase - Feature not implemented yet (PMAT-COVERAGE-005)
          // Takes >900s per test (60+ min for 4 tests), calls non-existent execute_mutants_parallel
          // Remove #[ignore] when implementing parallel mutation execution feature
async fn red_parallel_execution_must_handle_file_conflicts_safely() {
    // Multiple mutants for the SAME file
    let mutants = vec![
        create_mutant("test.rs", "fn test() { 1 + 2 }"),
        create_mutant("test.rs", "fn test() { 1 - 2 }"),
        create_mutant("test.rs", "fn test() { 1 * 2 }"),
    ];

    let work_dir = PathBuf::from(".");
    let executor = MutantExecutor::new(work_dir);

    // RED: This should not corrupt files or cause race conditions
    let results = executor.execute_mutants_parallel(&mutants, 3).await.unwrap();

    assert_eq!(results.len(), 3);

    // All mutants should complete (not crash from file conflicts)
    for result in &results {
        assert!(matches!(
            result.status,
            MutantStatus::Killed | MutantStatus::Survived | MutantStatus::CompileError
        ));
    }
}

#[tokio::test]
#[ignore] // TDD RED phase - Feature not implemented yet (PMAT-COVERAGE-005)
          // Takes >900s per test (60+ min for 4 tests), calls non-existent execute_mutants_parallel
          // Remove #[ignore] when implementing parallel mutation execution feature
async fn red_parallel_execution_must_respect_worker_count() {
    let mutants = create_test_mutants(10);

    let work_dir = PathBuf::from(".");
    let executor = MutantExecutor::new(work_dir);

    // RED: Should use exactly N workers (not more, not less)
    let results = executor.execute_mutants_parallel(&mutants, 2).await.unwrap();

    assert_eq!(results.len(), 10);

    // With 2 workers, should take ~5× the time of 1 mutant
    // (10 mutants / 2 workers = 5 rounds)
}

#[tokio::test]
async fn red_parallel_execution_must_preserve_original_files() {
    // Create a test file
    let test_file = PathBuf::from("/tmp/pmat_parallel_test.rs");
    let original_content = "fn original() { 42 }";
    std::fs::write(&test_file, original_content).unwrap();

    let mutants = vec![
        create_mutant_for_file(&test_file, "fn mutated1() { 1 }"),
        create_mutant_for_file(&test_file, "fn mutated2() { 2 }"),
    ];

    let work_dir = PathBuf::from("/tmp");
    let executor = MutantExecutor::new(work_dir);

    // RED: Parallel execution
    let _ = executor.execute_mutants_parallel(&mutants, 2).await.unwrap();

    // Original file must be unchanged
    let final_content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(final_content, original_content,
        "Original file was corrupted during parallel execution!");

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
}

#[tokio::test]
#[ignore] // TDD RED phase - Feature not implemented yet (PMAT-COVERAGE-005)
          // Takes >900s per test (60+ min for 4 tests), calls non-existent execute_mutants_parallel
          // Remove #[ignore] when implementing parallel mutation execution feature
async fn red_parallel_execution_must_not_deadlock() {
    let mutants = create_test_mutants(100);

    let work_dir = PathBuf::from(".");
    let executor = MutantExecutor::new(work_dir);

    // RED: Should complete without deadlock
    let start = Instant::now();
    let results = executor.execute_mutants_parallel(&mutants, 8).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 100);

    // Should complete in reasonable time (not hang forever)
    assert!(elapsed.as_secs() < 300, "Took too long, possible deadlock");
}

// Helper functions

fn create_test_mutants(count: usize) -> Vec<Mutant> {
    (0..count)
        .map(|i| create_mutant(&format!("test_{}.rs", i), &format!("fn test() {{ {} }}", i)))
        .collect()
}

fn create_mutant(file: &str, source: &str) -> Mutant {
    create_mutant_for_file(&PathBuf::from(file), source)
}

fn create_mutant_for_file(file: &PathBuf, source: &str) -> Mutant {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write(source.as_bytes());
    let id = hasher.finish();

    Mutant {
        id: format!("TEST_{:x}", id),
        original_file: file.clone(),
        mutated_source: source.to_string(),
        location: SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 10,
        },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "test_hash".to_string(),
        status: MutantStatus::Pending,
    }
}
