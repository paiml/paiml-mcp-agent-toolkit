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
use crate::models::comply_config::{ComplyThresholds, PmatYamlConfig};
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn handle_check(
    project_path: &Path,
    strict: bool,
    failures_only: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );

    eprintln!("Checking PMAT compliance for {}", project_path.display());

    let yaml_config = PmatYamlConfig::load(project_path).unwrap_or_default();
    let comply_config = &yaml_config.comply;
    announce_suppressions(project_path, comply_config);

    let config = load_or_create_project_config(project_path)?;
    let project_version = &config.pmat.version;

    let checks = build_all_compliance_checks(project_path, comply_config, project_version);
    let report = build_compliance_report(checks, project_version, failures_only);

    output_compliance_report(&report, format, project_path)?;
    let _ = update_last_check_timestamp(project_path);

    apply_exit_policy(&report, strict)
}

/// One-shot log summarizing the active `.pmat.yaml` configuration.
fn announce_suppressions(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) {
    let config_path = project_path.join(".pmat.yaml");
    if !config_path.exists() {
        return;
    }
    eprintln!("  Using configuration from .pmat.yaml");
    if !comply_config.suppressions.is_empty() {
        eprintln!(
            "  {} suppression rule(s) loaded",
            comply_config.suppressions.len()
        );
    }
}

/// Apply the report's exit policy: code 1 on failures, code 2 on strict warnings-only.
fn apply_exit_policy(report: &ComplianceReport, strict: bool) -> Result<()> {
    let failures = report
        .checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let warnings = report
        .checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();
    if !report.is_compliant {
        std::process::exit(1);
    }
    if strict && warnings > 0 && failures == 0 {
        std::process::exit(2);
    }
    Ok(())
}

fn build_all_compliance_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    project_version: &str,
) -> Vec<ComplianceCheck> {
    let mut checks = build_foundation_checks(project_path, comply_config, project_version);
    checks.extend(build_language_best_practices(project_path, comply_config));
    checks.extend(build_custom_score_checks(project_path, comply_config));
    checks.extend(build_provable_contract_checks(project_path, comply_config));
    checks.extend(build_contract_surface_checks(project_path, comply_config));
    checks.extend(build_agent_contract_checks(project_path, comply_config));
    checks.extend(build_commit_enforcement_checks(project_path, comply_config));
    checks.extend(build_binding_scope_checks(project_path, comply_config));
    checks.extend(build_work_ladder_checks(project_path, comply_config));
    checks.extend(build_falsification_unification_checks(
        project_path,
        comply_config,
    ));
    checks.extend(build_codegen_checks(project_path, comply_config));
    checks.extend(build_cot_proof_checks(project_path, comply_config));
    checks.extend(build_macs_checks(project_path, comply_config));
    checks
}

fn build_compliance_report(
    checks: Vec<ComplianceCheck>,
    project_version: &str,
    failures_only: bool,
) -> ComplianceReport {
    debug_assert!(
        !project_version.is_empty(),
        "project_version must not be empty"
    );
    let failures = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
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

    ComplianceReport {
        project_version: project_version.to_string(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: failures == 0,
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
    }
}

fn output_compliance_report(
    report: &ComplianceReport,
    format: ComplyOutputFormat,
    project_path: &Path,
) -> Result<()> {
    match format {
        ComplyOutputFormat::Text => print_compliance_text(report),
        ComplyOutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        ComplyOutputFormat::Markdown => print_compliance_markdown(report),
        ComplyOutputFormat::Sarif => output_sarif_or_fallback(report, project_path)?,
    }
    Ok(())
}

fn output_sarif_or_fallback(report: &ComplianceReport, project_path: &Path) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if let Some(sarif) = try_pv_lint_sarif(project_path) {
        println!("{sarif}");
        return Ok(());
    }
    // Fallback: JSON output
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}

fn try_pv_lint_sarif(project_path: &Path) -> Option<String> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let contracts_dir = resolve_contracts_dir(project_path)?;
    let output = std::process::Command::new("pv")
        .args([
            "lint",
            &contracts_dir.display().to_string(),
            "--format",
            "sarif",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    let sarif = String::from_utf8(output.stdout).ok()?;
    if sarif.is_empty() {
        None
    } else {
        Some(sarif)
    }
}

#[cfg(test)]
mod build_compliance_report_tests {
    use super::*;

    fn check(name: &str, status: CheckStatus) -> ComplianceCheck {
        ComplianceCheck {
            name: name.into(),
            status,
            message: format!("msg for {name}"),
            severity: Severity::Info,
        }
    }

    #[test]
    fn test_compliant_when_no_failures() {
        let checks = vec![check("a", CheckStatus::Pass), check("b", CheckStatus::Warn)];
        let report = build_compliance_report(checks, "1.0.0", false);
        assert!(report.is_compliant);
        assert_eq!(report.checks.len(), 2);
    }

    #[test]
    fn test_non_compliant_when_any_fail() {
        let checks = vec![check("a", CheckStatus::Pass), check("b", CheckStatus::Fail)];
        let report = build_compliance_report(checks, "1.0.0", false);
        assert!(!report.is_compliant);
    }

    #[test]
    fn test_failures_only_filter_drops_non_fail_checks() {
        let checks = vec![
            check("p", CheckStatus::Pass),
            check("w", CheckStatus::Warn),
            check("s", CheckStatus::Skip),
            check("f", CheckStatus::Fail),
        ];
        let report = build_compliance_report(checks, "1.0.0", true);
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.checks[0].name, "f");
    }

    #[test]
    fn test_failures_only_false_keeps_all_checks() {
        let checks = vec![
            check("p", CheckStatus::Pass),
            check("w", CheckStatus::Warn),
            check("f", CheckStatus::Fail),
        ];
        let report = build_compliance_report(checks, "1.0.0", false);
        assert_eq!(report.checks.len(), 3);
    }

    #[test]
    fn test_project_version_propagates() {
        let report = build_compliance_report(vec![], "2.5.0", false);
        assert_eq!(report.project_version, "2.5.0");
        assert!(!report.current_version.is_empty());
    }

    #[test]
    fn test_empty_checks_compliant() {
        // No failures = compliant by definition
        let report = build_compliance_report(vec![], "1.0.0", false);
        assert!(report.is_compliant);
        assert_eq!(report.checks.len(), 0);
    }

    #[test]
    fn test_apply_exit_policy_returns_ok_when_compliant_no_warnings() {
        let report = ComplianceReport {
            project_version: "1.0".into(),
            current_version: "1.0".into(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        // strict=false, no warnings → Ok
        assert!(apply_exit_policy(&report, false).is_ok());
    }

    #[test]
    fn test_apply_exit_policy_returns_ok_when_strict_but_no_warnings() {
        let report = ComplianceReport {
            project_version: "1.0".into(),
            current_version: "1.0".into(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![check("p", CheckStatus::Pass)],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        // is_compliant=true, no warnings even with strict → Ok (no exit)
        assert!(apply_exit_policy(&report, true).is_ok());
    }
}

// Provable-contracts enforcement helpers (shared by CB-1201 through CB-1209)
include!("check_pv_enforcement_helpers.rs");
// Provable-contracts enforcement checks (CB-1201, CB-1203)
include!("check_pv_enforcement.rs");
// Provable-contracts verification ladder (CB-1204 through CB-1207)
include!("check_pv_verification_ladder.rs");
// Provable-contracts quality gate (CB-1202, CB-1208, CB-1209)
include!("check_pv_quality_gate.rs");
include!("check_pv_quality.rs");
include!("check_contract_surfaces.rs");
include!("check_agent_contracts.rs");
include!("check_agent_iteration.rs");
include!("check_agent_autonomous.rs");
include!("check_commit_enforcement.rs");
include!("check_commit_enforcement_p2.rs");
include!("check_commit_enforcement_p3.rs");
include!("check_commit_enforcement_p4.rs");
include!("check_commit_enforcement_p5.rs");
include!("check_commit_enforcement_p6.rs");
include!("check_commit_enforcement_p7.rs");
include!("check_commit_enforcement_p8.rs");
include!("check_commit_enforcement_p9.rs");
include!("check_commit_enforcement_p10.rs");

// Split into submodule files for file_health (CB-040) compliance
include!("check_builders_foundation.rs");
include!("check_builders_contracts.rs");
include!("check_builders_commits.rs");
include!("check_builders_work.rs");
include!("check_individual_basic.rs");
include!("check_individual_cb.rs");
include!("check_individual_ci.rs");

include!("check_handlers_tests_inline.rs");
include!("check_pv_enforcement_helpers_tests.rs");
