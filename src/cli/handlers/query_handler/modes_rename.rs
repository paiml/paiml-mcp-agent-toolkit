// ── Suggest-rename mode ─────────────────────────────────────────────────────

pub(super) fn handle_suggest_rename_mode(
    index: &AgentContextIndex,
    project_path: &std::path::Path,
    format: &QueryOutputFormat,
    path_pattern: &Option<String>,
    limit: usize,
    quiet: bool,
    apply: bool,
) -> anyhow::Result<()> {
    let path_filter = path_pattern.as_deref();
    let mut suggestions = suggest_renames(index, path_filter);

    // Filter out NoSignal entries (no useful suggestion)
    suggestions.retain(|s| s.signal != RenameSignal::NoSignal);

    if suggestions.len() > limit {
        suggestions.truncate(limit);
    }

    if !quiet {
        eprintln!(
            "Found {} _part_ files with rename suggestions",
            suggestions.len()
        );
    }

    match format {
        QueryOutputFormat::Json => {
            print_suggest_rename_json(&suggestions);
        }
        QueryOutputFormat::Markdown => {
            print_suggest_rename_markdown(&suggestions);
        }
        _ => {
            print_suggest_rename_text(&suggestions);
        }
    }

    if apply {
        execute_renames(&suggestions, project_path)?;
    }

    Ok(())
}

fn print_suggest_rename_text(suggestions: &[RenameSuggestion]) {
    println!(
        "\n{BOLD}Rename Suggestions{RESET} ({} _part_ files found)\n",
        suggestions.len()
    );

    for (i, s) in suggestions.iter().enumerate() {
        let conf_color = if s.confidence >= 0.90 {
            BRIGHT_GREEN
        } else if s.confidence >= 0.70 {
            GREEN
        } else if s.confidence >= 0.50 {
            YELLOW
        } else {
            DIM
        };

        println!(
            "  {BOLD}{:>3}.{RESET} {conf_color}[{:.2}]{RESET} {DIM_CYAN}{}{RESET} -> {CYAN}{}{RESET}",
            i + 1,
            s.confidence,
            s.current_path,
            s.suggested_name
        );
        println!(
            "       {DIM}Signal: {signal:?} ({reasoning}) | {count} definitions{RESET}",
            signal = s.signal,
            reasoning = s.reasoning,
            count = s.definition_count
        );
        if let Some(ref parent) = s.parent_file {
            println!(
                "       {DIM}Parent: {parent} ({pattern}){RESET}",
                pattern = s.inclusion_pattern.as_deref().unwrap_or("unknown")
            );
        }
        println!();
    }
}

fn print_suggest_rename_json(suggestions: &[RenameSuggestion]) {
    match serde_json::to_string_pretty(suggestions) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("JSON serialization error: {e}"),
    }
}

fn print_suggest_rename_markdown(suggestions: &[RenameSuggestion]) {
    println!("# Rename Suggestions\n");
    println!("| # | Confidence | Current Path | Suggested Name | Signal | Definitions |");
    println!("|---|-----------|-------------|----------------|--------|-------------|");
    for (i, s) in suggestions.iter().enumerate() {
        println!(
            "| {} | {:.2} | `{}` | `{}` | {:?} | {} |",
            i + 1,
            s.confidence,
            s.current_path,
            s.suggested_name,
            s.signal,
            s.definition_count
        );
    }
}

fn execute_renames(
    suggestions: &[RenameSuggestion],
    project_path: &std::path::Path,
) -> anyhow::Result<()> {
    let applicable: Vec<&RenameSuggestion> = suggestions
        .iter()
        .filter(|s| s.confidence >= 0.70 && s.signal != RenameSignal::NoSignal)
        .collect();

    if applicable.is_empty() {
        eprintln!("{YELLOW}No suggestions with confidence >= 0.70 to apply{RESET}");
        return Ok(());
    }

    eprintln!("\n{BOLD}Applying {} renames...{RESET}\n", applicable.len());

    let mut success_count = 0;
    for s in &applicable {
        let old_path = project_path.join(&s.current_path);
        let new_path = project_path.join(&s.suggested_path);

        // Update parent file if detected
        if let Some(ref parent) = s.parent_file {
            let parent_path = project_path.join(parent);
            if parent_path.exists() {
                let old_filename = std::path::Path::new(&s.current_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if let Ok(content) = std::fs::read_to_string(&parent_path) {
                    let updated = content.replace(old_filename, &s.suggested_name);
                    if updated != content {
                        if let Err(e) = std::fs::write(&parent_path, &updated) {
                            eprintln!("  {RED}Failed to update parent {parent}: {e}{RESET}");
                            continue;
                        }
                        eprintln!("  {GREEN}Updated{RESET} {parent}");
                    }
                }
            }
        }

        // Perform git mv
        let status = std::process::Command::new("git")
            .args([
                "mv",
                &old_path.to_string_lossy(),
                &new_path.to_string_lossy(),
            ])
            .current_dir(project_path)
            .status();

        match status {
            Ok(exit) if exit.success() => {
                eprintln!(
                    "  {GREEN}Renamed{RESET} {} -> {}",
                    s.current_path, s.suggested_name
                );
                success_count += 1;
            }
            Ok(_) => {
                eprintln!("  {RED}git mv failed{RESET} for {}", s.current_path);
            }
            Err(e) => {
                eprintln!("  {RED}Error{RESET}: {e}");
            }
        }
    }

    eprintln!(
        "\n{BOLD}Done:{RESET} {success_count}/{} renames applied",
        applicable.len()
    );

    if success_count > 0 {
        eprintln!("{DIM}Run `cargo check` to verify the renames compile correctly{RESET}");
    }

    Ok(())
}

