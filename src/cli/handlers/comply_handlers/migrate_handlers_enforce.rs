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
///
/// ONE RULE, ONE IMPLEMENTATION. This used to build a report of its own: five
/// hand-picked checks — of which only Version Currency can ever return Fail —
/// plus a literal `recommendations: vec![]`. The consequences were a matched
/// pair of defects. (1) `comply check` computes `recommendations` from the same
/// two inputs a few hundred lines away and returned 1 and 2 entries on inputs
/// where report returned 0; the empty vector here was the single leaf in this
/// command that no input could move. (2) Because the four config-presence
/// checks top out at Warn, report answered `is_compliant: true` on a directory
/// where `comply check` had, in the same second, reported a Fail — and its JSON
/// was byte-identical for an empty project and a 121-file corpus stuffed with
/// SATD, dead code and duplication, bar the timestamp.
///
/// Both commands now render `compute_compliance_report`, so a compliance
/// verdict is the same verdict whichever command asked for it.
async fn handle_report(
    project_path: &Path,
    include_history: bool,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    let mut report = compute_compliance_report(project_path, false)?;
    // ONE FLAG, EVERY FORMAT. `--include-history` used to be read inside
    // the Text arm alone — and there it only printed "(Work history not yet
    // implemented)". The DEFAULT format is markdown, and json/sarif never
    // mentioned the flag, so `comply report --include-history` was
    // byte-identical to `comply report` for every user who did not pass
    // `-f text`. The history is part of the report now, so all four formats
    // carry it.
    report.history = report_history(include_history, project_path);

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
            if let Some(note) = report.project_version_source.note() {
                out.push_str(&format!("  {}\n", c::dim(&format!("({note})"))));
            }
            out.push_str(&format!("{} {}\n", c::label("Current PMAT:"), report.current_version));
            out.push_str(&format!("{} {}\n", c::label("Versions Behind:"), report.versions_behind));
            let status_str = if report.is_compliant {
                format!("{}COMPLIANT{}", c::BOLD_GREEN, c::RESET)
            } else {
                format!("{}NON-COMPLIANT{}", c::BOLD_RED, c::RESET)
            };
            out.push_str(&format!("{} {}\n", c::label("Status:"), status_str));
            out.push_str(&format!(
                "{} {} total \u{b7} {} pass \u{b7} {} warn \u{b7} {} fail \u{b7} {} skip\n\n",
                c::label("Checks:"),
                report.summary.total,
                report.summary.pass,
                report.summary.warn,
                report.summary.fail,
                report.summary.skip
            ));

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

            // Computed by the shared builder, and therefore printed. They used
            // to be a hardcoded empty vec and were rendered nowhere.
            if !report.breaking_changes.is_empty() {
                out.push_str(&format!("\n{}:\n", c::label("Breaking Changes")));
                for bc in &report.breaking_changes {
                    out.push_str(&format!("  v{}: {}\n", bc.version, bc.description));
                }
            }
            if !report.recommendations.is_empty() {
                out.push_str(&format!("\n{}:\n", c::label("Recommendations")));
                for rec in &report.recommendations {
                    out.push_str(&format!("  \u{2022} {}\n", rec));
                }
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
            if let Some(note) = report.project_version_source.note() {
                out.push_str(&format!("| Project Version caveat | {} |\n", note));
            }
            out.push_str(&format!("| Current PMAT | {} |\n", report.current_version));
            out.push_str(&format!(
                "| Versions Behind | {} |\n",
                report.versions_behind
            ));
            out.push_str(&format!(
                "| Status | {} |\n",
                if report.is_compliant {
                    "\u{2705} COMPLIANT"
                } else {
                    "\u{274c} NON-COMPLIANT"
                }
            ));
            out.push_str(&format!(
                "| Checks | {} total, {} pass, {} warn, {} fail, {} skip |\n\n",
                report.summary.total,
                report.summary.pass,
                report.summary.warn,
                report.summary.fail,
                report.summary.skip
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

            if !report.breaking_changes.is_empty() {
                out.push_str("\n## Breaking Changes\n\n");
                for bc in &report.breaking_changes {
                    out.push_str(&format!("- **v{}**: {}\n", bc.version, bc.description));
                }
            }
            if !report.recommendations.is_empty() {
                out.push_str("\n## Recommendations\n\n");
                for rec in &report.recommendations {
                    out.push_str(&format!("- {}\n", rec));
                }
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

/// `comply report` and `comply check` are one rule with two renderings.
///
/// REGRESSION (gate-A: one fabricated leaf, one wholly-inert command).
/// `handle_report` used to build its own `ComplianceReport` from five
/// hand-picked checks and finish it with the literal `recommendations: vec![]`
/// — the single leaf in the command that no input could move — while
/// `comply check` computed that same schema field from the same pinned version
/// a few hundred lines away. On a project pinned to 2.0.0, check returned two
/// recommendations and report returned none. And because the four
/// config-presence checks in report's list top out at Warn, report answered
/// `is_compliant: true` on trees where check reported a Fail, and its JSON was
/// byte-identical for an empty project and a 121-file defect corpus.
#[cfg(test)]
mod report_shares_check_computation_tests {
    use super::*;

    /// A project `comply report` will actually answer about.
    ///
    /// A directory holding nothing but `.pmat/` is refused by the shared
    /// computation ("an unmeasured project is not a compliant one"), so the
    /// fixture carries a manifest and a source file.
    fn project(pinned: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn ok() -> u8 { 1 }\n").expect("lib");
        if let Some(version) = pinned {
            std::fs::create_dir_all(dir.path().join(".pmat")).expect("mkdir .pmat");
            std::fs::write(
                dir.path().join(".pmat/project.toml"),
                format!("[pmat]\nversion = \"{version}\"\nauto_update = false\n"),
            )
            .expect("pin");
        }
        dir
    }

    /// `comply report -f json`, read back from the file `--output` writes —
    /// the same document the command prints to stdout.
    async fn report_json(dir: &Path) -> serde_json::Value {
        let out = dir.join("__report.json");
        handle_report(dir, false, ComplyOutputFormat::Json, Some(&out))
            .await
            .expect("report must succeed on a real project");
        let body = fs::read_to_string(&out).expect("report file");
        fs::remove_file(&out).ok();
        serde_json::from_str(&body).expect("report must be valid JSON")
    }

    /// THE hardcoded `vec![]`. A project two majors behind has
    /// recommendations, and report must publish the ones check computes.
    #[tokio::test]
    async fn recommendations_are_computed_not_hardcoded_empty() {
        // A fresh directory per invocation: a comply run WRITES into the tree
        // it measures (`.pmat/project.toml`, `.pmat/context.idx`), so a second
        // run over the same fixture answers a different question.
        let dir = project(Some("2.0.0"));
        let twin = project(Some("2.0.0"));
        let json = report_json(dir.path()).await;
        let got: Vec<String> = json["recommendations"]
            .as_array()
            .expect("recommendations array")
            .iter()
            .map(|r| r.as_str().unwrap_or_default().to_string())
            .collect();
        assert!(
            !got.is_empty(),
            "report hardcoded `recommendations: vec![]`; check computes them from the same \
             pinned version. json = {json}"
        );
        assert!(
            got.iter().any(|r| r.contains("comply migrate")),
            "a project pinned to 2.0.0 is behind: {got:?}"
        );
        let shared =
            compute_compliance_report(twin.path(), false).expect("shared computation must succeed");
        assert_eq!(
            got, shared.recommendations,
            "report must publish the SAME recommendations the shared computation produced"
        );
    }

    /// Report used to answer COMPLIANT on trees where check reported a Fail,
    /// because its five checks could only ever reach Warn.
    #[tokio::test]
    async fn report_and_check_return_one_verdict_not_two() {
        // Two identical fresh fixtures — see the note above: comply writes into
        // the tree it measures, so the same directory twice is two questions.
        let dir = project(None);
        let twin = project(None);
        let json = report_json(dir.path()).await;
        let shared =
            compute_compliance_report(twin.path(), false).expect("shared computation must succeed");

        assert!(
            shared.checks.len() > 5,
            "the shared set is the full compliance roster, not report's five-check stub: {}",
            shared.checks.len()
        );
        assert_eq!(
            json["checks"].as_array().expect("checks").len(),
            shared.checks.len(),
            "report ran its own stub instead of the compliance check set"
        );
        assert_eq!(
            json["is_compliant"].as_bool().expect("is_compliant"),
            shared.is_compliant,
            "report and check must not contradict each other on the same directory"
        );
        assert_eq!(
            json["summary"]["total"].as_u64().expect("summary.total") as usize,
            shared.checks.len()
        );
    }

    /// GATE A: a clean project and a defect-bearing one must not produce the
    /// same numbers. Report's JSON used to be byte-identical across an empty
    /// corpus and a 121-file one, bar the timestamp.
    #[tokio::test]
    async fn report_numbers_respond_to_the_project() {
        let clean = project(None);
        let dirty = project(None);
        // `unsafe` with no SAFETY comment (CB-020) and an undocumented
        // `#[ignore]` (CB-123): both are in the compliance check set.
        fs::write(
            dirty.path().join("src/bad.rs"),
            "pub fn boom() -> u8 {\n    let p = &1u8 as *const u8;\n    unsafe { *p }\n}\n\
             #[test]\n#[ignore]\nfn t() {}\n",
        )
        .expect("write bad.rs");

        let a = compute_compliance_report(clean.path(), false).expect("clean");
        let b = compute_compliance_report(dirty.path(), false).expect("dirty");

        assert_ne!(
            (a.summary.pass, a.summary.warn, a.summary.fail),
            (b.summary.pass, b.summary.warn, b.summary.fail),
            "a defect-bearing project must not tally identically to a clean one: \
             clean={:?} dirty={:?}",
            a.summary,
            b.summary
        );
        assert_eq!(a.summary.total, b.summary.total, "same roster, same length");
        assert_eq!(
            a.summary.pass + a.summary.warn + a.summary.fail + a.summary.skip,
            a.summary.total,
            "the tally must account for every check"
        );
    }

    /// `project_version` is the PMAT version the project pins, NOT the
    /// project's own version — a corpus whose Cargo.toml says 0.1.0 was
    /// reported as `project_version: 3.30.0`. When nothing is pinned the
    /// number is pmat's own, and the report now says so.
    #[test]
    fn an_unpinned_project_is_labelled_as_unpinned() {
        let dir = project(None);
        let (config, source) =
            load_project_config_with_source(dir.path()).expect("load unpinned config");
        assert_eq!(
            source,
            VersionSource::InstalledPmatDefault,
            "no .pmat/project.toml existed, so the version reported is pmat's own"
        );
        assert_eq!(config.pmat.version, PMAT_VERSION);
        assert!(
            source
                .note()
                .is_some_and(|n| n.contains("installed pmat's own")),
            "the caveat must be sayable in the rendered report"
        );

        let pinned = project(Some("2.0.0"));
        let (config, source) =
            load_project_config_with_source(pinned.path()).expect("load pinned config");
        assert_eq!(source, VersionSource::PinnedByProject);
        assert_eq!(config.pmat.version, "2.0.0");
        assert!(source.note().is_none(), "a real pin needs no caveat");
    }

    /// The caveat has to reach the reader, not just the struct.
    #[tokio::test]
    async fn the_unpinned_caveat_appears_in_every_rendered_format() {
        // One fresh unpinned fixture per rendering: the first run writes
        // `.pmat/project.toml`, after which the project genuinely IS pinned.
        let json = report_json(project(None).path()).await;
        assert_eq!(
            json["project_version_source"],
            serde_json::json!("not pinned by this project - defaulted to the installed pmat's own version"),
            "the JSON must not present pmat's own version as an unqualified project verdict"
        );

        for (format, name) in [
            (ComplyOutputFormat::Markdown, "markdown"),
            (ComplyOutputFormat::Text, "text"),
        ] {
            let dir = project(None);
            let out = dir.path().join(format!("__r-{name}"));
            handle_report(dir.path(), false, format, Some(&out))
                .await
                .expect("report");
            let body = fs::read_to_string(&out).expect("rendered report");
            assert!(
                body.contains("installed pmat's own"),
                "{name} report hides the caveat: {}",
                &body[..body.len().min(600)]
            );
        }
    }

    /// The stub answered a directory containing nothing but `.pmat/`; the
    /// shared computation refuses it, exactly as `comply check` does.
    #[tokio::test]
    async fn report_refuses_a_directory_with_nothing_to_comply() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".pmat")).expect("mkdir .pmat");
        let err = handle_report(dir.path(), false, ComplyOutputFormat::Json, None)
            .await
            .expect_err("a tree with only .pmat/ in it is not a project");
        assert!(
            err.to_string().contains("No project found"),
            "unexpected error: {err}"
        );
    }
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
