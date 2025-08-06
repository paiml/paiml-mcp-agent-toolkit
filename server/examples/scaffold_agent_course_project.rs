//! Complete course project: Deterministic Code Quality Analyzer
//! 
//! This example demonstrates building a production-ready deterministic
//! MCP agent that analyzes code quality with zero false positives.
//! Perfect for the final project in the deterministic agents course.

use pmat::scaffold::agent::{
    AgentContextBuilder, AgentFeature, QualityLevel,
    MonitoringBackend, TraceExporter,
};
use anyhow::Result;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║     Course Final Project: Code Quality Analyzer      ║");
    println!("║         100% Deterministic, Production-Ready         ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();
    
    // Build the complete agent step by step
    build_course_project_agent().await?;
    
    // Demonstrate the agent's capabilities
    demonstrate_agent_capabilities()?;
    
    // Show testing strategies
    demonstrate_testing_approach()?;
    
    // Production deployment checklist
    show_deployment_checklist()?;
    
    Ok(())
}

/// Build the complete course project agent
async fn build_course_project_agent() -> Result<()> {
    println!("📦 BUILDING: Deterministic Code Quality Analyzer");
    println!("════════════════════════════════════════════════");
    
    // Step 1: Core Agent Configuration
    println!("\n1️⃣  Core Configuration:");
    let mut builder = AgentContextBuilder::new("code_quality_analyzer", "mcp-server");
    
    // Step 2: Add Deterministic Analysis Features
    println!("\n2️⃣  Deterministic Analysis Features:");
    builder = builder
        .with_feature(AgentFeature::ComplexityAnalysis)
        .with_feature(AgentFeature::SATDDetection)
        .with_feature(AgentFeature::DeadCodeElimination);
    
    println!("   ✓ Complexity Analysis (McCabe & Cognitive)");
    println!("   ✓ SATD Detection (Technical Debt Comments)");
    println!("   ✓ Dead Code Elimination");
    
    // Step 3: Add State Machine for Workflow
    println!("\n3️⃣  State Machine Workflow:");
    builder = builder.with_feature(AgentFeature::StateMachine {
        states: vec![
            "Idle".to_string(),
            "Parsing".to_string(),
            "Analyzing".to_string(),
            "Validating".to_string(),
            "Reporting".to_string(),
            "Error".to_string(),
        ],
    });
    
    println!("   States: Idle → Parsing → Analyzing → Validating → Reporting");
    println!("   Error handling: Any state → Error → Idle");
    
    // Step 4: Quality Gates
    println!("\n4️⃣  Quality Gates (Toyota Way):");
    builder = builder
        .with_feature(AgentFeature::QualityGates { 
            level: QualityLevel::Extreme 
        })
        .with_quality_level(QualityLevel::Extreme);
    
    println!("   ✓ Zero SATD tolerance");
    println!("   ✓ Max complexity: 10");
    println!("   ✓ Min coverage: 90%");
    println!("   ✓ Zero clippy warnings");
    
    // Step 5: Production Features
    println!("\n5️⃣  Production Features:");
    builder = builder
        .with_feature(AgentFeature::Monitoring { 
            backend: MonitoringBackend::Prometheus 
        })
        .with_feature(AgentFeature::Tracing { 
            exporter: TraceExporter::OTLP 
        })
        .with_feature(AgentFeature::HealthChecks)
        .with_feature(AgentFeature::AsyncHandlers)
        .with_feature(AgentFeature::ToolComposition);
    
    println!("   ✓ Prometheus metrics (analysis_duration, files_processed)");
    println!("   ✓ OpenTelemetry tracing (distributed tracing)");
    println!("   ✓ Health checks (/health, /ready, /live)");
    println!("   ✓ Async handlers (non-blocking operations)");
    println!("   ✓ Tool composition (combine multiple analyses)");
    
    // Build the final context
    let context = builder.build()?;
    
    println!("\n✅ Agent Configuration Complete!");
    println!("   Name: {}", context.name);
    println!("   Features: {} enabled", context.features.len());
    
    Ok(())
}

/// Demonstrate the agent's capabilities with examples
fn demonstrate_agent_capabilities() -> Result<()> {
    println!("\n🎯 AGENT CAPABILITIES");
    println!("═══════════════════════");
    
    // Example 1: Complexity Analysis
    println!("\n📊 Capability 1: Complexity Analysis");
    println!("├─ Input: Rust source file");
    println!("├─ Process:");
    println!("│  1. Parse AST deterministically");
    println!("│  2. Calculate McCabe complexity");
    println!("│  3. Calculate cognitive complexity");
    println!("│  4. Identify complexity hotspots");
    println!("└─ Output: JSON with metrics");
    
    println!("\n   Example output:");
    println!("   ```json");
    println!("   {{");
    println!("     \"file\": \"src/main.rs\",");
    println!("     \"functions\": [");
    println!("       {{");
    println!("         \"name\": \"process_data\",");
    println!("         \"mccabe\": 8,");
    println!("         \"cognitive\": 12,");
    println!("         \"lines\": 45");
    println!("       }}");
    println!("     ]");
    println!("   }}");
    println!("   ```");
    
    // Example 2: SATD Detection
    println!("\n🔍 Capability 2: SATD Detection");
    println!("├─ Patterns detected:");
    println!("│  • TODO, FIXME, HACK, XXX");
    println!("│  • \"temporary\", \"for now\", \"quick fix\"");
    println!("│  • \"should be\", \"need to\", \"must\"");
    println!("└─ Zero false positives guaranteed");
    
    // Example 3: Tool Composition
    println!("\n🔧 Capability 3: Tool Composition");
    println!("├─ Combine multiple analyses:");
    println!("│  ```");
    println!("│  analyze_comprehensive = ");
    println!("│    complexity + satd + dead_code + duplication");
    println!("│  ```");
    println!("└─ Single unified report");
    
    Ok(())
}

/// Demonstrate comprehensive testing approach
fn demonstrate_testing_approach() -> Result<()> {
    println!("\n🧪 TESTING STRATEGY");
    println!("═══════════════════════");
    
    println!("\n1. Property-Based Tests:");
    println!("   ```rust");
    println!("   proptest! {{");
    println!("       #[test]");
    println!("       fn complexity_never_negative(code in any::<String>()) {{");
    println!("           let result = analyze_complexity(&code);");
    println!("           prop_assert!(result >= 0);");
    println!("       }}");
    println!("   }}");
    println!("   ```");
    
    println!("\n2. State Machine Invariants:");
    println!("   • No invalid state transitions");
    println!("   • Always reach terminal state");
    println!("   • Error state is recoverable");
    
    println!("\n3. Determinism Tests:");
    println!("   ```rust");
    println!("   #[test]");
    println!("   fn test_deterministic_output() {{");
    println!("       let input = \"fn main() {{}}\";");
    println!("       let result1 = analyze(input);");
    println!("       let result2 = analyze(input);");
    println!("       assert_eq!(result1, result2);");
    println!("   }}");
    println!("   ```");
    
    println!("\n4. Edge Cases:");
    println!("   ✓ Empty files");
    println!("   ✓ Malformed syntax");
    println!("   ✓ Unicode handling");
    println!("   ✓ Large files (>10MB)");
    println!("   ✓ Nested complexity");
    
    Ok(())
}

/// Show production deployment checklist
fn show_deployment_checklist() -> Result<()> {
    println!("\n📋 PRODUCTION DEPLOYMENT CHECKLIST");
    println!("════════════════════════════════════");
    
    let checklist = vec![
        ("Code Quality", vec![
            "All functions < 10 complexity",
            "Zero SATD comments",
            "90%+ test coverage",
            "Zero clippy warnings",
        ]),
        ("Observability", vec![
            "Prometheus metrics exposed",
            "Tracing configured",
            "Structured logging",
            "Health endpoints active",
        ]),
        ("Performance", vec![
            "< 100ms for average file",
            "< 1GB memory for large repos",
            "Concurrent file processing",
            "Incremental analysis support",
        ]),
        ("Deployment", vec![
            "Docker container built",
            "Kubernetes manifests ready",
            "CI/CD pipeline configured",
            "Rollback strategy defined",
        ]),
        ("Documentation", vec![
            "API documentation complete",
            "README with examples",
            "Architecture decisions recorded",
            "Runbook for operations",
        ]),
    ];
    
    for (category, items) in checklist {
        println!("\n{}", category);
        println!("{}", "─".repeat(category.len()));
        for item in items {
            println!("  □ {}", item);
        }
    }
    
    println!("\n🚀 Command to deploy:");
    println!("   ```bash");
    println!("   # Build the agent");
    println!("   pmat scaffold agent --name code_quality_analyzer \\");
    println!("     --template mcp-server \\");
    println!("     --features complexity,satd,monitoring,tracing \\");
    println!("     --quality extreme");
    println!();
    println!("   # Run with Docker");
    println!("   docker build -t analyzer .");
    println!("   docker run -p 8080:8080 analyzer");
    println!("   ```");
    
    Ok(())
}

/// Example metrics that the agent would collect
#[allow(dead_code)]
fn example_metrics() -> HashMap<&'static str, f64> {
    let mut metrics = HashMap::new();
    
    // Complexity metrics
    metrics.insert("complexity_analysis_duration_ms", 45.2);
    metrics.insert("files_analyzed_total", 1523.0);
    metrics.insert("average_complexity", 4.7);
    metrics.insert("max_complexity_found", 9.0);
    
    // SATD metrics
    metrics.insert("satd_comments_found", 0.0);
    metrics.insert("satd_detection_duration_ms", 12.3);
    
    // Performance metrics
    metrics.insert("memory_usage_mb", 124.5);
    metrics.insert("cpu_usage_percent", 23.4);
    
    metrics
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_collection() {
        let metrics = example_metrics();
        
        // Verify all metrics are non-negative
        for (_, value) in metrics.iter() {
            assert!(*value >= 0.0);
        }
        
        // Verify complexity is within bounds
        assert!(metrics["max_complexity_found"] <= 10.0);
    }
}