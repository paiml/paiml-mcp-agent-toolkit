//! Organizational intelligence command handlers
//!
//! This module integrates OIP (Organizational Intelligence Plugin) directly into PMAT
//! via shared library dependency, providing seamless organizational analysis capabilities.
//!
//! **Feature Flag**: This module requires the `org-intelligence` feature to be enabled.

#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(feature = "org-intelligence")]
use crate::cli::colors as c;
#[cfg(feature = "org-intelligence")]
use crate::cli::commands::OrgCommands;
#[cfg(feature = "org-intelligence")]
use anyhow::{Context, Result};
// NOTE: The organizational-intelligence-plugin (aprender-orchestrate) crate was
// repurposed in 0.41: the OIP API consumed by `handle_org_analyze`
// (analyzer::OrgAnalyzer, github::GitHubMiner, report::{AnalysisMetadata,
// AnalysisReport, ReportGenerator}, summarizer::{ReportSummarizer, SummaryConfig})
// was removed upstream with no replacement. The `org analyze` path is therefore
// no longer available; `org localize` (fault localization) is PMAT-native and
// unaffected.
#[cfg(feature = "org-intelligence")]
use std::path::{Path, PathBuf};
#[cfg(feature = "org-intelligence")]
use tracing::info;

/// Handle organizational intelligence commands
#[cfg(feature = "org-intelligence")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_org_command(org_cmd: OrgCommands) -> Result<()> {
    match org_cmd {
        OrgCommands::Analyze {
            org,
            output,
            max_concurrent,
            summarize,
            strip_pii,
            top_n,
            min_frequency,
        } => {
            handle_org_analyze(
                &org,
                &output,
                max_concurrent,
                summarize,
                strip_pii,
                top_n,
                min_frequency,
            )
            .await
        }
        OrgCommands::Localize {
            passed_coverage,
            failed_coverage,
            passed_count,
            failed_count,
            formula,
            top_n,
            output,
            ensemble,
            calibrated,
            confidence_threshold,
            enrich_tdg,
            repo,
        } => {
            handle_fault_localization(
                &passed_coverage,
                &failed_coverage,
                passed_count,
                failed_count,
                &formula,
                top_n,
                output.as_deref(),
                ensemble,
                calibrated,
                confidence_threshold,
                enrich_tdg,
                &repo,
            )
            .await
        }
    }
}

/// Handle organizational analysis
#[cfg(feature = "org-intelligence")]
async fn handle_org_analyze(
    org: &str,
    output: &PathBuf,
    _max_concurrent: usize,
    _summarize: bool,
    _strip_pii: bool,
    _top_n: usize,
    _min_frequency: usize,
) -> Result<()> {
    // The organizational-intelligence-plugin (aprender-orchestrate) crate was
    // repurposed in 0.41 and the organizational-analysis API that powered this
    // command (GitHub repo mining via GitHubMiner, OrgAnalyzer defect-pattern
    // analysis, and the AnalysisReport/ReportSummarizer report pipeline) was
    // removed upstream with no replacement. There is no longer any backing
    // implementation for `pmat org analyze`, so surface a clear, actionable
    // error instead of silently doing nothing.
    println!(
        "\n{}",
        c::header(&format!("Analyzing GitHub Organization: {}", org))
    );
    println!("   {} {:?}", c::label("Output:"), output);

    info!(
        "org analyze requested for {} but the upstream OIP analysis API was removed in aprender-orchestrate 0.41",
        org
    );

    anyhow::bail!(
        "`pmat org analyze` is no longer available: the upstream \
organizational-intelligence-plugin (aprender-orchestrate) crate removed its \
organizational-analysis API (GitHub mining, defect-pattern analysis, and report \
generation) in 0.41 with no replacement. Use `pmat org localize` for native \
Tarantula fault localization, which is unaffected."
    )
}

/// Handle fault localization using native Tarantula implementation (Issue #103)
#[cfg(feature = "org-intelligence")]
#[allow(clippy::too_many_arguments)]
async fn handle_fault_localization(
    passed_coverage: &Path,
    failed_coverage: &Path,
    passed_count: usize,
    failed_count: usize,
    formula: &str,
    top_n: usize,
    output: Option<&Path>,
    _ensemble: bool,
    _calibrated: bool,
    _confidence_threshold: f32,
    _enrich_tdg: bool,
    _repo: &Path,
) -> Result<()> {
    use crate::services::fault_localization::{
        FaultLocalizer, LcovParser, ReportFormat, SbflFormula,
    };

    println!(
        "\n{}",
        c::header("Tarantula Fault Localization (native implementation)")
    );
    println!("   {} {}", c::label("Formula:"), formula);
    println!(
        "   {} {}",
        c::label("Passed tests:"),
        c::number(&passed_count.to_string())
    );
    println!(
        "   {} {}",
        c::label("Failed tests:"),
        c::number(&failed_count.to_string())
    );
    println!(
        "   {} {}",
        c::label("Top-N:"),
        c::number(&top_n.to_string())
    );
    println!();

    // Parse LCOV files
    let passed_data = LcovParser::parse_file(passed_coverage)
        .context("Failed to parse passed coverage LCOV file")?;
    let failed_data = LcovParser::parse_file(failed_coverage)
        .context("Failed to parse failed coverage LCOV file")?;

    // Parse formula
    let sbfl_formula: SbflFormula = formula.parse().unwrap_or(SbflFormula::Tarantula);

    // Run localization
    let result = FaultLocalizer::run_localization(
        &passed_data,
        &failed_data,
        passed_count,
        failed_count,
        sbfl_formula,
        top_n,
    );

    // Determine output format
    let format = if output
        .map(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .unwrap_or(false)
    {
        ReportFormat::Json
    } else if output
        .map(|p| p.extension().and_then(|e| e.to_str()) == Some("yaml"))
        .unwrap_or(false)
    {
        ReportFormat::Yaml
    } else {
        ReportFormat::Terminal
    };

    // Generate report
    let report = FaultLocalizer::generate_report(&result, format)?;

    // Output
    if let Some(out_path) = output {
        std::fs::write(out_path, &report).context("Failed to write output file")?;
        println!("{}", c::pass(&format!("Report written to: {:?}", out_path)));
    } else {
        println!("{}", report);
    }

    Ok(())
}

#[cfg(all(test, feature = "org-intelligence"))]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_org_commands_enum_structure() {
        // Verify OrgCommands enum can be constructed
        let cmd = OrgCommands::Analyze {
            org: "testorg".to_string(),
            output: PathBuf::from("/tmp/test.yaml"),
            max_concurrent: 5,
            summarize: false,
            strip_pii: false,
            top_n: 10,
            min_frequency: 3,
        };

        match cmd {
            OrgCommands::Analyze { org, .. } => {
                assert_eq!(org, "testorg");
            }
        }
    }

    #[tokio::test]
    async fn test_handle_org_command_basic_structure() {
        // Test that handle_org_command function exists and has correct signature
        // This is a smoke test to ensure the function compiles
        let temp_file = NamedTempFile::new().unwrap();
        let cmd = OrgCommands::Analyze {
            org: "nonexistent-test-org-12345".to_string(),
            output: temp_file.path().to_path_buf(),
            max_concurrent: 1,
            summarize: false,
            strip_pii: false,
            top_n: 10,
            min_frequency: 3,
        };

        // This will fail (org doesn't exist) but proves the function signature is correct
        let result = handle_org_command(cmd).await;
        assert!(result.is_err(), "Expected error for nonexistent org");
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_org_handler_module_compiles() {
        // Smoke test to ensure module compiles with feature flag
        assert!(true);
    }
}
