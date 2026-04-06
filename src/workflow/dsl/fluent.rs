#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

// Fluent DSL for workflow creation
pub struct FluentWorkflow {
    builder: WorkflowBuilder,
}

impl FluentWorkflow {
    pub fn define(name: impl Into<String>) -> Self {
        Self {
            builder: WorkflowBuilder::new(name),
        }
    }

    pub fn then(self, step: WorkflowStep) -> Self {
        Self {
            builder: self.builder.add_step(step),
        }
    }

    pub fn parallel(self, steps: Vec<WorkflowStep>) -> Self {
        debug_assert!(!steps.is_empty(), "steps must not be empty");
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

    pub fn when(self, condition: impl Into<String>) -> ConditionalFlow {
        ConditionalFlow {
            workflow: self,
            condition: condition.into(),
        }
    }

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

    pub fn on_error(mut self, strategy: ErrorStrategy) -> Self {
        self.builder = self.builder.error_strategy(strategy);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.timeout(timeout);
        self
    }

    pub fn build(self) -> Workflow {
        self.builder.build()
    }
}

pub struct ConditionalFlow {
    workflow: FluentWorkflow,
    condition: String,
}

impl ConditionalFlow {
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
