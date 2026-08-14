#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests {
    use crate::workflow::dsl::*;
    use crate::{step, workflow};
    use std::time::Duration;

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
