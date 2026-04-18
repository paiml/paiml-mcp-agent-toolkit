// Work Falsification Unification — CB-1627 post-bind YAML drift.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.
//
// Contains:
//   CB-1627 — check_post_bind_yaml_drift (L3): warn when a bound YAML's
//             `falsification_tests[]` has grown since bind time.
//   is_inherited_receipt — shared helper used by CB-1625 and CB-1628 to
//             decide whether a log line is inherited (has `yaml`/`test_id`).

/// CB-1627 (L3): warn when the bound YAML's `falsification_tests[]` has
/// gained entries since bind time. The bind-time snapshot is implicit:
/// every `ProvableContract{ yaml_path, equation, test_id }` entry in the
/// ticket's claims roster was seeded *at bind time* from the YAML that
/// existed then. Any test_id currently in the YAML that is NOT in that
/// seeded roster is a post-bind addition — the contract owner should
/// re-bind (or explicitly opt out) to pick up the new coverage.
///
/// Result is always WARN — per spec §CB-1627 this is a drift signal, not
/// a hard failure. The ticket still completes; the warning surfaces the
/// missed coverage for the next planning cycle.
///
/// Skip semantics (tiered):
///   • no `.pmat-work/` directory                → Skip
///   • no ticket has `implements:` bindings      → Skip
///   • every binding's YAML is missing or has
///     no `falsification_tests[]`                → Skip
pub(crate) fn check_post_bind_yaml_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1627: Post-bind YAML Drift";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory".into(),
            severity: Severity::Info,
        };
    }

    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/*/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }

    let any_binding = contracts.iter().any(|c| !c.implements.is_empty());
    if !any_binding {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has `implements:` bindings".into(),
            severity: Severity::Info,
        };
    }

    let mut checked_bindings = 0usize;
    let mut drift: Vec<String> = Vec::new();

    for c in &contracts {
        let seeded = provable_contract_entries(c);
        for binding in &c.implements {
            let yaml_path = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml_body) = std::fs::read_to_string(&yaml_path) else {
                continue;
            };
            let current_ids = yaml_falsification_test_ids(&yaml_body);
            if current_ids.is_empty() {
                continue;
            }
            checked_bindings += 1;

            let bound_ids: std::collections::HashSet<&str> = seeded
                .iter()
                .filter(|(path, eq, _)| path == &binding.file && eq == &binding.equation)
                .map(|(_, _, tid)| tid.as_str())
                .collect();

            for id in &current_ids {
                if !bound_ids.contains(id.as_str()) {
                    drift.push(format!(
                        "{} → {}/{}#{}",
                        c.work_item_id, binding.contract, binding.equation, id
                    ));
                }
            }
        }
    }

    if checked_bindings == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No binding's YAML declares `falsification_tests[]`".into(),
            severity: Severity::Info,
        };
    }

    if drift.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} binding(s) checked, no post-bind YAML additions",
                checked_bindings
            ),
            severity: Severity::Info,
        };
    }

    let preview: Vec<String> = drift.iter().take(5).cloned().collect();
    let more = drift.len().saturating_sub(5);
    let suffix = if more > 0 {
        format!(", +{} more", more)
    } else {
        String::new()
    };
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Warn,
        message: format!(
            "{} new `falsification_tests[]` entry/entries post-bind — rebind or opt-out: {}{}",
            drift.len(),
            preview.join(", "),
            suffix
        ),
        severity: Severity::Warning,
    }
}

/// Decide whether a log line is an inherited (ProvableContract) receipt.
/// Per spec §falsification.log, inherited lines carry a `yaml` key;
/// manual lines carry a `method` key instead. Only inherited lines are
/// subject to the 4-field shape requirement in CB-1628.
fn is_inherited_receipt(v: &serde_json::Value) -> bool {
    v.get("yaml").and_then(|y| y.as_str()).is_some()
        || v.get("test_id").and_then(|t| t.as_str()).is_some()
}
