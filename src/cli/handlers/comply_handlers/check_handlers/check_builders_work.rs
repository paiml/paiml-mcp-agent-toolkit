fn build_work_ladder_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_work_ladder as wl;
    // Per-check parallel: 10 read-only ladder checks (parse/YAML/SHA/lean-map).
    run_checks_parallel(
        project_path,
        comply_config,
        vec![
            (
                "cb-1610",
                wl::check_ladder_parses as fn(&Path) -> ComplianceCheck,
            ),
            ("cb-1611", wl::check_ladder_bound_by_yaml),
            ("cb-1612", wl::check_ladder_l1_test_evidence),
            ("cb-1613", wl::check_ladder_l3_falsification),
            ("cb-1614", wl::check_ladder_l4_kani),
            ("cb-1615", wl::check_ladder_kani_harness_sha),
            ("cb-1616", wl::check_ladder_l5_lean),
            ("cb-1617", wl::check_ladder_downgrade_audit),
            ("cb-1618", wl::check_ladder_monotonicity),
            ("cb-1619", wl::check_ladder_completion_matches),
        ],
    )
}

/// CB-1620..CB-1629: work-falsification-unification enforcement (Component 29).
fn build_falsification_unification_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_falsification_unification as fu;
    // Per-check parallel: 10 read-only falsification-unification checks.
    run_checks_parallel(
        project_path,
        comply_config,
        vec![
            (
                "cb-1620",
                fu::check_inherited_roster_coverage as fn(&Path) -> ComplianceCheck,
            ),
            ("cb-1621", fu::check_expected_snapshot_drift),
            ("cb-1622", fu::check_roster_execution_coverage),
            ("cb-1623", fu::check_no_duplicate_provable_entries),
            ("cb-1624", fu::check_no_manual_deletion),
            ("cb-1625", fu::check_inherited_failure_fatal),
            ("cb-1626", fu::check_test_id_exists_in_yaml),
            ("cb-1627", fu::check_post_bind_yaml_drift),
            ("cb-1628", fu::check_per_run_log_line),
            ("cb-1629", fu::check_l4_timeout_gate),
        ],
    )
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
    // Per-check parallel: cot-proof was the single wall-time bottleneck
    // (~113s release; 10 read-only CoT checks that walk `.pmat-work/` + parse
    // derivations). Parallelizing them is the largest remaining win.
    run_checks_parallel(
        project_path,
        comply_config,
        vec![
            (
                "cb-1640",
                cp::check_assumption_references_resolve as fn(&Path) -> ComplianceCheck,
            ),
            ("cb-1641", cp::check_step_has_evidence_method),
            ("cb-1642", cp::check_existing_test_paths_resolve),
            ("cb-1643", cp::check_l3_structured_expr_present),
            ("cb-1644", cp::check_agent_run_replayable),
            ("cb-1645", cp::check_derived_yaml_obligations_present),
            ("cb-1646", cp::check_cot_derivation_sha_fresh),
            ("cb-1647", cp::check_no_orphan_steps),
            ("cb-1648", cp::check_l4_axiomatic_discharge_bounded),
            ("cb-1649", cp::check_l5_lean_theorem_mapping),
        ],
    )
}

/// CB-1650..CB-1658: Modern Agentic Coding Support checks (Component 32).
fn build_macs_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_macs as macs;
    vec![
        filter_check_by_config(
            macs::check_skill_effort_pinned(project_path),
            "cb-1650",
            comply_config,
        ),
        filter_check_by_config(
            macs::check_mcp_manifest_faithful(project_path),
            "cb-1656",
            comply_config,
        ),
        filter_check_by_config(
            macs::check_doc_model_drift(project_path),
            "cb-1657",
            comply_config,
        ),
        filter_check_by_config(
            macs::check_roadmap_fresh(project_path),
            "cb-1655",
            comply_config,
        ),
        filter_check_by_config(
            macs::check_receipt_provenance_present(project_path),
            "cb-1651",
            comply_config,
        ),
        filter_check_by_config(
            macs::check_ladder_claim_drift(project_path),
            "cb-1653",
            comply_config,
        ),
        filter_check_by_config(
            macs::check_refusal_events_acked(project_path),
            "cb-1654",
            comply_config,
        ),
        filter_check_by_config(
            macs::check_derivation_completeness(project_path),
            "cb-1658",
            comply_config,
        ),
    ]
}
