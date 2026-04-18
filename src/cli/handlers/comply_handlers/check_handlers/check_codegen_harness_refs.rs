// CB-1639: every declared Kani harness body references the generated contract
// macros for its ticket.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1639 (L4+): every `kani_harnesses[]` entry in an L4+ ticket's bound
/// YAML must be backed by a harness function body that references the
/// generated contract macros. An "orphaned" harness — a `#[kani::proof]`
/// function that doesn't invoke any generated `contracts::work::<ID>`
/// macros — proves nothing about the ticket's contract and is the exact
/// failure mode this gate catches.
///
/// Detection is file-level: the file declaring `fn <harness_name>` must
/// also contain one of the expected macro references (module path or
/// attribute). Function-level precision lands once Component 30 codegen
/// settles on stable macro naming.
///
/// # Accepted references
///
/// * `contracts::work::<ticket_id>` (module path, ticket id verbatim)
/// * `contracts::work::<TICKET_ID_with_underscores>` (Rust module naming)
/// * `#[pmat_work_contract(id = "<ticket_id>")]` (attribute form)
///
/// # Skip semantics (tiered)
///
/// * `.pmat-work/` absent                             → Skip
/// * no `.pmat-work/<ID>/contract.json` tickets       → Skip
/// * no ticket targets L4 or higher                   → Skip
/// * no L4+ ticket has `implements:` bindings         → Skip
/// * no L4+ bound YAML declares `kani_harnesses:`     → Skip
/// * no declared harness has a `fn <name>` body in
///   `kani/`, `tests/`, `harnesses/`, or `src/`       → Skip (Kani
///                                                     integration pending)
///
/// # Pass
///
/// Every found harness body lives in a file that references the ticket's
/// generated contract macros.
///
/// # Fail
///
/// Any found harness body lives in a file that does NOT reference the
/// ticket's generated contract macros (the orphan case).
pub(crate) fn check_kani_harness_macro_reference(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1639: Kani Harnesses Reference Generated Macros";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory present".into(),
            severity: Severity::Info,
        };
    }
    let Ok(entries) = std::fs::read_dir(&work_dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Unable to read `.pmat-work/`".into(),
            severity: Severity::Info,
        };
    };

    let mut saw_any_ticket = false;
    let mut saw_any_l4 = false;
    let mut saw_any_binding = false;
    let mut saw_any_harness = false;
    let mut evaluated_bodies = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(ticket_id) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" || ticket_id == "codegen" {
            continue;
        }
        let Some(contract) = load_contract_json(project_path, &ticket_id) else {
            continue;
        };
        saw_any_ticket = true;
        if !contract_level_at_least(&contract, 4) {
            continue;
        }
        saw_any_l4 = true;

        let Some(implements) = contract.get("implements").and_then(|v| v.as_array()) else {
            continue;
        };
        if implements.is_empty() {
            continue;
        }
        saw_any_binding = true;

        for binding in implements {
            let Some(yaml_rel) = binding.get("file").and_then(|v| v.as_str()) else {
                continue;
            };
            let yaml_path = if Path::new(yaml_rel).is_absolute() {
                PathBuf::from(yaml_rel)
            } else {
                project_path.join(yaml_rel)
            };
            let Ok(yaml_text) = std::fs::read_to_string(&yaml_path) else {
                continue;
            };
            let harnesses = yaml_kani_harness_names(&yaml_text);
            if harnesses.is_empty() {
                continue;
            }
            saw_any_harness = true;

            for h in &harnesses {
                let Some(body_file) = find_file_declaring_harness(project_path, h) else {
                    continue;
                };
                evaluated_bodies += 1;
                let Ok(body_text) = std::fs::read_to_string(&body_file) else {
                    continue;
                };
                if !file_references_generated_macros(&body_text, &ticket_id) {
                    let rel = body_file
                        .strip_prefix(project_path)
                        .unwrap_or(&body_file)
                        .display();
                    violations.push(format!(
                        "  {} harness `{}` in {} — no reference to `contracts::work::{}` or `#[pmat_work_contract(id = \"{}\")]`",
                        ticket_id, h, rel, ticket_id, ticket_id
                    ));
                }
            }
        }
    }

    if !saw_any_ticket {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }
    if !saw_any_l4 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket targets L4 or higher".into(),
            severity: Severity::Info,
        };
    }
    if !saw_any_binding {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ ticket has `implements:` bindings".into(),
            severity: Severity::Info,
        };
    }
    if !saw_any_harness {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ bound YAML declares `kani_harnesses:`".into(),
            severity: Severity::Info,
        };
    }
    if evaluated_bodies == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message:
                "No declared harness has a `fn <name>` body in `kani/`, `tests/`, `harnesses/`, or `src/` — Kani integration pending"
                    .into(),
            severity: Severity::Info,
        };
    }

    if !violations.is_empty() {
        let mut msg = format!(
            "{} Kani harness(es) do not reference generated contract macros:\n",
            violations.len()
        );
        let preview: Vec<&String> = violations.iter().take(5).collect();
        for line in preview {
            msg.push_str(line);
            msg.push('\n');
        }
        if violations.len() > 5 {
            msg.push_str(&format!("  …and {} more\n", violations.len() - 5));
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
        message: format!(
            "{} Kani harness body(ies) reference generated contract macros",
            evaluated_bodies
        ),
        severity: Severity::Info,
    }
}

/// Scan a YAML's top-level `kani_harnesses:` block and return every harness
/// name, regardless of whether the entry carries a `sha:` sibling. Handles
/// both string-form entries (`- verify_foo`) and object-form entries
/// (`- name: verify_foo\n    sha: abc`).
///
/// Line-based parser tolerating the subset of YAML used in practice; full
/// YAML loading is Component 29's concern.
fn yaml_kani_harness_names(content: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('-') && !line.starts_with('\t') {
            in_section = trimmed == "kani_harnesses:";
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix('-') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            if let Some(rest) = item.strip_prefix("name:") {
                let val = rest.trim().trim_matches('"').trim_matches('\'');
                if !val.is_empty() {
                    names.push(val.to_string());
                }
                continue;
            }
            if item.contains(':') {
                continue;
            }
            let val = item.trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                names.push(val.to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("name:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                names.push(val.to_string());
            }
        }
    }
    names
}

/// Walk `kani/`, `tests/`, `harnesses/`, `src/` (in that order) and return
/// the first `.rs` file whose text contains `fn <harness_name>`. Returns
/// `None` when no candidate file is found — caller treats that as "Kani
/// integration pending".
fn find_file_declaring_harness(project_path: &Path, harness_name: &str) -> Option<PathBuf> {
    let pattern = format!("fn {}", harness_name);
    for root in ["kani", "tests", "harnesses", "src"] {
        let root_path = project_path.join(root);
        if !root_path.exists() {
            continue;
        }
        for entry in WalkDir::new(&root_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(path) else {
                continue;
            };
            if text.contains(&pattern) {
                return Some(path.to_path_buf());
            }
        }
    }
    None
}

/// Return true iff `text` references the generated contract macros for
/// `ticket_id`. Accepts the verbatim ticket id, the underscored form
/// (`PMAT-200` → `PMAT_200`), or the attribute form
/// `#[pmat_work_contract(id = "<ticket_id>")]`.
fn file_references_generated_macros(text: &str, ticket_id: &str) -> bool {
    let underscored = ticket_id.replace('-', "_");
    if text.contains(&format!("contracts::work::{}", ticket_id)) {
        return true;
    }
    if text.contains(&format!("contracts::work::{}", underscored)) {
        return true;
    }
    if text.contains(&format!("id = \"{}\"", ticket_id)) {
        return true;
    }
    false
}
