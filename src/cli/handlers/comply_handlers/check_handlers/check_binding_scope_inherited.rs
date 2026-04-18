// CB-1603 Inherited Clause Integrity + CB-1604 Postcondition Weakening —
// included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

/// Extract `preconditions:` entries for a specific equation from a source
/// YAML's `equations:` block. Returns `None` if the equation or its
/// `preconditions:` key is absent (nothing to inherit); `Some(vec![])` if the
/// list exists but is empty.
///
/// Recognized shape:
/// ```yaml
/// equations:
///   rope:
///     preconditions:
///     - "foo"
///     - "bar"
/// ```
fn yaml_equation_preconditions(content: &str, equation: &str) -> Option<Vec<String>> {
    let mut in_equations = false;
    let mut in_target = false;
    let mut in_preconditions = false;
    let mut preconditions: Option<Vec<String>> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Indent level 0: top-level key
        if !line.starts_with(' ') {
            in_equations = trimmed == "equations:";
            in_target = false;
            in_preconditions = false;
            continue;
        }
        if !in_equations {
            continue;
        }
        let indent = line.len() - line.trim_start_matches(' ').len();
        // Indent 2 = equation key under `equations:`
        if indent == 2 && !trimmed.starts_with('-') {
            if let Some(idx) = trimmed.find(':') {
                let name = trimmed[..idx].trim();
                in_target = name == equation;
                in_preconditions = false;
            }
            continue;
        }
        if !in_target {
            continue;
        }
        // Indent 4 = field under the target equation
        if indent == 4 && !trimmed.starts_with('-') {
            if let Some(idx) = trimmed.find(':') {
                let key = trimmed[..idx].trim();
                in_preconditions = key == "preconditions";
                if in_preconditions && preconditions.is_none() {
                    preconditions = Some(Vec::new());
                }
                continue;
            }
        }
        // Indent ≥4, starts with `-` and we're in preconditions: a list item
        if in_preconditions && indent >= 4 && trimmed.starts_with('-') {
            let item = trimmed[1..].trim();
            let unquoted = item
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim_start_matches('\'')
                .trim_end_matches('\'')
                .to_string();
            if !unquoted.is_empty() {
                if let Some(v) = preconditions.as_mut() {
                    v.push(unquoted);
                }
            }
            continue;
        }
        // Any other line under the target equation that isn't a sub-field
        // continuation ends the preconditions list.
        if in_preconditions && indent < 4 {
            in_preconditions = false;
        }
    }
    preconditions
}

/// CB-1603 (L3): verify each bound equation's YAML-declared `preconditions:`
/// are reflected in the ticket's `contract.require[]`. Catches inheritance
/// pipeline regressions where a tightening of the bound equation's precond
/// set isn't propagated to in-flight tickets.
///
/// Spec §Inheritance: a ticket inherits preconditions from each bound
/// equation. This check enforces that inheritance at the contract level —
/// equivalent to verifying `inherited-clauses.json` against source YAML,
/// without requiring the intermediate artifact.
pub(crate) fn check_binding_inherited_clauses(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1603: Inherited Clause Integrity";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut missing: Vec<String> = Vec::new();
    let mut checked_bindings = 0usize;
    let mut bindings_with_preconds = 0usize;

    for contract in &contracts {
        // Collect require-clause descriptions + ids for this ticket. A YAML
        // precondition string matches if it appears either as a clause id
        // (e.g. "require.compiles") or as the human-readable description.
        let ticket_require: std::collections::HashSet<&str> = contract
            .require
            .iter()
            .flat_map(|c| [c.id.as_str(), c.description.as_str()])
            .collect();

        for binding in &contract.implements {
            checked_bindings += 1;
            let yaml_path = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml) = std::fs::read_to_string(&yaml_path) else {
                continue;
            };
            let Some(preconds) = yaml_equation_preconditions(&yaml, &binding.equation) else {
                continue; // No preconditions declared for this equation
            };
            if preconds.is_empty() {
                continue;
            }
            bindings_with_preconds += 1;
            for p in &preconds {
                if !ticket_require.contains(p.as_str()) {
                    missing.push(format!(
                        "  {} [{}] missing inherited: {}",
                        contract.work_item_id,
                        binding.key(),
                        p
                    ));
                }
            }
        }
    }

    if bindings_with_preconds == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "{} binding(s) checked; none declare YAML preconditions to inherit",
                checked_bindings
            ),
            severity: Severity::Info,
        };
    }

    if missing.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} binding(s) with preconditions — all inherited into `require:`",
                bindings_with_preconds
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = format!("{} inherited precondition(s) missing:\n", missing.len());
    for line in &missing {
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

/// CB-1604 (L3): a ticket cannot override an inherited postcondition with a
/// weaker threshold. Reuses the DbC subcontracting validator
/// (`validate_subcontracting`) against each ticket's
/// `inherited_postconditions` → `ensure` relationship.
///
/// # Skip semantics (tiered)
///
/// * no `.pmat-work/*/contract.json` tickets        → Skip
/// * no ticket declares `inherited_postconditions`  → Skip (iteration 1
///                                                   tickets, no parent)
///
/// # Fail
///
/// * any clause weakened, dropped, or incompatible between the inherited
///   parent and the child's `ensure:` vector
pub(crate) fn check_binding_postcondition_weakening(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1604: Postcondition Weakening";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/*/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }

    let mut checked = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for c in &contracts {
        if c.inherited_postconditions.is_empty() {
            continue;
        }
        checked += 1;
        if let Err(v) = crate::cli::handlers::work_contract::validate_subcontracting(
            &c.inherited_postconditions,
            &c.ensure,
        ) {
            violations.push(format!("  {} — {}", c.work_item_id, v));
        }
    }

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket carries inherited postconditions (all iteration 1)".into(),
            severity: Severity::Info,
        };
    }

    if !violations.is_empty() {
        let mut msg = format!(
            "{} ticket(s) weaken inherited postconditions:\n",
            violations.len()
        );
        for line in &violations {
            msg.push_str(line);
            msg.push('\n');
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
        message: format!("{} ticket(s) preserve inherited postconditions", checked),
        severity: Severity::Info,
    }
}
