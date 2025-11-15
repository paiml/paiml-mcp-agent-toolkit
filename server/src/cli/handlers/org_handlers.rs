//! Organizational intelligence command handlers
//!
//! This module integrates OIP (Organizational Intelligence Plugin) directly into PMAT
//! via shared library dependency, providing seamless organizational analysis capabilities.

use crate::cli::commands::OrgCommands;
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use organizational_intelligence_plugin::analyzer::OrgAnalyzer;
use organizational_intelligence_plugin::github::GitHubMiner;
use organizational_intelligence_plugin::report::{AnalysisMetadata, AnalysisReport, ReportGenerator};
use organizational_intelligence_plugin::summarizer::{ReportSummarizer, SummaryConfig};
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;
use tracing::{info, warn};

/// Handle organizational intelligence commands
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
    }
}

/// Handle organizational analysis
async fn handle_org_analyze(
    org: &str,
    output: &PathBuf,
    _max_concurrent: usize,
    summarize: bool,
    strip_pii: bool,
    top_n: usize,
    min_frequency: usize,
) -> Result<()> {
    println!("\n🔍 Analyzing GitHub Organization: {}", org);
    println!("   Output: {:?}", output);

    // Initialize GitHub client
    let github_token = env::var("GITHUB_TOKEN").ok();
    if github_token.is_none() {
        println!("⚠️  GITHUB_TOKEN not set - using unauthenticated requests (lower rate limits)");
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

    println!("\n📊 Organization Statistics:");
    println!("   Total repositories: {}", all_repos.len());
    println!("   Active (last 2 years): {}", repos.len());

    // Display top 5 repositories by stars
    let mut sorted_repos = repos.clone();
    sorted_repos.sort_by(|a, b| b.stars.cmp(&a.stars));

    println!("\n⭐ Top Repositories:");
    for (i, repo) in sorted_repos.iter().take(5).enumerate() {
        println!(
            "   {}. {} ({} ⭐) - {}",
            i + 1,
            repo.name,
            repo.stars,
            repo.language.as_deref().unwrap_or("Unknown")
        );
    }

    // Analyze repositories
    println!("\n🔍 Analyzing defect patterns in {} repositories...", sorted_repos.len());

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

        match analyzer.analyze_repository(&repo_url, &repo.name, 100).await {
            Ok(patterns) => {
                total_commits += 100;
                let pattern_count = patterns.len();
                all_patterns.extend(patterns);
                repos_analyzed += 1;
                pb.println(format!("   ✅ [{}/{}] {} - {} patterns found",
                    i + 1, sorted_repos.len(), repo.name, pattern_count));
                info!("✅ Analyzed {}", repo.name);
            }
            Err(e) => {
                warn!("Failed to analyze {}: {}", repo.name, e);
                pb.println(format!("   ⚠️  [{}/{}] {} - SKIPPED: {}",
                    i + 1, sorted_repos.len(), repo.name, e));
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

    println!("\n📄 Analysis Report:");
    println!("   Repositories: {}", repos_analyzed);
    println!("   Commits: {}", total_commits);
    println!("   Output: {:?}", output);

    // Phase 2: Optionally summarize results
    if summarize {
        let summary_path = output.with_extension("summary.yaml");

        println!("\n📊 Generating Summary...");
        println!("   Strip PII: {}", strip_pii);
        println!("   Top N categories: {}", top_n);
        println!("   Min frequency: {}", min_frequency);

        let config = SummaryConfig {
            strip_pii,
            top_n_categories: top_n,
            min_frequency,
            include_examples: false, // No examples for PMAT prompts
        };

        let summary = ReportSummarizer::summarize(output, config)
            .context("Failed to generate summary")?;

        ReportSummarizer::save_to_file(&summary, &summary_path)?;

        println!("\n✅ Summary Complete:");
        println!("   Defect patterns: {}", summary.organizational_insights.top_defect_categories.len());
        println!("   Output: {:?}", summary_path);
        println!(
            "\n💡 Use with: pmat prompt generate --task \"<task>\" --context \"<context>\" --summary {:?}",
            summary_path
        );
    } else {
        println!("\n💡 To generate summary: pmat org analyze --org {} --output {:?} --summarize --strip-pii", org, output);
    }

    Ok(())
}
