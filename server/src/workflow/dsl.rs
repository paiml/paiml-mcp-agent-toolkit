use super::*;
use pest::Parser;
use pest_derive::Parser;
use std::str::FromStr;

// Workflow DSL parser
#[derive(Parser)]
#[grammar = "workflow/workflow.pest"]
pub struct WorkflowParser;

// DSL compiler
pub struct DslCompiler;

impl DslCompiler {
    pub fn compile(source: &str) -> Result<Workflow, WorkflowError> {
        // For now, use YAML/JSON parsing
        serde_yaml::from_str(source)
            .or_else(|_| serde_json::from_str(source))
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))
    }

    pub fn compile_step(source: &str) -> Result<WorkflowStep, WorkflowError> {
        serde_yaml::from_str(source)
            .or_else(|_| serde_json::from_str(source))
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))
    }
}

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

// Macro for simplified DSL syntax
#[macro_export]
macro_rules! workflow {
    ($name:expr => { $($step:expr),* $(,)? }) => {{
        let mut wf = $crate::workflow::dsl::FluentWorkflow::define($name);
        $(
            wf = wf.then($step);
        )*
        wf.build()
    }};
}

#[macro_export]
macro_rules! step {
    (action: $agent:expr, $op:expr, { $($key:ident: $value:expr),* $(,)? }) => {{
        $crate::workflow::StepBuilder::action(
            format!("step_{}", uuid::Uuid::new_v4()),
            format!("{}.{}", $agent, $op),
            $agent,
            $op,
        )
        .params(serde_json::json!({ $($key: $value),* }))
        .build()
    }};
    
    (wait: $duration:expr) => {{
        $crate::workflow::WorkflowStep {
            id: format!("wait_{}", uuid::Uuid::new_v4()),
            name: "Wait".to_string(),
            step_type: $crate::workflow::StepType::Wait {
                duration: $duration,
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: std::collections::HashMap::new(),
        }
    }};
}

// YAML-based DSL example
pub const WORKFLOW_DSL_EXAMPLE: &str = r#"
name: quality_check_workflow
version: 1.0.0
description: Comprehensive quality check workflow
error_strategy: fail_fast
timeout: 300s

steps:
  - id: analyze
    name: Code Analysis
    type: action
    agent: analyzer
    operation: analyze
    params:
      language: rust
      metrics: [complexity, satd, entropy]
    retry:
      max_attempts: 3
      backoff:
        type: exponential
        initial: 1s
        multiplier: 2
        max: 10s

  - id: check_quality
    name: Quality Gate Check
    type: conditional
    condition: "steps.analyze.output.complexity < 10"
    if_true:
      id: transform
      name: Apply Transformations
      type: action
      agent: transformer
      operation: optimize
      params:
        level: aggressive
    if_false:
      id: alert
      name: Quality Alert
      type: action
      agent: notifier
      operation: alert
      params:
        severity: high
        message: "Code complexity exceeds threshold"

  - id: validate
    name: Final Validation
    type: action
    agent: validator
    operation: validate
    condition:
      expression: "steps.transform.status == 'completed'"
      skip_on_false: true
    on_error: skip
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fluent_dsl() {
        let workflow = FluentWorkflow::define("test_workflow")
            .then(step!(action: "analyzer", "analyze", {
                language: "rust"
            }))
            .then(step!(wait: Duration::from_secs(5)))
            .then(step!(action: "validator", "validate", {}))
            .on_error(ErrorStrategy::Continue)
            .with_timeout(Duration::from_secs(60))
            .build();

        assert_eq!(workflow.name, "test_workflow");
        assert_eq!(workflow.steps.len(), 3);
    }

    #[test]
    fn test_conditional_flow() {
        let analyze_step = step!(action: "analyzer", "analyze", {});
        let transform_step = step!(action: "transformer", "optimize", {});
        let validate_step = step!(action: "validator", "validate", {});

        let workflow = FluentWorkflow::define("conditional_workflow")
            .then(analyze_step)
            .when("result.score > 0.8")
                .do_this(transform_step)
                .otherwise(validate_step)
            .build();

        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_yaml_dsl_compilation() {
        let result = DslCompiler::compile(WORKFLOW_DSL_EXAMPLE);
        assert!(result.is_ok());
        
        let workflow = result.unwrap();
        assert_eq!(workflow.name, "quality_check_workflow");
        assert_eq!(workflow.version, "1.0.0");
    }

    #[test]
    fn test_workflow_macro() {
        let wf = workflow!("macro_workflow" => {
            step!(action: "analyzer", "analyze", { language: "rust" }),
            step!(wait: Duration::from_secs(2)),
            step!(action: "validator", "validate", {}),
        });

        assert_eq!(wf.name, "macro_workflow");
        assert_eq!(wf.steps.len(), 3);
    }
}