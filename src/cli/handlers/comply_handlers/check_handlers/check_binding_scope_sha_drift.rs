// CB-1601 Binding SHA Drift — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

// ─── CB-1601: SHA drift ──────────────────────────────────────────────────────

/// CB-1601 (L1): `ContractBinding::sha` must match SHA-256 of the YAML file
/// as it exists on disk today. A mismatch means the contract drifted after
/// the ticket was bound — either the YAML was edited (and the ticket
/// should re-bind), or the ticket is stale.
pub(crate) fn check_binding_sha_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1601: Binding SHA Drift";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut drifted: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, binding) in iter_bindings(&contracts) {
        checked += 1;
        let file = if binding.file.is_absolute() {
            binding.file.clone()
        } else {
            project_path.join(&binding.file)
        };
        let Ok(bytes) = std::fs::read(&file) else {
            missing.push(format!(
                "  {} [{}] -> {} (not found)",
                ticket,
                binding.key(),
                binding.file.display()
            ));
            continue;
        };
        let current = sha256_hex(&bytes);
        if current != binding.sha {
            drifted.push(format!(
                "  {} [{}] recorded {}… current {}…",
                ticket,
                binding.key(),
                &binding.sha[..binding.sha.len().min(8)],
                &current[..current.len().min(8)],
            ));
        }
    }

    if !drifted.is_empty() || !missing.is_empty() {
        let mut msg = format!("SHA drift in {}/{} binding(s)", drifted.len(), checked);
        if !missing.is_empty() {
            msg.push_str(&format!(" + {} missing YAML", missing.len()));
        }
        msg.push_str("\n");
        for line in drifted.iter().chain(missing.iter()) {
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
        message: format!("All {} binding SHA(s) match current YAML bytes", checked),
        severity: Severity::Info,
    }
}
