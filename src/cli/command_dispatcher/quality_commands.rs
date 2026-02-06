//! Quality Gate and Report Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains quality gate and report command execution.

#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::handlers;
use crate::cli::OutputFormat;
use std::path::PathBuf;

impl CommandDispatcher {
    /// Execute quality gate command (extracted for complexity reduction)
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_quality_gate_command(
        project_path: Option<PathBuf>,
        file: Option<PathBuf>,
        format: OutputFormat,
        fail_on_violation: bool,
        checks: Vec<String>,
        max_dead_code: Option<f64>,
        min_entropy: Option<f64>,
        max_complexity_p99: Option<usize>,
        include_provability: bool,
        output: Option<PathBuf>,
        perf: bool,
    ) -> anyhow::Result<()> {
        use crate::cli::enums::{QualityCheckType, QualityGateOutputFormat};

        // Convert OutputFormat to QualityGateOutputFormat
        let qg_format = match format {
            OutputFormat::Json => QualityGateOutputFormat::Json,
            OutputFormat::Table => QualityGateOutputFormat::Summary,
            OutputFormat::Yaml => QualityGateOutputFormat::Summary,
        };

        // Convert check strings to QualityCheckType
        let quality_checks: Vec<QualityCheckType> = checks
            .iter()
            .filter_map(|s| match s.as_str() {
                "dead_code" | "dead-code" => Some(QualityCheckType::DeadCode),
                "complexity" => Some(QualityCheckType::Complexity),
                "coverage" => Some(QualityCheckType::Coverage),
                "sections" => Some(QualityCheckType::Sections),
                "provability" => Some(QualityCheckType::Provability),
                "satd" => Some(QualityCheckType::Satd),
                "entropy" => Some(QualityCheckType::Entropy),
                "security" => Some(QualityCheckType::Security),
                "duplicates" => Some(QualityCheckType::Duplicates),
                "all" => Some(QualityCheckType::All),
                _ => None,
            })
            .collect();

        // Use defaults for optional parameters
        let max_dead = max_dead_code.unwrap_or(0.1); // 10% default
        let min_ent = min_entropy.unwrap_or(0.7); // 70% default
        let max_comp = max_complexity_p99.unwrap_or(20) as u32;

        crate::cli::analysis_utilities::handle_quality_gate(
            project_path.unwrap_or_else(|| PathBuf::from(".")),
            file,
            qg_format,
            fail_on_violation,
            quality_checks,
            max_dead,
            min_ent,
            max_comp,
            include_provability,
            output,
            perf,
        )
        .await
    }

    /// Execute report command (extracted for complexity reduction)
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_report_command(
        project_path: Option<PathBuf>,
        output_format: OutputFormat,
        include_visualizations: bool,
        include_executive_summary: bool,
        include_recommendations: bool,
        analyses: Vec<String>,
        confidence_threshold: Option<f64>,
        output: Option<PathBuf>,
        perf: bool,
        text: bool,
        markdown: bool,
        csv: bool,
    ) -> anyhow::Result<()> {
        use crate::cli::enums::{AnalysisType, ReportOutputFormat};

        // Convert OutputFormat to ReportOutputFormat
        let report_format = match output_format {
            OutputFormat::Json => ReportOutputFormat::Json,
            OutputFormat::Table => ReportOutputFormat::Text,
            OutputFormat::Yaml => ReportOutputFormat::Text,
        };

        // Convert analysis strings to AnalysisType
        let analysis_types: Vec<AnalysisType> = analyses
            .iter()
            .filter_map(|s| match s.as_str() {
                "complexity" => Some(AnalysisType::Complexity),
                "dead_code" | "dead-code" => Some(AnalysisType::DeadCode),
                "duplication" => Some(AnalysisType::Duplication),
                "technical_debt" | "technical-debt" => Some(AnalysisType::TechnicalDebt),
                "big_o" | "big-o" => Some(AnalysisType::BigO),
                "all" => Some(AnalysisType::All),
                _ => None,
            })
            .collect();

        // Convert confidence threshold to u8 (percentage)
        let confidence = (confidence_threshold.unwrap_or(0.8) * 100.0) as u8;

        handlers::enhanced_reporting_handlers::handle_generate_report(
            project_path.unwrap_or_else(|| PathBuf::from(".")),
            report_format,
            text,
            markdown,
            csv,
            include_visualizations,
            include_executive_summary,
            include_recommendations,
            analysis_types,
            confidence,
            output,
            perf,
        )
        .await
    }
}
