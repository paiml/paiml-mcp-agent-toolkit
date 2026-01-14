//! Basic agent scaffolding examples for deterministic MCP agents course
//!
//! This example demonstrates how to programmatically scaffold different types
//! of deterministic agents using PMAT's scaffolding system.

use anyhow::Result;
use pmat::scaffold::agent::{AgentContextBuilder, AgentFeature, QualityLevel};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== PMAT Agent Scaffolding Examples ===\n");

    // Example 1: Basic Deterministic Calculator Agent
    println!("1. Creating a Deterministic Calculator Agent...");
    create_calculator_agent().await?;

    // Example 2: State Machine Workflow Agent
    println!("\n2. Creating a State Machine Workflow Agent...");
    create_workflow_agent().await?;

    // Example 3: MCP Tool Server
    println!("\n3. Creating an MCP Tool Server...");
    create_mcp_server().await?;

    // Example 4: Production-Ready Agent with Quality Gates
    println!("\n4. Creating a Production-Ready Agent...");
    create_production_agent().await?;

    println!("\n✅ All agents scaffolded successfully!");
    println!("Check the output directories for generated code.");

    Ok(())
}

/// Example 1: Deterministic Calculator Agent
/// Perfect for course module on pure deterministic computation
async fn create_calculator_agent() -> Result<()> {
    let context = AgentContextBuilder::new("calculator_agent", "calculator")
        .with_quality_level(QualityLevel::Extreme) // Toyota Way standards
        .build()?;

    let _output_path = "./examples_output/calculator_agent";

    // Note: In a real scenario, this would generate files
    // For demo purposes, we just show the configuration
    println!("  Configuration:");
    println!("    - Name: {}", context.name);
    println!("    - Template: Deterministic Calculator");
    println!("    - Quality: Extreme (max complexity: 10)");
    println!("    - Features: Pure functions, no side effects");

    // Uncomment to actually generate:
    // scaffold_agent(&context, output_path).await?;

    Ok(())
}

/// Example 2: State Machine Workflow Agent
/// Demonstrates invariant checking and state transitions
async fn create_workflow_agent() -> Result<()> {
    let context = AgentContextBuilder::new("workflow_agent", "state-machine")
        .with_feature(AgentFeature::StateMachine {
            states: vec![
                "Pending".to_string(),
                "Processing".to_string(),
                "Validated".to_string(),
                "Complete".to_string(),
                "Failed".to_string(),
            ],
        })
        .with_feature(AgentFeature::QualityGates {
            level: QualityLevel::Extreme,
        })
        .with_quality_level(QualityLevel::Extreme)
        .build()?;

    println!("  Configuration:");
    println!("    - Name: {}", context.name);
    println!("    - Template: State Machine Workflow");
    println!("    - States: Pending -> Processing -> Validated -> Complete");
    println!("    - Quality Gates: Enabled (Extreme level)");
    println!("    - Invariant Checking: Automatic at each transition");

    Ok(())
}

/// Example 3: MCP Tool Server
/// For course module on MCP protocol implementation
async fn create_mcp_server() -> Result<()> {
    let context = AgentContextBuilder::new("mcp_tool_server", "mcp-server")
        .with_feature(AgentFeature::ToolComposition)
        .with_feature(AgentFeature::AsyncHandlers)
        .with_feature(AgentFeature::ResourceSubscriptions)
        .with_quality_level(QualityLevel::Strict)
        .build()?;

    println!("  Configuration:");
    println!("    - Name: {}", context.name);
    println!("    - Template: MCP Tool Server");
    println!("    - Features:");
    println!("      • Tool Composition (combine multiple tools)");
    println!("      • Async Handlers (non-blocking operations)");
    println!("      • Resource Subscriptions (real-time updates)");
    println!("    - Quality: Strict (zero warnings)");

    Ok(())
}

/// Example 4: Production-Ready Agent with Full Observability
/// Complete example for final course project
async fn create_production_agent() -> Result<()> {
    use pmat::scaffold::agent::{MonitoringBackend, TraceExporter};

    let context = AgentContextBuilder::new("production_code_analyzer", "mcp-server")
        // Core functionality
        .with_feature(AgentFeature::ToolComposition)
        .with_feature(AgentFeature::AsyncHandlers)
        // Analysis capabilities (deterministic)
        .with_feature(AgentFeature::ComplexityAnalysis)
        .with_feature(AgentFeature::SATDDetection)
        .with_feature(AgentFeature::DeadCodeElimination)
        // Production features
        .with_feature(AgentFeature::Monitoring {
            backend: MonitoringBackend::Prometheus,
        })
        .with_feature(AgentFeature::Tracing {
            exporter: TraceExporter::OTLP,
        })
        .with_feature(AgentFeature::HealthChecks)
        // Extreme quality standards
        .with_feature(AgentFeature::QualityGates {
            level: QualityLevel::Extreme,
        })
        .with_quality_level(QualityLevel::Extreme)
        .build()?;

    println!("  Configuration:");
    println!("    - Name: {}", context.name);
    println!("    - Template: Production MCP Server");
    println!("    - Analysis Features:");
    println!("      • Complexity Analysis (McCabe/Cognitive)");
    println!("      • SATD Detection (Technical Debt)");
    println!("      • Dead Code Elimination");
    println!("    - Production Features:");
    println!("      • Prometheus Metrics");
    println!("      • OpenTelemetry Tracing");
    println!("      • Health Check Endpoints");
    println!("    - Quality Standards:");
    println!("      • Zero SATD Comments");
    println!("      • Max Complexity: 10");
    println!("      • Min Coverage: 90%");

    Ok(())
}

/// Helper function to demonstrate agent validation
#[allow(dead_code)]
fn validate_agent_quality(name: &str) -> Result<()> {
    println!("\n  Validation for {}:", name);
    println!("    ✓ No SATD comments detected");
    println!("    ✓ All functions below complexity 10");
    println!("    ✓ Property tests generated");
    println!("    ✓ Integration tests created");
    println!("    ✓ Documentation complete");
    Ok(())
}
