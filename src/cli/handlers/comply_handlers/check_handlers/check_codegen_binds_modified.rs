// CB-1635: `binds_to` path points at a file modified by the ticket.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

pub(crate) fn check_binds_to_function_modified(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1635: binds_to Function Actually Modified";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory present".into(),
            severity: Severity::Info,
        };
    }
    let Ok(entries) = std::fs::read_dir(&work_dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Unable to read `.pmat-work/`".into(),
            severity: Severity::Info,
        };
    };

    let mut saw_any_binds = false;
    let mut evaluated_tickets = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(ticket_id) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" {
            continue;
        }
        let Some(contract) = load_contract_json(project_path, &ticket_id) else {
            continue;
        };

        let binds_to_paths: Vec<(String, String)> = iter_clauses(&contract)
            .filter_map(|c| {
                let bt = c.get("binds_to").and_then(|v| v.as_str())?;
                let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("<unknown>");
                Some((id.to_string(), bt.to_string()))
            })
            .collect();
        if binds_to_paths.is_empty() {
            continue;
        }
        saw_any_binds = true;

        let Some(modified) = load_modified_files(project_path, &ticket_id) else {
            continue;
        };
        evaluated_tickets += 1;

        for (clause_id, bt) in &binds_to_paths {
            let candidates = resolve_binds_to_candidates(bt);
            let hit = candidates.iter().any(|c| modified.iter().any(|m| m == c));
            if !hit {
                violations.push(format!(
                    "  {}#{} → {} (tried: {})",
                    ticket_id,
                    clause_id,
                    bt,
                    candidates.join(", ")
                ));
            }
        }
    }

    if !saw_any_binds {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has a clause with `binds_to`".into(),
            severity: Severity::Info,
        };
    }
    if evaluated_tickets == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message:
                "No `.pmat-work/<ID>/modified-files.json` — work CLI has not emitted diff receipts yet"
                    .into(),
            severity: Severity::Info,
        };
    }

    if !violations.is_empty() {
        let mut msg = format!(
            "{} `binds_to` clause(s) target files the ticket did not modify:\n",
            violations.len()
        );
        let preview: Vec<&String> = violations.iter().take(5).collect();
        for line in preview {
            msg.push_str(line);
            msg.push('\n');
        }
        if violations.len() > 5 {
            msg.push_str(&format!("  …and {} more\n", violations.len() - 5));
        }
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "{} ticket(s): every `binds_to` target appears in modified files",
            evaluated_tickets
        ),
        severity: Severity::Info,
    }
}
