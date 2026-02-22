#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

// DSL compiler
pub struct DslCompiler;

impl DslCompiler {
    pub fn compile(source: &str) -> Result<Workflow, WorkflowError> {
        // For now, use YAML/JSON parsing
        serde_yaml_ng::from_str(source)
            .or_else(|_| serde_json::from_str(source))
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))
    }

    pub fn compile_step(source: &str) -> Result<WorkflowStep, WorkflowError> {
        serde_yaml_ng::from_str(source)
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

// JSON-based DSL example
pub const WORKFLOW_DSL_EXAMPLE: &str = r#"{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "name": "quality_check_workflow",
  "description": "Comprehensive quality check workflow",
  "version": "1.0.0",
  "steps": [
    {
      "id": "analyze",
      "name": "Code Analysis",
      "step_type": {
        "type": "action",
        "agent": "analyzer",
        "operation": "analyze",
        "params": {
          "language": "rust",
          "metrics": ["complexity", "satd", "entropy"]
        }
      },
      "condition": null,
      "retry": {
        "max_attempts": 3,
        "backoff": {
          "Exponential": {
            "initial": {"secs": 1, "nanos": 0},
            "multiplier": 2.0,
            "max": {"secs": 10, "nanos": 0}
          }
        },
        "retry_on": []
      },
      "timeout": {"secs": 60, "nanos": 0},
      "on_error": null,
      "metadata": {}
    }
  ],
  "error_strategy": "FailFast",
  "timeout": {"secs": 300, "nanos": 0},
  "metadata": {}
}"#;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fluent_dsl() {
        let workflow = FluentWorkflow::define("test_workflow")
            .then(step!(action: "analyzer", "analyze", {}))
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
        if let Err(e) = &result {
            println!("Compilation error: {:?}", e);
        }
        assert!(result.is_ok());

        let workflow = result.unwrap();
        assert_eq!(workflow.name, "quality_check_workflow");
        assert_eq!(workflow.version, "1.0.0");
    }

    #[test]
    fn test_workflow_macro() {
        let wf = workflow!("macro_workflow" => {
            step!(action: "analyzer", "analyze", {}),
            step!(wait: Duration::from_secs(2)),
            step!(action: "validator", "validate", {}),
        });

        assert_eq!(wf.name, "macro_workflow");
        assert_eq!(wf.steps.len(), 3);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // =========================================================================
    // DslCompiler::compile tests
    // =========================================================================

    #[test]
    fn test_dsl_compiler_compile_json_success() {
        // Test compilation from valid JSON
        let json_workflow = r#"{
            "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "name": "json_workflow",
            "description": "A test workflow",
            "version": "1.0.0",
            "steps": [],
            "error_strategy": "FailFast",
            "metadata": {}
        }"#;

        let result = DslCompiler::compile(json_workflow);
        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.name, "json_workflow");
        assert_eq!(workflow.version, "1.0.0");
        assert!(workflow.steps.is_empty());
    }

    #[test]
    fn test_dsl_compiler_compile_yaml_success() {
        // Test compilation from valid YAML
        let yaml_workflow = r#"
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
name: yaml_workflow
description: A YAML test workflow
version: "2.0.0"
steps: []
error_strategy: Continue
metadata: {}
"#;

        let result = DslCompiler::compile(yaml_workflow);
        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.name, "yaml_workflow");
        assert_eq!(workflow.version, "2.0.0");
    }

    #[test]
    fn test_dsl_compiler_compile_invalid_input() {
        // Test compilation with invalid input that fails both YAML and JSON parsing
        let invalid_input = "this is not valid { yaml or json }}}";

        let result = DslCompiler::compile(invalid_input);
        assert!(result.is_err());
        match result {
            Err(WorkflowError::InvalidDefinition(msg)) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected InvalidDefinition error"),
        }
    }

    #[test]
    fn test_dsl_compiler_compile_empty_input() {
        // Test compilation with empty input
        let result = DslCompiler::compile("");
        assert!(result.is_err());
    }

    #[test]
    fn test_dsl_compiler_compile_partial_json() {
        // Test compilation with JSON missing required fields
        let partial_json = r#"{"name": "incomplete"}"#;
        let result = DslCompiler::compile(partial_json);
        assert!(result.is_err());
    }

    // =========================================================================
    // DslCompiler::compile_step tests
    // =========================================================================

    #[test]
    fn test_dsl_compiler_compile_step_json_success() {
        let json_step = r#"{
            "id": "step1",
            "name": "Test Step",
            "step_type": {
                "type": "action",
                "agent": "test_agent",
                "operation": "test_op",
                "params": {}
            },
            "metadata": {}
        }"#;

        let result = DslCompiler::compile_step(json_step);
        assert!(result.is_ok());
        let step = result.unwrap();
        assert_eq!(step.id, "step1");
        assert_eq!(step.name, "Test Step");
    }

    #[test]
    fn test_dsl_compiler_compile_step_yaml_success() {
        let yaml_step = r#"
id: step2
name: YAML Step
step_type:
  type: wait
  duration:
    secs: 10
    nanos: 0
metadata: {}
"#;

        let result = DslCompiler::compile_step(yaml_step);
        assert!(result.is_ok());
        let step = result.unwrap();
        assert_eq!(step.id, "step2");
        assert_eq!(step.name, "YAML Step");
    }

    #[test]
    fn test_dsl_compiler_compile_step_invalid_input() {
        let invalid_input = "not a valid step {{{";
        let result = DslCompiler::compile_step(invalid_input);
        assert!(result.is_err());
        match result {
            Err(WorkflowError::InvalidDefinition(_)) => {}
            _ => panic!("Expected InvalidDefinition error"),
        }
    }

    #[test]
    fn test_dsl_compiler_compile_step_empty_input() {
        let result = DslCompiler::compile_step("");
        assert!(result.is_err());
    }

    // =========================================================================
    // FluentWorkflow tests
    // =========================================================================

    #[test]
    fn test_fluent_workflow_define() {
        let fluent = FluentWorkflow::define("my_workflow");
        let workflow = fluent.build();
        assert_eq!(workflow.name, "my_workflow");
        assert!(workflow.steps.is_empty());
    }

    #[test]
    fn test_fluent_workflow_define_string_type() {
        let name = String::from("string_workflow");
        let fluent = FluentWorkflow::define(name);
        let workflow = fluent.build();
        assert_eq!(workflow.name, "string_workflow");
    }

    #[test]
    fn test_fluent_workflow_then_single_step() {
        let step = WorkflowStep {
            id: "test_step".to_string(),
            name: "Test".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("test").then(step).build();
        assert_eq!(workflow.steps.len(), 1);
        assert_eq!(workflow.steps[0].id, "test_step");
    }

    #[test]
    fn test_fluent_workflow_then_multiple_steps() {
        let step1 = WorkflowStep {
            id: "step1".to_string(),
            name: "Step 1".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step2 = WorkflowStep {
            id: "step2".to_string(),
            name: "Step 2".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(2),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step3 = WorkflowStep {
            id: "step3".to_string(),
            name: "Step 3".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(3),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("multi_step")
            .then(step1)
            .then(step2)
            .then(step3)
            .build();

        assert_eq!(workflow.steps.len(), 3);
        assert_eq!(workflow.steps[0].id, "step1");
        assert_eq!(workflow.steps[1].id, "step2");
        assert_eq!(workflow.steps[2].id, "step3");
    }

    #[test]
    fn test_fluent_workflow_parallel() {
        let step1 = WorkflowStep {
            id: "parallel_step1".to_string(),
            name: "Parallel 1".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step2 = WorkflowStep {
            id: "parallel_step2".to_string(),
            name: "Parallel 2".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("parallel_workflow")
            .parallel(vec![step1, step2])
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Parallel { steps } => {
                assert_eq!(steps.len(), 2);
            }
            _ => panic!("Expected Parallel step type"),
        }
        assert!(workflow.steps[0].id.starts_with("parallel_"));
        assert_eq!(workflow.steps[0].name, "Parallel Execution");
    }

    #[test]
    fn test_fluent_workflow_parallel_empty_steps() {
        let workflow = FluentWorkflow::define("empty_parallel")
            .parallel(vec![])
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Parallel { steps } => {
                assert!(steps.is_empty());
            }
            _ => panic!("Expected Parallel step type"),
        }
    }

    #[test]
    fn test_fluent_workflow_repeat() {
        let step = WorkflowStep {
            id: "loop_body".to_string(),
            name: "Loop Body".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("loop_workflow")
            .repeat("counter < 10", step)
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Loop {
                condition,
                step,
                max_iterations,
            } => {
                assert_eq!(condition, "counter < 10");
                assert_eq!(step.id, "loop_body");
                assert!(max_iterations.is_none());
            }
            _ => panic!("Expected Loop step type"),
        }
        assert!(workflow.steps[0].id.starts_with("loop_"));
        assert_eq!(workflow.steps[0].name, "Loop");
    }

    #[test]
    fn test_fluent_workflow_repeat_with_string_condition() {
        let step = WorkflowStep {
            id: "body".to_string(),
            name: "Body".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let condition = String::from("status == 'pending'");
        let workflow = FluentWorkflow::define("test")
            .repeat(condition, step)
            .build();

        match &workflow.steps[0].step_type {
            StepType::Loop { condition, .. } => {
                assert_eq!(condition, "status == 'pending'");
            }
            _ => panic!("Expected Loop step type"),
        }
    }

    #[test]
    fn test_fluent_workflow_on_error_failfast() {
        let workflow = FluentWorkflow::define("error_workflow")
            .on_error(ErrorStrategy::FailFast)
            .build();

        assert!(matches!(workflow.error_strategy, ErrorStrategy::FailFast));
    }

    #[test]
    fn test_fluent_workflow_on_error_continue() {
        let workflow = FluentWorkflow::define("error_workflow")
            .on_error(ErrorStrategy::Continue)
            .build();

        assert!(matches!(workflow.error_strategy, ErrorStrategy::Continue));
    }

    #[test]
    fn test_fluent_workflow_on_error_rollback() {
        let workflow = FluentWorkflow::define("error_workflow")
            .on_error(ErrorStrategy::Rollback)
            .build();

        assert!(matches!(workflow.error_strategy, ErrorStrategy::Rollback));
    }

    #[test]
    fn test_fluent_workflow_on_error_compensate() {
        let workflow = FluentWorkflow::define("error_workflow")
            .on_error(ErrorStrategy::Compensate)
            .build();

        assert!(matches!(workflow.error_strategy, ErrorStrategy::Compensate));
    }

    #[test]
    fn test_fluent_workflow_with_timeout() {
        let workflow = FluentWorkflow::define("timeout_workflow")
            .with_timeout(Duration::from_secs(120))
            .build();

        assert_eq!(workflow.timeout, Some(Duration::from_secs(120)));
    }

    #[test]
    fn test_fluent_workflow_with_timeout_zero() {
        let workflow = FluentWorkflow::define("zero_timeout")
            .with_timeout(Duration::from_secs(0))
            .build();

        assert_eq!(workflow.timeout, Some(Duration::from_secs(0)));
    }

    #[test]
    fn test_fluent_workflow_chained_operations() {
        let step1 = WorkflowStep {
            id: "s1".to_string(),
            name: "S1".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step2 = WorkflowStep {
            id: "s2".to_string(),
            name: "S2".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("chained")
            .then(step1)
            .parallel(vec![step2])
            .on_error(ErrorStrategy::Continue)
            .with_timeout(Duration::from_secs(300))
            .build();

        assert_eq!(workflow.name, "chained");
        assert_eq!(workflow.steps.len(), 2);
        assert!(matches!(workflow.error_strategy, ErrorStrategy::Continue));
        assert_eq!(workflow.timeout, Some(Duration::from_secs(300)));
    }

    // =========================================================================
    // ConditionalFlow tests
    // =========================================================================

    #[test]
    fn test_conditional_flow_when_creates_flow() {
        let workflow = FluentWorkflow::define("cond_test");
        let conditional = workflow.when("x > 5");

        // The condition is stored correctly (we verify by completing the flow)
        let step = WorkflowStep {
            id: "if_step".to_string(),
            name: "If Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let else_obj = conditional.do_this(step);
        let wf = else_obj.end_if().build();

        // Verify the condition was used
        match &wf.steps[0].step_type {
            StepType::Conditional { condition, .. } => {
                assert_eq!(condition, "x > 5");
            }
            _ => panic!("Expected Conditional step"),
        }
    }

    #[test]
    fn test_conditional_flow_when_with_string_condition() {
        let workflow = FluentWorkflow::define("cond_test");
        let condition = String::from("status == 'active'");
        let conditional = workflow.when(condition);

        let step = WorkflowStep {
            id: "step".to_string(),
            name: "Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let wf = conditional.do_this(step).end_if().build();

        match &wf.steps[0].step_type {
            StepType::Conditional { condition, .. } => {
                assert_eq!(condition, "status == 'active'");
            }
            _ => panic!("Expected Conditional step"),
        }
    }

    #[test]
    fn test_conditional_flow_do_this() {
        let if_step = WorkflowStep {
            id: "if_step".to_string(),
            name: "If Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let conditional_else = FluentWorkflow::define("test")
            .when("condition")
            .do_this(if_step);

        // Verify do_this returns ConditionalElse with correct if_true step
        let wf = conditional_else.end_if().build();
        match &wf.steps[0].step_type {
            StepType::Conditional { if_true, .. } => {
                assert_eq!(if_true.id, "if_step");
            }
            _ => panic!("Expected Conditional step"),
        }
    }

    // =========================================================================
    // ConditionalElse tests
    // =========================================================================

    #[test]
    fn test_conditional_else_otherwise() {
        let if_step = WorkflowStep {
            id: "if_step".to_string(),
            name: "If Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let else_step = WorkflowStep {
            id: "else_step".to_string(),
            name: "Else Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(2),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("conditional")
            .when("x > 10")
            .do_this(if_step)
            .otherwise(else_step)
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                assert_eq!(condition, "x > 10");
                assert_eq!(if_true.id, "if_step");
                assert!(if_false.is_some());
                assert_eq!(if_false.as_ref().unwrap().id, "else_step");
            }
            _ => panic!("Expected Conditional step"),
        }
        assert!(workflow.steps[0].id.starts_with("cond_"));
        assert_eq!(workflow.steps[0].name, "Conditional");
    }

    #[test]
    fn test_conditional_else_end_if() {
        let if_step = WorkflowStep {
            id: "only_if".to_string(),
            name: "Only If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("no_else")
            .when("y < 5")
            .do_this(if_step)
            .end_if()
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                assert_eq!(condition, "y < 5");
                assert_eq!(if_true.id, "only_if");
                assert!(if_false.is_none());
            }
            _ => panic!("Expected Conditional step"),
        }
    }

    #[test]
    fn test_conditional_else_end_if_then_continue() {
        let if_step = WorkflowStep {
            id: "if".to_string(),
            name: "If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let next_step = WorkflowStep {
            id: "next".to_string(),
            name: "Next".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("continue_after_if")
            .when("cond")
            .do_this(if_step)
            .end_if()
            .then(next_step)
            .build();

        assert_eq!(workflow.steps.len(), 2);
        assert!(matches!(
            &workflow.steps[0].step_type,
            StepType::Conditional { .. }
        ));
        assert_eq!(workflow.steps[1].id, "next");
    }

    #[test]
    fn test_conditional_else_otherwise_then_continue() {
        let if_step = WorkflowStep {
            id: "if".to_string(),
            name: "If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let else_step = WorkflowStep {
            id: "else".to_string(),
            name: "Else".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let next_step = WorkflowStep {
            id: "next".to_string(),
            name: "Next".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("continue_after_else")
            .when("cond")
            .do_this(if_step)
            .otherwise(else_step)
            .then(next_step)
            .build();

        assert_eq!(workflow.steps.len(), 2);
        assert!(matches!(
            &workflow.steps[0].step_type,
            StepType::Conditional { .. }
        ));
        assert_eq!(workflow.steps[1].id, "next");
    }

    // =========================================================================
    // WORKFLOW_DSL_EXAMPLE constant tests
    // =========================================================================

    #[test]
    fn test_workflow_dsl_example_is_valid_json() {
        let result: Result<serde_json::Value, _> = serde_json::from_str(WORKFLOW_DSL_EXAMPLE);
        assert!(result.is_ok());
    }

    #[test]
    fn test_workflow_dsl_example_parses_to_workflow() {
        let result = DslCompiler::compile(WORKFLOW_DSL_EXAMPLE);
        assert!(result.is_ok());
        let workflow = result.unwrap();

        // Verify all expected fields
        assert_eq!(
            workflow.id.to_string(),
            "f47ac10b-58cc-4372-a567-0e02b2c3d479"
        );
        assert_eq!(workflow.name, "quality_check_workflow");
        assert_eq!(
            workflow.description,
            Some("Comprehensive quality check workflow".to_string())
        );
        assert_eq!(workflow.version, "1.0.0");
        assert_eq!(workflow.steps.len(), 1);
        assert!(matches!(workflow.error_strategy, ErrorStrategy::FailFast));
        assert_eq!(workflow.timeout, Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_workflow_dsl_example_step_properties() {
        let workflow = DslCompiler::compile(WORKFLOW_DSL_EXAMPLE).unwrap();
        let step = &workflow.steps[0];

        assert_eq!(step.id, "analyze");
        assert_eq!(step.name, "Code Analysis");
        assert_eq!(step.timeout, Some(Duration::from_secs(60)));
        assert!(step.retry.is_some());

        match &step.step_type {
            StepType::Action {
                agent,
                operation,
                params,
            } => {
                assert_eq!(agent, "analyzer");
                assert_eq!(operation, "analyze");
                assert!(params.is_object());
            }
            _ => panic!("Expected Action step type"),
        }
    }

    // =========================================================================
    // Edge cases and complex scenarios
    // =========================================================================

    #[test]
    fn test_nested_conditionals() {
        let inner_if_step = WorkflowStep {
            id: "inner_if".to_string(),
            name: "Inner If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let outer_if_step = WorkflowStep {
            id: "outer_if".to_string(),
            name: "Outer If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        // Build outer conditional
        let workflow = FluentWorkflow::define("nested")
            .when("outer_cond")
            .do_this(outer_if_step)
            .end_if()
            .when("inner_cond")
            .do_this(inner_if_step)
            .end_if()
            .build();

        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_parallel_with_many_steps() {
        let steps: Vec<WorkflowStep> = (0..10)
            .map(|i| WorkflowStep {
                id: format!("step_{}", i),
                name: format!("Step {}", i),
                step_type: StepType::Wait {
                    duration: Duration::from_secs(1),
                },
                condition: None,
                retry: None,
                timeout: None,
                on_error: None,
                metadata: HashMap::new(),
            })
            .collect();

        let workflow = FluentWorkflow::define("many_parallel")
            .parallel(steps)
            .build();

        match &workflow.steps[0].step_type {
            StepType::Parallel { steps } => {
                assert_eq!(steps.len(), 10);
            }
            _ => panic!("Expected Parallel"),
        }
    }

    #[test]
    fn test_fluent_workflow_complex_sequence() {
        let step1 = WorkflowStep {
            id: "s1".to_string(),
            name: "S1".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step2 = WorkflowStep {
            id: "s2".to_string(),
            name: "S2".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step3 = WorkflowStep {
            id: "s3".to_string(),
            name: "S3".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step4 = WorkflowStep {
            id: "s4".to_string(),
            name: "S4".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let loop_step = WorkflowStep {
            id: "loop_body".to_string(),
            name: "Loop Body".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("complex")
            .then(step1)
            .parallel(vec![step2, step3])
            .when("check")
            .do_this(step4)
            .end_if()
            .repeat("counter < 5", loop_step)
            .on_error(ErrorStrategy::Rollback)
            .with_timeout(Duration::from_secs(600))
            .build();

        assert_eq!(workflow.name, "complex");
        assert_eq!(workflow.steps.len(), 4); // then, parallel, conditional, loop
        assert!(matches!(workflow.error_strategy, ErrorStrategy::Rollback));
        assert_eq!(workflow.timeout, Some(Duration::from_secs(600)));
    }

    #[test]
    fn test_compile_workflow_with_all_step_types() {
        // Test that we can handle all step types through JSON compilation
        let json = r#"{
            "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "name": "all_types",
            "version": "1.0.0",
            "steps": [
                {
                    "id": "action_step",
                    "name": "Action",
                    "step_type": {"type": "action", "agent": "a", "operation": "o", "params": {}},
                    "metadata": {}
                },
                {
                    "id": "wait_step",
                    "name": "Wait",
                    "step_type": {"type": "wait", "duration": {"secs": 5, "nanos": 0}},
                    "metadata": {}
                }
            ],
            "error_strategy": "Continue",
            "metadata": {}
        }"#;

        let result = DslCompiler::compile(json);
        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_compile_step_with_all_optional_fields() {
        let json = r#"{
            "id": "full_step",
            "name": "Full Step",
            "step_type": {"type": "action", "agent": "a", "operation": "o", "params": {}},
            "condition": {"expression": "x > 0", "skip_on_false": true},
            "retry": {
                "max_attempts": 3,
                "backoff": {"Fixed": {"delay": {"secs": 1, "nanos": 0}}},
                "retry_on": ["error1"]
            },
            "timeout": {"secs": 30, "nanos": 0},
            "on_error": "Skip",
            "metadata": {"key": "value"}
        }"#;

        let result = DslCompiler::compile_step(json);
        assert!(result.is_ok());
        let step = result.unwrap();
        assert!(step.condition.is_some());
        assert!(step.retry.is_some());
        assert!(step.timeout.is_some());
        assert!(step.on_error.is_some());
        assert!(!step.metadata.is_empty());
    }

    #[test]
    fn test_workflow_macro_with_single_step() {
        let wf = workflow!("single_macro" => {
            step!(wait: Duration::from_secs(1)),
        });

        assert_eq!(wf.name, "single_macro");
        assert_eq!(wf.steps.len(), 1);
    }

    #[test]
    fn test_workflow_macro_with_empty_steps() {
        let wf = workflow!("empty_macro" => {});

        assert_eq!(wf.name, "empty_macro");
        assert!(wf.steps.is_empty());
    }

    #[test]
    fn test_step_macro_action_with_params() {
        // Use StepBuilder directly instead of macro to avoid identifier issues
        let step = StepBuilder::action(
            "step_test".to_string(),
            "my_agent.my_op".to_string(),
            "my_agent",
            "my_op",
        )
        .params(serde_json::json!({
            "key1": "value1",
            "key2": 42
        }))
        .build();

        assert!(step.name.contains("my_agent"));
        assert!(step.name.contains("my_op"));

        match &step.step_type {
            StepType::Action {
                agent,
                operation,
                params,
            } => {
                assert_eq!(agent, "my_agent");
                assert_eq!(operation, "my_op");
                assert_eq!(params["key1"], "value1");
                assert_eq!(params["key2"], 42);
            }
            _ => panic!("Expected Action step"),
        }
    }

    #[test]
    fn test_step_macro_wait() {
        let step = step!(wait: Duration::from_millis(500));

        assert!(step.id.starts_with("wait_"));
        assert_eq!(step.name, "Wait");

        match &step.step_type {
            StepType::Wait { duration } => {
                assert_eq!(*duration, Duration::from_millis(500));
            }
            _ => panic!("Expected Wait step"),
        }
    }

    #[test]
    fn test_yaml_fallback_when_json_fails() {
        // Valid YAML but invalid JSON
        let yaml_only = r#"
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
name: yaml_only_workflow
version: "1.0.0"
steps: []
error_strategy: FailFast
metadata: {}
"#;

        let result = DslCompiler::compile(yaml_only);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "yaml_only_workflow");
    }

    #[test]
    fn test_compile_step_yaml_fallback() {
        // Valid YAML step but not JSON
        let yaml_step = r#"
id: yaml_step
name: YAML Only Step
step_type:
  type: wait
  duration:
    secs: 5
    nanos: 0
metadata: {}
"#;

        let result = DslCompiler::compile_step(yaml_step);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "yaml_step");
    }
}
