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

