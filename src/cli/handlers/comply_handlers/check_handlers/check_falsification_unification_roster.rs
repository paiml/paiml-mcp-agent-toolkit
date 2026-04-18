// Work Falsification Unification — roster integrity checks.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.
//
// Contains:
//   CB-1620 — check_inherited_roster_coverage (L1): every binding has
//             ProvableContract entries matching YAML falsification_tests[].
//   CB-1623 — check_no_duplicate_provable_entries (L3): no duplicate
//             (yaml_path, test_id) across a ticket's roster.
//   CB-1626 — check_test_id_exists_in_yaml (L1): referenced test_ids still
//             exist in the current YAML source.

// ─── CB-1620: Inherited roster coverage (WARN during migration) ──────────────

/// CB-1620 (L1): For every `ContractBinding`, the ticket's claim roster must
/// contain a matching `ProvableContract{}` entry for each test in the bound
/// YAML's `falsification_tests[]`. Missing entries surface as **warnings**
/// during the 30-day migration window (§Migration) so users have time to run
/// `pmat work migrate --seed-inherited-falsification` before fail-mode.
pub(crate) fn check_inherited_roster_coverage(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1620: Inherited Roster Coverage";
    let contracts = load_active_contracts(project_path);
    let any_binding = contracts.iter().any(|c| !c.implements.is_empty());
    if !any_binding {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has `implements:` bindings to enforce".into(),
            severity: Severity::Info,
        };
    }
    let mut gaps: Vec<String> = Vec::new();
    for c in &contracts {
        for binding in &c.implements {
            let file = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml) = std::fs::read_to_string(&file) else {
                continue;
            };
            let expected_ids = yaml_falsification_test_ids(&yaml);
            if expected_ids.is_empty() {
                continue;
            }
            let entries = provable_contract_entries(c);
            for id in &expected_ids {
                let present = entries.iter().any(|(path, eq, test_id)| {
                    path == &binding.file && eq == &binding.equation && test_id == id
                });
                if !present {
                    gaps.push(format!(
                        "{} → {}/{}#{}",
                        c.work_item_id, binding.contract, binding.equation, id
                    ));
                }
            }
        }
    }
    if gaps.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "All bound tickets have inherited ProvableContract entries".into(),
            severity: Severity::Info,
        }
    } else {
        let preview: Vec<String> = gaps.iter().take(5).cloned().collect();
        let more = gaps.len().saturating_sub(5);
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} inherited test(s) missing from roster (e.g. {}{}). Run `pmat work migrate --seed-inherited-falsification`.",
                gaps.len(),
                preview.join(", "),
                if more > 0 { format!(", +{} more", more) } else { String::new() }
            ),
            severity: Severity::Warning,
        }
    }
}

// ─── CB-1623: No duplicate (yaml_path, test_id) across a ticket's roster ─────

/// CB-1623 (L3): `FalsificationMethod::ProvableContract` entries with the
/// same `(yaml_path, test_id)` pair count as double-coverage and inflate
/// passing claim counts. The check walks every ticket's claim roster and
/// fails on the first duplicate found.
pub(crate) fn check_no_duplicate_provable_entries(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1623: No Duplicate ProvableContract Entries";
    let contracts = load_active_contracts(project_path);
    if !has_any_provable_contract_entries(&contracts) {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ProvableContract entries in any ticket yet".into(),
            severity: Severity::Info,
        };
    }
    let mut dupes: Vec<String> = Vec::new();
    for c in &contracts {
        let entries = provable_contract_entries(c);
        let mut seen: Vec<(PathBuf, String)> = Vec::new();
        for (path, _eq, test_id) in entries {
            let key = (path.clone(), test_id.clone());
            if seen.contains(&key) {
                dupes.push(format!(
                    "{} → {}#{}",
                    c.work_item_id,
                    path.display(),
                    test_id
                ));
            } else {
                seen.push(key);
            }
        }
    }
    if dupes.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "No duplicate (yaml_path, test_id) pairs in any ticket roster".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Duplicate ProvableContract entries: {}", dupes.join("; ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1626: Referenced test_id exists in YAML ──────────────────────────────

/// CB-1626 (L1): Each `ProvableContract.test_id` must exist in the bound
/// YAML's `falsification_tests[]` at scan time. A stale reference means the
/// YAML removed the test post-bind and the ticket's claim is unanchored.
pub(crate) fn check_test_id_exists_in_yaml(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1626: Referenced test_id Exists in YAML";
    let contracts = load_active_contracts(project_path);
    if !has_any_provable_contract_entries(&contracts) {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ProvableContract entries in any ticket yet".into(),
            severity: Severity::Info,
        };
    }
    let mut stale: Vec<String> = Vec::new();
    for c in &contracts {
        for (yaml_path, _eq, test_id) in provable_contract_entries(c) {
            let file = if yaml_path.is_absolute() {
                yaml_path.clone()
            } else {
                project_path.join(&yaml_path)
            };
            let Ok(yaml) = std::fs::read_to_string(&file) else {
                stale.push(format!(
                    "{} → {} (YAML missing)",
                    c.work_item_id,
                    yaml_path.display()
                ));
                continue;
            };
            let ids = yaml_falsification_test_ids(&yaml);
            if !ids.iter().any(|i| i == &test_id) {
                stale.push(format!(
                    "{} → {}#{}",
                    c.work_item_id,
                    yaml_path.display(),
                    test_id
                ));
            }
        }
    }
    if stale.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "All referenced test_ids exist in their YAML sources".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Stale test_id references: {}", stale.join("; ")),
            severity: Severity::Error,
        }
    }
}
