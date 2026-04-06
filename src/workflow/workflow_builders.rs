// Workflow builder for programmatic workflow creation
pub struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            workflow: Workflow {
                id: Uuid::new_v4(),
                name: name.into(),
                description: None,
                version: "1.0.0".to_string(),
                steps: Vec::new(),
                error_strategy: ErrorStrategy::FailFast,
                timeout: None,
                metadata: HashMap::new(),
            },
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.workflow.description = Some(desc.into());
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.workflow.version = version.into();
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_step(mut self, step: WorkflowStep) -> Self {
        self.workflow.steps.push(step);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn error_strategy(mut self, strategy: ErrorStrategy) -> Self {
        self.workflow.error_strategy = strategy;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.workflow.timeout = Some(timeout);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.workflow.metadata.insert(key.into(), value);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build(self) -> Workflow {
        self.workflow
    }
}

// Step builder
pub struct StepBuilder {
    step: WorkflowStep,
}

impl StepBuilder {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn action(
        id: impl Into<String>,
        name: impl Into<String>,
        agent: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self {
            step: WorkflowStep {
                id: id.into(),
                name: name.into(),
                step_type: StepType::Action {
                    agent: agent.into(),
                    operation: operation.into(),
                    params: Value::Object(serde_json::Map::new()),
                },
                condition: None,
                retry: None,
                timeout: None,
                on_error: None,
                metadata: HashMap::new(),
            },
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn params(mut self, new_params: Value) -> Self {
        if let StepType::Action { params, .. } = &mut self.step.step_type {
            *params = new_params;
        }
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn condition(mut self, expression: impl Into<String>, skip_on_false: bool) -> Self {
        self.step.condition = Some(StepCondition {
            expression: expression.into(),
            skip_on_false,
        });
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn retry(mut self, max_attempts: usize, backoff: BackoffStrategy) -> Self {
        self.step.retry = Some(RetryPolicy {
            max_attempts,
            backoff,
            retry_on: vec![],
        });
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.step.timeout = Some(timeout);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn on_error(mut self, handler: ErrorHandler) -> Self {
        self.step.on_error = Some(handler);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build(self) -> WorkflowStep {
        self.step
    }
}
