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
#[cfg(feature = "org-intelligence")]
use chrono::{Duration, Utc};
// NOTE: indicatif removed - using local progress types
#[cfg(feature = "org-intelligence")]
use crate::services::progress::{ProgressBar, ProgressStyle};
#[cfg(feature = "org-intelligence")]
use organizational_intelligence_plugin::analyzer::OrgAnalyzer;
#[cfg(feature = "org-intelligence")]
use organizational_intelligence_plugin::github::GitHubMiner;
#[cfg(feature = "org-intelligence")]
use organizational_intelligence_plugin::report::{
    AnalysisMetadata, AnalysisReport, ReportGenerator,
};
#[cfg(feature = "org-intelligence")]
use organizational_intelligence_plugin::summarizer::{ReportSummarizer, SummaryConfig};
#[cfg(feature = "org-intelligence")]
use std::env;
#[cfg(feature = "org-intelligence")]
use std::path::{Path, PathBuf};
#[cfg(feature = "org-intelligence")]
use tempfile::TempDir;
#[cfg(feature = "org-intelligence")]
use tracing::{info, warn};

/// Handle organizational intelligence commands
#[cfg(feature = "org-intelligence")]
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
    summarize: bool,
    strip_pii: bool,
    top_n: usize,
    min_frequency: usize,
) -> Result<()> {
    println!(
        "\n{}",
        c::header(&format!("Analyzing GitHub Organization: {}", org))
    );
    println!("   {} {:?}", c::label("Output:"), output);

    // Initialize GitHub client
    let github_token = env::var("GITHUB_TOKEN").ok();
    if github_token.is_none() {
        println!(
            "{}",
            c::warn("GITHUB_TOKEN not set - using unauthenticated requests (lower rate limits)")
        );
        println!("   Set GITHUB_TOKEN environment variable for higher rate limits");
    }

    let miner = GitHubMiner::new(github_token);

    // Fetch organization repositories
    info!("Fetching repositories for organization: {}", org);
    let all_repos = miner
        .fetch_organization_repos(org)
        .await
        .context("Failed to fetch organization repositories")?;

    info!("✅ Successfully fetched {} repositories", all_repos.len());

    // Filter repos updated in last 2 years
    let two_years_ago = Utc::now() - Duration::days(730);
    let repos = GitHubMiner::filter_by_date(all_repos.clone(), two_years_ago);

    println!("\n{}", c::subheader("Organization Statistics:"));
    println!(
        "   {} {}",
        c::label("Total repositories:"),
        c::number(&all_repos.len().to_string())
    );
    println!(
        "   {} {}",
        c::label("Active (last 2 years):"),
        c::number(&repos.len().to_string())
    );

    // Display top 5 repositories by stars
    let mut sorted_repos = repos.clone();
    sorted_repos.sort_by(|a, b| b.stars.cmp(&a.stars));

    println!("\n{}", c::subheader("Top Repositories:"));
    for (i, repo) in sorted_repos.iter().take(5).enumerate() {
        println!(
            "   {}. {} ({}) - {}",
            c::number(&(i + 1).to_string()),
            c::label(&repo.name),
            c::number(&format!("{} stars", repo.stars)),
            c::dim(repo.language.as_deref().unwrap_or("Unknown"))
        );
    }

    // Analyze repositories
    println!(
        "\n{} Analyzing defect patterns in {} repositories...",
        c::label(">>"),
        c::number(&sorted_repos.len().to_string())
    );

    let temp_dir = TempDir::new()?;
    let analyzer = OrgAnalyzer::new(temp_dir.path());

    let mut all_patterns = vec![];
    let mut total_commits = 0;
    let mut repos_analyzed = 0;

    // Create progress bar
    let pb = ProgressBar::new(sorted_repos.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Failed to set progress bar template")
            .progress_chars("#>-"),
    );

    for (i, repo) in sorted_repos.iter().enumerate() {
        pb.set_message(format!("Analyzing: {}", repo.name));

        let repo_url = format!("https://github.com/{}/{}", org, repo.name);

        match analyzer
            .analyze_repository(&repo_url, &repo.name, 100)
            .await
        {
            Ok(patterns) => {
                total_commits += 100;
                let pattern_count = patterns.len();
                all_patterns.extend(patterns);
                repos_analyzed += 1;
                pb.println(format!(
                    "   ✅ [{}/{}] {} - {} patterns found",
                    i + 1,
                    sorted_repos.len(),
                    repo.name,
                    pattern_count
                ));
                info!("✅ Analyzed {}", repo.name);
            }
            Err(e) => {
                warn!("Failed to analyze {}: {}", repo.name, e);
                pb.println(format!(
                    "   ⚠️  [{}/{}] {} - SKIPPED: {}",
                    i + 1,
                    sorted_repos.len(),
                    repo.name,
                    e
                ));
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message("Analysis complete!");
    println!();

    // Generate YAML report
    info!("Generating YAML report");
    let report_generator = ReportGenerator::new();

    let metadata = AnalysisMetadata {
        organization: org.to_string(),
        analysis_date: Utc::now().to_rfc3339(),
        repositories_analyzed: repos_analyzed,
        commits_analyzed: total_commits,
        analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let report = AnalysisReport {
        version: "1.0".to_string(),
        metadata,
        defect_patterns: all_patterns,
    };

    // Write report to file
    report_generator.write_to_file(&report, output).await?;

    println!("\n{}", c::subheader("Analysis Report:"));
    println!(
        "   {} {}",
        c::label("Repositories:"),
        c::number(&repos_analyzed.to_string())
    );
    println!(
        "   {} {}",
        c::label("Commits:"),
        c::number(&total_commits.to_string())
    );
    println!("   {} {:?}", c::label("Output:"), output);

    // Phase 2: Optionally summarize results
    if summarize {
        let summary_path = output.with_extension("summary.yaml");

        println!("\n{}", c::subheader("Generating Summary..."));
        println!("   {} {}", c::label("Strip PII:"), strip_pii);
        println!(
            "   {} {}",
            c::label("Top N categories:"),
            c::number(&top_n.to_string())
        );
        println!(
            "   {} {}",
            c::label("Min frequency:"),
            c::number(&min_frequency.to_string())
        );

        let config = SummaryConfig {
            strip_pii,
            top_n_categories: top_n,
            min_frequency,
            include_examples: false, // No examples for PMAT prompts
        };

        let summary =
            ReportSummarizer::summarize(output, config).context("Failed to generate summary")?;

        ReportSummarizer::save_to_file(&summary, &summary_path)?;

        println!("\n{}", c::pass("Summary Complete:"));
        println!(
            "   {} {}",
            c::label("Defect patterns:"),
            c::number(
                &summary
                    .organizational_insights
                    .top_defect_categories
                    .len()
                    .to_string()
            )
        );
        println!("   {} {:?}", c::label("Output:"), summary_path);
        println!(
            "\n{} Use with: pmat prompt generate --task \"<task>\" --context \"<context>\" --summary {:?}",
            c::dim("Tip:"),
            summary_path
        );
    } else {
        println!(
            "\n{} To generate summary: pmat org analyze --org {} --output {:?} --summarize --strip-pii",
            c::dim("Tip:"),
            org,
            output
        );
    }

    Ok(())
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
    debug_assert!(
        passed_coverage.exists(),
        "passed_coverage must exist: {}",
        passed_coverage.display()
    );
    debug_assert!(
        failed_coverage.exists(),
        "failed_coverage must exist: {}",
        failed_coverage.display()
    );
    debug_assert!(_repo.exists(), "_repo must exist: {}", _repo.display());
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
