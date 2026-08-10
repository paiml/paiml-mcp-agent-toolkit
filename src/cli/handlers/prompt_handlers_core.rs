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
    for (invocation, description) in LIST_PROMPTS_USAGE {
        println!("  {invocation:<41} {description}");
    }
    println!();
}

/// The invocations advertised by the `--list` footer.
///
/// These used to read `pmat prompt <name> …`. `prompt` takes a subcommand, so
/// every one of them was a command clap rejects with "unrecognized subcommand"
/// and exit 2 — the footer of a working `pmat prompt show --list` told the
/// reader to run five things that cannot run. The working form is
/// `pmat prompt show <name>`.
const LIST_PROMPTS_USAGE: &[(&str, &str)] = &[
    ("pmat prompt show <name>", "Show prompt in YAML format"),
    (
        "pmat prompt show <name> --format json",
        "Show prompt in JSON format",
    ),
    (
        "pmat prompt show <name> --format text",
        "Show just the prompt text",
    ),
    (
        "pmat prompt show <name> --show-variables",
        "Show available variables",
    ),
    (
        "pmat prompt show <name> --set VAR=value",
        "Override prompt variables",
    ),
];

/// Substitute `${VAR}` placeholders and put object keys in a stable order.
///
/// Substituting on the serialised *value* rather than on the emitted text keeps
/// YAML/JSON escaping correct, and covers the placeholders that live outside
/// the main `prompt` field (quality gates, validation tools, …).
///
/// The same walk rebuilds every object with sorted keys: the prompt model holds
/// `HashMap` fields, so four identical `pmat prompt book --title MYBOOK`
/// invocations produced four different md5s.
fn render_prompt_value(value: &Value, variables: &HashMap<String, String>) -> Value {
    match value {
        Value::String(s) => {
            let mut rendered = s.clone();
            for (key, replacement) in variables {
                rendered = rendered.replace(&format!("${{{key}}}"), replacement);
            }
            Value::String(rendered)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| render_prompt_value(item, variables))
                .collect(),
        ),
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut ordered = serde_json::Map::with_capacity(map.len());
            for key in keys {
                ordered.insert(
                    key.clone(),
                    render_prompt_value(&map[key.as_str()], variables),
                );
            }
            Value::Object(ordered)
        }
        other => other.clone(),
    }
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

    // Render output in requested format.
    //
    // Only the Text arm used to receive `variables`; Yaml and Json called
    // to_yaml()/to_json() and dropped the map on the floor. Since
    // `prompt book` and `prompt comply` hardcode Yaml, no flag could ever
    // substitute anything: `--title MYBOOK` left ten literal `${BOOK_TITLE}`s
    // in the emitted prompt.
    let output_str = match format {
        PromptOutputFormat::Yaml => {
            let rendered = render_prompt_value(&serde_json::to_value(&prompt)?, &variables);
            serde_yaml_ng::to_string(&rendered)?
        }
        PromptOutputFormat::Json => {
            let rendered = render_prompt_value(&serde_json::to_value(&prompt)?, &variables);
            serde_json::to_string_pretty(&rendered)?
        }
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod variable_substitution_tests {
    //! Regression tests for `pmat prompt book` / `pmat prompt comply`: both
    //! hardcode the Yaml output format, and the Yaml and Json arms of
    //! `show_prompt` dropped the variable map, so `--title MYBOOK` emitted ten
    //! literal `${BOOK_TITLE}`s. The HashMap-backed model also gave a different
    //! key order — and so different bytes — on every run.
    use super::{
        render_prompt_value, show_prompt, PromptOutputFormat, Value, WorkflowPrompt,
        LIST_PROMPTS_USAGE, PROMPTS,
    };
    use std::collections::HashMap;

    /// Every invocation the `--list` footer advertises must be one clap accepts.
    /// The footer used to print `pmat prompt <name>`, which exits 2 with
    /// "unrecognized subcommand" — advice that cannot be followed, printed by
    /// the command that is meant to teach the interface.
    #[test]
    fn test_list_usage_lines_are_invocations_clap_accepts() {
        // 8MB stack: the `Cli` enum overflows the default test stack, the same
        // reason the other clap parsing tests spawn their own thread.
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                use clap::Parser;

                for (invocation, _) in LIST_PROMPTS_USAGE {
                    // Substitute the placeholders for a concrete prompt and
                    // assignment so the line can go to the parser verbatim.
                    let concrete = invocation
                        .replace("<name>", "book-documentation")
                        .replace("VAR=value", "TITLE=x");
                    let argv: Vec<&str> = concrete.split_whitespace().collect();

                    assert!(
                        crate::cli::Cli::try_parse_from(&argv).is_ok(),
                        "advertised usage `{invocation}` is not a runnable command"
                    );
                }
            })
            .expect("spawn")
            .join()
            .expect("advertised usage lines must all parse");
    }

    #[test]
    fn test_show_prompt_substitutes_on_the_yaml_path() {
        // `prompt book` / `prompt comply` hardcode Yaml, and that arm dropped
        // the variable map entirely — this pins the call site, not just the
        // renderer. `--output` is used because the alternative is stdout.
        let temp = tempfile::TempDir::new().unwrap();
        let out = temp.path().join("book.yaml");

        show_prompt(
            "book-documentation",
            false,
            vec![(
                "BOOK_TITLE".to_string(),
                Value::String("MYBOOK".to_string()),
            )],
            PromptOutputFormat::Yaml,
            Some(out.clone()),
        )
        .unwrap();

        let written = std::fs::read_to_string(&out).unwrap();
        assert!(
            written.contains("MYBOOK"),
            "--title must reach the YAML that show_prompt writes"
        );
        assert!(
            !written.contains("${BOOK_TITLE}"),
            "the Yaml arm must not emit unsubstituted placeholders"
        );
    }

    fn yaml_of(name: &str, variables: &HashMap<String, String>) -> String {
        let raw = PROMPTS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, y)| *y)
            .expect("prompt must exist");
        let prompt = WorkflowPrompt::from_yaml(raw).expect("prompt must parse");
        let value = serde_json::to_value(&prompt).expect("prompt must serialize");
        serde_yaml_ng::to_string(&render_prompt_value(&value, variables)).expect("yaml")
    }

    #[test]
    fn test_book_variables_are_substituted_in_yaml_output() {
        let mut variables = HashMap::new();
        variables.insert("BOOK_TITLE".to_string(), "MYBOOK".to_string());
        variables.insert("MIN_PASS_RATE".to_string(), "42".to_string());

        let yaml = yaml_of("book-documentation", &variables);

        assert!(
            yaml.contains("MYBOOK"),
            "--title must reach the emitted prompt"
        );
        assert!(
            !yaml.contains("${BOOK_TITLE}"),
            "no ${{BOOK_TITLE}} placeholder may survive substitution"
        );
        assert!(!yaml.contains("${MIN_PASS_RATE}"));
    }

    #[test]
    fn test_identical_input_gives_identical_bytes() {
        let variables = HashMap::new();
        // Parsed afresh each time, as two separate invocations would: the
        // model's HashMap fields otherwise serialise in a different order per
        // parse.
        for (name, _) in PROMPTS {
            let first = yaml_of(name, &variables);
            let second = yaml_of(name, &variables);
            assert_eq!(first, second, "prompt {name} is not byte-stable");
        }
    }

    #[test]
    fn test_object_keys_are_emitted_in_sorted_order() {
        let value = serde_json::json!({"zulu": 1, "alpha": {"yankee": 2, "bravo": 3}});
        let rendered =
            serde_json::to_string(&render_prompt_value(&value, &HashMap::new())).unwrap();
        assert!(rendered.find("alpha").unwrap() < rendered.find("zulu").unwrap());
        assert!(rendered.find("bravo").unwrap() < rendered.find("yankee").unwrap());
    }
}
