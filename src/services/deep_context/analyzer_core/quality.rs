#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::Path;
use std::time::Duration;

use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::debug;

use crate::services::complexity::ComplexityReport;
use crate::services::deep_context::analyzer_core::types::ParallelAnalysisResults;
use crate::services::deep_context::DeepContextAnalyzer;
use crate::services::deep_context::{
    AstSummary, ComplexityMetricsForQA, ComplexitySummaryForQA, DeadCodeAnalysis, DeadCodeSummary,
    DeepContext, DeepContextResult, DefectSummary, FileComplexityMetricsForQA,
    FunctionComplexityForQA, Impact, PrioritizedRecommendation, Priority, QualityScorecard,
};
use crate::services::quality_gates::{QAVerification, QAVerificationResult};

/// Mean of the scorecard components that were actually measured.
///
/// `None` when nothing was measured — "we did not look" must not be reported as
/// a score. Every measured component participates; leaving one out is how
/// `overall_health` came to be a restatement of `complexity_score`.
fn average_measured(components: &[Option<f64>]) -> Option<f64> {
    let measured: Vec<f64> = components.iter().copied().flatten().collect();
    if measured.is_empty() {
        None
    } else {
        #[allow(clippy::cast_precision_loss)]
        Some(measured.iter().sum::<f64>() / measured.len() as f64)
    }
}

impl DeepContextAnalyzer {
    // New helper methods for phases that didn't exist before
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub(crate) async fn calculate_quality_scorecard(
        &self,
        analyses: &ParallelAnalysisResults,
        _defect_summary: &DefectSummary,
        project_path: &Path,
    ) -> anyhow::Result<QualityScorecard> {
        // Only measured values reach the scorecard. Four of these fields used to
        // be literals tagged `// Placeholder for now`, which made
        // `overall_health` identically `(complexity_score + 155) / 3` — so it
        // could only ever take three values, and an empty directory reported the
        // same "85.0% healthy, 65.0% covered" as a 3252-file repository.

        // Real: derived from violations actually found. Absent when no
        // complexity analysis ran — previously defaulted to 75.0, which is a
        // measurement-shaped answer to "we did not look".
        // The `files.is_empty()` guard matters: with nothing analysed there are
        // trivially zero violations, and the old arithmetic turned that into a
        // perfect 100.0 — an empty directory scoring better than real code.
        // "No files were analysed" is not a quality result.
        let complexity_score = analyses
            .complexity_report
            .as_ref()
            .filter(|report| !report.files.is_empty())
            .map(|report| {
                let violation_penalty = (report.violations.len() as f64 * 5.0).min(50.0);
                100.0 - violation_penalty
            });

        // Real: already computed by calculate_technical_debt() and sitting one
        // field away from where 40.0 was being invented.
        let technical_debt_hours = analyses
            .complexity_report
            .as_ref()
            .map(|report| f64::from(report.summary.technical_debt_hours));

        // Real when a coverage artifact exists, honestly absent otherwise.
        // `context` runs no coverage tool, so this reads whatever lcov/llvm-cov
        // output is already on disk and reports nothing when there is none.
        // It never shells out; see discover_line_coverage.
        let test_coverage = Self::measured_line_coverage(project_path);

        // Not measured. The SEI maintainability formula needs a Halstead volume,
        // and `ComplexityMetrics.halstead` is `None` at every production
        // construction site, so there is nothing to compute from. Modularity has
        // no implementation at all — the graph modularity in src/graph is
        // Newman-Girvan Q over a community assignment, a different quantity in a
        // different range, and relabelling it "Modularity Score %" would be a
        // new lie rather than a fix.
        let maintainability_index = None;
        let modularity_score = None;

        // Averaging over only what was measured. Averaging in a constant is how
        // the old value became a function of one real input and two literals.
        //
        // `test_coverage` used to be left out of this list. Since the other two
        // entries are permanently `None`, that made `overall_health` identically
        // `complexity_score` — a four-component scorecard whose headline number
        // reported one component and discarded the coverage it had just
        // measured off disk.
        let overall_health = average_measured(&[
            complexity_score,
            maintainability_index,
            modularity_score,
            test_coverage,
        ]);

        Ok(QualityScorecard {
            overall_health,
            complexity_score,
            maintainability_index,
            modularity_score,
            test_coverage,
            technical_debt_hours,
        })
    }

    /// Percentage of instrumented lines already recorded on disk, if any.
    ///
    /// Reads an existing lcov/llvm-cov artifact; never runs a coverage tool.
    /// Returns `None` when nothing is instrumented, so the caller reports
    /// "not measured" rather than inventing a figure.
    fn measured_line_coverage(project_root: &Path) -> Option<f64> {
        let coverage = crate::services::agent_context::query::discover_line_coverage(project_root)?;
        let (mut total, mut hit) = (0u64, 0u64);
        for lines in coverage.values() {
            for count in lines.values() {
                total += 1;
                if *count > 0 {
                    hit += 1;
                }
            }
        }
        if total == 0 {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        Some((hit as f64 / total as f64) * 100.0)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn generate_recommendations(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
    ) -> anyhow::Result<Vec<PrioritizedRecommendation>> {
        let mut recommendations = Vec::new();

        // Extract Method: Each recommendation type is handled by a focused method
        self.add_complexity_recommendations(&mut recommendations, analyses);
        self.add_defect_recommendations(&mut recommendations, defect_summary);
        self.add_satd_recommendations(&mut recommendations, analyses);

        Ok(recommendations)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn add_complexity_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        analyses: &ParallelAnalysisResults,
    ) {
        if let Some(complexity) = &analyses.complexity_report {
            for violation in &complexity.violations {
                if let Some(recommendation) = self.create_complexity_recommendation(violation) {
                    recommendations.push(recommendation);
                }
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn create_complexity_recommendation(
        &self,
        violation: &crate::services::complexity::Violation,
    ) -> Option<PrioritizedRecommendation> {
        match violation {
            crate::services::complexity::Violation::Error {
                function,
                value,
                threshold,
                message,
                ..
            }
            | crate::services::complexity::Violation::Warning {
                function,
                value,
                threshold,
                message,
                ..
            } => {
                function
                    .as_ref()
                    .map(|func_name| PrioritizedRecommendation {
                        title: format!("Refactor high-complexity function: {func_name}"),
                        description: format!(
                            "{message} (complexity: {value}, threshold: {threshold})"
                        ),
                        priority: self.determine_complexity_priority(*value),
                        estimated_effort: Duration::from_secs(3600), // 1 hour estimate
                        impact: Impact::High,
                        prerequisites: vec![],
                    })
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn determine_complexity_priority(&self, value: u16) -> Priority {
        if value > 25 {
            Priority::Critical
        } else if value > 20 {
            Priority::High
        } else {
            Priority::Medium
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn add_defect_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        defect_summary: &DefectSummary,
    ) {
        if defect_summary.total_defects > 50 {
            recommendations.push(PrioritizedRecommendation {
                title: "High defect count detected".to_string(),
                description: format!(
                    "Project has {} total defects. Consider a focused quality improvement sprint.",
                    defect_summary.total_defects
                ),
                priority: Priority::High,
                estimated_effort: Duration::from_secs(7200), // 2 hours
                impact: Impact::High,
                prerequisites: vec![],
            });
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn add_satd_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        analyses: &ParallelAnalysisResults,
    ) {
        if let Some(satd) = &analyses.satd_results {
            if satd.summary.total_items > 0 {
                recommendations.push(PrioritizedRecommendation {
                    title: "Technical debt detected".to_string(),
                    description: format!(
                        "Found {} SATD comments. Zero-tolerance policy requires immediate remediation.",
                        satd.summary.total_items
                    ),
                    priority: Priority::Critical,
                    estimated_effort: Duration::from_secs(satd.summary.total_items as u64 * 1800), // 30 min per SATD
                    impact: Impact::High,
                    prerequisites: vec![],
                });
            }
        }
    }

    /// Analyze project metadata (Makefile and README)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn analyze_project_metadata(
        &self,
        project_path: &Path,
    ) -> anyhow::Result<(
        Option<crate::models::project_meta::BuildInfo>,
        Option<crate::models::project_meta::ProjectOverview>,
    )> {
        use crate::services::{
            makefile_compressor::MakefileCompressor, project_meta_detector::ProjectMetaDetector,
            readme_compressor::ReadmeCompressor,
        };

        let detector = ProjectMetaDetector::new();
        let meta_files = detector.detect(project_path).await;

        let mut build_info = None;
        let mut project_overview = None;

        for meta_file in meta_files {
            match meta_file.file_type {
                crate::models::project_meta::MetaFileType::Makefile => {
                    let compressor = MakefileCompressor::new();
                    let compressed = compressor.compress(&meta_file.content);
                    build_info = Some(crate::models::project_meta::BuildInfo::from_makefile(
                        compressed,
                    ));
                    debug!("Makefile compressed and analyzed");
                }
                crate::models::project_meta::MetaFileType::Readme => {
                    let compressor = ReadmeCompressor::new();
                    let compressed = compressor.compress(&meta_file.content);
                    project_overview = Some(compressed.to_summary());
                    debug!("README compressed and analyzed");
                }
            }
        }

        Ok((build_info, project_overview))
    }

    /// Run QA verification on the deep context analysis results
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn run_qa_verification(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<QAVerificationResult> {
        // Convert DeepContext to the format expected by quality_gates
        let result = self.create_qa_compatible_result(context)?;

        // Create QA verification instance and generate report
        let qa_verification = QAVerification::new();
        let verification_report = qa_verification.generate_verification_report(&result);

        debug!(
            "QA verification report generated: overall status = {:?}",
            verification_report.overall
        );

        Ok(verification_report)
    }

    /// Convert complexity report to QA format
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn convert_complexity_report_to_qa(
        &self,
        report: &ComplexityReport,
    ) -> ComplexityMetricsForQA {
        ComplexityMetricsForQA {
            files: report
                .files
                .iter()
                .map(|f| FileComplexityMetricsForQA {
                    path: std::path::PathBuf::from(&f.path),
                    functions: f
                        .functions
                        .iter()
                        .map(|func| FunctionComplexityForQA {
                            name: func.name.clone(),
                            cyclomatic: u32::from(func.metrics.cyclomatic),
                            cognitive: u32::from(func.metrics.cognitive),
                            nesting_depth: u32::from(func.metrics.nesting_max),
                            start_line: func.line_start as usize,
                            end_line: func.line_end as usize,
                        })
                        .collect(),
                    total_cyclomatic: u32::from(f.total_complexity.cyclomatic),
                    total_cognitive: u32::from(f.total_complexity.cognitive),
                    total_lines: f.total_complexity.lines as usize,
                })
                .collect(),
            summary: ComplexitySummaryForQA {
                total_files: report.files.len(),
                total_functions: report.files.par_iter().map(|f| f.functions.len()).sum(),
            },
        }
    }

    /// Create fallback complexity metrics from file discovery
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn create_fallback_complexity_metrics(
        &self,
        context: &DeepContext,
    ) -> Option<ComplexityMetricsForQA> {
        let file_paths = self.collect_file_paths(&context.file_tree.root);
        let mut files_with_lines = Vec::new();
        let project_root = &context.metadata.project_root;

        debug!(
            "QA Fallback: Counting lines from {} files in {:?}",
            file_paths.len(),
            project_root
        );

        for path_str in &file_paths {
            if let Some(file_metrics) = self.process_file_for_fallback(path_str, project_root) {
                files_with_lines.push(file_metrics);
            }
        }

        if files_with_lines.is_empty() {
            None
        } else {
            Some(ComplexityMetricsForQA {
                files: files_with_lines,
                summary: ComplexitySummaryForQA {
                    total_files: 0,
                    total_functions: 0,
                },
            })
        }
    }

    /// Process single file for fallback metrics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn process_file_for_fallback(
        &self,
        path_str: &str,
        project_root: &std::path::Path,
    ) -> Option<FileComplexityMetricsForQA> {
        let full_path = if std::path::Path::new(path_str).is_absolute() {
            std::path::PathBuf::from(path_str)
        } else {
            project_root.join(path_str)
        };

        if full_path.exists() && full_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                let line_count = content.lines().count();

                if line_count > 0 {
                    return Some(FileComplexityMetricsForQA {
                        path: full_path,
                        functions: Vec::new(),
                        total_cyclomatic: 0,
                        total_cognitive: 0,
                        total_lines: line_count,
                    });
                }
            }
        }

        None
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn create_qa_compatible_result(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<DeepContextResult> {
        // Create complexity metrics from analysis results or fallback
        let complexity_metrics = if let Some(report) = context.analyses.complexity_report.as_ref() {
            Some(self.convert_complexity_report_to_qa(report))
        } else {
            self.create_fallback_complexity_metrics(context)
        };

        // Create dead code analysis from the results
        let dead_code_analysis = if let Some(ref dead_code) = context.analyses.dead_code_results {
            // Calculate total functions from complexity report if available
            let total_functions = context
                .analyses
                .complexity_report
                .as_ref()
                .map_or(0, |report| {
                    report
                        .files
                        .iter()
                        .map(|f| f.functions.len())
                        .sum::<usize>()
                });

            Some(DeadCodeAnalysis {
                summary: DeadCodeSummary {
                    total_functions,
                    dead_functions: dead_code.summary.dead_functions,
                    total_lines: dead_code
                        .ranked_files
                        .par_iter()
                        .map(|f| f.total_lines)
                        .sum(),
                    total_dead_lines: dead_code.summary.total_dead_lines,
                    dead_percentage: f64::from(dead_code.summary.dead_percentage),
                },
                dead_functions: vec![], // Not needed for QA verification
                warnings: vec![],
            })
        } else {
            None
        };

        // Create file paths list
        let file_paths = self.collect_file_paths(&context.file_tree.root);

        // Create AST summaries
        let ast_summaries = if context.analyses.ast_contexts.is_empty() {
            None
        } else {
            Some(
                context
                    .analyses
                    .ast_contexts
                    .iter()
                    .map(|ctx| AstSummary {
                        path: ctx.base.path.clone(),
                        language: ctx.base.language.clone(),
                        total_items: ctx.base.items.len(),
                        functions: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(item, crate::services::context::AstItem::Function { .. })
                            })
                            .count(),
                        classes: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(item, crate::services::context::AstItem::Struct { .. })
                            })
                            .count(),
                        imports: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(
                                    item,
                                    crate::services::context::AstItem::Use { .. }
                                        | crate::services::context::AstItem::Import { .. }
                                )
                            })
                            .count(),
                    })
                    .collect(),
            )
        };

        // Create language statistics
        let mut language_stats = FxHashMap::default();
        for ctx in &context.analyses.ast_contexts {
            *language_stats.entry(ctx.base.language.clone()).or_insert(0) += 1;
        }

        // Build the QA-compatible result
        Ok(DeepContextResult {
            metadata: context.metadata.clone(),
            file_tree: file_paths, // Vec<String> for quality_gates
            analyses: context.analyses.clone(),
            quality_scorecard: context.quality_scorecard.clone(),
            template_provenance: context.template_provenance.clone(),
            defect_summary: context.defect_summary.clone(),
            hotspots: context.hotspots.clone(),
            recommendations: context.recommendations.clone(),
            qa_verification: context.qa_verification.clone(),

            // Additional fields expected by quality_gates
            complexity_metrics,
            dead_code_analysis,
            ast_summaries,
            churn_analysis: context.analyses.churn_analysis.clone(),
            language_stats: Some(language_stats),

            // Project metadata fields
            build_info: context.build_info.clone(),
            project_overview: context.project_overview.clone(),
        })
    }
}

#[cfg(test)]
mod scorecard_tests {
    //! `overall_health` averaged `[complexity_score, maintainability_index,
    //! modularity_score]`. The last two are permanently `None` (no Halstead
    //! volume is ever produced, and modularity has no implementation), so the
    //! headline number was identically `complexity_score` — and the
    //! `test_coverage` it had just read off disk was thrown away.
    use super::*;
    use crate::services::complexity::{
        ComplexityMetrics, ComplexityReport, ComplexitySummary, FileComplexityMetrics,
    };
    use crate::services::deep_context::{DeepContextConfig, DefectSummary};

    fn one_clean_file() -> ComplexityReport {
        ComplexityReport {
            summary: ComplexitySummary::default(),
            violations: vec![],
            hotspots: vec![],
            files: vec![FileComplexityMetrics {
                path: "src/lib.rs".to_string(),
                total_complexity: ComplexityMetrics::default(),
                functions: vec![],
                classes: vec![],
            }],
        }
    }

    #[tokio::test]
    async fn overall_health_averages_in_the_coverage_it_measured() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join("target/coverage")).expect("mkdir");
        // Two instrumented lines, one hit => 50% line coverage.
        std::fs::write(
            tmp.path().join("target/coverage/lcov.info"),
            "SF:src/lib.rs\nDA:1,1\nDA:2,0\nend_of_record\n",
        )
        .expect("write lcov");

        let analyzer = DeepContextAnalyzer::new(DeepContextConfig::default());
        let analyses = ParallelAnalysisResults {
            complexity_report: Some(one_clean_file()),
            ..Default::default()
        };

        let card = analyzer
            .calculate_quality_scorecard(&analyses, &DefectSummary::default(), tmp.path())
            .await
            .expect("scorecard");

        assert_eq!(card.complexity_score, Some(100.0));
        assert_eq!(card.test_coverage, Some(50.0));
        // mean(100, 50) — not 100, which is what dropping test_coverage gave.
        let health = card.overall_health.expect("two components were measured");
        assert!(
            (health - 75.0).abs() < 1e-9,
            "overall_health must average every measured component, got {health}"
        );
    }

    #[test]
    fn average_measured_reports_nothing_when_nothing_was_measured() {
        assert_eq!(average_measured(&[None, None, None, None]), None);
        assert_eq!(
            average_measured(&[Some(40.0), None, None, Some(60.0)]),
            Some(50.0)
        );
    }
}
