// Enforce and report handlers for comply subcommands.
//
// This file is include!()'d into migrate_handlers.rs scope,
// which itself is include!()'d into comply_handlers/mod.rs.
// No `use` imports or `#!` inner attributes allowed.

/// Install git hooks for mandatory work tracking (W-006)
/// Implements master-plan-pmat-work-system.md enforcement
fn remove_pmat_hook(hook_path: &Path, markers: &[&str], hook_name: &str) -> Result<()> {
    use crate::cli::colors as c;
    if !hook_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(hook_path)?;
    if markers.iter().any(|m| content.contains(m)) {
        fs::remove_file(hook_path)?;
        crate::status_println!("{}", c::pass(&format!("Removed PMAT {hook_name} hook")));
    } else {
        println!("{}", c::warn(&format!("{hook_name} hook exists but is not PMAT - not removed")));
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
    use crate::cli::colors as c;
    match format {
        ComplyOutputFormat::Text => {
            println!("\n{}", c::pass("PMAT enforcement hooks installed!"));
            println!("   {} {}", c::label("Pre-commit hook:"), c::path(&hooks_dir.join("pre-commit").display().to_string()));
            println!("   {} {}", c::label("Pre-push hook:  "), c::path(&hooks_dir.join("pre-push").display().to_string()));
            println!("\nCommits will now require an active work ticket.");
            println!("Pushes will validate ComputeBrick compliance.");
            println!("Use '{}' to remove hooks.", c::label("pmat comply enforce --disable"));
        }
        ComplyOutputFormat::Json | ComplyOutputFormat::Sarif => {
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
        use crate::cli::colors as c;
        crate::status_println!("{}", c::label("This will install PMAT enforcement hooks:"));
        crate::status_println!("  - {}: Block commits without active work ticket", c::label("pre-commit"));
        crate::status_println!("  - {}: Validate spec compliance before push", c::label("pre-push"));
        crate::status_println!("\nProceed? [y/N] ");
        crate::status_println!("{}", c::dim("(Auto-proceeding due to non-interactive mode)"));
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
            use crate::cli::colors as c;
            let mut out = String::new();
            out.push_str(&format!("\n{}\n", c::rule()));
            out.push_str(&format!("{}\n", c::header("PMAT Compliance Report")));
            out.push_str(&format!("{}\n", c::rule()));
            out.push_str(&format!("\n{} {}\n", c::label("Generated:"), report.timestamp));
            out.push_str(&format!("{} {}\n", c::label("Project Version:"), report.project_version));
            out.push_str(&format!("{} {}\n", c::label("Current PMAT:"), report.current_version));
            let status_str = if report.is_compliant {
                format!("{}COMPLIANT{}", c::BOLD_GREEN, c::RESET)
            } else {
                format!("{}NON-COMPLIANT{}", c::BOLD_RED, c::RESET)
            };
            out.push_str(&format!("{} {}\n\n", c::label("Status:"), status_str));

            out.push_str(&format!("{}:\n", c::label("Checks")));
            for check in &report.checks {
                let line = format!("{}: {}", check.name, check.message);
                let formatted = match check.status {
                    CheckStatus::Pass => c::pass(&line),
                    CheckStatus::Warn => c::warn(&line),
                    CheckStatus::Fail => c::fail(&line),
                    CheckStatus::Skip => c::skip(&line),
                };
                out.push_str(&format!("  {}\n", formatted));
            }

            if include_history {
                out.push_str(&format!("\n{}:\n", c::label("Work History")));
                out.push_str(&format!("  {}\n", c::dim("(Work history not yet implemented)")));
            }

            out
        }
        ComplyOutputFormat::Json | ComplyOutputFormat::Sarif => {
            serde_json::to_string_pretty(&report)?
        }
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
        use crate::cli::colors as c;
        fs::write(output_path, &output_text)?;
        crate::status_println!("{}", c::pass(&format!("Compliance report written to {}", c::path(&output_path.display().to_string()))));
    } else {
        println!("{}", output_text);
    }

    Ok(())
}
