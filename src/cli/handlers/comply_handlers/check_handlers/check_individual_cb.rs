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
            name: "CB-060: ComputeBrick Compliance".into(),
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
            name: "CB-060: ComputeBrick Compliance".into(),
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
            name: "CB-060: ComputeBrick Compliance".into(),
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
            name: "CB-060: ComputeBrick Compliance".into(),
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
            name: "CB-120: OIP Tarantula Patterns (CB-120 to CB-124)".into(),
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
            name: "CB-120: OIP Tarantula Patterns (CB-120 to CB-124)".into(),
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
        "CB-125: Coverage Quality Patterns (CB-125 to CB-127)",
        all_issues,
        critical,
        error,
        warning,
        "No coverage quality issues detected",
    )
}

#[cfg(test)]
mod check_individual_cb_tests {
    //! Covers pure-compute helpers in check_individual_cb.rs (94 uncov on
    //! broad, 0% cov). Skips fs-walking compute_brick / oip_tarantula
    //! detectors.
    use super::*;
    use crate::cli::handlers::comply_cb_detect::{
        CbPatternViolation, Severity as CbSev,
    };

    fn viol(pid: &str, sev: CbSev, desc: &str) -> CbPatternViolation {
        CbPatternViolation {
            pattern_id: pid.to_string(),
            file: "src/a.rs".to_string(),
            line: 1,
            description: desc.to_string(),
            severity: sev,
        }
    }

    // ── append_violation: formats "PID: description (file:line)" ──

    #[test]
    fn test_append_violation_format_matches_template() {
        let mut issues = Vec::new();
        let v = viol("CB-001", CbSev::Warning, "bounds check");
        append_violation(&mut issues, &v);
        assert_eq!(issues, vec!["CB-001: bounds check (src/a.rs:1)"]);
    }

    #[test]
    fn test_append_violation_accumulates() {
        let mut issues = vec!["existing".to_string()];
        let v = viol("CB-002", CbSev::Error, "barrier");
        append_violation(&mut issues, &v);
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[1], "CB-002: barrier (src/a.rs:1)");
    }

    // ── collect_violations_with_counts (critical vs warning via is_critical flag) ──

    #[test]
    fn test_collect_violations_with_counts_empty_detections_returns_zeros() {
        let (issues, critical, warning) = collect_violations_with_counts(&[]);
        assert!(issues.is_empty());
        assert_eq!(critical, 0);
        assert_eq!(warning, 0);
    }

    #[test]
    fn test_collect_violations_with_counts_sums_critical_and_warning() {
        let detections = vec![
            (vec![viol("A", CbSev::Error, "x"), viol("B", CbSev::Error, "y")], true),
            (vec![viol("C", CbSev::Warning, "z")], false),
        ];
        let (issues, critical, warning) = collect_violations_with_counts(&detections);
        assert_eq!(issues.len(), 3);
        assert_eq!(critical, 2);
        assert_eq!(warning, 1);
    }

    // ── build_cb_result: 3 arms (Fail on critical, Warn on warning, Pass) ──

    #[test]
    fn test_build_cb_result_critical_fails() {
        let c = build_cb_result(vec!["x".to_string()], 1, 0);
        assert!(matches!(c.status, CheckStatus::Fail));
        assert_eq!(c.severity, Severity::Critical);
        assert!(c.message.contains("1 critical"));
    }

    #[test]
    fn test_build_cb_result_warning_only_warns() {
        let c = build_cb_result(vec!["x".to_string()], 0, 2);
        assert!(matches!(c.status, CheckStatus::Warn));
        assert_eq!(c.severity, Severity::Warning);
        assert!(c.message.contains("2 warnings"));
    }

    #[test]
    fn test_build_cb_result_no_violations_passes() {
        let c = build_cb_result(vec![], 0, 0);
        assert!(matches!(c.status, CheckStatus::Pass));
        assert_eq!(c.severity, Severity::Info);
        assert!(c.message.contains("no violations"));
    }

    #[test]
    fn test_build_cb_result_critical_plus_warnings_still_fails() {
        // critical present → Fail regardless of warnings.
        let c = build_cb_result(vec!["x".to_string()], 2, 3);
        assert!(matches!(c.status, CheckStatus::Fail));
        assert_eq!(c.severity, Severity::Critical);
        assert!(c.message.contains("2 critical"));
        assert!(c.message.contains("3 warnings"));
    }

    // ── collect_triaged_violations: classifies by CbSev → (critical, error, warning) ──

    #[test]
    fn test_collect_triaged_violations_splits_by_severity() {
        let sets = vec![vec![
            viol("A", CbSev::Critical, "cc"),
            viol("B", CbSev::Critical, "cd"),
            viol("C", CbSev::Error, "ee"),
            viol("D", CbSev::Warning, "ww"),
            // Info falls into the `_ => warning` arm.
            viol("E", CbSev::Info, "ii"),
        ]];
        let (issues, critical, error, warning) = collect_triaged_violations(&sets);
        assert_eq!(issues.len(), 5);
        assert_eq!(critical, 2);
        assert_eq!(error, 1);
        assert_eq!(warning, 2, "Warning + Info both count as warning");
    }

    // ── build_triaged_check: 4 arms (Fail-critical, Fail-error, Warn, Pass) ──

    #[test]
    fn test_build_triaged_check_critical_fails_critical() {
        let c = build_triaged_check("Test", vec!["x".into()], 1, 0, 0, "ok");
        assert!(matches!(c.status, CheckStatus::Fail));
        assert_eq!(c.severity, Severity::Critical);
        assert_eq!(c.name, "Test");
    }

    #[test]
    fn test_build_triaged_check_error_fails_error() {
        let c = build_triaged_check("Test", vec!["x".into()], 0, 1, 0, "ok");
        assert!(matches!(c.status, CheckStatus::Fail));
        assert_eq!(c.severity, Severity::Error);
    }

    #[test]
    fn test_build_triaged_check_warning_warns() {
        let c = build_triaged_check("Test", vec!["x".into()], 0, 0, 1, "ok");
        assert!(matches!(c.status, CheckStatus::Warn));
        assert_eq!(c.severity, Severity::Warning);
    }

    #[test]
    fn test_build_triaged_check_no_violations_passes_with_msg() {
        let c = build_triaged_check("Test", vec![], 0, 0, 0, "all-good");
        assert!(matches!(c.status, CheckStatus::Pass));
        assert_eq!(c.severity, Severity::Info);
        assert_eq!(c.message, "all-good");
    }

    // ── collect_cb_violations: thin wrapper over multiple detectors ──
    // collect_cb_violations runs fs-walking detectors so we just verify it
    // doesn't panic on an empty tempdir (all detectors → empty result).

    #[test]
    fn test_collect_cb_violations_empty_project_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        // has_probar=false + has_brick_dir=false skips the detectors that
        // require those fixtures — empty project → empty result.
        let (issues, critical, warning) = collect_cb_violations(tmp.path(), false, false);
        assert!(issues.is_empty());
        assert_eq!(critical, 0);
        assert_eq!(warning, 0);
    }
}

