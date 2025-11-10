//! Prompt command handlers for workflow prompts
//!
//! This module provides handlers for the `pmat prompt` command, which displays
//! pre-configured workflow prompts that enforce EXTREME TDD and Toyota Way quality principles.

use crate::cli::PromptOutputFormat;
use crate::models::prompt_model::WorkflowPrompt;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// List of all available prompts (embedded at compile time)
const PROMPTS: &[(&str, &str)] = &[
    (
        "code-coverage",
        include_str!("../../../prompts/code-coverage.yaml"),
    ),
    (
        "clean-repo-cruft",
        include_str!("../../../prompts/clean-repo-cruft.yaml"),
    ),
    (
        "continue",
        include_str!("../../../prompts/continue.yaml"),
    ),
    (
        "assert-cmd-testing",
        include_str!("../../../prompts/assert-cmd-testing.yaml"),
    ),
    (
        "documentation",
        include_str!("../../../prompts/documentation.yaml"),
    ),
    ("debug", include_str!("../../../prompts/debug.yaml")),
    (
        "mutation-testing",
        include_str!("../../../prompts/mutation-testing.yaml"),
    ),
    (
        "performance-optimization",
        include_str!("../../../prompts/performance-optimization.yaml"),
    ),
    (
        "quality-enforcement",
        include_str!("../../../prompts/quality-enforcement.yaml"),
    ),
    (
        "refactor-hotspots",
        include_str!("../../../prompts/refactor-hotspots.yaml"),
    ),
    (
        "security-audit",
        include_str!("../../../prompts/security-audit.yaml"),
    ),
];

/// Handle the prompt command
pub async fn handle_prompt(
    name: Option<String>,
    list: bool,
    show_variables: bool,
    set: Vec<(String, Value)>,
    format: PromptOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    // List all prompts
    if list {
        list_prompts();
        return Ok(());
    }

    // Show specific prompt
    if let Some(prompt_name) = name {
        show_prompt(&prompt_name, show_variables, set, format, output)?;
    } else {
        anyhow::bail!("Please specify a prompt name or use --list to see all available prompts");
    }

    Ok(())
}

/// List all available prompts
fn list_prompts() {
    println!("Available Prompts:");
    println!();

    for (name, yaml) in PROMPTS {
        // Parse to get description
        if let Ok(prompt) = WorkflowPrompt::from_yaml(yaml) {
            println!(
                "  {} - {} [{}]",
                name, prompt.description, prompt.priority
            );
        } else {
            println!("  {} - (parse error)", name);
        }
    }

    println!();
    println!("Usage:");
    println!("  pmat prompt <name>                       Show prompt in YAML format");
    println!("  pmat prompt <name> --format json         Show prompt in JSON format");
    println!("  pmat prompt <name> --format text         Show just the prompt text");
    println!("  pmat prompt <name> --show-variables      Show available variables");
    println!("  pmat prompt <name> --set VAR=value       Override prompt variables");
    println!();
}

/// Show a specific prompt
fn show_prompt(
    name: &str,
    show_variables: bool,
    set: Vec<(String, Value)>,
    format: PromptOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    // Find the prompt
    let yaml = PROMPTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, y)| *y)
        .with_context(|| format!("Prompt not found: {name}"))?;

    // Parse the prompt
    let prompt = WorkflowPrompt::from_yaml(yaml)
        .with_context(|| format!("Failed to parse prompt: {name}"))?;

    // Show variables if requested
    if show_variables {
        let variables = prompt.extract_variables();
        if variables.is_empty() {
            println!("No variables found in this prompt");
        } else {
            println!("Variables:");
            for var in variables {
                println!("  ${{{var}}}");
            }
        }
        return Ok(());
    }

    // Build variable map from --set flags
    let mut variables = HashMap::new();
    for (key, value) in set {
        let value_str = match value {
            Value::String(s) => s,
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            _ => value.to_string(),
        };
        variables.insert(key, value_str);
    }

    // Render output in requested format
    let output_str = match format {
        PromptOutputFormat::Yaml => prompt.to_yaml()?,
        PromptOutputFormat::Json => prompt.to_json()?,
        PromptOutputFormat::Text => prompt.to_text(&variables),
    };

    // Write to file or stdout
    if let Some(output_path) = output {
        std::fs::write(&output_path, &output_str)
            .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
        println!("Prompt written to {}", output_path.display());
    } else {
        println!("{output_str}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_prompts_parse() {
        for (name, yaml) in PROMPTS {
            let result = WorkflowPrompt::from_yaml(yaml);
            assert!(
                result.is_ok(),
                "Failed to parse prompt {}: {:?}",
                name,
                result.err()
            );
        }
    }

    #[test]
    fn test_all_prompts_have_required_fields() {
        for (name, yaml) in PROMPTS {
            let prompt = WorkflowPrompt::from_yaml(yaml).unwrap();
            assert!(!prompt.name.is_empty(), "Prompt {name} missing name");
            assert!(
                !prompt.description.is_empty(),
                "Prompt {name} missing description"
            );
            assert!(
                !prompt.category.is_empty(),
                "Prompt {name} missing category"
            );
            assert!(
                !prompt.priority.is_empty(),
                "Prompt {name} missing priority"
            );
            assert!(!prompt.prompt.is_empty(), "Prompt {name} missing prompt text");
        }
    }

    #[test]
    fn test_prompt_names_match_keys() {
        for (key, yaml) in PROMPTS {
            let prompt = WorkflowPrompt::from_yaml(yaml).unwrap();
            assert_eq!(
                &prompt.name, key,
                "Prompt name mismatch: key={key}, name={}",
                prompt.name
            );
        }
    }

    #[tokio::test]
    async fn test_handle_prompt_list() {
        let result = handle_prompt(
            None,
            true,
            false,
            vec![],
            PromptOutputFormat::Yaml,
            None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prompt_show_code_coverage() {
        let result = handle_prompt(
            Some("code-coverage".to_string()),
            false,
            false,
            vec![],
            PromptOutputFormat::Yaml,
            None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prompt_show_variables() {
        let result = handle_prompt(
            Some("code-coverage".to_string()),
            false,
            true,
            vec![],
            PromptOutputFormat::Yaml,
            None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prompt_not_found() {
        let result = handle_prompt(
            Some("nonexistent".to_string()),
            false,
            false,
            vec![],
            PromptOutputFormat::Yaml,
            None,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_prompt_json_format() {
        let result = handle_prompt(
            Some("continue".to_string()),
            false,
            false,
            vec![],
            PromptOutputFormat::Json,
            None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prompt_text_format() {
        let result = handle_prompt(
            Some("debug".to_string()),
            false,
            false,
            vec![],
            PromptOutputFormat::Text,
            None,
        )
        .await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_handle_prompt_with_valid_names(name in prop::sample::select(vec![
            "code-coverage",
            "clean-repo-cruft",
            "continue",
            "assert-cmd-testing",
            "documentation",
            "debug",
            "mutation-testing",
            "performance-optimization",
            "quality-enforcement",
            "refactor-hotspots",
            "security-audit",
        ])) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(handle_prompt(
                Some(name.to_string()),
                false,
                false,
                vec![],
                PromptOutputFormat::Yaml,
                None,
            ));
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_invalid_prompt_name_fails(invalid_name in "[a-z]{1,20}") {
            // Only test names that don't match our valid prompts
            let valid_names = ["code-coverage", "clean-repo-cruft", "continue",
                "assert-cmd-testing", "documentation", "debug",
                "mutation-testing", "performance-optimization",
                "quality-enforcement", "refactor-hotspots", "security-audit"];

            if !valid_names.contains(&invalid_name.as_str()) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(handle_prompt(
                    Some(invalid_name),
                    false,
                    false,
                    vec![],
                    PromptOutputFormat::Yaml,
                    None,
                ));
                prop_assert!(result.is_err());
            }
        }
    }
}
