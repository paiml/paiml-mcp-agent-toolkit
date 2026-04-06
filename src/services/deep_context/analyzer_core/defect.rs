#![cfg_attr(coverage_nightly, coverage(off))]

use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::debug;

use crate::models::tdg::{TDGScore, TDGSeverity, TDGSummary};
use crate::services::deep_context::analyzer_core::types::ParallelAnalysisResults;
use crate::services::deep_context::DeepContextAnalyzer;
use crate::services::deep_context::{
    DefectFactor, DefectHotspot, DefectSummary, FileLocation, Impact, Priority,
    RefactoringEstimate, TechnicalDebtCategory, TechnicalDebtSeverity,
};
use crate::services::tdg_calculator::TDGCalculator;

impl DeepContextAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn correlate_defects(
        &self,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<(DefectSummary, Vec<DefectHotspot>)> {
        // Step 1: Collect file TDG scores from all analyses
        let file_tdg_scores = self.collect_file_tdg_scores(analyses)?;

        // Step 2: Calculate TDG summary for the project
        let _tdg_calculator = TDGCalculator::new();
        let tdg_summary = self.calculate_tdg_summary(&file_tdg_scores)?;

        // Step 3: Build defect summary (now based on TDG)
        let defect_summary = self.build_tdg_defect_summary(&tdg_summary, analyses)?;

        // Step 4: Generate hotspots
        let hotspots = self.generate_tdg_hotspots(&file_tdg_scores)?;

        Ok((defect_summary, hotspots))
    }

    /// Collect file TDG scores from all available analyses
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub(crate) fn collect_file_tdg_scores(
        &self,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<FxHashMap<String, TDGScore>> {
        let mut file_tdg_scores = FxHashMap::default();

        if let Some(ref ast_contexts) = analyses.ast_contexts {
            for enhanced_context in ast_contexts {
                let file_path = enhanced_context.base.path.clone();

                // Extract actual churn score for this file
                let churn_score = if let Some(ref churn_analysis) = analyses.churn_analysis {
                    churn_analysis
                        .files
                        .iter()
                        .find(|f| {
                            f.path.to_string_lossy() == file_path
                                || f.relative_path == file_path
                                || file_path.ends_with(&f.relative_path)
                        })
                        .map_or(0.0, |f| f.churn_score)
                } else {
                    0.0
                };

                // Use TDG calculator to compute score for this file
                let tdg_score = TDGScore {
                    value: 1.5, // Default value - could be computed from components
                    components: crate::models::tdg::TDGComponents {
                        complexity: 1.0,
                        churn: f64::from(churn_score),
                        coupling: 0.5,
                        domain_risk: 0.5,
                        duplication: 0.5,
                        dead_code: 0.0, // CB-128: 6th TDG dimension
                    },
                    severity: TDGSeverity::Normal,
                    percentile: 50.0,
                    confidence: 0.8,
                };

                file_tdg_scores.insert(file_path, tdg_score);
            }
        }

        Ok(file_tdg_scores)
    }

    /// Calculate TDG summary from individual file scores
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn calculate_tdg_summary(
        &self,
        file_scores: &FxHashMap<String, TDGScore>,
    ) -> anyhow::Result<TDGSummary> {
        let total_files = file_scores.len();
        // Use parallel processing for score analysis
        let (values, severities): (Vec<_>, Vec<_>) = file_scores
            .par_iter()
            .map(|(_, score)| (score.value, &score.severity))
            .unzip();

        let mut tdg_values = values;

        // Count severities in parallel
        let critical_files = severities
            .par_iter()
            .filter(|s| matches!(s, TDGSeverity::Critical))
            .count();
        let warning_files = severities
            .par_iter()
            .filter(|s| matches!(s, TDGSeverity::Warning))
            .count();

        tdg_values.sort_unstable_by(|a, b| a.total_cmp(b));

        let average_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
        };

        let p95_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            let index = ((tdg_values.len() - 1) as f64 * 0.95) as usize;
            tdg_values[index.min(tdg_values.len() - 1)]
        };

        let p99_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            let index = ((tdg_values.len() - 1) as f64 * 0.99) as usize;
            tdg_values[index.min(tdg_values.len() - 1)]
        };

        // Create hotspots from top TDG scores
        let mut hotspots: Vec<_> = file_scores
            .iter()
            .map(|(path, score)| crate::models::tdg::TDGHotspot {
                path: path.clone(),
                tdg_score: score.value,
                primary_factor: "complexity".to_string(), // Default factor
                estimated_hours: score.value * 2.0,       // Simple estimation
            })
            .collect();
        hotspots.sort_unstable_by(|a, b| {
            b.tdg_score
                .partial_cmp(&a.tdg_score)
                .expect("internal error")
        });
        hotspots.truncate(10);

        Ok(TDGSummary {
            total_files,
            critical_files,
            warning_files,
            average_tdg,
            p95_tdg,
            p99_tdg,
            estimated_debt_hours: average_tdg * total_files as f64 * 2.0,
            hotspots,
        })
    }

    /// Build defect summary based on actual defect enumeration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn build_tdg_defect_summary(
        &self,
        tdg_summary: &TDGSummary,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<DefectSummary> {
        let mut total_defects = 0usize;
        let mut by_severity = FxHashMap::default();
        let mut by_type = FxHashMap::default();
        let mut total_loc = 0usize;

        // Process each analysis type
        self.process_complexity_violations(
            analyses,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
            &mut total_loc,
        );
        self.process_satd_violations(analyses, &mut total_defects, &mut by_severity, &mut by_type);
        self.process_dead_code_violations(
            analyses,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
        );
        self.process_tdg_violations(
            tdg_summary,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
        );

        let defect_density = self.calculate_defect_density(total_defects, total_loc);

        debug!(
            "Calculated defect summary: {} total defects, {} LOC, density = {:.2}",
            total_defects, total_loc, defect_density
        );

        Ok(DefectSummary {
            total_defects,
            by_severity,
            by_type,
            defect_density,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn process_complexity_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
        total_loc: &mut usize,
    ) {
        if let Some(ref complexity_report) = analyses.complexity_report {
            let complexity_violations = complexity_report.violations.len();
            *total_defects += complexity_violations;
            by_type.insert("Complexity".to_string(), complexity_violations);

            for violation in &complexity_report.violations {
                let severity = match violation {
                    crate::services::complexity::Violation::Error { .. } => "Critical",
                    crate::services::complexity::Violation::Warning { .. } => "Warning",
                };
                *by_severity.entry(severity.to_string()).or_insert(0) += 1;
            }

            for file in &complexity_report.files {
                *total_loc += file.total_complexity.lines as usize;
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn process_satd_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        if let Some(ref satd_results) = analyses.satd_results {
            let satd_count = satd_results.items.len();
            *total_defects += satd_count;
            by_type.insert("TechnicalDebt".to_string(), satd_count);

            for item in &satd_results.items {
                let severity = match item.severity {
                    crate::services::satd_detector::Severity::Critical => "Critical",
                    crate::services::satd_detector::Severity::High => "Critical",
                    crate::services::satd_detector::Severity::Medium => "Warning",
                    crate::services::satd_detector::Severity::Low => "Normal",
                };
                *by_severity.entry(severity.to_string()).or_insert(0) += 1;
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn process_dead_code_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        if let Some(ref dead_code_results) = analyses.dead_code_results {
            let dead_code_count = dead_code_results.summary.dead_functions
                + dead_code_results.summary.dead_classes
                + dead_code_results.summary.dead_modules;
            *total_defects += dead_code_count;
            by_type.insert("DeadCode".to_string(), dead_code_count);
            *by_severity.entry("Warning".to_string()).or_insert(0) += dead_code_count;
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn process_tdg_violations(
        &self,
        tdg_summary: &TDGSummary,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        let high_tdg_count = tdg_summary.critical_files + tdg_summary.warning_files;
        *total_defects += high_tdg_count;
        by_type.insert("TDG".to_string(), high_tdg_count);
        *by_severity.entry("Critical".to_string()).or_insert(0) += tdg_summary.critical_files;
        *by_severity.entry("Warning".to_string()).or_insert(0) += tdg_summary.warning_files;
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn calculate_defect_density(&self, total_defects: usize, total_loc: usize) -> f64 {
        if total_loc > 0 {
            (total_defects as f64 * 1000.0) / total_loc as f64
        } else {
            0.0
        }
    }

    /// Generate hotspots from TDG scores
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn generate_tdg_hotspots(
        &self,
        file_scores: &FxHashMap<String, TDGScore>,
    ) -> anyhow::Result<Vec<DefectHotspot>> {
        let mut hotspots: Vec<_> = file_scores
            .par_iter()
            .filter(|(_, score)| score.value > 1.5) // Filter above threshold
            .map(|(path, score)| DefectHotspot {
                location: FileLocation {
                    file: std::path::PathBuf::from(path),
                    line: 1,
                    column: 1,
                },
                composite_score: score.value as f32,
                contributing_factors: vec![DefectFactor::TechnicalDebt {
                    category: TechnicalDebtCategory::Implementation,
                    severity: TechnicalDebtSeverity::High,
                    age_days: 0,
                }],
                refactoring_effort: RefactoringEstimate {
                    estimated_hours: score.value as f32 * 2.0,
                    priority: Priority::High,
                    impact: Impact::Medium,
                    suggested_actions: vec!["Reduce TDG score".to_string()],
                },
            })
            .collect();

        hotspots.sort_unstable_by(|a, b| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .expect("internal error")
        });
        hotspots.truncate(20);

        Ok(hotspots)
    }
}
