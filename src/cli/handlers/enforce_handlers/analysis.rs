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

/// Refuse a `--file` argument that cannot be read as a file.
///
/// `handle_analyzing_state` refuses a nonexistent `--project-path` ("enforce
/// cannot report a verdict on a path it cannot read"), but the sibling entry
/// point `--file` had no such guard: `enforce extreme --file nope/zzz.rs` span
/// 100 iterations and printed `State: Violating / Score: 0.00 / Violations: 3`,
/// exit 0 — and exit **1** under `--ci-mode`, so CI failed and blamed code
/// quality for a path that does not exist. A directory was accepted too
/// (`--file tiny` scored 0.33).
///
/// The guard lives here because every entry point that can be given a `--file`
/// — the analysing state, `--validate-only` and `--list-violations` — runs the
/// complexity phase first, and it must be an **error**, not a quality-gate
/// failure: an unreadable path is bad input, not bad code.
fn ensure_file_target_readable(file: &Path) -> Result<()> {
    if !file.exists() {
        anyhow::bail!(
            "path not found: {} — enforce cannot report a verdict on a path it cannot read",
            file.display()
        );
    }
    if !file.is_file() {
        anyhow::bail!(
            "--file expects a regular file, but {} is not one — enforce cannot report a file verdict on it",
            file.display()
        );
    }
    Ok(())
}

/// Files under analysis whose source does not parse, as `path: reason`.
///
/// A file whose AST could not be built is not evidence of quality. The
/// complexity path falls back to a regex heuristic and prints "Warning: AST
/// analysis failed for …, using heuristic fallback" on stderr — and stderr is
/// not a return value, so `enforce extreme -p broken` (whose only source file is
/// `fn main( { let x = ;;;`) reported `state COMPLETE, score 1.0, 0 violations`,
/// exit 0. An EMPTY directory was correctly disclosed as unmeasured, because the
/// "nothing analysable" detector counted candidate files rather than successful
/// analyses; a project where every file failed to parse graded perfect.
fn unparseable_files(paths: impl Iterator<Item = PathBuf>) -> Vec<String> {
    let mut failures: Vec<String> = paths
        .filter_map(|p| {
            crate::tdg::ensure_parseable(&p)
                .err()
                .map(|e| format!("{}: {e}", p.display()))
        })
        .collect();
    failures.sort();
    failures
}

/// Disclosure text for a partially- or wholly-unparseable analysis set.
fn unparseable_reason(failures: &[String]) -> String {
    const SHOWN: usize = 3;
    let listed: Vec<&str> = failures
        .iter()
        .take(SHOWN)
        .map(std::string::String::as_str)
        .collect();
    let more = failures.len().saturating_sub(listed.len());
    if more > 0 {
        format!(
            "{} file(s) did not parse, so no AST could be built for them: {} (+{more} more)",
            failures.len(),
            listed.join("; ")
        )
    } else {
        format!(
            "{} file(s) did not parse, so no AST could be built for them: {}",
            failures.len(),
            listed.join("; ")
        )
    }
}

/// Run complexity analysis - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_complexity_analysis(
    project_path: &Path,
    profile: &QualityProfile,
    specific_file: Option<&Path>,
) -> Result<PhaseOutcome> {
    if let Some(file) = specific_file {
        ensure_file_target_readable(file)?;
    }

    // The complexity service is called directly rather than through the
    // printing CLI handler: the handler returns `()`, so its result had to be
    // thrown away and a sample violation invented in its place.
    let file_metrics = match complexity_metrics(project_path, profile, specific_file).await {
        Ok(m) => m,
        Err(unmeasured) => return Ok(unmeasured),
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

    // Files exist, but did their ASTs? The heuristic fallback produces metrics
    // for input the parser rejected, and those metrics are indistinguishable
    // from a real measurement once they reach the score. Disclose the gap the
    // same way an empty tree is disclosed — the phase keeps whatever it did
    // measure, and the verdict stops claiming to cover what it could not read.
    let failures = unparseable_files(file_metrics.iter().map(|m| PathBuf::from(&m.path)));
    if failures.len() == file_metrics.len() {
        let reason = warn_not_measured(
            "complexity",
            project_path,
            &format!("no source file parsed; {}", unparseable_reason(&failures)),
        );
        return Ok(PhaseOutcome::unmeasured(reason));
    }

    let mut violations = complexity_violations(&file_metrics, profile);
    violations.sort_by(|a, b| {
        b.current
            .total_cmp(&a.current)
            .then_with(|| a.location.cmp(&b.location))
    });

    // This is the phase that enumerates the analysable source set — the same
    // set its "no analysable source files" disclosure above is about — so it is
    // the one that can say how many files the run read. `progress.files_completed`
    // is computed from this instead of being the literal `0` it used to be.
    let files_examined = file_metrics.len();

    if failures.is_empty() {
        return Ok(PhaseOutcome::measured(violations).over_files(files_examined));
    }

    // Partly measured: keep the findings from the files that DID parse, and
    // still deny the run a clean bill of health for the ones that did not. Only
    // the files that parsed were measured, so only those are counted as read.
    let reason = warn_not_measured("complexity", project_path, &unparseable_reason(&failures));
    Ok(PhaseOutcome {
        violations,
        unmeasured: Some(reason),
        files_examined: files_examined.saturating_sub(failures.len()),
    })
}

/// SATD for exactly one file, for `--file` scope.
///
/// Shares `satd_violation` with the project path so the two cannot drift in what
/// they report; only the set of files differs.
/// The complexity metrics for one file or the whole project, or the
/// `PhaseOutcome` that says why they could not be measured.
async fn complexity_metrics(
    project_path: &Path,
    profile: &QualityProfile,
    specific_file: Option<&Path>,
) -> std::result::Result<Vec<crate::services::complexity::FileComplexityMetrics>, PhaseOutcome> {
    match specific_file {
        Some(file) => crate::services::complexity::analyze_file_complexity_uncached(file, None)
            .await
            .map(|m| vec![m])
            .map_err(|e| {
                PhaseOutcome::unmeasured(warn_not_measured("complexity", file, &e.to_string()))
            }),
        None => crate::cli::analysis_utilities::analyze_project_files(
            project_path,
            None,
            &[],
            profile.complexity_max,
            profile.complexity_max,
        )
        .await
        .map_err(|e| {
            PhaseOutcome::unmeasured(warn_not_measured(
                "complexity",
                project_path,
                &e.to_string(),
            ))
        }),
    }
}

/// One violation per function over the profile's cyclomatic ceiling; twice
/// the ceiling is `high`.
fn complexity_violations(
    file_metrics: &[crate::services::complexity::FileComplexityMetrics],
    profile: &QualityProfile,
) -> Vec<QualityViolation> {
    let severe = profile.complexity_max.saturating_mul(2);
    file_metrics
        .iter()
        .flat_map(|file| file.functions.iter().map(move |func| (file, func)))
        .filter(|(_, func)| func.metrics.cyclomatic > profile.complexity_max)
        .map(|(file, func)| QualityViolation {
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
        })
        .collect()
}

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

/// The 0-100 TDG floor a profile demands, derived from its `tdg_max` knob.
///
/// `QualityProfile::tdg_max` is written on the legacy 0.0-5.0 debt-gradient
/// (lower is better) that `TDGCalculator` produced. `pmat tdg` reports on a
/// 0-100 higher-is-better scale with critical-defect gating, and the two never
/// met: a file `pmat tdg` graded **9.06/F with 5 critical defects** produced no
/// tdg violation at all under `--profile extreme`, so `enforce extreme
/// --ci-mode` returned `COMPLETE, score 1.0, 0 violations`, exit 0 — a clean
/// bill of health for code the same binary grades F.
///
/// The gradient analyser is gone; this maps the existing profile knob onto the
/// scale the rest of the tool speaks, keeping the three profiles ordered:
/// extreme (1.0) → 90 (A-), strict (1.5) → 85, standard (2.5) → 75.
#[must_use]
fn tdg_score_floor(profile: &QualityProfile) -> f64 {
    (100.0 - profile.tdg_max * 10.0).clamp(0.0, 100.0)
}

/// One tdg violation for a file scored by the analyser behind `pmat tdg`.
///
/// `current`/`target` are the shortfall from a perfect 100 rather than the score
/// itself, because every other violation in this system is lower-is-better and
/// the composite score's `phase_score` reads them that way — a higher-is-better
/// `current` would have scored a 9/100 file as a clean 1.0 phase, i.e. reported
/// `Score: 1.00/1.00` alongside `State: Violating`. The score `pmat tdg` prints,
/// its grade, and any critical-defect count are named verbatim in `suggestion`.
fn tdg_violation(
    score: &crate::tdg::TdgScore,
    floor: f64,
    fallback_path: &Path,
) -> QualityViolation {
    let total = f64::from(score.total);
    let location = score.file_path.as_ref().map_or_else(
        || fallback_path.display().to_string(),
        |p| p.display().to_string(),
    );
    let defects = if score.critical_defects_count > 0 && score.critical_defects_suppressed.is_none()
    {
        format!(", {} critical defect(s)", score.critical_defects_count)
    } else {
        String::new()
    };
    QualityViolation {
        violation_type: "tdg".to_string(),
        severity: if score.grade == crate::tdg::Grade::F || !defects.is_empty() {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        location,
        current: 100.0 - total,
        target: 100.0 - floor,
        suggestion: format!(
            "TDG {total:.1}/100 (grade {:?}{defects}) is below the {floor:.0}/100 floor this profile requires — the same score `pmat tdg` reports for this file",
            score.grade
        ),
    }
}

/// Does this score fail the profile?
///
/// Below the floor, or carrying critical defects whose auto-fail was not
/// suppressed by the #279 no-git-history exemption.
fn tdg_score_fails(score: &crate::tdg::TdgScore, floor: f64) -> bool {
    if f64::from(score.total) < floor {
        return true;
    }
    score.has_critical_defects && score.critical_defects_suppressed.is_none()
}

/// Run TDG analysis - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_tdg_analysis(
    project_path: &Path,
    profile: &QualityProfile,
) -> Result<PhaseOutcome> {
    // The analyser is the one `pmat tdg` uses, not the legacy
    // `services::tdg_calculator::TDGCalculator`. Two TDG implementations on two
    // incompatible scales is what let `enforce` and `tdg` contradict each other
    // on the same file; the duplicate is deleted rather than taught the missing
    // case.
    let analyzer = match crate::tdg::TdgAnalyzer::new() {
        Ok(a) => a,
        Err(e) => {
            let reason = warn_not_measured("tdg", project_path, &e.to_string());
            return Ok(PhaseOutcome::unmeasured(reason));
        }
    };

    let scores = if project_path.is_dir() {
        match analyzer.analyze_project(project_path).await {
            Ok(project) => project.files,
            Err(e) => {
                let reason = warn_not_measured("tdg", project_path, &e.to_string());
                return Ok(PhaseOutcome::unmeasured(reason));
            }
        }
    } else {
        match analyzer.analyze_file(project_path).await {
            Ok(score) => vec![score],
            Err(e) => {
                let reason = warn_not_measured("tdg", project_path, &e.to_string());
                return Ok(PhaseOutcome::unmeasured(reason));
            }
        }
    };

    // No file was graded — the absence of a measurement, not a clean one.
    if scores.is_empty() {
        return Ok(PhaseOutcome::unmeasured(format!(
            "no file could be graded under {}",
            project_path.display()
        )));
    }

    let floor = tdg_score_floor(profile);
    let mut violations: Vec<QualityViolation> = scores
        .iter()
        .filter(|s| tdg_score_fails(s, floor))
        .map(|s| tdg_violation(s, floor, project_path))
        .collect();

    // Deterministic order: worst first, then by location.
    violations.sort_by(|a, b| {
        b.current
            .total_cmp(&a.current)
            .then_with(|| a.location.cmp(&b.location))
    });

    Ok(PhaseOutcome::measured(violations))
}

/// Wall-clock budget for the dead-code phase of an `enforce` run.
///
/// Was 60. Dead-code analysis shells out to `cargo check`, and that budget is
/// wall clock — it covers waiting for the build, not just the analysis — so on
/// a cold target directory, a large workspace, or a loaded machine it expired
/// on work that was progressing normally, and the phase reported
/// "dead code could not be measured" for a project that is perfectly
/// measurable. A false "not measured" is the worse failure here: it is exactly
/// the absence-rendered-as-a-result that the surrounding code goes to lengths
/// to avoid, and 60 seconds is not enough of a wall to be worth hitting.
///
/// It fires on a genuine hang, which is what a budget is for. This also stops
/// four tests flaking in CI, where a contended runner starved the blocking task
/// long enough to trip the old value — but the user-facing false negative is
/// the reason to change it.
const DEAD_CODE_BUDGET_SECS: u64 = 300;

/// Environment override for the budget above.
///
/// Not a test crutch — no test uses it. When this phase times out it tells the
/// user to "re-run with a larger --timeout", and `pmat enforce` has no such
/// flag, so the advice was unfollowable. This is the knob that advice refers to.
///
/// It matters because the budget is WALL CLOCK around a `cargo check`: it
/// measures the machine as much as the code. On a loaded or slow host a
/// perfectly ordinary project can exceed it and be reported as unmeasured, and
/// until now the only remedy was to wait and hope.
///
pub const DEAD_CODE_BUDGET_ENV: &str = "PMAT_DEAD_CODE_TIMEOUT_SECS";

/// The budget to use: the override when set and parseable, else the default.
fn dead_code_budget_secs() -> u64 {
    std::env::var(DEAD_CODE_BUDGET_ENV)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEAD_CODE_BUDGET_SECS)
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
        Some(10),                // top_files
        true,                    // include_unreachable
        5,                       // min_dead_lines
        false,                   // include_tests
        Some(capture.clone()),   // output
        false,                   // fail_on_violation
        15.0,                    // max_percentage
        dead_code_budget_secs(), // timeout
        Vec::new(),              // include
        Vec::new(),              // exclude
        8,                       // max_depth
        false,                   // no_cache
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

#[cfg(test)]
mod round4_measurement_contract_tests {
    //! Round-4 regressions for three ways `enforce` reported a verdict it had
    //! not measured.
    use super::{
        ensure_file_target_readable, run_complexity_analysis, run_tdg_analysis, tdg_score_floor,
        unparseable_files, QualityProfile,
    };
    use std::path::PathBuf;

    fn write_crate(dir: &std::path::Path, files: &[(&str, &str)]) {
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("write Cargo.toml");
        std::fs::create_dir_all(dir.join("src")).expect("create src");
        for (name, body) in files {
            std::fs::write(dir.join("src").join(name), body).expect("write source");
        }
    }

    // ── R03: --file must be refused like --project-path is ──────────────────

    #[test]
    fn a_missing_file_target_is_an_error_not_a_verdict() {
        let err = ensure_file_target_readable(&PathBuf::from("nope/zzz.rs"))
            .expect_err("a path that does not exist cannot be enforced against");
        let msg = err.to_string();
        assert!(
            msg.contains("path not found") && msg.contains("cannot read"),
            "must reuse the --project-path refusal wording, got {msg}"
        );
    }

    #[test]
    fn a_directory_is_not_a_file_target() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = ensure_file_target_readable(dir.path())
            .expect_err("--file names a file, not a directory");
        assert!(err.to_string().contains("regular file"), "got {}", err);
    }

    #[tokio::test]
    async fn complexity_phase_refuses_a_nonexistent_file_target() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("zzz.rs");
        let err = run_complexity_analysis(dir.path(), &QualityProfile::default(), Some(&missing))
            .await
            .expect_err("an unreadable --file is bad input, not a quality failure");
        assert!(err.to_string().contains("path not found"), "got {err}");
    }

    // ── R05: a file whose AST could not be built is not a measurement ───────

    #[test]
    fn unparseable_files_names_only_the_files_that_failed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let good = dir.path().join("good.rs");
        let bad = dir.path().join("bad.rs");
        std::fs::write(&good, "pub fn a() -> i32 { 1 }\n").expect("write");
        std::fs::write(&bad, "fn main( { let x = ;;;\n").expect("write");

        let failures = unparseable_files(vec![good, bad].into_iter());
        assert_eq!(failures.len(), 1, "only bad.rs fails: {failures:?}");
        assert!(failures[0].contains("bad.rs"), "got {failures:?}");
    }

    #[tokio::test]
    async fn a_project_whose_every_source_fails_to_parse_is_unmeasured() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_crate(dir.path(), &[("main.rs", "fn main( { let x = ;;;\n")]);

        let outcome = run_complexity_analysis(dir.path(), &QualityProfile::default(), None)
            .await
            .expect("phase runs");
        assert!(
            !outcome.is_measured(),
            "the heuristic fallback produced metrics for input the parser rejected; \
             that must not read as a clean measurement"
        );
        let reason = outcome.unmeasured.unwrap_or_default();
        assert!(reason.contains("did not parse"), "got {reason}");
    }

    #[tokio::test]
    async fn a_project_with_one_unparseable_file_still_discloses_the_gap() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_crate(
            dir.path(),
            &[
                ("lib.rs", "pub fn ok(a: i32) -> i32 { a * 2 }\n"),
                ("bad.rs", "fn main( { let x = ;;;\n"),
            ],
        );

        let outcome = run_complexity_analysis(dir.path(), &QualityProfile::default(), None)
            .await
            .expect("phase runs");
        assert!(
            !outcome.is_measured(),
            "one valid file plus one garbage file must not report as fully measured"
        );
        assert!(
            outcome
                .unmeasured
                .as_deref()
                .is_some_and(|r| r.contains("bad.rs")),
            "the disclosure must name the file that failed: {:?}",
            outcome.unmeasured
        );
    }

    // ── R01: enforce's TDG must be the TDG `pmat tdg` reports ───────────────

    #[test]
    fn the_tdg_floor_tracks_the_profile_and_stays_ordered() {
        let extreme = QualityProfile::default();
        let strict = QualityProfile {
            tdg_max: 1.5,
            ..QualityProfile::default()
        };
        let standard = QualityProfile {
            tdg_max: 2.5,
            ..QualityProfile::default()
        };
        assert!((tdg_score_floor(&extreme) - 90.0).abs() < 1e-9);
        assert!(tdg_score_floor(&extreme) > tdg_score_floor(&strict));
        assert!(tdg_score_floor(&strict) > tdg_score_floor(&standard));
        // A profile that gives TDG unreachable headroom must clamp, not go
        // negative and start failing everything.
        let lax = QualityProfile {
            tdg_max: 1000.0,
            ..QualityProfile::default()
        };
        assert!((tdg_score_floor(&lax) - 0.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn critical_defects_produce_a_tdg_violation_under_extreme() {
        // Five `.unwrap()` calls: `pmat tdg` grades this file 9.06/F with five
        // critical defects, while `enforce extreme --ci-mode` reported
        // COMPLETE 1.0 / 0 violations / exit 0, because its tdg phase ran a
        // different analyser on a different scale.
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("lib.rs");
        let mut body = String::new();
        for name in ["a", "b", "c", "d", "e"] {
            body.push_str(&format!(
                "pub fn {name}(s: &str) -> i32 {{\n    s.parse::<i32>().unwrap()\n}}\n"
            ));
        }
        std::fs::write(&file, body).expect("write");

        let outcome = run_tdg_analysis(&file, &QualityProfile::default())
            .await
            .expect("phase runs");
        assert!(outcome.is_measured(), "the file parses, so tdg measured it");
        assert_eq!(
            outcome.violations.len(),
            1,
            "a file graded F must produce a tdg violation: {:?}",
            outcome.violations
        );
        let v = &outcome.violations[0];
        assert_eq!(v.violation_type, "tdg");
        assert_eq!(v.severity, "high");
        assert!(
            v.suggestion.contains("/100"),
            "the suggestion must quote the 0-100 score `pmat tdg` reports, got {}",
            v.suggestion
        );
        // The legacy 0.0-5.0 debt gradient is gone: `target` is the shortfall
        // from 100 that the extreme profile allows, i.e. 10.
        assert!(
            (v.target - 10.0).abs() < 1e-9,
            "expected the extreme profile's 10-point allowance, got {}",
            v.target
        );
        assert!(
            v.current > v.target,
            "a violating file must overshoot its allowance: {} vs {}",
            v.current,
            v.target
        );
    }

    #[tokio::test]
    async fn a_clean_file_produces_no_tdg_violation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("clean.rs");
        std::fs::write(
            &file,
            "//! A tidy module.\n\n/// Adds two numbers.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        )
        .expect("write");

        let outcome = run_tdg_analysis(&file, &QualityProfile::default())
            .await
            .expect("phase runs");
        assert!(outcome.is_measured());
        assert!(
            outcome.violations.is_empty(),
            "a clean file must not be flagged: {:?}",
            outcome.violations
        );
    }
}
