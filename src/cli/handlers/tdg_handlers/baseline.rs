#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG baseline management: create, compare, list, update baselines
//!
//! Sprint 66 Phase 1: Baseline CRUD operations for quality tracking.

use super::display::{display_baseline_table, display_grade_distribution};
use super::is_analyzable_file;
use crate::tdg::TdgAnalyzer;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Handle TDG baseline subcommand (Sprint 66 Phase 1)
pub(super) async fn handle_baseline_command(
    command: crate::cli::commands::BaselineCommand,
    _analyzer: &TdgAnalyzer,
    _config: &super::TdgCommandConfig,
) -> Result<()> {
    use crate::cli::commands::BaselineCommand;

    match command {
        BaselineCommand::Create {
            path,
            output,
            with_git_context,
            name: _name,
        } => create_baseline(_analyzer, &path, &output, with_git_context).await,

        BaselineCommand::Compare {
            baseline,
            path,
            format,
            fail_on_regression,
        } => compare_baseline(_analyzer, &baseline, &path, format, fail_on_regression).await,

        BaselineCommand::List { path, format } => list_baselines(&path, format).await,

        BaselineCommand::Update {
            baseline,
            path,
            with_git_context,
        } => update_baseline(_analyzer, &baseline, &path, with_git_context).await,
    }
}

/// Extract git context for baseline creation
fn extract_git_context(
    path: &Path,
    with_git_context: bool,
) -> Option<crate::models::git_context::GitContext> {
    if !with_git_context {
        return None;
    }
    match crate::models::git_context::GitContext::try_from_current_dir(path) {
        Some(ctx) => {
            println!("   📍 Git: {} on {}", ctx.commit_sha_short, ctx.branch);
            Some(ctx)
        }
        None => {
            println!("   ⚠️  Warning: Not in a git repository, git context unavailable");
            None
        }
    }
}

/// Create a new TDG baseline for the project (Sprint 66 Phase 1)
pub(super) async fn create_baseline(
    analyzer: &TdgAnalyzer,
    path: &Path,
    output: &Path,
    with_git_context: bool,
) -> Result<()> {
    use crate::tdg::TdgBaseline;

    println!("🔨 Creating TDG baseline...");
    println!("   Path: {}", path.display());
    println!("   Output: {}", output.display());
    println!(
        "   Git context: {}",
        if with_git_context { "yes" } else { "no" }
    );

    let git_context = extract_git_context(path, with_git_context);
    let mut baseline = TdgBaseline::new(git_context);
    let (files_analyzed, files_skipped) =
        analyze_baseline_files(analyzer, path, &mut baseline).await?;

    println!();
    println!("\n✅ Analysis complete:");
    println!("   Files analyzed: {}", files_analyzed);
    println!("   Files skipped: {}", files_skipped);
    println!("   Average score: {:.1}", baseline.summary.avg_score);

    display_grade_distribution(&baseline);

    baseline.save(output)?;
    println!("\n💾 Baseline saved to: {}", output.display());

    Ok(())
}

/// Analyze files and populate the baseline
async fn analyze_baseline_files(
    analyzer: &TdgAnalyzer,
    path: &Path,
    baseline: &mut crate::tdg::TdgBaseline,
) -> Result<(usize, usize)> {
    use crate::tdg::BaselineEntry;
    use std::fs;
    use walkdir::WalkDir;

    let mut files_analyzed = 0;
    let mut files_skipped = 0;

    println!("\n📊 Analyzing files...");

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Never filter out the root entry (depth 0) — fixes `--path .`
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
                && !matches!(name.as_ref(), "target" | "node_modules" | "dist" | "build")
        })
    {
        let entry = entry?;
        if !entry.file_type().is_file() || !is_analyzable_file(entry.path()) {
            continue;
        }

        let file_path = entry.path();
        match analyzer.analyze_file(file_path).await {
            Ok(score) => {
                let content = fs::read(file_path)?;
                let entry = BaselineEntry {
                    content_hash: blake3::hash(&content),
                    score: score.clone(),
                    components: crate::tdg::storage::ComponentScores::default(),
                    git_context: None,
                };
                baseline.add_entry(file_path.to_path_buf(), entry);
                files_analyzed += 1;
                if files_analyzed % 10 == 0 {
                    print!(".");
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
            }
            Err(e) => {
                files_skipped += 1;
                if files_skipped <= 5 {
                    println!("   ⚠️  Skipped {}: {}", file_path.display(), e);
                }
            }
        }
    }

    Ok((files_analyzed, files_skipped))
}

/// Compare current state against a baseline (Sprint 66 Phase 1)
pub(super) async fn compare_baseline(
    analyzer: &TdgAnalyzer,
    baseline_path: &Path,
    current_path: &Path,
    format: crate::cli::TdgOutputFormat,
    fail_on_regression: bool,
) -> Result<()> {
    use crate::tdg::TdgBaseline;

    println!("📊 Comparing against baseline...");
    println!("   Baseline: {}", baseline_path.display());
    println!("   Current path: {}", current_path.display());

    // Load baseline
    let old_baseline = TdgBaseline::load(baseline_path)?;
    println!(
        "   📝 Loaded baseline: {} files, avg score {:.1}",
        old_baseline.summary.total_files, old_baseline.summary.avg_score
    );

    // Create new baseline for current state
    println!("\n🔍 Analyzing current state...");
    let temp_output = std::env::temp_dir().join("pmat-current-baseline.json");
    create_baseline(analyzer, current_path, &temp_output, false).await?;
    let new_baseline = TdgBaseline::load(&temp_output)?;

    // Clean up ephemeral baseline file
    std::fs::remove_file(&temp_output).ok();

    // Compare
    println!("\n📈 Computing comparison...");
    let comparison = old_baseline.compare(&new_baseline);

    // Format output
    let output_str = match format {
        crate::cli::TdgOutputFormat::Table | crate::cli::TdgOutputFormat::Markdown => {
            comparison.format_text()
        }
        crate::cli::TdgOutputFormat::Json => serde_json::to_string_pretty(&comparison)?,
        crate::cli::TdgOutputFormat::Sarif => {
            // SARIF not implemented yet, use text
            comparison.format_text()
        }
    };

    println!("\n{}", output_str);

    // Check for regressions
    if fail_on_regression && comparison.has_regressions() {
        return Err(anyhow::anyhow!(
            "Quality regression detected: {} file(s) regressed",
            comparison.regressed.len()
        ));
    }

    Ok(())
}

/// Find all baseline files in a directory
fn find_baseline_files(path: &Path) -> Vec<(PathBuf, crate::tdg::TdgBaseline)> {
    use crate::tdg::TdgBaseline;
    use walkdir::WalkDir;

    WalkDir::new(path)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?;
            if name.ends_with("-baseline.json") || name == ".pmat-baseline.json" {
                TdgBaseline::load(entry.path())
                    .ok()
                    .map(|b| (entry.path().to_path_buf(), b))
            } else {
                None
            }
        })
        .collect()
}

/// List all baselines in a directory (Sprint 66 Phase 1)
async fn list_baselines(path: &Path, format: crate::cli::TdgOutputFormat) -> Result<()> {
    println!("📋 Listing baselines in: {}", path.display());

    let baselines = find_baseline_files(path);

    if baselines.is_empty() {
        println!("   No baselines found");
        return Ok(());
    }

    println!("\n📊 Found {} baseline(s):\n", baselines.len());

    match format {
        crate::cli::TdgOutputFormat::Table | crate::cli::TdgOutputFormat::Markdown => {
            for (path, baseline) in &baselines {
                display_baseline_table(path, baseline);
            }
        }
        crate::cli::TdgOutputFormat::Json => {
            let output: Vec<_> = baselines
                .iter()
                .map(|(path, baseline)| {
                    serde_json::json!({
                        "path": path.display().to_string(),
                        "version": baseline.version,
                        "created_at": baseline.created_at,
                        "total_files": baseline.summary.total_files,
                        "avg_score": baseline.summary.avg_score,
                        "git_context": baseline.git_context
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        crate::cli::TdgOutputFormat::Sarif => {
            for (path, baseline) in &baselines {
                println!("📝 {}", path.display());
                println!(
                    "   Files: {} | Avg: {:.1}",
                    baseline.summary.total_files, baseline.summary.avg_score
                );
            }
        }
    }

    Ok(())
}

/// Update an existing baseline (Sprint 66 Phase 1)
async fn update_baseline(
    analyzer: &TdgAnalyzer,
    baseline_path: &Path,
    project_path: &Path,
    with_git_context: bool,
) -> Result<()> {
    println!("🔄 Updating baseline...");
    println!("   Baseline: {}", baseline_path.display());
    println!("   Path: {}", project_path.display());

    // Simply re-create the baseline (overwrites the file)
    create_baseline(analyzer, project_path, baseline_path, with_git_context).await?;

    println!("\n✅ Baseline updated successfully");

    Ok(())
}
