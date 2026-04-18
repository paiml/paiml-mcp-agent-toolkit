// CB-1641 and CB-1642 — Evidence method presence and path resolution.
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

// ─── CB-1641 — Every structured step has evidence_method ─────────────────────

pub(crate) fn check_step_has_evidence_method(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1641: Step Has Evidence Method";
    let contracts = load_contract_values(project_path);
    let mut missing: Vec<String> = Vec::new();
    let mut structured_seen = 0usize;
    for (ticket, contract) in &contracts {
        for step in cot_steps(contract) {
            if !is_structured(step) {
                continue;
            }
            structured_seen += 1;
            if step.get("evidence_method").is_none() {
                missing.push(format!("{}:{}", ticket, step_id(step)));
            }
        }
    }
    if structured_seen == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No structured CoT steps found — migration pending".into(),
            severity: Severity::Info,
        };
    }
    if missing.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} structured step(s) declare evidence_method",
                structured_seen
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} step(s) missing evidence_method: {}",
                missing.len(),
                missing.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1642 — ExistingTest evidence paths resolve on disk ───────────────────

pub(crate) fn check_existing_test_paths_resolve(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1642: Existing Test Path Resolves";
    let contracts = load_contract_values(project_path);
    let mut missing: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, contract) in &contracts {
        for step in cot_steps(contract) {
            let Some(method) = step.get("evidence_method") else {
                continue;
            };
            let Some(existing) = method.get("ExistingTest") else {
                continue;
            };
            checked += 1;
            let path_str = existing
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("<no path>");
            let abs = project_path.join(path_str);
            if !abs.exists() {
                missing.push(format!("{}:{} -> {}", ticket, step_id(step), path_str));
            }
        }
    }
    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `ExistingTest` evidence method references found".into(),
            severity: Severity::Info,
        };
    }
    if missing.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!("{} ExistingTest reference(s) resolve on disk", checked),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} ExistingTest reference(s) missing on disk: {}",
                missing.len(),
                missing.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}
