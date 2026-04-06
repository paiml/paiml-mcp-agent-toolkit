#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handlers for Claude Code sub-agent generation.

use crate::cli::colors as c;
use crate::scaffold::agent::{PmatSubAgent, SubAgentGenerator};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// List all available sub-agents.
pub fn list_subagents(show_all: bool) -> Result<()> {
    let agents = if show_all {
        PmatSubAgent::all()
    } else {
        PmatSubAgent::all_mvp()
    };

    println!("{}", c::header("Available PMAT Sub-Agents"));
    println!();

    let mut mvp_count = 0;
    let mut future_count = 0;

    for agent in agents {
        let status = if agent.is_mvp() {
            mvp_count += 1;
            format!("{}✓ MVP{}", c::GREEN, c::RESET)
        } else {
            future_count += 1;
            format!("{}○ Future{}", c::DIM, c::RESET)
        };

        println!(
            "  {} {} - {}",
            status,
            c::label(agent.name()),
            agent.description()
        );

        // Show primary tools
        let tools = agent.primary_tools();
        println!("    {}: {}", c::dim("Tools"), tools.join(", "));
        println!();
    }

    println!(
        "Total: {} MVP, {} Future",
        c::number(&mvp_count.to_string()),
        c::number(&future_count.to_string())
    );

    if !show_all {
        println!();
        println!(
            "{}",
            c::dim("Tip: Use --all to see all sub-agents (including future phases)")
        );
    }

    Ok(())
}

/// Create a specific sub-agent.
pub fn create_subagent(agent_name: &str, output_dir: Option<PathBuf>) -> Result<()> {
    // Parse agent name
    let agent: PmatSubAgent = agent_name.parse()?;

    // Check if MVP
    if !agent.is_mvp() {
        bail!(
            "Sub-agent '{}' is not yet implemented (future phase). MVP agents: {}",
            agent.name(),
            PmatSubAgent::all_mvp()
                .iter()
                .map(|a| a.name())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Determine output directory
    let output = output_dir.unwrap_or_else(|| PathBuf::from(".claude/subagents"));

    println!("{} {}", c::label("Creating sub-agent:"), agent.name());
    println!("  {}: {}", c::dim("Description"), agent.description());
    println!(
        "  {}: {}",
        c::dim("Output"),
        c::path(&output.display().to_string())
    );
    println!();

    // Generate sub-agent
    let generator = SubAgentGenerator::new();
    let path = generator.export_for_claude_code(agent, &output)?;

    println!("{}", c::pass("Sub-agent created successfully!"));
    println!(
        "  {}: {}",
        c::dim("File"),
        c::path(&path.display().to_string())
    );
    println!();
    println!("{}", c::subheader("Next steps:"));
    println!("  1. Review the generated sub-agent definition");
    println!("  2. Customize if needed (prompts, tools, examples)");
    println!("  3. Use with Claude Code: Place in .claude/subagents/");
    println!("  4. Invoke: @{} <command>", agent.name());

    Ok(())
}

/// Create all MVP sub-agents.
pub fn create_all_mvp_subagents(output_dir: Option<PathBuf>) -> Result<()> {
    let output = output_dir.unwrap_or_else(|| PathBuf::from(".claude/subagents"));

    println!("{}", c::label("Creating all MVP sub-agents..."));
    println!(
        "  {}: {}",
        c::dim("Output"),
        c::path(&output.display().to_string())
    );
    println!();

    let generator = SubAgentGenerator::new();
    let paths = generator.export_all_mvp(&output)?;

    println!(
        "{} Created {} sub-agents successfully!",
        c::pass(""),
        c::number(&paths.len().to_string())
    );
    println!();

    for (agent, path) in PmatSubAgent::all_mvp().iter().zip(paths.iter()) {
        println!(
            "  {} {} - {}",
            c::pass(""),
            c::label(agent.name()),
            c::path(&path.file_name().expect("internal error").to_string_lossy())
        );
    }

    println!();
    println!("{}", c::subheader("Next steps:"));
    println!(
        "  1. Review generated sub-agents in {}",
        c::path(&output.display().to_string())
    );
    println!("  2. Place them in your project's .claude/subagents/ directory");
    println!("  3. Use with Claude Code: @<agent-name> <command>");
    println!();
    println!("{}", c::subheader("Example usage:"));
    println!("  {}", c::dim("@complexity-analyst analyze src/"));
    println!("  {}", c::dim("@mutation-tester test src/lib.rs"));
    println!("  {}", c::dim("@satd-detector scan for old debt"));

    Ok(())
}

/// Check for required markdown sections in sub-agent definition.
fn check_required_sections(content: &str, issues: &mut Vec<String>) {
    let required_sections = [
        "# ", // Title
        "## Description",
        "## Capabilities",
        "## Tools Used",
        "## Role Definition",
        "## Communication Protocol",
        "## Implementation Workflow",
        "## Example Invocations",
        "## Quality Gates",
    ];

    for section in required_sections {
        if !content.contains(section) {
            issues.push(format!("Missing required section: {}", section));
        }
    }
}

/// Check for common content patterns in sub-agent definition.
fn check_content_patterns(content: &str, warnings: &mut Vec<String>) {
    if !content.contains("MCP") {
        warnings.push("No MCP tools mentioned (expected for PMAT sub-agents)".to_string());
    }

    if !content.contains("@") {
        warnings.push("No example invocations with @ syntax".to_string());
    }

    if content.contains("TODO") || content.contains("TBD") {
        warnings.push("Contains placeholder text (TODO/TBD)".to_string());
    }
}

/// Check markdown formatting rules.
fn check_markdown_format(content: &str, issues: &mut Vec<String>) {
    if !content.starts_with("# ") {
        issues.push("File should start with # title".to_string());
    }
}

/// Report validation results to the user.
fn report_validation_results(issues: &[String], warnings: &[String]) -> Result<()> {
    if issues.is_empty() && warnings.is_empty() {
        println!("{}", c::pass("Validation passed!"));
        println!("  All required sections present");
        println!("  Markdown format valid");
        println!("  No issues found");
        return Ok(());
    }

    if !issues.is_empty() {
        println!("{}", c::fail("Validation failed!"));
        println!();
        println!("{}", c::subheader("Issues:"));
        for issue in issues {
            println!("  {}", c::fail(issue));
        }
    }

    if !warnings.is_empty() {
        println!();
        println!("{}", c::subheader("Warnings:"));
        for warning in warnings {
            println!("  {}", c::warn(warning));
        }
    }

    if issues.is_empty() {
        println!();
        println!("{}", c::pass("Validation passed with warnings"));
        Ok(())
    } else {
        bail!("Validation failed with {} issues", issues.len());
    }
}

/// Validate a sub-agent definition file.
pub fn validate_subagent(file_path: &Path) -> Result<()> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    println!(
        "{} {}",
        c::label("Validating sub-agent:"),
        c::path(&file_path.display().to_string())
    );
    println!();

    if !file_path.exists() {
        bail!("File not found: {}", file_path.display());
    }

    let content = std::fs::read_to_string(file_path)?;

    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    check_required_sections(&content, &mut issues);
    check_content_patterns(&content, &mut warnings);
    check_markdown_format(&content, &mut issues);

    report_validation_results(&issues, &warnings)
}

/// Show MCP tool mapping for sub-agents.
pub fn show_tool_mapping(agent_name: Option<String>) -> Result<()> {
    let mapping = SubAgentGenerator::get_tool_mapping();

    if let Some(name) = agent_name {
        // Show mapping for specific agent
        let agent: PmatSubAgent = name.parse()?;
        let tools = mapping
            .get(&agent)
            .ok_or_else(|| anyhow::anyhow!("No tool mapping found for agent: {}", agent.name()))?;

        println!("{} {}", c::label("MCP Tool Mapping:"), agent.name());
        println!("  {}: {}", c::dim("Description"), agent.description());
        println!();
        println!("{}", c::subheader("Primary Tools:"));
        for tool in tools {
            println!("  {}•{} pmat__{}", c::CYAN, c::RESET, tool);
        }
    } else {
        // Show mapping for all agents
        println!("{}", c::header("MCP Tool Mapping (All Sub-Agents)"));
        println!();

        for agent in PmatSubAgent::all_mvp() {
            let tools = mapping.get(&agent).expect("internal error");
            let status = if agent.is_mvp() {
                format!("{}✓{}", c::GREEN, c::RESET)
            } else {
                format!("{}○{}", c::DIM, c::RESET)
            };

            println!("{} {}", status, c::label(agent.name()));
            for tool in tools {
                println!("    {}•{} pmat__{}", c::CYAN, c::RESET, tool);
            }
            println!();
        }
    }

    Ok(())
}

/// Export MCP tool mapping as JSON.
pub fn export_tool_mapping_json(output_path: &Path) -> Result<()> {
    use std::collections::HashMap;

    let mapping = SubAgentGenerator::get_tool_mapping();

    // Convert to JSON-serializable format
    let json_mapping: HashMap<String, Vec<String>> = mapping
        .iter()
        .map(|(agent, tools)| {
            (
                agent.name().to_string(),
                tools.iter().map(|t| format!("pmat__{}", t)).collect(),
            )
        })
        .collect();

    let json = serde_json::to_string_pretty(&json_mapping)?;
    std::fs::write(output_path, json)?;

    println!(
        "{} Exported tool mapping to: {}",
        c::pass(""),
        c::path(&output_path.display().to_string())
    );

    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_subagents_mvp() {
        // Should not panic
        let result = list_subagents(false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_subagents_all() {
        let result = list_subagents(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_subagent_valid() {
        let temp_dir = TempDir::new().expect("internal error");
        let output = temp_dir.path().to_path_buf();

        let result = create_subagent("complexity-analyst", Some(output.clone()));
        assert!(result.is_ok());

        // Verify file was created
        let expected_file = output.join("complexity-analyst.md");
        assert!(expected_file.exists());
    }

    #[test]
    fn test_create_subagent_invalid() {
        let temp_dir = TempDir::new().expect("internal error");
        let output = temp_dir.path().to_path_buf();

        let result = create_subagent("invalid-agent", Some(output));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_subagent_future_phase() {
        let temp_dir = TempDir::new().expect("internal error");
        let output = temp_dir.path().to_path_buf();

        let result = create_subagent("rust-quality-expert", Some(output));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not yet implemented"));
    }

    #[test]
    fn test_create_all_mvp() {
        let temp_dir = TempDir::new().expect("internal error");
        let output = temp_dir.path().to_path_buf();

        let result = create_all_mvp_subagents(Some(output.clone()));
        assert!(result.is_ok());

        // Verify all MVP agents were created
        for agent in PmatSubAgent::all_mvp() {
            let expected_file = output.join(format!("{}.md", agent.name()));
            assert!(
                expected_file.exists(),
                "Expected file not found: {}",
                expected_file.display()
            );
        }
    }

    #[test]
    fn test_validate_valid_subagent() {
        let temp_dir = TempDir::new().expect("internal error");
        let output = temp_dir.path().to_path_buf();

        // Create a sub-agent first
        create_subagent("complexity-analyst", Some(output.clone())).expect("internal error");

        let file = output.join("complexity-analyst.md");
        let result = validate_subagent(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_missing_file() {
        let result = validate_subagent(Path::new("/nonexistent/file.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_show_tool_mapping_all() {
        let result = show_tool_mapping(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_tool_mapping_specific() {
        let result = show_tool_mapping(Some("complexity-analyst".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_tool_mapping() {
        let temp_dir = TempDir::new().expect("internal error");
        let output = temp_dir.path().join("tool_mapping.json");

        let result = export_tool_mapping_json(&output);
        assert!(result.is_ok());
        assert!(output.exists());

        // Verify JSON is valid
        let content = std::fs::read_to_string(&output).expect("internal error");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("internal error");
        assert!(parsed.is_object());
    }
}
