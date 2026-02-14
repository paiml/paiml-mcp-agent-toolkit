/// CB-400: Check Shell & Makefile Quality using bashrs
///
/// Uses bashrs to lint:
/// - CB-400: Git hooks (pre-commit, pre-push, etc.)
/// - CB-401: Makefile
/// - CB-402: Shell scripts (*.sh)
pub(crate) fn check_shell_makefile_quality(project_path: &Path) -> ComplianceCheck {
    use super::comply_cb_detect::{
        detect_cb400_git_hooks_quality,
        detect_cb401_makefile_quality,
        detect_cb402_shell_script_quality,
    };

    let mut all_issues: Vec<String> = Vec::new();
    let mut warning_count = 0;
    let mut error_count = 0;

    // CB-400: Git hooks
    let hook_violations = detect_cb400_git_hooks_quality(project_path);
    for v in &hook_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        match v.severity {
            super::comply_cb_detect::Severity::Error | super::comply_cb_detect::Severity::Critical => error_count += 1,
            _ => warning_count += 1,
        }
    }

    // CB-401: Makefile
    let makefile_violations = detect_cb401_makefile_quality(project_path);
    for v in &makefile_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        match v.severity {
            super::comply_cb_detect::Severity::Error | super::comply_cb_detect::Severity::Critical => error_count += 1,
            _ => warning_count += 1,
        }
    }

    // CB-402: Shell scripts
    let shell_violations = detect_cb402_shell_script_quality(project_path);
    for v in &shell_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        match v.severity {
            super::comply_cb_detect::Severity::Error | super::comply_cb_detect::Severity::Critical => error_count += 1,
            _ => warning_count += 1,
        }
    }

    let total_violations = hook_violations.len() + makefile_violations.len() + shell_violations.len();

    if total_violations == 0 {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".to_string(),
            status: CheckStatus::Pass,
            message: "bashrs: All shell scripts and Makefiles pass quality checks".to_string(),
            severity: Severity::Info,
        }
    } else if error_count > 0 {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "bashrs: {} errors, {} warnings:\n{}",
                error_count,
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Error,
        }
    } else {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "bashrs: {} warnings:\n{}",
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-130: Agent Context Adoption (PMAT-470)
///
/// Checks whether the project has a RAG-powered agent context index
/// set up for intelligent code search. Validates:
/// - Index exists at .pmat/context.idx or .pmat/context.db
/// - Index is fresh (less than 24 hours old)
/// - CLAUDE.md references pmat_query_code (optional)
pub(crate) fn check_agent_context_adoption(project_path: &Path) -> ComplianceCheck {
    let report = detect_cb130_agent_context_adoption(project_path);

    let mut issues: Vec<String> = Vec::new();
    let mut warning_count = 0;

    if !report.index_exists {
        issues.push("CB-130: No agent context index found at .pmat/context.idx or .pmat/context.db".to_string());
        issues.push("  Run 'pmat query \"test\" --rebuild-index' to build the index".to_string());
        warning_count += 1;
    } else {
        if report.index_stale {
            let age = report.index_age_hours.unwrap_or(0.0);
            issues.push(format!(
                "CB-130: Agent context index is stale ({:.0} hours old, threshold: 24h)",
                age
            ));
            issues.push(
                "  Run 'pmat query \"test\" --rebuild-index' to refresh".to_string(),
            );
            warning_count += 1;
        }

        if report.function_count == 0 {
            issues.push("CB-130: Agent context index has 0 functions".to_string());
            warning_count += 1;
        }
    }

    if !report.claude_md_configured {
        issues.push(
            "CB-130: CLAUDE.md does not reference pmat_query_code or pmat query".to_string(),
        );
        issues.push(
            "  Add agent context instructions to CLAUDE.md for agent adoption".to_string(),
        );
        warning_count += 1;
    }

    // Check for missing required patterns
    if !report.missing_required_patterns.is_empty() {
        for pattern in &report.missing_required_patterns {
            issues.push(format!("CB-130: CLAUDE.md missing required: \"{}\"", pattern));
        }
        issues.push("  Add pmat query decision tree to CLAUDE.md".to_string());
        warning_count += 1;
    }

    // Check for forbidden patterns (potential grep usage instructions)
    if !report.forbidden_patterns_found.is_empty() {
        for found in &report.forbidden_patterns_found {
            issues.push(format!(
                "CB-130: CLAUDE.md contains forbidden pattern \"{}\" at line {}",
                found.pattern, found.line
            ));
        }
        issues.push("  Remove grep/find examples from CLAUDE.md (use pmat query instead)".to_string());
        warning_count += 1;
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-130: Agent Context Adoption".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "Agent context index: {} functions, CLAUDE.md configured",
                report.function_count
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-130: Agent Context Adoption".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} issues:\n{}",
                warning_count,
                issues
                    .iter()
                    .map(|i| format!("    - {}", i))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            severity: Severity::Warning,
        }
    }
}

/// Aggregate CB pattern violations into a ComplianceCheck, with optional suppression filtering.
/// Check if a violation is suppressed by config.
fn is_cb_suppressed(v: &CbPatternViolation, config: Option<&ComplyConfig>) -> bool {
    config.is_some_and(|c| c.is_suppressed(&v.pattern_id, &v.file).is_some())
}

/// Format suppression suffix for display.
fn suppression_suffix(count: u32, prefix: &str) -> String {
    if count > 0 {
        format!("{prefix}{count} suppressed via .pmat.yaml")
    } else {
        String::new()
    }
}

/// Truncate issue list for display (max 20 items).
fn truncate_issues(issues: Vec<String>) -> Vec<String> {
    if issues.len() <= 20 {
        return issues;
    }
    let extra = issues.len() - 20;
    let mut truncated: Vec<String> = issues.into_iter().take(20).collect();
    truncated.push(format!("    ... and {extra} more"));
    truncated
}

/// Aggregate CB pattern violations into a ComplianceCheck, with optional suppression filtering.
fn aggregate_violations(
    check_name: &str,
    detectors: &[(&str, Vec<CbPatternViolation>)],
    comply_config: Option<&ComplyConfig>,
    fail_on_error: bool,
) -> ComplianceCheck {
    let mut all_issues: Vec<String> = Vec::new();
    let mut suppressed_count = 0u32;
    let mut counts = [0u32; 3]; // [error, warning, info]

    for (_id, violations) in detectors {
        for v in violations {
            if is_cb_suppressed(v, comply_config) {
                suppressed_count += 1;
                continue;
            }
            all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
            match v.severity {
                super::comply_cb_detect::Severity::Error => counts[0] += 1,
                super::comply_cb_detect::Severity::Warning => counts[1] += 1,
                _ => counts[2] += 1,
            }
        }
    }

    let total: u32 = counts.iter().sum();
    if total == 0 {
        let suffix = suppression_suffix(suppressed_count, " (");
        let close = if suppressed_count > 0 { ")" } else { "" };
        return ComplianceCheck {
            name: check_name.to_string(),
            status: CheckStatus::Pass,
            message: format!("No violations detected{suffix}{close}"),
            severity: Severity::Info,
        };
    }

    let display = truncate_issues(all_issues);
    let suffix = suppression_suffix(suppressed_count, ", ");
    let status = if fail_on_error && counts[0] > 0 { CheckStatus::Fail } else { CheckStatus::Warn };
    ComplianceCheck {
        name: check_name.to_string(),
        status,
        message: format!(
            "[Advisory] {} errors, {} warnings, {} info{}:\n{}",
            counts[0], counts[1], counts[2], suffix, display.join("\n"),
        ),
        severity: Severity::Warning,
    }
}

/// Rust Best Practices Detection (CB-500 through CB-518)
#[allow(dead_code)]
pub(crate) fn check_rust_best_practices(project_path: &Path) -> ComplianceCheck {
    check_rust_best_practices_with_config(project_path, None)
}

pub(crate) fn check_rust_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    // Quick exit: no Cargo.toml means not a Rust project — skip all CB-500 checks
    if !project_path.join("Cargo.toml").exists() {
        return ComplianceCheck {
            name: "CB-500: Rust Best Practices (CB-500 to CB-527)".to_string(),
            status: CheckStatus::Pass,
            message: "Not a Rust project (no Cargo.toml found)".to_string(),
            severity: Severity::Info,
        };
    }

    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        ("CB-500", detect_cb500_publish_hygiene(project_path)),
        ("CB-501", detect_cb501_unwrap_density(project_path)),
        ("CB-502", detect_cb502_expect_quality(project_path)),
        ("CB-503", detect_cb503_clippy_config(project_path)),
        ("CB-504", detect_cb504_deny_config(project_path)),
        ("CB-505", detect_cb505_workspace_lint_hygiene(project_path)),
        ("CB-506", detect_cb506_string_byte_indexing(project_path)),
        ("CB-507", detect_cb507_panic_macros(project_path)),
        ("CB-508", detect_cb508_lossy_numeric_casts(project_path)),
        ("CB-509", detect_cb509_feature_gate_coverage(project_path)),
        ("CB-510", detect_cb510_include_macro_hygiene(project_path)),
        ("CB-511", detect_cb511_flaky_timing_tests(project_path)),
        ("CB-512", detect_cb512_error_propagation_gap(project_path)),
        ("CB-513", detect_cb513_silent_error_swallowing(project_path)),
        ("CB-514", detect_cb514_debug_eprintln_leaks(project_path)),
        ("CB-515", detect_cb515_catch_all_match_default(project_path)),
        ("CB-516", detect_cb516_hardcoded_magic_numbers(project_path)),
        ("CB-517", detect_cb517_stale_debug_artifacts(project_path)),
        ("CB-518", detect_cb518_expensive_clone_in_loop(project_path)),
        ("CB-519", detect_cb519_lossy_data_pipeline(project_path)),
        ("CB-520", detect_cb520_expensive_init_in_loop(project_path)),
        ("CB-521", detect_cb521_format_without_magic_bytes(project_path)),
        ("CB-522", detect_cb522_untested_path_normalization(project_path)),
        ("CB-523", detect_cb523_external_config_over_embedded(project_path)),
        ("CB-524", detect_cb524_incomplete_enum_match(project_path)),
        ("CB-525", detect_cb525_hardcoded_field_names(project_path)),
        ("CB-526", detect_cb526_single_path_resolution(project_path)),
        ("CB-527", detect_cb527_incomplete_pattern_list(project_path)),
    ];

    aggregate_violations(
        "CB-500: Rust Best Practices (CB-500 to CB-527)",
        &detectors,
        comply_config,
        false,
    )
}

/// Lua Best Practices Detection (CB-600 through CB-607)
#[allow(dead_code)]
pub(crate) fn check_lua_best_practices(project_path: &Path) -> ComplianceCheck {
    check_lua_best_practices_with_config(project_path, None)
}

pub(crate) fn check_lua_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    let lua_files = super::comply_cb_detect::walkdir_lua_files(project_path);
    if lua_files.is_empty() {
        return ComplianceCheck {
            name: "CB-600: Lua Best Practices (CB-600 to CB-619)".to_string(),
            status: CheckStatus::Pass,
            message: "Not a Lua project (no .lua files found)".to_string(),
            severity: Severity::Info,
        };
    }

    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        ("CB-600", super::comply_cb_detect::detect_cb600_implicit_globals(project_path)),
        ("CB-601", super::comply_cb_detect::detect_cb601_nil_unsafe_access(project_path)),
        ("CB-602", super::comply_cb_detect::detect_cb602_pcall_error_handling(project_path)),
        ("CB-603", super::comply_cb_detect::detect_cb603_deprecated_dangerous_api(project_path)),
        ("CB-604", super::comply_cb_detect::detect_cb604_unused_variables(project_path)),
        ("CB-605", super::comply_cb_detect::detect_cb605_string_concat_in_loop(project_path)),
        ("CB-606", super::comply_cb_detect::detect_cb606_missing_module_return(project_path)),
        ("CB-607", super::comply_cb_detect::detect_cb607_colon_dot_confusion(project_path)),
        ("CB-608", super::comply_cb_detect::detect_cb608_unchecked_nil_err(project_path)),
        ("CB-609", super::comply_cb_detect::detect_cb609_assert_in_library(project_path)),
        ("CB-610", super::comply_cb_detect::detect_cb610_string_accumulator_in_loop(project_path)),
        ("CB-611", super::comply_cb_detect::detect_cb611_weak_table_misuse(project_path)),
        ("CB-612", super::comply_cb_detect::detect_cb612_test_framework(project_path)),
        ("CB-613", super::comply_cb_detect::detect_cb613_require_cycles(project_path)),
        ("CB-614", super::comply_cb_detect::detect_cb614_global_protection(project_path)),
        ("CB-615", super::comply_cb_detect::detect_cb615_coroutine_checks(project_path)),
        ("CB-616", super::comply_cb_detect::detect_cb616_type_annotations(project_path)),
        ("CB-617", super::comply_cb_detect::detect_cb617_openresty_checks(project_path)),
        ("CB-618", super::comply_cb_detect::detect_cb618_ffi_safety(project_path)),
        ("CB-619", super::comply_cb_detect::detect_cb619_oop_patterns(project_path)),
    ];

    aggregate_violations(
        "CB-600: Lua Best Practices (CB-600 to CB-619)",
        &detectors,
        comply_config,
        false,
    )
}

#[allow(dead_code)]
pub(crate) fn check_sql_best_practices(project_path: &Path) -> ComplianceCheck {
    check_sql_best_practices_with_config(project_path, None)
}

pub(crate) fn check_sql_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    let sql_files = super::comply_cb_detect::walkdir_sql_files(project_path);
    if sql_files.is_empty() {
        return ComplianceCheck {
            name: "CB-700: SQL Best Practices (CB-700 to CB-705)".to_string(),
            status: CheckStatus::Pass,
            message: "Not a SQL project (no .sql files found)".to_string(),
            severity: Severity::Info,
        };
    }

    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        ("CB-700", super::comply_cb_detect::detect_cb700_select_star(project_path)),
        ("CB-701", super::comply_cb_detect::detect_cb701_missing_where(project_path)),
        ("CB-702", super::comply_cb_detect::detect_cb702_implicit_join(project_path)),
        ("CB-703", super::comply_cb_detect::detect_cb703_sql_injection(project_path)),
        ("CB-704", super::comply_cb_detect::detect_cb704_missing_index_hint(project_path)),
        ("CB-705", super::comply_cb_detect::detect_cb705_n_plus_1_query(project_path)),
    ];

    aggregate_violations(
        "CB-700: SQL Best Practices (CB-700 to CB-705)",
        &detectors,
        comply_config,
        true,
    )
}

#[allow(dead_code)]
pub(crate) fn check_markdown_best_practices(project_path: &Path) -> ComplianceCheck {
    check_markdown_best_practices_with_config(project_path, None)
}

pub(crate) fn check_markdown_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    let md_files = super::comply_cb_detect::walkdir_markdown_files(project_path);
    if md_files.is_empty() {
        return ComplianceCheck {
            name: "CB-900: Markdown Best Practices (CB-900 to CB-904)".to_string(),
            status: CheckStatus::Pass,
            message: "No Markdown files found".to_string(),
            severity: Severity::Info,
        };
    }

    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        ("CB-900", super::comply_cb_detect::detect_cb900_broken_internal_link(project_path)),
        ("CB-901", super::comply_cb_detect::detect_cb901_heading_hierarchy_skip(project_path)),
        ("CB-902", super::comply_cb_detect::detect_cb902_missing_alt_text(project_path)),
        ("CB-903", super::comply_cb_detect::detect_cb903_bare_url(project_path)),
        ("CB-904", super::comply_cb_detect::detect_cb904_long_line(project_path)),
    ];

    aggregate_violations(
        "CB-900: Markdown Best Practices (CB-900 to CB-904)",
        &detectors,
        comply_config,
        false,
    )
}

#[allow(dead_code)]
pub(crate) fn check_yaml_best_practices(project_path: &Path) -> ComplianceCheck {
    check_yaml_best_practices_with_config(project_path, None)
}

pub(crate) fn check_yaml_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    let yaml_files = super::comply_cb_detect::walkdir_yaml_files(project_path);
    if yaml_files.is_empty() {
        return ComplianceCheck {
            name: "CB-950: YAML Best Practices (CB-950 to CB-954)".to_string(),
            status: CheckStatus::Pass,
            message: "No YAML files found".to_string(),
            severity: Severity::Info,
        };
    }

    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        ("CB-950", super::comply_cb_detect::detect_cb950_truthy_ambiguity(project_path)),
        ("CB-951", super::comply_cb_detect::detect_cb951_excessive_nesting(project_path)),
        ("CB-952", super::comply_cb_detect::detect_cb952_missing_required_fields(project_path)),
        ("CB-953", super::comply_cb_detect::detect_cb953_unpinned_action(project_path)),
        ("CB-954", super::comply_cb_detect::detect_cb954_plaintext_secret(project_path)),
    ];

    aggregate_violations(
        "CB-950: YAML Best Practices (CB-950 to CB-954)",
        &detectors,
        comply_config,
        true,
    )
}

#[allow(dead_code)]
pub(crate) fn check_model_quality(project_path: &Path) -> ComplianceCheck {
    check_model_quality_with_config(project_path, None)
}

pub(crate) fn check_model_quality_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    let model_files = super::comply_cb_detect::walkdir_model_files(project_path);
    if model_files.is_empty() {
        return ComplianceCheck {
            name: "CB-1000: MLOps Model Quality (CB-1000 to CB-1008)".to_string(),
            status: CheckStatus::Pass,
            message: "No model files found (*.gguf, *.apr, *.safetensors)".to_string(),
            severity: Severity::Info,
        };
    }

    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        ("CB-1000", super::comply_cb_detect::detect_cb1000_missing_model_card(project_path)),
        ("CB-1001", super::comply_cb_detect::detect_cb1001_oversized_tensor_count(project_path)),
        ("CB-1002", super::comply_cb_detect::detect_cb1002_missing_tokenizer(project_path)),
        ("CB-1004", super::comply_cb_detect::detect_cb1004_missing_architecture(project_path)),
        ("CB-1005", super::comply_cb_detect::detect_cb1005_quantization_mismatch(project_path)),
        ("CB-1006", super::comply_cb_detect::detect_cb1006_sharded_without_index(project_path)),
        ("CB-1007", super::comply_cb_detect::detect_cb1007_excessive_file_size(project_path)),
        ("CB-1008", super::comply_cb_detect::detect_cb1008_apr_missing_crc(project_path)),
    ];

    aggregate_violations(
        "CB-1000: MLOps Model Quality (CB-1000 to CB-1008)",
        &detectors,
        comply_config,
        true,
    )
}

#[allow(dead_code)]
pub(crate) fn check_scala_best_practices(project_path: &Path) -> ComplianceCheck {
    check_scala_best_practices_with_config(project_path, None)
}

pub(crate) fn check_scala_best_practices_with_config(
    project_path: &Path,
    comply_config: Option<&ComplyConfig>,
) -> ComplianceCheck {
    let scala_files = super::comply_cb_detect::walkdir_scala_files(project_path);
    if scala_files.is_empty() {
        return ComplianceCheck {
            name: "CB-800: Scala Best Practices (CB-800 to CB-805)".to_string(),
            status: CheckStatus::Pass,
            message: "Not a Scala project (no .scala files found)".to_string(),
            severity: Severity::Info,
        };
    }

    let detectors: Vec<(&str, Vec<CbPatternViolation>)> = vec![
        ("CB-800", super::comply_cb_detect::detect_cb800_mutable_collection(project_path)),
        ("CB-801", super::comply_cb_detect::detect_cb801_null_usage(project_path)),
        ("CB-802", super::comply_cb_detect::detect_cb802_wildcard_import(project_path)),
        ("CB-803", super::comply_cb_detect::detect_cb803_return_statement(project_path)),
        ("CB-804", super::comply_cb_detect::detect_cb804_var_declaration(project_path)),
        ("CB-805", super::comply_cb_detect::detect_cb805_blocking_in_future(project_path)),
    ];

    aggregate_violations(
        "CB-800: Scala Best Practices (CB-800 to CB-805)",
        &detectors,
        comply_config,
        true,
    )
}

// Three-layer CLI (review/audit) extracted for file health (CB-040)
include!("review_audit_handlers.rs");

// Check handler tests extracted for file health (CB-040)
include!("check_handlers_tests.rs");
