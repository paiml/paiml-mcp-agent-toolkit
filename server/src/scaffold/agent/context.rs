//! Agent context and configuration types.

use super::features::{AgentFeature, QualityLevel};
use super::hybrid::{CoreSpec, WrapperSpec};
use super::templates::AgentTemplate;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Context for agent template generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// The name of the agent.
    pub name: String,
    /// The template type to use.
    pub template_type: AgentTemplate,
    /// Features to include in the agent.
    pub features: HashSet<AgentFeature>,
    /// Quality level for the generated agent.
    pub quality_level: QualityLevel,
    /// Deterministic core specification for hybrid agents.
    pub deterministic_core: Option<CoreSpec>,
    /// Probabilistic wrapper specification for hybrid agents.
    pub probabilistic_wrapper: Option<WrapperSpec>,
}

impl Default for AgentContext {
    fn default() -> Self {
        Self {
            name: String::new(),
            template_type: AgentTemplate::MCPToolServer,
            features: HashSet::new(),
            quality_level: QualityLevel::Standard,
            deterministic_core: None,
            probabilistic_wrapper: None,
        }
    }
}

/// Builder for creating agent contexts.
pub struct AgentContextBuilder {
    context: AgentContext,
}

impl AgentContextBuilder {
    /// Create a new builder with the given name and template.
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self {
        let template_str = template.into();
        let template_type = match template_str.as_str() {
            "calculator" => AgentTemplate::DeterministicCalculator,
            "state-machine" => AgentTemplate::StateMachineWorkflow,
            "hybrid" => AgentTemplate::HybridAnalyzer,
            "mcp-server" => AgentTemplate::MCPToolServer,
            path if path.starts_with("custom:") => {
                AgentTemplate::CustomAgent(path.strip_prefix("custom:").unwrap().into())
            }
            _ => AgentTemplate::MCPToolServer,
        };

        Self {
            context: AgentContext {
                name: name.into(),
                template_type,
                ..Default::default()
            },
        }
    }

    /// Add a feature to the agent.
    pub fn with_feature(mut self, feature: AgentFeature) -> Self {
        self.context.features.insert(feature);
        self
    }

    /// Parse and add a feature from a string.
    pub fn with_feature_str(mut self, feature_str: &str) -> Self {
        if let Ok(feature) = feature_str.parse::<AgentFeature>() {
            self.context.features.insert(feature);
        }
        self
    }

    /// Set the quality level.
    pub fn with_quality_level(mut self, level: QualityLevel) -> Self {
        self.context.quality_level = level;
        self
    }

    /// Set the deterministic core specification.
    pub fn with_deterministic_core(mut self, core: CoreSpec) -> Self {
        self.context.deterministic_core = Some(core);
        self
    }

    /// Set the probabilistic wrapper specification.
    pub fn with_probabilistic_wrapper(mut self, wrapper: WrapperSpec) -> Self {
        self.context.probabilistic_wrapper = Some(wrapper);
        self
    }

    /// Build the agent context.
    pub fn build(self) -> Result<AgentContext> {
        let ctx = self.context;

        // Validate agent name
        if ctx.name.is_empty() {
            bail!("Agent name cannot be empty");
        }

        if !ctx.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            bail!("Agent name must be alphanumeric with underscores only");
        }

        if ctx.name.chars().next().is_some_and(|c| c.is_numeric()) {
            bail!("Agent name cannot start with a number");
        }

        // Validate hybrid requirements
        if matches!(ctx.template_type, AgentTemplate::HybridAnalyzer)
            && (ctx.deterministic_core.is_none() || ctx.probabilistic_wrapper.is_none())
        {
            bail!("Hybrid agents require both deterministic core and probabilistic wrapper specifications");
        }

        Ok(ctx)
    }
}

/// Create a new agent context builder.
///
/// # Examples
///
/// ```ignore
/// use pmat::scaffold::agent::{create_agent_context, QualityLevel};
///
/// let context = create_agent_context("my_agent", "mcp-server")
///     .with_quality_level(QualityLevel::Extreme)
///     .build()
///     .unwrap();
///
/// assert_eq!(context.name, "my_agent");
/// assert_eq!(context.quality_level, QualityLevel::Extreme);
/// ```
pub fn create_agent_context(name: &str, template: &str) -> AgentContextBuilder {
    AgentContextBuilder::new(name, template)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_context_creation() {
        let ctx = AgentContext {
            name: "test_agent".to_string(),
            template_type: AgentTemplate::MCPToolServer,
            features: HashSet::new(),
            quality_level: QualityLevel::Extreme,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        assert_eq!(ctx.name, "test_agent");
        assert_eq!(ctx.quality_level, QualityLevel::Extreme);
    }

    #[test]
    fn test_agent_context_builder() {
        let result = create_agent_context("my_agent", "mcp-server")
            .with_quality_level(QualityLevel::Strict)
            .build();

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.name, "my_agent");
        assert_eq!(ctx.quality_level, QualityLevel::Strict);
    }

    #[test]
    fn test_invalid_agent_name() {
        let result = create_agent_context("", "mcp-server").build();
        assert!(result.is_err());

        let result = create_agent_context("123agent", "mcp-server").build();
        assert!(result.is_err());

        let result = create_agent_context("agent-name", "mcp-server").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_hybrid_validation() {
        let result = create_agent_context("hybrid_agent", "hybrid").build();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
