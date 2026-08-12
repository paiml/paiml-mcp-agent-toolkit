#![cfg_attr(coverage_nightly, coverage(off))]
//! Main handler functions for Big-O complexity analysis commands

use crate::cli::{BigOOutputFormat, Path};
use crate::services::big_o_analyzer::{BigOAnalysisConfig, BigOAnalyzer};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};

use super::filters::apply_report_filters;
use super::output::{format_analysis_output, write_analysis_output};

/// Warning printed when `--analyze-space` is passed.
///
/// Space complexity is computed and printed for every function regardless of
/// the flag — `analyze_space_complexity` reaches the config struct and no
/// analyzer or renderer reads it — so `--analyze-space` changes nothing. It was
/// accepted in silence, which read as if it had switched an extra analysis on.
/// Named as a const so the note is covered by a test rather than only by eye.
pub(super) const ANALYZE_SPACE_NOOP_NOTE: &str =
    "note: --analyze-space is a no-op — space complexity is always reported alongside time";

/// Handle Big-O complexity analysis command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_big_o(
    project_path: PathBuf,
    format: BigOOutputFormat,
    confidence_threshold: u8,
    analyze_space: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    high_complexity_only: bool,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    // A nonexistent path previously produced an empty report and exit 0.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    let start_time = std::time::Instant::now();

    if analyze_space {
        eprintln!("{ANALYZE_SPACE_NOOP_NOTE}");
    }

    print_analysis_header(&project_path, confidence_threshold);

    let config = build_analysis_config(
        project_path,
        include,
        exclude,
        confidence_threshold,
        analyze_space,
    );

    if perf {
        debug!("Analysis configuration: {:?}", config);
    }

    let analyzer = BigOAnalyzer::new();
    let mut report = analyzer.analyze(config).await?;

    apply_report_filters(&mut report, high_complexity_only, top_files, perf);

    let output_content = format_analysis_output(&analyzer, &report, format)?;
    write_analysis_output(&output_content, output).await?;

    print_analysis_summary(&report, start_time.elapsed(), perf);

    Ok(())
}

/// Print analysis header information
pub(super) fn print_analysis_header(project_path: &Path, confidence_threshold: u8) {
    info!("🔍 Starting Big-O complexity analysis");
    info!("📂 Project path: {}", project_path.display());
    info!("🎯 Confidence threshold: {}%", confidence_threshold);
}

/// Build analysis configuration
pub(super) fn build_analysis_config(
    project_path: PathBuf,
    include: Vec<String>,
    exclude: Vec<String>,
    confidence_threshold: u8,
    analyze_space: bool,
) -> BigOAnalysisConfig {
    BigOAnalysisConfig {
        project_path,
        include_patterns: include,
        exclude_patterns: exclude,
        confidence_threshold,
        analyze_space_complexity: analyze_space,
    }
}

/// Print analysis summary
pub(super) fn print_analysis_summary(
    report: &crate::services::big_o_analyzer::BigOAnalysisReport,
    elapsed: std::time::Duration,
    perf: bool,
) {
    info!("✅ Big-O analysis completed in {:?}", elapsed);
    info!("📊 Analyzed {} functions", report.analyzed_functions);

    if !report.high_complexity_functions.is_empty() {
        info!(
            "⚠️ Found {} functions with high complexity",
            report.high_complexity_functions.len()
        );
    }

    if perf {
        // This used to be `info!`, which the default `warn`-level EnvFilter
        // discards: the only user-visible effect of `--perf` on `analyze big-o`
        // required the user to also pass `-v`. A performance readout routed
        // through a log sink the default filter drops is not "Show performance
        // metrics". The wall-clock line comes from the analyze router; this is
        // the one measurement only big-o can compute.
        let functions_per_sec = report.analyzed_functions as f64 / elapsed.as_secs_f64();
        crate::cli::handlers::analysis_handlers::perf_report::emit_detail(
            "analyze big-o",
            "throughput",
            &format!("{functions_per_sec:.0} functions/second"),
        );
    }
}

#[cfg(test)]
mod analyze_space_noop_tests {
    use super::*;

    /// The note is the only thing that tells a user `--analyze-space` does
    /// nothing; nothing covered it, so it could be deleted without a failure.
    #[test]
    fn the_noop_note_says_the_flag_is_a_no_op() {
        assert!(ANALYZE_SPACE_NOOP_NOTE.contains("--analyze-space"));
        assert!(ANALYZE_SPACE_NOOP_NOTE.contains("no-op"));
        assert!(ANALYZE_SPACE_NOOP_NOTE.contains("always reported"));
    }

    /// The `--analyze-space` help text must not promise an extra analysis the
    /// flag does not enable. It read "Analyze space complexity in addition to
    /// time" while changing nothing at all.
    #[test]
    fn help_text_does_not_promise_an_extra_analysis() {
        use clap::Subcommand;
        let help = crate::cli::commands::on_big_stack(|| {
            let cmd = crate::cli::commands::AnalyzeCommands::augment_subcommands(
                clap::Command::new("analyze"),
            );
            let help = cmd
                .get_subcommands()
                .find(|s| s.get_name() == "big-o")
                .expect("big-o subcommand must exist")
                .get_arguments()
                .find(|a| a.get_id() == "analyze_space")
                .expect("--analyze-space must exist")
                .get_help()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            help
        });
        assert!(
            help.contains("NO-OP"),
            "help must state the flag is inert, got: {help}"
        );
        assert!(
            !help.contains("in addition to time"),
            "help must not promise an analysis the flag does not enable, got: {help}"
        );
    }
}
