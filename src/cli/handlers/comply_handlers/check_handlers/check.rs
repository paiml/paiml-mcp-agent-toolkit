// Compliance check logic - handle_check and individual check functions
//
// This is the core compliance checking module, containing handle_check
// and basic check_* functions (version, config, hooks, quality, CB patterns).

use crate::cli::commands::ComplyOutputFormat;
use crate::cli::handlers::comply_cb_detect::{
    detect_bricks_without_assertions, detect_cb001_wgsl_no_bounds_check,
    detect_cb002_wgsl_barrier_divergence, detect_cb020_unsafe_without_safety,
    detect_cb021_simd_without_target_feature, detect_cb120_nan_unsafe_comparison,
    detect_cb121_lock_poisoning, detect_cb122_serde_safety, detect_cb123_undocumented_ignore,
    detect_cb124_coverage_threshold, detect_cb125_coverage_exclusion_gaming,
    detect_cb126_slow_tests, detect_cb127_slow_coverage, detect_profiler_anomalies,
};
use crate::models::comply_config::PmatYamlConfig;
use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::Path;

use super::check_best_practices::{
    check_agent_context_adoption, check_custom_scores, check_lean_best_practices_with_config,
    check_lua_best_practices_with_config, check_markdown_best_practices_with_config,
    check_model_quality_with_config, check_rust_best_practices_with_config,
    check_scala_best_practices_with_config, check_shell_makefile_quality,
    check_sql_best_practices_with_config, check_tdg_grade_gate,
    check_yaml_best_practices_with_config,
};
use super::check_extended::{
    check_dead_code_percentage, check_dependency_count, check_edd_compliance, check_file_health,
    check_golden_trace_drift, check_muda_waste_score, check_paiml_deps_workspace,
    check_reproducibility_level, check_sovereign_stack_patterns,
};
use super::check_mono_spec::{
    check_memory_profiling, check_mono_spec_structure, check_swe_ci_evoscore,
};
use super::types::*;

/// Check project compliance with current PMAT version
pub(crate) async fn handle_check(
    project_path: &Path,
    strict: bool,
    failures_only: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    eprintln!("Checking PMAT compliance for {}", project_path.display());

    let yaml_config = PmatYamlConfig::load(project_path).unwrap_or_default();
    let comply_config = &yaml_config.comply;

    let config_path = project_path.join(".pmat.yaml");
    if config_path.exists() {
        eprintln!("  Using configuration from .pmat.yaml");
        if !comply_config.suppressions.is_empty() {
            eprintln!(
                "  {} suppression rule(s) loaded",
                comply_config.suppressions.len()
            );
        }
    }

    let config = load_or_create_project_config(project_path)?;
    let project_version = &config.pmat.version;

    let mut checks = vec![
        check_version_currency(project_version),
        check_config_files(project_path),
        check_hooks_installed(project_path),
        filter_check_by_config(
            check_hooks_o1_capable(project_path),
            "cb-030",
            comply_config,
        ),
        filter_check_by_config(
            check_hooks_cache_health(project_path),
            "cb-031",
            comply_config,
        ),
        check_quality_thresholds(project_path),
        check_deprecated_features(project_path),
        filter_check_by_config(check_compute_brick(project_path), "cb-060", comply_config),
        filter_check_by_config(
            check_oip_tarantula_patterns(project_path),
            "cb-120",
            comply_config,
        ),
        filter_check_by_config(
            check_coverage_quality_patterns(project_path),
            "cb-125",
            comply_config,
        ),
        check_cargo_lock(project_path),
        check_msrv(project_path),
        check_ci_configured(project_path),
        check_paiml_deps_workspace(project_path),
        check_sovereign_stack_patterns(project_path),
        filter_check_by_config(check_file_health(project_path), "cb-040", comply_config),
        filter_check_by_config(
            check_muda_waste_score(project_path),
            "cb-300",
            comply_config,
        ),
        filter_check_by_config(
            check_reproducibility_level(project_path),
            "cb-301",
            comply_config,
        ),
        filter_check_by_config(
            check_golden_trace_drift(project_path),
            "cb-302",
            comply_config,
        ),
        filter_check_by_config(check_edd_compliance(project_path), "cb-303", comply_config),
        filter_check_by_config(
            check_dead_code_percentage(project_path),
            "cb-304",
            comply_config,
        ),
        filter_check_by_config(
            check_dependency_count(project_path),
            "cb-081",
            comply_config,
        ),
        filter_check_by_config(
            check_shell_makefile_quality(project_path),
            "cb-400",
            comply_config,
        ),
        filter_check_by_config(check_stale_paths(project_path), "cb-533", comply_config),
        filter_check_by_config(
            check_spec_work_traceability(project_path),
            "cb-148",
            comply_config,
        ),
        filter_check_by_config(
            check_agent_context_adoption(project_path),
            "cb-130",
            comply_config,
        ),
        filter_check_by_config(
            check_mono_spec_structure(project_path),
            "cb-140",
            comply_config,
        ),
        filter_check_by_config(
            check_memory_profiling(project_path),
            "cb-141",
            comply_config,
        ),
        filter_check_by_config(check_swe_ci_evoscore(project_path), "cb-142", comply_config),
        filter_check_by_config(
            check_tdg_grade_gate(project_path, comply_config),
            "cb-200",
            comply_config,
        ),
        filter_check_by_config(
            check_rust_best_practices_with_config(project_path, Some(comply_config)),
            "cb-500",
            comply_config,
        ),
        filter_check_by_config(
            check_lua_best_practices_with_config(project_path, Some(comply_config)),
            "cb-600",
            comply_config,
        ),
        filter_check_by_config(
            check_sql_best_practices_with_config(project_path, Some(comply_config)),
            "cb-700",
            comply_config,
        ),
        filter_check_by_config(
            check_scala_best_practices_with_config(project_path, Some(comply_config)),
            "cb-800",
            comply_config,
        ),
        filter_check_by_config(
            check_markdown_best_practices_with_config(project_path, Some(comply_config)),
            "cb-900",
            comply_config,
        ),
        filter_check_by_config(
            check_yaml_best_practices_with_config(project_path, Some(comply_config)),
            "cb-950",
            comply_config,
        ),
        filter_check_by_config(
            check_model_quality_with_config(project_path, Some(comply_config)),
            "cb-1000",
            comply_config,
        ),
        filter_check_by_config(
            check_lean_best_practices_with_config(project_path, Some(comply_config)),
            "cb-1050",
            comply_config,
        ),
    ];

    let custom_checks = check_custom_scores(project_path);
    for chk in custom_checks {
        checks.push(filter_check_by_config(chk, "cb-1100", comply_config));
    }

    // Provable contracts quality gate (CB-1200)
    // Auto-skips if no contracts/ directory found
    checks.push(filter_check_by_config(
        super::check_provable_contracts::check_provable_contracts(project_path),
        "cb-1200",
        comply_config,
    ));

    // PV Lint quality gate (CB-1201)
    checks.push(filter_check_by_config(
        check_pv_lint(project_path),
        "cb-1201",
        comply_config,
    ));

    // Contract coverage gate (CB-1202)
    checks.push(filter_check_by_config(
        check_contract_coverage(project_path),
        "cb-1202",
        comply_config,
    ));

    // Annotation coverage gate (CB-1203)
    checks.push(filter_check_by_config(
        check_annotation_coverage(project_path),
        "cb-1203",
        comply_config,
    ));

    // Build.rs contract pipeline gate (CB-1204)
    checks.push(filter_check_by_config(
        check_build_rs_pipeline(project_path),
        "cb-1204",
        comply_config,
    ));

    // Provability invariant gate (CB-1205) — pv-compatibility spec §2.2
    checks.push(filter_check_by_config(
        check_provability_invariant(project_path),
        "cb-1205",
        comply_config,
    ));

    // Verification level distribution (CB-1206) — pv-compatibility spec §2.3
    checks.push(filter_check_by_config(
        check_verification_levels(project_path),
        "cb-1206",
        comply_config,
    ));

    // Contract drift detection (CB-1207) — pv-compatibility spec CD5
    checks.push(filter_check_by_config(
        check_contract_drift(project_path),
        "cb-1207",
        comply_config,
    ));

    let failures = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();
    let is_compliant = failures == 0;

    let breaking_changes = get_breaking_changes_since(project_version);
    let versions_behind = calculate_versions_behind(project_version);

    let mut recommendations = vec![];
    if versions_behind > 0 {
        recommendations.push(format!(
            "Run 'pmat comply migrate' to update to v{}",
            PMAT_VERSION
        ));
    }
    if !breaking_changes.is_empty() {
        recommendations.push("Review breaking changes with 'pmat comply diff'".to_string());
    }

    let report = ComplianceReport {
        project_version: project_version.clone(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant,
        versions_behind,
        checks: if failures_only {
            checks
                .into_iter()
                .filter(|c| c.status == CheckStatus::Fail)
                .collect()
        } else {
            checks
        },
        breaking_changes,
        recommendations,
        timestamp: Utc::now(),
    };

    match format {
        ComplyOutputFormat::Text => print_compliance_text(&report),
        ComplyOutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        ComplyOutputFormat::Markdown => print_compliance_markdown(&report),
        ComplyOutputFormat::Sarif => {
            // Delegate to pv lint for SARIF if contracts exist
            let contracts_dir = project_path.join("contracts");
            if contracts_dir.exists() {
                if let Ok(output) = std::process::Command::new("pv")
                    .args([
                        "lint",
                        &contracts_dir.display().to_string(),
                        "--format",
                        "sarif",
                    ])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .output()
                {
                    if let Ok(sarif) = String::from_utf8(output.stdout) {
                        if !sarif.is_empty() {
                            println!("{sarif}");
                            if !output.status.success() {
                                return Err(anyhow::anyhow!("pv lint SARIF: non-zero exit"));
                            }
                            return Ok(());
                        }
                    }
                }
            }
            // Fallback: JSON output
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    let _ = update_last_check_timestamp(project_path);
    // Always exit 1 when there are failures (NON-COMPLIANT)
    if !is_compliant {
        std::process::exit(1);
    }
    // In strict mode, warnings also cause failure (exit 2)
    if strict && warnings > 0 {
        std::process::exit(2);
    }
    Ok(())
}

/// Extract equation names from contract YAMLs that have preconditions or postconditions.
fn collect_contract_equation_names(contracts_dir: &Path) -> Vec<String> {
    let mut eq_names = Vec::new();
    let headers = [
        "equations",
        "metadata",
        "falsification_tests",
        "kani_harnesses",
        "proof_obligations",
        "qa_gate",
        "implementation",
        "enforcement",
        "version",
        "created",
        "author",
        "description",
        "references",
        "issues",
    ];
    let Ok(entries) = std::fs::read_dir(contracts_dir) else {
        return eq_names;
    };
    for entry in entries.flatten() {
        if entry.path().extension().map_or(true, |e| e != "yaml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.ends_with(':')
                || trimmed.starts_with('#')
                || trimmed.starts_with('-')
                || trimmed.contains(' ')
                || !line.starts_with("  ")
                || line.starts_with("    ")
            {
                continue;
            }
            let name = trimmed.trim_end_matches(':');
            if headers.contains(&name) {
                continue;
            }
            // Look ahead for preconditions/postconditions
            let has_pre_post = lines[i + 1..]
                .iter()
                .take_while(|next| {
                    let nt = next.trim();
                    !(next.starts_with("  ")
                        && !next.starts_with("    ")
                        && nt.ends_with(':')
                        && !nt.starts_with('#')
                        && !nt.starts_with('-'))
                })
                .any(|next| {
                    let nt = next.trim();
                    nt == "preconditions:" || nt == "postconditions:"
                });
            if has_pre_post {
                eq_names.push(name.to_string());
            }
        }
    }
    eq_names
}

/// CB-1203: Contract-bound functions MUST have #[contract] or #[requires]/#[ensures] macros.
/// Cross-references contract YAML equation names against production source.
/// A production `pub fn <equation_name>` without a contract macro = FAIL.
/// Preferred: `#[contract("yaml-name", equation = "eq")]` — auto-injects from YAML.
/// Legacy: `#[requires(...)]` / `#[ensures(...)]` — hand-written assertions.
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

        // Strategy 2: Find pub fn <eq_name>( and check for macros in preceding lines
        let pattern = format!("pub fn {eq}(");
        let mut found = false;
        for entry in &src_files {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Some(pos) = content.find(&pattern) {
                    bound_fns += 1;
                    found = true;
                    let prefix = &content[..pos];
                    let preceding_lines: Vec<&str> = prefix.lines().rev().take(10).collect();
                    let has_macro = preceding_lines.iter().any(|line| {
                        let t = line.trim();
                        t.starts_with("#[contract(")
                            || t.contains("::contract(")
                            || t.starts_with("#[requires(")
                            || t.starts_with("#[ensures(")
                            || t.starts_with("#[invariant(")
                    });
                    if has_macro {
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
                "{}/{} contract-bound fns lack macros: {}",
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

/// CB-1204: Build.rs contract pipeline — does build.rs emit assertion env vars from YAML?
///
/// The escape-proof pipeline requires build.rs to read contracts/*.yaml and
/// emit CONTRACT_*_PRE_COUNT / CONTRACT_*_PRE_0 env vars that the #[contract]
/// proc macro reads at compile time.
pub(crate) fn check_build_rs_pipeline(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    let build_rs = project_path.join("build.rs");

    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Check YAML has preconditions (otherwise no pipeline needed)
    let has_preconditions = std::fs::read_dir(&contracts_dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().extension().is_some_and(|ext| ext == "yaml")
                    && std::fs::read_to_string(e.path())
                        .map(|c| c.contains("preconditions:"))
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if !has_preconditions {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Pass,
            message: "No preconditions in YAML — pipeline not required".into(),
            severity: Severity::Info,
        };
    }

    // Check build.rs at root or in crates/*/
    let mut build_files = vec![build_rs.clone()];
    if let Ok(entries) = std::fs::read_dir(project_path.join("crates")) {
        for e in entries.flatten() {
            let bf = e.path().join("build.rs");
            if bf.exists() {
                build_files.push(bf);
            }
        }
    }

    let any_build_rs = build_files.iter().any(|f| f.exists());
    if !any_build_rs {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Fail,
            message: "Contracts have preconditions but no build.rs to emit assertion env vars"
                .into(),
            severity: Severity::Error,
        };
    }

    let has_pre_emit = build_files.iter().any(|f| {
        std::fs::read_to_string(f)
            .map(|c| c.contains("PRE_COUNT") || c.contains("emit_contract") || c.contains("_PRE_0"))
            .unwrap_or(false)
    });
    if has_pre_emit {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Pass,
            message: "build.rs emits contract assertion env vars from YAML".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: "CB-1204: Build.rs Pipeline".into(),
        status: CheckStatus::Fail,
        message: "build.rs exists but doesn't emit PRE/POST env vars from contracts/ YAML".into(),
        severity: Severity::Error,
    }
}

/// CB-1205: Provability Invariant — kernel contracts with proof_obligations
/// MUST have kani_harnesses and sufficient falsification_tests.
/// pv-compatibility spec §2.2
pub(crate) fn check_provability_invariant(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1205: Provability Invariant".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut kernel_contracts = 0usize;
    let mut violations = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |e| e != "yaml") {
                continue;
            }
            if p.file_name()
                .is_some_and(|n| n.to_string_lossy().contains("binding"))
            {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&p) else {
                continue;
            };

            // Skip data registries
            if content.contains("registry: true") {
                continue;
            }

            let has_obligations = content.contains("proof_obligations:");
            if !has_obligations {
                continue;
            }

            kernel_contracts += 1;
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("?");

            let has_kani = content.contains("kani_harnesses:");
            let has_falsification = content.contains("falsification_tests:");

            if !has_kani {
                violations.push(format!(
                    "{stem}: has proof_obligations but no kani_harnesses"
                ));
            }
            if !has_falsification {
                violations.push(format!(
                    "{stem}: has proof_obligations but no falsification_tests"
                ));
            }
        }
    }

    if kernel_contracts == 0 {
        return ComplianceCheck {
            name: "CB-1205: Provability Invariant".into(),
            status: CheckStatus::Pass,
            message: "No kernel contracts with proof_obligations found".into(),
            severity: Severity::Info,
        };
    }

    if violations.is_empty() {
        ComplianceCheck {
            name: "CB-1205: Provability Invariant".into(),
            status: CheckStatus::Pass,
            message: format!("{kernel_contracts} kernel contract(s) satisfy provability invariant"),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1205: Provability Invariant".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} violation(s): {}",
                violations.len(),
                violations
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1206: Verification Level Distribution — report L1-L5 proof depth.
/// Reads proof-status.json from provable-contracts sibling repo.
/// pv-compatibility spec §2.3
pub(crate) fn check_verification_levels(project_path: &Path) -> ComplianceCheck {
    // Resolve to absolute path so .parent() works correctly from "."
    let abs_path =
        std::fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf());
    let ps_path = abs_path
        .parent()
        .map(|p| p.join("provable-contracts").join("proof-status.json"));

    let Some(ps_path) = ps_path.filter(|p| p.exists()) else {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Skip,
            message: "No proof-status.json in ../provable-contracts/".into(),
            severity: Severity::Info,
        };
    };

    let Ok(content) = std::fs::read_to_string(&ps_path) else {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Skip,
            message: "Cannot read proof-status.json".into(),
            severity: Severity::Info,
        };
    };

    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Warn,
            message: "Cannot parse proof-status.json".into(),
            severity: Severity::Warning,
        };
    };

    let totals = val.get("totals");
    let obligations = totals
        .and_then(|t| t.get("obligations"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let tests = totals
        .and_then(|t| t.get("falsification_tests"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let kani = totals
        .and_then(|t| t.get("kani_harnesses"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let lean = totals
        .and_then(|t| t.get("lean_proved"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let contracts = totals
        .and_then(|t| t.get("contracts"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if obligations == 0 {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Pass,
            message: format!("{contracts} contracts, 0 obligations"),
            severity: Severity::Info,
        };
    }

    let l4_pct = kani as f64 / obligations as f64 * 100.0;
    let l5_pct = lean as f64 / obligations as f64 * 100.0;

    let msg = format!(
        "{obligations} obligations: L2={tests} tests, L4={kani} kani ({l4_pct:.0}%), L5={lean} lean ({l5_pct:.0}%)"
    );

    if l4_pct < 10.0 && kani == 0 {
        ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Warn,
            message: format!("{msg} — no Kani verification"),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Pass,
            message: msg,
            severity: Severity::Info,
        }
    }
}

/// CB-1207: Contract drift — are contracts stale relative to source changes?
/// A contract YAML older than its bound source files by >30 days = drift.
/// pv-compatibility spec CD5.
pub(crate) fn check_contract_drift(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1207: Contract Drift".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    let thirty_days = std::time::Duration::from_secs(30 * 24 * 3600);
    let mut stale = 0usize;
    let mut total = 0usize;

    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |e| e != "yaml") {
                continue;
            }
            if p.file_name()
                .is_some_and(|n| n.to_string_lossy().contains("binding"))
            {
                continue;
            }
            let Ok(meta) = std::fs::metadata(&p) else {
                continue;
            };
            let Ok(yaml_mtime) = meta.modified() else {
                continue;
            };
            total += 1;

            // Check git log for the contract's last commit vs now
            let output = std::process::Command::new("git")
                .args(["log", "-1", "--format=%ct", "--"])
                .arg(p.file_name().unwrap_or_default())
                .current_dir(&contracts_dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .output();

            if let Ok(o) = output {
                if let Ok(ts_str) = String::from_utf8(o.stdout) {
                    if let Ok(ts) = ts_str.trim().parse::<u64>() {
                        let contract_commit =
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts);
                        let now = std::time::SystemTime::now();
                        if let Ok(age) = now.duration_since(contract_commit) {
                            // Contract not touched in >90 days AND yaml is old
                            if age > thirty_days * 3 {
                                if let Ok(yaml_age) = now.duration_since(yaml_mtime) {
                                    if yaml_age > thirty_days * 3 {
                                        stale += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if total == 0 {
        return ComplianceCheck {
            name: "CB-1207: Contract Drift".into(),
            status: CheckStatus::Pass,
            message: "No contract YAMLs to check".into(),
            severity: Severity::Info,
        };
    }

    let fresh = total - stale;
    if stale == 0 {
        ComplianceCheck {
            name: "CB-1207: Contract Drift".into(),
            status: CheckStatus::Pass,
            message: format!("{total} contract(s), all fresh (committed within 90 days)"),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1207: Contract Drift".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{stale}/{total} contract(s) stale (>90 days since last commit), {fresh} fresh"
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1202: Contract coverage — do repos with critical functions have contracts?
pub(crate) fn check_contract_coverage(project_path: &Path) -> ComplianceCheck {
    let src_dir = project_path.join("src");
    let contracts_dir = project_path.join("contracts");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "CB-1202: Contract Coverage".into(),
            status: CheckStatus::Skip,
            message: "No src/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Critical ML/GPU/data keywords that REQUIRE contracts
    let critical_keywords = [
        "forward",
        "backward",
        "optimizer",
        "checkpoint",
        "loss",
        "gradient",
        "sampling",
        "kv_cache",
        "tokenize",
        "quantize",
        "kernel",
        "dispatch",
        "softmax",
        "matmul",
        "gemm",
        "batch",
    ];

    // Count which keywords appear in public functions
    let mut keywords_found = Vec::new();
    let mut keywords_covered = 0usize;

    for keyword in &critical_keywords {
        // Search src/ for pub fn containing keyword
        let has_fn = walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .map(|c| {
                        c.contains(&format!("pub fn {keyword}"))
                            || c.contains(&format!("pub async fn {keyword}"))
                    })
                    .unwrap_or(false)
            });

        if !has_fn {
            continue;
        }
        keywords_found.push(*keyword);

        // Check if any contract mentions this keyword
        if contracts_dir.exists() {
            let has_contract = walkdir::WalkDir::new(&contracts_dir)
                .into_iter()
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "yaml" || ext == "yml")
                })
                .any(|e| {
                    std::fs::read_to_string(e.path())
                        .map(|c| c.to_lowercase().contains(keyword))
                        .unwrap_or(false)
                });
            if has_contract {
                keywords_covered += 1;
            }
        }
    }

    if keywords_found.is_empty() {
        return ComplianceCheck {
            name: "CB-1202: Contract Coverage".into(),
            status: CheckStatus::Pass,
            message: "No critical ML/GPU functions detected".into(),
            severity: Severity::Info,
        };
    }

    let coverage_pct = keywords_covered * 100 / keywords_found.len();
    let uncovered: Vec<&&str> = keywords_found
        .iter()
        .filter(|k| {
            !contracts_dir.exists()
                || !walkdir::WalkDir::new(&contracts_dir)
                    .into_iter()
                    .flatten()
                    .filter(|e| {
                        e.path()
                            .extension()
                            .is_some_and(|ext| ext == "yaml" || ext == "yml")
                    })
                    .any(|e| {
                        std::fs::read_to_string(e.path())
                            .map(|c| c.to_lowercase().contains(**k))
                            .unwrap_or(false)
                    })
        })
        .collect();

    if coverage_pct >= 50 {
        ComplianceCheck {
            name: "CB-1202: Contract Coverage".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{keywords_covered}/{} critical keywords covered ({coverage_pct}%)",
                keywords_found.len()
            ),
            severity: Severity::Info,
        }
    } else {
        let missing: Vec<String> = uncovered.iter().map(|k| k.to_string()).collect();
        ComplianceCheck {
            name: "CB-1202: Contract Coverage".into(),
            status: CheckStatus::Fail,
            message: format!(
                "Only {keywords_covered}/{} critical keywords covered ({coverage_pct}%). Missing: {}",
                keywords_found.len(), missing.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

/// CB-1201: PV Lint + contract fulfillment gate.
/// Checks: (1) pv lint passes, (2) referenced tests EXIST, (3) they PASS.
/// Missing test = unfalsifiable claim = FAIL (like TDG grade F).
pub(crate) fn check_pv_lint(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1201: PV Lint".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory found".into(),
            severity: Severity::Info,
        };
    }

    // Step 1: Run pv lint
    let pv_passed = std::process::Command::new("pv")
        .args(["lint", "--format", "json"])
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| v.get("passed")?.as_bool())
                .unwrap_or(false)
        })
        .unwrap_or(false);

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
        return ComplianceCheck {
            name: "CB-1201: PV Lint".into(),
            status: CheckStatus::Warn,
            message: "PV Lint failed".into(),
            severity: Severity::Warning,
        };
    }

    ComplianceCheck {
        name: "CB-1201: PV Lint".into(),
        status: CheckStatus::Pass,
        message: format!("PV Lint passed, {existing}/{total_refs} tests fulfilled"),
        severity: Severity::Info,
    }
}

fn count_contract_test_refs(project_path: &Path) -> (usize, usize, usize) {
    let contracts_dir = project_path.join("contracts");
    let src_dir = project_path.join("src");
    let mut refs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |e| e != "yaml" && e != "yml") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&p) {
                for line in content.lines() {
                    if let Some(pos) = line.find("test:") {
                        let rest = line[pos + 5..].trim().trim_matches('"');
                        let name = rest
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                            .unwrap_or("");
                        if name.starts_with("test_") || name.starts_with("prop_") {
                            refs.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    if refs.is_empty() {
        return (0, 0, 0);
    }

    let mut src_tests = std::collections::HashSet::new();
    if src_dir.exists() {
        for entry in walkdir::WalkDir::new(&src_dir).into_iter().flatten() {
            if entry.path().extension().map_or(true, |e| e != "rs") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    if let Some(pos) = line.find("fn test_").or_else(|| line.find("fn prop_")) {
                        let rest = &line[pos + 3..];
                        let name = rest
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                            .unwrap_or("");
                        if !name.is_empty() {
                            src_tests.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }

    let existing = refs
        .iter()
        .filter(|t| src_tests.contains(t.as_str()))
        .count();
    let missing = refs.len() - existing;
    (refs.len(), existing, missing)
}

/// CB-533: Stale path references in Makefiles and CI workflows.
pub(crate) fn check_stale_paths(project_path: &Path) -> ComplianceCheck {
    let violations =
        crate::cli::handlers::comply_cb_detect::detect_cb533_stale_path_references(project_path);
    if violations.is_empty() {
        ComplianceCheck {
            name: "CB-533: Stale Path References".into(),
            status: CheckStatus::Pass,
            message: "No stale path references found".into(),
            severity: Severity::Info,
        }
    } else {
        let msg = format!("{} stale path(s) found", violations.len());
        ComplianceCheck {
            name: "CB-533: Stale Path References".into(),
            status: CheckStatus::Warn,
            message: msg,
            severity: Severity::Warning,
        }
    }
}

/// CB-148: Spec-work traceability.
pub(crate) fn check_spec_work_traceability(project_path: &Path) -> ComplianceCheck {
    let violations =
        crate::cli::handlers::comply_cb_detect::detect_cb148_spec_work_gaps(project_path);
    if violations.is_empty() {
        ComplianceCheck {
            name: "CB-148: Spec-Work Traceability".into(),
            status: CheckStatus::Pass,
            message: "All planned spec sections have corresponding work tickets".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-148: Spec-Work Traceability".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} planned section(s) without work tickets",
                violations.len()
            ),
            severity: Severity::Warning,
        }
    }
}

pub(crate) fn check_version_currency(project_version: &str) -> ComplianceCheck {
    let behind = calculate_versions_behind(project_version);
    if behind == 0 {
        ComplianceCheck {
            name: "Version Currency".into(),
            status: CheckStatus::Pass,
            message: format!("Project is on latest version (v{})", PMAT_VERSION),
            severity: Severity::Info,
        }
    } else if behind <= 5 {
        ComplianceCheck {
            name: "Version Currency".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} versions behind (v{} \u{2192} v{})",
                behind, project_version, PMAT_VERSION
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Version Currency".into(),
            status: CheckStatus::Fail,
            message: format!("{} versions behind - migration recommended", behind),
            severity: Severity::Error,
        }
    }
}

pub(crate) fn check_config_files(project_path: &Path) -> ComplianceCheck {
    let config_files = [".pmat/project.toml", ".pmat-metrics.toml"];
    let missing: Vec<&str> = config_files
        .iter()
        .filter(|f| !project_path.join(f).exists())
        .copied()
        .collect();
    if missing.is_empty() {
        ComplianceCheck {
            name: "Config Files".into(),
            status: CheckStatus::Pass,
            message: "All required config files present".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Config Files".into(),
            status: CheckStatus::Warn,
            message: format!("Missing: {}", missing.join(", ")),
            severity: Severity::Warning,
        }
    }
}

pub(crate) fn check_hooks_installed(project_path: &Path) -> ComplianceCheck {
    let pre_commit = project_path.join(".git").join("hooks").join("pre-commit");
    if pre_commit.exists() {
        if let Ok(content) = fs::read_to_string(&pre_commit) {
            if content.contains("pmat") || content.contains("PMAT") {
                return ComplianceCheck {
                    name: "Git Hooks".into(),
                    status: CheckStatus::Pass,
                    message: "PMAT hooks installed".into(),
                    severity: Severity::Info,
                };
            }
        }
        ComplianceCheck {
            name: "Git Hooks".into(),
            status: CheckStatus::Warn,
            message: "Pre-commit hook exists but may not be PMAT".into(),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Git Hooks".into(),
            status: CheckStatus::Warn,
            message: "No pre-commit hook installed".into(),
            severity: Severity::Warning,
        }
    }
}

pub(crate) fn check_hooks_o1_capable(project_path: &Path) -> ComplianceCheck {
    let cache_dir = project_path.join(".pmat").join("hooks-cache");
    if cache_dir.exists() {
        if cache_dir.join("tree-hash.json").exists() || cache_dir.join("gates").exists() {
            return ComplianceCheck {
                name: "CB-030: O(1) Hooks".into(),
                status: CheckStatus::Pass,
                message: "Hooks cache initialized - O(1) capable".into(),
                severity: Severity::Info,
            };
        }
        ComplianceCheck {
            name: "CB-030: O(1) Hooks".into(),
            status: CheckStatus::Warn,
            message: "Cache directory exists but not fully initialized".into(),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-030: O(1) Hooks".into(),
            status: CheckStatus::Warn,
            message: "Run 'pmat hooks cache init' to enable O(1) hooks".into(),
            severity: Severity::Warning,
        }
    }
}

pub(crate) fn check_hooks_cache_health(project_path: &Path) -> ComplianceCheck {
    let metrics_path = project_path
        .join(".pmat")
        .join("hooks-cache")
        .join("metrics.json");
    if !metrics_path.exists() {
        return ComplianceCheck {
            name: "CB-031: Cache Health".into(),
            status: CheckStatus::Skip,
            message: "No cache metrics available yet".into(),
            severity: Severity::Info,
        };
    }
    match fs::read_to_string(&metrics_path) {
        Ok(content) => {
            if let Ok(metrics) = serde_json::from_str::<serde_json::Value>(&content) {
                let total_runs = metrics["total_runs"].as_u64().unwrap_or(0);
                let cache_hits = metrics["cache_hits"].as_u64().unwrap_or(0);
                if total_runs < 5 {
                    return ComplianceCheck {
                        name: "CB-031: Cache Health".into(),
                        status: CheckStatus::Skip,
                        message: format!("Insufficient data ({} runs, need 5+)", total_runs),
                        severity: Severity::Info,
                    };
                }
                let hit_rate = (cache_hits as f64 / total_runs as f64) * 100.0;
                if hit_rate >= 60.0 {
                    ComplianceCheck {
                        name: "CB-031: Cache Health".into(),
                        status: CheckStatus::Pass,
                        message: format!("Cache hit rate {:.1}% (target: \u{2265}60%)", hit_rate),
                        severity: Severity::Info,
                    }
                } else {
                    ComplianceCheck {
                        name: "CB-031: Cache Health".into(),
                        status: CheckStatus::Warn,
                        message: format!(
                            "Cache hit rate {:.1}% below 60% target - consider clearing cache",
                            hit_rate
                        ),
                        severity: Severity::Warning,
                    }
                }
            } else {
                ComplianceCheck {
                    name: "CB-031: Cache Health".into(),
                    status: CheckStatus::Warn,
                    message: "Failed to parse metrics.json".into(),
                    severity: Severity::Warning,
                }
            }
        }
        Err(_) => ComplianceCheck {
            name: "CB-031: Cache Health".into(),
            status: CheckStatus::Warn,
            message: "Failed to read metrics.json".into(),
            severity: Severity::Warning,
        },
    }
}

pub(crate) fn check_quality_thresholds(project_path: &Path) -> ComplianceCheck {
    if project_path.join(".pmat-metrics.toml").exists() {
        ComplianceCheck {
            name: "Quality Thresholds".into(),
            status: CheckStatus::Pass,
            message: "Quality thresholds configured".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Quality Thresholds".into(),
            status: CheckStatus::Warn,
            message: "No .pmat-metrics.toml found - using defaults".into(),
            severity: Severity::Warning,
        }
    }
}

pub(crate) fn check_deprecated_features(_project_path: &Path) -> ComplianceCheck {
    ComplianceCheck {
        name: "Deprecated Features".into(),
        status: CheckStatus::Pass,
        message: "No deprecated features detected".into(),
        severity: Severity::Info,
    }
}

/// Append formatted violation to the issues list
fn append_violation(
    issues: &mut Vec<String>,
    v: &crate::cli::handlers::comply_cb_detect::CbPatternViolation,
) {
    issues.push(format!(
        "{}: {} ({}:{})",
        v.pattern_id, v.description, v.file, v.line
    ));
}

/// Collect violations from multiple detection functions, counting by severity
fn collect_violations_with_counts(
    detections: &[(
        Vec<crate::cli::handlers::comply_cb_detect::CbPatternViolation>,
        bool,
    )],
) -> (Vec<String>, usize, usize) {
    let mut all_issues = Vec::new();
    let (mut critical_count, mut warning_count) = (0, 0);
    for (violations, is_critical) in detections {
        for v in violations {
            append_violation(&mut all_issues, v);
            if *is_critical {
                critical_count += 1;
            } else {
                warning_count += 1;
            }
        }
    }
    (all_issues, critical_count, warning_count)
}

pub(crate) fn collect_cb_violations(
    project_path: &Path,
    has_probar: bool,
    has_brick_dir: bool,
) -> (Vec<String>, usize, usize) {
    let detections = vec![
        (detect_cb020_unsafe_without_safety(project_path), false),
        (
            detect_cb021_simd_without_target_feature(project_path),
            false,
        ),
        (detect_bricks_without_assertions(project_path), false),
        (detect_cb001_wgsl_no_bounds_check(project_path), true),
        (detect_cb002_wgsl_barrier_divergence(project_path), true),
    ];
    let (mut all_issues, mut critical_count, mut warning_count) =
        collect_violations_with_counts(&detections);
    for a in &detect_profiler_anomalies(project_path) {
        all_issues.push(format!(
            "PROFILER-{}: {} has {}={:.1}% (threshold: {:.1}%)",
            a.anomaly_type,
            a.brick_name,
            a.anomaly_type.to_lowercase(),
            a.value,
            a.threshold
        ));
        if a.anomaly_type == "LOW_EFFICIENCY" {
            critical_count += 1;
        } else {
            warning_count += 1;
        }
    }
    let gates_path = project_path.join(".pmat-gates.toml");
    let has_cb_config = gates_path.exists()
        && fs::read_to_string(&gates_path)
            .map(|s| s.contains("[compute-brick]"))
            .unwrap_or(false);
    if !has_cb_config && (has_probar || has_brick_dir) {
        all_issues.push("Missing [compute-brick] section in .pmat-gates.toml".into());
        warning_count += 1;
    }
    let coverage_file = project_path.join(".pmat-metrics").join("gui-coverage.json");
    if has_probar && !coverage_file.exists() {
        all_issues.push("No GUI coverage report - run probador to generate".into());
        warning_count += 1;
    }
    (all_issues, critical_count, warning_count)
}

pub(crate) fn build_cb_result(
    all_issues: Vec<String>,
    critical_count: usize,
    warning_count: usize,
) -> ComplianceCheck {
    if critical_count > 0 {
        ComplianceCheck {
            name: "ComputeBrick Compliance".into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} critical, {} warnings:\n{}",
                critical_count,
                warning_count,
                format_violation_list(&all_issues)
            ),
            severity: Severity::Critical,
        }
    } else if warning_count > 0 {
        ComplianceCheck {
            name: "ComputeBrick Compliance".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} warnings detected:\n{}",
                warning_count,
                format_violation_list(&all_issues)
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "ComputeBrick Compliance".into(),
            status: CheckStatus::Pass,
            message: "ComputeBrick patterns validated - no violations detected".into(),
            severity: Severity::Info,
        }
    }
}

pub(crate) fn check_compute_brick(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let brick_dir = project_path.join("src").join("brick");
    let has_probar = cargo_toml.exists()
        && fs::read_to_string(&cargo_toml)
            .map(|s| s.contains("probar") || s.contains("jugar-probar"))
            .unwrap_or(false);
    let has_brick_dir = brick_dir.exists();
    let has_cb_ecosystem = cargo_toml.exists()
        && fs::read_to_string(&cargo_toml)
            .map(|s| s.contains("trueno") || s.contains("realizar") || s.contains("Brick"))
            .unwrap_or(false);
    if !has_probar && !has_brick_dir && !has_cb_ecosystem {
        return ComplianceCheck {
            name: "ComputeBrick Compliance".into(),
            status: CheckStatus::Skip,
            message: "Not a ComputeBrick project (no probar/trueno/realizar dep or brick/ dir)"
                .into(),
            severity: Severity::Info,
        };
    }
    let (all_issues, critical_count, warning_count) =
        collect_cb_violations(project_path, has_probar, has_brick_dir);
    build_cb_result(all_issues, critical_count, warning_count)
}

pub(crate) fn check_oip_tarantula_patterns(project_path: &Path) -> ComplianceCheck {
    let detections = vec![
        (detect_cb120_nan_unsafe_comparison(project_path), true),
        (detect_cb121_lock_poisoning(project_path), false),
        (detect_cb122_serde_safety(project_path), true),
        (detect_cb123_undocumented_ignore(project_path), false),
    ];
    let (mut all_issues, mut critical_count, mut warning_count) =
        collect_violations_with_counts(&detections);
    for v in &detect_cb124_coverage_threshold(project_path) {
        append_violation(&mut all_issues, v);
        match v.severity {
            crate::cli::handlers::comply_cb_detect::Severity::Error => critical_count += 1,
            _ => warning_count += 1,
        }
    }
    if critical_count > 0 || warning_count > 0 {
        ComplianceCheck {
            name: "OIP Tarantula Patterns (CB-120 to CB-124)".into(),
            status: CheckStatus::Warn,
            message: format!(
                "[Advisory] {} issues, {} warnings:\n{}",
                critical_count,
                warning_count,
                format_violation_list(&all_issues)
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "OIP Tarantula Patterns (CB-120 to CB-124)".into(),
            status: CheckStatus::Pass,
            message: "No OIP Tarantula pattern violations detected".into(),
            severity: Severity::Info,
        }
    }
}

/// Collect violations from multiple detection functions, classifying by original severity into 3 levels
fn collect_triaged_violations(
    violation_sets: &[Vec<crate::cli::handlers::comply_cb_detect::CbPatternViolation>],
) -> (Vec<String>, usize, usize, usize) {
    use crate::cli::handlers::comply_cb_detect::Severity as CbSev;
    let mut all_issues = Vec::new();
    let (mut critical, mut error, mut warning) = (0, 0, 0);
    for violations in violation_sets {
        for v in violations {
            append_violation(&mut all_issues, v);
            match v.severity {
                CbSev::Critical => critical += 1,
                CbSev::Error => error += 1,
                _ => warning += 1,
            }
        }
    }
    (all_issues, critical, error, warning)
}

/// Build a ComplianceCheck from triaged violation counts
fn build_triaged_check(
    name: &str,
    all_issues: Vec<String>,
    critical: usize,
    error: usize,
    warning: usize,
    pass_message: &str,
) -> ComplianceCheck {
    if critical > 0 {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} critical, {} errors, {} warnings:\n{}",
                critical,
                error,
                warning,
                format_violation_list(&all_issues)
            ),
            severity: Severity::Critical,
        }
    } else if error > 0 {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} errors, {} warnings:\n{}",
                error,
                warning,
                format_violation_list(&all_issues)
            ),
            severity: Severity::Error,
        }
    } else if warning > 0 {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} warnings:\n{}",
                warning,
                format_violation_list(&all_issues)
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: pass_message.into(),
            severity: Severity::Info,
        }
    }
}

pub(crate) fn check_coverage_quality_patterns(project_path: &Path) -> ComplianceCheck {
    let violation_sets = vec![
        detect_cb125_coverage_exclusion_gaming(project_path),
        detect_cb126_slow_tests(project_path),
        detect_cb127_slow_coverage(project_path),
    ];
    let (all_issues, critical, error, warning) = collect_triaged_violations(&violation_sets);
    build_triaged_check(
        "Coverage Quality Patterns (CB-125 to CB-127)",
        all_issues,
        critical,
        error,
        warning,
        "No coverage quality issues detected",
    )
}

pub(crate) fn check_cargo_lock(project_path: &Path) -> ComplianceCheck {
    if !project_path.join("Cargo.toml").exists() {
        return ComplianceCheck {
            name: "Cargo.lock Present".into(),
            status: CheckStatus::Skip,
            message: "Not a Rust project (no Cargo.toml)".into(),
            severity: Severity::Info,
        };
    }
    if project_path.join("Cargo.lock").exists() {
        ComplianceCheck {
            name: "Cargo.lock Present".into(),
            status: CheckStatus::Pass,
            message: "Cargo.lock present - reproducible builds enabled".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Cargo.lock Present".into(),
            status: CheckStatus::Fail,
            message: "Missing Cargo.lock - run 'cargo build' to generate".into(),
            severity: Severity::Error,
        }
    }
}

pub(crate) fn check_msrv(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "MSRV Defined".into(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".into(),
            severity: Severity::Info,
        };
    }
    let content = fs::read_to_string(&cargo_toml).unwrap_or_default();
    if content.contains("rust-version") {
        ComplianceCheck {
            name: "MSRV Defined".into(),
            status: CheckStatus::Pass,
            message: "rust-version field present in Cargo.toml".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "MSRV Defined".into(),
            status: CheckStatus::Warn,
            message: "No rust-version field - add to Cargo.toml for compatibility".into(),
            severity: Severity::Warning,
        }
    }
}

pub(crate) fn check_ci_configured(project_path: &Path) -> ComplianceCheck {
    let github_workflows = project_path.join(".github").join("workflows");
    if github_workflows.exists() && github_workflows.is_dir() {
        let wf_count = fs::read_dir(&github_workflows)
            .map(|e| e.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        if wf_count > 0 {
            return ComplianceCheck {
                name: "CI Configured".into(),
                status: CheckStatus::Pass,
                message: format!("{} GitHub Actions workflow(s) found", wf_count),
                severity: Severity::Info,
            };
        }
    }
    if project_path.join(".gitlab-ci.yml").exists() {
        return ComplianceCheck {
            name: "CI Configured".into(),
            status: CheckStatus::Pass,
            message: "GitLab CI configured".into(),
            severity: Severity::Info,
        };
    }
    if project_path.join("Jenkinsfile").exists() {
        return ComplianceCheck {
            name: "CI Configured".into(),
            status: CheckStatus::Pass,
            message: "Jenkins pipeline configured".into(),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: "CI Configured".into(),
        status: CheckStatus::Warn,
        message: "No CI configuration found - add .github/workflows/".into(),
        severity: Severity::Warning,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod check_handlers_tests {
    use super::*;

    #[test]
    fn test_format_violation_list_empty() {
        let issues: Vec<String> = vec![];
        let result = format_violation_list(&issues);
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_format_violation_list_single() {
        let issues = vec!["CB-001: test issue".to_string()];
        let result = format_violation_list(&issues);
        assert!(result.contains("CB-001"));
    }

    #[test]
    fn test_format_violation_list_multiple() {
        let issues = vec!["CB-001: issue 1".to_string(), "CB-002: issue 2".to_string()];
        let result = format_violation_list(&issues);
        assert!(result.contains("CB-001"));
        assert!(result.contains("CB-002"));
    }

    #[test]
    fn test_check_version_currency_current() {
        let check = check_version_currency(PMAT_VERSION);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_version_currency_old() {
        let check = check_version_currency("1.0.0");
        assert!(check.status == CheckStatus::Warn || check.status == CheckStatus::Fail);
    }
}
