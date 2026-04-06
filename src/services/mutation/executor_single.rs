// include!()'d from executor.rs
// Single and sequential mutant execution methods

impl MutantExecutor {
    /// Execute test suite on a single mutant
    ///
    /// Uses RAII pattern with MutantGuard to ensure original file is ALWAYS restored
    /// even if process is interrupted (SIGINT/Ctrl+C)
    ///
    /// BUG-064 FIX: Uses atomic write operations to prevent file corruption
    pub async fn execute_mutant(&self, mutant: &Mutant) -> Result<MutationResult> {
        let start_time = Instant::now();

        // Step 1: Create MutantGuard to handle backup and automatic restoration
        // The guard will restore the file when dropped, even on panic or early return
        let mut guard = super::guard::MutantGuard::new(&mutant.original_file).await?;

        // Step 2: Write mutated source ATOMICALLY (BUG-064 FIX)
        // Using atomic write ensures file is either fully written or unchanged
        // Prevents "491 lines -> 5 lines" corruption on timeout/interruption
        self.atomic_write(&mutant.original_file, &mutant.mutated_source)
            .await
            .context("Failed to write mutated source atomically")?;

        // Step 3: Run tests with timeout (smart filtering)
        let test_result = timeout(self.timeout, self.run_cargo_test_for_mutant(mutant)).await;

        // Step 4: Explicitly restore file (not strictly necessary, but preferred)
        // The guard's Drop implementation is our safety net if this fails
        if let Err(e) = guard.restore().await {
            eprintln!("Warning: Error restoring file: {}", e);
            // Continue - we'll still create a result
        }

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
                (
                    MutantStatus::Timeout,
                    vec![],
                    Some("Test execution timed out".to_string()),
                )
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

    /// Execute tests on multiple mutants sequentially
    pub async fn execute_mutants(&self, mutants: &[Mutant]) -> Result<Vec<MutationResult>> {
        debug_assert!(!mutants.is_empty(), "mutants must not be empty");
        let mut results = Vec::new();

        for (i, mutant) in mutants.iter().enumerate() {
            println!(
                "  [{}/{}] Testing mutant {}...",
                i + 1,
                mutants.len(),
                mutant.id
            );

            match self.execute_mutant(mutant).await {
                Ok(result) => {
                    let status_symbol = match result.status {
                        MutantStatus::Killed => "✅",
                        MutantStatus::Survived => "❌",
                        MutantStatus::CompileError => "🔧",
                        MutantStatus::Timeout => "⏱️",
                        _ => "❓",
                    };
                    println!(
                        "    {} {:?} ({}ms)",
                        status_symbol, result.status, result.execution_time_ms
                    );
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
}
