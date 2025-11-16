// Organizational Intelligence Integration Example
// Demonstrates all 3 options: CLI, MCP, and Direct Integration

use anyhow::Result;
use pmat::prompts::DefectAwarePromptGenerator;

/// Option B: CLI-style usage (can be wrapped in actual CLI command later)
fn demonstrate_cli_usage() -> Result<()> {
    println!("\n========== Option B: CLI Usage ==========\n");

    // Simulates: pmat prompt generate --summary <file> --task <task>
    let summary_path = "/tmp/paiml-summary.yaml";

    if !std::path::Path::new(summary_path).exists() {
        println!("⚠️  Summary file not found. Run OIP analysis first.");
        println!("   cd ../organizational-intelligence-plugin");
        println!("   cargo run -- analyze --org paiml --output /tmp/paiml-full-analysis.yaml");
        println!("   cargo run -- summarize --input /tmp/paiml-full-analysis.yaml \\");
        println!("       --output /tmp/paiml-summary.yaml --strip-pii");
        return Ok(());
    }

    let generator = DefectAwarePromptGenerator::from_file(summary_path)?;

    let task = "Implement a new HTTP client for external API integration";
    let context = "Building a resilient service with retry logic and timeouts";

    let prompt = generator.generate_prompt(task, context);

    println!("Generated Prompt (truncated):");
    println!("{}", &prompt[..prompt.len().min(500)]);
    println!("...\n[{} total characters]\n", prompt.len());

    Ok(())
}

/// Option A: MCP Server Integration (simulated - actual implementation would be in mcp_pmcp/tool_functions.rs)
fn demonstrate_mcp_usage() -> Result<()> {
    println!("\n========== Option A: MCP Server Usage ==========\n");

    // This simulates what the MCP tool would do
    // Actual implementation would be:
    // async fn generate_defect_aware_prompt(
    //     task: String,
    //     context: String,
    //     summary_path: String,
    // ) -> Result<String>

    let summary_path = "/tmp/paiml-summary.yaml";

    if !std::path::Path::new(summary_path).exists() {
        println!("⚠️  Summary file not found.");
        return Ok(());
    }

    let generator = DefectAwarePromptGenerator::from_file(summary_path)?;

    // Simulate MCP tool call
    let mcp_request = serde_json::json!({
        "tool": "generate_defect_aware_prompt",
        "arguments": {
            "task": "Fix a performance bottleneck in API response",
            "context": "Users reporting 5+ second response times",
            "summary_path": summary_path
        }
    });

    println!("MCP Request:");
    println!("{}\n", serde_json::to_string_pretty(&mcp_request)?);

    let prompt = generator.generate_prompt(
        "Fix a performance bottleneck in API response",
        "Users reporting 5+ second response times",
    );

    let mcp_response = serde_json::json!({
        "tool": "generate_defect_aware_prompt",
        "result": prompt,
        "metadata": {
            "repositories_analyzed": generator.metadata.repositories_analyzed,
            "commits_analyzed": generator.metadata.commits_analyzed,
            "prompt_length": prompt.len()
        }
    });

    println!("MCP Response (metadata only):");
    println!(
        "{}\n",
        serde_json::to_string_pretty(&mcp_response["metadata"])?
    );

    Ok(())
}

/// Option C: Direct Org Analyze Integration
fn demonstrate_org_analyze_usage() -> Result<()> {
    println!("\n========== Option C: Org Analyze Integration ==========\n");

    // This would be: pmat org analyze --org paiml --output summary.yaml
    // Internally it would call OIP's functionality

    println!("Proposed Usage:");
    println!("  pmat org analyze --org paiml --output /tmp/paiml-summary.yaml");
    println!("  pmat org analyze --org mycompany --summarize --strip-pii\n");

    println!("Benefits:");
    println!("  ✅ Single command instead of 2 (analyze + summarize)");
    println!("  ✅ Integrated into pmat CLI");
    println!("  ✅ Can be used in CI/CD pipelines");
    println!("  ✅ Outputs directly to DefectAwarePromptGenerator format\n");

    println!("Implementation Options:");
    println!("  1. Call OIP binary as subprocess");
    println!("  2. Link to OIP library (if published to crates.io)");
    println!("  3. Re-implement core logic in pmat (code duplication)\n");

    println!("Recommendation: Option 1 (subprocess) for MVP\n");

    Ok(())
}

/// All-in-one example showing real-world workflow
fn demonstrate_end_to_end_workflow() -> Result<()> {
    println!("\n========== End-to-End Workflow ==========\n");

    let summary_path = "/tmp/paiml-summary.yaml";

    if !std::path::Path::new(summary_path).exists() {
        println!("⚠️  Summary file not found. Complete workflow requires:");
        println!("\n1. Analyze organization (OIP Phase 1):");
        println!("   cd ../organizational-intelligence-plugin");
        println!("   cargo run -- analyze --org paiml --output /tmp/paiml-analysis.yaml\n");

        println!("2. Summarize analysis (OIP Phase 2):");
        println!("   cargo run -- summarize \\");
        println!("       --input /tmp/paiml-analysis.yaml \\");
        println!("       --output /tmp/paiml-summary.yaml \\");
        println!("       --strip-pii\n");

        println!("3. Generate prompts (PMAT Phase 4):");
        println!("   cargo run --example organizational_intelligence_integration\n");

        return Ok(());
    }

    let generator = DefectAwarePromptGenerator::from_file(summary_path)?;

    println!("✅ Loaded organizational intelligence:");
    println!(
        "   Repositories: {}",
        generator.metadata.repositories_analyzed
    );
    println!("   Commits: {}", generator.metadata.commits_analyzed);
    println!("   Defect patterns: {}\n", generator.defect_patterns.len());

    // Example 1: HTTP Client Development
    println!("Example 1: Develop HTTP Client");
    let prompt1 = generator.generate_prompt(
        "Implement HTTP client with retry logic",
        "External API integration for payment processing",
    );
    println!("  Prompt length: {} chars", prompt1.len());
    println!(
        "  Contains IntegrationFailures warning: {}",
        prompt1.contains("IntegrationFailures")
    );
    println!(
        "  Contains quality gates: {}\n",
        prompt1.contains("Quality Gates")
    );

    // Example 2: Configuration Parser
    println!("Example 2: Build Configuration Parser");
    let prompt2 = generator.generate_prompt(
        "Create YAML configuration parser",
        "Microservices configuration management",
    );
    println!("  Prompt length: {} chars", prompt2.len());
    println!(
        "  Contains ConfigurationErrors warning: {}",
        prompt2.contains("ConfigurationErrors")
    );
    println!(
        "  Contains TDG threshold: {}\n",
        prompt2.contains("TDG Score: 85")
    );

    // Example 3: Prevention Prompts
    println!("Example 3: Category-Specific Prevention");
    if let Some(prevention) = generator.generate_prevention_prompt("IntegrationFailures") {
        println!("  Prevention prompt for IntegrationFailures:");
        println!("  Length: {} chars", prevention.len());
        println!(
            "  Contains frequency: {}",
            prevention.contains("Historical Frequency")
        );
        println!("  Contains TDG score: {}\n", prevention.contains("TDG"));
    }

    println!("✅ All examples complete!\n");

    Ok(())
}

fn main() -> Result<()> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Organizational Intelligence Integration Examples");
    println!("  Phase 4: DefectAwarePromptGenerator");
    println!("═══════════════════════════════════════════════════════");

    // Run all demonstrations
    demonstrate_cli_usage()?;
    demonstrate_mcp_usage()?;
    demonstrate_org_analyze_usage()?;
    demonstrate_end_to_end_workflow()?;

    println!("═══════════════════════════════════════════════════════");
    println!("  Summary: All 3 Options Demonstrated");
    println!("═══════════════════════════════════════════════════════");
    println!("\n✅ Option A (MCP): Ready for integration in mcp_pmcp/tool_functions.rs");
    println!("✅ Option B (CLI): Can add `pmat prompt generate` command");
    println!("✅ Option C (Org): Can add `pmat org analyze` command\n");

    println!("Next Steps:");
    println!("  1. Add MCP tool: generate_defect_aware_prompt");
    println!("  2. Add CLI command: pmat prompt generate");
    println!("  3. Add CLI command: pmat org analyze");
    println!("  4. Update pmat-book documentation\n");

    Ok(())
}
