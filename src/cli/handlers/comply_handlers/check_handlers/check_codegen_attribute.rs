// CB-1631, CB-1632: Attribute-scan checks for #[pmat_work_contract(...)] usages.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

// ─── CB-1631: Attribute references generated module ─────────────────────────

/// CB-1631 (L2): Every `#[pmat_work_contract(id = "PMAT-530")]` in the
/// codebase requires a `contracts/work/PMAT-530.rs` file to exist. A missing
/// file means the attribute references a closed/purged ticket or the user
/// forgot to run `pmat work codegen`.
pub(crate) fn check_attribute_has_generated_module(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1631: Attribute Has Generated Module";
    let usages = collect_attribute_usages(project_path);
    if usages.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `#[pmat_work_contract]` attribute usage found in `src/`".into(),
            severity: Severity::Info,
        };
    }
    let mut missing: Vec<String> = Vec::new();
    for usage in &usages {
        let generated = project_path
            .join("contracts")
            .join("work")
            .join(format!("{}.rs", usage.id));
        if !generated.exists() {
            missing.push(format!(
                "{}: attribute id=\"{}\" but {} is missing",
                usage.file.display(),
                usage.id,
                generated.display()
            ));
        }
    }
    if missing.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "All {} `#[pmat_work_contract]` usage(s) resolve to generated modules",
                usages.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Missing generated module(s): {}", missing.join("; ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1632: Attribute's require/ensure IDs match clauses ───────────────────

/// CB-1632 (L2): Every `require = "X"` and `ensure = "Y"` argument in a
/// `#[pmat_work_contract]` attribute must match a clause id in the referenced
/// ticket's `contract.json`. Typos here compile — the proc macro would fail
/// only at generation time — so this static check catches them early.
pub(crate) fn check_attribute_clause_ids_exist(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1632: Attribute Clause IDs Exist";
    let usages = collect_attribute_usages(project_path);
    if usages.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `#[pmat_work_contract]` attribute usage found in `src/`".into(),
            severity: Severity::Info,
        };
    }
    let mut mismatches: Vec<String> = Vec::new();
    for usage in &usages {
        let Some(contract) = load_contract_json(project_path, &usage.id) else {
            mismatches.push(format!(
                "{}: ticket `{}` has no `.pmat-work/{}/contract.json`",
                usage.file.display(),
                usage.id,
                usage.id
            ));
            continue;
        };
        let ids = clause_ids_from_json(&contract);
        for claim in usage.requires.iter().chain(usage.ensures.iter()) {
            if !ids.iter().any(|i| i == claim) {
                mismatches.push(format!(
                    "{}: attribute references `{}` on {} but no matching clause id",
                    usage.file.display(),
                    claim,
                    usage.id
                ));
            }
        }
    }
    if mismatches.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "All {} attribute clause id(s) resolve to ticket clauses",
                usages.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Clause id mismatches: {}", mismatches.join("; ")),
            severity: Severity::Error,
        }
    }
}
