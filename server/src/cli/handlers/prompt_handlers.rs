//! Prompt command handlers for workflow prompts
//!
//! This module provides handlers for the `pmat prompt` command, which displays
//! pre-configured workflow prompts that enforce EXTREME TDD and Toyota Way quality principles.

use crate::cli::commands::PromptCommands;
use crate::cli::PromptOutputFormat;
use crate::models::prompt_model::WorkflowPrompt;
use crate::prompts::DefectAwarePromptGenerator;
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
    ("continue", include_str!("../../../prompts/continue.yaml")),
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
    (
        "comply-pmat",
        include_str!("../../../prompts/comply-pmat.yaml"),
    ),
    (
        "book-documentation",
        include_str!("../../../prompts/book-documentation.yaml"),
    ),
    (
        "repo-image",
        include_str!("../../../prompts/repo-image.yaml"),
    ),
    (
        "github-ticket",
        include_str!("../../../prompts/github-ticket.yaml"),
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
            println!("  {} - {} [{}]", name, prompt.description, prompt.priority);
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
            assert!(
                !prompt.prompt.is_empty(),
                "Prompt {name} missing prompt text"
            );
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
        let result = handle_prompt(None, true, false, vec![], PromptOutputFormat::Yaml, None).await;
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

/// New dispatcher for prompt subcommands (Phase 4)
pub async fn handle_prompt_command(prompt_cmd: PromptCommands) -> Result<()> {
    match prompt_cmd {
        PromptCommands::Show {
            name,
            list,
            show_variables,
            set,
            format,
            output,
        } => handle_prompt(name, list, show_variables, set, format, output).await,
        PromptCommands::Generate {
            task,
            context,
            summary,
            output,
        } => handle_generate_prompt(&task, &context, &summary, &output).await,
        PromptCommands::Ticket {
            ticket,
            summary,
            output,
        } => handle_ticket_prompt(&ticket, summary.as_ref(), &output).await,
        PromptCommands::Implement {
            spec,
            summary,
            output,
        } => handle_implement_prompt(&spec, summary.as_ref(), &output).await,
        PromptCommands::ScaffoldNewRepo {
            spec,
            include_pmat,
            include_bashrs,
            include_roadmap,
            output,
        } => {
            handle_scaffold_repo_prompt(
                &spec,
                include_pmat,
                include_bashrs,
                include_roadmap,
                &output,
            )
            .await
        }
        PromptCommands::Comply {
            min_grade,
            baseline,
            roadmap,
            output,
        } => {
            handle_comply_prompt(
                &min_grade,
                baseline.as_ref(),
                roadmap.as_ref(),
                &output,
            )
            .await
        }
        PromptCommands::Book {
            title,
            book_type,
            target_pages,
            min_pass_rate,
            output,
        } => {
            handle_book_prompt(
                title.as_deref(),
                &book_type,
                target_pages,
                min_pass_rate,
                &output,
            )
            .await
        }
        PromptCommands::RepoImage {
            repo_name,
            description,
            github_org,
            language,
            course_series,
            output,
        } => {
            handle_repo_image_prompt(
                repo_name.as_deref(),
                description.as_deref(),
                &github_org,
                language.as_deref(),
                course_series,
                &output,
            )
            .await
        }
        PromptCommands::GithubIssue {
            issue,
            org,
            repo,
            test_cmd,
            build_cmd,
            output,
        } => {
            handle_github_issue_prompt(
                &issue,
                org.as_deref(),
                repo.as_deref(),
                &test_cmd,
                &build_cmd,
                &output,
            )
            .await
        }
    }
}

/// Handle defect-aware prompt generation
async fn handle_generate_prompt(
    task: &str,
    context: &str,
    summary_path: &PathBuf,
    output: &Option<PathBuf>,
) -> Result<()> {
    let generator = DefectAwarePromptGenerator::from_file(summary_path)
        .context("Failed to load organizational intelligence summary")?;

    let prompt = generator.generate_prompt(task, context);

    if let Some(output_path) = output {
        std::fs::write(output_path, &prompt)
            .context(format!("Failed to write prompt to {:?}", output_path))?;
        println!("✅ Defect-aware prompt written to {:?}", output_path);
    } else {
        println!("{}", prompt);
    }

    Ok(())
}

/// Handle EXTREME TDD ticket prompt generation
async fn handle_ticket_prompt(
    ticket: &str,
    summary_path: Option<&PathBuf>,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut prompt = format!(
        "# EXTREME TDD: Fix Ticket\n\n\
         ## Ticket\n{}\n\n\
         ## Workflow\n\
         1. **RED**: Write failing test that reproduces the issue\n\
         2. **GREEN**: Implement minimal fix to make test pass\n\
         3. **REFACTOR**: Clean up code while keeping tests green\n\
         4. **VERIFY**: Run full test suite and quality gates\n\
         5. **COMMIT**: Only commit if all gates pass\n\n\
         ## Quality Gates (Before Commit)\n\
         ```bash\n\
         pmat analyze tdg --threshold 85\n\
         cargo test --all-features\n\
         cargo llvm-cov report --summary-only\n\
         ```\n\n",
        ticket
    );

    // Add organizational intelligence if available
    if let Some(summary) = summary_path {
        if summary.exists() {
            let generator = DefectAwarePromptGenerator::from_file(summary)?;
            prompt.push_str(&format!(
                "## Organizational Intelligence\n\
                 Based on analysis of {} repositories with {} commits:\n\n\
                 ### Common Defect Patterns to Avoid\n",
                generator.metadata.repositories_analyzed, generator.metadata.commits_analyzed
            ));

            for pattern in generator.defect_patterns.iter().take(5) {
                let tdg = pattern
                    .quality_signals
                    .avg_tdg_score
                    .map(|s| format!("{:.1}", s))
                    .unwrap_or_else(|| "N/A".to_string());
                prompt.push_str(&format!(
                    "- {} ({} occurrences, TDG: {})\n",
                    pattern.category, pattern.frequency, tdg
                ));
            }
            prompt.push('\n');
        }
    }

    if let Some(output_path) = output {
        std::fs::write(output_path, &prompt)?;
        println!("✅ Ticket prompt written to {:?}", output_path);
    } else {
        println!("{}", prompt);
    }

    Ok(())
}

/// Handle specification-based implementation prompt
async fn handle_implement_prompt(
    spec_path: &PathBuf,
    summary_path: Option<&PathBuf>,
    output: &Option<PathBuf>,
) -> Result<()> {
    let spec_content = std::fs::read_to_string(spec_path).context(format!(
        "Failed to read specification file: {:?}",
        spec_path
    ))?;

    let mut prompt = format!(
        "# Implementation from Specification\n\n\
         ## Specification\n{}\n\n\
         ## Implementation Strategy\n\
         1. **Analyze**: Break down spec into testable components\n\
         2. **RED**: Write tests for each component (EXTREME TDD)\n\
         3. **GREEN**: Implement to pass tests\n\
         4. **REFACTOR**: Optimize while maintaining test coverage\n\
         5. **VERIFY**: All quality gates must pass\n\n\
         ## Quality Requirements\n\
         - TDG Score: 85+\n\
         - Test Coverage: 85%+\n\
         - Max Complexity: 10\n\
         - Zero SATD (Self-Admitted Technical Debt)\n\n",
        spec_content
    );

    // Add organizational intelligence if available
    if let Some(summary) = summary_path {
        if summary.exists() {
            let generator = DefectAwarePromptGenerator::from_file(summary)?;
            prompt.push_str(&format!(
                "## Organizational Quality Standards\n\
                 Based on {} repositories, {} commits:\n\n",
                generator.metadata.repositories_analyzed, generator.metadata.commits_analyzed
            ));

            for pattern in generator.defect_patterns.iter().take(3) {
                if let Some(tdg) = pattern.quality_signals.avg_tdg_score {
                    prompt.push_str(&format!(
                        "⚠️  Avoid {}: {} historical occurrences (TDG: {:.1})\n",
                        pattern.category, pattern.frequency, tdg
                    ));
                }
            }
            prompt.push('\n');
        }
    }

    if let Some(output_path) = output {
        std::fs::write(output_path, &prompt)?;
        println!("✅ Implementation prompt written to {:?}", output_path);
    } else {
        println!("{}", prompt);
    }

    Ok(())
}

/// Handle new repository scaffolding prompt
async fn handle_scaffold_repo_prompt(
    spec_path: &PathBuf,
    include_pmat: bool,
    include_bashrs: bool,
    include_roadmap: bool,
    output: &Option<PathBuf>,
) -> Result<()> {
    let spec_content = std::fs::read_to_string(spec_path).context(format!(
        "Failed to read specification file: {:?}",
        spec_path
    ))?;

    let mut prompt = format!(
        "# Scaffold New Repository\n\n\
         ## Repository Specification\n{}\n\n\
         ## Setup Checklist\n\n",
        spec_content
    );

    if include_pmat {
        prompt.push_str(
            "### PMAT Tools Integration\n\
             - [ ] Add `pmat` as dev dependency\n\
             - [ ] Configure `.git/hooks/pre-commit` with `pmat hooks install --tdg-enforcement`\n\
             - [ ] Add quality gates to CI/CD: `pmat quality-gate --fail-on-violation`\n\
             - [ ] Configure TDG thresholds in `.pmat/config.toml`\n\
             - [ ] Set up `pmat context` for AI-assisted development\n\n",
        );
    }

    if include_bashrs {
        prompt.push_str(
            "### bashrs Integration\n\
             - [ ] Install bashrs: `cargo install bashrs`\n\
             - [ ] Add bashrs linting to pre-commit hooks\n\
             - [ ] Configure bashrs in `.bashrsrc` (if needed)\n\
             - [ ] Lint all bash scripts: `find . -name '*.sh' -exec bashrs lint {} \\;`\n\n",
        );
    }

    if include_roadmap {
        prompt.push_str(
            "### Roadmapping Tools\n\
             - [ ] Initialize roadmap: `pmat roadmap init`\n\
             - [ ] Define milestones in `docs/roadmap/`\n\
             - [ ] Set up milestone tracking in project board\n\
             - [ ] Configure roadmap visualization\n\n",
        );
    }

    prompt.push_str(
        "## Repository Structure\n\
         ```\n\
         repo-name/\n\
         ├── .git/hooks/          # Pre-commit, pre-push hooks\n\
         ├── .pmat/               # PMAT configuration\n\
         ├── docs/\n\
         │   ├── specifications/  # Markdown specs\n\
         │   └── roadmap/         # Milestone tracking\n\
         ├── src/                 # Source code\n\
         ├── tests/               # Test suites (>85% coverage)\n\
         ├── scripts/             # Bash scripts (bashrs-validated)\n\
         ├── Cargo.toml           # Rust manifest (or equivalent)\n\
         └── README.md            # Project documentation\n\
         ```\n\n\
         ## EXTREME TDD Workflow\n\
         1. Write specification in `docs/specifications/`\n\
         2. Generate prompt: `pmat prompt implement --spec <spec.md>`\n\
         3. RED → GREEN → REFACTOR cycle\n\
         4. Quality gates before commit (enforced by hooks)\n\
         5. Continuous roadmap updates\n\n",
    );

    if let Some(output_path) = output {
        std::fs::write(output_path, &prompt)?;
        println!("✅ Scaffold prompt written to {:?}", output_path);
    } else {
        println!("{}", prompt);
    }

    Ok(())
}

/// Handle PMAT compliance enforcement prompt
async fn handle_comply_prompt(
    min_grade: &str,
    baseline: Option<&PathBuf>,
    roadmap: Option<&PathBuf>,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut vars = HashMap::new();
    vars.insert("MIN_GRADE".to_string(), Value::String(min_grade.to_string()));

    if let Some(baseline_path) = baseline {
        vars.insert(
            "BASELINE_PATH".to_string(),
            Value::String(baseline_path.display().to_string()),
        );
    }

    if let Some(roadmap_path) = roadmap {
        vars.insert(
            "ROADMAP_PATH".to_string(),
            Value::String(roadmap_path.display().to_string()),
        );
    }

    show_prompt("comply-pmat", false, vars.into_iter().collect(), PromptOutputFormat::Yaml, output.clone())?;
    Ok(())
}

/// Handle book documentation prompt
async fn handle_book_prompt(
    title: Option<&str>,
    book_type: &str,
    target_pages: u32,
    min_pass_rate: u8,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut vars = HashMap::new();

    if let Some(t) = title {
        vars.insert("BOOK_TITLE".to_string(), Value::String(t.to_string()));
    }

    vars.insert("BOOK_TYPE".to_string(), Value::String(book_type.to_string()));
    vars.insert("TARGET_PAGES".to_string(), Value::String(target_pages.to_string()));
    vars.insert("MIN_PASS_RATE".to_string(), Value::String(min_pass_rate.to_string()));

    show_prompt("book-documentation", false, vars.into_iter().collect(), PromptOutputFormat::Yaml, output.clone())?;
    Ok(())
}

/// Handle repository image/documentation prompt
async fn handle_repo_image_prompt(
    repo_name: Option<&str>,
    description: Option<&str>,
    github_org: &str,
    language: Option<&str>,
    course_series: bool,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut vars = HashMap::new();

    if let Some(name) = repo_name {
        vars.insert("REPO_NAME".to_string(), Value::String(name.to_string()));
    }

    if let Some(desc) = description {
        vars.insert("REPO_DESCRIPTION".to_string(), Value::String(desc.to_string()));
    }

    vars.insert("GITHUB_ORG".to_string(), Value::String(github_org.to_string()));

    if let Some(lang) = language {
        vars.insert("PRIMARY_LANGUAGE".to_string(), Value::String(lang.to_string()));
    }

    vars.insert("COURSE_SERIES".to_string(), Value::String(course_series.to_string()));

    show_prompt("repo-image", false, vars.into_iter().collect(), PromptOutputFormat::Yaml, output.clone())?;
    Ok(())
}

/// Handle GitHub issue-driven development prompt
async fn handle_github_issue_prompt(
    issue: &str,
    org: Option<&str>,
    repo: Option<&str>,
    test_cmd: &str,
    build_cmd: &str,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut vars = HashMap::new();

    // Determine if issue is a URL or number
    if issue.starts_with("http") {
        vars.insert("ISSUE_URL".to_string(), Value::String(issue.to_string()));
    } else {
        vars.insert("ISSUE_NUMBER".to_string(), Value::String(issue.to_string()));

        if let Some(organization) = org {
            vars.insert("GITHUB_ORG".to_string(), Value::String(organization.to_string()));
        }

        if let Some(repository) = repo {
            vars.insert("GITHUB_REPO".to_string(), Value::String(repository.to_string()));
        }
    }

    vars.insert("TEST_CMD".to_string(), Value::String(test_cmd.to_string()));
    vars.insert("BUILD_CMD".to_string(), Value::String(build_cmd.to_string()));

    show_prompt("github-ticket", false, vars.into_iter().collect(), PromptOutputFormat::Yaml, output.clone())?;
    Ok(())
}
