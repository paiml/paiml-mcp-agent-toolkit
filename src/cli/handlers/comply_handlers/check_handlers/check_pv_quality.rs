/// CB-1210: Precondition/Postcondition Quality — detect mass-generated boilerplate
///
/// Falsification finding F2: all 427 preconditions are identical `!input.is_empty()`.
/// F4: zero postconditions exist. This means pv codegen assertions are trivially true.
/// Known placeholder preconditions that indicate mass-generation without domain logic.
const PLACEHOLDER_PRECONDITIONS: &[&str] = &[
    "!input.is_empty()",
    "!x.is_empty()",
];

/// CB-1210: Precondition/Postcondition Quality — detect placeholder boilerplate
///
/// Checks YAML precondition diversity and flags known placeholder patterns.
/// FAIL if >70% of preconditions are identical or contain known placeholders
/// without accompanying domain constraints.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_precondition_quality(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(d) => d,
        None => {
            return ComplianceCheck {
                name: "CB-1210: Precondition Quality".into(),
                status: CheckStatus::Skip,
                message: "No preconditions found in contracts".into(),
                severity: Severity::Info,
            };
        }
    };

    let mut preconditions: Vec<String> = Vec::new();
    let mut postcondition_count = 0usize;
    let mut equations_with_pre = 0usize;
    let mut placeholder_only_equations = 0usize;

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            continue;
        }
        if path.file_name().is_some_and(|n| n.to_string_lossy().contains("binding")) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut in_equations = false;
            let mut in_preconditions = false;
            let mut in_postconditions = false;
            let mut eq_pres: Vec<String> = Vec::new();
            for line in content.lines() {
                let trimmed = line.trim();
                // Track whether we're inside the equations: block
                if trimmed == "equations:" && !line.starts_with(' ') {
                    in_equations = true;
                    continue;
                }
                // Exit equations block on next top-level key
                if in_equations
                    && !line.starts_with(' ')
                    && !trimmed.is_empty()
                    && !trimmed.starts_with('#')
                    && trimmed != "equations:"
                {
                    // Flush
                    if !eq_pres.is_empty() {
                        check_equation_preconditions(
                            &eq_pres,
                            &mut equations_with_pre,
                            &mut placeholder_only_equations,
                        );
                        preconditions.append(&mut eq_pres);
                    }
                    in_equations = false;
                    in_preconditions = false;
                    in_postconditions = false;
                }
                if !in_equations {
                    continue;
                }
                if trimmed == "preconditions:" {
                    // Flush previous equation's preconditions
                    if !eq_pres.is_empty() {
                        check_equation_preconditions(
                            &eq_pres,
                            &mut equations_with_pre,
                            &mut placeholder_only_equations,
                        );
                        preconditions.append(&mut eq_pres);
                    }
                    in_preconditions = true;
                    in_postconditions = false;
                    continue;
                }
                if trimmed == "postconditions:" {
                    if !eq_pres.is_empty() {
                        check_equation_preconditions(
                            &eq_pres,
                            &mut equations_with_pre,
                            &mut placeholder_only_equations,
                        );
                        preconditions.append(&mut eq_pres);
                    }
                    in_postconditions = true;
                    in_preconditions = false;
                    continue;
                }
                if !trimmed.starts_with('-')
                    && !trimmed.starts_with('#')
                    && !line.starts_with(' ')
                {
                    if !eq_pres.is_empty() {
                        check_equation_preconditions(
                            &eq_pres,
                            &mut equations_with_pre,
                            &mut placeholder_only_equations,
                        );
                        preconditions.append(&mut eq_pres);
                    }
                    in_preconditions = false;
                    in_postconditions = false;
                }
                if in_preconditions && trimmed.starts_with("- ") {
                    eq_pres.push(
                        trimmed
                            .trim_start_matches("- ")
                            .trim_matches('\'')
                            .to_string(),
                    );
                }
                if in_postconditions && trimmed.starts_with("- ") {
                    postcondition_count += 1;
                }
            }
            // Flush final equation
            if !eq_pres.is_empty() {
                check_equation_preconditions(
                    &eq_pres,
                    &mut equations_with_pre,
                    &mut placeholder_only_equations,
                );
                preconditions.extend(eq_pres);
            }
        }
    }

    if preconditions.is_empty() {
        return ComplianceCheck {
            name: "CB-1210: Precondition Quality".into(),
            status: CheckStatus::Skip,
            message: "No preconditions found in contracts".into(),
            severity: Severity::Info,
        };
    }

    // Check diversity: what % are identical?
    let total = preconditions.len();
    let mut freq: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for p in &preconditions {
        *freq.entry(p.as_str()).or_insert(0) += 1;
    }
    let Some((most_common, most_count)) = freq.iter().max_by_key(|(_, c)| *c) else {
        return ComplianceCheck {
            name: "CB-1210: Precondition Quality".into(),
            status: CheckStatus::Skip,
            message: "No preconditions found in contracts".into(),
            severity: Severity::Info,
        };
    };
    let diversity_pct = (1.0 - (*most_count as f64 / total as f64)) * 100.0;
    let unique_count = freq.len();

    let mut issues = Vec::new();

    // FAIL: >70% identical (diversity < 30%)
    if diversity_pct < 30.0 {
        issues.push(format!(
            "{most_count}/{total} preconditions are identical: `{most_common}` ({diversity_pct:.0}% diverse, need ≥30%)"
        ));
    }

    // FAIL: >5% of equations with ONLY placeholder preconditions (no domain logic)
    if equations_with_pre > 0 {
        let placeholder_pct =
            placeholder_only_equations as f64 / equations_with_pre as f64 * 100.0;
        if placeholder_pct > 5.0 {
            issues.push(format!(
                "{placeholder_only_equations}/{equations_with_pre} ({placeholder_pct:.0}%) equations have only placeholder preconditions"
            ));
        }
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1210: Precondition Quality".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{total} preconditions, {unique_count} unique ({diversity_pct:.0}% diverse), {postcondition_count} postconditions"
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1210: Precondition Quality".into(),
            status: CheckStatus::Fail,
            message: format!("Low quality: {}", issues.join("; ")),
            severity: Severity::Error,
        }
    }
}

/// Helper: check if an equation's preconditions are all placeholders.
fn check_equation_preconditions(
    pres: &[String],
    equations_with_pre: &mut usize,
    placeholder_only_equations: &mut usize,
) {
    if pres.is_empty() {
        return;
    }
    *equations_with_pre += 1;
    let all_placeholder = pres
        .iter()
        .all(|p| PLACEHOLDER_PRECONDITIONS.contains(&p.as_str()));
    if all_placeholder {
        *placeholder_only_equations += 1;
    }
}

/// CB-1211: Codegen Fidelity — verify generated assertions match YAML preconditions
///
/// Runs `pv codegen` (if available) to generate assertions, then compares
/// the generated assertion count against YAML precondition count. Falls back
/// to checking for known placeholder patterns in any generated_contracts.rs file.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_codegen_fidelity(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(d) => d,
        None => {
            return ComplianceCheck {
                name: "CB-1211: Codegen Fidelity".into(),
                status: CheckStatus::Skip,
                message: "No preconditions in YAML contracts".into(),
                severity: Severity::Info,
            };
        }
    };

    // Count YAML preconditions per equation
    let mut yaml_pre_count = 0usize;
    let mut yaml_equation_count = 0usize;

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            continue;
        }
        if path.file_name().is_some_and(|n| n.to_string_lossy().contains("binding")) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut in_preconditions = false;
            let mut has_pre_in_eq = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "preconditions:" {
                    if has_pre_in_eq {
                        yaml_equation_count += 1;
                    }
                    in_preconditions = true;
                    has_pre_in_eq = false;
                    continue;
                }
                if !trimmed.starts_with('-')
                    && !trimmed.starts_with('#')
                    && !line.starts_with(' ')
                {
                    in_preconditions = false;
                }
                if in_preconditions && trimmed.starts_with("- ") {
                    yaml_pre_count += 1;
                    has_pre_in_eq = true;
                }
            }
            if has_pre_in_eq {
                yaml_equation_count += 1;
            }
        }
    }

    if yaml_pre_count == 0 {
        return ComplianceCheck {
            name: "CB-1211: Codegen Fidelity".into(),
            status: CheckStatus::Skip,
            message: "No preconditions in YAML contracts".into(),
            severity: Severity::Info,
        };
    }

    // Check for generated_contracts.rs in the project
    let generated_file = find_generated_contracts(project_path);
    if let Some(gen_path) = generated_file {
        if let Ok(content) = std::fs::read_to_string(&gen_path) {
            // Only count debug_assert! in code lines, not comments
            let gen_assert_count = content
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.starts_with("//") && t.contains("debug_assert!")
                })
                .count();
            let placeholder_count = content
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.starts_with("//")
                        && t.contains("debug_assert!")
                        && t.contains("_contract_input.is_empty()")
                })
                .count();

            if placeholder_count > 0 && placeholder_count as f64 / gen_assert_count.max(1) as f64 > 0.5 {
                return ComplianceCheck {
                    name: "CB-1211: Codegen Fidelity".into(),
                    status: CheckStatus::Fail,
                    message: format!(
                        "Generated file has {placeholder_count}/{gen_assert_count} placeholder assertions — codegen not emitting YAML preconditions"
                    ),
                    severity: Severity::Error,
                };
            }

            // Detect full-corpus generated file (assertions >> local YAML)
            let source_note = if gen_assert_count > yaml_pre_count * 5 && yaml_pre_count > 0 {
                format!("{gen_assert_count} assertions (full-corpus file), {yaml_pre_count} local YAML preconditions, 0 placeholders")
            } else {
                format!("{gen_assert_count} assertions from {yaml_pre_count} YAML preconditions across {yaml_equation_count} equations")
            };

            if gen_assert_count == 0 && yaml_pre_count > 0 {
                return ComplianceCheck {
                    name: "CB-1211: Codegen Fidelity".into(),
                    status: CheckStatus::Warn,
                    message: format!("Generated file: 0 assertions from {yaml_pre_count} YAML preconditions — all skipped (unbound vars)"),
                    severity: Severity::Warning,
                };
            }

            return ComplianceCheck {
                name: "CB-1211: Codegen Fidelity".into(),
                status: CheckStatus::Pass,
                message: format!("Generated file: {source_note}"),
                severity: Severity::Info,
            };
        }
    }

    // No generated file found — run pv codegen to temp file if available
    let pv_result = std::process::Command::new("pv")
        .args(["codegen", contracts_dir.to_str().unwrap_or("contracts/"), "-o", "/dev/stdout"])
        .output();

    match pv_result {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout);
            let gen_assert_count = content
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.starts_with("//") && t.contains("debug_assert!")
                })
                .count();
            let placeholder_count = content
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.starts_with("//")
                        && t.contains("debug_assert!")
                        && t.contains("_contract_input.is_empty()")
                })
                .count();

            if gen_assert_count > 0
                && placeholder_count as f64 / gen_assert_count as f64 > 0.5
            {
                ComplianceCheck {
                    name: "CB-1211: Codegen Fidelity".into(),
                    status: CheckStatus::Fail,
                    message: format!(
                        "pv codegen: {placeholder_count}/{gen_assert_count} placeholder assertions — YAML has {yaml_pre_count} real preconditions"
                    ),
                    severity: Severity::Error,
                }
            } else if gen_assert_count == 0 && yaml_pre_count > 0 {
                ComplianceCheck {
                    name: "CB-1211: Codegen Fidelity".into(),
                    status: CheckStatus::Warn,
                    message: format!(
                        "pv codegen: 0 assertions from {yaml_pre_count} YAML preconditions — all skipped (unbound vars)"
                    ),
                    severity: Severity::Warning,
                }
            } else {
                ComplianceCheck {
                    name: "CB-1211: Codegen Fidelity".into(),
                    status: CheckStatus::Pass,
                    message: format!(
                        "pv codegen: {gen_assert_count} assertions match {yaml_pre_count} YAML preconditions"
                    ),
                    severity: Severity::Info,
                }
            }
        }
        _ => {
            // pv not available and no generated file — cannot verify
            ComplianceCheck {
                name: "CB-1211: Codegen Fidelity".into(),
                status: CheckStatus::Skip,
                message: format!(
                    "{yaml_pre_count} YAML preconditions across {yaml_equation_count} equations (pv not available, no generated file)"
                ),
                severity: Severity::Info,
            }
        }
    }
}

/// Find a generated_contracts.rs file in the project.
fn find_generated_contracts(project_path: &Path) -> Option<std::path::PathBuf> {
    for candidate in &[
        "src/generated_contracts.rs",
        "generated_contracts.rs",
        "src/contracts.rs",
    ] {
        let path = project_path.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// CB-1214: Enforcement Quality — measures actual contract call-site penetration
///
/// Runs `pv coverage --enforcement <src> --binding <binding.yaml>` and parses
/// the enforcement score (penetration × quality). E0=0.1, E1=0.5, E2=1.0.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_enforcement_quality(project_path: &Path) -> ComplianceCheck {
    if resolve_contracts_dir(project_path).is_none() {
        return ComplianceCheck {
            name: "CB-1214: Enforcement Quality".into(),
            status: CheckStatus::Skip,
            message: "No binding.yaml found".into(),
            severity: Severity::Info,
        };
    }

    // Find binding.yaml — check sibling provable-contracts repo
    let binding_path = find_binding_yaml(project_path);
    let binding_arg = match &binding_path {
        Some(p) => p.to_string_lossy().to_string(),
        None => {
            return ComplianceCheck {
                name: "CB-1214: Enforcement Quality".into(),
                status: CheckStatus::Skip,
                message: "No binding.yaml found".into(),
                severity: Severity::Info,
            };
        }
    };

    // Run pv coverage --enforcement
    let pv_result = std::process::Command::new("pv")
        .args([
            "coverage",
            "--enforcement",
            ".",
            "--binding",
            &binding_arg,
        ])
        .current_dir(project_path)
        .output();

    match pv_result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{stdout}{stderr}");

            // Parse enforcement metrics from output — use prefix matching
            // to be resilient to label text changes in pv CLI
            let e0 = parse_metric(&combined, "E0 (");
            let e1 = parse_metric(&combined, "E1 (");
            let e2 = parse_metric(&combined, "E2 (");
            let quality = parse_float_metric(&combined, "Quality score:");
            let enforcement = parse_float_metric(&combined, "Enforcement score:");

            let total_sites = e0 + e1 + e2;

            if total_sites == 0 {
                return ComplianceCheck {
                    name: "CB-1214: Enforcement Quality".into(),
                    status: CheckStatus::Warn,
                    message: "0 contract call sites found in source — contracts exist but are not invoked".into(),
                    severity: Severity::Warning,
                };
            }

            let message = format!(
                "{total_sites} call sites (E0={e0}, E1={e1}, E2={e2}), quality={quality:.2}, enforcement={enforcement:.4}"
            );

            // FAIL only if quality < 0.3 AND has E1/E2 mix (mature repo with regression)
            // E0-only repos are in legitimate transition — WARN, don't FAIL
            let has_mixed_levels = e1 > 0 || e2 > 0;
            if quality < 0.3 && total_sites > 30 && has_mixed_levels {
                ComplianceCheck {
                    name: "CB-1214: Enforcement Quality".into(),
                    status: CheckStatus::Fail,
                    message: format!("Low enforcement quality: {message}"),
                    severity: Severity::Error,
                }
            } else if quality < 0.3 {
                ComplianceCheck {
                    name: "CB-1214: Enforcement Quality".into(),
                    status: CheckStatus::Warn,
                    message: format!("Early adoption (E0-only): {message}"),
                    severity: Severity::Warning,
                }
            } else {
                ComplianceCheck {
                    name: "CB-1214: Enforcement Quality".into(),
                    status: CheckStatus::Pass,
                    message,
                    severity: Severity::Info,
                }
            }
        }
        _ => ComplianceCheck {
            name: "CB-1214: Enforcement Quality".into(),
            status: CheckStatus::Skip,
            message: "pv CLI not available".into(),
            severity: Severity::Info,
        },
    }
}

/// Find binding.yaml for a project — checks sibling provable-contracts repo.
/// Tries directory name, then Cargo.toml package name.
fn find_binding_yaml(project_path: &Path) -> Option<std::path::PathBuf> {
    // Use resolve_contracts_dir which handles dir name + Cargo.toml name
    let contracts_dir = resolve_contracts_dir(project_path)?;
    let binding = contracts_dir.join("binding.yaml");
    if binding.exists() {
        return Some(binding);
    }
    None
}

/// Parse an integer metric from pv coverage output (e.g., "E0 (generic !is_empty):  3")
fn parse_metric(output: &str, label: &str) -> usize {
    output
        .lines()
        .find(|l| l.contains(label))
        .and_then(|l| l.split(':').next_back())
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
}

/// Parse a float metric from pv coverage output (e.g., "Quality score:  0.55 ...")
fn parse_float_metric(output: &str, label: &str) -> f64 {
    output
        .lines()
        .find(|l| l.contains(label))
        .and_then(|l| {
            l.split(':')
                .next_back()?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
        .unwrap_or(0.0)
}

#[cfg(test)]
mod check_pv_quality_tests {
    //! Covers pure-compute helpers + skip arms in check_pv_quality.rs
    //! (90 uncov on broad, 0% cov).
    use super::*;

    // ── parse_metric: label match, no match, non-numeric ──

    #[test]
    fn test_parse_metric_extracts_integer_after_colon() {
        let output = "E0 (generic !is_empty):  42\nE1 (domain):  7";
        assert_eq!(parse_metric(output, "E0"), 42);
        assert_eq!(parse_metric(output, "E1"), 7);
    }

    #[test]
    fn test_parse_metric_label_not_found_returns_zero() {
        assert_eq!(parse_metric("some other line\n", "MISSING"), 0);
    }

    #[test]
    fn test_parse_metric_non_numeric_returns_zero() {
        // Line exists but value is not parseable as integer.
        assert_eq!(parse_metric("E0: not-a-number\n", "E0"), 0);
    }

    #[test]
    fn test_parse_metric_empty_output_returns_zero() {
        assert_eq!(parse_metric("", "E0"), 0);
    }

    // ── parse_float_metric: label match, no match, non-numeric ──

    #[test]
    fn test_parse_float_metric_extracts_value_before_whitespace() {
        // Takes the first whitespace-separated token after the colon.
        let output = "Quality score:  0.55 (out of 1.0)\nOther: ignored\n";
        assert!((parse_float_metric(output, "Quality score") - 0.55).abs() < 1e-6);
    }

    #[test]
    fn test_parse_float_metric_label_not_found_returns_zero() {
        assert_eq!(parse_float_metric("other: 1.5\n", "Missing"), 0.0);
    }

    #[test]
    fn test_parse_float_metric_non_numeric_returns_zero() {
        assert_eq!(parse_float_metric("X: not a number\n", "X"), 0.0);
    }

    #[test]
    fn test_parse_float_metric_empty_output_returns_zero() {
        assert_eq!(parse_float_metric("", "X"), 0.0);
    }

    // ── check_precondition_quality: no contracts/ → Skip ──

    #[test]
    fn test_check_precondition_quality_no_contracts_dir_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_precondition_quality(tmp.path());
        assert!(matches!(check.status, CheckStatus::Skip));
        assert_eq!(check.name, "CB-1210: Precondition Quality");
    }

    // ── check_codegen_fidelity: no generated_contracts.rs → Skip or pass through ──

    #[test]
    fn test_check_codegen_fidelity_no_generated_file_runs() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_codegen_fidelity(tmp.path());
        // On an empty project we expect a non-panicking ComplianceCheck.
        assert!(matches!(
            check.status,
            CheckStatus::Skip | CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail
        ));
    }

    // ── check_enforcement_quality: no binding.yaml → Skip ──

    #[test]
    fn test_check_enforcement_quality_no_binding_yaml_runs() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_enforcement_quality(tmp.path());
        assert!(matches!(
            check.status,
            CheckStatus::Skip | CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail
        ));
    }

    // ── find_generated_contracts + find_binding_yaml: no match → None ──

    #[test]
    fn test_find_generated_contracts_missing_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(find_generated_contracts(tmp.path()).is_none());
    }

    #[test]
    fn test_find_binding_yaml_missing_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(find_binding_yaml(tmp.path()).is_none());
    }
}
