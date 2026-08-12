#![cfg_attr(coverage_nightly, coverage(off))]
//! Quality analysis functions for enforcement

use super::types::{PhaseOutcome, QualityProfile, QualityViolation};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Per-phase analysis scope for enforcement.
///
/// When `--file` is given, phases with native single-file support
/// (complexity, TDG) analyze exactly that file, while phases that must walk
/// a directory tree (SATD, dead code, duplication) fall back to the file's
/// parent module directory instead of scanning the whole project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisScope {
    /// Analyze the whole project rooted at this path
    Project { root: PathBuf },
    /// Analyze a single file; `module_dir` is the containing directory used
    /// by phases that cannot operate on a lone file
    SingleFile { file: PathBuf, module_dir: PathBuf },
}

impl AnalysisScope {
    /// Resolve scope from the project root and the optional `--file` argument.
    /// Relative file paths are resolved against the project root.
    #[must_use]
    pub fn resolve(project_path: &Path, specific_file: Option<&Path>) -> Self {
        match specific_file {
            None => Self::Project {
                root: project_path.to_path_buf(),
            },
            Some(f) => {
                let file = if f.is_absolute() {
                    f.to_path_buf()
                } else {
                    project_path.join(f)
                };
                let module_dir = file
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .map_or_else(|| project_path.to_path_buf(), Path::to_path_buf);
                Self::SingleFile { file, module_dir }
            }
        }
    }

    /// Exact file under analysis, when scoped to a single file
    #[must_use]
    pub fn single_file(&self) -> Option<&Path> {
        match self {
            Self::Project { .. } => None,
            Self::SingleFile { file, .. } => Some(file),
        }
    }

    /// Root for phases that must walk a directory (SATD, dead code,
    /// duplication): the project root, or the file's parent module dir
    #[must_use]
    pub fn walk_root(&self) -> &Path {
        match self {
            Self::Project { root } => root,
            Self::SingleFile { module_dir, .. } => module_dir,
        }
    }

    /// Path for phases that accept either a file or a directory (TDG)
    #[must_use]
    pub fn file_or_root(&self) -> &Path {
        match self {
            Self::Project { root } => root,
            Self::SingleFile { file, .. } => file,
        }
    }
}

/// Scratch path used to capture a printing analysis handler's JSON report.
///
/// `enforce` used to invoke these handlers purely for their side effects and
/// then push a hardcoded `QualityViolation`. The consequence was that
/// `enforce extreme --list-violations` returned the SAME cast of violations
/// for an empty directory as for this repository — pointing at
/// `server/src/cli/handlers/enforce_handlers.rs`, a path that exists in no
/// project — and a literal `coverage 65.0`. A violation must come from the
/// analysis that was actually run, so the handler's own JSON is captured and
/// parsed instead of discarded.
fn capture_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "pmat-enforce-{tag}-{}-{nanos}.json",
        std::process::id()
    ))
}

/// Read and delete a captured JSON report.
///
/// `None` means the analysis produced nothing readable — the caller must then
/// report no violations rather than substitute a constant.
fn take_captured_json(path: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(path).ok();
    let _ = std::fs::remove_file(path);
    serde_json::from_str(&text?).ok()
}

/// Report that a signal could not be produced, and hand the reason back.
///
/// Printing was all this used to do, which is why the gap never reached the
/// verdict: stderr is not a return value. The reason now travels with the
/// `PhaseOutcome` so the score can exclude the phase and the report can name it.
fn warn_not_measured(kind: &str, path: &Path, reason: &str) -> String {
    eprintln!("⚠️  {kind} not measured for {}: {reason}", path.display());
    format!("{kind} not measured: {reason}")
}

/// Run complexity analysis - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_complexity_analysis(
    project_path: &Path,
    profile: &QualityProfile,
    specific_file: Option<&Path>,
) -> Result<PhaseOutcome> {
    // The complexity service is called directly rather than through the
    // printing CLI handler: the handler returns `()`, so its result had to be
    // thrown away and a sample violation invented in its place.
    let file_metrics = match specific_file {
        Some(file) => {
            match crate::services::complexity::analyze_file_complexity_uncached(file, None).await {
                Ok(m) => vec![m],
                Err(e) => {
                    let reason = warn_not_measured("complexity", file, &e.to_string());
                    return Ok(PhaseOutcome::unmeasured(reason));
                }
            }
        }
        None => {
            match crate::cli::analysis_utilities::analyze_project_files(
                project_path,
                None,
                &[],
                profile.complexity_max,
                profile.complexity_max,
            )
            .await
            {
                Ok(m) => m,
                Err(e) => {
                    let reason = warn_not_measured("complexity", project_path, &e.to_string());
                    return Ok(PhaseOutcome::unmeasured(reason));
                }
            }
        }
    };

    // Zero analysable files is not a clean result — it is the absence of one.
    // The analyzers below return `Ok` for input they never read (a nonexistent
    // path yields an empty metric set, not an error), so without this the phase
    // reports "measured, no violations" for a tree it never opened.
    if file_metrics.is_empty() {
        return Ok(PhaseOutcome::unmeasured(format!(
            "no analysable source files under {}",
            project_path.display()
        )));
    }

    let severe = profile.complexity_max.saturating_mul(2);
    let mut violations = Vec::new();
    for file in &file_metrics {
        for func in &file.functions {
            if func.metrics.cyclomatic > profile.complexity_max {
                violations.push(QualityViolation {
                    violation_type: "complexity".to_string(),
                    severity: if func.metrics.cyclomatic > severe {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    location: format!("{}:{}:{}", file.path, func.line_start, func.name),
                    current: f64::from(func.metrics.cyclomatic),
                    target: f64::from(profile.complexity_max),
                    suggestion: "Extract method pattern - split the function into smaller units"
                        .to_string(),
                });
            }
        }
    }

    // Deterministic order: worst first, then by location.
    violations.sort_by(|a, b| {
        b.current
            .total_cmp(&a.current)
            .then_with(|| a.location.cmp(&b.location))
    });

    Ok(PhaseOutcome::measured(violations))
}

/// SATD for exactly one file, for `--file` scope.
///
/// Shares `satd_violation` with the project path so the two cannot drift in what
/// they report; only the set of files differs.
fn single_file_satd(
    detector: &crate::services::satd_detector::SATDDetector,
    file: &Path,
    profile: &QualityProfile,
) -> Result<PhaseOutcome> {
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            let reason = warn_not_measured("satd", file, &e.to_string());
            return Ok(PhaseOutcome::unmeasured(reason));
        }
    };
    let debts = match detector.extract_from_content(&content, file) {
        Ok(d) => d,
        Err(e) => {
            let reason = warn_not_measured("satd", file, &e.to_string());
            return Ok(PhaseOutcome::unmeasured(reason));
        }
    };

    if debts.len() <= profile.satd_allowed {
        return Ok(PhaseOutcome::measured(Vec::new()));
    }
    Ok(PhaseOutcome::measured(
        debts
            .iter()
            .map(|item| satd_violation(item, debts.len(), profile))
            .collect(),
    ))
}

/// Run SATD analysis - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// This used to call the *printing* `analyze satd` handler purely for its side
/// effect, discard its `()`, and leave the branch that should have produced
/// violations holding nothing but the comment "For now, we know project
/// maintains zero SATD". `let violations = Vec::new()` was returned unmodified
/// on every path, so SATD could not contribute a violation under any input:
/// `pmat enforce extreme` printed "Found 40 SATD items" and then reported
/// `State: Complete, Score: 1.00/1.00, Violations: 0` — an enforcer that had
/// stopped enforcing while still looking like it worked.
///
/// It now calls the detector directly, the way `run_tdg_analysis` does, and
/// turns what it finds into violations against `profile.satd_allowed`.
pub async fn run_satd_analysis(
    project_path: &Path,
    profile: &QualityProfile,
    specific_file: Option<&Path>,
) -> Result<PhaseOutcome> {
    use crate::services::satd_detector::SATDDetector;

    let detector = SATDDetector::new();

    // `--file` means THIS file. `AnalysisScope::walk_root` hands directory
    // phases the file's parent module dir, on the assumption that SATD "must
    // walk a directory" — but the detector reads content, so a single file is
    // exactly what it can take. The parent-dir fallback attributed a sibling's
    // `// TODO` to the named file: `enforce extreme --file good.rs --ci-mode`
    // exited 1 reporting one violation whose own location was `bad.rs`, so a
    // clean file failed CI on code it does not contain.
    if let Some(file) = specific_file {
        return single_file_satd(&detector, file, profile);
    }

    // `include_tests: false` — SATD in test code is not production debt, which
    // matches what the SATD gate in `pmat verify` counts.
    let result = match detector.analyze_project(project_path, false).await {
        Ok(result) => result,
        // A path that cannot be analysed is not evidence of zero debt. This
        // used to `bail!`, which was the strongest thing the old
        // `Result<Vec<_>>` signature could express; `unmeasured` says it exactly
        // — the run continues, and the verdict discloses the gap.
        Err(e) => {
            let reason = warn_not_measured("satd", project_path, &e.to_string());
            return Ok(PhaseOutcome::unmeasured(reason));
        }
    };

    if result.total_files_analyzed == 0 {
        return Ok(PhaseOutcome::unmeasured(format!(
            "no analysable source files under {}",
            project_path.display()
        )));
    }

    let found = result.summary.total_items;
    if found <= profile.satd_allowed {
        return Ok(PhaseOutcome::measured(Vec::new()));
    }

    // One violation per item, each locatable, so `--format json` consumers can
    // act on them instead of being told only that a count was exceeded.
    Ok(PhaseOutcome::measured(
        result
            .items
            .iter()
            .map(|item| satd_violation(item, found, profile))
            .collect(),
    ))
}

/// One SATD violation. Shared by the project and `--file` paths so the two
/// cannot drift in what they report; only the file set differs.
fn satd_violation(
    item: &crate::services::satd_detector::TechnicalDebt,
    found: usize,
    profile: &QualityProfile,
) -> QualityViolation {
    QualityViolation {
        violation_type: "satd".to_string(),
        severity: format!("{:?}", item.severity).to_lowercase(),
        location: format!("{}:{}:{}", item.file.display(), item.line, item.column),
        current: found as f64,
        target: profile.satd_allowed as f64,
        suggestion: format!(
            "Resolve the {:?} debt marker ({}) or track it outside the source",
            item.category,
            item.text.trim()
        ),
    }
}

/// Run TDG analysis - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_tdg_analysis(
    project_path: &Path,
    profile: &QualityProfile,
) -> Result<PhaseOutcome> {
    use crate::services::tdg_calculator::TDGCalculator;

    // `TDGCalculator` is the analyser whose scale `profile.tdg_max` is written
    // against (0.0-5.0, lower is better). The previous code called the
    // printing `analyze tdg` handler, dropped its `()` result and pushed a
    // literal `2.3` for a file that does not exist in any checkout.
    let calculator = TDGCalculator::new();
    let mut violations = Vec::new();

    if project_path.is_dir() {
        match calculator.analyze_directory(project_path).await {
            Ok(summary) => {
                for hotspot in &summary.hotspots {
                    if hotspot.tdg_score > profile.tdg_max {
                        violations.push(QualityViolation {
                            violation_type: "tdg".to_string(),
                            severity: if hotspot.tdg_score > profile.tdg_max * 2.0 {
                                "high".to_string()
                            } else {
                                "medium".to_string()
                            },
                            location: hotspot.path.clone(),
                            current: hotspot.tdg_score,
                            target: profile.tdg_max,
                            suggestion: format!(
                                "Reduce technical debt - primary factor: {}",
                                hotspot.primary_factor
                            ),
                        });
                    }
                }
            }
            Err(e) => {
                let reason = warn_not_measured("tdg", project_path, &e.to_string());
                return Ok(PhaseOutcome::unmeasured(reason));
            }
        }
    } else {
        match calculator.calculate_file(project_path).await {
            Ok(score) => {
                if score.value > profile.tdg_max {
                    violations.push(QualityViolation {
                        violation_type: "tdg".to_string(),
                        severity: if score.value > profile.tdg_max * 2.0 {
                            "high".to_string()
                        } else {
                            "medium".to_string()
                        },
                        location: project_path.display().to_string(),
                        current: score.value,
                        target: profile.tdg_max,
                        suggestion: "Refactor high-complexity functions to reduce technical debt"
                            .to_string(),
                    });
                }
            }
            Err(e) => {
                let reason = warn_not_measured("tdg", project_path, &e.to_string());
                return Ok(PhaseOutcome::unmeasured(reason));
            }
        }
    }

    Ok(PhaseOutcome::measured(violations))
}

/// Run dead code analysis - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_dead_code_analysis(
    project_path: &Path,
    _profile: &QualityProfile,
) -> Result<PhaseOutcome> {
    use crate::cli::handlers::dead_code_handlers::handle_analyze_dead_code;
    use crate::cli::DeadCodeOutputFormat;

    let mut violations = Vec::new();

    // The handler's JSON is captured to a scratch file and parsed. Discarding
    // it and pushing a literal is what made every project report dead code in
    // `server/src/services/ast_typescript_dispatch.rs:9`, a file no checkout
    // contains.
    let capture = capture_path("dead-code");
    let ran = handle_analyze_dead_code(
        project_path.to_path_buf(),
        DeadCodeOutputFormat::Json,
        Some(10),              // top_files
        true,                  // include_unreachable
        5,                     // min_dead_lines
        false,                 // include_tests
        Some(capture.clone()), // output
        false,                 // fail_on_violation
        15.0,                  // max_percentage
        60,                    // timeout
        Vec::new(),            // include
        Vec::new(),            // exclude
        8,                     // max_depth
    )
    .await;

    match ran {
        Ok(()) => match take_captured_json(&capture) {
            Some(report) => {
                let files = report.get("files").and_then(serde_json::Value::as_array);
                for file in files.into_iter().flatten() {
                    let dead_lines = file
                        .get("dead_lines")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0);
                    if dead_lines <= 0.0 {
                        continue;
                    }
                    let path = file
                        .get("path")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("<unknown>");
                    violations.push(QualityViolation {
                        violation_type: "dead_code".to_string(),
                        severity: "low".to_string(),
                        location: path.to_string(),
                        current: dead_lines,
                        target: 0.0,
                        suggestion: "Remove dead code attributes and unused functions".to_string(),
                    });
                }
            }
            None => {
                let reason =
                    warn_not_measured("dead code", project_path, "no parsable JSON report");
                return Ok(PhaseOutcome::unmeasured(reason));
            }
        },
        Err(e) => {
            let _ = std::fs::remove_file(&capture);
            let reason = warn_not_measured("dead code", project_path, &e.to_string());
            return Ok(PhaseOutcome::unmeasured(reason));
        }
    }

    Ok(PhaseOutcome::measured(violations))
}

/// Run duplication analysis - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_duplication_analysis(
    project_path: &Path,
    profile: &QualityProfile,
) -> Result<PhaseOutcome> {
    use crate::cli::handlers::duplication_analysis::{
        handle_analyze_duplicates, DuplicateAnalysisConfig,
    };
    use crate::cli::{DuplicateOutputFormat, DuplicateType};

    let mut violations = Vec::new();

    // Capture the detector's own JSON. The literal `current: 15.0` that used to
    // be pushed here was reported for an empty directory as readily as for a
    // real tree, which is the same defect as the complexity/TDG samples above.
    let capture = capture_path("duplication");
    let dup_config = DuplicateAnalysisConfig {
        project_path: project_path.to_path_buf(),
        detection_type: DuplicateType::Exact,
        threshold: 0.8,
        min_lines: 10,
        max_tokens: 100,
        format: DuplicateOutputFormat::Json,
        perf: false,
        include: None,
        exclude: None,
        output: Some(capture.clone()),
        top_files: 0, // 0 = all files
    };

    match handle_analyze_duplicates(dup_config).await {
        Ok(()) => match take_captured_json(&capture) {
            Some(report) => {
                let duplicate_lines = report
                    .get("duplicate_lines")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);
                if duplicate_lines > profile.duplication_max_lines as f64 {
                    let percentage = report
                        .get("duplication_percentage")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0);
                    violations.push(QualityViolation {
                        violation_type: "duplication".to_string(),
                        severity: if percentage >= 10.0 { "medium" } else { "low" }.to_string(),
                        location: format!("{} ({percentage:.1}% of lines)", project_path.display()),
                        current: duplicate_lines,
                        target: profile.duplication_max_lines as f64,
                        suggestion: "Extract common code into shared utilities".to_string(),
                    });
                }
            }
            None => {
                let reason =
                    warn_not_measured("duplication", project_path, "no parsable JSON report");
                return Ok(PhaseOutcome::unmeasured(reason));
            }
        },
        Err(e) => {
            let _ = std::fs::remove_file(&capture);
            let reason = warn_not_measured("duplication", project_path, &e.to_string());
            return Ok(PhaseOutcome::unmeasured(reason));
        }
    }

    Ok(PhaseOutcome::measured(violations))
}

/// Overall line coverage the project has already measured, if any.
///
/// pmat does not run a coverage tool itself, so the only honest source is an
/// lcov report the project produced (`cargo llvm-cov --lcov --output-path
/// lcov.info`). `None` means nothing was measured.
fn read_measured_line_coverage(project_path: &Path) -> Option<f64> {
    const CANDIDATES: [&str; 5] = [
        "lcov.info",
        "coverage/lcov.info",
        "target/coverage/lcov.info",
        "target/llvm-cov/lcov.info",
        "target/llvm-cov-target/lcov.info",
    ];

    for rel in CANDIDATES {
        let Ok(text) = std::fs::read_to_string(project_path.join(rel)) else {
            continue;
        };
        let mut lines_found: u64 = 0;
        let mut lines_hit: u64 = 0;
        for line in text.lines() {
            if let Some(v) = line.trim().strip_prefix("LF:") {
                lines_found += v.trim().parse::<u64>().unwrap_or(0);
            } else if let Some(v) = line.trim().strip_prefix("LH:") {
                lines_hit += v.trim().parse::<u64>().unwrap_or(0);
            }
        }
        if lines_found > 0 {
            return Some(lines_hit as f64 / lines_found as f64 * 100.0);
        }
    }

    None
}

/// Run coverage analysis - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_coverage_analysis(
    project_path: &Path,
    profile: &QualityProfile,
) -> Result<PhaseOutcome> {
    // This function used to read `let coverage = 65.0; // Simulated coverage`,
    // so every project on earth was reported 15 points short of the 80% floor —
    // including an empty directory with no tests and no source. A gate that
    // cannot measure a signal must say it did not measure it, not report a
    // failure it invented.
    let Some(coverage) = read_measured_line_coverage(project_path) else {
        let reason = warn_not_measured(
            "coverage",
            project_path,
            "no lcov report found (run `cargo llvm-cov --lcov --output-path lcov.info`)",
        );
        return Ok(PhaseOutcome::unmeasured(reason));
    };

    let mut violations = Vec::new();

    if coverage < profile.coverage_min {
        violations.push(QualityViolation {
            violation_type: "coverage".to_string(),
            severity: "high".to_string(),
            location: "project".to_string(),
            current: coverage,
            target: profile.coverage_min,
            suggestion: format!(
                "Increase test coverage by {:.1}%",
                profile.coverage_min - coverage
            ),
        });
    }

    Ok(PhaseOutcome::measured(violations))
}

#[cfg(test)]
mod measured_violation_tests {
    //! Regression tests for the fabricated `--list-violations` output: every
    //! phase used to push a literal violation (paths that exist in no project,
    //! `coverage 65.0`) regardless of what it had just analysed.
    use super::{
        read_measured_line_coverage, run_complexity_analysis, run_coverage_analysis,
        run_duplication_analysis, run_tdg_analysis, QualityProfile,
    };

    #[tokio::test]
    async fn test_empty_directory_yields_no_violations() {
        let temp = tempfile::TempDir::new().unwrap();
        let profile = QualityProfile::default();
        let root = temp.path();

        assert!(
            run_complexity_analysis(root, &profile, None)
                .await
                .unwrap()
                .violations
                .is_empty(),
            "an empty directory has no functions, so it can have no complexity violations"
        );
        assert!(
            run_tdg_analysis(root, &profile)
                .await
                .unwrap()
                .violations
                .is_empty(),
            "an empty directory has no files, so it can have no TDG hotspots"
        );
        assert!(
            run_duplication_analysis(root, &profile)
                .await
                .unwrap()
                .violations
                .is_empty(),
            "an empty directory has no duplicated lines"
        );

        // Coverage is the case the old assertion blurred. There is no lcov
        // report, so coverage was NOT measured — which is a different fact from
        // "measured, and clean", and the one that must not be credited.
        let coverage = run_coverage_analysis(root, &profile).await.unwrap();
        assert!(coverage.violations.is_empty());
        assert!(
            !coverage.is_measured(),
            "absent coverage data must report as unmeasured, not as a clean phase"
        );
    }

    #[tokio::test]
    async fn test_complexity_violation_points_at_the_analysed_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let src = temp.path().join("hot.rs");
        let mut body = String::from("pub fn tangled(n: i32) -> i32 {\n    let mut acc = 0;\n");
        for i in 0..30 {
            body.push_str(&format!("    if n == {i} {{ acc += {i}; }}\n"));
        }
        body.push_str("    acc\n}\n");
        std::fs::write(&src, body).unwrap();

        let profile = QualityProfile::default();
        let outcome = run_complexity_analysis(temp.path(), &profile, Some(&src))
            .await
            .unwrap();
        assert!(outcome.is_measured(), "the file parses, so this phase ran");
        let violations = outcome.violations;

        assert!(
            !violations.is_empty(),
            "a 31-branch function must exceed the default max of {}",
            profile.complexity_max
        );
        let v = &violations[0];
        assert_eq!(v.violation_type, "complexity");
        assert!(
            v.location.contains("hot.rs"),
            "violation must name the analysed file, got {:?}",
            v.location
        );
        assert!(
            !v.location
                .contains("server/src/cli/handlers/enforce_handlers.rs"),
            "the hardcoded sample location must be gone"
        );
        assert!(
            v.current > f64::from(profile.complexity_max),
            "reported complexity {} must be the measured value",
            v.current
        );
    }

    #[test]
    fn test_coverage_is_read_from_an_lcov_report() {
        let temp = tempfile::TempDir::new().unwrap();
        assert_eq!(read_measured_line_coverage(temp.path()), None);

        std::fs::write(
            temp.path().join("lcov.info"),
            "SF:src/a.rs\nLF:100\nLH:90\nend_of_record\nSF:src/b.rs\nLF:100\nLH:80\nend_of_record\n",
        )
        .unwrap();

        let measured = read_measured_line_coverage(temp.path()).unwrap();
        assert!(
            (measured - 85.0).abs() < 1e-9,
            "expected the lcov totals (170/200), got {measured}"
        );
    }

    #[tokio::test]
    async fn test_coverage_violation_uses_the_measured_value() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("lcov.info"),
            "SF:src/a.rs\nLF:100\nLH:42\nend_of_record\n",
        )
        .unwrap();

        let profile = QualityProfile::default();
        let outcome = run_coverage_analysis(temp.path(), &profile).await.unwrap();
        assert!(
            outcome.is_measured(),
            "an lcov report exists, so this phase measured"
        );
        let violations = outcome.violations;

        assert_eq!(violations.len(), 1);
        assert!(
            (violations[0].current - 42.0).abs() < 1e-9,
            "coverage must be the measured 42.0, not the old simulated 65.0; got {}",
            violations[0].current
        );
    }
}

#[cfg(test)]
mod scope_tests {
    use super::AnalysisScope;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_resolve_without_file_is_project_scope() {
        let scope = AnalysisScope::resolve(Path::new("/proj"), None);
        assert_eq!(
            scope,
            AnalysisScope::Project {
                root: PathBuf::from("/proj")
            }
        );
        assert_eq!(scope.single_file(), None);
        assert_eq!(scope.walk_root(), Path::new("/proj"));
        assert_eq!(scope.file_or_root(), Path::new("/proj"));
    }

    #[test]
    fn test_resolve_relative_file_joins_project_root() {
        let scope =
            AnalysisScope::resolve(Path::new("/proj"), Some(Path::new("src/utils/scratch.rs")));
        assert_eq!(
            scope.single_file(),
            Some(Path::new("/proj/src/utils/scratch.rs"))
        );
        // Directory-walk phases are scoped to the parent module, not the project root
        assert_eq!(scope.walk_root(), Path::new("/proj/src/utils"));
        assert_eq!(
            scope.file_or_root(),
            Path::new("/proj/src/utils/scratch.rs")
        );
    }

    #[test]
    fn test_resolve_absolute_file_kept_as_is() {
        let scope = AnalysisScope::resolve(Path::new("/proj"), Some(Path::new("/other/lib.rs")));
        assert_eq!(scope.single_file(), Some(Path::new("/other/lib.rs")));
        assert_eq!(scope.walk_root(), Path::new("/other"));
    }

    #[test]
    fn test_resolve_bare_filename_uses_project_root_as_module_dir() {
        let scope = AnalysisScope::resolve(Path::new("/proj"), Some(Path::new("main.rs")));
        assert_eq!(scope.single_file(), Some(Path::new("/proj/main.rs")));
        assert_eq!(scope.walk_root(), Path::new("/proj"));
    }

    #[test]
    fn test_resolve_empty_parent_falls_back_to_project_root() {
        // Empty project root + bare filename → joined path has an empty parent
        let scope = AnalysisScope::resolve(Path::new(""), Some(Path::new("scratch.rs")));
        assert_eq!(scope.walk_root(), Path::new(""));
        assert_eq!(scope.single_file(), Some(Path::new("scratch.rs")));
    }
}
