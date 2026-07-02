// Work ladder declaration checks (CB-1610 parses, CB-1611 bounded-by-yaml).
// Included into `check_work_ladder.rs`; do not add file-level `use` or `#!`
// attributes here.

// ─── CB-1610: verification_level parses ──────────────────────────────────────

/// CB-1610 (L1): the stored `verification_level` string on every ticket must
/// parse strictly to a known ladder variant. Catches typos like `"L3 "`,
/// `"l4"`, or free-form strings like `"strong"` that silently downgrade
/// enforcement. Scans the RAW contract.json — since MACS-004 the typed field
/// migrates leniently on read, so only the raw file still shows the typo
/// (`pmat work migrate --levels` rewrites it).
pub(crate) fn check_ladder_parses(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1610: Verification Level Parses";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let mut bad: Vec<String> = Vec::new();
    for c in &contracts {
        let raw_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("contract.json");
        let Ok(text) = std::fs::read_to_string(&raw_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let Some(raw_level) = value.get("verification_level").and_then(|v| v.as_str()) else {
            continue; // absent = serde default (L3), nothing stored to lint
        };
        if VerificationLevel::parse_strict(raw_level).is_none() {
            bad.push(format!(
                "  {} -> verification_level='{}'",
                c.work_item_id, raw_level
            ));
        }
    }

    if !bad.is_empty() {
        let mut msg = format!("{} ticket(s) have unparseable level:\n", bad.len());
        for line in &bad {
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
        message: format!("All {} ticket level(s) parse to L0..L5", contracts.len()),
        severity: Severity::Info,
    }
}

// ─── CB-1611: target ≤ max attainable ─────────────────────────────────────────

/// CB-1611 (L1): `verification_level` claimed by a ticket cannot exceed the
/// max attainable level of its weakest binding. Catches tickets that claim
/// `L4` without any `kani_harnesses:` in the bound YAML.
pub(crate) fn check_ladder_bound_by_yaml(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1611: Target ≤ Max Attainable";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let mut over: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for c in &contracts {
        let claimed = c.verification_level; // typed since MACS-004; CB-1610 scans raw JSON
        if c.implements.is_empty() {
            // Unbound tickets are bounded to L1 per spec; warn but do not hard-fail here
            // (CB-1610 handles invalid strings; this check only concerns bound tickets).
            continue;
        }
        checked += 1;
        let ceiling = max_attainable_for_ticket(project_path, c);
        if claimed > ceiling {
            over.push(format!(
                "  {} claims {} but bindings cap at {}",
                c.work_item_id, claimed, ceiling
            ));
        }
    }

    if !over.is_empty() {
        let mut msg = format!("{} ticket(s) overclaim verification level:\n", over.len());
        for line in &over {
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

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has `implements:` bindings to bound against".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} bound ticket(s) within max-attainable ceiling", checked),
        severity: Severity::Info,
    }
}
