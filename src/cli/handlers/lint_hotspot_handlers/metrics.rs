#![cfg_attr(coverage_nightly, coverage(off))]
//! Hotspot metrics, scoring, enforcement, and quality gates

use super::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Find the file with highest defect density (including detailed violations).
///
/// Returns `Ok(None)` when no file carried a violation — a measured "nothing
/// found", which the caller renders as an explicitly empty result rather than
/// as an error.
///
/// # Errors
///
/// Returns an error if the operation fails
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn find_hotspot_with_details(
    file_metrics: HashMap<PathBuf, FileMetrics>,
) -> Result<Option<LintHotspot>> {
    let debug = std::env::var("LINT_HOTSPOT_DEBUG").is_ok();
    if debug {
        eprintln!("🔍 Finding hotspot from {} files", file_metrics.len());
    }

    // DETERMINISM: iterating the HashMap directly and keeping the first file to
    // reach `max_density` meant that two files with equal density produced a
    // different hotspot from run to run on unchanged input. Visit in path order
    // and break ties on the path.
    let mut entries: Vec<(PathBuf, FileMetrics)> = file_metrics.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hotspot_file: Option<LintHotspot> = None;
    let mut max_density = 0.0;

    for (file_path, metrics) in entries {
        if debug {
            eprintln!(
                "  File: {}, SLOC: {}, Errors: {}, Warnings: {}",
                file_path.display(),
                metrics.sloc,
                metrics.severity_counts.error,
                metrics.severity_counts.warning
            );
        }

        if metrics.sloc == 0 {
            continue;
        }

        let total_violations = calculate_total_violations(&metrics);
        if total_violations == 0 {
            continue;
        }

        let density = calculate_defect_density(total_violations, metrics.sloc);

        if density > max_density {
            max_density = density;

            let mut top_lints: Vec<_> = metrics.violations.into_iter().collect();
            // DETERMINISM: ties previously fell back to HashMap order.
            top_lints.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            top_lints.truncate(10);

            let mut detailed_violations = metrics.detailed_violations;
            sort_violations(&mut detailed_violations);

            hotspot_file = Some(LintHotspot {
                file: file_path,
                defect_density: density,
                total_violations,
                sloc: metrics.sloc,
                severity_distribution: metrics.severity_counts,
                top_lints,
                detailed_violations,
            });
        }
    }

    Ok(hotspot_file)
}

/// Stable ordering for a violation list so identical input renders identically.
pub(crate) fn sort_violations(violations: &mut [ViolationDetail]) {
    violations.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.column.cmp(&b.column))
            .then_with(|| a.lint_name.cmp(&b.lint_name))
            .then_with(|| a.message.cmp(&b.message))
    });
}

/// Calculate enforcement metadata
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn calculate_enforcement_metadata(
    hotspot: &LintHotspot,
    min_confidence: f64,
) -> EnforcementMetadata {
    // Simple heuristic: higher density = higher priority
    let enforcement_score = (hotspot.defect_density * 10.0).min(10.0);
    let enforcement_priority = (enforcement_score as u8).max(1);

    // Estimate fix time based on violations (5 minutes per violation)
    let estimated_fix_time = (hotspot.total_violations as u32) * 300;

    // Confidence based on lint types (some are easier to automate)
    let automation_confidence = if hotspot
        .top_lints
        .iter()
        .any(|(lint, _)| lint.contains("unused") || lint.contains("redundant"))
    {
        0.9
    } else {
        0.7
    };

    EnforcementMetadata {
        enforcement_score,
        requires_enforcement: enforcement_score >= 7.0 && automation_confidence >= min_confidence,
        estimated_fix_time,
        automation_confidence,
        enforcement_priority,
    }
}

/// Generate refactor chain for automated fixes
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn generate_refactor_chain(hotspot: &LintHotspot, min_confidence: f64) -> RefactorChain {
    let mut steps = Vec::new();
    let mut total_impact = 0;

    for (lint_code, count) in &hotspot.top_lints {
        let (confidence, description) = match lint_code.as_str() {
            s if s.contains("unused") => (0.95, "Remove unused code"),
            s if s.contains("redundant") => (0.90, "Remove redundant code"),
            s if s.contains("needless") => (0.85, "Simplify needless patterns"),
            s if s.contains("too_many_arguments") => (0.80, "Extract context objects"),
            _ => (0.70, "Apply clippy suggestion"),
        };

        if confidence >= min_confidence {
            steps.push(RefactorStep {
                id: format!("fix-{lint_code}"),
                lint: lint_code.clone(),
                confidence,
                impact: *count,
                description: description.to_string(),
            });
            total_impact += count;
        }
    }

    // DETERMINISM: the id used to embed `Utc::now()`, so the same input never
    // produced the same output twice and no baseline could be diffed. It is now
    // derived from the file the chain is for.
    let id = format!("lint-hotspot-{}", hotspot.file.display());

    // A chain with no steps automates nothing; the old expression was
    // `sum / steps.len()` = 0/0 = NaN, which serde renders as JSON `null`.
    let automation_confidence = if steps.is_empty() {
        0.0
    } else {
        steps.iter().map(|s| s.confidence).sum::<f64>() / steps.len() as f64
    };

    RefactorChain {
        id,
        estimated_reduction: total_impact,
        automation_confidence,
        steps,
    }
}

/// Check quality gates
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn check_quality_gates(hotspot: &LintHotspot, max_density: f64) -> QualityGateStatus {
    let mut violations = Vec::new();

    if hotspot.defect_density > max_density {
        violations.push(QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: max_density,
            actual: hotspot.defect_density,
            severity: "blocking".to_string(),
        });
    }

    if hotspot.total_violations > 50 {
        violations.push(QualityViolation {
            rule: "max_single_file_violations".to_string(),
            threshold: 50.0,
            actual: hotspot.total_violations as f64,
            severity: "warning".to_string(),
        });
    }

    let passed = violations.is_empty();
    let blocking = violations.iter().any(|v| v.severity == "blocking");

    QualityGateStatus {
        passed,
        violations,
        blocking,
    }
}

/// Build final `LintHotspotResult` from file metrics (cognitive complexity <=8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn build_lint_hotspot_result(
    file_metrics: HashMap<PathBuf, FileMetrics>,
) -> Result<Option<LintHotspotResult>> {
    let (all_violations, summary_by_file, total_project_violations) =
        collect_project_violations(&file_metrics);

    let Some(hotspot) = find_hotspot_with_details(file_metrics)? else {
        return Ok(None);
    };

    Ok(Some(LintHotspotResult {
        hotspot,
        all_violations,
        summary_by_file,
        total_project_violations,
        enforcement: None,
        refactor_chain: None,
        quality_gate: QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        },
    }))
}

/// Collect all violations across the project (cognitive complexity <=7)
fn collect_project_violations(
    file_metrics: &HashMap<PathBuf, FileMetrics>,
) -> (Vec<ViolationDetail>, HashMap<PathBuf, FileSummary>, usize) {
    let mut all_violations = Vec::new();
    let mut summary_by_file = HashMap::new();
    let mut total_project_violations = 0;

    for (file_path, metrics) in file_metrics {
        all_violations.extend(metrics.detailed_violations.clone());

        let total_file_violations = calculate_total_violations(metrics);
        total_project_violations += total_file_violations;

        let defect_density = calculate_defect_density(total_file_violations, metrics.sloc);

        summary_by_file.insert(
            file_path.clone(),
            FileSummary {
                total_violations: total_file_violations,
                errors: metrics.severity_counts.error,
                warnings: metrics.severity_counts.warning,
                sloc: metrics.sloc,
                defect_density,
            },
        );
    }

    // DETERMINISM: `all_violations` was built by iterating a HashMap, so the
    // JSON array came out in a different order on every run for the same tree.
    sort_violations(&mut all_violations);

    (all_violations, summary_by_file, total_project_violations)
}

/// Calculate total violations for a file (cognitive complexity <=2)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn calculate_total_violations(metrics: &FileMetrics) -> usize {
    metrics.severity_counts.error
        + metrics.severity_counts.warning
        + metrics.severity_counts.suggestion
}

/// Calculate defect density (cognitive complexity <=2)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn calculate_defect_density(violations: usize, sloc: usize) -> f64 {
    if sloc > 0 {
        violations as f64 / sloc as f64
    } else {
        0.0
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ===================
    // calculate_total_violations Tests
    // ===================

    #[test]
    fn test_calculate_total_violations_all_types() {
        let metrics = FileMetrics {
            severity_counts: SeverityDistribution {
                error: 2,
                warning: 3,
                suggestion: 1,
                note: 5, // Note is not counted
            },
            sloc: 100,
            violations: HashMap::new(),
            detailed_violations: vec![],
        };
        // error + warning + suggestion = 2 + 3 + 1 = 6
        assert_eq!(calculate_total_violations(&metrics), 6);
    }

    #[test]
    fn test_calculate_total_violations_zeros() {
        let metrics = FileMetrics {
            severity_counts: SeverityDistribution::default(),
            sloc: 50,
            violations: HashMap::new(),
            detailed_violations: vec![],
        };
        assert_eq!(calculate_total_violations(&metrics), 0);
    }

    #[test]
    fn test_calculate_total_violations_only_errors() {
        let metrics = FileMetrics {
            severity_counts: SeverityDistribution {
                error: 10,
                warning: 0,
                suggestion: 0,
                note: 0,
            },
            sloc: 100,
            violations: HashMap::new(),
            detailed_violations: vec![],
        };
        assert_eq!(calculate_total_violations(&metrics), 10);
    }

    // ===================
    // calculate_defect_density Tests
    // ===================

    #[test]
    fn test_calculate_defect_density_normal() {
        // 10 violations / 100 SLOC = 0.1
        let density = calculate_defect_density(10, 100);
        assert!((density - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_calculate_defect_density_zero_sloc() {
        // Should return 0.0 to avoid division by zero
        let density = calculate_defect_density(5, 0);
        assert_eq!(density, 0.0);
    }

    #[test]
    fn test_calculate_defect_density_zero_violations() {
        let density = calculate_defect_density(0, 100);
        assert_eq!(density, 0.0);
    }

    #[test]
    fn test_calculate_defect_density_high() {
        // 50 violations / 10 SLOC = 5.0
        let density = calculate_defect_density(50, 10);
        assert!((density - 5.0).abs() < 0.001);
    }

    // ===================
    // check_quality_gates Tests
    // ===================

    #[test]
    fn test_check_quality_gates_pass() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/test.rs"),
            defect_density: 0.05, // Below max
            total_violations: 10, // Below 50
            sloc: 200,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: vec![],
        };

        let status = check_quality_gates(&hotspot, 0.1);
        assert!(status.passed);
        assert!(!status.blocking);
        assert!(status.violations.is_empty());
    }

    #[test]
    fn test_check_quality_gates_fail_density() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/test.rs"),
            defect_density: 0.15, // Above max of 0.1
            total_violations: 30,
            sloc: 200,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: vec![],
        };

        let status = check_quality_gates(&hotspot, 0.1);
        assert!(!status.passed);
        assert!(status.blocking);
        assert_eq!(status.violations.len(), 1);
        assert_eq!(status.violations[0].rule, "max_defect_density");
    }

    /// #699: the shipped default must be able to fail a realistically dirty
    /// file. It was 5.0 — compared against `violations / sloc`, so it required
    /// 5 lint violations on every single line. Observed on a real fixture run:
    /// the hotspot measured `defect_density: 2.0` (6 violations over 3 SLOC,
    /// i.e. 200 violations per 100 lines) and the gate reported
    /// `"passed": true, "blocking": false` with exit 0.
    #[test]
    fn test_default_max_density_fails_a_measurably_dirty_hotspot() {
        let hotspot = LintHotspot {
            file: PathBuf::from("main.rs"),
            defect_density: 2.0,
            total_violations: 6,
            sloc: 3,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: vec![],
        };

        let status = check_quality_gates(&hotspot, DEFAULT_MAX_DENSITY);
        assert!(
            !status.passed,
            "a hotspot at 200 violations per 100 lines must not pass the default gate"
        );
        assert!(status.blocking);
        assert_eq!(status.violations[0].rule, "max_defect_density");
        assert_eq!(status.violations[0].threshold, DEFAULT_MAX_DENSITY);
    }

    /// The default is expressed in the unit the gate actually compares in
    /// (`violations / sloc`), and stays reachable: a clean file must pass.
    #[test]
    fn test_default_max_density_passes_a_clean_hotspot() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/lib.rs"),
            defect_density: 0.01, // 1 violation per 100 lines
            total_violations: 2,
            sloc: 200,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: vec![],
        };

        let status = check_quality_gates(&hotspot, DEFAULT_MAX_DENSITY);
        assert!(status.passed);
        assert!(!status.blocking);
    }

    #[test]
    fn test_check_quality_gates_fail_violations() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/test.rs"),
            defect_density: 0.05,
            total_violations: 55, // Above 50
            sloc: 1100,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: vec![],
        };

        let status = check_quality_gates(&hotspot, 0.1);
        assert!(!status.passed);
        assert!(!status.blocking); // Warning, not blocking
        assert_eq!(status.violations.len(), 1);
        assert_eq!(status.violations[0].rule, "max_single_file_violations");
    }

    #[test]
    fn test_check_quality_gates_fail_both() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/test.rs"),
            defect_density: 0.5,   // Way above max
            total_violations: 100, // Above 50
            sloc: 200,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: vec![],
        };

        let status = check_quality_gates(&hotspot, 0.1);
        assert!(!status.passed);
        assert!(status.blocking);
        assert_eq!(status.violations.len(), 2);
    }

    // ===================
    // calculate_enforcement_metadata Tests
    // ===================

    #[test]
    fn test_calculate_enforcement_metadata_low_density() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/test.rs"),
            defect_density: 0.3,
            total_violations: 5,
            sloc: 100,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: vec![],
        };

        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);
        assert!((metadata.enforcement_score - 3.0).abs() < 0.1);
        assert_eq!(metadata.estimated_fix_time, 5 * 300); // 5 violations * 300 seconds
        assert!(!metadata.requires_enforcement); // Score < 7.0
    }

    #[test]
    fn test_calculate_enforcement_metadata_high_density_with_unused() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/test.rs"),
            defect_density: 0.8,
            total_violations: 20,
            sloc: 25,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![("unused_variable".to_string(), 10)],
            detailed_violations: vec![],
        };

        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);
        assert!((metadata.enforcement_score - 8.0).abs() < 0.1);
        assert_eq!(metadata.automation_confidence, 0.9); // Has "unused" lint
        assert!(metadata.requires_enforcement); // Score >= 7.0 and confidence >= 0.7
    }

    #[test]
    fn test_calculate_enforcement_metadata_high_density_no_easy_fixes() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/test.rs"),
            defect_density: 1.0,
            total_violations: 30,
            sloc: 30,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![("clippy::too_many_arguments".to_string(), 15)],
            detailed_violations: vec![],
        };

        let metadata = calculate_enforcement_metadata(&hotspot, 0.8);
        assert!((metadata.enforcement_score - 10.0).abs() < 0.1); // Capped at 10.0
        assert_eq!(metadata.automation_confidence, 0.7); // No "unused" or "redundant"
        assert!(!metadata.requires_enforcement); // Confidence 0.7 < min 0.8
    }

    // ===================
    // Determinism (identical input ⇒ identical output)
    // ===================

    fn tied_file_metrics() -> HashMap<PathBuf, FileMetrics> {
        let mut map = HashMap::new();
        for name in [
            "src/alpha.rs",
            "src/beta.rs",
            "src/gamma.rs",
            "src/delta.rs",
            "src/epsilon.rs",
            "src/zeta.rs",
            "src/eta.rs",
            "src/theta.rs",
        ] {
            let mut violations = HashMap::new();
            // Two lints with an identical count: sorting by count alone leaves
            // their relative order to the HashMap.
            violations.insert("clippy::aaa".to_string(), 2);
            violations.insert("clippy::bbb".to_string(), 2);
            map.insert(
                PathBuf::from(name),
                FileMetrics {
                    severity_counts: SeverityDistribution {
                        error: 1,
                        warning: 3,
                        suggestion: 0,
                        note: 0,
                    },
                    sloc: 100,
                    violations,
                    detailed_violations: vec![ViolationDetail {
                        file: PathBuf::from(name),
                        line: 1,
                        column: 1,
                        end_line: 1,
                        end_column: 2,
                        lint_name: "clippy::aaa".to_string(),
                        message: "m".to_string(),
                        severity: "warning".to_string(),
                        suggestion: None,
                        machine_applicable: false,
                    }],
                },
            );
        }
        map
    }

    /// Five builds over identical input must agree on the hotspot file, the
    /// `top_lints` order and the `all_violations` order. Before the fix all
    /// three came out of a `HashMap` in per-process hash order.
    #[test]
    fn test_build_lint_hotspot_result_is_deterministic_across_five_runs() {
        let mut baseline: Option<String> = None;
        for run in 0..5 {
            let result = build_lint_hotspot_result(tied_file_metrics()).unwrap();
            let json = serde_json::to_string(&result).unwrap();
            match &baseline {
                None => baseline = Some(json),
                Some(first) => {
                    assert_eq!(first, &json, "run {run} differed for identical input")
                }
            }
        }
    }

    /// The refactor-chain id used to embed `Utc::now()`, so `enforcement-json`
    /// changed every second for the same input.
    #[test]
    fn test_refactor_chain_id_is_input_derived_not_clock_derived() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/deep/nested/thing.rs"),
            defect_density: 0.5,
            total_violations: 7,
            sloc: 14,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![("clippy::unused_self".to_string(), 7)],
            detailed_violations: vec![],
        };
        let first = generate_refactor_chain(&hotspot, 0.5).id;
        let second = generate_refactor_chain(&hotspot, 0.5).id;
        assert_eq!(first, second);
        // The full path, not the basename: two agents proposed different
        // schemes and basenames collide across directories (src/a/mod.rs and
        // src/b/mod.rs would share an id). What the test is really pinning is
        // that the id is derived from the INPUT and not from a clock -- which
        // is the assert_eq! above.
        assert_eq!(first, "lint-hotspot-src/deep/nested/thing.rs");
    }
}
