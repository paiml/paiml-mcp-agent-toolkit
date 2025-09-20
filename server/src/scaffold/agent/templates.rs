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

#[async_trait]
impl TemplateGenerator for MCPServerTemplate {
    fn generate(&self, ctx: &AgentContext) -> Result<GeneratedFiles> {
        let mut files = GeneratedFiles::new();

        // Generate Cargo.toml
        files.add_text_file("Cargo.toml", generate_mcp_cargo_toml(ctx));

        // Generate main.rs
        files.add_text_file("src/main.rs", generate_mcp_main(ctx));

        // Generate MCP server implementation
        files.add_text_file("src/mcp/mod.rs", generate_mcp_mod());
        files.add_text_file("src/mcp/server.rs", generate_mcp_server(ctx));
        files.add_text_file("src/mcp/tools.rs", generate_mcp_tools());
        files.add_text_file("src/mcp/transport.rs", generate_mcp_transport());

        // Generate agent core
        files.add_text_file("src/agent/mod.rs", generate_agent_mod());
        files.add_text_file("src/agent/core.rs", generate_agent_core(ctx));
        files.add_text_file("src/agent/handlers.rs", generate_agent_handlers());

        // Generate quality module if needed
        if ctx.quality_level != QualityLevel::Standard {
            files.add_text_file("src/quality/mod.rs", generate_quality_mod());
            files.add_text_file("src/quality/invariants.rs", generate_invariants());
            files.add_text_file("src/quality/validators.rs", generate_validators());
        }

        // Generate tests
        files.add_text_file("tests/integration.rs", generate_integration_tests(ctx));
        files.add_text_file("tests/deterministic.rs", generate_deterministic_tests());

        // Generate configuration files
        files.add_text_file(".pmat/agent.toml", generate_agent_config(ctx));
        files.add_text_file(".pmat/quality-gates.toml", generate_quality_gates(ctx));

        // Generate README
        files.add_text_file("README.md", generate_readme(ctx));

        Ok(files)
    }

    fn validate_context(&self, ctx: &AgentContext) -> Result<()> {
        if ctx.name.is_empty() {
            bail!("Agent name is required");
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
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

#[async_trait]
impl TemplateGenerator for StateMachineTemplate {
    fn generate(&self, ctx: &AgentContext) -> Result<GeneratedFiles> {
        let mut files = GeneratedFiles::new();

        // Generate Cargo.toml
        files.add_text_file("Cargo.toml", generate_state_machine_cargo_toml(ctx));

        // Generate main.rs
        files.add_text_file("src/main.rs", generate_state_machine_main(ctx));

        // Generate state machine implementation
        files.add_text_file("src/agent/mod.rs", generate_state_machine_mod());
        files.add_text_file("src/agent/state.rs", generate_state_definitions(ctx));
        files.add_text_file("src/agent/transitions.rs", generate_transitions());
        files.add_text_file("src/agent/invariants.rs", generate_state_invariants());

        // Generate tests
        files.add_text_file("tests/state_transitions.rs", generate_state_tests());
        files.add_text_file("tests/invariants.rs", generate_invariant_tests());

        Ok(files)
    }

    fn validate_context(&self, ctx: &AgentContext) -> Result<()> {
        if ctx.name.is_empty() {
            bail!("Agent name is required");
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

// Helper functions for generating file contents

fn generate_mcp_cargo_toml(ctx: &AgentContext) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
pmcp = "0.3.1"
tokio = {{ version = "1.40", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
async-trait = "0.1"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}

[dev-dependencies]
proptest = "1.5"
tokio-test = "0.4"
criterion = "0.5"

[[bench]]
name = "performance"
harness = false
"#,
        ctx.name
    )
}

fn generate_mcp_main(ctx: &AgentContext) -> String {
    format!(
        r#"//! {} - MCP Agent Server

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod agent;
mod mcp;
{}

#[tokio::main]
async fn main() -> Result<()> {{
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting {} agent server");

    // Start MCP server
    mcp::server::run().await
}}
"#,
        ctx.name,
        if ctx.quality_level == QualityLevel::Standard {
            ""
        } else {
            "mod quality;"
        },
        ctx.name
    )
}

fn generate_mcp_mod() -> String {
    r"//! MCP server implementation.

pub mod server;
pub mod tools;
pub mod transport;
"
    .to_string()
}

fn generate_mcp_server(ctx: &AgentContext) -> String {
    format!(
        r#"//! MCP server implementation for {}.

use anyhow::Result;
use pmcp::{{Server, ServerBuilder}};
use tracing::info;

use crate::agent::core::AgentCore;
use super::tools::register_tools;
use super::transport::create_transport;

/// Run the MCP server.
pub async fn run() -> Result<()> {{
    let transport = create_transport().await?;
    let agent = AgentCore::new();
    
    let server = ServerBuilder::new()
        .name("{}")
        .version(env!("CARGO_PKG_VERSION"))
        .build();

    register_tools(&server, agent)?;

    info!("MCP server listening");
    server.serve(transport).await?;
    
    Ok(())
}}
"#,
        ctx.name, ctx.name
    )
}

fn generate_mcp_tools() -> String {
    r#"//! Tool definitions for MCP server.

use anyhow::Result;
use pmcp::{Server, Tool};
use serde_json::Value;

use crate::agent::core::AgentCore;

/// Register all tools with the MCP server.
pub fn register_tools(server: &Server, agent: AgentCore) -> Result<()> {
    // Register analyze tool
    server.register_tool(Tool {
        name: "analyze".to_string(),
        description: "Analyze input data".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "string",
                    "description": "Data to analyze"
                }
            },
            "required": ["data"]
        }),
        handler: Box::new(move |params: Value| {
            let agent = agent.clone();
            Box::pin(async move {
                agent.analyze(params).await
            })
        }),
    });

    Ok(())
}
"#
    .to_string()
}

fn generate_mcp_transport() -> String {
    r"//! Transport layer for MCP server.

use anyhow::Result;
use pmcp::transport::{StdioTransport, Transport};

/// Create the transport layer for the MCP server.
pub async fn create_transport() -> Result<Box<dyn Transport>> {
    Ok(Box::new(StdioTransport::new()))
}
"
    .to_string()
}

fn generate_agent_mod() -> String {
    r"//! Agent implementation.

pub mod core;
pub mod handlers;
"
    .to_string()
}

fn generate_agent_core(ctx: &AgentContext) -> String {
    format!(
        r#"//! Core agent implementation for {}.

use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core agent implementation.
#[derive(Clone)]
pub struct AgentCore {{
    state: Arc<RwLock<AgentState>>,
}}

/// Agent state.
#[derive(Default)]
struct AgentState {{
    // Add state fields as needed
}}

impl AgentCore {{
    /// Create a new agent instance.
    pub fn new() -> Self {{
        Self {{
            state: Arc::new(RwLock::new(AgentState::default())),
        }}
    }}

    /// Analyze input data.
    pub async fn analyze(&self, params: Value) -> Result<Value> {{
        // Implementation here
        Ok(serde_json::json!({{
            "status": "success",
            "result": "Analysis complete"
        }}))
    }}
}}
"#,
        ctx.name
    )
}

fn generate_agent_handlers() -> String {
    r#"//! Request handlers for the agent.

use anyhow::Result;
use serde_json::Value;

/// Handle a request.
pub async fn handle_request(request: Value) -> Result<Value> {
    // Implementation here
    Ok(serde_json::json!({
        "status": "success"
    }))
}
"#
    .to_string()
}

fn generate_quality_mod() -> String {
    r"//! Quality enforcement module.

pub mod invariants;
pub mod validators;
"
    .to_string()
}

fn generate_invariants() -> String {
    r"//! Runtime invariants for quality enforcement.

use anyhow::Result;

/// Check all invariants.
pub fn check_invariants() -> Result<()> {
    // Add invariant checks here
    Ok(())
}
"
    .to_string()
}

fn generate_validators() -> String {
    r"//! Input and output validators.

use anyhow::Result;
use serde_json::Value;

/// Validate input.
pub fn validate_input(input: &Value) -> Result<()> {
    // Add validation logic here
    Ok(())
}

/// Validate output.
pub fn validate_output(output: &Value) -> Result<()> {
    // Add validation logic here
    Ok(())
}
"
    .to_string()
}

fn generate_integration_tests(ctx: &AgentContext) -> String {
    format!(
        r"//! Integration tests for {}.

#[tokio::test]
async fn test_agent_startup() {{
    // Test agent initialization
}}

#[tokio::test]
async fn test_tool_invocation() {{
    // Test tool invocation
}}
",
        ctx.name
    )
}

fn generate_deterministic_tests() -> String {
    r"//! Determinism verification tests.

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_deterministic_behavior(input in any::<String>()) {
        // Verify deterministic behavior
    }
}
"
    .to_string()
}

fn generate_agent_config(ctx: &AgentContext) -> String {
    format!(
        r#"# Agent configuration
[agent]
name = "{}"
version = "0.1.0"
template = "mcp-server"

[quality]
level = "{:?}"
"#,
        ctx.name, ctx.quality_level
    )
}

fn generate_quality_gates(ctx: &AgentContext) -> String {
    let level = &ctx.quality_level;
    format!(
        r#"# Quality gate configuration
[complexity]
max_cyclomatic = {}
max_cognitive = {}
max_nesting = {}

[coverage]
min_line = {}
min_branch = {}
min_function = {}

[linting]
deny = ["unsafe_code", "missing_docs", "clippy::all"]
warn = ["clippy::pedantic"]

[satd]
allowed_markers = []
scan_comments = true
"#,
        level.max_complexity(),
        level.max_cognitive_complexity(),
        level.max_nesting(),
        level.min_line_coverage(),
        level.min_branch_coverage(),
        level.min_function_coverage()
    )
}

fn generate_readme(ctx: &AgentContext) -> String {
    format!(
        r"# {}

MCP Agent Server generated with PMAT.

## Quick Start

```bash
cargo build --release
cargo run
```

## Testing

```bash
cargo test
cargo test --test integration
```

## Quality

This agent follows {} quality standards.
",
        ctx.name,
        match ctx.quality_level {
            QualityLevel::Standard => "standard",
            QualityLevel::Strict => "strict",
            QualityLevel::Extreme => "Toyota Way extreme",
        }
    )
}

// State machine template helpers

fn generate_state_machine_cargo_toml(ctx: &AgentContext) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
tokio = {{ version = "1.40", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
anyhow = "1.0"
tracing = "0.1"

[dev-dependencies]
proptest = "1.5"
tokio-test = "0.4"
"#,
        ctx.name
    )
}

fn generate_state_machine_main(ctx: &AgentContext) -> String {
    format!(
        r#"//! {} - State Machine Agent

use anyhow::Result;

mod agent;

#[tokio::main]
async fn main() -> Result<()> {{
    println!("Starting {} state machine");
    // Implementation here
    Ok(())
}}
"#,
        ctx.name, ctx.name
    )
}

fn generate_state_machine_mod() -> String {
    r"//! State machine implementation.

pub mod state;
pub mod transitions;
pub mod invariants;
"
    .to_string()
}

fn generate_state_definitions(ctx: &AgentContext) -> String {
    format!(
        r"//! State definitions for {}.

use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum State {{
    Initial,
    Processing,
    Complete,
    Error(String),
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {{
    Start,
    Process,
    Finish,
    Fail(String),
}}
",
        ctx.name
    )
}

fn generate_transitions() -> String {
    r"//! State transition logic.

use anyhow::Result;
use super::state::{State, Event};

/// Apply a transition.
pub fn transition(state: &State, event: &Event) -> Result<State> {
    match (state, event) {
        (State::Initial, Event::Start) => Ok(State::Processing),
        (State::Processing, Event::Finish) => Ok(State::Complete),
        (State::Processing, Event::Fail(msg)) => Ok(State::Error(msg.clone())),
        _ => Ok(state.clone()),
    }
}
"
    .to_string()
}

fn generate_state_invariants() -> String {
    r#"//! State invariants.

use anyhow::Result;
use super::state::State;

/// Check state invariants.
pub fn check_invariants(state: &State) -> Result<()> {
    match state {
        State::Error(msg) if msg.is_empty() => {
            anyhow::bail!("Error state must have a message");
        }
        _ => Ok(()),
    }
}
"#
    .to_string()
}

fn generate_state_tests() -> String {
    r"//! State transition tests.

#[test]
fn test_valid_transitions() {
    // Test valid state transitions
}

#[test]
fn test_invalid_transitions() {
    // Test invalid state transitions
}
"
    .to_string()
}

fn generate_invariant_tests() -> String {
    r"//! Invariant tests.

#[test]
fn test_state_invariants() {
    // Test state invariants
}
"
    .to_string()
}

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
