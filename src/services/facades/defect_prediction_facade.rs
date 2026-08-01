#![allow(unused)]
//! Defect Prediction Analysis Facade
//!
//! Provides a simplified interface for defect prediction and risk analysis.
//!
//! # Measured-or-absent (#657)
//!
//! Two figures used to be fabricated here:
//!
//! * `churn_score` was the literal `0.3` for every file, with the comment
//!   "Would be calculated from git history". A constant that renders as a
//!   measurement is exactly what `pmat-no-fabrication-v1` forbids, and it also
//!   made the output independent of the input. It is now measured from git,
//!   and is `None` (JSON `null`) — never a plausible default, never `0.0` —
//!   when no repository can be found.
//! * `total_files_analyzed` was `predictions.len()` **after** truncation to
//!   `top_files`, so 3 files reported 3, 20 files reported 10, and the whole
//!   3863-file repository also reported 10. A cap presented as a total is a
//!   measured-or-absent violation; the cap is now named
//!   (`predictions_reported` / `predictions_truncated`) and the totals are
//!   real counts.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::services::git_analysis::GitAnalysisService;
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Probability at which a file is classified `RiskLevel::High`.
///
/// GH #685 (round 3): `--high-risk-only` help said "(probability > 0.7)" while
/// this band actually begins at 0.6, so the flag returned files at 0.6069833
/// and 0.657 -- both below the documented cut. The help text now quotes this
/// constant, and a test asserts the two agree, so they cannot drift apart.
pub const HIGH_RISK_PROBABILITY: f32 = 0.6;

/// Git history window, in days, used for the churn measurement.
const CHURN_WINDOW_DAYS: u32 = 90;

/// Commits within the window that put a file at the top of the churn scale.
///
/// The score used to be `FileChurnMetrics::churn_score`, which is normalized
/// against the MAXIMUM in the analyzed set. That made it a property of the
/// command line rather than of the file: `cold.rs` (1 commit, confirmed by
/// `analyze churn`) scored 0.142 from the repo root and 1.0 — with
/// `contributing_factors: ["Frequent changes"]` and a 2.6x higher defect
/// probability — when analyzed from its own directory. Nothing about the file
/// changed. These fixed scales make the score reproducible and comparable, and
/// they are reported in `churn_source` so the reader can see the yardstick.
const COMMITS_AT_FULL_SCALE: f32 = 20.0;
/// Lines added+deleted within the window that put a file at the top of the scale.
const CHANGED_LINES_AT_FULL_SCALE: f32 = 1000.0;
/// Distinct import statements that put a file at the top of the coupling scale.
const IMPORTS_AT_FULL_SCALE: f32 = 20.0;
/// `churn_score` above this is described as "frequent changes" — with the
/// measured commit count named alongside, never on its own.
const FREQUENT_CHANGE_SCORE: f32 = 0.5;

/// Request for defect prediction analysis
#[derive(Debug, Clone)]
pub struct DefectPredictionRequest {
    pub project_path: PathBuf,
    pub confidence_threshold: f32,
    pub min_lines: usize,
    pub include_low_confidence: bool,
    pub high_risk_only: bool,
    pub include_recommendations: bool,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    /// Maximum predictions to report; `0` means "all" (the CLI documents
    /// `--top-files 0` as "0 = all").
    pub top_files: usize,
}

/// Result of defect prediction analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectPredictionResult {
    /// Eligible source files found under the requested path.
    pub total_files_discovered: usize,
    /// Files actually scored (discovered files minus unreadable ones and ones
    /// below `min_lines`). #657: this used to be the post-truncation
    /// prediction count, i.e. a cap of 10 presented as a total.
    pub total_files_analyzed: usize,
    /// Analyzed files that survived the `high_risk_only` / confidence filters.
    pub files_matching_filters: usize,
    pub high_risk_files: usize,
    pub medium_risk_files: usize,
    pub low_risk_files: usize,
    /// How many predictions are actually listed in `predictions`.
    pub predictions_reported: usize,
    /// True when `top_files` cut the list short.
    pub predictions_truncated: bool,
    /// Where `churn_score` came from, or why it is absent.
    pub churn_source: ChurnSource,
    /// Where `coupling_score` came from (#657: it had no provenance and was a
    /// constant).
    pub coupling_source: CouplingSource,
    /// Why `duplication_score` is absent (#657: it was the constant 0.1).
    pub duplication_source: DuplicationSource,
    pub predictions: Vec<FilePrediction>,
    pub summary: String,
    pub recommendations: Vec<String>,
}

/// Provenance of the churn metric (#657: never report a churn number without
/// saying where it came from).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChurnSource {
    /// Measured from `git log` over the last `window_days` days.
    GitHistory {
        window_days: u32,
        files_with_churn: usize,
        /// Commits within the window that score 1.0. Named so the score can be
        /// read back to a commit count; it is NOT relative to the other files
        /// in this run.
        commits_at_full_scale: u32,
        /// Changed lines within the window that score 1.0.
        changed_lines_at_full_scale: u32,
    },
    /// Not measured; `churn_score` is `null` and is excluded from the
    /// probability instead of being replaced by a default.
    NotMeasured { reason: String },
}

impl ChurnSource {
    /// Human-readable one-liner for text/summary renderers.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::GitHistory {
                window_days,
                files_with_churn,
                commits_at_full_scale,
                ..
            } => format!(
                "git history, last {window_days} days ({files_with_churn} files with commits; \
                 {commits_at_full_scale} commits = 1.0)"
            ),
            Self::NotMeasured { reason } => format!("not measured ({reason})"),
        }
    }
}

/// Provenance of the coupling metric.
///
/// #657 named `coupling_score` explicitly: it was the compile-time constant
/// `0.2` for every file (`jq '[.predictions[].metrics.coupling_score]|unique'`
/// returned `[0.2]` over 109 files) with the comment "would be calculated from
/// dependency analysis". It is now efferent coupling — the distinct import
/// statements the file itself declares — and the yardstick is named.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CouplingSource {
    /// Counted from the file's own import statements.
    EfferentImports { imports_at_full_scale: u32 },
    /// Not measured for this run.
    NotMeasured { reason: String },
}

impl CouplingSource {
    /// Human-readable one-liner for text/summary renderers.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::EfferentImports {
                imports_at_full_scale,
            } => format!(
                "distinct import statements per file ({imports_at_full_scale} imports = 1.0)"
            ),
            Self::NotMeasured { reason } => format!("not measured ({reason})"),
        }
    }
}

/// Provenance of the duplication metric.
///
/// #657: `duplication_score` was the compile-time constant `0.1` for every file
/// ("would be calculated from duplicate detector"). Cross-file clone detection
/// is not run by this command, so the honest report is an absent value with the
/// reason attached — never a plausible default, never 0.0.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DuplicationSource {
    NotMeasured { reason: String },
}

impl DuplicationSource {
    /// The single reason this command has: it does not run clone detection.
    #[must_use]
    pub fn not_run() -> Self {
        Self::NotMeasured {
            reason: "defect-prediction does not run clone detection; \
                     run `pmat analyze duplicates` for measured duplication"
                .to_string(),
        }
    }

    /// Human-readable one-liner for text/summary renderers.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::NotMeasured { reason } => format!("not measured ({reason})"),
        }
    }
}

/// Prediction for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePrediction {
    pub file_path: String,
    pub defect_probability: f32,
    pub risk_level: RiskLevel,
    pub confidence: f32,
    pub metrics: FileRiskMetrics,
    pub contributing_factors: Vec<String>,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// Risk metrics for a file, each normalised to 0.0-1.0.
///
/// `None` means pmat did not measure the factor for this file — not that it
/// measured zero. Four of the five were compile-time constants
/// (`churn_score = 0.3`, `coupling_score = 0.2`, `duplication_score = 0.1`,
/// `confidence = 0.75`), each tagged "would be calculated from …", so
/// `defect_probability` was a function of file length alone while five named
/// "ML-based" metrics were presented as measurements (GH #657). churn was
/// reported as 0.3 even for directories with no git repository at all.
///
/// See `contracts/pmat-no-fabrication-v1.yaml`, equation `measured_or_absent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRiskMetrics {
    /// Length-derived proxy: physical lines / 100, capped at 1.0. `lines` is
    /// reported beside it so the reader can see what it is derived from.
    pub complexity_score: f32,
    /// Measured 0-1 git churn, or `None` when churn could not be measured.
    ///
    /// #657: this was the constant `0.3` for every file. It must never be a
    /// default, and never `0.0` — zero reads as "this file never changed",
    /// which is a finding, not an absence.
    pub churn_score: Option<f32>,
    /// Measured 0-1 efferent coupling (distinct imports), or `None` when the
    /// file's language has no import form this counts. Was the constant `0.2`.
    pub coupling_score: Option<f32>,
    pub size_score: f32,
    /// Always `None`: this command does not run clone detection. Was the
    /// constant `0.1`.
    pub duplication_score: Option<f32>,
    /// Evidence for the scores above — measured, not derived.
    pub lines: usize,
    /// Distinct import statements found in the file.
    pub imports: Option<usize>,
    /// Commits touching this file inside the churn window.
    pub commits_in_window: Option<usize>,
    /// Lines added+deleted inside the churn window.
    pub changed_lines_in_window: Option<usize>,
}

/// What git actually recorded for one file inside the window.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChurnObservation {
    pub commits: usize,
    pub changed_lines: usize,
}

impl ChurnObservation {
    /// Absolute 0-1 score. Deliberately NOT normalized against the other files
    /// in the run — see `COMMITS_AT_FULL_SCALE`.
    #[must_use]
    pub fn score(self) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        let commit_factor = (self.commits as f32 / COMMITS_AT_FULL_SCALE).min(1.0);
        #[allow(clippy::cast_precision_loss)]
        let change_factor = (self.changed_lines as f32 / CHANGED_LINES_AT_FULL_SCALE).min(1.0);
        (commit_factor * 0.6 + change_factor * 0.4).clamp(0.0, 1.0)
    }
}

/// Churn measured from git, keyed by canonical absolute path.
struct ChurnIndex {
    by_path: HashMap<PathBuf, ChurnObservation>,
    /// Canonical paths git has under version control, when they could be
    /// listed. A file that is NOT tracked has no history to measure, which is
    /// different from a tracked file with no commits in the window.
    tracked: Option<std::collections::HashSet<PathBuf>>,
    source: ChurnSource,
}

impl ChurnIndex {
    /// Measure churn for `path`, walking up to the repository root if needed.
    fn measure(path: &Path) -> Self {
        match GitAnalysisService::analyze_code_churn(path, CHURN_WINDOW_DAYS) {
            Ok(analysis) => {
                let mut by_path = HashMap::with_capacity(analysis.files.len());
                for file in &analysis.files {
                    let key = file
                        .path
                        .canonicalize()
                        .unwrap_or_else(|_| file.path.clone());
                    by_path.insert(
                        key,
                        ChurnObservation {
                            commits: file.commit_count,
                            changed_lines: file.additions + file.deletions,
                        },
                    );
                }
                let files_with_churn = by_path.len();
                Self {
                    by_path,
                    tracked: tracked_files(path),
                    source: ChurnSource::GitHistory {
                        window_days: CHURN_WINDOW_DAYS,
                        files_with_churn,
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        commits_at_full_scale: COMMITS_AT_FULL_SCALE as u32,
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        changed_lines_at_full_scale: CHANGED_LINES_AT_FULL_SCALE as u32,
                    },
                }
            }
            Err(e) => {
                // Say what actually happens: the score is absent, it is NOT
                // silently replaced by a stand-in. (The old message claimed a
                // "falling back to file age" that never happened.)
                tracing::warn!(
                    "churn not measured for {}: {e}; churn_score will be null and is \
                     excluded from the defect probability",
                    path.display()
                );
                Self {
                    by_path: HashMap::new(),
                    tracked: None,
                    source: ChurnSource::NotMeasured {
                        reason: e.to_string(),
                    },
                }
            }
        }
    }

    /// Churn for one file. `None` when there is no git history to read for it
    /// — no repository, or the file is not under version control. A TRACKED
    /// file with no commits in the window is a real `0.0`.
    fn observation_for(&self, file: &Path) -> Option<ChurnObservation> {
        if matches!(self.source, ChurnSource::NotMeasured { .. }) {
            return None;
        }
        let key = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
        if let Some(observed) = self.by_path.get(&key) {
            return Some(*observed);
        }
        match &self.tracked {
            // Untracked: git holds no history for this file, so churn is
            // absent, not zero. This is also what makes `confidence` a
            // per-file number instead of one constant for the whole run.
            Some(tracked) if !tracked.contains(&key) => None,
            _ => Some(ChurnObservation::default()),
        }
    }
}

/// Canonical paths under version control, or `None` if git cannot list them.
fn tracked_files(path: &Path) -> Option<std::collections::HashSet<PathBuf>> {
    let root = GitAnalysisService::discover_repository_root(path)?;
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "-z"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let listing = String::from_utf8_lossy(&output.stdout);
    Some(
        listing
            .split('\0')
            .filter(|entry| !entry.is_empty())
            .map(|entry| {
                let joined = root.join(entry);
                joined.canonicalize().unwrap_or(joined)
            })
            .collect(),
    )
}

/// Distinct import statements declared by a file — efferent coupling (Ce).
///
/// `None` when the language is not one whose imports this recognises, so a
/// language we cannot read is reported as unmeasured rather than as zero
/// coupling.
fn count_imports(path: &Path, content: &str) -> Option<usize> {
    let prefixes: &[&str] = match path.extension().and_then(|e| e.to_str())? {
        "rs" => &["use ", "extern crate "],
        "py" => &["import ", "from "],
        "js" | "ts" => &["import ", "const {", "require(", "export * from"],
        "java" => &["import "],
        "c" | "cpp" => &["#include"],
        _ => return None,
    };
    let distinct: std::collections::BTreeSet<&str> = content
        .lines()
        .map(str::trim)
        .filter(|line| prefixes.iter().any(|p| line.starts_with(p)))
        .collect();
    Some(distinct.len())
}

/// Weight of each risk factor. They sum to 1.0; when a factor is unmeasured
/// the remaining weights are renormalized rather than a default being invented.
const W_COMPLEXITY: f32 = 0.30;
const W_CHURN: f32 = 0.25;
const W_COUPLING: f32 = 0.20;
const W_SIZE: f32 = 0.15;
const W_DUPLICATION: f32 = 0.10;

/// Weight of the factors that were actually measured for this file.
///
/// This is what `confidence` reports: it is the share of the model that had
/// data, per file. `duplication` is never measured by this command, so the
/// ceiling is 0.90; a file with no git history drops to 0.65.
fn measured_weight(metrics: &FileRiskMetrics) -> f32 {
    let mut weight = W_COMPLEXITY + W_SIZE;
    if metrics.churn_score.is_some() {
        weight += W_CHURN;
    }
    if metrics.coupling_score.is_some() {
        weight += W_COUPLING;
    }
    if metrics.duplication_score.is_some() {
        weight += W_DUPLICATION;
    }
    weight
}

/// Combine the measured factors into a 0-1 probability.
///
/// An absent factor drops its term *and* its weight, so the probability stays
/// derived only from what was actually measured — never from a stand-in.
fn combine_probability(metrics: &FileRiskMetrics) -> f32 {
    let mut weighted = metrics.complexity_score * W_COMPLEXITY + metrics.size_score * W_SIZE;

    if let Some(churn) = metrics.churn_score {
        weighted += churn * W_CHURN;
    }
    if let Some(coupling) = metrics.coupling_score {
        weighted += coupling * W_COUPLING;
    }
    if let Some(duplication) = metrics.duplication_score {
        weighted += duplication * W_DUPLICATION;
    }

    let total_weight = measured_weight(metrics);
    if total_weight <= 0.0 {
        return 0.0;
    }
    (weighted / total_weight).clamp(0.0, 1.0)
}

fn classify(probability: f32) -> RiskLevel {
    match probability {
        p if p >= 0.8 => RiskLevel::Critical,
        p if p >= HIGH_RISK_PROBABILITY => RiskLevel::High,
        p if p >= 0.4 => RiskLevel::Medium,
        _ => RiskLevel::Low,
    }
}

/// Facade for defect prediction analysis
#[derive(Clone)]
pub struct DefectPredictionFacade {
    registry: Arc<ServiceRegistry>,
}

impl DefectPredictionFacade {
    /// Create a new defect prediction facade
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform defect prediction analysis on a project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze_project(
        &self,
        request: DefectPredictionRequest,
    ) -> Result<DefectPredictionResult> {
        let files = self.discover_files(&request).await?;
        let churn = ChurnIndex::measure(&request.project_path);

        // #657: every discovered file is scored. There is no hidden
        // `take(top_files * 2)` cap any more — that cap was being reported as
        // the total.
        let mut analyzed = 0usize;
        let mut matching = Vec::new();
        for file_path in &files {
            let Ok(prediction) = self.analyze_file(file_path, &request, &churn).await else {
                continue;
            };
            analyzed += 1;
            if passes_filters(&prediction, &request) {
                matching.push(prediction);
            }
        }

        // Deterministic order: probability descending, path ascending as the
        // tie-break so equal-probability files can never swap between runs.
        matching.sort_by(|a, b| {
            b.defect_probability
                .partial_cmp(&a.defect_probability)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.file_path.cmp(&b.file_path))
        });

        let coupling_source = if matching.iter().any(|p| p.metrics.coupling_score.is_some()) {
            CouplingSource::EfferentImports {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                imports_at_full_scale: IMPORTS_AT_FULL_SCALE as u32,
            }
        } else {
            CouplingSource::NotMeasured {
                reason: "no analyzed file used an import form this counts".to_string(),
            }
        };

        Ok(build_result(
            files.len(),
            analyzed,
            matching,
            MetricSources {
                churn: churn.source,
                coupling: coupling_source,
                duplication: DuplicationSource::not_run(),
            },
            &request,
        ))
    }

    /// Discover source files to analyze, in a deterministic order.
    async fn discover_files(&self, request: &DefectPredictionRequest) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut files = Vec::new();
        // sort_by_file_name: readdir order is filesystem-dependent, and the
        // prediction list is derived from it — identical input must produce
        // identical output.
        for entry in WalkDir::new(&request.project_path)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.is_file() && Self::matches_filters(path, request) {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    /// Include/exclude patterns plus the source-extension allow-list.
    fn matches_filters(path: &Path, request: &DefectPredictionRequest) -> bool {
        let path_str = path.to_string_lossy();

        if let Some(ref excludes) = request.exclude {
            if excludes.iter().any(|pattern| path_str.contains(pattern)) {
                return false;
            }
        }

        if let Some(ref includes) = request.include {
            if !includes.iter().any(|pattern| path_str.contains(pattern)) {
                return false;
            }
        }

        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| matches!(ext, "rs" | "py" | "js" | "ts" | "cpp" | "c" | "java"))
    }

    /// Analyze a single file for defect probability
    async fn analyze_file(
        &self,
        file_path: &PathBuf,
        request: &DefectPredictionRequest,
        churn: &ChurnIndex,
    ) -> Result<FilePrediction> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let lines = content.lines().count();

        // Skip files below minimum line threshold
        if lines < request.min_lines {
            return Err(anyhow::anyhow!("File too small"));
        }

        let observation = churn.observation_for(file_path);
        let imports = count_imports(file_path, &content);

        #[allow(clippy::cast_precision_loss)]
        let metrics = FileRiskMetrics {
            complexity_score: (lines as f32 / 100.0).min(1.0),
            // Measured (#657) — no longer the constant 0.3.
            churn_score: observation.map(ChurnObservation::score),
            // Measured (#657) — no longer the constant 0.2.
            coupling_score: imports.map(|count| (count as f32 / IMPORTS_AT_FULL_SCALE).min(1.0)),
            size_score: (lines as f32 / 1000.0).min(1.0),
            // Absent (#657) — no longer the constant 0.1; this command runs no
            // clone detection, so there is nothing to report.
            duplication_score: None,
            lines,
            imports,
            commits_in_window: observation.map(|o| o.commits),
            changed_lines_in_window: observation.map(|o| o.changed_lines),
        };

        let defect_probability = combine_probability(&metrics);

        Ok(FilePrediction {
            file_path: file_path.display().to_string(),
            defect_probability,
            risk_level: classify(defect_probability),
            confidence: confidence_for(&metrics),
            contributing_factors: contributing_factors(&metrics),
            metrics,
        })
    }

    /// Quick analysis with defaults
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn quick_analysis(&self, project_path: PathBuf) -> Result<DefectPredictionResult> {
        let request = DefectPredictionRequest {
            project_path,
            confidence_threshold: 0.5,
            min_lines: 50,
            include_low_confidence: false,
            high_risk_only: false,
            include_recommendations: true,
            include: None,
            exclude: Some(vec!["test".to_string(), "vendor".to_string()]),
            top_files: 10,
        };

        self.analyze_project(request).await
    }
}

/// Confidence is the share of the model's weight that was measured FOR THIS
/// FILE — not a judgement, and not a constant.
///
/// #657: this was the literal `0.75` for every file inside any repository
/// (`distinct=1` over 109 files) with the comment "would be based on data
/// quality". It now moves with what could actually be read: 0.90 when git has
/// history for the file and its imports were counted, 0.65 for a file git does
/// not track, lower again for a language whose imports are not recognised.
fn confidence_for(metrics: &FileRiskMetrics) -> f32 {
    measured_weight(metrics).clamp(0.0, 1.0)
}

/// Contributing factors, each naming the measurement behind it.
///
/// "Frequent changes" used to be asserted about files changed exactly ONCE: the
/// churn score was normalized against the analyzed subset, so in a small scope
/// the max-normalizer pushed some file to 1.0 no matter how little it changed —
/// a fresh `git init` with ONE commit reported "Frequent changes" for both of
/// its files. The score is now absolute and the commit count is printed with
/// the claim, so the reader can check it.
fn contributing_factors(metrics: &FileRiskMetrics) -> Vec<String> {
    let mut factors = Vec::new();
    // One factor per input: `complexity_score` and `size_score` are both
    // derived from the same line count, so they must not be reported as two
    // independent findings.
    if metrics.complexity_score > 0.7 || metrics.size_score > 0.7 {
        factors.push(format!("Long file ({} lines)", metrics.lines));
    }
    if metrics
        .churn_score
        .is_some_and(|c| c > FREQUENT_CHANGE_SCORE)
    {
        let commits = metrics.commits_in_window.unwrap_or(0);
        factors.push(format!(
            "Frequent changes ({commits} commits in the last {CHURN_WINDOW_DAYS} days)"
        ));
    }
    if metrics.coupling_score.is_some_and(|c| c > 0.7) {
        let imports = metrics.imports.unwrap_or(0);
        factors.push(format!("High coupling ({imports} distinct imports)"));
    }
    factors
}

fn passes_filters(prediction: &FilePrediction, request: &DefectPredictionRequest) -> bool {
    if request.high_risk_only && matches!(prediction.risk_level, RiskLevel::Low | RiskLevel::Medium)
    {
        return false;
    }
    if !request.include_low_confidence && prediction.confidence < request.confidence_threshold {
        return false;
    }
    true
}

/// Where each risk factor came from, or why it is absent.
pub struct MetricSources {
    pub churn: ChurnSource,
    pub coupling: CouplingSource,
    pub duplication: DuplicationSource,
}

/// Assemble the result, keeping the counts honest about what each one is.
fn build_result(
    total_files_discovered: usize,
    total_files_analyzed: usize,
    matching: Vec<FilePrediction>,
    sources: MetricSources,
    request: &DefectPredictionRequest,
) -> DefectPredictionResult {
    let files_matching_filters = matching.len();
    let high_risk_files = matching
        .iter()
        .filter(|p| matches!(p.risk_level, RiskLevel::Critical | RiskLevel::High))
        .count();
    let medium_risk_files = matching
        .iter()
        .filter(|p| matches!(p.risk_level, RiskLevel::Medium))
        .count();
    let low_risk_files = matching
        .iter()
        .filter(|p| matches!(p.risk_level, RiskLevel::Low))
        .count();

    // `--top-files 0` is documented as "0 = all"; the old code truncated to
    // zero and reported nothing.
    let mut predictions = matching;
    let predictions_truncated = request.top_files > 0 && predictions.len() > request.top_files;
    if predictions_truncated {
        predictions.truncate(request.top_files);
    }
    let predictions_reported = predictions.len();

    let mut summary = format!(
        "Analyzed {total_files_analyzed} of {total_files_discovered} discovered files: \
         {high_risk_files} high risk, {medium_risk_files} medium risk, {low_risk_files} low risk. \
         Churn: {}. Coupling: {}. Duplication: {}.",
        sources.churn.describe(),
        sources.coupling.describe(),
        sources.duplication.describe()
    );
    if predictions_truncated {
        summary.push_str(&format!(
            " Showing top {predictions_reported} of {files_matching_filters} matching files \
             (--top-files {}).",
            request.top_files
        ));
    }

    let mut recommendations = Vec::new();
    if high_risk_files > 0 {
        recommendations.push("Focus testing and review efforts on high-risk files".to_string());
    }
    if request.include_recommendations {
        for prediction in predictions.iter().take(3) {
            if !prediction.contributing_factors.is_empty() {
                recommendations.push(format!(
                    "{}: Address {}",
                    prediction.file_path,
                    prediction.contributing_factors.join(", ")
                ));
            }
        }
    }

    DefectPredictionResult {
        total_files_discovered,
        total_files_analyzed,
        files_matching_filters,
        high_risk_files,
        medium_risk_files,
        low_risk_files,
        predictions_reported,
        predictions_truncated,
        churn_source: sources.churn,
        coupling_source: sources.coupling,
        duplication_source: sources.duplication,
        predictions,
        summary,
        recommendations,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_registry::ServiceRegistry;
    use tempfile::TempDir;

    fn facade() -> DefectPredictionFacade {
        DefectPredictionFacade::new(Arc::new(ServiceRegistry::new()))
    }

    fn request(path: &Path, top_files: usize) -> DefectPredictionRequest {
        DefectPredictionRequest {
            project_path: path.to_path_buf(),
            confidence_threshold: 0.0,
            min_lines: 0,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
            top_files,
        }
    }

    /// Write `count` distinct .rs files into `dir`.
    fn write_sources(dir: &Path, count: usize) {
        std::fs::create_dir_all(dir).unwrap();
        for i in 0..count {
            let body: String = (0..(10 + i)).map(|n| format!("// line {n}\n")).collect();
            std::fs::write(dir.join(format!("file_{i:03}.rs")), body).unwrap();
        }
    }

    fn init_repo(root: &Path) {
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .expect("git must be available");
            assert!(out.status.success(), "git {args:?}: {:?}", out.stderr);
        };
        git(&["init", "--initial-branch=main"]);
        git(&["config", "user.email", "t@example.com"]);
        git(&["config", "user.name", "T"]);
    }

    #[tokio::test]
    async fn test_defect_prediction_facade_creation() {
        let _facade = facade();
    }

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(RiskLevel::Critical, RiskLevel::Critical);
    }

    // ── #657: a cap must not be presented as a total ────────────────────────

    /// Observed defect: 3 files -> 3, 5 -> 5, 20 -> 10, and the entire
    /// 3863-file repository -> 10, because `total_files_analyzed` was
    /// `predictions.len()` after truncation to `top_files`.
    #[tokio::test]
    async fn test_totals_are_totals_not_the_top_files_cap() {
        let temp = TempDir::new().unwrap();
        write_sources(temp.path(), 25);

        let result = facade()
            .analyze_project(request(temp.path(), 10))
            .await
            .unwrap();

        assert_eq!(result.total_files_discovered, 25);
        assert_eq!(
            result.total_files_analyzed, 25,
            "25 files were analyzed; reporting the --top-files cap (10) as the total is the bug"
        );
        assert_eq!(result.predictions_reported, 10);
        assert!(result.predictions_truncated);
        assert!(
            result.summary.contains("25"),
            "summary must state the real total: {}",
            result.summary
        );
    }

    #[tokio::test]
    async fn test_no_truncation_flag_when_under_cap() {
        let temp = TempDir::new().unwrap();
        write_sources(temp.path(), 4);

        let result = facade()
            .analyze_project(request(temp.path(), 10))
            .await
            .unwrap();

        assert_eq!(result.total_files_analyzed, 4);
        assert_eq!(result.predictions_reported, 4);
        assert!(!result.predictions_truncated);
    }

    /// `--top-files 0` is documented as "0 = all"; it used to truncate to zero.
    #[tokio::test]
    async fn test_top_files_zero_means_all() {
        let temp = TempDir::new().unwrap();
        write_sources(temp.path(), 7);

        let result = facade()
            .analyze_project(request(temp.path(), 0))
            .await
            .unwrap();

        assert_eq!(result.predictions_reported, 7);
        assert!(!result.predictions_truncated);
    }

    // ── #657: churn must be measured, or absent — never a constant ──────────

    #[tokio::test]
    async fn test_churn_is_absent_not_defaulted_outside_a_repo() {
        let temp = TempDir::new().unwrap();
        write_sources(temp.path(), 3);

        let result = facade()
            .analyze_project(request(temp.path(), 10))
            .await
            .unwrap();

        assert!(matches!(
            result.churn_source,
            ChurnSource::NotMeasured { .. }
        ));
        for p in &result.predictions {
            assert_eq!(
                p.metrics.churn_score, None,
                "unmeasurable churn must be null, never 0.0 and never a default"
            );
        }
    }

    /// The heart of #657: a NON-ROOT path inside a repository must still get
    /// measured churn. Pre-fix this produced `churn_score: null` for every
    /// file because only `<path>/.git` was checked.
    #[tokio::test]
    async fn test_churn_is_measured_from_a_subdirectory_of_a_repo() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        init_repo(root);
        let subdir = root.join("src/utils");
        write_sources(&subdir, 3);

        for i in 0..2 {
            std::fs::write(
                subdir.join("file_000.rs"),
                format!("// revision {i}\nfn f() {{}}\n"),
            )
            .unwrap();
            let out = std::process::Command::new("git")
                .args(["add", "."])
                .current_dir(root)
                .output()
                .unwrap();
            assert!(out.status.success());
            let out = std::process::Command::new("git")
                .args(["commit", "-m", "c"])
                .current_dir(root)
                .output()
                .unwrap();
            assert!(out.status.success());
        }

        let result = facade()
            .analyze_project(request(&subdir, 10))
            .await
            .unwrap();

        assert!(
            matches!(result.churn_source, ChurnSource::GitHistory { .. }),
            "churn must be measured for a subdirectory of a repo, got {:?}",
            result.churn_source
        );
        assert!(
            result
                .predictions
                .iter()
                .any(|p| p.metrics.churn_score.is_some_and(|c| c > 0.0)),
            "at least the committed file must have non-zero measured churn"
        );
    }

    // ── determinism ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_analysis_is_identical_across_5_runs() {
        let temp = TempDir::new().unwrap();
        write_sources(temp.path(), 30);

        let first = serde_json::to_string(
            &facade()
                .analyze_project(request(temp.path(), 10))
                .await
                .unwrap(),
        )
        .unwrap();
        for i in 1..5 {
            let again = serde_json::to_string(
                &facade()
                    .analyze_project(request(temp.path(), 10))
                    .await
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(first, again, "defect prediction differed on run {i}");
        }
    }

    // ── probability derivation ──────────────────────────────────────────────

    /// Metrics with every factor measured, for the derivation tests.
    fn metrics_all_measured(churn: Option<f32>, coupling: Option<f32>) -> FileRiskMetrics {
        FileRiskMetrics {
            complexity_score: 1.0,
            churn_score: churn,
            coupling_score: coupling,
            size_score: 1.0,
            duplication_score: None,
            lines: 1000,
            imports: coupling.map(|_| 20),
            commits_in_window: churn.map(|_| 4),
            changed_lines_in_window: churn.map(|_| 80),
        }
    }

    #[test]
    fn test_probability_renormalizes_when_churn_absent() {
        let base = metrics_all_measured(None, Some(1.0));
        // All measured factors maxed and churn absent → 1.0, not 0.75. A
        // missing factor must not silently drag the score down.
        assert!((combine_probability(&base) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_probability_varies_with_measured_churn() {
        let metrics = |churn: f32| FileRiskMetrics {
            complexity_score: 0.5,
            churn_score: Some(churn),
            coupling_score: Some(0.2),
            size_score: 0.1,
            duplication_score: None,
            lines: 50,
            imports: Some(4),
            commits_in_window: Some(2),
            changed_lines_in_window: Some(20),
        };
        assert!(combine_probability(&metrics(0.9)) > combine_probability(&metrics(0.1)));
    }

    #[test]
    fn test_confidence_drops_when_churn_unmeasured() {
        let measured = metrics_all_measured(Some(0.4), Some(0.2));
        let mut absent = measured.clone();
        absent.churn_score = None;
        assert!(confidence_for(&absent) < confidence_for(&measured));
    }

    #[test]
    fn test_churn_source_describe_says_not_measured() {
        let src = ChurnSource::NotMeasured {
            reason: "No git repository found".to_string(),
        };
        assert!(src.describe().contains("not measured"));
    }

    // ── #657 round 3: coupling, duplication and confidence ──────────────────

    /// Observed: `jq '[.predictions[].metrics.coupling_score]|unique'` returned
    /// `[0.2]` over 109 real files — one distinct value, because the field was
    /// the literal `0.2`. It must now move with the file's own imports.
    #[tokio::test]
    async fn test_coupling_varies_per_file_and_is_derived_from_imports() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("bare.rs"),
            "fn a() {}\nfn b() {}\nfn c() {}\n",
        )
        .unwrap();
        let many: String = (0..30)
            .map(|i| format!("use std::collections::x{i};\n"))
            .collect();
        std::fs::write(
            temp.path().join("coupled.rs"),
            format!("{many}fn a() {{}}\n"),
        )
        .unwrap();

        let result = facade()
            .analyze_project(request(temp.path(), 0))
            .await
            .unwrap();

        let score = |name: &str| {
            result
                .predictions
                .iter()
                .find(|p| p.file_path.ends_with(name))
                .unwrap_or_else(|| panic!("{name} missing"))
                .metrics
                .clone()
        };
        let bare = score("bare.rs");
        let coupled = score("coupled.rs");
        assert_eq!(bare.imports, Some(0));
        assert_eq!(coupled.imports, Some(30));
        assert_eq!(bare.coupling_score, Some(0.0));
        assert_eq!(coupled.coupling_score, Some(1.0));
        assert!(
            bare.coupling_score != coupled.coupling_score,
            "coupling must not be one constant for every file"
        );
        assert!(matches!(
            result.coupling_source,
            CouplingSource::EfferentImports { .. }
        ));
    }

    /// Observed: `duplication_score` was the literal `0.1` for every file, with
    /// no provenance. This command runs no clone detection, so it must be
    /// absent with a reason — never a plausible default and never 0.0.
    #[tokio::test]
    async fn test_duplication_is_absent_with_a_reason() {
        let temp = TempDir::new().unwrap();
        write_sources(temp.path(), 3);

        let result = facade()
            .analyze_project(request(temp.path(), 0))
            .await
            .unwrap();

        for p in &result.predictions {
            assert_eq!(
                p.metrics.duplication_score, None,
                "unmeasured duplication must be null, never the old 0.1 and never 0.0"
            );
        }
        let DuplicationSource::NotMeasured { reason } = &result.duplication_source;
        assert!(
            reason.contains("clone detection"),
            "the reason must say what was not run: {reason}"
        );
        assert!(result.summary.contains("Duplication: not measured"));
    }

    /// Observed: `confidence` was 0.75 for every file inside any git repo
    /// (distinct=1 over 109 files). It is now the share of the model that was
    /// measured FOR THAT FILE, so a file git does not track scores lower.
    #[test]
    fn test_confidence_is_the_measured_share_not_a_constant() {
        let full = metrics_all_measured(Some(0.4), Some(0.5));
        let no_churn = metrics_all_measured(None, Some(0.5));
        let no_coupling = metrics_all_measured(Some(0.4), None);

        // complexity .30 + size .15 + churn .25 + coupling .20 = 0.90
        assert!((confidence_for(&full) - 0.90).abs() < 1e-5);
        assert!((confidence_for(&no_churn) - 0.65).abs() < 1e-5);
        assert!((confidence_for(&no_coupling) - 0.70).abs() < 1e-5);
        assert!(confidence_for(&no_churn) < confidence_for(&full));
    }

    /// Observed: a fresh `git init` with ONE commit reported
    /// `contributing_factors: ["Frequent changes"]` for both of its files,
    /// because the churn score was normalized against the analyzed subset —
    /// the maximum in the set is always 1.0, however small it is.
    #[tokio::test]
    async fn test_one_commit_is_not_frequent_changes() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        init_repo(root);
        std::fs::write(root.join("x.rs"), "fn x() {}\n").unwrap();
        std::fs::write(root.join("y.rs"), "fn y() {}\n").unwrap();
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap();
            assert!(out.status.success());
        };
        git(&["add", "."]);
        git(&["commit", "-m", "one"]);

        let result = facade().analyze_project(request(root, 0)).await.unwrap();

        assert_eq!(result.predictions.len(), 2);
        for p in &result.predictions {
            assert_eq!(p.metrics.commits_in_window, Some(1));
            assert!(
                p.metrics
                    .churn_score
                    .is_some_and(|c| c < FREQUENT_CHANGE_SCORE),
                "1 commit must not score as high churn: {:?}",
                p.metrics.churn_score
            );
            assert!(
                !p.contributing_factors
                    .iter()
                    .any(|f| f.contains("Frequent")),
                "a file changed once is not 'Frequent changes': {:?}",
                p.contributing_factors
            );
        }
    }

    /// Observed: `cold.rs` (1 commit) scored churn 0.142 from the repo root and
    /// 1.0 from its own directory, with a 2.6x higher defect probability —
    /// the score was a property of the command line, not of the file.
    #[tokio::test]
    async fn test_churn_is_a_property_of_the_file_not_of_the_scope() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        init_repo(root);
        let sub = root.join("sub/deep");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("cold.rs"), "fn cold() {}\n").unwrap();
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap();
            assert!(out.status.success());
        };
        git(&["add", "."]);
        git(&["commit", "-m", "one"]);
        // A second, much busier file so the max-normalizer would have had
        // something to normalize against at the root but not in the subtree.
        for i in 0..8 {
            std::fs::write(root.join("hot.rs"), format!("// rev {i}\nfn hot() {{}}\n")).unwrap();
            git(&["add", "."]);
            git(&["commit", "-m", "hot"]);
        }

        let from_root = facade().analyze_project(request(root, 0)).await.unwrap();
        let from_sub = facade().analyze_project(request(&sub, 0)).await.unwrap();

        let cold_at = |r: &DefectPredictionResult| {
            r.predictions
                .iter()
                .find(|p| p.file_path.ends_with("cold.rs"))
                .expect("cold.rs must be predicted")
                .clone()
        };
        let a = cold_at(&from_root);
        let b = cold_at(&from_sub);
        assert_eq!(
            a.metrics.churn_score, b.metrics.churn_score,
            "the same file must score the same churn whatever -p is used"
        );
        assert!(
            (a.defect_probability - b.defect_probability).abs() < f32::EPSILON,
            "defect probability moved with the analysis scope: {} vs {}",
            a.defect_probability,
            b.defect_probability
        );
    }

    #[test]
    fn test_churn_observation_score_is_absolute() {
        // 20 commits = full commit scale; 1 commit is nowhere near it.
        let one = ChurnObservation {
            commits: 1,
            changed_lines: 10,
        };
        let many = ChurnObservation {
            commits: 20,
            changed_lines: 1000,
        };
        assert!(one.score() < 0.1, "1 commit scored {}", one.score());
        assert!((many.score() - 1.0).abs() < f32::EPSILON);
        // Saturating, never above 1.0.
        let absurd = ChurnObservation {
            commits: 10_000,
            changed_lines: 10_000_000,
        };
        assert!((absurd.score() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_count_imports_is_none_for_unknown_languages() {
        assert_eq!(count_imports(Path::new("a.txt"), "import os\n"), None);
        assert_eq!(
            count_imports(Path::new("a.py"), "import os\nimport os\n"),
            Some(1)
        );
        assert_eq!(
            count_imports(Path::new("a.rs"), "use a;\nuse b;\nfn f() {}\n"),
            Some(2)
        );
    }
}
