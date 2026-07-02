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
pub async fn handle_work_migrate(
    path: Option<PathBuf>,
    dry_run: bool,
    backup: bool,
    levels: bool,
) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    // MACS-004: --levels migrates .pmat-work contract verification levels
    // and is independent of the roadmap.yaml migration below.
    if levels {
        return migrate_verification_levels(&project_path, dry_run);
    }
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

/// MACS-004: rewrite legacy `verification_level` strings in every
/// `.pmat-work/<TICKET>/contract.json`. Lenient-parseable values ("l4",
/// "L3 ", "L4 (kani_proof)") are canonicalized to recover intent; values
/// outside the ladder become "L0" plus an audit note in
/// `references.spec_sections` (no silent rewrite).
fn migrate_verification_levels(project_path: &std::path::Path, dry_run: bool) -> Result<()> {
    use crate::cli::colors as c;
    use crate::cli::handlers::work_verification_level::VerificationLevel;

    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        println!("{}", c::warn("No .pmat-work directory — nothing to migrate"));
        return Ok(());
    }

    let mut migrated = 0usize;
    let mut invalid = 0usize;
    let mut scanned = 0usize;

    for entry in std::fs::read_dir(&work_dir).context("read .pmat-work")?.flatten() {
        let contract_path = entry.path().join("contract.json");
        if !contract_path.exists() {
            continue;
        }
        let text = match std::fs::read_to_string(&contract_path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let mut value: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let Some(raw) = value.get("verification_level").and_then(|v| v.as_str()) else {
            continue;
        };
        scanned += 1;
        if VerificationLevel::parse_strict(raw).is_some() {
            continue; // already canonical
        }
        let raw_owned = raw.to_string();
        let first_token = raw_owned.split_whitespace().next().unwrap_or("");
        let (new_level, note) = match VerificationLevel::parse_lenient(&raw_owned)
            .or_else(|| VerificationLevel::parse_lenient(first_token))
        {
            Some(level) => (level, "canonicalized"),
            None => {
                invalid += 1;
                (VerificationLevel::L0, "invalid; downgraded")
            }
        };
        let audit = format!(
            "MIGRATION(MACS-004): verification_level '{}' -> {} ({})",
            raw_owned, new_level, note
        );
        println!(
            "  {} {}: '{}' -> {}",
            if dry_run { c::dim("[dry-run]") } else { c::pass("") },
            entry.file_name().to_string_lossy(),
            raw_owned,
            new_level
        );
        if dry_run {
            migrated += 1;
            continue;
        }
        value["verification_level"] = serde_json::Value::String(new_level.to_string());
        if let Some(sections) = value
            .get_mut("references")
            .and_then(|r| r.get_mut("spec_sections"))
            .and_then(|s| s.as_array_mut())
        {
            sections.push(serde_json::Value::String(audit));
        } else {
            value["references"] = serde_json::json!({
                "arxiv": [], "spec_sections": [audit],
                "five_whys_id": null, "oracle_context": null
            });
        }
        let pretty = serde_json::to_string_pretty(&value).context("serialize contract")?;
        std::fs::write(&contract_path, pretty).context("write contract")?;
        migrated += 1;
    }

    println!();
    println!(
        "{}",
        c::pass(&format!(
            "Level migration: {scanned} contract(s) scanned, {migrated} rewritten, {invalid} invalid -> L0"
        ))
    );
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod level_migration_tests {
    use super::*;

    fn write_contract_with_level(project: &std::path::Path, ticket: &str, level: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "work_item_id": ticket,
                "verification_level": level,
                "references": {"arxiv": [], "spec_sections": []}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn read_level(project: &std::path::Path, ticket: &str) -> (String, Vec<String>) {
        let text = std::fs::read_to_string(
            project.join(".pmat-work").join(ticket).join("contract.json"),
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let level = v["verification_level"].as_str().unwrap().to_string();
        let notes = v["references"]["spec_sections"]
            .as_array()
            .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default();
        (level, notes)
    }

    #[test]
    fn legacy_strings_interactive_to_typed() {
        let project = tempfile::tempdir().unwrap();
        write_contract_with_level(project.path(), "T-CASE", "l4");
        write_contract_with_level(project.path(), "T-ANNOT", "L4 (kani_proof)");
        write_contract_with_level(project.path(), "T-BAD", "strong");
        write_contract_with_level(project.path(), "T-OK", "L3");

        migrate_verification_levels(project.path(), false).unwrap();

        let (level, notes) = read_level(project.path(), "T-CASE");
        assert_eq!(level, "L4", "case corruption recovers intent");
        assert!(notes.iter().any(|n| n.contains("MIGRATION(MACS-004)")));

        let (level, _) = read_level(project.path(), "T-ANNOT");
        assert_eq!(level, "L4", "annotated variant recovers first token");

        let (level, notes) = read_level(project.path(), "T-BAD");
        assert_eq!(level, "L0", "invalid value downgrades to L0");
        assert!(
            notes.iter().any(|n| n.contains("'strong'") && n.contains("invalid")),
            "audit note records the original value: {notes:?}"
        );

        let (level, notes) = read_level(project.path(), "T-OK");
        assert_eq!(level, "L3", "canonical values untouched");
        assert!(notes.is_empty(), "no audit note for untouched contracts");
    }

    #[test]
    fn dry_run_leaves_files_untouched() {
        let project = tempfile::tempdir().unwrap();
        write_contract_with_level(project.path(), "T-DRY", "l5");
        migrate_verification_levels(project.path(), true).unwrap();
        let (level, notes) = read_level(project.path(), "T-DRY");
        assert_eq!(level, "l5");
        assert!(notes.is_empty());
    }

    #[test]
    fn contract_deserializes_typed_with_migration() {
        use crate::cli::handlers::work_contract::WorkContract;
        use crate::cli::handlers::work_verification_level::VerificationLevel;
        let parse = |raw: &str| -> VerificationLevel {
            let base = WorkContract::new("T".to_string(), "abc".to_string());
            let mut value = serde_json::to_value(&base).expect("to_value");
            value["verification_level"] = serde_json::Value::String(raw.to_string());
            let contract: WorkContract =
                serde_json::from_value(value).expect("contract parses");
            contract.verification_level
        };
        assert_eq!(parse("L4"), VerificationLevel::L4);
        assert_eq!(parse("l4"), VerificationLevel::L4, "lenient read migration");
        assert_eq!(parse("L4 (kani_proof)"), VerificationLevel::L4, "annotated");
        assert_eq!(parse("strong"), VerificationLevel::L0, "invalid -> L0");
    }
}
