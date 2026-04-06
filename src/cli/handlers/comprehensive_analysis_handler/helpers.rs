#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::ComprehensiveAnalysisConfig;
use crate::services::facades::analysis_orchestrator::{
    AnalysisOrchestrator, ComprehensiveAnalysisRequest, ComprehensiveAnalysisResult,
};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::{Path, PathBuf};
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
    debug_assert!(
        analysis_path.exists(),
        "analysis_path must exist: {}",
        analysis_path.display()
    );
    let registry = Arc::new(ServiceRegistry::new());
    let orchestrator = AnalysisOrchestrator::new(registry);

    let request = create_analysis_request(analysis_path, config);
    orchestrator.analyze(request).await
}

pub(super) fn create_analysis_request(
    path: PathBuf,
    config: &ComprehensiveAnalysisConfig,
) -> ComprehensiveAnalysisRequest {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
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

pub(super) fn create_additional_config(
    config: &ComprehensiveAnalysisConfig,
) -> AdditionalAnalysisConfig<'_> {
    AdditionalAnalysisConfig {
        project_path: &config.project_path,
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
    pub(super) project_path: &'a Path,
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
    // Add duplicate detection if requested
    if config.include_duplicates {
        eprintln!("👥 Detecting duplicates...");
        // Would integrate with duplicate detector service
        // For now, just note it in the summary
        result.summary.recommendations.push(
            "Duplicate detection analysis requested - integrate with duplicate detector"
                .to_string(),
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
            project_path: config.project_path.to_path_buf(),
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
            result.summary.total_issues += defect_result.high_risk_files;
            result.summary.critical_issues += defect_result.high_risk_files;

            if defect_result.high_risk_files > 0 {
                result.summary.recommendations.push(format!(
                    "Focus on {} high-risk files identified by defect prediction",
                    defect_result.high_risk_files
                ));
            }
        }
    }

    Ok(result)
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
