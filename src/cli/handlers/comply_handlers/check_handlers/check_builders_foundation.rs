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
            check_workspace_member_registry_deps(project_path),
            "cb-081-f",
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

