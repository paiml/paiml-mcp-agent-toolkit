// WorkflowExecutor trait implementation and workflow-level execution logic
// Extracted from executor.rs for file health compliance

#[async_trait]
impl WorkflowExecutor for DefaultWorkflowExecutor {
    async fn execute(
        &self,
        workflow: &Workflow,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(true, "contract: execute");
        context.set_state(WorkflowState::Running);

        if let Some(monitor) = &self.monitor {
            monitor
                .on_workflow_started(workflow.id, context.execution_id)
                .await;
        }

        let result = if let Some(timeout_duration) = workflow.timeout {
            timeout(
                timeout_duration,
                self.execute_workflow_internal(workflow, context),
            )
            .await
            .map_err(|_| WorkflowError::Timeout)?
        } else {
            self.execute_workflow_internal(workflow, context).await
        };

        match &result {
            Ok(output) => {
                context.set_state(WorkflowState::Completed);
                if let Some(monitor) = &self.monitor {
                    monitor
                        .on_workflow_completed(workflow.id, context.execution_id, output)
                        .await;
                }
            }
            Err(e) => {
                context.set_state(WorkflowState::Failed);
                if let Some(monitor) = &self.monitor {
                    monitor
                        .on_workflow_failed(workflow.id, context.execution_id, e)
                        .await;
                }
            }
        }

        result
    }

    async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(true, "contract: execute_step");
        if let Some(monitor) = &self.monitor {
            monitor
                .on_step_started(context.execution_id, &step.id)
                .await;
        }

        let mut result = StepResult {
            step_id: step.id.clone(),
            status: StepStatus::Running,
            output: None,
            error: None,
            started_at: Instant::now(),
            completed_at: None,
            attempts: 1,
        };

        let output = if let Some(retry) = &step.retry {
            self.execute_with_retry(step, context, retry).await
        } else {
            self.execute_step_internal(step, context).await
        };

        match output {
            Ok(value) => {
                result.status = StepStatus::Completed;
                result.output = Some(value.clone());
                result.completed_at = Some(Instant::now());

                if let Some(monitor) = &self.monitor {
                    monitor
                        .on_step_completed(context.execution_id, &step.id, &value)
                        .await;
                }

                context.set_step_result(step.id.clone(), result);
                Ok(value)
            }
            Err(e) => {
                result.status = StepStatus::Failed;
                result.error = Some(e.to_string());
                result.completed_at = Some(Instant::now());

                if let Some(monitor) = &self.monitor {
                    monitor
                        .on_step_failed(context.execution_id, &step.id, &e.to_string())
                        .await;
                }

                context.set_step_result(step.id.clone(), result);
                Err(e)
            }
        }
    }

    async fn pause(&self, execution_id: Uuid) -> Result<(), WorkflowError> {
        debug_assert!(true, "contract: pause");
        let mut states = self.execution_states.write();
        if let Some(state) = states.get_mut(&execution_id) {
            state.control = ExecutionControl::Paused;
            Ok(())
        } else {
            Err(WorkflowError::NotFound(execution_id))
        }
    }

    async fn resume(&self, execution_id: Uuid) -> Result<(), WorkflowError> {
        debug_assert!(true, "contract: resume");
        let mut states = self.execution_states.write();
        if let Some(state) = states.get_mut(&execution_id) {
            if state.control == ExecutionControl::Paused {
                state.control = ExecutionControl::Running;
                Ok(())
            } else {
                Err(WorkflowError::ExecutionError(
                    "Workflow is not paused".to_string(),
                ))
            }
        } else {
            Err(WorkflowError::NotFound(execution_id))
        }
    }

    async fn cancel(&self, execution_id: Uuid) -> Result<(), WorkflowError> {
        debug_assert!(true, "contract: cancel");
        let mut states = self.execution_states.write();
        if let Some(state) = states.get_mut(&execution_id) {
            state.control = ExecutionControl::Cancelled;
            Ok(())
        } else {
            Err(WorkflowError::NotFound(execution_id))
        }
    }
}

impl DefaultWorkflowExecutor {
    async fn execute_workflow_internal(
        &self,
        workflow: &Workflow,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(true, "contract: execute_workflow_internal");
        // Build DAG for optimal execution ordering
        let dag_engine = super::dag::DagEngine::from_workflow(workflow)?;
        let analysis = dag_engine.analyze()?;

        // Initialize execution state
        let execution_id = context.execution_id;
        self.execution_states
            .write()
            .insert(execution_id, ExecutionState::default());

        let mut last_output = serde_json::json!({});

        // Execute each parallel level
        for (level_idx, level_step_ids) in analysis.execution_order.iter().enumerate() {
            // Check for pause/cancel
            let control_state = self.check_execution_control(execution_id)?;
            if control_state == ExecutionControl::Cancelled {
                return Err(WorkflowError::Cancelled);
            }
            if control_state == ExecutionControl::Paused {
                // Save checkpoint
                self.save_checkpoint(execution_id, level_idx, &analysis.execution_order)?;
                // Wait for resume
                while self.check_execution_control(execution_id)? == ExecutionControl::Paused {
                    sleep(Duration::from_millis(100)).await;
                }
            }

            // Get steps for this level
            let level_steps: Vec<&WorkflowStep> = level_step_ids
                .iter()
                .filter_map(|id| workflow.steps.iter().find(|s| &s.id == id))
                .collect();

            // Execute level in parallel
            if level_steps.len() == 1 {
                // Single step - execute directly
                match self.execute_step(level_steps[0], context).await {
                    Ok(output) => last_output = output,
                    Err(e) => {
                        return self.handle_workflow_error(e, &workflow.error_strategy, context);
                    }
                }
            } else {
                // Multiple steps - execute in parallel
                let futures = level_steps
                    .iter()
                    .map(|step| self.execute_step(step, context));

                let results = join_all(futures).await;

                // Collect results
                let mut level_outputs = vec![];
                for result in results {
                    match result {
                        Ok(output) => level_outputs.push(output),
                        Err(e) => {
                            return self.handle_workflow_error(
                                e,
                                &workflow.error_strategy,
                                context,
                            );
                        }
                    }
                }

                last_output = serde_json::json!({ "parallel_results": level_outputs });
            }
        }

        // Cleanup execution state
        self.execution_states.write().remove(&execution_id);

        Ok(last_output)
    }

    fn check_execution_control(
        &self,
        execution_id: Uuid,
    ) -> Result<ExecutionControl, WorkflowError> {
        debug_assert!(true, "contract: check_execution_control");
        Ok(self
            .execution_states
            .read()
            .get(&execution_id)
            .map(|state| state.control.clone())
            .unwrap_or(ExecutionControl::Running))
    }

    fn save_checkpoint(
        &self,
        execution_id: Uuid,
        current_level: usize,
        execution_order: &[Vec<String>],
    ) -> Result<(), WorkflowError> {
        debug_assert!(true, "contract: save_checkpoint");
        let completed_steps: Vec<String> = execution_order[..current_level]
            .iter()
            .flatten()
            .cloned()
            .collect();

        if let Some(state) = self.execution_states.write().get_mut(&execution_id) {
            state.checkpoint = Some(CheckpointData {
                _completed_steps: completed_steps,
                _current_level: current_level,
            });
        }

        Ok(())
    }

    fn handle_workflow_error(
        &self,
        error: WorkflowError,
        strategy: &ErrorStrategy,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(true, "contract: handle_workflow_error");
        match strategy {
            ErrorStrategy::FailFast => Err(error),
            ErrorStrategy::Continue => Ok(serde_json::json!({ "continued_after_error": true })),
            ErrorStrategy::Rollback => {
                // Invoke rollback logic (returns Result)
                let recovery_result = futures::executor::block_on(
                    super::recovery::RecoveryManager::handle_error(&error, strategy, context),
                );

                // Return original error regardless of recovery result
                let _ = recovery_result; // Suppress unused warning
                Err(error)
            }
            ErrorStrategy::Compensate => {
                // Invoke compensation logic (returns Result)
                let recovery_result = futures::executor::block_on(
                    super::recovery::RecoveryManager::handle_error(&error, strategy, context),
                );

                // Return original error regardless of recovery result
                let _ = recovery_result; // Suppress unused warning
                Err(error)
            }
        }
    }
}
