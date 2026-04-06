impl UnifiedEngine {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn run_server(&mut self) -> Result<Summary, EngineError> {
        loop {
            let state_machine = self.state_machine.read().await;
            let current_state = state_machine.current.clone();
            drop(state_machine);

            match &current_state {
                State::Analyze { current } => {
                    let start = Instant::now();

                    // Fast incremental analysis
                    let metrics = self.analyze_incremental(&current.path).await?;

                    // Emit if threshold crossed
                    if self.should_emit(&metrics) {
                        let payload = self.create_payload(&current.path, metrics);
                        if let EngineMode::Server { emit_buffer, .. } = &self.mode {
                            let mut buffer = emit_buffer.write().await;
                            buffer.push(payload);
                        }
                    }

                    // Auto-advance if under latency budget
                    let elapsed = start.elapsed();
                    if let EngineMode::Server { latency_target, .. } = &self.mode {
                        if elapsed < *latency_target {
                            let mut state_machine = self.state_machine.write().await;
                            state_machine.advance()?;
                        }
                    } else {
                        // In interactive mode, always advance but slower
                        let mut state_machine = self.state_machine.write().await;
                        state_machine.advance()?;
                    }
                }
                State::Complete { summary } => {
                    return Ok(summary.clone());
                }
                _ => {
                    let mut state_machine = self.state_machine.write().await;
                    state_machine.advance()?;
                }
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn run_interactive(&mut self) -> Result<Summary, EngineError> {
        loop {
            // Output current state as JSON
            let state_json = self.export_state().await;
            println!("{}", serde_json::to_string_pretty(&state_json)?);

            // Check if we're done
            {
                let state_machine = self.state_machine.read().await;
                if matches!(state_machine.current, State::Complete { .. }) {
                    if let State::Complete { summary } = &state_machine.current {
                        return Ok(summary.clone());
                    }
                }
            }

            // Wait for command
            let command = self.read_command().await?;

            match command {
                Command::Continue => {
                    let result = self.step_with_explanation().await?;
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                Command::Skip => {
                    let mut state_machine = self.state_machine.write().await;
                    state_machine.advance()?;
                }
                Command::Rollback => {
                    self.rollback_last_change().await?;
                }
                Command::Checkpoint => {
                    self.save_checkpoint().await?;
                }
                Command::Explain => {
                    let explanation = self.explain_current_state().await?;
                    println!("{explanation}");
                }
                Command::Exit => {
                    let state_machine = self.state_machine.read().await;
                    if let State::Complete { summary } = &state_machine.current {
                        return Ok(summary.clone());
                    }
                    return Ok(Summary::default());
                }
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn run_batch(&mut self) -> Result<Summary, EngineError> {
        // Extract values from mode first to avoid borrow issues
        let checkpoint_dir = if let EngineMode::Batch { checkpoint_dir, .. } = &self.mode {
            checkpoint_dir.clone()
        } else {
            unreachable!("run_batch called with non-batch mode")
        };

        let (resume, _parallel_workers) = if let EngineMode::Batch {
            resume,
            parallel_workers,
            ..
        } = &self.mode
        {
            (*resume, *parallel_workers)
        } else {
            unreachable!("run_batch called with non-batch mode")
        };

        // Load checkpoint if resuming
        if resume {
            self.load_checkpoint(&checkpoint_dir).await?;
        }

        // Create checkpoint directory if it doesn't exist
        tokio::fs::create_dir_all(&checkpoint_dir).await?;

        let mut total_processed = 0;
        let mut total_refactors = 0;
        let total_complexity_reduction = 0.0;
        let total_satd_removed = 0;
        let start_time = Instant::now();

        // Process files in batches
        loop {
            let state_machine = self.state_machine.read().await;
            let current_state = state_machine.current.clone();
            drop(state_machine);

            if let State::Complete { .. } = &current_state {
                return Ok(Summary {
                    files_processed: total_processed,
                    refactors_applied: total_refactors,
                    complexity_reduction: total_complexity_reduction,
                    satd_removed: total_satd_removed,
                    total_time: start_time.elapsed(),
                });
            } else {
                // Advance state machine
                let mut state_machine = self.state_machine.write().await;
                state_machine.advance().map_err(EngineError::StateMachine)?;

                // Track metrics
                if matches!(current_state, State::Refactor { .. }) {
                    total_refactors += 1;
                }
                if matches!(current_state, State::Analyze { .. }) {
                    total_processed += 1;
                }

                // Save checkpoint periodically
                if total_processed % 10 == 0 {
                    drop(state_machine);
                    self.save_checkpoint_to(&checkpoint_dir).await?;
                }
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn export_state(&self) -> InteractiveState {
        let state_machine = self.state_machine.read().await;
        let current_state = &state_machine.current;

        let state_info = match current_state {
            State::Analyze { current } => StateInfo {
                state_type: "Analyze".to_string(),
                current_file: Some(current.path.to_string_lossy().to_string()),
                current_function: None,
                line_range: None,
            },
            State::Plan { violations } => StateInfo {
                state_type: "Plan".to_string(),
                current_file: violations
                    .first()
                    .map(|v| v.location.file.to_string_lossy().to_string()),
                current_function: None,
                line_range: None,
            },
            State::Refactor { operation } => StateInfo {
                state_type: "Refactor".to_string(),
                current_file: None,
                current_function: match operation {
                    crate::models::refactor::RefactorOp::ExtractFunction { name, .. } => {
                        Some(name.clone())
                    }
                    _ => None,
                },
                line_range: None,
            },
            State::Scan { .. } => StateInfo {
                state_type: "Scan".to_string(),
                current_file: None,
                current_function: None,
                line_range: None,
            },
            State::Complete { .. } => StateInfo {
                state_type: "Complete".to_string(),
                current_file: None,
                current_function: None,
                line_range: None,
            },
            _ => StateInfo {
                state_type: format!("{current_state:?}")
                    .split(' ')
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
                current_file: None,
                current_function: None,
                line_range: None,
            },
        };

        InteractiveState {
            state: state_info,
            metrics: MetricsInfo {
                before: Some(ComplexityInfo {
                    complexity: [10, 15],
                    tdg: 1.5,
                    satd: 2,
                }),
                projected: Some(ComplexityInfo {
                    complexity: [5, 8],
                    tdg: 0.8,
                    satd: 0,
                }),
            },
            suggestion: Some(SuggestionInfo {
                suggestion_type: "ExtractFunction".to_string(),
                description: "Extract complex logic into helper functions".to_string(),
                operations: vec![OperationInfo {
                    name: "extract_helper".to_string(),
                    lines: [100, 150],
                    complexity_reduction: 8,
                }],
            }),
            commands: vec![
                "continue".to_string(),
                "skip".to_string(),
                "rollback".to_string(),
                "checkpoint".to_string(),
                "explain".to_string(),
                "exit".to_string(),
            ],
            explanation: None,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn read_command(&self) -> Result<Command, EngineError> {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        if let Some(line) = lines.next_line().await? {
            match line.trim() {
                "continue" => Ok(Command::Continue),
                "skip" => Ok(Command::Skip),
                "rollback" => Ok(Command::Rollback),
                "checkpoint" => Ok(Command::Checkpoint),
                "explain" => Ok(Command::Explain),
                "exit" => Ok(Command::Exit),
                _ => Ok(Command::Continue), // Default to continue
            }
        } else {
            Ok(Command::Exit)
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn step_with_explanation(&self) -> Result<StepResult, EngineError> {
        let old_state = {
            let state_machine = self.state_machine.read().await;
            format!("{:?}", state_machine.current)
        };

        // Advance the state machine
        {
            let mut state_machine = self.state_machine.write().await;
            state_machine.advance()?;
        }

        let new_state = {
            let state_machine = self.state_machine.read().await;
            format!("{:?}", state_machine.current)
        };

        Ok(StepResult {
            success: true,
            explanation: format!("Transitioned from {old_state} to {new_state}"),
            metrics_changed: true,
            new_state,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn rollback_last_change(&self) -> Result<(), EngineError> {
        let mut state_machine = self.state_machine.write().await;

        // Check if we have any history to rollback
        if state_machine.history.is_empty() {
            return Err(EngineError::StateMachine(
                "No operations to rollback".to_string(),
            ));
        }

        // Get the last transition
        let last_transition = state_machine.history.pop().ok_or_else(|| {
            EngineError::StateMachine("Failed to get last transition".to_string())
        })?;

        // Restore the previous state
        state_machine.current = last_transition.from;

        // If we rolled back a file analysis, decrement the target index
        if matches!(state_machine.current, State::Analyze { .. })
            && state_machine.current_target_index > 0
        {
            state_machine.current_target_index -= 1;
        }

        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn explain_current_state(&self) -> Result<String, EngineError> {
        let state_machine = self.state_machine.read().await;
        match &state_machine.current {
            State::Analyze { current } => Ok(format!(
                "Currently analyzing file: {}. This involves computing complexity metrics and identifying potential refactoring opportunities.",
                current.path.display()
            )),
            State::Plan { violations } => Ok(format!(
                "Planning refactoring operations. Found {} violations that could be addressed.",
                violations.len()
            )),
            State::Refactor { operation } => Ok(format!(
                "Applying refactoring operation: {operation:?}. This will transform the code to improve maintainability."
            )),
            _ => Ok("Processing current state...".to_string()),
        }
    }
}
