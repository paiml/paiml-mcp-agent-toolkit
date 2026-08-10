// MCP server template generation: TemplateGenerator impl and all
// generation functions for Cargo.toml, main.rs, MCP server/tools/transport,
// agent core/handlers, quality module, tests, config, and README.

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

/// The manifest declares only targets the scaffold actually writes.
///
/// It used to end with `[[bench]] name = "performance"` while no `benches/`
/// directory was ever generated, so the scaffolded project was dead on arrival:
/// `cargo check` failed at manifest PARSE time with "can't find `performance`
/// bench at `benches/performance.rs`", before a single line of the generated
/// code was compiled. A declared target with no file (and the criterion
/// dev-dependency that only existed to serve it) is a promise the scaffold does
/// not keep.
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

// None of the generated sources below carry a
// `#[provable_contracts_macros::contract(...)]` attribute any more. pmat's own
// sources do, and the attribute was copied into the templates — but
// `provable_contracts_macros` is not among the `[dependencies]` the scaffold
// writes, so every generated file that used it failed to resolve the macro
// crate. Generated code may only use what the generated manifest declares.
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
    r#"//! Transport layer for MCP server.

use anyhow::Result;
use pmcp::transport::{StdioTransport, Transport};

/// Create the transport layer for the MCP server.
pub async fn create_transport() -> Result<Box<dyn Transport>> {
    Ok(Box::new(StdioTransport::new()))
}
"#
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod mcp_scaffold_manifest_tests {
    use super::*;

    fn generate_mcp_project(quality: QualityLevel) -> GeneratedFiles {
        MCPServerTemplate::default()
            .generate(&AgentContext {
                name: "ag".to_string(),
                template_type: AgentTemplate::MCPToolServer,
                features: Default::default(),
                quality_level: quality,
                deterministic_core: None,
                probabilistic_wrapper: None,
            })
            .expect("mcp-server template must generate")
    }

    /// Collect `(target directory, target name)` for every `[[bench]]`,
    /// `[[bin]]` and `[[example]]` section declared in a manifest.
    fn declared_targets(manifest: &str) -> Vec<(&'static str, String)> {
        let mut targets = Vec::new();
        let mut section: Option<&'static str> = None;
        for line in manifest.lines() {
            let trimmed = line.trim();
            match trimmed {
                "[[bench]]" => section = Some("benches"),
                "[[bin]]" => section = Some("src/bin"),
                "[[example]]" => section = Some("examples"),
                _ if trimmed.starts_with('[') => section = None,
                _ => {
                    if let (Some(dir), Some(rest)) = (section, trimmed.strip_prefix("name")) {
                        let name = rest.trim_start_matches(['=', ' ']).trim_matches('"');
                        targets.push((dir, name.to_string()));
                    }
                }
            }
        }
        targets
    }

    /// The generated manifest declared `[[bench]] name = "performance"` while
    /// the scaffold wrote no `benches/` directory, so `cargo check` in a freshly
    /// scaffolded project died before compiling anything: "can't find
    /// `performance` bench at `benches/performance.rs`". A manifest that cannot
    /// be parsed is a project that was never generated.
    #[test]
    fn every_declared_target_has_a_generated_file() {
        for quality in [
            QualityLevel::Standard,
            QualityLevel::Strict,
            QualityLevel::Extreme,
        ] {
            let files = generate_mcp_project(quality);
            let manifest = files
                .files
                .get(&PathBuf::from("Cargo.toml"))
                .expect("Cargo.toml generated")
                .as_str()
                .expect("text manifest")
                .to_string();

            for (dir, name) in declared_targets(&manifest) {
                let expected = PathBuf::from(format!("{dir}/{name}.rs"));
                assert!(
                    files.files.contains_key(&expected),
                    "manifest declares {dir} target {name:?} but {} was never generated",
                    expected.display()
                );
            }
        }
    }

    /// `src/mcp/server.rs` (and four more generated files) carried
    /// `#[provable_contracts_macros::contract(...)]`, copied from pmat's own
    /// sources, while that crate appears nowhere in the generated
    /// `[dependencies]`.
    #[test]
    fn generated_sources_use_only_declared_dependencies() {
        let files = generate_mcp_project(QualityLevel::Extreme);
        let manifest = files
            .files
            .get(&PathBuf::from("Cargo.toml"))
            .expect("Cargo.toml generated")
            .as_str()
            .expect("text manifest")
            .to_string();

        for (path, content) in &files.files {
            let Some(text) = content.as_str() else {
                continue;
            };
            if !path.to_string_lossy().ends_with(".rs") {
                continue;
            }
            assert!(
                !text.contains("provable_contracts_macros"),
                "{} uses provable_contracts_macros, which the manifest does not declare:\n{manifest}",
                path.display()
            );
        }
    }
}
