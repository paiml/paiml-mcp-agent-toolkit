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
    // A `debug_assert!` is not a guard in a release build: `comply check -p
    // /does/not/exist` sailed past it, and the first thing that touched the
    // filesystem was `load_or_create_project_config`, which tried to
    // `create_dir_all("/does/not/exist/.pmat")`. That surfaced as "Permission
    // denied (os error 13)" and exit 126 — an error about the wrong thing,
    // pointing at a permissions problem the user does not have. Reject the
    // missing path by name, like every other analysis handler does.
    if !project_path.exists() {
        anyhow::bail!("Path not found: {}", project_path.display());
    }

    // A path that exists is not yet a project. An EMPTY directory came back
    // "Project Version: 3.30.0 / Versions Behind: 0 / Status: COMPLIANT",
    // "154 checks (0 fail)", exit 0 — the same headline check count this
    // repository reports, and a better verdict than this repository gets. Every
    // one of those 154 checks had skipped for want of anything to look at, and
    // `load_or_create_project_config` had meanwhile WRITTEN a `.pmat/` directory
    // into the empty tree to invent the version it then reported as current.
    // `project-diag` already refuses this input by name; comply must not answer
    // a compliance question about a path with nothing in it to comply.
    if let Some(reason) = no_project_here(project_path) {
        anyhow::bail!(
            "No project found at {}: {reason} — comply has nothing to check here, and an unmeasured project is not a compliant one",
            project_path.display()
        );
    }

    crate::status_eprintln!("Checking PMAT compliance for {}", project_path.display());

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

/// Why this path holds no project, or `None` if it plausibly does.
///
/// Deliberately permissive — one manifest, one VCS directory or one source file
/// anywhere in the tree is enough. The case being refused is the one where a
/// verdict is pure fabrication: nothing to read, so nothing measured.
fn no_project_here(project_path: &Path) -> Option<String> {
    if project_path.is_file() {
        return None;
    }

    const MANIFESTS: [&str; 12] = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "setup.py",
        "go.mod",
        "build.sbt",
        "pom.xml",
        "build.gradle",
        "Makefile",
        "lakefile.lean",
        ".pmat.yaml",
        ".git",
    ];
    if MANIFESTS.iter().any(|m| project_path.join(m).exists()) {
        return None;
    }

    // No manifest is not decisive on its own: comply checks Markdown, shell and
    // SQL too. Any file the tool could read counts, `.pmat/` excepted — comply
    // writes that itself, so accepting it would let the command manufacture its
    // own evidence of a project on the second run.
    let mut saw_file = false;
    if let Ok(entries) = std::fs::read_dir(project_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name == ".pmat" {
                continue;
            }
            saw_file = true;
            break;
        }
    }
    if saw_file {
        return None;
    }

    Some(
        "no manifest (Cargo.toml, package.json, pyproject.toml, go.mod, …), no .git, and no files"
            .to_string(),
    )
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
    crate::status_eprintln!("  Using configuration from .pmat.yaml");
    if !comply_config.suppressions.is_empty() {
        crate::status_eprintln!(
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

/// A named compliance-check group and the thunk that produces its checks.
type CheckGroup<'a> = (
    &'static str,
    Box<dyn Fn() -> Vec<ComplianceCheck> + Send + Sync + 'a>,
);

fn build_all_compliance_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    project_version: &str,
) -> Vec<ComplianceCheck> {
    // Data-driven group list: each entry is independent and side-effect-free
    // w.r.t. the others, which lets `run_check_groups` both report live
    // per-group progress AND run the groups concurrently.
    let groups: Vec<CheckGroup> = vec![
        (
            "foundation",
            Box::new(move || build_foundation_checks(project_path, comply_config, project_version)),
        ),
        (
            "language",
            Box::new(move || build_language_best_practices(project_path, comply_config)),
        ),
        (
            "custom-score",
            Box::new(move || build_custom_score_checks(project_path, comply_config)),
        ),
        (
            "provable-contracts",
            Box::new(move || build_provable_contract_checks(project_path, comply_config)),
        ),
        (
            "contract-surfaces",
            Box::new(move || build_contract_surface_checks(project_path, comply_config)),
        ),
        (
            "agent-contracts",
            Box::new(move || build_agent_contract_checks(project_path, comply_config)),
        ),
        (
            "commit-enforcement",
            Box::new(move || build_commit_enforcement_checks(project_path, comply_config)),
        ),
        (
            "binding-scope",
            Box::new(move || build_binding_scope_checks(project_path, comply_config)),
        ),
        (
            "work-ladder",
            Box::new(move || build_work_ladder_checks(project_path, comply_config)),
        ),
        (
            "falsification",
            Box::new(move || build_falsification_unification_checks(project_path, comply_config)),
        ),
        (
            "codegen",
            Box::new(move || build_codegen_checks(project_path, comply_config)),
        ),
        (
            "cot-proof",
            Box::new(move || build_cot_proof_checks(project_path, comply_config)),
        ),
        (
            "macs",
            Box::new(move || build_macs_checks(project_path, comply_config)),
        ),
    ];
    run_check_groups(groups)
}

/// Run each check group concurrently, emitting a live per-group status line
/// (name · count · fail/warn tally · elapsed) to stderr as each completes, plus
/// a final summary. Replaces the previous silent sequential scan: a long
/// compliance run is now observable (you can see which group is slow) and
/// wall-time drops from Σ(group) to max(group). Groups are independent and
/// each reads its own inputs, so concurrency is safe.
fn run_check_groups(groups: Vec<CheckGroup>) -> Vec<ComplianceCheck> {
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let total = groups.len();
    let overall = std::time::Instant::now();
    let done = AtomicUsize::new(0);

    // Each group is timed and reports the moment it finishes (completion order).
    let mut grouped: Vec<(usize, Vec<ComplianceCheck>)> = groups
        .into_par_iter()
        .enumerate()
        .map(|(idx, (name, run))| {
            let start = std::time::Instant::now();
            let checks = run();
            let fails = checks
                .iter()
                .filter(|c| c.status == CheckStatus::Fail)
                .count();
            let warns = checks
                .iter()
                .filter(|c| c.status == CheckStatus::Warn)
                .count();
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            crate::status_eprintln!(
                "  [{n:>2}/{total}] {name:<19} {:>3} checks · {fails} fail · {warns} warn · {:.1}s",
                checks.len(),
                start.elapsed().as_secs_f64()
            );
            (idx, checks)
        })
        .collect();

    // Restore declaration order so report output is deterministic.
    grouped.sort_by_key(|(idx, _)| *idx);
    let all: Vec<ComplianceCheck> = grouped.into_iter().flat_map(|(_, c)| c).collect();

    let fails = all.iter().filter(|c| c.status == CheckStatus::Fail).count();
    crate::status_eprintln!(
        "  ── comply: {} checks in {:.1}s ({fails} fail) ──",
        all.len(),
        overall.elapsed().as_secs_f64()
    );
    all
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

/// `-f sarif` emits SARIF — pmat's own compliance findings, always.
///
/// This used to try the external `pv` binary and, when that was unavailable
/// (which is the normal case: it needs a sibling contracts directory AND `pv`
/// on PATH), silently print pmat's plain JSON report instead. The document
/// carried no `$schema`, no `version` and no `runs[]`, so GitHub's
/// upload-sarif action rejected it — and it was byte-identical to `-f json`
/// apart from the timestamp, so nothing in the output said the fallback had
/// happened. Even on the happy path the SARIF described pv's YAML contract
/// lint and never the 154 compliance checks the command was run for; pv's runs
/// are now MERGED into pmat's document rather than replacing it.
fn output_sarif_or_fallback(report: &ComplianceReport, project_path: &Path) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let mut sarif = build_sarif(report, project_path);
    if let Some(pv_runs) = pv_lint_runs(project_path) {
        if let Some(runs) = sarif["runs"].as_array_mut() {
            runs.extend(pv_runs);
        }
    }
    println!("{}", serde_json::to_string_pretty(&sarif)?);
    Ok(())
}

/// The SARIF rule id for a check: its `CB-nnn` clause when it has one, else a
/// slug of its name, so a consumer can suppress a single finding.
fn sarif_rule_id(name: &str) -> String {
    for token in name.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        let upper = token.to_ascii_uppercase();
        if upper.starts_with("CB-") && upper[3..].chars().all(|c| c.is_ascii_digit()) {
            return upper;
        }
    }
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("pmat/{}", slug.trim_matches('-'))
}

/// SARIF level for a check result. Skipped and passing checks produce no
/// result at all — a SARIF run reports findings, not a roll call.
fn sarif_level(check: &ComplianceCheck) -> Option<&'static str> {
    match check.status {
        CheckStatus::Fail => Some(match check.severity {
            Severity::Critical | Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "note",
        }),
        CheckStatus::Warn => Some("warning"),
        CheckStatus::Pass | CheckStatus::Skip => None,
    }
}

/// A SARIF 2.1.0 document for pmat's own compliance report.
fn build_sarif(report: &ComplianceReport, project_path: &Path) -> serde_json::Value {
    let uri = project_path.display().to_string();

    let rules: Vec<serde_json::Value> = report
        .checks
        .iter()
        .map(|check| {
            serde_json::json!({
                "id": sarif_rule_id(&check.name),
                "name": check.name,
                "shortDescription": { "text": check.name },
            })
        })
        .collect();

    let results: Vec<serde_json::Value> = report
        .checks
        .iter()
        .filter_map(|check| {
            let level = sarif_level(check)?;
            Some(serde_json::json!({
                "ruleId": sarif_rule_id(&check.name),
                "level": level,
                "message": { "text": format!("{}: {}", check.name, check.message) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": uri }
                    }
                }],
            }))
        })
        .collect();

    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": PMAT_VERSION,
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules,
                }
            },
            "results": results,
        }]
    })
}

/// pv's SARIF `runs[]`, when the external contract linter is available and
/// produced a SARIF document. Anything else is reported on stderr rather than
/// swallowed.
fn pv_lint_runs(project_path: &Path) -> Option<Vec<serde_json::Value>> {
    let raw = try_pv_lint_sarif(project_path)?;
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(value) => value.get("runs").and_then(|runs| runs.as_array()).cloned(),
        Err(e) => {
            eprintln!("Warning: `pv lint --format sarif` output was not JSON ({e}); reporting pmat's compliance checks only");
            None
        }
    }
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

    /// `comply check -f sarif` used to print pmat's plain JSON report whenever
    /// the external `pv` linter was unavailable: no `$schema`, no `version`, no
    /// `runs[]` — a document GitHub's upload-sarif action rejects, and
    /// byte-identical to `-f json` apart from the timestamp.
    #[test]
    fn sarif_is_sarif_and_carries_pmats_own_checks() {
        let report = ComplianceReport {
            project_version: "1.0".into(),
            current_version: "1.0".into(),
            is_compliant: false,
            versions_behind: 0,
            checks: vec![
                check("Version Currency", CheckStatus::Pass),
                ComplianceCheck {
                    name: "CB-030: O(1) Hooks".into(),
                    status: CheckStatus::Fail,
                    message: "hook missing".into(),
                    severity: Severity::Error,
                },
                ComplianceCheck {
                    name: "Quality Thresholds".into(),
                    status: CheckStatus::Warn,
                    message: "close to the limit".into(),
                    severity: Severity::Warning,
                },
                check("Disabled Check", CheckStatus::Skip),
            ],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };

        let sarif = build_sarif(&report, Path::new("."));
        assert_eq!(sarif["version"], "2.1.0");
        assert!(sarif["$schema"].is_string(), "SARIF needs a $schema");
        let runs = sarif["runs"].as_array().expect("SARIF needs runs[]");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0]["tool"]["driver"]["name"], "pmat");

        // One result per Fail/Warn, none for Pass/Skip.
        let results = runs[0]["results"].as_array().expect("results[]");
        assert_eq!(results.len(), 2, "{results:#?}");
        assert_eq!(results[0]["ruleId"], "CB-030");
        assert_eq!(results[0]["level"], "error");
        assert!(results[0]["message"]["text"]
            .as_str()
            .expect("message")
            .contains("hook missing"));
        assert_eq!(results[1]["ruleId"], "pmat/quality-thresholds");
        assert_eq!(results[1]["level"], "warning");

        // The document must NOT be the plain compliance report.
        assert!(sarif.get("checks").is_none());
        assert!(sarif.get("is_compliant").is_none());
    }

    #[test]
    fn sarif_rule_ids_prefer_the_cb_clause() {
        assert_eq!(sarif_rule_id("CB-1204: Verification Ladder"), "CB-1204");
        assert_eq!(
            sarif_rule_id("Cargo.lock Present"),
            "pmat/cargo-lock-present"
        );
        assert_eq!(
            sarif_rule_id("OIP Tarantula Patterns (CB-120 to CB-124)"),
            "CB-120"
        );
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

include!("check_path_guard_tests.rs");
include!("check_empty_project_guard_tests.rs");
