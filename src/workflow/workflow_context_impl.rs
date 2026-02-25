impl WorkflowContext {
    pub fn new(
        workflow_id: Uuid,
        agent_registry: Arc<crate::agents::registry::AgentRegistry>,
    ) -> Self {
        Self {
            workflow_id,
            execution_id: Uuid::new_v4(),
            variables: Arc::new(RwLock::new(HashMap::new())),
            step_results: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(WorkflowState::Created)),
            started_at: Instant::now(),
            agent_registry,
        }
    }

    pub fn set_variable(&self, name: String, value: Value) {
        self.variables.write().insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<Value> {
        self.variables.read().get(name).cloned()
    }

    pub fn set_step_result(&self, step_id: String, result: StepResult) {
        self.step_results.write().insert(step_id, result);
    }

    pub fn get_step_result(&self, step_id: &str) -> Option<StepResult> {
        self.step_results.read().get(step_id).cloned()
    }

    pub fn set_state(&self, state: WorkflowState) {
        *self.state.write() = state;
    }

    pub fn get_state(&self) -> WorkflowState {
        *self.state.read()
    }

    pub fn get_elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

// Workflow executor trait
#[async_trait]
pub trait WorkflowExecutor: Send + Sync {
    async fn execute(
        &self,
        workflow: &Workflow,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError>;
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError>;
    async fn pause(&self, execution_id: Uuid) -> Result<(), WorkflowError>;
    async fn resume(&self, execution_id: Uuid) -> Result<(), WorkflowError>;
    async fn cancel(&self, execution_id: Uuid) -> Result<(), WorkflowError>;
}

// Workflow repository
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    async fn save(&self, workflow: &Workflow) -> Result<(), WorkflowError>;
    async fn get(&self, id: Uuid) -> Result<Option<Workflow>, WorkflowError>;
    async fn list(&self) -> Result<Vec<Workflow>, WorkflowError>;
    async fn delete(&self, id: Uuid) -> Result<(), WorkflowError>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Workflow>, WorkflowError>;
}

// Workflow monitor
#[async_trait]
pub trait WorkflowMonitor: Send + Sync {
    async fn on_workflow_started(&self, workflow_id: Uuid, execution_id: Uuid);
    async fn on_workflow_completed(&self, workflow_id: Uuid, execution_id: Uuid, result: &Value);
    async fn on_workflow_failed(
        &self,
        workflow_id: Uuid,
        execution_id: Uuid,
        error: &WorkflowError,
    );
    async fn on_step_started(&self, execution_id: Uuid, step_id: &str);
    async fn on_step_completed(&self, execution_id: Uuid, step_id: &str, result: &Value);
    async fn on_step_failed(&self, execution_id: Uuid, step_id: &str, error: &str);
    async fn get_metrics(&self, execution_id: Uuid) -> WorkflowMetrics;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub execution_id: Uuid,
    pub workflow_id: Uuid,
    pub state: WorkflowState,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub skipped_steps: usize,
    pub elapsed_time: Duration,
    pub average_step_time: Option<Duration>,
    pub retry_count: usize,
}
