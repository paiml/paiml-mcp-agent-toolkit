#![cfg_attr(coverage_nightly, coverage(off))]
//! Agent template definitions and implementations.

use super::context::AgentContext;
use super::features::QualityLevel;
use super::generator::{GeneratedFiles, TemplateGenerator};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Available agent templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentTemplate {
    /// Deterministic calculator agent.
    DeterministicCalculator,
    /// State machine workflow agent.
    StateMachineWorkflow,
    /// Hybrid analyzer with deterministic core and probabilistic wrapper.
    HybridAnalyzer,
    /// MCP tool server agent.
    MCPToolServer,
    /// Custom agent template from a path.
    CustomAgent(PathBuf),
}

/// MCP server template generator.
pub struct MCPServerTemplate {
    /// Template name.
    name: String,
    /// Template description.
    description: String,
}

impl Default for MCPServerTemplate {
    fn default() -> Self {
        Self {
            name: "mcp-server".to_string(),
            description: "MCP tool server with async handlers and resource management".to_string(),
        }
    }
}

/// State machine template generator.
pub struct StateMachineTemplate {
    /// Template name.
    name: String,
    /// Template description.
    description: String,
}

impl Default for StateMachineTemplate {
    fn default() -> Self {
        Self {
            name: "state-machine".to_string(),
            description: "State machine agent with transitions and invariants".to_string(),
        }
    }
}

// MCP server template generation: Cargo.toml, main.rs, server, tools, transport,
// agent core/handlers, quality module, tests, config, and README generators.
include!("templates_mcp_generation.rs");

// State machine template generation: Cargo.toml, main.rs, state definitions,
// transitions, invariants, and test generators.
include!("templates_state_machine_generation.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_template_generation() {
        let template = MCPServerTemplate::default();
        let ctx = AgentContext {
            name: "test_agent".to_string(),
            template_type: AgentTemplate::MCPToolServer,
            features: Default::default(),
            quality_level: QualityLevel::Standard,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        let result = template.generate(&ctx);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.contains_file(&PathBuf::from("Cargo.toml")));
        assert!(files.contains_file(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_state_machine_template_generation() {
        let template = StateMachineTemplate::default();
        let ctx = AgentContext {
            name: "test_state_machine".to_string(),
            template_type: AgentTemplate::StateMachineWorkflow,
            features: Default::default(),
            quality_level: QualityLevel::Extreme,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        let result = template.generate(&ctx);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.contains_file(&PathBuf::from("src/agent/state.rs")));
    }
}
