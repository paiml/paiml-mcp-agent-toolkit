// include!()'d from executor.rs
// Resumable mutant execution with signal handling

async fn load_or_create_state(
    executor: &MutantExecutor,
    mutants: &[Mutant],
    state_path: &Path,
) -> Result<super::state::MutationState> {
    debug_assert!(state_path.exists(), "state_path must exist: {}", state_path.display());
    if state_path.exists() {
        println!("  📋 Found existing state at {}", state_path.display());
        super::state::MutationState::load(state_path).await
    } else {
        if let Some(parent) = state_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }
        println!("  📋 Creating new state file at {}", state_path.display());
        let state = super::state::MutationState::new(
            &executor.work_dir,
            mutants.to_vec(),
            executor.timeout.as_secs(),
            false,
            None,
        );
        state.save(state_path).await?;
        Ok(state)
    }
}

fn format_mutant_status(result: &MutationResult) -> &'static str {
    match result.status {
        MutantStatus::Killed => "✅",
        MutantStatus::Survived => "❌",
        MutantStatus::CompileError => "🔧",
        MutantStatus::Timeout => "⏱️",
        _ => "❓",
    }
}

fn print_final_stats(results: &[MutationResult]) {
    debug_assert!(!results.is_empty(), "results must not be empty");
    println!(
        "  ✅ Mutation testing complete! Processed {} mutants",
        results.len()
    );
    println!(
        "  📊 Stats: {} killed, {} survived, {} errors",
        results.iter().filter(|r| r.status == MutantStatus::Killed).count(),
        results.iter().filter(|r| r.status == MutantStatus::Survived).count(),
        results.iter().filter(|r| r.status == MutantStatus::CompileError || r.status == MutantStatus::Timeout).count()
    );
}

impl MutantExecutor {
    /// Execute mutants with resumable testing
    ///
    /// This function saves the state to disk periodically to allow
    /// resuming an interrupted test run.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn execute_mutants_resumable(
        &self,
        mutants: &[Mutant],
        state_path: &Path,
        save_interval: Duration,
    ) -> Result<Vec<MutationResult>> {
        debug_assert!(state_path.exists(), "state_path must exist: {}", state_path.display());
        let mut state = load_or_create_state(self, mutants, state_path).await?;

        // Handle case where everything is already done
        if state.is_complete() {
            println!("  ✅ All mutants already tested!");
            return Ok(state.completed_mutants);
        }

        println!(
            "  🚀 Resuming mutation testing with {} mutants to process ({:.1}% complete)",
            state.pending_mutants.len(),
            state.completion_percentage()
        );

        // Set up periodic state saving
        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

        // Spawn save task
        let state_path_clone = state_path.to_path_buf();
        let state_clone = state.clone();
        let save_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(save_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Save state
                        if let Err(e) = state_clone.save_with_backup(&state_path_clone).await {
                            eprintln!("  ⚠️  Error saving state: {}", e);
                        } else {
                            println!("  💾 Saved progress ({:.1}% complete)",
                                state_clone.completion_percentage());
                        }
                    }
                    _ = rx.recv() => {
                        // Final save on completion
                        if let Err(e) = state_clone.save_with_backup(&state_path_clone).await {
                            eprintln!("  ⚠️  Error saving final state: {}", e);
                        }
                        break;
                    }
                }
            }
        });

        // Set up signal handler
        let (signal_tx, mut signal_rx) = tokio::sync::mpsc::channel::<()>(1);

        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let signal_tx_clone = signal_tx.clone();

            // Handle SIGINT
            tokio::spawn(async move {
                let mut sigint =
                    signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");

                sigint.recv().await;
                println!("\n  🛑 Received interrupt signal, stopping gracefully...");
                let _ = signal_tx_clone.send(()).await;
            });

            // Handle SIGTERM
            let signal_tx_clone = signal_tx.clone();
            tokio::spawn(async move {
                let mut sigterm =
                    signal(SignalKind::terminate()).expect("Failed to set up SIGTERM handler");

                sigterm.recv().await;
                println!("\n  🛑 Received termination signal, stopping gracefully...");
                let _ = signal_tx_clone.send(()).await;
            });
        }

        // Process mutants
        let mut results = state.completed_mutants.clone();

        // First clone the pending mutants to avoid the borrow conflict
        let pending_mutants = state.pending_mutants.clone();

        'outer: for mutant in pending_mutants.iter() {
            // Check for signal
            if let Ok(()) = signal_rx.try_recv() {
                println!("  🛑 Graceful shutdown requested, stopping execution");
                break 'outer;
            }

            println!("  [{}] Testing mutant {}...", results.len() + 1, mutant.id);

            match self.execute_mutant(mutant).await {
                Ok(result) => {
                    println!(
                        "    {} {:?} ({}ms)",
                        format_mutant_status(&result), result.status, result.execution_time_ms
                    );

                    // Add to state and results
                    {
                        // Use a scope to drop the mutable borrow before continuing
                        state.add_result(result.clone());
                    }
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("    ⚠️  Error executing mutant {}: {}", mutant.id, e);

                    // Create error result
                    let error_result = MutationResult {
                        mutant: mutant.clone(),
                        status: MutantStatus::CompileError,
                        test_failures: vec![],
                        execution_time_ms: 0,
                        error_message: Some(e.to_string()),
                    };

                    // Add to state and results
                    {
                        // Use a scope to drop the mutable borrow before continuing
                        state.add_result(error_result.clone());
                    }
                    results.push(error_result);
                }
            }
        }

        // Signal save task to complete
        let _ = tx.send(()).await;

        // Wait for save task
        if let Err(e) = save_task.await {
            eprintln!("  ⚠️  Error in state saving task: {}", e);
        }

        print_final_stats(&results);

        Ok(results)
    }

    /// Get default state path for a project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn get_default_state_path(&self) -> PathBuf {
        self.work_dir.join(".pmat").join("mutation_state.json")
    }
}
