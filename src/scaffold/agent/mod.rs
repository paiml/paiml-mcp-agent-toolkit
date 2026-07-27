#![cfg_attr(coverage_nightly, coverage(off))]
//! Agent scaffolding system for generating deterministic MCP agents.
//!
//! This module provides comprehensive scaffolding capabilities for creating
//! production-grade MCP agents with deterministic cores, state machines,
//! and hybrid architectures.

pub mod context;
pub mod error;
pub mod features;
pub mod generator;
pub mod hybrid;
// REMOVED: interactive module requires dialoguer crate (saves 14 transitive deps)
// pub mod interactive;
pub mod invariants;
pub mod registry;
pub mod subagents;
pub mod templates;

pub use context::{AgentContext, AgentContextBuilder};
pub use error::ScaffoldError;
pub use features::{AgentFeature, MonitoringBackend, QualityLevel, TraceExporter};
pub use generator::{FileContent, GeneratedFiles, TemplateGenerator};
pub use hybrid::{
    BoundarySpec, CoreSpec, ErrorPropagation, FallbackStrategy, HybridAgentSpec, ModelType,
    SerializationFormat, ValidationStrategy, VerificationMethod, WrapperSpec,
};
// REMOVED: InteractiveScaffolder requires dialoguer crate
// pub use interactive::InteractiveScaffolder;
pub use invariants::{
    AgentContext as InvariantAgentContext, AgentEvent, AgentState, AgentStateMachine, Invariant,
    InvariantChecker, InvariantViolation, ViolationAction, ViolationHandler,
};
pub use registry::TemplateRegistry;
pub use subagents::{PmatSubAgent, SubAgentGenerator};
pub use templates::{AgentTemplate, MCPServerTemplate, StateMachineTemplate};

use anyhow::Result;
use std::path::Path;

/// Scaffold a new agent with the given configuration.
///
/// # Examples
///
/// ```no_run
/// use pmat::scaffold::agent::{scaffold_agent, AgentContext, AgentTemplate, QualityLevel};
/// use std::collections::HashSet;
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// let context = AgentContext {
///     name: "my_agent".to_string(),
///     template_type: AgentTemplate::MCPToolServer,
///     features: HashSet::new(),
///     quality_level: QualityLevel::Extreme,
///     deterministic_core: None,
///     probabilistic_wrapper: None,
/// };
///
/// scaffold_agent(&context, Path::new("./my_agent")).await?;
/// # Ok(())
/// # }
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn scaffold_agent(context: &AgentContext, output: &Path) -> Result<()> {
    let registry = TemplateRegistry::new();
    let generator = registry.get(&context.template_type)?;

    generator.validate_context(context)?;
    let files = generator.generate(context)?;

    files.write_to_disk(output).await?;
    generator.post_generation_hooks(output)?;

    Ok(())
}

/// List all available agent templates.
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn list_templates() -> Vec<String> {
    let registry = TemplateRegistry::new();
    registry.list_available()
}

/// Validate a template file.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn validate_template(path: &Path) -> Result<()> {
    let registry = TemplateRegistry::new();
    registry.validate_template_file(path)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn test_list_templates_returns_vec() {
        let templates = list_templates();
        assert!(
            !templates.is_empty(),
            "the built-in template list must not be empty"
        );
    }

    #[test]
    fn test_validate_template_nonexistent() {
        let result = validate_template(Path::new("/nonexistent/template.toml"));
        // Should fail for nonexistent file
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_context_builder_exists() {
        // Verify AgentContextBuilder is exported and usable
        let _builder: Option<AgentContextBuilder> = None;
    }

    #[test]
    fn test_template_generator_trait() {
        // Verify GeneratedFiles type is exported
        let _files: Option<GeneratedFiles> = None;
    }

    #[test]
    fn test_agent_template_export() {
        // Verify AgentTemplate enum is accessible
        let template = AgentTemplate::MCPToolServer;
        let debug_str = format!("{:?}", template);
        assert!(debug_str.contains("MCPToolServer"));
    }

    #[test]
    fn test_quality_level_export() {
        // Verify QualityLevel enum is accessible
        let level = QualityLevel::Extreme;
        let debug_str = format!("{:?}", level);
        assert!(debug_str.contains("Extreme"));
    }

    #[test]
    fn test_agent_feature_export() {
        // Verify AgentFeature enum is accessible
        let _feature: Option<AgentFeature> = None;
    }

    #[test]
    fn test_monitoring_backend_export() {
        // Verify MonitoringBackend enum is accessible
        let _backend: Option<MonitoringBackend> = None;
    }

    #[test]
    fn test_trace_exporter_export() {
        // Verify TraceExporter enum is accessible
        let _exporter: Option<TraceExporter> = None;
    }

    #[test]
    fn test_hybrid_agent_spec_export() {
        // Verify HybridAgentSpec type is accessible
        let _spec: Option<HybridAgentSpec> = None;
    }

    #[test]
    fn test_core_spec_export() {
        // Verify CoreSpec type is accessible
        let _spec: Option<CoreSpec> = None;
    }

    #[test]
    fn test_wrapper_spec_export() {
        // Verify WrapperSpec type is accessible
        let _spec: Option<WrapperSpec> = None;
    }

    #[test]
    fn test_boundary_spec_export() {
        // Verify BoundarySpec type is accessible
        let _spec: Option<BoundarySpec> = None;
    }

    #[test]
    fn test_model_type_export() {
        // Verify ModelType enum is accessible
        let _model: Option<ModelType> = None;
    }

    #[test]
    fn test_fallback_strategy_export() {
        // Verify FallbackStrategy enum is accessible
        let _strategy: Option<FallbackStrategy> = None;
    }

    #[test]
    fn test_validation_strategy_export() {
        // Verify ValidationStrategy enum is accessible
        let _strategy: Option<ValidationStrategy> = None;
    }

    #[test]
    fn test_serialization_format_export() {
        // Verify SerializationFormat enum is accessible
        let _format: Option<SerializationFormat> = None;
    }

    #[test]
    fn test_verification_method_export() {
        // Verify VerificationMethod enum is accessible
        let _method: Option<VerificationMethod> = None;
    }

    #[test]
    fn test_error_propagation_export() {
        // Verify ErrorPropagation enum is accessible
        let _propagation: Option<ErrorPropagation> = None;
    }

    // REMOVED: InteractiveScaffolder requires dialoguer crate
    // #[test]
    // fn test_interactive_scaffolder_export() {
    //     // Verify InteractiveScaffolder type is accessible
    //     let _scaffolder: Option<InteractiveScaffolder> = None;
    // }

    #[test]
    fn test_agent_state_machine_trait_exists() {
        // Verify AgentStateMachine trait is accessible as a trait
        fn _ensure_trait_exists<T: AgentStateMachine>() {}
    }

    #[test]
    fn test_agent_state_trait_exists() {
        // Verify AgentState trait is accessible as a trait
        fn _ensure_trait_exists<T: AgentState>() {}
    }

    #[test]
    fn test_agent_event_trait_exists() {
        // Verify AgentEvent trait is accessible as a trait
        fn _ensure_trait_exists<T: AgentEvent>() {}
    }

    #[test]
    fn test_invariant_trait_exists() {
        // Verify Invariant trait is accessible as a trait
        fn _ensure_trait_exists<S, C, T: Invariant<S, C>>() {}
    }

    #[test]
    fn test_invariant_checker_struct_exists() {
        // Verify InvariantChecker struct is accessible (requires generics)
        #[derive(Clone, Debug)]
        struct TestState;
        impl AgentState for TestState {}

        #[derive(Clone)]
        struct TestCtx;
        impl super::InvariantAgentContext for TestCtx {}

        let _checker: Option<InvariantChecker<TestState, TestCtx>> = None;
    }

    #[test]
    fn test_invariant_violation_export() {
        // Verify InvariantViolation type is accessible
        let _violation: Option<InvariantViolation> = None;
    }

    #[test]
    fn test_violation_action_export() {
        // Verify ViolationAction type is accessible
        let _action: Option<ViolationAction> = None;
    }

    #[test]
    fn test_violation_handler_export() {
        // Verify ViolationHandler type is accessible
        let _handler: Option<ViolationHandler> = None;
    }

    #[test]
    fn test_template_registry_export() {
        // Verify TemplateRegistry type is accessible
        let registry = TemplateRegistry::new();
        let _ = registry.list_available();
    }

    #[test]
    fn test_pmat_sub_agent_export() {
        // Verify PmatSubAgent type is accessible
        let _agent: Option<PmatSubAgent> = None;
    }

    #[test]
    fn test_sub_agent_generator_export() {
        // Verify SubAgentGenerator type is accessible
        let _generator: Option<SubAgentGenerator> = None;
    }

    #[test]
    fn test_mcp_server_template_export() {
        // Verify MCPServerTemplate type is accessible
        let _template: Option<MCPServerTemplate> = None;
    }

    #[test]
    fn test_state_machine_template_export() {
        // Verify StateMachineTemplate type is accessible
        let _template: Option<StateMachineTemplate> = None;
    }

    #[test]
    fn test_file_content_export() {
        // Verify FileContent type is accessible
        let _content: Option<FileContent> = None;
    }

    #[test]
    fn test_scaffold_error_export() {
        // Verify ScaffoldError is re-exported from error module
        let err = ScaffoldError::UserCancelled;
        assert_eq!(err.to_string(), "Operation cancelled by user");
    }

    #[test]
    fn test_invariant_agent_context_export() {
        // Verify InvariantAgentContext trait is accessible
        fn _ensure_trait_exists<T: InvariantAgentContext>() {}
    }
}
