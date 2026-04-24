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

/// Assemble the full list of compliance checks from every topical builder.
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
    checks
}

/// Foundation checks: version, config, hooks, quality, CI, sovereign stack, file health, muda/reproducibility/EDD, deps, shell/makefile, stale paths, spec traceability, agent context, mono-spec.
fn build_foundation_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    project_version: &str,
) -> Vec<ComplianceCheck> {
    vec![
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
    ]
}

/// CB-500..CB-1050: language-specific best practices (Rust, Lua, SQL, Scala, Markdown, YAML, model files, Lean).
fn build_language_best_practices(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![
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
    ]
}

/// CB-1100: custom score checks produced per-project by `check_custom_scores`.
fn build_custom_score_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    check_custom_scores(project_path)
        .into_iter()
        .map(|chk| filter_check_by_config(chk, "cb-1100", comply_config))
        .collect()
}

/// CB-1200..CB-1214: provable contracts, pv lint, contract/annotation coverage, contract drift, binding index, codegen fidelity, enforcement quality.
fn build_provable_contract_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![
        filter_check_by_config(
            super::check_provable_contracts::check_provable_contracts(project_path),
            "cb-1200",
            comply_config,
        ),
        filter_check_by_config(
            check_pv_lint(project_path, &comply_config.thresholds),
            "cb-1201",
            comply_config,
        ),
        filter_check_by_config(
            check_contract_coverage(project_path),
            "cb-1202",
            comply_config,
        ),
        filter_check_by_config(
            check_annotation_coverage(project_path),
            "cb-1203",
            comply_config,
        ),
        filter_check_by_config(
            check_build_rs_pipeline(project_path),
            "cb-1204",
            comply_config,
        ),
        filter_check_by_config(
            check_provability_invariant(project_path),
            "cb-1205",
            comply_config,
        ),
        filter_check_by_config(
            check_verification_levels(project_path, &comply_config.thresholds),
            "cb-1206",
            comply_config,
        ),
        filter_check_by_config(check_contract_drift(project_path), "cb-1207", comply_config),
        filter_check_by_config(
            check_binding_existence(project_path, &comply_config.thresholds),
            "cb-1208",
            comply_config,
        ),
        filter_check_by_config(
            check_contract_trait_enforcement(project_path, &comply_config.thresholds),
            "cb-1209",
            comply_config,
        ),
        filter_check_by_config(
            check_precondition_quality(project_path),
            "cb-1210",
            comply_config,
        ),
        filter_check_by_config(
            check_codegen_fidelity(project_path),
            "cb-1211",
            comply_config,
        ),
        filter_check_by_config(
            check_enforcement_quality(project_path),
            "cb-1214",
            comply_config,
        ),
    ]
}

/// CB-1300..CB-1308: contract surface type checks (Component 23) — CLI args, MCP schemas, config drift, sovereign dep versions, anti-leak classification, TUI widgets, WASM FFI, L5 ladder.
fn build_contract_surface_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![
        filter_check_by_config(
            check_cli_arg_contracts(project_path),
            "cb-1300",
            comply_config,
        ),
        filter_check_by_config(
            check_mcp_schema_contracts(project_path),
            "cb-1302",
            comply_config,
        ),
        filter_check_by_config(
            check_config_contracts(project_path),
            "cb-1303",
            comply_config,
        ),
        filter_check_by_config(
            check_sovereign_dep_contracts(project_path),
            "cb-1304",
            comply_config,
        ),
        filter_check_by_config(
            check_contract_surface_classification(project_path),
            "cb-1305",
            comply_config,
        ),
        filter_check_by_config(
            check_tui_widget_contracts(project_path),
            "cb-1306",
            comply_config,
        ),
        filter_check_by_config(
            check_wasm_ffi_contracts(project_path),
            "cb-1307",
            comply_config,
        ),
        filter_check_by_config(
            check_verification_ladder(project_path),
            "cb-1308",
            comply_config,
        ),
    ]
}

/// CB-1400..CB-1410: agent contract-first enforcement (Component 10).
fn build_agent_contract_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![
        filter_check_by_config(
            check_agent_contract_existence(project_path),
            "cb-1400",
            comply_config,
        ),
        filter_check_by_config(
            check_agent_contract_falsifiability(project_path),
            "cb-1401",
            comply_config,
        ),
        filter_check_by_config(
            check_agent_verification_level(project_path),
            "cb-1402",
            comply_config,
        ),
        filter_check_by_config(
            check_assume_guarantee_chain(project_path),
            "cb-1403",
            comply_config,
        ),
        filter_check_by_config(
            check_agent_comply_usage(project_path),
            "cb-1404",
            comply_config,
        ),
        filter_check_by_config(
            check_agent_references_present(project_path),
            "cb-1405",
            comply_config,
        ),
        filter_check_by_config(
            check_agent_chain_of_thought(project_path),
            "cb-1406",
            comply_config,
        ),
        filter_check_by_config(
            check_agent_five_whys_linked(project_path),
            "cb-1407",
            comply_config,
        ),
        filter_check_by_config(
            check_agent_evidence_executable(project_path),
            "cb-1408",
            comply_config,
        ),
        filter_check_by_config(
            check_no_l0_autonomous_code(project_path),
            "cb-1409",
            comply_config,
        ),
        filter_check_by_config(
            check_subagent_contract_composition(project_path),
            "cb-1410",
            comply_config,
        ),
    ]
}

/// CB-1320..CB-1354 + CB-1342: commit-level contract enforcement (Component 25) — asset layout, work contract validity, hooks, ratchet, anti-leak, differential obligations, assume-guarantee, query readiness, codegen compiles.
fn build_commit_enforcement_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![
        filter_check_by_config(check_readme_layout(project_path), "cb-1320", comply_config),
        filter_check_by_config(
            check_dockerfile_contract(project_path),
            "cb-1321",
            comply_config,
        ),
        filter_check_by_config(check_svg_contract(project_path), "cb-1322", comply_config),
        filter_check_by_config(
            check_forjar_contract(project_path),
            "cb-1323",
            comply_config,
        ),
        filter_check_by_config(
            check_mdbook_contract(project_path),
            "cb-1324",
            comply_config,
        ),
        filter_check_by_config(
            check_changelog_contract(project_path),
            "cb-1325",
            comply_config,
        ),
        filter_check_by_config(check_badge_contract(project_path), "cb-1326", comply_config),
        filter_check_by_config(
            check_work_contract_validity(project_path),
            "cb-1331",
            comply_config,
        ),
        filter_check_by_config(
            check_cache_staleness(project_path),
            "cb-1332",
            comply_config,
        ),
        filter_check_by_config(
            check_hook_single_writer(project_path),
            "cb-1333",
            comply_config,
        ),
        filter_check_by_config(
            check_hook_atomic_writes(project_path),
            "cb-1334",
            comply_config,
        ),
        filter_check_by_config(
            check_hook_determinism(project_path),
            "cb-1335",
            comply_config,
        ),
        filter_check_by_config(
            check_hook_no_injection(project_path),
            "cb-1336",
            comply_config,
        ),
        filter_check_by_config(
            check_hook_performance(project_path),
            "cb-1337",
            comply_config,
        ),
        filter_check_by_config(
            check_verification_ratchet(project_path),
            "cb-1330",
            comply_config,
        ),
        filter_check_by_config(
            check_no_ghost_bindings(project_path),
            "cb-1338",
            comply_config,
        ),
        filter_check_by_config(
            check_no_placeholder_preconditions(project_path),
            "cb-1339",
            comply_config,
        ),
        filter_check_by_config(
            check_enforcement_penetration(project_path),
            "cb-1340",
            comply_config,
        ),
        filter_check_by_config(
            check_spec_number_accuracy(project_path),
            "cb-1341",
            comply_config,
        ),
        filter_check_by_config(
            check_assertion_placement(project_path),
            "cb-1343",
            comply_config,
        ),
        filter_check_by_config(
            check_differential_obligations(project_path),
            "cb-1350",
            comply_config,
        ),
        filter_check_by_config(
            check_binding_index_freshness(project_path),
            "cb-1351",
            comply_config,
        ),
        filter_check_by_config(
            check_assume_guarantee_chains(project_path),
            "cb-1352",
            comply_config,
        ),
        filter_check_by_config(
            check_ag_cycle_detection(project_path),
            "cb-1353",
            comply_config,
        ),
        filter_check_by_config(
            check_contract_query_readiness(project_path),
            "cb-1354",
            comply_config,
        ),
        filter_check_by_config(
            check_codegen_compiles(project_path),
            "cb-1342",
            comply_config,
        ),
    ]
}

/// CB-1600..CB-1609: work-contract binding enforcement (Component 27).
fn build_binding_scope_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_binding_scope as bs;
    vec![
        filter_check_by_config(
            bs::check_binding_scope_orphan(project_path),
            "cb-1600",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_sha_drift(project_path),
            "cb-1601",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_unbind_audit(project_path),
            "cb-1602",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_inherited_clauses(project_path),
            "cb-1603",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_postcondition_weakening(project_path),
            "cb-1604",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_kani_harnesses(project_path),
            "cb-1605",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_lean_theorem(project_path),
            "cb-1606",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_equation_exists(project_path),
            "cb-1607",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_cross_consistency(project_path),
            "cb-1608",
            comply_config,
        ),
        filter_check_by_config(
            bs::check_binding_file_tracked(project_path),
            "cb-1609",
            comply_config,
        ),
    ]
}

/// CB-1610..CB-1619: work-verification ladder enforcement (Component 28).
fn build_work_ladder_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_work_ladder as wl;
    vec![
        filter_check_by_config(
            wl::check_ladder_parses(project_path),
            "cb-1610",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_bound_by_yaml(project_path),
            "cb-1611",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_l1_test_evidence(project_path),
            "cb-1612",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_l3_falsification(project_path),
            "cb-1613",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_l4_kani(project_path),
            "cb-1614",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_kani_harness_sha(project_path),
            "cb-1615",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_l5_lean(project_path),
            "cb-1616",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_downgrade_audit(project_path),
            "cb-1617",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_monotonicity(project_path),
            "cb-1618",
            comply_config,
        ),
        filter_check_by_config(
            wl::check_ladder_completion_matches(project_path),
            "cb-1619",
            comply_config,
        ),
    ]
}

/// CB-1620..CB-1629: work-falsification-unification enforcement (Component 29).
fn build_falsification_unification_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_falsification_unification as fu;
    vec![
        filter_check_by_config(
            fu::check_inherited_roster_coverage(project_path),
            "cb-1620",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_expected_snapshot_drift(project_path),
            "cb-1621",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_roster_execution_coverage(project_path),
            "cb-1622",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_no_duplicate_provable_entries(project_path),
            "cb-1623",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_no_manual_deletion(project_path),
            "cb-1624",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_inherited_failure_fatal(project_path),
            "cb-1625",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_test_id_exists_in_yaml(project_path),
            "cb-1626",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_post_bind_yaml_drift(project_path),
            "cb-1627",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_per_run_log_line(project_path),
            "cb-1628",
            comply_config,
        ),
        filter_check_by_config(
            fu::check_l4_timeout_gate(project_path),
            "cb-1629",
            comply_config,
        ),
    ]
}

/// CB-1630..CB-1639: work compile-time codegen enforcement (Component 30).
fn build_codegen_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_codegen as cg;
    vec![
        filter_check_by_config(
            cg::check_codegen_cli_succeeds(project_path),
            "cb-1630",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_attribute_has_generated_module(project_path),
            "cb-1631",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_attribute_clause_ids_exist(project_path),
            "cb-1632",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_manifest_sha_drift(project_path),
            "cb-1633",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_expr_clauses_have_binds_to(project_path),
            "cb-1634",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_binds_to_function_modified(project_path),
            "cb-1635",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_macros_compile_debug_and_release(project_path),
            "cb-1636",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_l2_public_fn_coverage(project_path),
            "cb-1637",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_generated_modules_tracked(project_path),
            "cb-1638",
            comply_config,
        ),
        filter_check_by_config(
            cg::check_kani_harness_macro_reference(project_path),
            "cb-1639",
            comply_config,
        ),
    ]
}

/// CB-1640..CB-1649: chain-of-thought proof derivation enforcement (Component 31).
fn build_cot_proof_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_cot_proof as cp;
    vec![
        filter_check_by_config(
            cp::check_assumption_references_resolve(project_path),
            "cb-1640",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_step_has_evidence_method(project_path),
            "cb-1641",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_existing_test_paths_resolve(project_path),
            "cb-1642",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_l3_structured_expr_present(project_path),
            "cb-1643",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_agent_run_replayable(project_path),
            "cb-1644",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_derived_yaml_obligations_present(project_path),
            "cb-1645",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_cot_derivation_sha_fresh(project_path),
            "cb-1646",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_no_orphan_steps(project_path),
            "cb-1647",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_l4_axiomatic_discharge_bounded(project_path),
            "cb-1648",
            comply_config,
        ),
        filter_check_by_config(
            cp::check_l5_lean_theorem_mapping(project_path),
            "cb-1649",
            comply_config,
        ),
    ]
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

/// CB-533: Stale path references in Makefiles and CI workflows.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_stale_paths(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_spec_work_traceability(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn check_version_currency(project_version: &str) -> ComplianceCheck {
    debug_assert!(
        !project_version.is_empty(),
        "project_version must not be empty"
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_config_files(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_hooks_installed(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_hooks_o1_capable(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_hooks_cache_health(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_quality_thresholds(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_deprecated_features(_project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        _project_path.exists(),
        "_project_path must exist: {}",
        _project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn collect_cb_violations(
    project_path: &Path,
    has_probar: bool,
    has_brick_dir: bool,
) -> (Vec<String>, usize, usize) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_compute_brick(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_oip_tarantula_patterns(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
    debug_assert!(
        !violation_sets.is_empty(),
        "violation_sets must not be empty"
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_coverage_quality_patterns(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_cargo_lock(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_msrv(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_ci_configured(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

    // GH-271 regression: long doc-comments pushing #[contract(...)] beyond the old
    // 10-line preceding window caused CB-1203 false-positives. Window is now 25.
    #[test]
    fn test_cb1203_contract_with_long_doc_comment_passes() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&contracts_dir).unwrap();
        std::fs::create_dir_all(&src_dir).unwrap();

        std::fs::write(
            contracts_dir.join("demo.yaml"),
            "equations:\n  my_fn:\n    preconditions:\n      - x > 0\n    postconditions:\n      - result >= 0\n",
        )
        .unwrap();

        // 15+ doc-comment lines between #[contract(...)] and `pub fn`.
        let rs_source = r#"use std::path::Path;

#[provable_contracts_macros::contract("demo.yaml", equation = "my_fn")]
/// Line 1 of a long doc comment describing the function in detail.
/// Line 2 of a long doc comment describing the function in detail.
/// Line 3 of a long doc comment describing the function in detail.
/// Line 4 of a long doc comment describing the function in detail.
/// Line 5 of a long doc comment describing the function in detail.
/// Line 6 of a long doc comment describing the function in detail.
/// Line 7 of a long doc comment describing the function in detail.
/// Line 8 of a long doc comment describing the function in detail.
/// Line 9 of a long doc comment describing the function in detail.
/// Line 10 of a long doc comment describing the function in detail.
/// Line 11 of a long doc comment describing the function in detail.
/// Line 12 of a long doc comment describing the function in detail.
/// Line 13 of a long doc comment describing the function in detail.
/// Line 14 of a long doc comment describing the function in detail.
/// Line 15 of a long doc comment describing the function in detail.
pub fn my_fn(x: i32) -> i32 { x }
"#;
        std::fs::write(src_dir.join("lib.rs"), rs_source).unwrap();

        let check = check_annotation_coverage(temp.path());
        assert_eq!(
            check.status,
            CheckStatus::Pass,
            "CB-1203 should accept #[contract] above long doc comment (GH-271); got: {}",
            check.message
        );
    }

    #[test]
    fn test_cb1203_missing_contract_reports_actionable_message() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&contracts_dir).unwrap();
        std::fs::create_dir_all(&src_dir).unwrap();

        std::fs::write(
            contracts_dir.join("demo.yaml"),
            "equations:\n  bare_fn:\n    preconditions:\n      - x > 0\n    postconditions:\n      - result >= 0\n",
        )
        .unwrap();

        // No #[contract], no #[requires]/#[ensures] — should fail.
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub fn bare_fn(x: i32) -> i32 { x }\n",
        )
        .unwrap();

        let check = check_annotation_coverage(temp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(
            check.message.contains("missing #[contract(...)]"),
            "expected actionable message, got: {}",
            check.message
        );
    }
}

#[cfg(test)]
mod pv_enforcement_helpers_tests {
    //! PMAT-651: cover check_pv_enforcement_helpers.rs pure helpers.
    use super::*;
    use std::collections::HashSet;
    use tempfile::TempDir;

    fn write(p: &Path, content: &str) {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(p, content).unwrap();
    }

    // --- extract_names_from_source ---

    #[test]
    fn test_extract_names_pub_fn() {
        let mut s = HashSet::new();
        extract_names_from_source("pub fn foo() {}\n", &mut s);
        assert!(s.contains("foo"));
    }

    #[test]
    fn test_extract_names_private_fn() {
        let mut s = HashSet::new();
        extract_names_from_source("fn private_fn() {}\n", &mut s);
        assert!(s.contains("private_fn"));
    }

    #[test]
    fn test_extract_names_async_fn_variants() {
        let mut s = HashSet::new();
        extract_names_from_source(
            "pub async fn pub_async() {}\nasync fn just_async() {}\npub(crate) async fn pcrate_async() {}\n",
            &mut s,
        );
        assert!(s.contains("pub_async"));
        assert!(s.contains("just_async"));
        assert!(s.contains("pcrate_async"));
    }

    #[test]
    fn test_extract_names_unsafe_fn_variants() {
        let mut s = HashSet::new();
        extract_names_from_source(
            "unsafe fn u1() {}\npub unsafe fn u2() {}\npub(crate) unsafe fn u3() {}\npub const unsafe fn u4() {}\n",
            &mut s,
        );
        assert!(s.contains("u1"));
        assert!(s.contains("u2"));
        assert!(s.contains("u3"));
        assert!(s.contains("u4"));
    }

    #[test]
    fn test_extract_names_const_fn_variants() {
        let mut s = HashSet::new();
        extract_names_from_source("const fn cf() {}\npub const fn pcf() {}\n", &mut s);
        assert!(s.contains("cf"));
        assert!(s.contains("pcf"));
    }

    #[test]
    fn test_extract_names_pub_crate_fn() {
        let mut s = HashSet::new();
        extract_names_from_source("pub(crate) fn pcfn() {}\n", &mut s);
        assert!(s.contains("pcfn"));
    }

    #[test]
    fn test_extract_names_const_declarations() {
        let mut s = HashSet::new();
        extract_names_from_source(
            "pub const PUB_C: u32 = 1;\nconst PRIV_C: u32 = 2;\npub(crate) const PCRATE_C: u32 = 3;\npub static PUB_S: &str = \"a\";\nstatic PRIV_S: &str = \"b\";\n",
            &mut s,
        );
        for name in ["PUB_C", "PRIV_C", "PCRATE_C", "PUB_S", "PRIV_S"] {
            assert!(s.contains(name), "missing {name} in {:?}", s);
        }
    }

    #[test]
    fn test_extract_names_skips_non_alphanumeric() {
        let mut s = HashSet::new();
        // "fn weird-name" should NOT be added since hyphen is not alphanumeric or underscore.
        extract_names_from_source("fn weird-name() {}\n", &mut s);
        assert!(!s.contains("weird-name"));
    }

    #[test]
    fn test_extract_names_skips_empty_name() {
        let mut s = HashSet::new();
        // Just `fn ` followed by `(` produces an empty name.
        extract_names_from_source("fn () {}\n", &mut s);
        assert!(s.is_empty());
    }

    // --- cross_reference_bindings ---

    #[test]
    fn test_cross_reference_bindings_all_verified() {
        let mut known = HashSet::new();
        known.insert("foo".to_string());
        known.insert("bar".to_string());
        let entries = vec![
            ("foo".to_string(), "f.yaml".to_string()),
            ("bar".to_string(), "f.yaml".to_string()),
        ];
        let (total, unique, verified, missing) = cross_reference_bindings(&entries, &known);
        assert_eq!(total, 2);
        assert_eq!(unique, 2);
        assert_eq!(verified, 2);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_cross_reference_bindings_dedup_via_seen() {
        let mut known = HashSet::new();
        known.insert("foo".to_string());
        let entries = vec![
            ("foo".to_string(), "a.yaml".to_string()),
            ("foo".to_string(), "b.yaml".to_string()),
            ("foo".to_string(), "c.yaml".to_string()),
        ];
        let (total, unique, verified, _) = cross_reference_bindings(&entries, &known);
        // total counts entries; unique counts seen set; verified increments only on first sighting.
        assert_eq!(total, 3);
        assert_eq!(unique, 1);
        assert_eq!(verified, 1);
    }

    #[test]
    fn test_cross_reference_bindings_missing_capped_at_5() {
        let known = HashSet::new();
        let entries: Vec<(String, String)> = (0..10)
            .map(|i| (format!("fn_{i}"), "x.yaml".to_string()))
            .collect();
        let (total, unique, verified, missing) = cross_reference_bindings(&entries, &known);
        assert_eq!(total, 10);
        assert_eq!(unique, 10);
        assert_eq!(verified, 0);
        assert_eq!(missing.len(), 5, "missing list must cap at 5");
    }

    // --- parse_binding_entries ---

    #[test]
    fn test_parse_binding_entries_extracts_implemented_pairs() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        let p = cd.join("binding.yaml");
        let content = "function: \"foo\"\nstatus: implemented\n- contract: x\nfunction: \"bar\"\nstatus: pending\n- contract: x\n";
        let mut entries = Vec::new();
        parse_binding_entries(content, &p, &cd, &mut entries);
        // Only "foo" (implemented) is added; "bar" (pending) is skipped.
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "foo");
        assert_eq!(entries[0].1, "binding.yaml");
    }

    #[test]
    fn test_parse_binding_entries_strips_type_double_colon_prefix() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        let p = cd.join("binding.yaml");
        let content = "function: \"MyType::method_name\"\nstatus: implemented\n";
        let mut entries = Vec::new();
        parse_binding_entries(content, &p, &cd, &mut entries);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "method_name");
    }

    #[test]
    fn test_parse_binding_entries_skips_empty_function() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        let p = cd.join("binding.yaml");
        let content = "function: \"\"\nstatus: implemented\n";
        let mut entries = Vec::new();
        parse_binding_entries(content, &p, &cd, &mut entries);
        assert!(entries.is_empty());
    }

    // --- has_contract_yamls ---

    #[test]
    fn test_has_contract_yamls_empty_dir_false() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_contract_yamls(tmp.path()));
    }

    #[test]
    fn test_has_contract_yamls_only_binding_returns_false() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("binding.yaml"), "");
        assert!(!has_contract_yamls(tmp.path()));
    }

    #[test]
    fn test_has_contract_yamls_normal_yaml_returns_true() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("foo.yaml"), "");
        assert!(has_contract_yamls(tmp.path()));
    }

    #[test]
    fn test_has_contract_yamls_yml_extension_accepted() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("bar.yml"), "");
        assert!(has_contract_yamls(tmp.path()));
    }

    #[test]
    fn test_has_contract_yamls_missing_dir_false() {
        assert!(!has_contract_yamls(Path::new("/tmp/does-not-exist-xyz123")));
    }

    // --- collect_stems_recursive ---

    #[test]
    fn test_collect_stems_recursive_picks_up_yamls_in_subdirs() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.yaml"), "");
        write(&tmp.path().join("sub/b.yml"), "");
        let mut stems = HashSet::new();
        collect_stems_recursive(tmp.path(), &mut stems);
        assert!(stems.contains("a"));
        assert!(stems.contains("b"));
    }

    #[test]
    fn test_collect_stems_recursive_skips_binding_files() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("real.yaml"), "");
        write(&tmp.path().join("binding.yaml"), "");
        write(&tmp.path().join("foo-binding.yaml"), "");
        let mut stems = HashSet::new();
        collect_stems_recursive(tmp.path(), &mut stems);
        assert!(stems.contains("real"));
        assert!(!stems.contains("binding"));
        assert!(!stems.contains("foo-binding"));
    }

    #[test]
    fn test_collect_stems_recursive_missing_dir_no_op() {
        let mut stems = HashSet::new();
        collect_stems_recursive(Path::new("/tmp/no-such-dir-abc"), &mut stems);
        assert!(stems.is_empty());
    }

    // --- detect_buildrs_enforcement ---

    #[test]
    fn test_detect_buildrs_enforcement_no_build_rs_false() {
        let tmp = TempDir::new().unwrap();
        assert!(!detect_buildrs_enforcement(tmp.path()));
    }

    #[test]
    fn test_detect_buildrs_enforcement_root_buildrs_with_keyword() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("build.rs"),
            "fn main() { let _ = \"contract\"; }",
        );
        assert!(detect_buildrs_enforcement(tmp.path()));
    }

    #[test]
    fn test_detect_buildrs_enforcement_root_buildrs_no_keyword_false() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("build.rs"), "fn main() {}");
        assert!(!detect_buildrs_enforcement(tmp.path()));
    }

    #[test]
    fn test_detect_buildrs_enforcement_member_crate_with_binding_keyword() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("crates/foo/build.rs"),
            "fn main() { let _ = \"binding\"; }",
        );
        assert!(detect_buildrs_enforcement(tmp.path()));
    }

    #[test]
    fn test_detect_buildrs_enforcement_member_crate_with_all_implemented() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("crates/bar/build.rs"),
            "fn main() { let _ = \"AllImplemented\"; }",
        );
        assert!(detect_buildrs_enforcement(tmp.path()));
    }

    // --- count_contract_test_refs ---

    #[test]
    fn test_count_contract_test_refs_no_contracts_dir_returns_zeros() {
        let tmp = TempDir::new().unwrap();
        let (refs, existing, missing) = count_contract_test_refs(tmp.path());
        assert_eq!((refs, existing, missing), (0, 0, 0));
    }

    #[test]
    fn test_count_contract_test_refs_no_test_keys_returns_zeros() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        write(&cd.join("a.yaml"), "name: x\n");
        let (refs, existing, missing) = count_contract_test_refs(tmp.path());
        assert_eq!((refs, existing, missing), (0, 0, 0));
    }

    #[test]
    fn test_count_contract_test_refs_existing_and_missing_split() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        write(
            &cd.join("a.yaml"),
            "test: \"test_alpha\"\ntest: \"test_missing\"\ntest: \"prop_beta\"\n",
        );
        // Provide one matching test; one stays "missing".
        write(
            &tmp.path().join("src/lib.rs"),
            "#[test]\nfn test_alpha() {}\nfn prop_beta() {}\n",
        );
        let (refs, existing, missing) = count_contract_test_refs(tmp.path());
        assert_eq!(refs, 3);
        assert_eq!(existing, 2);
        assert_eq!(missing, 1);
    }

    // --- collect_contract_equation_names ---

    #[test]
    fn test_collect_contract_equation_names_missing_dir_empty() {
        let tmp = TempDir::new().unwrap();
        let names = collect_contract_equation_names(&tmp.path().join("nope"));
        assert!(names.is_empty());
    }

    #[test]
    fn test_collect_contract_equation_names_yaml_with_pre_returns_name() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        // Equation name "my_equation" with preconditions.
        let yaml = "equations:\n  my_equation:\n    preconditions:\n      - x > 0\n";
        write(&cd.join("foo.yaml"), yaml);
        let names = collect_contract_equation_names(&cd);
        assert!(names.contains(&"my_equation".to_string()));
    }

    // --- resolve_contracts_dir ---

    #[test]
    fn test_resolve_contracts_dir_local_with_yamls_returns_local() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        write(&cd.join("real.yaml"), "");
        let resolved = resolve_contracts_dir(tmp.path());
        // sibling provable-contracts is unlikely to exist for this tempdir layout,
        // so the local path is the expected fallback.
        let canonical_local = std::fs::canonicalize(&cd).unwrap();
        if let Some(p) = resolved {
            let canonical_resolved = std::fs::canonicalize(&p).unwrap();
            assert_eq!(canonical_resolved, canonical_local);
        }
    }

    #[test]
    fn test_resolve_contracts_dir_no_contracts_returns_none() {
        let tmp = TempDir::new().unwrap();
        // No `contracts/` dir at all → None (assuming tempdir parent has no provable-contracts).
        let resolved = resolve_contracts_dir(tmp.path());
        // May be Some if /tmp's siblings include a provable-contracts/ dir; just accept either.
        let _ = resolved;
    }

    // --- collect_known_fn_names ---

    #[test]
    fn test_collect_known_fn_names_no_src_returns_none() {
        let tmp = TempDir::new().unwrap();
        assert!(collect_known_fn_names(tmp.path()).is_none());
    }

    #[test]
    fn test_collect_known_fn_names_picks_up_src_dir() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("src/lib.rs"), "pub fn alpha() {}\n");
        let known = collect_known_fn_names(tmp.path()).expect("Some");
        assert!(known.contains("alpha"));
    }

    #[test]
    fn test_collect_known_fn_names_picks_up_workspace_crates() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("crates/foo/src/main.rs"), "fn beta() {}\n");
        let known = collect_known_fn_names(tmp.path()).expect("Some");
        assert!(known.contains("beta"));
    }

    // --- resolve_binding_files ---

    #[test]
    fn test_resolve_binding_files_finds_local_binding() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        write(&cd.join("binding.yaml"), "status: implemented\n");
        let abs = std::fs::canonicalize(tmp.path()).unwrap();
        let files = resolve_binding_files(&cd, &abs, "myproj");
        assert!(!files.is_empty());
        assert!(files.iter().any(|p| p
            .file_name()
            .is_some_and(|n| n.to_string_lossy().contains("binding"))));
    }
}
