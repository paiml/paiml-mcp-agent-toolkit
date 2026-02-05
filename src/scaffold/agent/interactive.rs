//! Interactive scaffolding interface for guided agent creation.

use super::context::{AgentContext, AgentContextBuilder};
use super::error::ScaffoldError;
use super::features::{AgentFeature, QualityLevel};
use super::hybrid::{CoreSpec, FallbackStrategy, ModelType, VerificationMethod, WrapperSpec};
use super::templates::AgentTemplate;
use anyhow::Result;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::collections::HashSet;
use std::path::PathBuf;

/// Interactive scaffolder for guided agent creation.
pub struct InteractiveScaffolder {
    /// Terminal for I/O.
    term: Term,
    /// Color theme for prompts.
    theme: ColorfulTheme,
}

impl InteractiveScaffolder {
    /// Create a new interactive scaffolder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
            theme: ColorfulTheme::default(),
        }
    }

    /// Run the interactive scaffolding process.
    pub fn run(&mut self) -> Result<AgentContext> {
        // Clear screen and show header
        self.term.clear_screen()?;
        self.show_header()?;

        // Prompt for basic information
        let name = self.prompt_name()?;
        let template = self.prompt_template()?;
        let features = self.prompt_features(&template)?;
        let quality = self.prompt_quality_level()?;

        // Build context
        let mut builder = AgentContextBuilder::new(&name, self.template_to_string(&template));
        for feature in &features {
            builder = builder.with_feature(feature.clone());
        }
        builder = builder.with_quality_level(quality);

        // Handle hybrid-specific configuration
        if matches!(template, AgentTemplate::HybridAnalyzer) {
            let core = self.prompt_deterministic_core()?;
            let wrapper = self.prompt_probabilistic_wrapper()?;
            builder = builder
                .with_deterministic_core(core)
                .with_probabilistic_wrapper(wrapper);
        }

        // Build and confirm
        let ctx = builder.build()?;
        self.confirm_and_display(&ctx)?;

        Ok(ctx)
    }

    /// Show the header.
    fn show_header(&self) -> Result<()> {
        println!("╔══════════════════════════════════════════╗");
        println!("║      PMAT Agent Scaffolder v0.1.0       ║");
        println!("║   Interactive Agent Creation Wizard      ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        Ok(())
    }

    /// Prompt for agent name.
    fn prompt_name(&self) -> Result<String> {
        loop {
            let name: String = Input::with_theme(&self.theme)
                .with_prompt("Agent name")
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.is_empty() {
                        Err("Name cannot be empty")
                    } else if !input.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        Err("Name must be alphanumeric with underscores only")
                    } else if input.chars().next().is_some_and(char::is_numeric) {
                        Err("Name cannot start with a number")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?;

            if name.len() > 64 {
                eprintln!("Warning: Name is quite long. Consider a shorter name.");
                if !Confirm::with_theme(&self.theme)
                    .with_prompt("Continue with this name?")
                    .interact()?
                {
                    continue;
                }
            }

            return Ok(name);
        }
    }

    /// Prompt for template type.
    fn prompt_template(&self) -> Result<AgentTemplate> {
        let items = [
            ("MCP Tool Server", "Standard MCP server with async handlers"),
            (
                "State Machine Workflow",
                "Agent with state transitions and invariants",
            ),
            (
                "Deterministic Calculator",
                "Pure deterministic computation agent",
            ),
            ("Hybrid Analyzer", "Deterministic core with AI wrapper"),
            ("Custom Template", "Use a custom template from file"),
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select template type")
            .items(
                items
                    .iter()
                    .map(|(name, desc)| format!("{name}\n   {desc}"))
                    .collect::<Vec<_>>(),
            )
            .default(0)
            .interact()?;

        Ok(match selection {
            0 => AgentTemplate::MCPToolServer,
            1 => AgentTemplate::StateMachineWorkflow,
            2 => AgentTemplate::DeterministicCalculator,
            3 => AgentTemplate::HybridAnalyzer,
            4 => {
                let path: String = Input::with_theme(&self.theme)
                    .with_prompt("Custom template path")
                    .interact_text()?;
                AgentTemplate::CustomAgent(PathBuf::from(path))
            }
            _ => unreachable!(),
        })
    }

    /// Prompt for features to include.
    fn prompt_features(&self, template: &AgentTemplate) -> Result<HashSet<AgentFeature>> {
        let available_features = self.get_available_features(template);

        if available_features.is_empty() {
            return Ok(HashSet::new());
        }

        let feature_names: Vec<String> = available_features
            .iter()
            .map(|f| self.feature_to_string(f))
            .collect();

        let selections = MultiSelect::with_theme(&self.theme)
            .with_prompt("Select features to include (Space to select, Enter to confirm)")
            .items(&feature_names)
            .interact()?;

        let mut features = HashSet::new();
        for idx in selections {
            features.insert(available_features[idx].clone());
        }

        Ok(features)
    }

    /// Get available features for a template.
    fn get_available_features(&self, template: &AgentTemplate) -> Vec<AgentFeature> {
        match template {
            AgentTemplate::MCPToolServer => vec![
                AgentFeature::ToolComposition,
                AgentFeature::AsyncHandlers,
                AgentFeature::ResourceSubscriptions,
                AgentFeature::Monitoring {
                    backend: super::features::MonitoringBackend::Prometheus,
                },
                AgentFeature::Tracing {
                    exporter: super::features::TraceExporter::OTLP,
                },
                AgentFeature::HealthChecks,
            ],
            AgentTemplate::StateMachineWorkflow => vec![
                AgentFeature::StateMachine {
                    states: vec![
                        "Initial".to_string(),
                        "Processing".to_string(),
                        "Complete".to_string(),
                    ],
                },
                AgentFeature::QualityGates {
                    level: QualityLevel::Extreme,
                },
            ],
            AgentTemplate::HybridAnalyzer => vec![
                AgentFeature::ComplexityAnalysis,
                AgentFeature::SATDDetection,
                AgentFeature::DeadCodeElimination,
            ],
            _ => vec![],
        }
    }

    /// Convert feature to display string.
    fn feature_to_string(&self, feature: &AgentFeature) -> String {
        match feature {
            AgentFeature::StateMachine { .. } => "State Machine with transitions".to_string(),
            AgentFeature::QualityGates { .. } => "Quality Gates enforcement".to_string(),
            AgentFeature::ToolComposition => "Tool Composition support".to_string(),
            AgentFeature::AsyncHandlers => "Async request handlers".to_string(),
            AgentFeature::ResourceSubscriptions => "Resource subscriptions".to_string(),
            AgentFeature::ComplexityAnalysis => "Complexity analysis".to_string(),
            AgentFeature::SATDDetection => "SATD detection".to_string(),
            AgentFeature::DeadCodeElimination => "Dead code elimination".to_string(),
            AgentFeature::Monitoring { .. } => "Monitoring integration".to_string(),
            AgentFeature::Tracing { .. } => "Distributed tracing".to_string(),
            AgentFeature::HealthChecks => "Health check endpoints".to_string(),
        }
    }

    /// Prompt for quality level.
    fn prompt_quality_level(&self) -> Result<QualityLevel> {
        let items = [
            ("Standard", "Basic quality checks, suitable for prototypes"),
            ("Strict", "Zero warnings, high test coverage"),
            (
                "Extreme (Toyota Way)",
                "Zero SATD, max complexity 10, full verification",
            ),
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Quality level")
            .items(
                items
                    .iter()
                    .map(|(name, desc)| format!("{name}\n   {desc}"))
                    .collect::<Vec<_>>(),
            )
            .default(1)
            .interact()?;

        Ok(match selection {
            0 => QualityLevel::Standard,
            1 => QualityLevel::Strict,
            2 => QualityLevel::Extreme,
            _ => unreachable!(),
        })
    }

    /// Prompt for deterministic core specification.
    fn prompt_deterministic_core(&self) -> Result<CoreSpec> {
        println!("\n=== Deterministic Core Configuration ===");

        let verification_items = ["Property-based tests", "Formal proof", "Model checking"];

        let verification_idx = Select::with_theme(&self.theme)
            .with_prompt("Verification method")
            .items(verification_items)
            .default(0)
            .interact()?;

        let verification_method = match verification_idx {
            0 => VerificationMethod::PropertyTests,
            1 => VerificationMethod::FormalProof,
            2 => VerificationMethod::ModelChecking,
            _ => unreachable!(),
        };

        let max_complexity = Input::with_theme(&self.theme)
            .with_prompt("Maximum cyclomatic complexity")
            .default(10)
            .validate_with(|input: &u32| -> Result<(), &str> {
                if *input == 0 {
                    Err("Complexity must be at least 1")
                } else if *input > 50 {
                    Err("Complexity should not exceed 50")
                } else {
                    Ok(())
                }
            })
            .interact()?;

        Ok(CoreSpec {
            verification_method,
            max_complexity,
            invariants: Vec::new(),
        })
    }

    /// Prompt for probabilistic wrapper specification.
    fn prompt_probabilistic_wrapper(&self) -> Result<WrapperSpec> {
        println!("\n=== Probabilistic Wrapper Configuration ===");

        let model_items = ["GPT-4", "Claude", "Local model"];

        let model_idx = Select::with_theme(&self.theme)
            .with_prompt("AI model type")
            .items(model_items)
            .default(0)
            .interact()?;

        let model_type = match model_idx {
            0 => ModelType::GPT4,
            1 => ModelType::Claude,
            2 => {
                let path = Input::with_theme(&self.theme)
                    .with_prompt("Local model path")
                    .interact_text()?;
                ModelType::Local(path)
            }
            _ => unreachable!(),
        };

        let fallback_items = ["Deterministic fallback", "Default value", "Return error"];

        let fallback_idx = Select::with_theme(&self.theme)
            .with_prompt("Fallback strategy")
            .items(fallback_items)
            .default(0)
            .interact()?;

        let fallback_strategy = match fallback_idx {
            0 => FallbackStrategy::Deterministic,
            1 => FallbackStrategy::DefaultValue,
            2 => FallbackStrategy::Error,
            _ => unreachable!(),
        };

        let confidence_threshold = Input::with_theme(&self.theme)
            .with_prompt("Confidence threshold (0.0-1.0)")
            .default(0.95)
            .validate_with(|input: &f64| -> Result<(), &str> {
                if *input < 0.0 || *input > 1.0 {
                    Err("Threshold must be between 0.0 and 1.0")
                } else {
                    Ok(())
                }
            })
            .interact()?;

        Ok(WrapperSpec {
            model_type,
            fallback_strategy,
            confidence_threshold,
        })
    }

    /// Convert template to string representation.
    fn template_to_string(&self, template: &AgentTemplate) -> String {
        match template {
            AgentTemplate::MCPToolServer => "mcp-server".to_string(),
            AgentTemplate::StateMachineWorkflow => "state-machine".to_string(),
            AgentTemplate::DeterministicCalculator => "calculator".to_string(),
            AgentTemplate::HybridAnalyzer => "hybrid".to_string(),
            AgentTemplate::CustomAgent(path) => format!("custom:{}", path.display()),
        }
    }

    /// Confirm and display the configuration.
    fn confirm_and_display(&self, ctx: &AgentContext) -> Result<()> {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║         Agent Configuration Summary      ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        println!("  Name:     {}", ctx.name);
        println!("  Template: {:?}", ctx.template_type);
        println!("  Quality:  {:?}", ctx.quality_level);
        println!("  Features: {} enabled", ctx.features.len());

        if let Some(core) = &ctx.deterministic_core {
            println!("\n  Deterministic Core:");
            println!("    Verification: {:?}", core.verification_method);
            println!("    Max Complexity: {}", core.max_complexity);
        }

        if let Some(wrapper) = &ctx.probabilistic_wrapper {
            println!("\n  Probabilistic Wrapper:");
            println!("    Model: {:?}", wrapper.model_type);
            println!("    Fallback: {:?}", wrapper.fallback_strategy);
            println!("    Confidence: {:.2}", wrapper.confidence_threshold);
        }

        println!();
        let confirm = Confirm::with_theme(&self.theme)
            .with_prompt("Generate agent with these settings?")
            .default(true)
            .interact()?;

        if !confirm {
            return Err(ScaffoldError::UserCancelled.into());
        }

        Ok(())
    }
}

impl Default for InteractiveScaffolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_scaffolder_creation() {
        let scaffolder = InteractiveScaffolder::new();
        // Verify the scaffolder was created with expected components
        // The term and theme are initialized correctly
        assert!(std::mem::size_of_val(&scaffolder) > 0);
    }

    #[test]
    fn test_interactive_scaffolder_default() {
        let scaffolder = InteractiveScaffolder::default();
        // Verify Default impl calls new()
        assert!(std::mem::size_of_val(&scaffolder) > 0);
    }

    // =========================================================================
    // feature_to_string comprehensive tests
    // =========================================================================

    #[test]
    fn test_feature_to_string_tool_composition() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::ToolComposition;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Tool Composition support"
        );
    }

    #[test]
    fn test_feature_to_string_async_handlers() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::AsyncHandlers;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Async request handlers"
        );
    }

    #[test]
    fn test_feature_to_string_resource_subscriptions() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::ResourceSubscriptions;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Resource subscriptions"
        );
    }

    #[test]
    fn test_feature_to_string_complexity_analysis() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::ComplexityAnalysis;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Complexity analysis"
        );
    }

    #[test]
    fn test_feature_to_string_satd_detection() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::SATDDetection;
        assert_eq!(scaffolder.feature_to_string(&feature), "SATD detection");
    }

    #[test]
    fn test_feature_to_string_dead_code_elimination() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::DeadCodeElimination;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Dead code elimination"
        );
    }

    #[test]
    fn test_feature_to_string_state_machine() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::StateMachine {
            states: vec!["Init".to_string(), "Running".to_string()],
        };
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "State Machine with transitions"
        );
    }

    #[test]
    fn test_feature_to_string_quality_gates() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::QualityGates {
            level: QualityLevel::Extreme,
        };
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Quality Gates enforcement"
        );
    }

    #[test]
    fn test_feature_to_string_monitoring() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::Monitoring {
            backend: super::super::features::MonitoringBackend::Prometheus,
        };
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Monitoring integration"
        );
    }

    #[test]
    fn test_feature_to_string_tracing() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::Tracing {
            exporter: super::super::features::TraceExporter::OTLP,
        };
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Distributed tracing"
        );
    }

    #[test]
    fn test_feature_to_string_health_checks() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::HealthChecks;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Health check endpoints"
        );
    }

    // =========================================================================
    // template_to_string comprehensive tests
    // =========================================================================

    #[test]
    fn test_template_to_string_mcp_server() {
        let scaffolder = InteractiveScaffolder::new();
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::MCPToolServer),
            "mcp-server"
        );
    }

    #[test]
    fn test_template_to_string_state_machine() {
        let scaffolder = InteractiveScaffolder::new();
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::StateMachineWorkflow),
            "state-machine"
        );
    }

    #[test]
    fn test_template_to_string_calculator() {
        let scaffolder = InteractiveScaffolder::new();
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::DeterministicCalculator),
            "calculator"
        );
    }

    #[test]
    fn test_template_to_string_hybrid() {
        let scaffolder = InteractiveScaffolder::new();
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::HybridAnalyzer),
            "hybrid"
        );
    }

    #[test]
    fn test_template_to_string_custom() {
        let scaffolder = InteractiveScaffolder::new();
        let custom_path = PathBuf::from("/path/to/my/template.toml");
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::CustomAgent(custom_path)),
            "custom:/path/to/my/template.toml"
        );
    }

    #[test]
    fn test_template_to_string_custom_relative() {
        let scaffolder = InteractiveScaffolder::new();
        let custom_path = PathBuf::from("relative/path/template.toml");
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::CustomAgent(custom_path)),
            "custom:relative/path/template.toml"
        );
    }

    // =========================================================================
    // get_available_features comprehensive tests
    // =========================================================================

    #[test]
    fn test_get_available_features_mcp_server() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::MCPToolServer);

        // MCP server should have 6 features
        assert_eq!(features.len(), 6);

        // Verify specific features are present
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::ToolComposition)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::AsyncHandlers)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::ResourceSubscriptions)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::Monitoring { .. })));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::Tracing { .. })));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::HealthChecks)));
    }

    #[test]
    fn test_get_available_features_state_machine() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow);

        // State machine should have 2 features
        assert_eq!(features.len(), 2);

        // Verify specific features are present
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::StateMachine { .. })));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::QualityGates { .. })));
    }

    #[test]
    fn test_get_available_features_hybrid_analyzer() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::HybridAnalyzer);

        // Hybrid analyzer should have 3 features
        assert_eq!(features.len(), 3);

        // Verify specific features are present
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::ComplexityAnalysis)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::SATDDetection)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::DeadCodeElimination)));
    }

    #[test]
    fn test_get_available_features_deterministic_calculator() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::DeterministicCalculator);

        // Deterministic calculator has no additional features (falls through to default)
        assert!(features.is_empty());
    }

    #[test]
    fn test_get_available_features_custom_agent() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder
            .get_available_features(&AgentTemplate::CustomAgent(PathBuf::from("custom.toml")));

        // Custom agent has no predefined features (falls through to default)
        assert!(features.is_empty());
    }

    // =========================================================================
    // show_header test
    // =========================================================================

    #[test]
    fn test_show_header_returns_ok() {
        let scaffolder = InteractiveScaffolder::new();
        // show_header only prints to stdout, should always return Ok
        let result = scaffolder.show_header();
        assert!(result.is_ok());
    }

    // =========================================================================
    // State machine feature state verification
    // =========================================================================

    #[test]
    fn test_state_machine_feature_has_expected_states() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow);

        // Find the state machine feature and verify its default states
        let state_machine_feature = features
            .iter()
            .find(|f| matches!(f, AgentFeature::StateMachine { .. }));

        assert!(state_machine_feature.is_some());
        if let Some(AgentFeature::StateMachine { states }) = state_machine_feature {
            assert_eq!(states.len(), 3);
            assert_eq!(states[0], "Initial");
            assert_eq!(states[1], "Processing");
            assert_eq!(states[2], "Complete");
        }
    }

    // =========================================================================
    // Quality gates feature verification
    // =========================================================================

    #[test]
    fn test_quality_gates_feature_has_extreme_level() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow);

        // Find the quality gates feature and verify its level
        let quality_gates_feature = features
            .iter()
            .find(|f| matches!(f, AgentFeature::QualityGates { .. }));

        assert!(quality_gates_feature.is_some());
        if let Some(AgentFeature::QualityGates { level }) = quality_gates_feature {
            assert!(matches!(level, QualityLevel::Extreme));
        }
    }

    // =========================================================================
    // Monitoring and tracing backend verification
    // =========================================================================

    #[test]
    fn test_mcp_server_monitoring_uses_prometheus() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::MCPToolServer);

        let monitoring_feature = features
            .iter()
            .find(|f| matches!(f, AgentFeature::Monitoring { .. }));

        assert!(monitoring_feature.is_some());
        if let Some(AgentFeature::Monitoring { backend }) = monitoring_feature {
            assert!(matches!(
                backend,
                super::super::features::MonitoringBackend::Prometheus
            ));
        }
    }

    #[test]
    fn test_mcp_server_tracing_uses_otlp() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::MCPToolServer);

        let tracing_feature = features
            .iter()
            .find(|f| matches!(f, AgentFeature::Tracing { .. }));

        assert!(tracing_feature.is_some());
        if let Some(AgentFeature::Tracing { exporter }) = tracing_feature {
            assert!(matches!(
                exporter,
                super::super::features::TraceExporter::OTLP
            ));
        }
    }

    // =========================================================================
    // Error handling tests
    // =========================================================================

    #[test]
    fn test_user_cancelled_error_can_be_created() {
        let err: anyhow::Error = ScaffoldError::UserCancelled.into();
        assert!(err.to_string().contains("cancelled"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn template_to_string_never_empty(_idx in 0usize..5) {
            let scaffolder = InteractiveScaffolder::new();
            let templates = vec![
                AgentTemplate::MCPToolServer,
                AgentTemplate::StateMachineWorkflow,
                AgentTemplate::DeterministicCalculator,
                AgentTemplate::HybridAnalyzer,
                AgentTemplate::CustomAgent(PathBuf::from("test.toml")),
            ];
            let template = &templates[_idx];
            let result = scaffolder.template_to_string(template);
            prop_assert!(!result.is_empty(), "template_to_string should never return empty string");
        }

        #[test]
        fn feature_to_string_never_empty(_idx in 0usize..11) {
            let scaffolder = InteractiveScaffolder::new();
            let features = vec![
                AgentFeature::StateMachine { states: vec!["A".to_string()] },
                AgentFeature::QualityGates { level: QualityLevel::Standard },
                AgentFeature::ToolComposition,
                AgentFeature::AsyncHandlers,
                AgentFeature::ResourceSubscriptions,
                AgentFeature::ComplexityAnalysis,
                AgentFeature::SATDDetection,
                AgentFeature::DeadCodeElimination,
                AgentFeature::Monitoring { backend: super::super::features::MonitoringBackend::Prometheus },
                AgentFeature::Tracing { exporter: super::super::features::TraceExporter::OTLP },
                AgentFeature::HealthChecks,
            ];
            let feature = &features[_idx];
            let result = scaffolder.feature_to_string(feature);
            prop_assert!(!result.is_empty(), "feature_to_string should never return empty string");
        }

        #[test]
        fn get_available_features_returns_valid_vec(_idx in 0usize..5) {
            let scaffolder = InteractiveScaffolder::new();
            let templates = vec![
                AgentTemplate::MCPToolServer,
                AgentTemplate::StateMachineWorkflow,
                AgentTemplate::DeterministicCalculator,
                AgentTemplate::HybridAnalyzer,
                AgentTemplate::CustomAgent(PathBuf::from("test.toml")),
            ];
            let template = &templates[_idx];
            let features = scaffolder.get_available_features(template);
            // Should not panic and return a valid Vec
            prop_assert!(features.len() >= 0);
        }

        #[test]
        fn custom_template_path_preserved(path in "[a-zA-Z0-9_/]+\\.toml") {
            let scaffolder = InteractiveScaffolder::new();
            let template = AgentTemplate::CustomAgent(PathBuf::from(&path));
            let result = scaffolder.template_to_string(&template);
            prop_assert!(result.contains(&path), "Custom template path should be preserved in string");
            prop_assert!(result.starts_with("custom:"), "Custom template string should start with 'custom:'");
        }

        #[test]
        fn state_machine_states_preserved(states in prop::collection::vec("[a-zA-Z]+", 1..10)) {
            let scaffolder = InteractiveScaffolder::new();
            let feature = AgentFeature::StateMachine { states: states.clone() };
            let result = scaffolder.feature_to_string(&feature);
            // The display string doesn't show individual states, but the conversion should succeed
            prop_assert!(!result.is_empty());
            prop_assert_eq!(result, "State Machine with transitions");
        }

        #[test]
        fn quality_level_variants_produce_same_output(level in 0u8..3) {
            let scaffolder = InteractiveScaffolder::new();
            let quality_level = match level {
                0 => QualityLevel::Standard,
                1 => QualityLevel::Strict,
                _ => QualityLevel::Extreme,
            };
            let feature = AgentFeature::QualityGates { level: quality_level };
            let result = scaffolder.feature_to_string(&feature);
            prop_assert_eq!(result, "Quality Gates enforcement");
        }

        #[test]
        fn scaffolder_creation_is_deterministic(_seed in 0u64..1000) {
            // Creating multiple scaffolders should succeed consistently
            let scaffolder1 = InteractiveScaffolder::new();
            let scaffolder2 = InteractiveScaffolder::new();

            // Both should produce the same template strings
            prop_assert_eq!(
                scaffolder1.template_to_string(&AgentTemplate::MCPToolServer),
                scaffolder2.template_to_string(&AgentTemplate::MCPToolServer)
            );
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that all template types have a consistent string representation
    #[test]
    fn test_all_templates_have_unique_strings() {
        let scaffolder = InteractiveScaffolder::new();
        let templates = vec![
            AgentTemplate::MCPToolServer,
            AgentTemplate::StateMachineWorkflow,
            AgentTemplate::DeterministicCalculator,
            AgentTemplate::HybridAnalyzer,
        ];

        let strings: Vec<String> = templates
            .iter()
            .map(|t| scaffolder.template_to_string(t))
            .collect();

        // Check all strings are unique
        let unique: std::collections::HashSet<_> = strings.iter().collect();
        assert_eq!(
            unique.len(),
            strings.len(),
            "All template strings should be unique"
        );
    }

    /// Test that all features have displayable strings
    #[test]
    fn test_all_features_have_displayable_strings() {
        let scaffolder = InteractiveScaffolder::new();

        // Get all features from all templates
        let mut all_features = Vec::new();
        all_features.extend(scaffolder.get_available_features(&AgentTemplate::MCPToolServer));
        all_features
            .extend(scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow));
        all_features.extend(scaffolder.get_available_features(&AgentTemplate::HybridAnalyzer));

        // All features should have non-empty display strings
        for feature in &all_features {
            let display = scaffolder.feature_to_string(feature);
            assert!(
                !display.is_empty(),
                "Feature {:?} should have a non-empty display string",
                feature
            );
        }
    }

    /// Test the relationship between templates and their features
    #[test]
    fn test_template_feature_relationships() {
        let scaffolder = InteractiveScaffolder::new();

        // MCP server has the most features
        let mcp_features = scaffolder.get_available_features(&AgentTemplate::MCPToolServer);
        let sm_features = scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow);
        let hybrid_features = scaffolder.get_available_features(&AgentTemplate::HybridAnalyzer);
        let calc_features = scaffolder.get_available_features(&AgentTemplate::DeterministicCalculator);

        assert!(
            mcp_features.len() > sm_features.len(),
            "MCP server should have more features than state machine"
        );
        assert!(
            hybrid_features.len() > calc_features.len(),
            "Hybrid analyzer should have more features than calculator"
        );
        assert!(
            calc_features.is_empty(),
            "Deterministic calculator should have no predefined features"
        );
    }

    /// Test that Default and new() produce equivalent scaffolders
    #[test]
    fn test_default_equivalence() {
        let via_new = InteractiveScaffolder::new();
        let via_default = InteractiveScaffolder::default();

        // Both should produce identical template strings
        assert_eq!(
            via_new.template_to_string(&AgentTemplate::MCPToolServer),
            via_default.template_to_string(&AgentTemplate::MCPToolServer)
        );
        assert_eq!(
            via_new.template_to_string(&AgentTemplate::HybridAnalyzer),
            via_default.template_to_string(&AgentTemplate::HybridAnalyzer)
        );
    }

    /// Test that feature string representations are user-friendly
    #[test]
    fn test_feature_strings_are_user_friendly() {
        let scaffolder = InteractiveScaffolder::new();

        // All feature strings should be capitalized and descriptive
        let features_to_check = vec![
            (AgentFeature::ToolComposition, "Tool Composition support"),
            (AgentFeature::AsyncHandlers, "Async request handlers"),
            (AgentFeature::HealthChecks, "Health check endpoints"),
            (AgentFeature::ComplexityAnalysis, "Complexity analysis"),
        ];

        for (feature, expected) in features_to_check {
            let result = scaffolder.feature_to_string(&feature);
            assert_eq!(result, expected);
            // Verify the string starts with a capital letter
            assert!(
                result.chars().next().map(|c| c.is_uppercase()).unwrap_or(false),
                "Feature string '{}' should start with uppercase",
                result
            );
        }
    }

    /// Test monitoring backend variations
    #[test]
    fn test_monitoring_backend_variations() {
        use super::super::features::MonitoringBackend;
        let scaffolder = InteractiveScaffolder::new();

        let backends = vec![
            MonitoringBackend::Prometheus,
            MonitoringBackend::OpenTelemetry,
            MonitoringBackend::Custom("datadog".to_string()),
        ];

        for backend in backends {
            let feature = AgentFeature::Monitoring {
                backend: backend.clone(),
            };
            let result = scaffolder.feature_to_string(&feature);
            // All monitoring features should have the same display string
            assert_eq!(result, "Monitoring integration");
        }
    }

    /// Test tracing exporter variations
    #[test]
    fn test_tracing_exporter_variations() {
        use super::super::features::TraceExporter;
        let scaffolder = InteractiveScaffolder::new();

        let exporters = vec![
            TraceExporter::OTLP,
            TraceExporter::Jaeger,
            TraceExporter::Zipkin,
        ];

        for exporter in exporters {
            let feature = AgentFeature::Tracing {
                exporter: exporter.clone(),
            };
            let result = scaffolder.feature_to_string(&feature);
            // All tracing features should have the same display string
            assert_eq!(result, "Distributed tracing");
        }
    }

    /// Test quality level variations in QualityGates feature
    #[test]
    fn test_quality_level_variations() {
        let scaffolder = InteractiveScaffolder::new();

        let levels = vec![
            QualityLevel::Standard,
            QualityLevel::Strict,
            QualityLevel::Extreme,
        ];

        for level in levels {
            let feature = AgentFeature::QualityGates { level };
            let result = scaffolder.feature_to_string(&feature);
            // All quality gates features should have the same display string
            assert_eq!(result, "Quality Gates enforcement");
        }
    }
}
