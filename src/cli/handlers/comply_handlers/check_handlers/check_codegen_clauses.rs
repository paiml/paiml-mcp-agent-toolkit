// CB-1634, CB-1638: Expr/binds_to linkage and generated-modules-tracked gates.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

// ─── CB-1634: Clauses with `expr` have `binds_to` ───────────────────────────

/// CB-1634 (L3): A clause with an `expr` field (codegen-ready Rust
/// expression) must also have a `binds_to` field (fully-qualified function
/// path). Without `binds_to`, the generator has no target to wrap — the
/// clause exists but doesn't apply to any code.
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
            }
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
    if orphaned.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "All clauses with `expr` also declare `binds_to`".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} clause(s) with `expr` missing `binds_to`: {}",
                orphaned.len(),
                orphaned.join(", ")
            ),
            severity: Severity::Error,
        }
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
