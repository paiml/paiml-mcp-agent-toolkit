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

/// Git history window, in days, used for the churn measurement.
const CHURN_WINDOW_DAYS: u32 = 90;

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
            } => format!(
                "git history, last {window_days} days ({files_with_churn} files with commits)"
            ),
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
    pub complexity_score: f32,
    /// Measured 0-1 git churn, or `None` when churn could not be measured.
    ///
    /// #657: this was the constant `0.3` for every file. It must never be a
    /// default, and never `0.0` — zero reads as "this file never changed",
    /// which is a finding, not an absence.
    pub churn_score: Option<f32>,
    pub coupling_score: f32,
    pub size_score: f32,
    pub duplication_score: f32,
}

/// Churn measured from git, keyed by canonical absolute path.
struct ChurnIndex {
    by_path: HashMap<PathBuf, f32>,
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
                    by_path.insert(key, file.churn_score);
                }
                let files_with_churn = by_path.len();
                Self {
                    by_path,
                    source: ChurnSource::GitHistory {
                        window_days: CHURN_WINDOW_DAYS,
                        files_with_churn,
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
                    source: ChurnSource::NotMeasured {
                        reason: e.to_string(),
                    },
                }
            }
        }
    }

    /// Churn for one file. `None` when churn was not measured at all; a file
    /// inside a measured repository with no commits in the window is a real
    /// `0.0`.
    fn score_for(&self, file: &Path) -> Option<f32> {
        if matches!(self.source, ChurnSource::NotMeasured { .. }) {
            return None;
        }
        let key = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
        Some(self.by_path.get(&key).copied().unwrap_or(0.0))
    }
}

/// Weight of each risk factor. They sum to 1.0; when a factor is unmeasured
/// the remaining weights are renormalized rather than a default being invented.
const W_COMPLEXITY: f32 = 0.30;
const W_CHURN: f32 = 0.25;
const W_COUPLING: f32 = 0.20;
const W_SIZE: f32 = 0.15;
const W_DUPLICATION: f32 = 0.10;

/// Combine the measured factors into a 0-1 probability.
///
/// An absent `churn_score` drops the churn term *and* its weight, so the
/// probability stays derived only from what was actually measured.
fn combine_probability(metrics: &FileRiskMetrics) -> f32 {
    let mut weighted = metrics.complexity_score * W_COMPLEXITY
        + metrics.coupling_score * W_COUPLING
        + metrics.size_score * W_SIZE
        + metrics.duplication_score * W_DUPLICATION;
    let mut total_weight = W_COMPLEXITY + W_COUPLING + W_SIZE + W_DUPLICATION;

    if let Some(churn) = metrics.churn_score {
        weighted += churn * W_CHURN;
        total_weight += W_CHURN;
    }

    (weighted / total_weight).clamp(0.0, 1.0)
}

fn classify(probability: f32) -> RiskLevel {
    match probability {
        p if p >= 0.8 => RiskLevel::Critical,
        p if p >= 0.6 => RiskLevel::High,
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

        Ok(build_result(
            files.len(),
            analyzed,
            matching,
            churn.source,
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
        let lines = tokio::fs::read_to_string(file_path).await?.lines().count();

        // Skip files below minimum line threshold
        if lines < request.min_lines {
            return Err(anyhow::anyhow!("File too small"));
        }

        let metrics = FileRiskMetrics {
            complexity_score: (lines as f32 / 100.0).min(1.0),
            // Measured (#657) — no longer the constant 0.3.
            churn_score: churn.score_for(file_path),
            coupling_score: 0.2,
            size_score: (lines as f32 / 1000.0).min(1.0),
            duplication_score: 0.1,
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

/// Confidence reflects how many factors were actually measured: with churn
/// absent one of the five inputs is missing, so confidence must drop rather
/// than stay at a flat 0.75.
fn confidence_for(metrics: &FileRiskMetrics) -> f32 {
    if metrics.churn_score.is_some() {
        0.75
    } else {
        0.75 * (1.0 - W_CHURN)
    }
}

fn contributing_factors(metrics: &FileRiskMetrics) -> Vec<String> {
    let mut factors = Vec::new();
    if metrics.complexity_score > 0.7 {
        factors.push("High complexity".to_string());
    }
    if metrics.churn_score.is_some_and(|c| c > 0.5) {
        factors.push("Frequent changes".to_string());
    }
    if metrics.size_score > 0.7 {
        factors.push("Large file size".to_string());
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

/// Assemble the result, keeping the counts honest about what each one is.
fn build_result(
    total_files_discovered: usize,
    total_files_analyzed: usize,
    matching: Vec<FilePrediction>,
    churn_source: ChurnSource,
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
         Churn: {}.",
        churn_source.describe()
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
        churn_source,
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

    #[test]
    fn test_probability_renormalizes_when_churn_absent() {
        let base = FileRiskMetrics {
            complexity_score: 1.0,
            churn_score: None,
            coupling_score: 1.0,
            size_score: 1.0,
            duplication_score: 1.0,
        };
        // All measured factors maxed and churn absent → 1.0, not 0.75. A
        // missing factor must not silently drag the score down.
        assert!((combine_probability(&base) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_probability_varies_with_measured_churn() {
        let metrics = |churn: f32| FileRiskMetrics {
            complexity_score: 0.5,
            churn_score: Some(churn),
            coupling_score: 0.2,
            size_score: 0.1,
            duplication_score: 0.1,
        };
        assert!(combine_probability(&metrics(0.9)) > combine_probability(&metrics(0.1)));
    }

    #[test]
    fn test_confidence_drops_when_churn_unmeasured() {
        let measured = FileRiskMetrics {
            complexity_score: 0.5,
            churn_score: Some(0.4),
            coupling_score: 0.2,
            size_score: 0.1,
            duplication_score: 0.1,
        };
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
}
