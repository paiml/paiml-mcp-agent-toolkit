//! RED tests for mutation cleanup on interrupt/timeout
//!
//! CRITICAL BUG FOUND: When mutation testing times out or is interrupted,
//! the mutated file is left in place (not restored from backup).
//!
//! ROOT CAUSE: execute_mutant() modifies files in place, SIGINT kills before restore
//!
//! FIX: NEVER modify original file - use temp files ONLY (Toyota Way: proper design)

use pmat::services::mutation::{Mutant, MutantExecutor, MutantStatus, MutationOperatorType, SourceLocation};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn red_must_restore_file_on_timeout() {
    // Create a test file
    let test_file = std::env::temp_dir().join("test_timeout_restore.rs");
    let original_content = "fn original() { 42 }";
    std::fs::write(&test_file, original_content).unwrap();

    let mutant = Mutant {
        id: "TEST_TIMEOUT".to_string(),
        original_file: test_file.clone(),
        mutated_source: "fn mutated() { 999 }".to_string(),
        location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 10 },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "test".to_string(),
        status: MutantStatus::Pending,
    };

    let executor = MutantExecutor::new(std::env::temp_dir())
        .with_timeout(1); // 1 second timeout

    // Execute mutant (will timeout)
    let _ = timeout(Duration::from_millis(500), executor.execute_mutant(&mutant)).await;

    // RED: File MUST be restored to original, even after timeout!
    let final_content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(final_content, original_content,
        "File was NOT restored after timeout! This is the bug we found.");

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
}

#[tokio::test]
async fn red_must_restore_file_on_panic() {
    // Create a test file
    let test_file = std::env::temp_dir().join("test_panic_restore.rs");
    let original_content = "fn original() { 42 }";
    std::fs::write(&test_file, original_content).unwrap();

    let mutant = Mutant {
        id: "TEST_PANIC".to_string(),
        original_file: test_file.clone(),
        mutated_source: "fn mutated() { panic!() }".to_string(),
        location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 10 },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "test".to_string(),
        status: MutantStatus::Pending,
    };

    let executor = MutantExecutor::new(std::env::temp_dir());

    // Execute mutant (may panic during test run)
    let _ = executor.execute_mutant(&mutant).await;

    // RED: File MUST be restored even if panic occurs!
    let final_content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(final_content, original_content,
        "File was NOT restored after panic!");

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
}

#[tokio::test]
async fn red_must_clean_up_backup_files() {
    // Create a test file
    let test_file = std::env::temp_dir().join("test_backup_cleanup.rs");
    let original_content = "fn original() { 42 }";
    std::fs::write(&test_file, original_content).unwrap();

    let mutant = Mutant {
        id: "TEST_BACKUP".to_string(),
        original_file: test_file.clone(),
        mutated_source: "fn mutated() { 0 }".to_string(),
        location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 10 },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "test".to_string(),
        status: MutantStatus::Pending,
    };

    let executor = MutantExecutor::new(std::env::temp_dir())
        .with_timeout(1);

    // Execute mutant
    let _ = executor.execute_mutant(&mutant).await;

    // RED: Backup file MUST be cleaned up!
    let backup_file = test_file.with_extension("pmat_backup");
    assert!(!backup_file.exists(),
        "Backup file was not cleaned up! Found: {}", backup_file.display());

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(&backup_file);
}

#[tokio::test]
async fn red_original_file_must_never_be_modified() {
    // This is the REAL fix: NEVER touch the original file!

    // Create a test file
    let test_file = std::env::temp_dir().join("test_never_modified.rs");
    let original_content = "fn original() { 42 }";
    std::fs::write(&test_file, original_content).unwrap();

    let mutant = Mutant {
        id: "TEST_NEVER_MODIFY".to_string(),
        original_file: test_file.clone(),
        mutated_source: "fn mutated() { 999 }".to_string(),
        location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 10 },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "test".to_string(),
        status: MutantStatus::Pending,
    };

    let executor = MutantExecutor::new(std::env::temp_dir());

    // Execute mutant
    let _ = executor.execute_mutant(&mutant).await;

    // RED: Original file MUST be completely unchanged!
    // We should NEVER write to the original file, only to temp files
    let final_content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(final_content, original_content,
        "CRITICAL: Original file was modified! Should use temp files only.");

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
}
