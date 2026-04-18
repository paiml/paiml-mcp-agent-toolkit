// CB-1606 Lean Theorem Linkage — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

/// Scan a YAML for `lean_theorem:` block and return the status value if
/// present, else `None`. Recognizes nested form only — matching the shape
/// used by provable-contracts YAML files:
///
/// ```yaml
/// lean_theorem:
///   status: proved
///   name: rope_periodicity
/// ```
///
/// Quotes around the value are stripped. Case-insensitive comparison is
/// left to callers.
fn yaml_lean_theorem_status(yaml: &str) -> Option<String> {
    let mut in_lean = false;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            in_lean = trimmed.starts_with("lean_theorem:");
            continue;
        }
        if !in_lean {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("status:") {
            let value = rest
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// True if the loaded contract JSON references a BLOCK-ON-PROOF follow-up.
/// We deliberately match against the serialized JSON text — the schema
/// doesn't have a dedicated linked-tickets field yet, so any mention in
/// work_item_id, notes, oracle_context, chain_of_thought, clauses, or
/// references counts as a link. Case-insensitive.
fn contract_has_block_on_proof_link(project_path: &Path, ticket_id: &str) -> bool {
    let contract_path = project_path
        .join(".pmat-work")
        .join(ticket_id)
        .join("contract.json");
    let Ok(bytes) = std::fs::read(&contract_path) else {
        return false;
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return false;
    };
    text.to_ascii_uppercase().contains("BLOCK-ON-PROOF")
}

/// CB-1606 (L5): for every binding whose YAML declares a `lean_theorem:`
/// block with a status other than `proved`, the owning ticket must
/// reference a `BLOCK-ON-PROOF` follow-up (somewhere in the contract.json
/// free text — the schema doesn't yet have a dedicated linked-tickets
/// field).
///
/// Tiered skip semantics:
///   - no `implements:` entries                  → Skip
///   - no bound YAML declares `lean_theorem:`    → Skip
///   - all Lean theorems are `status: proved`    → Pass (no stranding risk)
///   - else                                      → Pass/Fail per BLOCK-ON-PROOF
pub(crate) fn check_binding_lean_theorem(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1606: Lean Theorem Linkage";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut bindings_with_lean = 0usize;
    let mut unproved_without_link: Vec<String> = Vec::new();
    let mut checked_unproved = 0usize;

    for contract in &contracts {
        for binding in &contract.implements {
            let file = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml) = std::fs::read_to_string(&file) else {
                continue;
            };
            let Some(status) = yaml_lean_theorem_status(&yaml) else {
                continue;
            };
            bindings_with_lean += 1;
            if status.eq_ignore_ascii_case("proved") {
                continue;
            }
            checked_unproved += 1;
            if !contract_has_block_on_proof_link(project_path, &contract.work_item_id) {
                unproved_without_link.push(format!(
                    "  {} [{}] lean_theorem.status='{}' — no BLOCK-ON-PROOF follow-up referenced",
                    contract.work_item_id,
                    binding.key(),
                    status
                ));
            }
        }
    }

    if bindings_with_lean == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No bound YAML declares `lean_theorem:`".into(),
            severity: Severity::Info,
        };
    }

    if checked_unproved == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} binding(s) have lean_theorem — all `status: proved`",
                bindings_with_lean
            ),
            severity: Severity::Info,
        };
    }

    if unproved_without_link.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} unproved lean_theorem binding(s) all reference BLOCK-ON-PROOF follow-up",
                checked_unproved
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = format!(
        "{} unproved lean_theorem binding(s) stranded without BLOCK-ON-PROOF follow-up:\n",
        unproved_without_link.len()
    );
    for line in &unproved_without_link {
        msg.push_str(line);
        msg.push('\n');
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}
