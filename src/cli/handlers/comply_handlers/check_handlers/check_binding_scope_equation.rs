// CB-1607 Binding Equation Identifier — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

// ─── CB-1607: Equation identifier exists in YAML ─────────────────────────────

/// Scan the YAML line-wise for `equations:` and collect the two-space-indented
/// identifiers below it. Accepts both block-style `rope:` (followed by a
/// nested body) and flow-style `rope: {}` on one line. Returns `None` if
/// no `equations:` section exists (e.g. alternate YAML shape).
fn yaml_equation_names(content: &str) -> Option<Vec<String>> {
    let mut in_equations = false;
    let mut saw_equations = false;
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Any non-indented, non-blank line ends the equations: section
        if !line.starts_with(' ') {
            in_equations = trimmed == "equations:";
            if in_equations {
                saw_equations = true;
            }
            continue;
        }
        if !in_equations {
            continue;
        }
        // Two-space indent exactly = a key under equations:
        if line.starts_with("  ") && !line.starts_with("    ") && !trimmed.starts_with('-') {
            if let Some(idx) = trimmed.find(':') {
                let name = trimmed[..idx].trim();
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }
    }
    if saw_equations {
        Some(names)
    } else {
        None
    }
}

/// CB-1607 (L3): the `equation` half of a `<contract>/<equation>` binding
/// must exist as a key under `equations:` in the referenced YAML. Fat-fingered
/// equation names would otherwise bind cleanly (file found, SHA recorded) but
/// point at nothing.
pub(crate) fn check_binding_equation_exists(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1607: Binding Equation Identifier";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut missing: Vec<String> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, binding) in iter_bindings(&contracts) {
        checked += 1;
        let file = if binding.file.is_absolute() {
            binding.file.clone()
        } else {
            project_path.join(&binding.file)
        };
        let Ok(content) = std::fs::read_to_string(&file) else {
            unreadable.push(format!("  {} [{}]", ticket, binding.key()));
            continue;
        };
        match yaml_equation_names(&content) {
            Some(names) if names.contains(&binding.equation) => {}
            Some(_) => missing.push(format!(
                "  {} [{}] equation '{}' not found in {}",
                ticket,
                binding.key(),
                binding.equation,
                binding.file.display()
            )),
            None => {
                // No equations: section — treat as skip-equivalent
            }
        }
    }

    if !missing.is_empty() || !unreadable.is_empty() {
        let mut msg = format!("{} equation(s) not found in referenced YAML", missing.len());
        if !unreadable.is_empty() {
            msg.push_str(&format!(" + {} unreadable", unreadable.len()));
        }
        msg.push('\n');
        for line in missing.iter().chain(unreadable.iter()) {
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
        message: format!("All {} equation(s) resolve in their YAML", checked),
        severity: Severity::Info,
    }
}
