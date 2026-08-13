#![cfg_attr(coverage_nightly, coverage(off))]
//! One assessment of a project, read by every surface.
//!
//! `enforce extreme` used to answer the same question in two different places:
//!
//! * the state machine (`handle_analyzing_state`), which `--ci-mode`,
//!   `--format json` and `--validate-only` read, ran three phases and turned
//!   every phase that could not be measured into a visible `not_measured`
//!   finding; and
//! * `list_all_violations`, which `--list-violations` read, ran six phases and
//!   kept only `PhaseOutcome::violations`, dropping `PhaseOutcome::unmeasured`
//!   on the floor.
//!
//! So one run of one binary over one directory reported three answers at once:
//!
//! ```text
//! $ pmat enforce extreme -p empty --list-violations   # Found 0 violations, exit 0
//! $ pmat enforce extreme -p empty --ci-mode           # Violations: 3, exit 1
//! $ pmat enforce extreme -p empty --format json       # "state":"VIOLATING", 3 violations
//! ```
//!
//! and `--list-violations -p /nonexistent` printed `Found 0 violations`, exit 0,
//! for a path the state machine refuses to grade at all.
//!
//! This module is the single source. Every surface calls [`assess_project`] and
//! renders what it returns; none of them collects violations of its own. The
//! phase set lives here, once: a verdict that claims the extreme profile was met
//! has to cover every dimension the profile sets a threshold for, and a
//! dimension that could not be measured is disclosed rather than credited.

use super::analysis::{
    run_complexity_analysis, run_coverage_analysis, run_dead_code_analysis,
    run_duplication_analysis, run_satd_analysis, run_tdg_analysis, AnalysisScope,
};
use super::types::{EnforcementState, PhaseOutcome, QualityProfile, QualityViolation};
use crate::cli::colors as c;
use anyhow::Result;
use std::path::Path;

/// The verdict of one run: what was found, what could not be looked at, and how
/// close the measured part came to the profile.
#[derive(Debug, Clone)]
pub struct QualityAssessment {
    /// Disclosures first (`violation_type == "not_measured"`), then findings,
    /// in phase order. This is the ONLY violation list `enforce` produces.
    pub violations: Vec<QualityViolation>,
    /// Composite score over the phases that ran, scaled by how many of them
    /// there were.
    pub score: f64,
    /// Phases that produced a measurement.
    pub measured_phases: usize,
    /// Phases attempted.
    pub total_phases: usize,
    /// How many source files the run actually read.
    ///
    /// The largest count any phase reported: phases that do not enumerate files
    /// report `0`, and the complexity phase enumerates exactly the analysable
    /// source set. `progress.files_completed` is this minus the files that carry
    /// a finding; before it was measured, that field was the literal `0` for
    /// every project, empty directories and 121-file corpora alike.
    pub files_examined: usize,
}

impl QualityAssessment {
    /// Did any dimension go unmeasured?
    #[must_use]
    pub fn any_unmeasured(&self) -> bool {
        self.measured_phases < self.total_phases
    }

    /// The verdict every surface must agree on.
    ///
    /// `Complete` asserts that the profile was met, so it requires evidence for
    /// every dimension — not merely the absence of findings among the dimensions
    /// that happened to run. This is the one place that rule is written down:
    /// `--ci-mode`'s exit code, `--validate-only`'s pass/fail and
    /// `--list-violations`' exit code are all this function.
    #[must_use]
    pub fn verdict_state(&self) -> EnforcementState {
        if self.violations.is_empty() && !self.any_unmeasured() {
            EnforcementState::Complete
        } else {
            EnforcementState::Violating
        }
    }
}

/// Score one analysis phase in `[0.0, 1.0]`: 1.0 when the phase found nothing
/// to report, otherwise how close its worst violation sits to the limit the
/// profile allows (a function at twice the allowed complexity scores 0.5).
pub(super) fn phase_score(violations: &[QualityViolation]) -> f64 {
    violations
        .iter()
        .map(violation_score)
        .fold(1.0_f64, f64::min)
}

/// Is this dimension's threshold a floor (higher is better) rather than a
/// ceiling?
///
/// Five of the six phases measure something you want less of — complexity, SATD
/// markers, TDG, dead lines, duplicated lines — and breach their threshold from
/// above. Coverage is the one that breaches from below. Nothing else may be
/// added to this list without a threshold that works the same way.
fn is_floor_dimension(violation_type: &str) -> bool {
    violation_type == "coverage"
}

/// How close one violation sits to the limit it breached, in `[0.0, 1.0]`.
///
/// This used to open with `if v.current <= v.target { 1.0 }` — a second, weaker
/// copy of the question "is this a violation?", living downstream of the
/// analyzer that had already answered it. For the five ceiling dimensions the
/// two copies agreed; for coverage, whose threshold is a floor, the copy
/// contradicted the analyzer and scored a real breach a full 1.0. A crate with
/// 0% coverage against the extreme profile's 80% printed
/// `State: Violating / Score: 1.00/1.00 / Violations: 1`.
///
/// A violation is a breach by construction — the phase that emitted it decided
/// that. This function only asks how bad it is, which is the ratio of the
/// measurement to the limit, oriented by which side of the limit the dimension
/// is breached from.
fn violation_score(v: &QualityViolation) -> f64 {
    let ratio = if is_floor_dimension(&v.violation_type) {
        // 40% coverage against an 80% floor scores 0.5.
        if v.target > 0.0 {
            v.current / v.target
        } else {
            // A floor of zero cannot be breached; if a phase claims otherwise,
            // its own judgement stands and the breach is total.
            0.0
        }
    } else if v.current > 0.0 {
        // Complexity 40 against a ceiling of 10 scores 0.25.
        v.target / v.current
    } else {
        // A zero-tolerance dimension (e.g. satd_allowed = 0) that was
        // nevertheless violated.
        0.0
    };
    ratio.clamp(0.0, 1.0)
}

/// Announce a phase on stderr and run it.
///
/// The progress lines belong to the run, not to one surface: `--list-violations`
/// printed them and the state machine did not, which is how the two came to look
/// like different operations in the first place.
macro_rules! phase {
    ($label:expr, $call:expr) => {{
        $crate::status_eprintln!("  {} {}...", c::dim(">>"), $label);
        $call.await?
    }};
}

/// Assess a project against a profile — the one measurement `enforce` makes.
///
/// # Errors
///
/// Returns an error when the target path cannot be read. An unreadable path is
/// bad input, not bad code, and must not be answered with a verdict: both
/// `--list-violations` and the state machine now refuse it identically, where
/// only the state machine used to.
pub async fn assess_project(
    project_path: &Path,
    profile: &QualityProfile,
    specific_file: Option<&Path>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<QualityAssessment> {
    // `enforce extreme -p /nope` reported `Complete 1.00/1.00`, exit 0 — the
    // analyzers return `Ok` for input they never read, so every phase came back
    // clean. Refuse up front, as the other path-taking commands in this release
    // do.
    if !project_path.exists() {
        anyhow::bail!(
            "path not found: {} — enforce cannot report a verdict on a path it cannot read",
            project_path.display()
        );
    }

    let scope = AnalysisScope::resolve(project_path, specific_file);
    if let AnalysisScope::SingleFile { module_dir, .. } = &scope {
        // Directory-walk phases cannot target a lone file.
        eprintln!(
            "  {} Single-file mode: SATD/dead-code/duplication scoped to parent module {}",
            c::dim(">>"),
            module_dir.display()
        );
    }

    let outcomes: Vec<(&str, PhaseOutcome)> = vec![
        (
            "complexity",
            phase!(
                "Analyzing complexity",
                run_complexity_analysis(scope.walk_root(), profile, scope.single_file())
            ),
        ),
        (
            "satd",
            phase!(
                "Analyzing technical debt (SATD)",
                run_satd_analysis(scope.walk_root(), profile, scope.single_file())
            ),
        ),
        (
            "tdg",
            phase!(
                "Analyzing technical debt gradient",
                run_tdg_analysis(scope.file_or_root(), profile)
            ),
        ),
        (
            "dead code",
            phase!(
                "Analyzing dead code",
                run_dead_code_analysis(scope.walk_root(), profile)
            ),
        ),
        (
            "duplication",
            phase!(
                "Analyzing code duplication",
                run_duplication_analysis(scope.walk_root(), profile)
            ),
        ),
        (
            "coverage",
            phase!(
                "Checking test coverage",
                run_coverage_analysis(scope.walk_root(), profile)
            ),
        ),
    ];

    summarize(
        outcomes,
        project_path,
        specific_file,
        include_pattern,
        exclude_pattern,
    )
}

/// Turn the phase outcomes into the single violation list and the score.
///
/// Split out so the aggregation can be tested without running six analyzers,
/// and so there is exactly one implementation of "an unmeasured phase becomes a
/// visible finding".
pub(super) fn summarize(
    mut outcomes: Vec<(&str, PhaseOutcome)>,
    project_path: &Path,
    specific_file: Option<&Path>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<QualityAssessment> {
    // `--include` / `--exclude` are applied BEFORE the score is computed, so the
    // score and the violation list describe the same set of files: filtering the
    // printed list alone would leave the score measuring files the report denies
    // looking at.
    if include_pattern.is_some() || exclude_pattern.is_some() {
        let filter = crate::utils::file_filter::FileFilter::from_optional(
            &include_pattern.cloned(),
            &exclude_pattern.cloned(),
        )?;
        for (_, outcome) in &mut outcomes {
            outcome
                .violations
                .retain(|v| violation_is_included(&filter, project_path, v));
        }
    }

    // An unmeasured phase is excluded from the mean rather than scored 0: the
    // score then honestly describes what WAS measured, while the fraction that
    // could be measured keeps a partial assessment from presenting as a complete
    // one. A fully measured clean project still scores exactly 1.0.
    let measured: Vec<f64> = outcomes
        .iter()
        .filter(|(_, o)| o.is_measured())
        .map(|(_, o)| phase_score(&o.violations))
        .collect();
    let total_phases = outcomes.len();
    let measured_phases = measured.len();
    let files_examined = outcomes
        .iter()
        .map(|(_, o)| o.files_examined)
        .max()
        .unwrap_or(0);
    let score = if measured.is_empty() {
        0.0
    } else {
        let mean = measured.iter().sum::<f64>() / measured.len() as f64;
        mean * (measured_phases as f64 / total_phases as f64)
    };

    // Each gap becomes a visible finding. It is not a quality violation, so it
    // is typed apart from one — but it does deny the run a clean bill of health,
    // which is the whole point: a check that could not run has not passed. These
    // are the disclosures `--list-violations` used to discard.
    let location = specific_file.map_or_else(
        || project_path.display().to_string(),
        |p| p.display().to_string(),
    );
    let mut violations: Vec<QualityViolation> = outcomes
        .iter()
        .filter_map(|(kind, o)| {
            o.unmeasured.as_ref().map(|reason| QualityViolation {
                violation_type: "not_measured".to_string(),
                severity: "error".to_string(),
                location: location.clone(),
                current: 0.0,
                target: 0.0,
                suggestion: format!(
                    "{kind} could not be measured ({reason}); this verdict does not cover it"
                ),
            })
        })
        .collect();

    for (_, outcome) in outcomes {
        violations.extend(outcome.violations);
    }

    Ok(QualityAssessment {
        violations,
        score,
        measured_phases,
        total_phases,
        files_examined,
    })
}

/// Does `--include`/`--exclude` admit the file this violation names?
///
/// Locations are `path:line:name` or a bare path, and the path is usually
/// absolute while the patterns a user writes (`src/*`) are relative to the
/// project, so the path is matched in project-relative form. Matching both
/// forms and OR-ing them would defeat `--exclude`, whose answer must be "no"
/// if EITHER form matches.
pub(super) fn violation_is_included(
    filter: &crate::utils::file_filter::FileFilter,
    project_path: &Path,
    violation: &QualityViolation,
) -> bool {
    let raw = violation
        .location
        .split(':')
        .next()
        .unwrap_or(&violation.location);
    let path = Path::new(raw);
    filter.should_include(path.strip_prefix(project_path).unwrap_or(path))
}
