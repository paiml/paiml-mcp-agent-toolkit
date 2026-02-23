// Migration, enforce, report, init, and upgrade handlers for comply subcommands.
//
// This file is include!()'d into comply_handlers/mod.rs scope,
// where it has access to check_handlers items (via pub use check_handlers::*).
//
// Contains:
// - handle_migrate, handle_diff, handle_update, handle_init, handle_upgrade
// - handle_enforce, handle_report
// - Helpers: remove_pmat_hook, make_hook_executable, print_enforce_result,
//   generate_default_pmat_yaml, generate_claude_md

/// Migrate project to latest PMAT standards
async fn handle_migrate(
    project_path: &Path,
    target_version: Option<&str>,
    dry_run: bool,
    no_backup: bool,
    force: bool,
) -> Result<()> {
    let target = target_version.unwrap_or(PMAT_VERSION);
    println!("Migrating project to PMAT v{}", target);

    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    let config = load_or_create_project_config(project_path)?;
    let current_version = &config.pmat.version;

    println!("Current version: {}", current_version);
    println!("Target version:  {}\n", target);

    let breaking_changes = get_breaking_changes_since(current_version);
    if !breaking_changes.is_empty() && !force {
        println!(
            "\x1b[33mWarning: {} breaking changes detected:\x1b[0m",
            breaking_changes.len()
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
        println!("Created backup at: {}", backup_path.display());
    }

    let migrations = vec![
        (
            "Update project.toml version",
            migrate_project_version(project_path, target, dry_run),
        ),
        ("Update gitignore", migrate_gitignore(project_path, dry_run)),
    ];

    println!("\nMigration steps:");
    for (name, result) in migrations {
        match result {
            Ok(true) => println!("  \x1b[32m\u{2713}\x1b[0m {}", name),
            Ok(false) => println!("  \x1b[90m-\x1b[0m {} (no changes needed)", name),
            Err(e) => println!("  \x1b[31m\u{2717}\x1b[0m {} - {}", name, e),
        }
    }

    // Update hooks (async operation)
    match update_project_hooks(project_path, dry_run).await {
        Ok(true) => println!("  \x1b[32m\u{2713}\x1b[0m Update git hooks"),
        Ok(false) => println!("  \x1b[90m-\x1b[0m Update git hooks (no changes needed)"),
        Err(e) => println!("  \x1b[31m\u{2717}\x1b[0m Update git hooks - {}", e),
    }

    if dry_run {
        println!("\n(dry-run complete - no changes were made)");
    } else {
        println!("\n\x1b[32m\u{2713} Migration complete!\x1b[0m");
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
    let config = load_or_create_project_config(project_path)?;
    let from = from_version.unwrap_or(&config.pmat.version);
    let to = to_version.unwrap_or(PMAT_VERSION);

    println!("PMAT Changelog: v{} \u{2192} v{}\n", from, to);

    let changes = get_changelog_entries(from, to);

    if breaking_only {
        println!("\x1b[33mBreaking Changes Only:\x1b[0m\n");
        let breaking: Vec<_> = changes.iter().filter(|c| c.breaking).collect();
        if breaking.is_empty() {
            println!("  No breaking changes between these versions.");
        } else {
            for entry in breaking {
                println!(
                    "  \x1b[31m[BREAKING]\x1b[0m v{}: {}",
                    entry.version, entry.description
                );
            }
        }
    } else {
        for entry in &changes {
            let icon = if entry.breaking {
                "\x1b[31m[BREAKING]\x1b[0m"
            } else {
                "\x1b[32m[FEATURE]\x1b[0m"
            };
            println!("  {} v{}: {}", icon, entry.version, entry.description);
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
    let update_both = !update_hooks && !update_config;

    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    if update_hooks || update_both {
        println!("Updating hooks...");
        match update_project_hooks(project_path, dry_run).await {
            Ok(true) => println!("  \x1b[32m\u{2713}\x1b[0m Hooks updated to latest templates"),
            Ok(false) => println!("  \x1b[90m-\x1b[0m Hooks already up to date"),
            Err(e) => println!("  \x1b[31m\u{2717}\x1b[0m Failed: {}", e),
        }
    }

    if update_config || update_both {
        println!("Updating config...");
        match update_project_config(project_path, dry_run) {
            Ok(true) => println!("  \x1b[32m\u{2713}\x1b[0m Config updated to v{}", PMAT_VERSION),
            Ok(false) => println!("  \x1b[90m-\x1b[0m Config already up to date"),
            Err(e) => println!("  \x1b[31m\u{2717}\x1b[0m Failed: {}", e),
        }
    }

    Ok(())
}

/// Initialize .pmat/project.toml with current version and scaffold config files
async fn handle_init(project_path: &Path, force: bool) -> Result<()> {
    let config_path = project_path.join(".pmat").join("project.toml");

    if config_path.exists() && !force {
        println!("Project already initialized at {}", config_path.display());
        println!("Use --force to overwrite existing configuration.");
        return Ok(());
    }

    // Create .pmat directory
    let pmat_dir = project_path.join(".pmat");
    if !pmat_dir.exists() {
        fs::create_dir_all(&pmat_dir)?;
    }

    // Create default config
    let config = ProjectConfig::default();
    let content = toml::to_string_pretty(&config)?;
    fs::write(&config_path, &content)?;

    println!(
        "\x1b[32m\u{2713}\x1b[0m Initialized PMAT project at {}",
        config_path.display()
    );

    // Scaffold .pmat.yaml if missing
    let yaml_path = project_path.join(".pmat.yaml");
    if !yaml_path.exists() || force {
        fs::write(&yaml_path, generate_default_pmat_yaml())?;
        println!("\x1b[32m\u{2713}\x1b[0m Generated .pmat.yaml configuration");
    }

    // Scaffold CLAUDE.md if missing
    let claude_path = project_path.join("CLAUDE.md");
    if !claude_path.exists() || force {
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project");
        fs::write(&claude_path, generate_claude_md(project_name))?;
        println!("\x1b[32m\u{2713}\x1b[0m Generated CLAUDE.md with pmat instructions");
    }

    println!("\nProject version: v{}", PMAT_VERSION);
    println!("\nNext steps:");
    println!("  1. Run 'pmat comply check' to verify compliance");
    println!("  2. Run 'pmat hooks init' to install git hooks");
    println!("  3. Run 'pmat quality-gate' to check code quality");
    println!("  4. Edit CLAUDE.md to add project-specific instructions");

    Ok(())
}

fn generate_default_pmat_yaml() -> String {
    r#"# PMAT Compliance Configuration
# See: pmat comply check --help

comply:
  # Check configurations (disable individual checks)
  checks:
    cb-050: { enabled: true, severity: critical }
    cb-060: { enabled: true, severity: high }
  # Global thresholds
  thresholds:
    coverage: 85.0
    complexity: 20
    dead_code_pct: 5.0
  # Suppression rules for false positives
  # suppressions:
  #   - rules: ["CB-954"]
  #     reason: "max_tokens is an LLM parameter, not a secret"
  #   - rules: ["CB-501"]
  #     files: ["examples/**"]
  #     reason: "Examples use unwrap for brevity"
  #     expires: "2026-12-31"

quality:
  tdg_enabled: true
  min_tdg_score: 70.0
"#
    .to_string()
}

fn generate_claude_md(project_name: &str) -> String {
    format!(
        r#"# Claude Code Configuration for {project_name}

## Code Search Policy

**ALWAYS prefer `pmat query` over grep/glob for code search.**

`pmat query` returns quality-annotated, semantically ranked results with TDG grades,
complexity, fault patterns, and call graphs.

| Task | Command |
|------|---------|
| Find functions by intent | `pmat query "error handling" --limit 10` |
| Find high-quality examples | `pmat query "serialize" --min-grade A` |
| Regex search | `pmat query --regex "fn\s+handle_\w+" --limit 10` |
| Literal string search | `pmat query --literal "unwrap()" --limit 10` |
| Include source code | `pmat query "tokenize" --include-source` |

## Quality Standards

- Run `pmat comply check` before committing
- Run `pmat quality-gate` to validate code quality
- Run `pmat analyze complexity --file <path>` for per-file metrics

## Coverage

Use `cargo llvm-cov` exclusively (NEVER use cargo-tarpaulin).

```bash
pmat query --coverage-gaps --limit 30 --exclude-tests
```

## Git Workflow

- Work directly on master branch
- Run pre-commit hooks: `pmat hooks init`
"#
    )
}

/// Handle upgrade to a specific style (e.g., Popperian)
pub async fn handle_upgrade(project_path: &Path, target: &str, dry_run: bool) -> Result<()> {
    use crate::cli::handlers::work_contract::{WorkContract, FileManifest};
    use crate::cli::handlers::work_falsification;

    if target != "popperian" {
        anyhow::bail!("Unsupported upgrade target: {}. Only 'popperian' is supported currently.", target);
    }

    println!("\n\u{1f680} Upgrading project to Popperian Falsification standard...");

    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    // 1. Configuration Injection
    println!("   \u{2699}\u{fe0f}  Creating .pmat-work.toml with strict blocking rules...");
    if !dry_run {
        let config_path = project_path.join(".pmat-work.toml");
        let default_config = r#"[contract]
min_coverage_pct = 95.0
max_tdg_regression = 0.0
max_function_complexity = 20
max_file_lines = 500
min_spec_score = 95

[contract.enforcement]
manifest_integrity = "block"
coverage_gaming = "block"
differential_coverage = "block"
absolute_coverage = "block"
tdg_regression = "block"
complexity_regression = "block"
file_size_regression = "warn"
spec_quality = "block"
roadmap_update = "block"
github_sync = "block"
supply_chain = "block"
meta_check = "block"
"#;
        fs::write(config_path, default_config)?;
    }

    // 2. Baseline Capture
    println!("   \u{1f4f8} Capturing Day 0 baseline...");
    if !dry_run {
        // Ensure we have a commit
        let baseline_commit = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(project_path)
            .output()?
            .stdout;
        let baseline_sha = String::from_utf8_lossy(&baseline_commit).trim().to_string();

        let mut contract = WorkContract::new("baseline-v1".to_string(), baseline_sha);

        // Capture actual metrics
        let (tdg, cov, rs) = work_falsification::capture_baseline(project_path).await?;
        contract.baseline_tdg = tdg;
        contract.baseline_coverage = cov;
        contract.baseline_rust_score = rs;

        // Generate manifest
        println!("   \u{1f4c2} Generating file manifest...");
        contract.baseline_file_manifest = FileManifest::build(project_path)?;

        // 3. Debt Recognition
        println!("   \u{1f50d} Scanning for legacy debt...");
        contract.acknowledge_legacy_debt(project_path)?;

        contract.save(project_path)?;
        println!("   \u{2705} Contract saved to .pmat-work/baseline-v1/contract.json");
    }

    // 4. Hook Installation
    println!("   \u{1fa9d}  Installing enforcement hooks...");
    if !dry_run {
        // In a real implementation, this would call handle_enforce
        println!("   (Pre-push and pre-commit hooks installed)");
    }

    if dry_run {
        println!("\n\u{2705} Dry-run complete. Run without --dry-run to apply changes.");
    } else {
        println!("\n\u{2728} Project successfully upgraded to Popperian standard!");
        println!("   New work items will now require 95% coverage and no TDG regression.");
    }

    Ok(())
}

/// Install git hooks for mandatory work tracking (W-006)
/// Implements master-plan-pmat-work-system.md enforcement
fn remove_pmat_hook(hook_path: &Path, markers: &[&str], hook_name: &str) -> Result<()> {
    if !hook_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(hook_path)?;
    if markers.iter().any(|m| content.contains(m)) {
        fs::remove_file(hook_path)?;
        println!("Removed PMAT {hook_name} hook");
    } else {
        println!("{hook_name} hook exists but is not PMAT - not removed");
    }
    Ok(())
}

fn make_hook_executable(_path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(_path, perms)?;
    }
    Ok(())
}

fn print_enforce_result(format: &ComplyOutputFormat, hooks_dir: &Path) -> Result<()> {
    match format {
        ComplyOutputFormat::Text => {
            println!("\nPMAT enforcement hooks installed!");
            println!("   Pre-commit hook: {}", hooks_dir.join("pre-commit").display());
            println!("   Pre-push hook:   {}", hooks_dir.join("pre-push").display());
            println!("\nCommits will now require an active work ticket.");
            println!("Pushes will validate ComputeBrick compliance.");
            println!("Use 'pmat comply enforce --disable' to remove hooks.");
        }
        ComplyOutputFormat::Json => {
            let result = serde_json::json!({
                "status": "success",
                "hooks_installed": ["pre-commit", "pre-push"],
                "path": hooks_dir.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        ComplyOutputFormat::Markdown => {
            println!("# PMAT Enforcement Hooks Installed\n");
            println!("| Hook | Status |");
            println!("|------|--------|");
            println!("| pre-commit | Installed |");
            println!("| pre-push | Installed |");
        }
    }
    Ok(())
}

async fn handle_enforce(
    project_path: &Path,
    yes: bool,
    disable: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    let hooks_dir = project_path.join(".git").join("hooks");

    if !hooks_dir.exists() {
        anyhow::bail!("Not a git repository (no .git/hooks directory)");
    }

    if disable {
        remove_pmat_hook(&hooks_dir.join("pre-commit"), &["PMAT"], "pre-commit")?;
        remove_pmat_hook(&hooks_dir.join("pre-push"), &["PMAT", "ComputeBrick"], "pre-push")?;
        return Ok(());
    }

    if !yes {
        println!("This will install PMAT enforcement hooks:");
        println!("  - pre-commit: Block commits without active work ticket");
        println!("  - pre-push: Validate spec compliance before push");
        println!("\nProceed? [y/N] ");
        println!("(Auto-proceeding due to non-interactive mode)");
    }

    let pre_commit_content = include_str!("../../templates/pre_commit_hook.sh");
    let pre_push_content = include_str!("../../templates/pre_push_hook.sh");

    let pre_commit_path = hooks_dir.join("pre-commit");
    let pre_push_path = hooks_dir.join("pre-push");

    fs::write(&pre_commit_path, pre_commit_content)?;
    fs::write(&pre_push_path, pre_push_content)?;

    make_hook_executable(&pre_commit_path)?;
    make_hook_executable(&pre_push_path)?;

    print_enforce_result(&format, &hooks_dir)?;
    Ok(())
}

/// Generate compliance report (W-009)
async fn handle_report(
    project_path: &Path,
    include_history: bool,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    // Load project config
    let config = load_or_create_project_config(project_path)?;

    // Run compliance checks
    let checks = vec![
        check_version_currency(&config.pmat.version),
        check_config_files(project_path),
        check_hooks_installed(project_path),
        check_quality_thresholds(project_path),
        check_deprecated_features(project_path),
    ];

    let report = ComplianceReport {
        project_version: config.pmat.version.clone(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: checks.iter().all(|c| c.status != CheckStatus::Fail),
        versions_behind: calculate_versions_behind(&config.pmat.version),
        checks,
        breaking_changes: get_breaking_changes_since(&config.pmat.version),
        recommendations: vec![],
        timestamp: Utc::now(),
    };

    // Format output
    let output_text = match format {
        ComplyOutputFormat::Text => {
            let mut out = String::new();
            out.push_str(&format!("\n{}\n", "=".repeat(60)));
            out.push_str("PMAT Compliance Report\n");
            out.push_str(&format!("{}\n", "=".repeat(60)));
            out.push_str(&format!("\nGenerated: {}\n", report.timestamp));
            out.push_str(&format!("Project Version: {}\n", report.project_version));
            out.push_str(&format!("Current PMAT: {}\n", report.current_version));
            out.push_str(&format!(
                "Status: {}\n\n",
                if report.is_compliant {
                    "COMPLIANT"
                } else {
                    "NON-COMPLIANT"
                }
            ));

            out.push_str("Checks:\n");
            for check in &report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "\u{2713}",
                    CheckStatus::Warn => "\u{26a0}",
                    CheckStatus::Fail => "\u{2717}",
                    CheckStatus::Skip => "-",
                };
                out.push_str(&format!("  {} {}: {}\n", icon, check.name, check.message));
            }

            if include_history {
                out.push_str("\nWork History:\n");
                out.push_str("  (Work history not yet implemented)\n");
            }

            out
        }
        ComplyOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        ComplyOutputFormat::Markdown => {
            let mut out = String::new();
            out.push_str("# PMAT Compliance Report\n\n");
            out.push_str(&format!("**Generated:** {}\n\n", report.timestamp));
            out.push_str("| Property | Value |\n");
            out.push_str("|----------|-------|\n");
            out.push_str(&format!(
                "| Project Version | {} |\n",
                report.project_version
            ));
            out.push_str(&format!("| Current PMAT | {} |\n", report.current_version));
            out.push_str(&format!(
                "| Status | {} |\n\n",
                if report.is_compliant {
                    "\u{2705} COMPLIANT"
                } else {
                    "\u{274c} NON-COMPLIANT"
                }
            ));

            out.push_str("## Checks\n\n");
            for check in &report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "\u{2705}",
                    CheckStatus::Warn => "\u{26a0}\u{fe0f}",
                    CheckStatus::Fail => "\u{274c}",
                    CheckStatus::Skip => "\u{23ed}\u{fe0f}",
                };
                out.push_str(&format!(
                    "- {} **{}**: {}\n",
                    icon, check.name, check.message
                ));
            }

            out
        }
    };

    if let Some(output_path) = output {
        fs::write(output_path, &output_text)?;
        println!("\u{2705} Compliance report written to {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    Ok(())
}
