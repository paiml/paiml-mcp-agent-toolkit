#![cfg_attr(coverage_nightly, coverage(off))]

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
