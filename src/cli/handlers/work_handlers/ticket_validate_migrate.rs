/// Collect semantic warnings for roadmap items (helper for handle_work_validate)
fn collect_semantic_warnings(roadmap: &crate::models::roadmap::Roadmap) -> Vec<String> {
    let mut warnings = Vec::new();
    for item in &roadmap.roadmap {
        if item.acceptance_criteria.is_empty()
            && !matches!(item.status, ItemStatus::Cancelled)
        {
            warnings.push(format!("⚠️  {} has no acceptance criteria", item.id));
        }
        if item.id.chars().count() > 50 {
            let truncated: String = item.id.chars().take(30).collect();
            warnings.push(format!(
                "⚠️  {} has a long ID ({} chars) - consider using shorter IDs",
                truncated,
                item.id.chars().count()
            ));
        }
    }
    warnings
}

/// Print valid roadmap with semantic validation (helper for handle_work_validate)
fn print_valid_roadmap(
    roadmap: &crate::models::roadmap::Roadmap,
    verbose: bool,
    fix: bool,
) {
    use crate::cli::colors as c;
    println!("{}", c::pass("Syntax valid"));
    println!("   {} {}", c::label("Version:"), roadmap.roadmap_version);
    println!("   {} {}", c::label("Items:"), c::number(&roadmap.roadmap.len().to_string()));
    println!(
        "   {} {}",
        c::label("GitHub:"),
        if roadmap.github_enabled {
            roadmap.github_repo.as_deref().unwrap_or("not configured")
        } else {
            "disabled"
        }
    );
    println!();

    let warnings = collect_semantic_warnings(roadmap);
    if !warnings.is_empty() {
        println!("{}", c::subheader(&format!("Warnings ({}):", warnings.len())));
        for warning in &warnings {
            println!("   {}", warning);
        }
        println!();
    }

    if verbose {
        println!("{}", c::subheader("📋 Items:"));
        for item in &roadmap.roadmap {
            println!("   {} [{:?}] - {}", c::path(&item.id), item.status, item.title);
        }
    }

    if fix && !warnings.is_empty() {
        println!("{}", c::dim("💡 Tip: Use `pmat work migrate` to auto-fix issues"));
    }

    println!("{}", c::pass("Validation passed"));
}

/// Print YAML parse error with context and suggestions (helper for handle_work_validate)
fn print_yaml_error_context(error_msg: &str, content: &str) {
    use crate::cli::colors as c;
    println!("{}", c::fail("Validation failed"));
    println!();
    println!("{} {}", c::label("Error:"), error_msg);
    println!();

    if let Some(line) = extract_line_from_yaml_error(error_msg) {
        let lines: Vec<&str> = content.lines().collect();
        if line > 0 && line <= lines.len() {
            println!("{}", c::subheader(&format!("Context (around line {}):", line)));
            let start = line.saturating_sub(3);
            let end = std::cmp::min(line + 2, lines.len());
            for (i, l) in lines[start..end].iter().enumerate() {
                let line_num = start + i + 1;
                let marker = if line_num == line {
                    format!("{}>>>{}", c::RED, c::RESET)
                } else {
                    "   ".to_string()
                };
                println!("{} {:4}: {}", marker, line_num, l);
            }
            println!();
        }
    }

    println!("{}", c::dim("💡 Common fixes:"));
    println!("   {}", c::dim("- Use valid status values: completed, done, wip, planned, blocked, review"));
    println!("   {}", c::dim("- Quote strings with special characters: `:`, `<`, `>`"));
    println!("   {}", c::dim("- Use proper YAML indentation (2 spaces)"));
    println!();
    println!("{}", c::dim("Run `pmat work status --list` to see all valid status values."));
}

/// Handle work validate command (Part B: UX Improvements)
///
/// Validates roadmap.yaml syntax and content with actionable error messages.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_validate(path: Option<PathBuf>, verbose: bool, fix: bool) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("{}", c::label(&format!("🔍 Validating roadmap: {}", c::path(&roadmap_path.display().to_string()))));
    println!();

    if !roadmap_path.exists() {
        anyhow::bail!(
            "Roadmap not found: {}\n\nRun `pmat work init` to create one.",
            roadmap_path.display()
        );
    }

    let content = std::fs::read_to_string(&roadmap_path).context("Failed to read roadmap file")?;

    match serde_yaml_ng::from_str::<crate::models::roadmap::Roadmap>(&content) {
        Ok(roadmap) => {
            print_valid_roadmap(&roadmap, verbose, fix);
            Ok(())
        }
        Err(e) => {
            print_yaml_error_context(&format!("{}", e), &content);
            anyhow::bail!("Roadmap validation failed")
        }
    }
}

/// Handle work migrate command (Part B: UX Improvements)
///
/// Auto-fixes common roadmap.yaml issues.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_migrate(path: Option<PathBuf>, dry_run: bool, backup: bool) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("{}", c::label(&format!("🔄 Migrating roadmap: {}", c::path(&roadmap_path.display().to_string()))));
    println!();

    if !roadmap_path.exists() {
        anyhow::bail!(
            "Roadmap not found: {}\n\nRun `pmat work init` to create one.",
            roadmap_path.display()
        );
    }

    let content = std::fs::read_to_string(&roadmap_path)?;
    let mut changes: Vec<String> = Vec::new();
    let mut new_content = content.clone();

    // 1. Normalize status values
    let status_patterns = [
        ("status: done", "status: completed"),
        ("status: Done", "status: completed"),
        ("status: DONE", "status: completed"),
        ("status: finished", "status: completed"),
        ("status: in progress", "status: inprogress"),
        ("status: In Progress", "status: inprogress"),
        ("status: WIP", "status: inprogress"),
        ("status: wip", "status: inprogress"),
        ("status: stuck", "status: blocked"),
        ("status: on-hold", "status: blocked"),
        ("status: todo", "status: planned"),
        ("status: TODO", "status: planned"),
        ("status: open", "status: planned"),
    ];

    for (old, new) in status_patterns {
        if new_content.contains(old) {
            changes.push(format!("Normalize status: {} → {}", old, new));
            new_content = new_content.replace(old, new);
        }
    }

    // 2. Quote special characters in titles
    let special_chars = [':', '<', '>', '≥', '≤', '±', 'ε', '→', '↔'];
    for line in content.lines() {
        if line.trim_start().starts_with("title:") || line.trim_start().starts_with("- title:") {
            let has_special = special_chars
                .iter()
                .any(|ch| line.contains(*ch) && !line.contains("\""));
            if has_special && !line.contains("\"") {
                // This is a simplistic check - in practice we'd need proper YAML parsing
                changes.push(format!("Consider quoting: {}", line.trim()));
            }
        }
    }

    if changes.is_empty() {
        println!("{}", c::pass("No migrations needed - roadmap is already up to date"));
        return Ok(());
    }

    println!("{} {} potential changes:", c::subheader("Found"), c::number(&changes.len().to_string()));
    for change in &changes {
        println!("   • {}", change);
    }
    println!();

    if dry_run {
        println!("{}", c::dim("(Dry run - no changes made)"));
        return Ok(());
    }

    // Create backup
    if backup {
        let backup_path = roadmap_path.with_extension("yaml.bak");
        std::fs::write(&backup_path, &content)?;
        println!("{}", c::pass(&format!("Created backup: {}", c::path(&backup_path.display().to_string()))));
    }

    // Write changes
    std::fs::write(&roadmap_path, &new_content)?;
    println!("{}", c::pass(&format!("Updated roadmap: {}", c::path(&roadmap_path.display().to_string()))));

    // Verify the changes
    if serde_yaml_ng::from_str::<crate::models::roadmap::Roadmap>(&new_content).is_ok() {
        println!("{}", c::pass("Verified: updated roadmap is valid"));
    } else {
        println!("{}", c::warn("Warning: updated roadmap may have issues - check manually"));
    }

    Ok(())
}

/// Handle work list-statuses command (Part B: UX Improvements)
///
/// Lists all valid status values with descriptions and aliases.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_work_list_statuses() -> Result<()> {
    use crate::cli::colors as c;
    println!("{}\n", c::subheader("📋 Valid Status Values"));
    println!("{}{:<15} {:<25} DESCRIPTION{}", c::BOLD, "STATUS", "ALIASES", c::RESET);
    println!("{}", c::separator());

    let statuses = [
        (
            "planned",
            "todo, open, pending, new",
            "Task not yet started",
        ),
        (
            "inprogress",
            "wip, active, started",
            "Currently being worked on",
        ),
        (
            "blocked",
            "stuck, waiting, on-hold",
            "Cannot proceed (waiting on something)",
        ),
        (
            "review",
            "reviewing, pr, pending-review",
            "Ready for or in code review",
        ),
        (
            "completed",
            "done, finished, closed",
            "Work finished successfully",
        ),
        (
            "cancelled",
            "canceled, dropped, wontfix",
            "Work abandoned or not needed",
        ),
    ];

    for (status, aliases, description) in statuses {
        println!("{}{:<15}{} {:<25} {}", c::CYAN, status, c::RESET, aliases, description);
    }

    println!();
    println!("{}", c::dim("💡 All status values are case-insensitive and support hyphens/underscores."));
    println!("   {}", c::dim("Example: 'In-Progress', 'in_progress', 'InProgress', 'WIP' all work."));

    Ok(())
}
