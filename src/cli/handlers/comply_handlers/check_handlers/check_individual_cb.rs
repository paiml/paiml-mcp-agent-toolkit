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

