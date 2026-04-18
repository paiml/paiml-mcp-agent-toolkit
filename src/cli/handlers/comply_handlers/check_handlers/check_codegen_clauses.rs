// CB-1634, CB-1638: Expr/binds_to linkage and generated-modules-tracked gates.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

// ─── CB-1634: Clauses with `expr` have `binds_to` + attribute coverage ──────

/// CB-1634 (L3): A clause with an `expr` field (codegen-ready Rust
/// expression) must also have a `binds_to` field (fully-qualified function
/// path), AND the ticket's `binds_to` targets must be wrapped by at least
/// one `#[pmat_work_contract(id = "<TICKET>")]` attribute in `src/`.
///
/// Without `binds_to`, the generator has no target to wrap — the clause
/// exists but doesn't apply to any code. Without a matching attribute
/// somewhere in the tree, `binds_to` points at a function that isn't
/// wearing the wrapper, so the generated preconditions/postconditions are
/// never invoked at runtime.
pub(crate) fn check_expr_clauses_have_binds_to(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1634: expr Clauses Have binds_to";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory present".into(),
            severity: Severity::Info,
        };
    }
    let mut orphaned: Vec<String> = Vec::new();
    let mut bound_tickets: Vec<String> = Vec::new();
    let mut saw_expr = false;
    let Ok(entries) = std::fs::read_dir(&work_dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Unable to read `.pmat-work/`".into(),
            severity: Severity::Info,
        };
    };
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
        let mut ticket_has_expr_with_binds = false;
        for clause in iter_clauses(&contract) {
            let has_expr = clause.get("expr").is_some_and(|v| !v.is_null());
            if !has_expr {
                continue;
            }
            saw_expr = true;
            let has_binds = clause.get("binds_to").is_some_and(|v| !v.is_null());
            if !has_binds {
                let id = clause
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                orphaned.push(format!("{}#{}", ticket_id, id));
            } else {
                ticket_has_expr_with_binds = true;
            }
        }
        if ticket_has_expr_with_binds {
            bound_tickets.push(ticket_id);
        }
    }
    if !saw_expr {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No clause has an `expr` field yet".into(),
            severity: Severity::Info,
        };
    }
    if !orphaned.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} clause(s) with `expr` missing `binds_to`: {}",
                orphaned.len(),
                orphaned.join(", ")
            ),
            severity: Severity::Error,
        };
    }

    // Tightening: every ticket with `expr`+`binds_to` clauses must have ≥1
    // `#[pmat_work_contract(id = "<TICKET>")]` usage in `src/`. Otherwise
    // the binding intent is orphaned — the contract says "wrap this
    // function" but no function is wearing the attribute.
    let usages = collect_attribute_usages(project_path);
    let usage_ids: std::collections::HashSet<&str> =
        usages.iter().map(|u| u.id.as_str()).collect();
    let unbound: Vec<&String> = bound_tickets
        .iter()
        .filter(|t| !usage_ids.contains(t.as_str()))
        .collect();
    if !unbound.is_empty() {
        let names: Vec<String> = unbound.iter().map(|s| s.to_string()).collect();
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} ticket(s) with `expr`+`binds_to` clauses but no `#[pmat_work_contract(id = ...)]` usage in `src/`: {}",
                names.len(),
                names.join(", ")
            ),
            severity: Severity::Error,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "All clauses with `expr` declare `binds_to`; {} ticket(s) wrap ≥1 function with `#[pmat_work_contract]`",
            bound_tickets.len()
        ),
        severity: Severity::Info,
    }
}

// ─── CB-1638: Generated modules tracked in git ──────────────────────────────

/// CB-1638 (L3): Every `.rs` file under `contracts/work/` must be tracked
/// in git. An untracked file here means a developer ran `pmat work codegen`
/// without committing the output — next contributor's build will silently
/// regenerate.
pub(crate) fn check_generated_modules_tracked(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1638: Generated Modules Git-Tracked";
    let dir = project_path.join("contracts").join("work");
    if !dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `contracts/work/` directory present".into(),
            severity: Severity::Info,
        };
    }
    let mut untracked: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for entry in WalkDir::new(&dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        checked += 1;
        let out = std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch"])
            .arg(p)
            .current_dir(project_path)
            .output();
        match out {
            Ok(o) if o.status.success() => {}
            _ => untracked.push(p.display().to_string()),
        }
    }
    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.rs` files under `contracts/work/`".into(),
            severity: Severity::Info,
        };
    }
    if untracked.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!("All {} generated module(s) git-tracked", checked),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Untracked generated file(s): {}", untracked.join(", ")),
            severity: Severity::Error,
        }
    }
}
