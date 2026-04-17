// Provable-contracts enforcement checks (CB-1201, CB-1203)
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1203: Contract-bound functions MUST have #[contract] or #[requires]/#[ensures] macros.
/// Cross-references contract YAML equation names against production source.
/// A production `pub fn <equation_name>` without a contract macro = FAIL.
/// Preferred: `#[contract("yaml-name", equation = "eq")]` — auto-injects from YAML.
/// Legacy: `#[requires(...)]` / `#[ensures(...)]` — hand-written assertions.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn check_annotation_coverage(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }
    // Support both flat (src/) and workspace (crates/*/src/) layouts
    let src_dir = project_path.join("src");
    let crates_dir = project_path.join("crates");
    if !src_dir.exists() && !crates_dir.exists() {
        return ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Skip,
            message: "No src/ or crates/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Collect equation names with preconditions/postconditions (Refs #273)
    let eq_names = collect_contract_equation_names(&contracts_dir);

    if eq_names.is_empty() {
        return ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Pass,
            message: "No contract equations found".into(),
            severity: Severity::Info,
        };
    }

    // For each equation name, find production pub fn and check for macros
    // Function-level check: macro must be in the 10 lines before pub fn
    let mut bound_fns = 0usize;
    let mut with_macro = 0usize;
    let mut missing = Vec::new();

    // Collect all source files — support both src/ and crates/*/src/ layouts
    let mut src_files: Vec<_> = Vec::new();
    let search_dirs: Vec<std::path::PathBuf> = if src_dir.exists() {
        vec![src_dir.clone()]
    } else {
        // Workspace: search all crates/*/src/
        std::fs::read_dir(&crates_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let s = e.path().join("src");
                s.exists().then_some(s)
            })
            .collect()
    };
    for sdir in &search_dirs {
        src_files.extend(
            walkdir::WalkDir::new(sdir)
                .into_iter()
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .filter(|e| {
                    let fname = e.file_name().to_string_lossy();
                    !fname.contains("test") && !fname.contains("contract_test")
                }),
        );
    }
    // Sort: blis/ and lib-level files first (kernel implementations)
    src_files.sort_by(|a, b| {
        let a_blis = a.path().to_string_lossy().contains("/blis/");
        let b_blis = b.path().to_string_lossy().contains("/blis/");
        b_blis.cmp(&a_blis)
    });

    // Also collect contract YAML stems for #[contract("stem", equation = "eq")] matching
    let mut yaml_stems: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(true, |e| e != "yaml") {
                continue;
            }
            if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    yaml_stems.insert(stem.to_string(), content);
                }
            }
        }
    }

    // Preload source lines that are #[contract] attributes (not string literals)
    // Matches both `#[contract(` and `#[provable_contracts_macros::contract(`
    let mut contract_attr_lines = Vec::new();
    for entry in &src_files {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with("#[contract(") || t.contains("::contract(") {
                    contract_attr_lines.push(t.to_string());
                }
            }
        }
    }

    for eq in &eq_names {
        // Strategy 1: Check if any #[contract] attribute references this equation
        let attr_pattern = format!("equation = \"{eq}\"");
        if contract_attr_lines
            .iter()
            .any(|line| line.contains(&attr_pattern))
        {
            bound_fns += 1;
            with_macro += 1;
            continue; // Covered by #[contract] macro — assertions come from YAML
        }

        // Strategy 2: Find pub fn <eq_name>( and check for macros in preceding lines.
        // GH-271: Window expanded from 10 → 25 lines to accommodate functions with
        // long doc comments. Doc-comment blocks that push the `#[contract(...)]`
        // beyond the window were the main source of CB-1203 false positives.
        let pattern = format!("pub fn {eq}(");
        let mut found = false;
        for entry in &src_files {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Some(pos) = content.find(&pattern) {
                    bound_fns += 1;
                    found = true;
                    let prefix = &content[..pos];
                    let preceding_lines: Vec<&str> = prefix.lines().rev().take(25).collect();
                    let has_macro = preceding_lines.iter().any(|line| {
                        let t = line.trim();
                        t.starts_with("#[contract(")
                            || t.contains("::contract(")
                            || t.starts_with("#[requires(")
                            || t.starts_with("#[ensures(")
                            || t.starts_with("#[invariant(")
                    });
                    // Also check function body for contract_pre_*/debug_assert! with contract comment
                    let body_start = &content[pos..];
                    let body_snippet: String = body_start.lines().take(20).collect::<Vec<_>>().join("\n");
                    let has_body_contract = body_snippet.contains("contract_pre_")
                        || body_snippet.contains("contract_post_")
                        || body_snippet.contains("// Contract:");
                    if has_macro || has_body_contract {
                        with_macro += 1;
                    } else {
                        let rel = entry
                            .path()
                            .strip_prefix(project_path)
                            .unwrap_or(entry.path());
                        missing.push(format!("{eq} in {}", rel.display()));
                    }
                    break;
                }
            }
        }
        // Equation has no matching pub fn — not a failure (might be test-only or delegated)
        if !found {
            // silently skip
        }
    }

    if bound_fns == 0 {
        return ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Pass,
            message: format!("{} equations, 0 production pub fns found", eq_names.len()),
            severity: Severity::Info,
        };
    }

    if !missing.is_empty() {
        ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Fail,
            message: format!(
                "{}/{} contract-bound fns missing #[contract(...)] annotation \
                 (add `#[provable_contracts_macros::contract(\"<yaml>\", equation = \"<name>\")]` \
                 within 25 lines above `pub fn`): {}",
                missing.len(),
                bound_fns,
                missing
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            severity: Severity::Error,
        }
    } else {
        ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Pass,
            message: format!("{with_macro}/{bound_fns} contract-bound fns have macros"),
            severity: Severity::Info,
        }
    }
}


/// CB-1201: PV Lint + contract fulfillment gate.
/// Checks: (1) pv lint passes, (2) referenced tests EXIST, (3) they PASS.
/// Missing test = unfalsifiable claim = FAIL (like TDG grade F).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_pv_lint(project_path: &Path, thresholds: &ComplyThresholds) -> ComplianceCheck {
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(dir) => dir,
        None => {
            return ComplianceCheck {
                name: "CB-1201: PV Lint".into(),
                status: CheckStatus::Skip,
                message: "No contracts/ directory found".into(),
                severity: Severity::Info,
            };
        }
    };

    // Step 1: Run pv lint on resolved contracts dir — avoids scanning work/ YAMLs
    let (pv_passed, pv_error_detail) = std::process::Command::new("pv")
        .args(["lint", &contracts_dir.display().to_string(), "--format", "json"])
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map(|o| {
            let json_val = String::from_utf8(o.stdout)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
            let passed = json_val
                .as_ref()
                .and_then(|v| v.get("passed")?.as_bool())
                .unwrap_or(false);
            // Extract first error finding for diagnostics
            let detail = json_val
                .as_ref()
                .and_then(|v| v.get("findings")?.as_array())
                .and_then(|arr| arr.iter().find(|f| {
                    f.get("severity").and_then(|s| s.as_str()) == Some("error")
                        || f.get("severity").and_then(|s| s.as_str()) == Some("ERROR")
                }))
                .and_then(|f| f.get("message").and_then(|m| m.as_str()))
                .map(|s| s.to_string())
                .or_else(|| {
                    // Fallback: first line of stderr
                    String::from_utf8(o.stderr).ok()
                        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
                        .filter(|s| !s.is_empty())
                });
            (passed, detail)
        })
        .unwrap_or((false, None));

    // Step 2: Check test fulfillment
    let (total_refs, existing, missing) = count_contract_test_refs(project_path);

    if total_refs > 0 && missing > 0 {
        return ComplianceCheck {
            name: "CB-1201: PV Lint".into(),
            status: CheckStatus::Fail,
            message: format!(
                "Unfalsifiable: {missing}/{total_refs} contract tests missing ({}% unfulfilled)",
                missing * 100 / total_refs
            ),
            severity: Severity::Error,
        };
    }

    if !pv_passed {
        let msg = match pv_error_detail {
            Some(detail) => format!("PV Lint failed: {detail}"),
            None => "PV Lint failed".into(),
        };
        let (status, severity) = if thresholds.pv_lint_is_error {
            (CheckStatus::Fail, Severity::Error)
        } else {
            (CheckStatus::Warn, Severity::Warning)
        };
        return ComplianceCheck {
            name: "CB-1201: PV Lint".into(),
            status,
            message: msg,
            severity,
        };
    }

    ComplianceCheck {
        name: "CB-1201: PV Lint".into(),
        status: CheckStatus::Pass,
        message: format!("PV Lint passed, {existing}/{total_refs} tests fulfilled"),
        severity: Severity::Info,
    }
}

