#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::ComprehensiveAnalysisConfig;
use crate::services::facades::analysis_orchestrator::{
    AnalysisOrchestrator, ComprehensiveAnalysisRequest, ComprehensiveAnalysisResult,
};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn init_timing(perf: bool) -> Option<std::time::Instant> {
    if perf {
        Some(std::time::Instant::now())
    } else {
        None
    }
}

pub(super) fn determine_analysis_path(config: &ComprehensiveAnalysisConfig) -> PathBuf {
    if let Some(single_file) = &config.file {
        single_file.clone()
    } else if !config.files.is_empty() {
        // Multiple files - analyze the common parent directory
        // For now, just use the project path
        config.project_path.clone()
    } else {
        // Full project analysis
        config.project_path.clone()
    }
}

pub(super) async fn run_orchestrated_analysis(
    analysis_path: PathBuf,
    config: &ComprehensiveAnalysisConfig,
) -> Result<ComprehensiveAnalysisResult> {
    let registry = Arc::new(ServiceRegistry::new());
    let orchestrator = AnalysisOrchestrator::new(registry);

    let request = create_analysis_request(analysis_path, config);
    orchestrator.analyze(request).await
}

pub(super) fn create_analysis_request(
    path: PathBuf,
    config: &ComprehensiveAnalysisConfig,
) -> ComprehensiveAnalysisRequest {
    ComprehensiveAnalysisRequest {
        path,
        include_complexity: config.include_complexity,
        include_dead_code: config.include_dead_code,
        include_satd: config.include_tdg, // Using TDG flag for SATD
        include_tests: false,
        language: None, // Auto-detect
        parallel: true, // Use parallel execution for performance
    }
}

pub(super) async fn enhance_results_if_needed(
    result: ComprehensiveAnalysisResult,
    config: &ComprehensiveAnalysisConfig,
) -> Result<ComprehensiveAnalysisResult> {
    if config.include_duplicates || config.include_defects {
        let additional_config = create_additional_config(config);
        enhance_with_additional_analyses(result, additional_config).await
    } else {
        Ok(result)
    }
}

/// Build the config for the sub-analyses the orchestrator does not cover.
///
/// The scope is `determine_analysis_path(config)` — the same path the
/// orchestrated analysis uses — NOT `config.project_path`. Handing these
/// sub-analyses the project path (default ".") meant `analyze comprehensive
/// --file <one file>` ran defect prediction over the caller's *current
/// directory* and attributed the result to that single file: identical
/// commands reported `total_issues: 49498` from inside this repo and `0` from
/// an empty cwd, and stderr leaked the tell ("churn not measured for .").
pub(super) fn create_additional_config(
    config: &ComprehensiveAnalysisConfig,
) -> AdditionalAnalysisConfig<'_> {
    AdditionalAnalysisConfig {
        project_path: determine_analysis_path(config),
        include_duplicates: config.include_duplicates,
        include_defects: config.include_defects,
        confidence_threshold: config.confidence_threshold,
        min_lines: config.min_lines,
        include: &config.include,
        exclude: &config.exclude,
        top_files: config.top_files,
    }
}

pub(super) fn report_completion_and_performance(
    start: Option<std::time::Instant>,
    config: &ComprehensiveAnalysisConfig,
    result: &ComprehensiveAnalysisResult,
) {
    use crate::cli::colors as c_;
    if let Some(start_time) = start {
        let elapsed = start_time.elapsed();
        eprintln!(
            "{} Comprehensive analysis completed in {}{elapsed:?}{}",
            c_::pass(""),
            c_::BOLD_WHITE,
            c_::RESET
        );

        if config.perf {
            print_performance_breakdown(result, elapsed.as_millis() as u64);
        }
    } else {
        eprintln!("{}", c_::pass("Comprehensive analysis completed"));
    }
}

/// Configuration for additional analyses
pub(super) struct AdditionalAnalysisConfig<'a> {
    /// The path these sub-analyses must scan. This is the *resolved* analysis
    /// scope from `determine_analysis_path`, so under `--file` it is that one
    /// file — it is not necessarily `config.project_path`.
    pub(super) project_path: PathBuf,
    pub(super) include_duplicates: bool,
    pub(super) include_defects: bool,
    pub(super) confidence_threshold: f32,
    pub(super) min_lines: usize,
    pub(super) include: &'a Option<String>,
    pub(super) exclude: &'a Option<String>,
    pub(super) top_files: usize,
}

/// Enhance results with additional analyses not covered by the orchestrator
pub(super) async fn enhance_with_additional_analyses(
    mut result: ComprehensiveAnalysisResult,
    config: AdditionalAnalysisConfig<'_>,
) -> Result<ComprehensiveAnalysisResult> {
    // Duplicate detection is NOT wired into comprehensive analysis. It used to
    // announce "👥 Detecting duplicates..." and then push the developer note
    // "Duplicate detection analysis requested - integrate with duplicate
    // detector" into the user-facing recommendations, which reads as a finding
    // about the codebase rather than a missing feature — and the flag defaults
    // to on, so every run carried it. Say what is true, on stderr, and leave
    // the report free of a result nothing measured.
    if config.include_duplicates {
        eprintln!(
            "ℹ️  Duplicate detection is not part of comprehensive analysis; \
             run `pmat analyze duplicates` for clone results."
        );
    }

    // Add defect prediction if requested
    if config.include_defects {
        eprintln!("🐛 Predicting defects...");

        // Use our defect prediction facade
        use crate::services::facades::defect_prediction_facade::{
            DefectPredictionFacade, DefectPredictionRequest,
        };
        use crate::services::service_registry::ServiceRegistry;

        let registry = Arc::new(ServiceRegistry::new());
        let facade = DefectPredictionFacade::new(registry);

        let request = DefectPredictionRequest {
            project_path: config.project_path.clone(),
            confidence_threshold: config.confidence_threshold,
            min_lines: config.min_lines,
            include_low_confidence: false,
            high_risk_only: false,
            include_recommendations: true,
            include: config.include.as_ref().map(|s| vec![s.clone()]),
            exclude: config.exclude.as_ref().map(|s| vec![s.clone()]),
            top_files: config.top_files,
        };

        if let Ok(defect_result) = facade.analyze_project(request).await {
            record_defect_prediction(&mut result.summary, defect_result.high_risk_files);
        }
    }

    Ok(result)
}

/// Fold defect prediction's `high_risk_files` into the summary — as a
/// recommendation only.
///
/// It used to be added to BOTH `total_issues` and `critical_issues`. That is a
/// FILE count going into an ISSUE counter, and the two are not commensurable:
/// this repo reported `total_files: 4355` with `total_issues: 50747` and
/// `critical_issues: 49784`, of which 49498 were "high-risk files" — eleven
/// times more issues than there are files, while the analyses actually present
/// in the report summed to 1249. An issue count must stay bounded by what the
/// listed analyses found.
pub(super) fn record_defect_prediction(
    summary: &mut crate::services::facades::analysis_orchestrator::AnalysisSummary,
    high_risk_files: usize,
) {
    if high_risk_files > 0 {
        summary.recommendations.push(format!(
            "Focus on {high_risk_files} high-risk files identified by defect prediction"
        ));
    }
}

/// Print performance breakdown
pub(super) fn print_performance_breakdown(result: &ComprehensiveAnalysisResult, total_ms: u64) {
    use crate::cli::colors as c_;
    eprintln!("\n{}", c_::subheader("⏱️  Performance Breakdown:"));
    eprintln!(
        "  {}: {}ms",
        c_::label("Total execution time"),
        c_::number(&total_ms.to_string())
    );
    eprintln!(
        "  {}: {}ms",
        c_::label("Analysis duration"),
        c_::number(&result.duration_ms.to_string())
    );
    eprintln!(
        "  {}: {}",
        c_::label("Files analyzed"),
        c_::number(&result.summary.total_files.to_string())
    );
    eprintln!(
        "  {}: {}",
        c_::label("Issues found"),
        c_::number(&result.summary.total_issues.to_string())
    );

    if result.summary.total_files > 0 {
        let ms_per_file = total_ms as f64 / result.summary.total_files as f64;
        eprintln!(
            "  {}: {}ms",
            c_::label("Average time per file"),
            c_::number(&format!("{ms_per_file:.2}"))
        );
    }
}

#[cfg(test)]
mod additional_config_scope_tests {
    //! Regression tests for `--file` scoping. The duplicates/defect-prediction
    //! enhancement used to receive `config.project_path` (default ".") while
    //! the orchestrated analysis received the `--file` path, so the sub-analysis
    //! scanned the caller's current directory and its counts were attributed to
    //! the single file.
    use super::{create_additional_config, determine_analysis_path};
    use crate::cli::handlers::comprehensive_analysis_handler::types::ComprehensiveAnalysisConfig;
    use crate::cli::ComprehensiveOutputFormat;
    use std::path::{Path, PathBuf};

    fn config_with_file(project_path: &str, file: Option<&str>) -> ComprehensiveAnalysisConfig {
        ComprehensiveAnalysisConfig {
            project_path: PathBuf::from(project_path),
            file: file.map(PathBuf::from),
            files: Vec::new(),
            format: ComprehensiveOutputFormat::Json,
            include_duplicates: false,
            include_dead_code: false,
            include_defects: true,
            include_complexity: false,
            include_tdg: false,
            confidence_threshold: 0.5,
            min_lines: 10,
            include: None,
            exclude: None,
            output: None,
            perf: false,
            executive_summary: false,
            top_files: 10,
        }
    }

    #[test]
    fn test_single_file_scopes_the_additional_analyses_to_that_file() {
        let config = config_with_file(".", Some("/corpus/tiny/src/lib.rs"));
        let additional = create_additional_config(&config);

        assert_eq!(
            additional.project_path,
            Path::new("/corpus/tiny/src/lib.rs"),
            "--file must scope defect prediction to that file, not to the cwd"
        );
        assert_ne!(
            additional.project_path,
            Path::new("."),
            "passing \".\" here is what made the issue counts depend on the caller's cwd"
        );
    }

    /// `high_risk_files` is a FILE count from defect prediction; it used to be
    /// added to `total_issues` AND `critical_issues`, so a 4,355-file project
    /// reported 50,747 issues of which 49,498 were files.
    #[test]
    fn test_defect_prediction_does_not_inflate_the_issue_counters() {
        use super::record_defect_prediction;
        use crate::services::facades::analysis_orchestrator::AnalysisSummary;

        let mut summary = AnalysisSummary {
            total_files: 4355,
            total_issues: 1249,
            critical_issues: 286,
            quality_score: 97.0,
            recommendations: Vec::new(),
        };

        record_defect_prediction(&mut summary, 49498);

        assert_eq!(
            summary.total_issues, 1249,
            "a file count must not be added to the issue count"
        );
        assert_eq!(
            summary.critical_issues, 286,
            "a file count must not be added to the critical-issue count"
        );
        assert!(
            summary.total_issues <= summary.total_files * 100,
            "issue counts must stay bounded by the analyzed corpus"
        );
        assert!(
            summary
                .recommendations
                .iter()
                .any(|r| r.contains("49498 high-risk files")),
            "the figure is still reported, as its own recommendation: {:?}",
            summary.recommendations
        );
    }

    #[test]
    fn test_scope_matches_the_orchestrated_analysis_path() {
        for (project, file) in [
            (".", Some("src/lib.rs")),
            ("/some/project", None),
            ("/some/project", Some("/other/file.rs")),
        ] {
            let config = config_with_file(project, file);
            assert_eq!(
                create_additional_config(&config).project_path,
                determine_analysis_path(&config),
                "every sub-analysis must use the same resolved path as the orchestrator"
            );
        }
    }
}
