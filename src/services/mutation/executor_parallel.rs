// include!()'d from executor.rs
// Parallel mutant execution methods

impl MutantExecutor {
    /// Execute tests on multiple mutants in parallel using thread pool
    ///
    /// Uses tokio tasks for parallel execution with temporary file isolation
    /// to avoid file conflicts. Each mutant test runs independently.
    pub async fn execute_mutants_parallel(
        &self,
        mutants: &[Mutant],
        workers: usize,
    ) -> Result<Vec<MutationResult>> {
        debug_assert!(!mutants.is_empty(), "mutants must not be empty");
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        println!("  🚀 Parallel execution with {} workers", workers);

        // Create semaphore to limit concurrent executions
        let semaphore = Arc::new(Semaphore::new(workers));
        let mut tasks = Vec::new();
        let total_mutants = mutants.len();

        for (i, mutant) in mutants.iter().enumerate() {
            let sem = semaphore.clone();
            let mutant = mutant.clone();
            let executor = self.clone();
            let index = i + 1;

            // Spawn task for each mutant
            let task = tokio::spawn(async move {
                // Acquire permit (blocks if all workers busy)
                let _permit = sem.acquire().await.expect("semaphore not closed");

                println!(
                    "  [{}/{}] Testing mutant {}...",
                    index, total_mutants, mutant.id
                );

                match executor.execute_mutant_isolated(&mutant).await {
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
                        Ok(result)
                    }
                    Err(e) => {
                        eprintln!("    ⚠️  Error executing mutant {}: {}", mutant.id, e);
                        Ok(MutationResult {
                            mutant: mutant.clone(),
                            status: MutantStatus::CompileError,
                            test_failures: vec![],
                            execution_time_ms: 0,
                            error_message: Some(e.to_string()),
                        })
                    }
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }

        Ok(results)
    }

    /// Execute a single mutant in isolation (for parallel execution)
    ///
    /// Uses a unique temporary file for this mutant to avoid conflicts
    async fn execute_mutant_isolated(&self, mutant: &Mutant) -> Result<MutationResult> {
        use std::time::Instant;

        let start_time = Instant::now();

        // Create unique scratch file for this mutant (no conflicts!)
        let temp_dir = std::env::temp_dir();
        let unique_file = temp_dir.join(format!("pmat_{}_{}.rs", std::process::id(), mutant.id));

        // Write mutated source to unique scratch file
        fs::write(&unique_file, &mutant.mutated_source)
            .await
            .context("Failed to write isolated mutant")?;

        // Run tests with timeout (smart filtering)
        let test_result = timeout(self.timeout, self.run_cargo_test_for_mutant(mutant)).await;

        // Cleanup scratch file
        let _ = fs::remove_file(&unique_file).await;

        // Parse results
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        let (status, test_failures, error_message) = match test_result {
            Ok(Ok(output)) => self.parse_test_output(&output),
            Ok(Err(e)) => (MutantStatus::CompileError, vec![], Some(e.to_string())),
            Err(_) => (
                MutantStatus::Timeout,
                vec![],
                Some("Test execution timed out".to_string()),
            ),
        };

        Ok(MutationResult {
            mutant: mutant.clone(),
            status,
            test_failures,
            execution_time_ms,
            error_message,
        })
    }
}
