// Migration, diff, and update handlers for comply subcommands.
//
// This file is include!()'d into migrate_handlers.rs scope,
// which itself is include!()'d into comply_handlers/mod.rs.
// No `use` imports or `#!` inner attributes allowed.

/// Migrate project to latest PMAT standards
async fn handle_migrate(
    project_path: &Path,
    target_version: Option<&str>,
    dry_run: bool,
    no_backup: bool,
    force: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::cli::colors as c;

    let target = target_version.unwrap_or(PMAT_VERSION);
    println!("{}", c::header(&format!("Migrating project to PMAT v{}", target)));

    if dry_run {
        println!("{}\n", c::dim("(dry-run mode - no changes will be made)"));
    }

    let config = load_or_create_project_config(project_path)?;
    let current_version = &config.pmat.version;

    println!("{} {}", c::label("Current version:"), current_version);
    println!("{} {}\n", c::label("Target version: "), target);

    let breaking_changes = get_breaking_changes_since(current_version);
    if !breaking_changes.is_empty() && !force {
        println!(
            "{}",
            c::warn(&format!("{} breaking changes detected:", breaking_changes.len()))
        );
        for change in &breaking_changes {
            println!("  - v{}: {}", change.version, change.description);
        }
        println!("\nUse --force to proceed anyway\n");
        if !force {
            return Ok(());
        }
    }

    if !no_backup && !dry_run {
        let backup_path = project_path.join(".pmat").join("backup");
        fs::create_dir_all(&backup_path)?;
        println!("{} {}", c::pass("Created backup at:"), c::path(&backup_path.display().to_string()));
    }

    let migrations = vec![
        (
            "Update project.toml version",
            migrate_project_version(project_path, target, dry_run),
        ),
        ("Update gitignore", migrate_gitignore(project_path, dry_run)),
    ];

    println!("\n{}", c::label("Migration steps:"));
    for (name, result) in migrations {
        match result {
            Ok(true) => println!("  {}", c::pass(name)),
            Ok(false) => println!("  {}", c::skip(&format!("{} (no changes needed)", name))),
            Err(e) => println!("  {}", c::fail(&format!("{} - {}", name, e))),
        }
    }

    // Update hooks (async operation)
    match update_project_hooks(project_path, dry_run).await {
        Ok(true) => println!("  {}", c::pass("Update git hooks")),
        Ok(false) => println!("  {}", c::skip("Update git hooks (no changes needed)")),
        Err(e) => println!("  {}", c::fail(&format!("Update git hooks - {}", e))),
    }

    if dry_run {
        println!("\n{}", c::dim("(dry-run complete - no changes were made)"));
    } else {
        println!("\n{}", c::pass("Migration complete!"));
    }

    Ok(())
}

/// Show changelog between versions
async fn handle_diff(
    project_path: &Path,
    from_version: Option<&str>,
    to_version: Option<&str>,
    breaking_only: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::cli::colors as c;

    let config = load_or_create_project_config(project_path)?;
    let from = from_version.unwrap_or(&config.pmat.version);
    let to = to_version.unwrap_or(PMAT_VERSION);

    println!("{}\n", c::header(&format!("PMAT Changelog: v{} \u{2192} v{}", from, to)));

    let changes = get_changelog_entries(from, to);

    if breaking_only {
        println!("{}\n", c::warn("Breaking Changes Only:"));
        let breaking: Vec<_> = changes.iter().filter(|c| c.breaking).collect();
        if breaking.is_empty() {
            println!("  {}", c::dim("No breaking changes between these versions."));
        } else {
            for entry in breaking {
                println!(
                    "  {}[BREAKING]{} v{}: {}",
                    c::BOLD_RED, c::RESET,
                    entry.version, entry.description
                );
            }
        }
    } else {
        for entry in &changes {
            if entry.breaking {
                println!(
                    "  {}[BREAKING]{} v{}: {}",
                    c::BOLD_RED, c::RESET,
                    entry.version, entry.description
                );
            } else {
                println!(
                    "  {}[FEATURE]{} v{}: {}",
                    c::BOLD_GREEN, c::RESET,
                    entry.version, entry.description
                );
            }
        }
    }

    Ok(())
}

/// Update hooks and configs
async fn handle_update(
    project_path: &Path,
    update_hooks: bool,
    update_config: bool,
    dry_run: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::cli::colors as c;

    let update_both = !update_hooks && !update_config;

    if dry_run {
        println!("{}\n", c::dim("(dry-run mode - no changes will be made)"));
    }

    if update_hooks || update_both {
        println!("{}", c::label("Updating hooks..."));
        match update_project_hooks(project_path, dry_run).await {
            Ok(true) => println!("  {}", c::pass("Hooks updated to latest templates")),
            Ok(false) => println!("  {}", c::skip("Hooks already up to date")),
            Err(e) => println!("  {}", c::fail(&format!("Failed: {}", e))),
        }
    }

    if update_config || update_both {
        println!("{}", c::label("Updating config..."));
        match update_project_config(project_path, dry_run) {
            Ok(true) => println!("  {}", c::pass(&format!("Config updated to v{}", PMAT_VERSION))),
            Ok(false) => println!("  {}", c::skip("Config already up to date")),
            Err(e) => println!("  {}", c::fail(&format!("Failed: {}", e))),
        }
    }

    Ok(())
}
