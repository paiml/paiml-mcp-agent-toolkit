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

/// Debt tickets under `.pmat-tickets/`, newest filename last.
///
/// `pmat comply upgrade` writes one flat YAML per ticket there (`ticket_id`,
/// `category`, `created_at`, `status`). Only those four keys are read, and a key
/// the file does not carry comes back `None` rather than as a guess — the
/// listing is evidence from disk, not a reconstruction.
///
/// Returns an empty vec when the directory does not exist: "no tickets" is a
/// result, and the renderers say so.
fn collect_ticket_history(project_path: &Path) -> Vec<TicketHistoryEntry> {
    let dir = project_path.join(".pmat-tickets");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut tickets: Vec<TicketHistoryEntry> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .map(|path| {
            let name = path
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            let body = fs::read_to_string(&path).unwrap_or_default();
            TicketHistoryEntry {
                file: format!(".pmat-tickets/{name}"),
                ticket_id: ticket_yaml_value(&body, "ticket_id"),
                category: ticket_yaml_value(&body, "category"),
                status: ticket_yaml_value(&body, "status"),
                created_at: ticket_yaml_value(&body, "created_at"),
            }
        })
        .collect();

    // Filename order, so two runs over one directory produce one report.
    tickets.sort_by(|a, b| a.file.cmp(&b.file));
    tickets
}

/// Read one top-level scalar out of a debt ticket, or `None` if absent.
fn ticket_yaml_value(body: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    body.lines()
        .find(|line| line.starts_with(&prefix))
        .map(|line| {
            line[prefix.len()..]
                .trim()
                .trim_matches('"')
                .trim()
                .to_string()
        })
        .filter(|v| !v.is_empty())
}

/// The report's history section: `Some` exactly when the user asked for it.
///
/// Split out so the WIRING is testable on its own. The bug was never in the
/// listing — it was that `include_history` was consulted inside the `-f text`
/// arm and nowhere else, so the default markdown report and the json/sarif
/// report dropped it.
fn report_history(include_history: bool, project_path: &Path) -> Option<Vec<TicketHistoryEntry>> {
    include_history.then(|| collect_ticket_history(project_path))
}

/// One history line, in the words of the ticket file.
fn ticket_history_line(entry: &TicketHistoryEntry) -> String {
    let id = entry.ticket_id.as_deref().unwrap_or(&entry.file);
    let status = entry.status.as_deref().unwrap_or("status not recorded");
    let category = entry.category.as_deref().unwrap_or("category not recorded");
    let created = entry.created_at.as_deref().unwrap_or("created_at not recorded");
    format!("{id} [{status}] {category} — {created}")
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
        // ONE FLAG, EVERY FORMAT. `--include-history` used to be read inside
        // the Text arm alone — and there it only printed "(Work history not yet
        // implemented)". The DEFAULT format is markdown, and json/sarif never
        // mentioned the flag, so `comply report --include-history` was
        // byte-identical to `comply report` for every user who did not pass
        // `-f text`. The history is part of the report now, so all four formats
        // carry it.
        history: report_history(include_history, project_path),
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

            if let Some(history) = &report.history {
                out.push_str(&format!("\n{}:\n", c::label("Ticket History")));
                if history.is_empty() {
                    out.push_str(&format!("  {}\n", c::dim("no tickets in .pmat-tickets/")));
                } else {
                    for entry in history {
                        out.push_str(&format!("  {}\n", ticket_history_line(entry)));
                    }
                }
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

            if let Some(history) = &report.history {
                out.push_str("\n## Ticket History\n\n");
                if history.is_empty() {
                    out.push_str("_No tickets in `.pmat-tickets/`._\n");
                } else {
                    for entry in history {
                        out.push_str(&format!("- {}\n", ticket_history_line(entry)));
                    }
                }
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

#[cfg(test)]
mod include_history_tests {
    //! `--include-history` was read inside the `-f text` arm ALONE, and there it
    //! only printed "(Work history not yet implemented)". The default format is
    //! markdown, so `comply report --include-history` was byte-identical to
    //! `comply report` for every user who did not pass `-f text`; `-f json` was
    //! identical too.
    use super::*;

    const TICKET: &str = "# PMAT Legacy Debt Ticket\nticket_id: \"DEBT-001\"\ncategory: \"coverage\"\ncreated_at: \"2026-08-12T00:00:00Z\"\nstatus: \"open\"\n";

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".pmat-tickets")).expect("mkdir");
        std::fs::write(dir.path().join(".pmat-tickets/DEBT-001.yaml"), TICKET).expect("write");
        dir
    }

    #[test]
    fn tickets_on_disk_are_read_not_invented() {
        let dir = fixture();
        let history = collect_ticket_history(dir.path());
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].ticket_id.as_deref(), Some("DEBT-001"));
        assert_eq!(history[0].category.as_deref(), Some("coverage"));
        assert_eq!(history[0].status.as_deref(), Some("open"));
        assert_eq!(history[0].file, ".pmat-tickets/DEBT-001.yaml");
    }

    /// "No tickets" is a result, not an error and not silence.
    #[test]
    fn a_project_with_no_ticket_store_yields_an_empty_listing() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(collect_ticket_history(dir.path()).is_empty());
    }

    /// THE WIRING, not the listing: `--include-history` must reach the report
    /// itself, which every format renders from — it used to be read inside the
    /// `-f text` arm alone.
    #[test]
    fn the_flag_reaches_the_report_every_format_renders() {
        let dir = fixture();
        assert!(
            report_history(false, dir.path()).is_none(),
            "no flag, no section"
        );
        let history = report_history(true, dir.path()).expect("--include-history must be honoured");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].ticket_id.as_deref(), Some("DEBT-001"));
    }

    /// A key the file does not carry comes back `None` rather than a guess.
    #[test]
    fn a_missing_key_is_absent_rather_than_fabricated() {
        assert_eq!(
            ticket_yaml_value(TICKET, "ticket_id").as_deref(),
            Some("DEBT-001")
        );
        assert_eq!(ticket_yaml_value(TICKET, "resolution"), None);
        let line = ticket_history_line(&TicketHistoryEntry {
            file: ".pmat-tickets/x.yaml".to_string(),
            ticket_id: None,
            category: None,
            status: None,
            created_at: None,
        });
        assert!(line.contains("status not recorded"), "{line}");
        assert!(line.contains("category not recorded"), "{line}");
    }
}
