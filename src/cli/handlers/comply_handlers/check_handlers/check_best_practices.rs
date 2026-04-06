// Best practices checks - CB-400, CB-130, CB-500+, CB-600+, CB-700+, CB-800+, CB-900+, CB-950+, CB-1000+, CB-1050+
//
// Originally check_handlers_part2.rs, contains language-specific best practices
// detection functions and the aggregate_violations helper.

use crate::cli::handlers::comply_cb_detect::{self, CbPatternViolation};
use crate::models::comply_config::ComplyConfig;
use std::path::Path;

use super::types::*;
// Re-export check_tdg_grade_gate and check_custom_scores from their submodules
pub(crate) use super::check_tdg_grade::{check_custom_scores, check_tdg_grade_gate};

fn is_cb_suppressed(v: &CbPatternViolation, config: Option<&ComplyConfig>) -> bool {
    config.is_some_and(|c| c.is_suppressed(&v.pattern_id, &v.file).is_some())
}

fn suppression_suffix(count: u32, prefix: &str) -> String {
    debug_assert!(count > 0, "count must be positive");
    if count > 0 {
        format!("{prefix}{count} suppressed via .pmat.yaml")
    } else {
        String::new()
    }
}

fn truncate_issues(issues: Vec<String>) -> Vec<String> {
    debug_assert!(!issues.is_empty(), "issues must not be empty");
    if issues.len() <= 20 {
        return issues;
    }
    let extra = issues.len() - 20;
    let mut truncated: Vec<String> = issues.into_iter().take(20).collect();
    truncated.push(format!("    ... and {extra} more"));
    truncated
}

fn aggregate_violations(
    check_name: &str,
    detectors: &[(&str, Vec<CbPatternViolation>)],
    comply_config: Option<&ComplyConfig>,
    fail_on_error: bool,
) -> ComplianceCheck {
    debug_assert!(!check_name.is_empty(), "check_name must not be empty");
    let mut all_issues: Vec<String> = Vec::new();
    let mut suppressed_count = 0u32;
    let mut counts = [0u32; 3]; // [error, warning, info]
    for (_id, violations) in detectors {
        for v in violations {
            if is_cb_suppressed(v, comply_config) {
                suppressed_count += 1;
                continue;
            }
            all_issues.push(format!(
                "{}: {} ({}:{})",
                v.pattern_id, v.description, v.file, v.line
            ));
            match v.severity {
                comply_cb_detect::Severity::Error => counts[0] += 1,
                comply_cb_detect::Severity::Warning => counts[1] += 1,
                _ => counts[2] += 1,
            }
        }
    }
    let total: u32 = counts.iter().sum();
    if total == 0 {
        let suffix = suppression_suffix(suppressed_count, " (");
        let close = if suppressed_count > 0 { ")" } else { "" };
        return ComplianceCheck {
            name: check_name.into(),
            status: CheckStatus::Pass,
            message: format!("No violations detected{suffix}{close}"),
            severity: Severity::Info,
        };
    }
    let display = truncate_issues(all_issues);
    let suffix = suppression_suffix(suppressed_count, ", ");
    let status = if fail_on_error && counts[0] > 0 {
        CheckStatus::Fail
    } else {
        CheckStatus::Warn
    };
    ComplianceCheck {
        name: check_name.into(),
        status,
        message: format!(
            "[Advisory] {} errors, {} warnings, {} info{}:\n{}",
            counts[0],
            counts[1],
            counts[2],
            suffix,
            display.join("\n")
        ),
        severity: Severity::Warning,
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_shell_makefile_quality(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use comply_cb_detect::{
        detect_cb400_git_hooks_quality, detect_cb401_makefile_quality,
        detect_cb402_shell_script_quality,
    };
    let mut all_issues: Vec<String> = Vec::new();
    let (mut warning_count, mut error_count) = (0, 0);
    for v in &detect_cb400_git_hooks_quality(project_path) {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        match v.severity {
            comply_cb_detect::Severity::Error | comply_cb_detect::Severity::Critical => {
                error_count += 1
            }
            _ => warning_count += 1,
        }
    }
    for v in &detect_cb401_makefile_quality(project_path) {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        match v.severity {
            comply_cb_detect::Severity::Error | comply_cb_detect::Severity::Critical => {
                error_count += 1
            }
            _ => warning_count += 1,
        }
    }
    for v in &detect_cb402_shell_script_quality(project_path) {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        match v.severity {
            comply_cb_detect::Severity::Error | comply_cb_detect::Severity::Critical => {
                error_count += 1
            }
            _ => warning_count += 1,
        }
    }
    let total_violations = all_issues.len();
    if total_violations == 0 {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".into(),
            status: CheckStatus::Pass,
            message: "bashrs: All shell scripts and Makefiles pass quality checks".into(),
            severity: Severity::Info,
        }
    } else if error_count > 0 {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".into(),
            status: CheckStatus::Fail,
            message: format!(
                "bashrs: {} errors, {} warnings:\n{}",
                error_count,
                warning_count,
                format_violation_list(&all_issues)
            ),
            severity: Severity::Error,
        }
    } else {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".into(),
            status: CheckStatus::Warn,
            message: format!(
                "bashrs: {} warnings:\n{}",
                warning_count,
                format_violation_list(&all_issues)
            ),
            severity: Severity::Warning,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_agent_context_adoption(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let report = comply_cb_detect::detect_cb130_agent_context_adoption(project_path);
    let mut issues: Vec<String> = Vec::new();
    let mut warning_count = 0;
    if !report.index_exists {
        issues.push(
            "CB-130: No agent context index found at .pmat/context.idx or .pmat/context.db".into(),
        );
        issues.push("  Run 'pmat query \"test\" --rebuild-index' to build the index".into());
        warning_count += 1;
    } else if report.index_stale {
        issues.push(format!(
            "CB-130: Agent context index is stale ({:.0} hours old, threshold: 24h)",
            report.index_age_hours.unwrap_or(0.0)
        ));
        issues.push("  Run 'pmat query \"test\" --rebuild-index' to refresh".into());
        warning_count += 1;
        if report.function_count == 0 {
            issues.push("CB-130: Agent context index has 0 functions".into());
            warning_count += 1;
        }
    } else if report.function_count == 0 {
        issues.push("CB-130: Agent context index has 0 functions".into());
        warning_count += 1;
    }
    if !report.claude_md_configured {
        issues.push("CB-130: CLAUDE.md does not reference pmat_query_code or pmat query".into());
        issues.push("  Add agent context instructions to CLAUDE.md for agent adoption".into());
        warning_count += 1;
    }
    if !report.missing_required_patterns.is_empty() {
        for pattern in &report.missing_required_patterns {
            issues.push(format!(
                "CB-130: CLAUDE.md missing required: \"{}\"",
                pattern
            ));
        }
        issues.push("  Add pmat query decision tree to CLAUDE.md".into());
        warning_count += 1;
    }
    if !report.forbidden_patterns_found.is_empty() {
        for found in &report.forbidden_patterns_found {
            issues.push(format!(
                "CB-130: CLAUDE.md contains forbidden pattern \"{}\" at line {}",
                found.pattern, found.line
            ));
        }
        issues.push("  Remove grep/find examples from CLAUDE.md (use pmat query instead)".into());
        warning_count += 1;
    }
    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-130: Agent Context Adoption".into(),
            status: CheckStatus::Pass,
            message: format!(
                "Agent context index: {} functions, CLAUDE.md configured",
                report.function_count
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-130: Agent Context Adoption".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} issues:\n{}",
                warning_count,
                issues
                    .iter()
                    .map(|i| format!("    - {}", i))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            severity: Severity::Warning,
        }
    }
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_rust_best_practices(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    check_rust_best_practices_with_config(project_path, None)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_rust_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if !project_path.join("Cargo.toml").exists() {
        return ComplianceCheck {
            name: "CB-500: Rust Best Practices (CB-500 to CB-530)".into(),
            status: CheckStatus::Pass,
            message: "Not a Rust project (no Cargo.toml found)".into(),
            severity: Severity::Info,
        };
    }
    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        (
            "CB-500",
            comply_cb_detect::detect_cb500_publish_hygiene(project_path),
        ),
        (
            "CB-501",
            comply_cb_detect::detect_cb501_unwrap_density(project_path),
        ),
        (
            "CB-502",
            comply_cb_detect::detect_cb502_expect_quality(project_path),
        ),
        (
            "CB-503",
            comply_cb_detect::detect_cb503_clippy_config(project_path),
        ),
        (
            "CB-504",
            comply_cb_detect::detect_cb504_deny_config(project_path),
        ),
        (
            "CB-505",
            comply_cb_detect::detect_cb505_workspace_lint_hygiene(project_path),
        ),
        (
            "CB-506",
            comply_cb_detect::detect_cb506_string_byte_indexing(project_path),
        ),
        (
            "CB-507",
            comply_cb_detect::detect_cb507_panic_macros(project_path),
        ),
        (
            "CB-508",
            comply_cb_detect::detect_cb508_lossy_numeric_casts(project_path),
        ),
        (
            "CB-509",
            comply_cb_detect::detect_cb509_feature_gate_coverage(project_path),
        ),
        (
            "CB-510",
            comply_cb_detect::detect_cb510_include_macro_hygiene(project_path),
        ),
        (
            "CB-511",
            comply_cb_detect::detect_cb511_flaky_timing_tests(project_path),
        ),
        (
            "CB-512",
            comply_cb_detect::detect_cb512_error_propagation_gap(project_path),
        ),
        (
            "CB-513",
            comply_cb_detect::detect_cb513_silent_error_swallowing(project_path),
        ),
        (
            "CB-514",
            comply_cb_detect::detect_cb514_debug_eprintln_leaks(project_path),
        ),
        (
            "CB-515",
            comply_cb_detect::detect_cb515_catch_all_match_default(project_path),
        ),
        (
            "CB-516",
            comply_cb_detect::detect_cb516_hardcoded_magic_numbers(project_path),
        ),
        (
            "CB-517",
            comply_cb_detect::detect_cb517_stale_debug_artifacts(project_path),
        ),
        (
            "CB-518",
            comply_cb_detect::detect_cb518_expensive_clone_in_loop(project_path),
        ),
        (
            "CB-519",
            comply_cb_detect::detect_cb519_lossy_data_pipeline(project_path),
        ),
        (
            "CB-520",
            comply_cb_detect::detect_cb520_expensive_init_in_loop(project_path),
        ),
        (
            "CB-521",
            comply_cb_detect::detect_cb521_format_without_magic_bytes(project_path),
        ),
        (
            "CB-522",
            comply_cb_detect::detect_cb522_untested_path_normalization(project_path),
        ),
        (
            "CB-523",
            comply_cb_detect::detect_cb523_external_config_over_embedded(project_path),
        ),
        (
            "CB-524",
            comply_cb_detect::detect_cb524_incomplete_enum_match(project_path),
        ),
        (
            "CB-525",
            comply_cb_detect::detect_cb525_hardcoded_field_names(project_path),
        ),
        (
            "CB-526",
            comply_cb_detect::detect_cb526_single_path_resolution(project_path),
        ),
        (
            "CB-527",
            comply_cb_detect::detect_cb527_incomplete_pattern_list(project_path),
        ),
        (
            "CB-528",
            comply_cb_detect::detect_cb528_division_by_length(project_path),
        ),
        (
            "CB-529",
            comply_cb_detect::detect_cb529_pmat_tracked_in_git(project_path),
        ),
        (
            "CB-530",
            comply_cb_detect::detect_cb530_log_without_clamp(project_path),
        ),
    ];
    aggregate_violations(
        "CB-500: Rust Best Practices (CB-500 to CB-530)",
        &detectors,
        comply_config,
        false,
    )
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_lua_best_practices(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    check_lua_best_practices_with_config(project_path, None)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_lua_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if comply_cb_detect::walkdir_lua_files(project_path).is_empty() {
        return ComplianceCheck {
            name: "CB-600: Lua Best Practices (CB-600 to CB-619)".into(),
            status: CheckStatus::Pass,
            message: "Not a Lua project (no .lua files found)".into(),
            severity: Severity::Info,
        };
    }
    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        (
            "CB-600",
            comply_cb_detect::detect_cb600_implicit_globals(project_path),
        ),
        (
            "CB-601",
            comply_cb_detect::detect_cb601_nil_unsafe_access(project_path),
        ),
        (
            "CB-602",
            comply_cb_detect::detect_cb602_pcall_error_handling(project_path),
        ),
        (
            "CB-603",
            comply_cb_detect::detect_cb603_deprecated_dangerous_api(project_path),
        ),
        (
            "CB-604",
            comply_cb_detect::detect_cb604_unused_variables(project_path),
        ),
        (
            "CB-605",
            comply_cb_detect::detect_cb605_string_concat_in_loop(project_path),
        ),
        (
            "CB-606",
            comply_cb_detect::detect_cb606_missing_module_return(project_path),
        ),
        (
            "CB-607",
            comply_cb_detect::detect_cb607_colon_dot_confusion(project_path),
        ),
        (
            "CB-608",
            comply_cb_detect::detect_cb608_unchecked_nil_err(project_path),
        ),
        (
            "CB-609",
            comply_cb_detect::detect_cb609_assert_in_library(project_path),
        ),
        (
            "CB-610",
            comply_cb_detect::detect_cb610_string_accumulator_in_loop(project_path),
        ),
        (
            "CB-611",
            comply_cb_detect::detect_cb611_weak_table_misuse(project_path),
        ),
        (
            "CB-612",
            comply_cb_detect::detect_cb612_test_framework(project_path),
        ),
        (
            "CB-613",
            comply_cb_detect::detect_cb613_require_cycles(project_path),
        ),
        (
            "CB-614",
            comply_cb_detect::detect_cb614_global_protection(project_path),
        ),
        (
            "CB-615",
            comply_cb_detect::detect_cb615_coroutine_checks(project_path),
        ),
        (
            "CB-616",
            comply_cb_detect::detect_cb616_type_annotations(project_path),
        ),
        (
            "CB-617",
            comply_cb_detect::detect_cb617_openresty_checks(project_path),
        ),
        (
            "CB-618",
            comply_cb_detect::detect_cb618_ffi_safety(project_path),
        ),
        (
            "CB-619",
            comply_cb_detect::detect_cb619_oop_patterns(project_path),
        ),
    ];
    aggregate_violations(
        "CB-600: Lua Best Practices (CB-600 to CB-619)",
        &detectors,
        comply_config,
        false,
    )
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_sql_best_practices(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    check_sql_best_practices_with_config(project_path, None)
}
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_sql_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if comply_cb_detect::walkdir_sql_files(project_path).is_empty() {
        return ComplianceCheck {
            name: "CB-700: SQL Best Practices (CB-700 to CB-705)".into(),
            status: CheckStatus::Pass,
            message: "Not a SQL project (no .sql files found)".into(),
            severity: Severity::Info,
        };
    }
    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        (
            "CB-700",
            comply_cb_detect::detect_cb700_select_star(project_path),
        ),
        (
            "CB-701",
            comply_cb_detect::detect_cb701_missing_where(project_path),
        ),
        (
            "CB-702",
            comply_cb_detect::detect_cb702_implicit_join(project_path),
        ),
        (
            "CB-703",
            comply_cb_detect::detect_cb703_sql_injection(project_path),
        ),
        (
            "CB-704",
            comply_cb_detect::detect_cb704_missing_index_hint(project_path),
        ),
        (
            "CB-705",
            comply_cb_detect::detect_cb705_n_plus_1_query(project_path),
        ),
    ];
    aggregate_violations(
        "CB-700: SQL Best Practices (CB-700 to CB-705)",
        &detectors,
        comply_config,
        true,
    )
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_markdown_best_practices(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    check_markdown_best_practices_with_config(project_path, None)
}
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_markdown_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if comply_cb_detect::walkdir_markdown_files(project_path).is_empty() {
        return ComplianceCheck {
            name: "CB-900: Markdown Best Practices (CB-900 to CB-904)".into(),
            status: CheckStatus::Pass,
            message: "No Markdown files found".into(),
            severity: Severity::Info,
        };
    }
    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        (
            "CB-900",
            comply_cb_detect::detect_cb900_broken_internal_link(project_path),
        ),
        (
            "CB-901",
            comply_cb_detect::detect_cb901_heading_hierarchy_skip(project_path),
        ),
        (
            "CB-902",
            comply_cb_detect::detect_cb902_missing_alt_text(project_path),
        ),
        (
            "CB-903",
            comply_cb_detect::detect_cb903_bare_url(project_path),
        ),
        (
            "CB-904",
            comply_cb_detect::detect_cb904_long_line(project_path),
        ),
    ];
    aggregate_violations(
        "CB-900: Markdown Best Practices (CB-900 to CB-904)",
        &detectors,
        comply_config,
        false,
    )
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_yaml_best_practices(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    check_yaml_best_practices_with_config(project_path, None)
}
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_yaml_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if comply_cb_detect::walkdir_yaml_files(project_path).is_empty() {
        return ComplianceCheck {
            name: "CB-950: YAML Best Practices (CB-950 to CB-954)".into(),
            status: CheckStatus::Pass,
            message: "No YAML files found".into(),
            severity: Severity::Info,
        };
    }
    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        (
            "CB-950",
            comply_cb_detect::detect_cb950_truthy_ambiguity(project_path),
        ),
        (
            "CB-951",
            comply_cb_detect::detect_cb951_excessive_nesting(project_path),
        ),
        (
            "CB-952",
            comply_cb_detect::detect_cb952_missing_required_fields(project_path),
        ),
        (
            "CB-953",
            comply_cb_detect::detect_cb953_unpinned_action(project_path),
        ),
        (
            "CB-954",
            comply_cb_detect::detect_cb954_plaintext_secret(project_path),
        ),
    ];
    aggregate_violations(
        "CB-950: YAML Best Practices (CB-950 to CB-954)",
        &detectors,
        comply_config,
        true,
    )
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_model_quality(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    check_model_quality_with_config(project_path, None)
}
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_model_quality_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if comply_cb_detect::walkdir_model_files(project_path).is_empty() {
        return ComplianceCheck {
            name: "CB-1000: MLOps Model Quality (CB-1000 to CB-1008)".into(),
            status: CheckStatus::Pass,
            message: "No model files found (*.gguf, *.apr, *.safetensors)".into(),
            severity: Severity::Info,
        };
    }
    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        (
            "CB-1000",
            comply_cb_detect::detect_cb1000_missing_model_card(project_path),
        ),
        (
            "CB-1001",
            comply_cb_detect::detect_cb1001_oversized_tensor_count(project_path),
        ),
        (
            "CB-1002",
            comply_cb_detect::detect_cb1002_missing_tokenizer(project_path),
        ),
        (
            "CB-1004",
            comply_cb_detect::detect_cb1004_missing_architecture(project_path),
        ),
        (
            "CB-1005",
            comply_cb_detect::detect_cb1005_quantization_mismatch(project_path),
        ),
        (
            "CB-1006",
            comply_cb_detect::detect_cb1006_sharded_without_index(project_path),
        ),
        (
            "CB-1007",
            comply_cb_detect::detect_cb1007_excessive_file_size(project_path),
        ),
        (
            "CB-1008",
            comply_cb_detect::detect_cb1008_apr_missing_crc(project_path),
        ),
    ];
    aggregate_violations(
        "CB-1000: MLOps Model Quality (CB-1000 to CB-1008)",
        &detectors,
        comply_config,
        true,
    )
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_scala_best_practices(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    check_scala_best_practices_with_config(project_path, None)
}
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_scala_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if comply_cb_detect::walkdir_scala_files(project_path).is_empty() {
        return ComplianceCheck {
            name: "CB-800: Scala Best Practices (CB-800 to CB-805)".into(),
            status: CheckStatus::Pass,
            message: "Not a Scala project (no .scala files found)".into(),
            severity: Severity::Info,
        };
    }
    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        (
            "CB-800",
            comply_cb_detect::detect_cb800_mutable_collection(project_path),
        ),
        (
            "CB-801",
            comply_cb_detect::detect_cb801_null_usage(project_path),
        ),
        (
            "CB-802",
            comply_cb_detect::detect_cb802_wildcard_import(project_path),
        ),
        (
            "CB-803",
            comply_cb_detect::detect_cb803_return_statement(project_path),
        ),
        (
            "CB-804",
            comply_cb_detect::detect_cb804_var_declaration(project_path),
        ),
        (
            "CB-805",
            comply_cb_detect::detect_cb805_blocking_in_future(project_path),
        ),
    ];
    aggregate_violations(
        "CB-800: Scala Best Practices (CB-800 to CB-805)",
        &detectors,
        comply_config,
        true,
    )
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_lean_best_practices(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    check_lean_best_practices_with_config(project_path, None)
}
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_lean_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if comply_cb_detect::walkdir_lean_files(project_path).is_empty() {
        return ComplianceCheck {
            name: "CB-1050: Lean 4 Best Practices (CB-1050 to CB-1053)".into(),
            status: CheckStatus::Pass,
            message: "Not a Lean project (no .lean files found)".into(),
            severity: Severity::Info,
        };
    }
    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        (
            "CB-1050",
            comply_cb_detect::detect_cb1050_sorry_usage(project_path),
        ),
        (
            "CB-1051",
            comply_cb_detect::detect_cb1051_axiom_usage(project_path),
        ),
        (
            "CB-1052",
            comply_cb_detect::detect_cb1052_theorem_coverage(project_path),
        ),
        (
            "CB-1053",
            comply_cb_detect::detect_cb1053_undocumented_theorems(project_path),
        ),
    ];
    aggregate_violations(
        "CB-1050: Lean 4 Best Practices (CB-1050 to CB-1053)",
        &detectors,
        comply_config,
        true,
    )
}
