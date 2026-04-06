#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

// Fluent DSL for workflow creation
pub struct FluentWorkflow {
    builder: WorkflowBuilder,
}

impl FluentWorkflow {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn define(name: impl Into<String>) -> Self {
        Self {
            builder: WorkflowBuilder::new(name),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn then(self, step: WorkflowStep) -> Self {
        Self {
            builder: self.builder.add_step(step),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parallel(self, steps: Vec<WorkflowStep>) -> Self {
        let parallel_step = WorkflowStep {
            id: format!("parallel_{}", uuid::Uuid::new_v4()),
            name: "Parallel Execution".to_string(),
            step_type: StepType::Parallel { steps },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        self.then(parallel_step)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn when(self, condition: impl Into<String>) -> ConditionalFlow {
        ConditionalFlow {
            workflow: self,
            condition: condition.into(),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn repeat(self, condition: impl Into<String>, step: WorkflowStep) -> Self {
        let loop_step = WorkflowStep {
            id: format!("loop_{}", uuid::Uuid::new_v4()),
            name: "Loop".to_string(),
            step_type: StepType::Loop {
                condition: condition.into(),
                step: Box::new(step),
                max_iterations: None,
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        self.then(loop_step)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn on_error(mut self, strategy: ErrorStrategy) -> Self {
        self.builder = self.builder.error_strategy(strategy);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.timeout(timeout);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build(self) -> Workflow {
        self.builder.build()
    }
}

pub struct ConditionalFlow {
    workflow: FluentWorkflow,
    condition: String,
}

impl ConditionalFlow {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn do_this(self, step: WorkflowStep) -> ConditionalElse {
        ConditionalElse {
            workflow: self.workflow,
            condition: self.condition,
            if_true: step,
        }
    }
}

pub struct ConditionalElse {
    workflow: FluentWorkflow,
    condition: String,
    if_true: WorkflowStep,
}

impl ConditionalElse {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn otherwise(self, step: WorkflowStep) -> FluentWorkflow {
        let conditional = WorkflowStep {
            id: format!("cond_{}", uuid::Uuid::new_v4()),
            name: "Conditional".to_string(),
            step_type: StepType::Conditional {
                condition: self.condition,
                if_true: Box::new(self.if_true),
                if_false: Some(Box::new(step)),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        self.workflow.then(conditional)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn end_if(self) -> FluentWorkflow {
        let conditional = WorkflowStep {
            id: format!("cond_{}", uuid::Uuid::new_v4()),
            name: "Conditional".to_string(),
            step_type: StepType::Conditional {
                condition: self.condition,
                if_true: Box::new(self.if_true),
                if_false: None,
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        self.workflow.then(conditional)
    }
}
