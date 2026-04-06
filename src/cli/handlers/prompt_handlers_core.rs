/// Handle the prompt command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

/// New dispatcher for prompt subcommands (Phase 4)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        } => handle_comply_prompt(&min_grade, baseline.as_ref(), roadmap.as_ref(), &output).await,
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
