//! Mutant test execution
//!
//! Executes tests on mutated code to empirically measure mutation score.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::time::timeout;

use super::types::{Mutant, MutantStatus, MutationResult};

/// Default timeout for test execution (10 minutes)
const DEFAULT_TIMEOUT_SECS: u64 = 600;

/// Executes tests on mutants
pub struct MutantExecutor {
    /// Timeout for test execution
    timeout: Duration,

    /// Working directory for test execution
    work_dir: PathBuf,
}

impl MutantExecutor {
    /// Create new executor with default settings
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            work_dir,
        }
    }

    /// Create executor with custom timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout = Duration::from_secs(timeout_secs);
        self
    }

    /// Execute test suite on a single mutant
    pub async fn execute_mutant(&self, mutant: &Mutant) -> Result<MutationResult> {
        let start_time = Instant::now();

        // Step 1: Backup original file
        let backup_path = self.create_backup(&mutant.original_file).await?;

        // Step 2: Write mutated source
        let write_result = fs::write(&mutant.original_file, &mutant.mutated_source).await;
        if let Err(e) = write_result {
            // Restore backup before returning error
            let _ = self.restore_backup(&mutant.original_file, &backup_path).await;
            return Err(e).context("Failed to write mutated source");
        }

        // Step 3: Run tests with timeout
        let test_result = timeout(
            self.timeout,
            self.run_cargo_test()
        ).await;

        // Step 4: Restore original file
        self.restore_backup(&mutant.original_file, &backup_path).await?;

        // Step 5: Parse results
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        let (status, test_failures, error_message) = match test_result {
            Ok(Ok(output)) => self.parse_test_output(&output),
            Ok(Err(e)) => {
                // Compilation or test execution error
                (MutantStatus::CompileError, vec![], Some(e.to_string()))
            }
            Err(_) => {
                // Timeout
                (MutantStatus::Timeout, vec![], Some("Test execution timed out".to_string()))
            }
        };

        Ok(MutationResult {
            mutant: mutant.clone(),
            status,
            test_failures,
            execution_time_ms,
            error_message,
        })
    }

    /// Execute tests on multiple mutants in parallel
    pub async fn execute_mutants(&self, mutants: &[Mutant]) -> Result<Vec<MutationResult>> {
        // For safety, run sequentially to avoid file conflicts
        // Future optimization: run in parallel with file locking
        let mut results = Vec::new();

        for (i, mutant) in mutants.iter().enumerate() {
            println!("  [{}/{}] Testing mutant {}...", i + 1, mutants.len(), mutant.id);

            match self.execute_mutant(mutant).await {
                Ok(result) => {
                    let status_symbol = match result.status {
                        MutantStatus::Killed => "✅",
                        MutantStatus::Survived => "❌",
                        MutantStatus::CompileError => "🔧",
                        MutantStatus::Timeout => "⏱️",
                        _ => "❓",
                    };
                    println!("    {} {:?} ({}ms)", status_symbol, result.status, result.execution_time_ms);
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("    ⚠️  Error executing mutant {}: {}", mutant.id, e);
                    // Create error result
                    results.push(MutationResult {
                        mutant: mutant.clone(),
                        status: MutantStatus::CompileError,
                        test_failures: vec![],
                        execution_time_ms: 0,
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Create backup of original file
    async fn create_backup(&self, original_path: &Path) -> Result<PathBuf> {
        let backup_path = original_path.with_extension("pmat_backup");
        fs::copy(original_path, &backup_path).await
            .context("Failed to create backup")?;
        Ok(backup_path)
    }

    /// Restore original file from backup
    async fn restore_backup(&self, original_path: &Path, backup_path: &Path) -> Result<()> {
        fs::copy(backup_path, original_path).await
            .context("Failed to restore backup")?;
        fs::remove_file(backup_path).await
            .context("Failed to remove backup")?;
        Ok(())
    }

    /// Run cargo test in working directory
    async fn run_cargo_test(&self) -> Result<String> {
        let output = Command::new("cargo")
            .arg("test")
            .arg("--lib")
            .arg("--")
            .arg("--nocapture")
            .current_dir(&self.work_dir)
            .output()
            .context("Failed to run cargo test")?;

        // Combine stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr);

        Ok(combined)
    }

    /// Parse test output to determine mutant status
    fn parse_test_output(&self, output: &str) -> (MutantStatus, Vec<String>, Option<String>) {
        // Check for compilation errors
        if output.contains("error: could not compile") || output.contains("error[E") {
            return (
                MutantStatus::CompileError,
                vec![],
                Some("Compilation failed".to_string())
            );
        }

        // Extract test failures
        let test_failures = self.extract_test_failures(output);

        // Determine status based on test results
        let status = if !test_failures.is_empty() {
            // At least one test failed -> mutant was killed
            MutantStatus::Killed
        } else if output.contains("test result: ok") {
            // All tests passed -> mutant survived
            MutantStatus::Survived
        } else {
            // Unclear status, default to survived
            MutantStatus::Survived
        };

        (status, test_failures, None)
    }

    /// Extract failed test names from output
    fn extract_test_failures(&self, output: &str) -> Vec<String> {
        let mut failures = Vec::new();

        for line in output.lines() {
            // Look for "test <name> ... FAILED" pattern (not "test result:")
            if line.starts_with("test ")
                && line.contains("... FAILED")
                && !line.starts_with("test result") {
                if let Some(test_name) = line.split_whitespace().nth(1) {
                    failures.push(test_name.to_string());
                }
            }
        }

        failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mutation::types::{MutationOperatorType, SourceLocation};

    #[test]
    fn test_parse_compilation_error() {
        let executor = MutantExecutor::new(PathBuf::from("."));
        let output = "error[E0308]: mismatched types\n  --> src/lib.rs:10:5";

        let (status, failures, error) = executor.parse_test_output(output);

        assert_eq!(status, MutantStatus::CompileError);
        assert!(failures.is_empty());
        assert!(error.is_some());
    }

    #[test]
    fn test_parse_test_failure() {
        let executor = MutantExecutor::new(PathBuf::from("."));
        let output = "running 3 tests\ntest test_add ... FAILED\ntest test_sub ... ok\n\ntest result: FAILED. 1 passed; 1 failed";

        let (status, failures, error) = executor.parse_test_output(output);

        assert_eq!(status, MutantStatus::Killed);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0], "test_add");
        assert!(error.is_none());
    }

    #[test]
    fn test_parse_all_tests_passed() {
        let executor = MutantExecutor::new(PathBuf::from("."));
        let output = "running 3 tests\ntest test_add ... ok\ntest test_sub ... ok\n\ntest result: ok. 2 passed; 0 failed";

        let (status, failures, error) = executor.parse_test_output(output);

        assert_eq!(status, MutantStatus::Survived);
        assert!(failures.is_empty());
        assert!(error.is_none());
    }

    #[test]
    fn test_extract_multiple_failures() {
        let executor = MutantExecutor::new(PathBuf::from("."));
        let output = "test test_add ... FAILED\ntest test_sub ... FAILED\ntest test_mul ... ok";

        let failures = executor.extract_test_failures(output);

        assert_eq!(failures.len(), 2);
        assert!(failures.contains(&"test_add".to_string()));
        assert!(failures.contains(&"test_sub".to_string()));
    }
}
