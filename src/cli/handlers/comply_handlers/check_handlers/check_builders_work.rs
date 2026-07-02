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

/// CB-1650..CB-1658: Modern Agentic Coding Support checks (Component 32).
fn build_macs_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_macs as macs;
    vec![
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
