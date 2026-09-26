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

    let contract_attr_lines = cb1203_collect_contract_attr_lines(&src_files);

    for eq in &eq_names {
        match cb1203_classify_equation(eq, &contract_attr_lines, &src_files, project_path) {
            Cb1203EqOutcome::BoundWithMacro => {
                bound_fns += 1;
                with_macro += 1;
            }
            Cb1203EqOutcome::BoundMissing(msg) => {
                bound_fns += 1;
                missing.push(msg);
            }
            // No matching pub fn — not bound (might be test-only or delegated).
            Cb1203EqOutcome::NotFound => {}
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

/// Collect every source line that is a `#[contract(...)]` attribute (matches both
/// `#[contract(` and `#[provable_contracts_macros::contract(`). Extracted from
/// `check_annotation_coverage` to keep it under the complexity gate.
fn cb1203_collect_contract_attr_lines(src_files: &[walkdir::DirEntry]) -> Vec<String> {
    let mut lines = Vec::new();
    for entry in src_files {
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with("#[contract(") || t.contains("::contract(") {
                lines.push(t.to_string());
            }
        }
    }
    lines
}

/// Outcome of classifying one contract equation against the source tree.
enum Cb1203EqOutcome {
    /// Covered by a `#[contract(... equation = "eq")]` attribute, or a `pub fn`
    /// was found that carries a contract macro / body contract.
    BoundWithMacro,
    /// A `pub fn eq(` was found but it has no contract annotation.
    BoundMissing(String),
    /// No matching `pub fn` — not bound (might be test-only or delegated).
    NotFound,
}

/// Classify a single contract equation. Extracted from `check_annotation_coverage`
/// to keep that function under the complexity gate; logic is preserved
/// (see the `test_cb1203_*` characterization tests).
fn cb1203_classify_equation(
    eq: &str,
    contract_attr_lines: &[String],
    src_files: &[walkdir::DirEntry],
    project_path: &Path,
) -> Cb1203EqOutcome {
    // Strategy 1: a #[contract] attribute references this equation.
    let attr_pattern = format!("equation = \"{eq}\"");
    if contract_attr_lines
        .iter()
        .any(|line| line.contains(&attr_pattern))
    {
        return Cb1203EqOutcome::BoundWithMacro;
    }

    // Strategy 2: find `pub fn <eq>(` and check for a contract macro in the 25
    // preceding lines (GH-271) or a body contract. First match wins.
    let pattern = format!("pub fn {eq}(");
    for entry in src_files {
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let Some(pos) = content.find(&pattern) else {
            continue;
        };
        let preceding_lines: Vec<&str> = content[..pos].lines().rev().take(25).collect();
        let has_macro = preceding_lines.iter().any(|line| {
            let t = line.trim();
            t.starts_with("#[contract(")
                || t.contains("::contract(")
                || t.starts_with("#[requires(")
                || t.starts_with("#[ensures(")
                || t.starts_with("#[invariant(")
        });
        let body_snippet: String = content[pos..].lines().take(20).collect::<Vec<_>>().join("\n");
        let has_body_contract = body_snippet.contains("contract_pre_")
            || body_snippet.contains("contract_post_")
            || body_snippet.contains("// Contract:");
        if has_macro || has_body_contract {
            return Cb1203EqOutcome::BoundWithMacro;
        }
        let rel = entry
            .path()
            .strip_prefix(project_path)
            .unwrap_or(entry.path());
        return Cb1203EqOutcome::BoundMissing(format!("{eq} in {}", rel.display()));
    }
    Cb1203EqOutcome::NotFound
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

    // Step 1: Run pv lint on resolved contracts dir — avoids scanning work/ YAMLs.
    // An Unknown or an error is a Fail whatever `pv_lint_is_error` says: it is
    // not a verdict, so it can never read as one (ONT-11, #1422).
    let (pv_passed, pv_error_detail) = match run_pv_lint(project_path, &contracts_dir) {
        PvLintRun::Judged { passed, detail } => (passed, detail),
        PvLintRun::Unknown(line) => {
            return ComplianceCheck {
                name: "CB-1201: PV Lint".into(),
                status: CheckStatus::Fail,
                message: format!("PV Lint could not decide (Unknown, not armed) — {line}"),
                severity: Severity::Error,
            };
        }
        PvLintRun::Errored(line) => {
            return ComplianceCheck {
                name: "CB-1201: PV Lint".into(),
                status: CheckStatus::Fail,
                message: format!("PV Lint errored — {line}"),
                severity: Severity::Error,
            };
        }
    };

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


/// What one `pv lint` run said. pv's exit code is its verdict: 0 pass,
/// 1 reject, 2 Unknown (a `decline:` line), 3 error (an `error:` line)
/// (ONT-001 R-21, #1422).
#[derive(Debug, Clone, PartialEq)]
enum PvLintRun {
    /// pv judged the contracts; `passed` is its JSON `passed` field.
    Judged {
        passed: bool,
        detail: Option<String>,
    },
    /// pv could not decide (exit 2), or could not be run at all.
    Unknown(String),
    /// pv itself failed (exit 3).
    Errored(String),
}

fn run_pv_lint(project_path: &Path, contracts_dir: &Path) -> PvLintRun {
    match std::process::Command::new("pv")
        .args(["lint", &contracts_dir.display().to_string(), "--format", "json"])
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(o) => classify_pv_lint(
            o.status.code(),
            &String::from_utf8_lossy(&o.stdout),
            &String::from_utf8_lossy(&o.stderr),
        ),
        Err(e) => pv_lint_spawn_error(&e),
    }
}

/// A `pv` that cannot be started has decided nothing.
fn pv_lint_spawn_error(e: &std::io::Error) -> PvLintRun {
    if e.kind() == std::io::ErrorKind::NotFound {
        PvLintRun::Unknown("decline: pv not found".into())
    } else {
        PvLintRun::Unknown(format!("decline: pv could not be run: {e}"))
    }
}

fn classify_pv_lint(code: Option<i32>, stdout: &str, stderr: &str) -> PvLintRun {
    match code {
        Some(2) => {
            return PvLintRun::Unknown(tagged_line("decline:", stderr, stdout).unwrap_or_else(
                || "decline: pv exited 2 without a decline: line".into(),
            ))
        }
        Some(3) => {
            return PvLintRun::Errored(
                tagged_line("error:", stderr, stdout)
                    .unwrap_or_else(|| "error: pv exited 3 without an error: line".into()),
            )
        }
        _ => {}
    }
    let json_val = serde_json::from_str::<serde_json::Value>(stdout).ok();
    let passed = json_val
        .as_ref()
        .and_then(|v| v.get("passed")?.as_bool())
        .unwrap_or(false);
    // Extract first error finding for diagnostics
    let detail = json_val
        .as_ref()
        .and_then(|v| v.get("findings")?.as_array())
        .and_then(|arr| {
            arr.iter().find(|f| {
                f.get("severity").and_then(|s| s.as_str()) == Some("error")
                    || f.get("severity").and_then(|s| s.as_str()) == Some("ERROR")
            })
        })
        .and_then(|f| f.get("message").and_then(|m| m.as_str()))
        .map(|s| s.to_string())
        .or_else(|| {
            // Fallback: first line of stderr
            stderr
                .lines()
                .next()
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty())
        });
    PvLintRun::Judged { passed, detail }
}

/// The first line of stderr, then stdout, that starts with `tag`, verbatim.
fn tagged_line(tag: &str, stderr: &str, stdout: &str) -> Option<String> {
    stderr
        .lines()
        .chain(stdout.lines())
        .map(str::trim)
        .find(|l| l.starts_with(tag))
        .map(str::to_string)
}
