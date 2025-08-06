//! Scaffolding system for generating projects and agents.

pub mod agent;

// Re-export key types for convenience
pub use agent::{
    scaffold_agent, AgentContext, AgentContextBuilder, AgentFeature, AgentTemplate,
    InteractiveScaffolder, QualityLevel, ScaffoldError, TemplateRegistry,
};