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
    /// Used by tests; production path calls analysis_utilities::handle_quality_gate directly (#230).
    #[allow(clippy::too_many_arguments, dead_code)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
            _ => QualityGateOutputFormat::Summary,
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
        let max_comp = max_complexity_p99.unwrap_or(20) as u32;

        crate::cli::analysis_utilities::handle_quality_gate(
            project_path.unwrap_or_else(|| PathBuf::from(".")),
            file,
            qg_format,
            fail_on_violation,
            quality_checks,
            max_dead,
            min_entropy,
            max_comp,
            include_provability,
            output,
            perf,
        )
        .await
    }

    /// Execute report command (extracted for complexity reduction)
    ///
    /// Issue #672: this used to take the generic `OutputFormat` and re-derive a
    /// `ReportOutputFormat` from it with `Json => Json, _ => Text`. Combined
    /// with the caller's `ReportOutputFormat -> OutputFormat` narrowing, every
    /// declared format other than `json` was silently rewritten to `text`, so
    /// `pmat report --format csv -o out.csv` wrote a plain-text report ("CODE
    /// QUALITY REPORT …") into a .csv file. The declared format is now carried
    /// end to end and `handle_generate_report` decides what is renderable.
    #[allow(clippy::too_many_arguments)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn execute_report_command(
        project_path: Option<PathBuf>,
        report_format: crate::cli::enums::ReportOutputFormat,
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
        use crate::cli::enums::AnalysisType;

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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod report_format_passthrough_tests {
    use super::*;
    use crate::cli::enums::ReportOutputFormat;

    /// Issue #672 type-level pin. `execute_report_command` used to accept the
    /// generic `OutputFormat`, which cannot represent `csv`/`markdown`, so the
    /// dispatcher collapsed every non-json `--format` to `text` and
    /// `pmat report --format csv -o out.csv` wrote a plain-text report. If the
    /// parameter is ever narrowed back to a type that loses format identity,
    /// this stops compiling.
    #[test]
    fn test_execute_report_command_carries_the_declared_report_format() {
        #[allow(clippy::type_complexity)]
        fn accepts_declared_format<F, Fut>(_f: F)
        where
            F: Fn(
                Option<PathBuf>,
                ReportOutputFormat,
                bool,
                bool,
                bool,
                Vec<String>,
                Option<f64>,
                Option<PathBuf>,
                bool,
                bool,
                bool,
                bool,
            ) -> Fut,
        {
        }
        accepts_declared_format(CommandDispatcher::execute_report_command);
    }

    /// Every `ReportOutputFormat` variant must be representable end to end —
    /// i.e. no variant may be indistinguishable from another after routing.
    /// `OutputFormat` had no csv/markdown variants, which is exactly how the
    /// identity was lost.
    #[test]
    fn test_all_report_format_variants_are_distinguishable() {
        let variants = [
            ReportOutputFormat::Json,
            ReportOutputFormat::Csv,
            ReportOutputFormat::Markdown,
            ReportOutputFormat::Text,
            ReportOutputFormat::Html,
            ReportOutputFormat::Pdf,
            ReportOutputFormat::Dashboard,
        ];
        let mut seen = std::collections::BTreeSet::new();
        for v in &variants {
            assert!(seen.insert(v.to_string()), "duplicate rendering for {v}");
        }
        assert_eq!(seen.len(), variants.len());
    }
}
