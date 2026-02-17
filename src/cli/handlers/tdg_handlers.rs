#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::commands::TdgCommand;
use crate::cli::TdgOutputFormat;
use crate::tdg::{Grade, TdgAnalyzer, TdgConfig};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Discover git working directory from a path (shell fallback when git-lib disabled)
fn discover_git_workdir(path: &Path) -> Option<PathBuf> {
    #[cfg(feature = "git-lib")]
    {
        git2::Repository::discover(path)
            .ok()
            .and_then(|repo| repo.workdir().map(Path::to_path_buf))
    }
    #[cfg(not(feature = "git-lib"))]
    {
        Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(path)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
    }
}

/// Configuration for TDG command handling
pub struct TdgCommandConfig {
    pub path: PathBuf,
    pub command: Option<TdgCommand>,
    pub format: TdgOutputFormat,
    pub config: Option<PathBuf>,
    pub quiet: bool,
    pub include_components: bool,
    pub min_grade: Option<String>,
    pub output: Option<PathBuf>,
    /// Sprint 65: Include git context (commit SHA, branch, author)
    pub with_git_context: bool,
    /// Issue #78: Enable detailed explanation mode with function-level breakdown
    pub explain: bool,
    /// Issue #78: Complexity threshold for filtering functions in --explain mode
    pub threshold: u32,
    /// Issue #78: Baseline git ref for progress tracking in --explain mode
    pub baseline: Option<String>,
    /// Terminal graph visualization of dependencies
    pub viz: bool,
    /// Visualization theme
    pub viz_theme: String,
}

/// Check if path should be skipped (test/bench files)
fn should_skip_path(config: &TdgCommandConfig) -> bool {
    if !config.path.is_file() {
        return false;
    }
    let path_str = config.path.to_string_lossy();
    path_str.contains("/tests/") || path_str.contains("/benches/")
}

/// Setup git context for analyzer if enabled
fn setup_git_context(analyzer: &mut TdgAnalyzer, config: &TdgCommandConfig) {
    if !config.with_git_context {
        return;
    }
    let search_path = if config.path.is_file() {
        config.path.parent().unwrap_or(&config.path)
    } else {
        &config.path
    };
    let git_context = discover_git_workdir(search_path)
        .and_then(|workdir| crate::models::git_context::GitContext::try_from_current_dir(&workdir));
    analyzer.set_git_context(git_context);
}

/// Handle TDG command execution
pub async fn handle_tdg_command(config: TdgCommandConfig) -> Result<()> {
    if should_skip_path(&config) {
        if !config.quiet {
            println!("Skipping test file: {}", config.path.display());
        }
        return Ok(());
    }

    let tdg_config = load_tdg_configuration(&config)?;
    let mut analyzer = TdgAnalyzer::with_storage(tdg_config)?;
    setup_git_context(&mut analyzer, &config);

    if let Some(ref cmd) = config.command {
        return handle_tdg_subcommand(cmd.clone(), &analyzer, &config).await;
    }

    if config.explain {
        return handle_explain_mode(&analyzer, &config).await;
    }

    #[cfg(feature = "viz")]
    if config.viz {
        return handle_viz_mode(&analyzer, &config).await;
    }

    let score = execute_tdg_analysis(&analyzer, &config).await?;
    validate_minimum_grade(&score, &config)?;

    let git_context = analyzer.get_git_context();
    let output_str = format_tdg_output(&score, git_context, &config)?;
    write_tdg_output(&output_str, &config)?;

    Ok(())
}

/// Load TDG configuration from file or use default (cognitive complexity ≤3)
fn load_tdg_configuration(config: &TdgCommandConfig) -> Result<TdgConfig> {
    if let Some(config_path) = &config.config {
        let config_content = fs::read_to_string(config_path)?;
        Ok(toml::from_str(&config_content)?)
    } else {
        Ok(TdgConfig::default())
    }
}

/// Handle TDG subcommands (cognitive complexity ≤8)
async fn handle_tdg_subcommand(
    cmd: TdgCommand,
    analyzer: &TdgAnalyzer,
    config: &TdgCommandConfig,
) -> Result<()> {
    match cmd {
        TdgCommand::Compare { source1, source2 } => {
            handle_compare_command(analyzer, &source1, &source2, config).await
        }
        TdgCommand::History {
            commit,
            since,
            range,
            path,
            format,
        } => handle_history_command(analyzer, commit, since, range, path, format, config).await,
        TdgCommand::Baseline { command } => {
            handle_baseline_command(command, analyzer, config).await
        }
        TdgCommand::CheckRegression {
            baseline,
            path,
            format,
            fail_on_regression,
            max_score_drop,
            allow_grade_drop,
        } => {
            handle_check_regression(
                analyzer,
                baseline.as_path(),
                path.as_path(),
                format.clone(),
                fail_on_regression,
                max_score_drop,
                allow_grade_drop,
            )
            .await
        }
        TdgCommand::CheckQuality {
            path,
            min_grade,
            format,
            fail_on_violation,
            new_files_only,
            baseline,
        } => {
            handle_check_quality(
                analyzer,
                path.as_path(),
                min_grade.as_deref(),
                format.clone(),
                fail_on_violation,
                new_files_only,
                baseline.as_ref(),
            )
            .await
        }
        TdgCommand::Diagnostics { .. }
        | TdgCommand::Storage { .. }
        | TdgCommand::Dashboard { .. }
        | TdgCommand::Config(_) => {
            super::tdg_diagnostic_handler::handle_tdg_diagnostics(&cmd, &config.path).await
        }
    }
}

/// Handle TDG compare subcommand (cognitive complexity ≤4)
async fn handle_compare_command(
    analyzer: &TdgAnalyzer,
    source1: &Path,
    source2: &Path,
    config: &TdgCommandConfig,
) -> Result<()> {
    let comparison = analyzer.compare(source1, source2).await?;
    let output_str = format_comparison(comparison, config.format.clone())?;

    if let Some(output_path) = &config.output {
        fs::write(output_path, output_str)?;
    } else {
        println!("{output_str}");
    }

    Ok(())
}

/// Query TDG history records based on command flags
async fn query_history_records(
    storage: &crate::tdg::TieredStore,
    commit: Option<String>,
    since: Option<String>,
    range: Option<String>,
    repo_path: &Path,
) -> Result<Vec<crate::tdg::FullTdgRecord>> {
    if let Some(commit_ref) = commit {
        let found: Vec<crate::tdg::FullTdgRecord> = storage.get_by_commit(&commit_ref).await?;
        if found.is_empty() {
            return Err(anyhow!(
                "No TDG data found for commit '{}'. Ensure TDG was run with --with-git-context.",
                commit_ref
            ));
        }
        return Ok(found);
    }
    let all_records = storage.get_all_with_git_context().await?;
    if let Some(since_ref) = since {
        return filter_by_git_since(&since_ref, all_records, repo_path);
    }
    if let Some(range_ref) = range {
        return filter_by_git_range(&range_ref, all_records, repo_path);
    }
    Ok(all_records)
}

/// Handle TDG history subcommand (Sprint 65 Phase 3)
async fn handle_history_command(
    analyzer: &TdgAnalyzer,
    commit: Option<String>,
    since: Option<String>,
    range: Option<String>,
    path_filter: Option<PathBuf>,
    format: TdgOutputFormat,
    config: &TdgCommandConfig,
) -> Result<()> {
    let storage = analyzer
        .storage()
        .ok_or_else(|| anyhow!("TDG storage not initialized. Run with --with-git-context flag."))?;

    let mut records = query_history_records(storage, commit, since, range, &config.path).await?;

    if let Some(target_path) = path_filter {
        records.retain(|r| r.identity.path == target_path);
    }

    if records.is_empty() {
        println!("No TDG history found matching criteria.");
        return Ok(());
    }

    let output_str = format_history_output(&records, format)?;
    match &config.output {
        Some(output_path) => fs::write(output_path, output_str)?,
        None => println!("{output_str}"),
    }

    Ok(())
}

/// Filter records by git "since" reference
fn filter_by_git_since(
    since_ref: &str,
    mut records: Vec<crate::tdg::storage::FullTdgRecord>,
    repo_path: &Path,
) -> Result<Vec<crate::tdg::storage::FullTdgRecord>> {
    // Get timestamp of the "since" commit using shell git
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ct", since_ref])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to resolve git ref: {since_ref}"));
    }

    let since_time: i64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|_| anyhow!("Invalid timestamp from git log"))?;

    // Filter records to commits after since_time
    records.retain(|r| {
        if let Some(git_ctx) = &r.git_context {
            let record_time = git_ctx.commit_timestamp.timestamp();
            record_time > since_time
        } else {
            false
        }
    });

    Ok(records)
}

/// Filter records by git commit range
fn filter_by_git_range(
    range_ref: &str,
    mut records: Vec<crate::tdg::storage::FullTdgRecord>,
    repo_path: &Path,
) -> Result<Vec<crate::tdg::storage::FullTdgRecord>> {
    // Parse range (e.g., "HEAD~10..HEAD" or "v2.177.0..v2.178.0")
    let parts: Vec<&str> = range_ref.split("..").collect();
    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid range format. Expected 'start..end' (e.g., HEAD~10..HEAD)"
        ));
    }

    // Get timestamps using shell git
    let get_timestamp = |git_ref: &str| -> Result<i64> {
        let output = Command::new("git")
            .args(["log", "-1", "--format=%ct", git_ref])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to resolve git ref: {git_ref}"));
        }

        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|_| anyhow!("Invalid timestamp from git log"))
    };

    let start_time = get_timestamp(parts[0])?;
    let end_time = get_timestamp(parts[1])?;

    // Filter records within time range
    records.retain(|r| {
        if let Some(git_ctx) = &r.git_context {
            let record_time = git_ctx.commit_timestamp.timestamp();
            record_time >= start_time && record_time <= end_time
        } else {
            false
        }
    });

    Ok(records)
}

/// Handle TDG baseline subcommand (Sprint 66 Phase 1)
async fn handle_baseline_command(
    command: crate::cli::commands::BaselineCommand,
    _analyzer: &TdgAnalyzer,
    _config: &TdgCommandConfig,
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

/// Display grade distribution histogram
fn display_grade_distribution(baseline: &crate::tdg::TdgBaseline) {
    let mut grade_counts: std::collections::HashMap<Grade, usize> =
        std::collections::HashMap::new();
    let mut f_grade_files: Vec<String> = Vec::new();

    for (path, entry) in &baseline.files {
        *grade_counts.entry(entry.score.grade).or_insert(0) += 1;
        if entry.score.grade == Grade::F {
            f_grade_files.push(format!(
                "     {} ({:.1})",
                path.display(),
                entry.score.total
            ));
        }
    }

    println!("\n📊 Grade Distribution:");
    let grade_order = [
        Grade::APLus,
        Grade::A,
        Grade::AMinus,
        Grade::BPlus,
        Grade::B,
        Grade::BMinus,
        Grade::CPlus,
        Grade::C,
        Grade::CMinus,
        Grade::D,
        Grade::F,
    ];
    for grade in grade_order {
        let count = grade_counts.get(&grade).unwrap_or(&0);
        if *count > 0 {
            let bar = "█".repeat((*count).min(30));
            println!("   {:>3}: {:>4} {}", grade, count, bar);
        }
    }

    display_f_grade_warning(&f_grade_files);
}

/// Display F-grade warning if any files have F grades
fn display_f_grade_warning(f_grade_files: &[String]) {
    if f_grade_files.is_empty() {
        return;
    }
    println!(
        "\n⚠️  F-Grade Warning: {} file(s) with F grade:",
        f_grade_files.len()
    );
    for file in f_grade_files.iter().take(10) {
        println!("{}", file);
    }
    if f_grade_files.len() > 10 {
        println!("     ... and {} more", f_grade_files.len() - 10);
    }
    println!("\n   F-grades cap project score at B. Fix these to improve project grade.");
}

/// Create a new TDG baseline for the project (Sprint 66 Phase 1)
async fn create_baseline(
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

/// Check if file is analyzable by extension
fn is_analyzable_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str(),
            Some(
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "jsx"
                    | "java"
                    | "c"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "go"
                    | "rb"
                    | "php"
                    | "swift"
                    | "kt"
                    | "kts"
            )
        )
    } else {
        false
    }
}

/// Compare current state against a baseline (Sprint 66 Phase 1)
async fn compare_baseline(
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
        return Err(anyhow!(
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

/// Display baseline in table format
fn display_baseline_table(path: &Path, baseline: &crate::tdg::TdgBaseline) {
    println!("📝 {}", path.display());
    println!("   Version: {}", baseline.version);
    println!(
        "   Created: {}",
        baseline.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!("   Files: {}", baseline.summary.total_files);
    println!("   Avg Score: {:.1}", baseline.summary.avg_score);
    if let Some(git_ctx) = &baseline.git_context {
        println!("   Git: {} on {}", git_ctx.commit_sha_short, git_ctx.branch);
    }
    println!();
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

/// Execute TDG analysis on file or directory (cognitive complexity ≤3)
async fn execute_tdg_analysis(
    analyzer: &TdgAnalyzer,
    config: &TdgCommandConfig,
) -> Result<crate::tdg::TdgScore> {
    if config.path.is_dir() {
        Ok(analyzer.analyze_project(&config.path).await?.average())
    } else {
        analyzer.analyze_file(&config.path).await
    }
}

/// Validate minimum grade requirement (cognitive complexity ≤4)
fn validate_minimum_grade(score: &crate::tdg::TdgScore, config: &TdgCommandConfig) -> Result<()> {
    if let Some(min_grade_str) = &config.min_grade {
        let min_grade = parse_grade(min_grade_str)?;
        if score.grade < min_grade {
            return Err(anyhow!(
                "Grade {} is below minimum required grade {}",
                format_grade(score.grade),
                format_grade(min_grade)
            ));
        }
    }
    Ok(())
}

/// Format TDG output based on config (cognitive complexity ≤3)
fn format_tdg_output(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    config: &TdgCommandConfig,
) -> Result<String> {
    if config.quiet {
        Ok(format!("{:.1}", score.total))
    } else {
        format_tdg_score(
            score.clone(),
            git_context,
            config.format.clone(),
            config.include_components,
        )
    }
}

/// Write TDG output to file or stdout (cognitive complexity ≤3)
fn write_tdg_output(output_str: &str, config: &TdgCommandConfig) -> Result<()> {
    if let Some(output_path) = &config.output {
        fs::write(output_path, output_str)?;
    } else {
        println!("{output_str}");
    }
    Ok(())
}

fn format_tdg_score(
    score: crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    format: TdgOutputFormat,
    include_components: bool,
) -> Result<String> {
    match format {
        TdgOutputFormat::Table => {
            let mut output = String::new();

            // Header
            output.push_str("╭─────────────────────────────────────────────────╮\n");
            if let Some(file_path) = &score.file_path {
                output.push_str(&format!(
                    "│  TDG Score Report: {}              │\n",
                    file_path.display()
                ));
            } else {
                output.push_str("│  TDG Score Report                              │\n");
            }
            output.push_str("├─────────────────────────────────────────────────┤\n");

            // Overall score
            output.push_str(&format!(
                "│  Overall Score: {:.1}/100 ({})                  │\n",
                score.total,
                format_grade(score.grade)
            ));
            output.push_str(&format!(
                "│  Language: {:?} (confidence: {:.0}%)             │\n",
                score.language,
                score.confidence * 100.0
            ));

            // Sprint 65: Git context (if available)
            if let Some(git) = git_context {
                output.push_str("│                                                 │\n");
                output.push_str("│  🔗 Git Context:                                │\n");
                output.push_str(&format!(
                    "│  ├─ Commit:  {}                     │\n",
                    &git.commit_sha_short
                ));
                output.push_str(&format!(
                    "│  ├─ Branch:  {}                               │\n",
                    &git.branch
                ));
                output.push_str(&format!(
                    "│  └─ Author:  {}                          │\n",
                    &git.author_name
                ));
            }

            if include_components {
                output.push_str("│                                                 │\n");
                output.push_str("│  📊 Breakdown:                                  │\n");
                output.push_str(&format!(
                    "│  ├─ Structural:     {:.1}/25                    │\n",
                    score.structural_complexity
                ));
                output.push_str(&format!(
                    "│  ├─ Semantic:       {:.1}/20                    │\n",
                    score.semantic_complexity
                ));
                output.push_str(&format!(
                    "│  ├─ Duplication:    {:.1}/20                    │\n",
                    score.duplication_ratio
                ));
                output.push_str(&format!(
                    "│  ├─ Coupling:       {:.1}/15                    │\n",
                    score.coupling_score
                ));
                output.push_str(&format!(
                    "│  ├─ Documentation:  {:.1}/10                    │\n",
                    score.doc_coverage
                ));
                output.push_str(&format!(
                    "│  └─ Consistency:    {:.1}/10                    │\n",
                    score.consistency_score
                ));
            }

            output.push_str("╰─────────────────────────────────────────────────╯\n");
            Ok(output)
        }
        TdgOutputFormat::Json => {
            let json_value = serde_json::json!({
                "file": score.file_path.map(|p| p.to_string_lossy().to_string()),
                "language": format!("{:?}", score.language),
                "confidence": score.confidence,
                "score": {
                    "total": score.total,
                    "grade": format_grade(score.grade),
                    "breakdown": if include_components {
                        Some(serde_json::json!({
                            "structural_complexity": score.structural_complexity,
                            "semantic_complexity": score.semantic_complexity,
                            "duplication": score.duplication_ratio,
                            "coupling": score.coupling_score,
                            "documentation": score.doc_coverage,
                            "consistency": score.consistency_score,
                        }))
                    } else {
                        None
                    }
                },
                "git_context": git_context.map(|git| serde_json::json!({
                    "commit_sha": git.commit_sha,
                    "commit_sha_short": git.commit_sha_short,
                    "branch": git.branch,
                    "author_name": git.author_name,
                    "author_email": git.author_email,
                    "commit_timestamp": git.commit_timestamp.to_rfc3339(),
                    "commit_message": git.commit_message,
                    "tags": git.tags,
                    "is_clean": git.is_clean,
                    "uncommitted_files": git.uncommitted_files,
                }))
            });
            Ok(serde_json::to_string_pretty(&json_value)?)
        }
        TdgOutputFormat::Markdown => {
            let mut output = String::new();

            output.push_str("# TDG Score Report\n\n");
            if let Some(file_path) = &score.file_path {
                output.push_str(&format!("**File**: `{}`\n\n", file_path.display()));
            }

            output.push_str(&format!(
                "**Overall Score**: {:.1}/100 ({})\n",
                score.total,
                format_grade(score.grade)
            ));
            output.push_str(&format!(
                "**Language**: {:?} (confidence: {:.0}%)\n\n",
                score.language,
                score.confidence * 100.0
            ));

            if include_components {
                output.push_str("## Component Breakdown\n\n");
                output.push_str("| Component | Score | Max |\n");
                output.push_str("|-----------|-------|-----|\n");
                output.push_str(&format!(
                    "| Structural Complexity | {:.1} | 25 |\n",
                    score.structural_complexity
                ));
                output.push_str(&format!(
                    "| Semantic Complexity | {:.1} | 20 |\n",
                    score.semantic_complexity
                ));
                output.push_str(&format!(
                    "| Duplication | {:.1} | 20 |\n",
                    score.duplication_ratio
                ));
                output.push_str(&format!(
                    "| Coupling | {:.1} | 15 |\n",
                    score.coupling_score
                ));
                output.push_str(&format!(
                    "| Documentation | {:.1} | 10 |\n",
                    score.doc_coverage
                ));
                output.push_str(&format!(
                    "| Consistency | {:.1} | 10 |\n",
                    score.consistency_score
                ));
            }

            Ok(output)
        }
        TdgOutputFormat::Sarif => {
            // For SARIF format, return simplified score
            Ok(format!("{:.1}", score.total))
        }
    }
}

fn format_comparison(
    comparison: crate::tdg::Comparison,
    format: TdgOutputFormat,
) -> Result<String> {
    if format == TdgOutputFormat::Table {
        let mut output = String::new();
        output.push_str("╭─────────────────────────────────────────────────╮\n");
        output.push_str("│  TDG Comparison                                 │\n");
        output.push_str("├─────────────────────────────────────────────────┤\n");
        output.push_str(&format!(
            "│  Source 1: {:.1} ({})                           │\n",
            comparison.source1.total,
            format_grade(comparison.source1.grade)
        ));
        output.push_str(&format!(
            "│  Source 2: {:.1} ({})                           │\n",
            comparison.source2.total,
            format_grade(comparison.source2.grade)
        ));
        output.push_str(&format!(
            "│  Difference: {:+.1}                             │\n",
            comparison.delta
        ));

        output.push_str(&format!(
            "│  Winner: {}                                      │\n",
            comparison.winner
        ));

        output.push_str("╰─────────────────────────────────────────────────╯\n");
        Ok(output)
    } else {
        // For other formats, output as JSON
        let json_value = serde_json::json!({
            "source1": {
                "total": comparison.source1.total,
                "grade": format_grade(comparison.source1.grade),
            },
            "source2": {
                "total": comparison.source2.total,
                "grade": format_grade(comparison.source2.grade),
            },
            "difference": comparison.delta,
            "winner": comparison.winner
        });
        Ok(serde_json::to_string_pretty(&json_value)?)
    }
}

/// Format TDG history output (Sprint 65 Phase 3)
fn format_history_output(
    records: &[crate::tdg::storage::FullTdgRecord],
    format: TdgOutputFormat,
) -> Result<String> {
    use chrono::{DateTime, Utc};

    if format == TdgOutputFormat::Table {
        let mut output = String::new();
        output.push_str(
            "╭──────────────────────────────────────────────────────────────────────────╮\n",
        );
        output.push_str(
            "│  TDG History                                                             │\n",
        );
        output.push_str(
            "├──────────────────────────────────────────────────────────────────────────┤\n",
        );

        for record in records {
            if let Some(git_ctx) = &record.git_context {
                let timestamp: DateTime<Utc> = git_ctx.commit_timestamp;
                let date_str = timestamp.format("%Y-%m-%d %H:%M").to_string();

                output.push_str(&format!(
                    "│  📝 {} - {} ({})                                            │\n",
                    git_ctx.commit_sha_short,
                    format_grade(record.score.grade),
                    record.score.total
                ));
                output.push_str(&format!(
                    "│  ├─ Branch:  {}                                                           │\n",
                    git_ctx.branch
                ));
                output.push_str(&format!(
                    "│  ├─ Author:  {}                                                      │\n",
                    git_ctx.author_name
                ));
                output.push_str(&format!(
                    "│  ├─ Date:    {}                                                  │\n",
                    date_str
                ));
                output.push_str(&format!(
                    "│  └─ File:    {}                                                          │\n",
                    record.identity.path.display()
                ));
                output.push_str("│                                                                          │\n");
            }
        }

        output.push_str(
            "╰──────────────────────────────────────────────────────────────────────────╯\n",
        );
        Ok(output)
    } else {
        // JSON format
        let json_records: Vec<_> = records
            .iter()
            .map(|r| {
                serde_json::json!({
                    "file_path": r.identity.path,
                    "score": {
                        "total": r.score.total,
                        "grade": format_grade(r.score.grade),
                        "structural_complexity": r.score.structural_complexity,
                        "semantic_complexity": r.score.semantic_complexity,
                        "duplication_ratio": r.score.duplication_ratio,
                        "coupling_score": r.score.coupling_score,
                        "doc_coverage": r.score.doc_coverage,
                        "consistency_score": r.score.consistency_score,
                        "entropy_score": r.score.entropy_score,
                    },
                    "git_context": r.git_context.as_ref().map(|git| serde_json::json!({
                        "commit_sha": git.commit_sha,
                        "commit_sha_short": git.commit_sha_short,
                        "branch": git.branch,
                        "author_name": git.author_name,
                        "author_email": git.author_email,
                        "commit_timestamp": git.commit_timestamp,
                        "commit_message": git.commit_message,
                        "tags": git.tags,
                    })),
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "history": json_records,
            "total_records": records.len()
        }))?)
    }
}

fn format_grade(grade: Grade) -> String {
    match grade {
        Grade::APLus => "A+",
        Grade::A => "A",
        Grade::AMinus => "A-",
        Grade::BPlus => "B+",
        Grade::B => "B",
        Grade::BMinus => "B-",
        Grade::CPlus => "C+",
        Grade::C => "C",
        Grade::CMinus => "C-",
        Grade::D => "D",
        Grade::F => "F",
    }
    .to_string()
}

/// Handle check-regression command (Sprint 66 Phase 2)
async fn handle_check_regression(
    analyzer: &TdgAnalyzer,
    baseline_path: &Path,
    current_path: &Path,
    format: crate::cli::TdgOutputFormat,
    fail_on_regression: bool,
    max_score_drop: Option<f32>,
    allow_grade_drop: bool,
) -> Result<()> {
    use crate::tdg::{GateConfig, QualityGate, RegressionGate, TdgBaseline};

    println!("🔍 Checking for quality regressions...");

    // Load baseline
    let baseline = TdgBaseline::load(baseline_path)?;
    println!(
        "   ✅ Loaded baseline: {} files",
        baseline.summary.total_files
    );

    // Create current baseline
    let temp_output = std::env::temp_dir().join("pmat-regression-check.json");
    create_baseline(analyzer, current_path, &temp_output, false).await?;
    let current = TdgBaseline::load(&temp_output)?;
    std::fs::remove_file(&temp_output).ok();

    // Configure gate
    let mut config = GateConfig::default();
    if let Some(drop) = max_score_drop {
        config.max_score_drop = drop;
    }
    config.allow_grade_drop = allow_grade_drop;

    // Run regression gate
    let gate = RegressionGate::new(config);
    let result = gate.check(&baseline, &current)?;

    // Display results
    match &format {
        crate::cli::TdgOutputFormat::Table => display_gate_result_table(&result),
        crate::cli::TdgOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        crate::cli::TdgOutputFormat::Sarif => {
            println!("SARIF format not yet implemented for quality gates");
        }
        crate::cli::TdgOutputFormat::Markdown => {
            println!("Markdown format not yet implemented for quality gates");
        }
    }

    // Exit with error if requested and gate failed
    if fail_on_regression && !result.passed {
        return Err(anyhow::anyhow!("Quality regression detected"));
    }

    Ok(())
}

/// Run the primary quality gate based on mode
fn run_primary_gate(
    new_files_only: bool,
    min_grade_str: Option<&str>,
    baseline_path: Option<&PathBuf>,
    current: &crate::tdg::TdgBaseline,
) -> Result<crate::tdg::GateResult> {
    use crate::tdg::{GateConfig, MinimumGradeGate, NewFileGate, QualityGate, TdgBaseline};

    if new_files_only {
        let baseline_path = baseline_path
            .ok_or_else(|| anyhow::anyhow!("Baseline required for --new-files-only mode"))?;
        let baseline = TdgBaseline::load(baseline_path)?;
        let mut config = GateConfig::default();
        if let Some(grade_str) = min_grade_str {
            config.new_file_min_grade = parse_grade(grade_str)?;
        }
        NewFileGate::new(config).check(&baseline, current)
    } else {
        let baseline = TdgBaseline::new(None);
        let mut config = GateConfig::default();
        if let Some(grade_str) = min_grade_str {
            config.default_min_grade = parse_grade(grade_str)?;
        }
        MinimumGradeGate::new(config).check(&baseline, current)
    }
}

/// Display gate result in the requested format
fn display_gate_result(
    result: &crate::tdg::GateResult,
    format: &crate::cli::TdgOutputFormat,
) -> Result<()> {
    match format {
        crate::cli::TdgOutputFormat::Table => display_gate_result_table(result),
        crate::cli::TdgOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        crate::cli::TdgOutputFormat::Sarif => {
            println!("SARIF format not yet implemented for quality gates");
        }
        crate::cli::TdgOutputFormat::Markdown => {
            println!("Markdown format not yet implemented for quality gates");
        }
    }
    Ok(())
}

/// Handle check-quality command (Sprint 66 Phase 2)
async fn handle_check_quality(
    analyzer: &TdgAnalyzer,
    path: &Path,
    min_grade_str: Option<&str>,
    format: crate::cli::TdgOutputFormat,
    fail_on_violation: bool,
    new_files_only: bool,
    baseline_path: Option<&PathBuf>,
) -> Result<()> {
    use crate::tdg::{FGradeGate, QualityGate, TdgBaseline};

    println!("🔍 Checking quality thresholds...");

    let temp_output = std::env::temp_dir().join("pmat-quality-check.json");
    create_baseline(analyzer, path, &temp_output, false).await?;
    let current = TdgBaseline::load(&temp_output)?;
    std::fs::remove_file(&temp_output).ok();

    let f_grade_result = FGradeGate::with_defaults().check(&TdgBaseline::new(None), &current)?;
    let result = run_primary_gate(new_files_only, min_grade_str, baseline_path, &current)?;

    if !f_grade_result.violations.is_empty() {
        println!("\n⚠️  F-Grade Warning: {}", f_grade_result.message);
        println!("   F-grades cap project score at B regardless of average.");
        display_gate_result(&f_grade_result, &format)?;
        println!();
    }

    display_gate_result(&result, &format)?;

    if fail_on_violation && (!result.passed || !f_grade_result.passed) {
        return Err(anyhow::anyhow!("Quality violations detected"));
    }

    Ok(())
}

/// Display gate result in table format
fn display_gate_result_table(result: &crate::tdg::GateResult) {
    println!("\n{}", result.message);

    if !result.violations.is_empty() {
        println!("\n📋 Violations:");
        println!("┌────────────────────────────────┬──────────────┬──────────┬────────────────────────────────┐");
        println!("│ File                           │ Type         │ Severity │ Message                        │");
        println!("├────────────────────────────────┼──────────────┼──────────┼────────────────────────────────┤");

        for violation in &result.violations {
            let path = format!("{}", violation.path.display());
            let vtype = format!("{:?}", violation.violation_type);
            let sev = format!("{:?}", violation.severity);
            println!(
                "│ {:<30} │ {:<12} │ {:<8} │ {:<30} │",
                path.get(..path.len().min(30)).unwrap_or(&path),
                vtype.get(..vtype.len().min(12)).unwrap_or(&vtype),
                sev.get(..sev.len().min(8)).unwrap_or(&sev),
                violation
                    .message
                    .get(..violation.message.len().min(30))
                    .unwrap_or(&violation.message)
            );
        }
        println!("└────────────────────────────────┴──────────────┴──────────┴────────────────────────────────┘");
    }
}

/// Parse grade string to Grade enum
fn parse_grade(s: &str) -> Result<crate::tdg::Grade> {
    use crate::tdg::Grade;
    match s.to_uppercase().as_str() {
        "A+" => Ok(Grade::APLus),
        "A" => Ok(Grade::A),
        "A-" => Ok(Grade::AMinus),
        "B+" => Ok(Grade::BPlus),
        "B" => Ok(Grade::B),
        "B-" => Ok(Grade::BMinus),
        "C+" => Ok(Grade::CPlus),
        "C" => Ok(Grade::C),
        "C-" => Ok(Grade::CMinus),
        "D" => Ok(Grade::D),
        "F" => Ok(Grade::F),
        _ => Err(anyhow::anyhow!(
            "Invalid grade: {s}. Valid grades: A+, A, A-, B+, B, B-, C+, C, C-, D, F"
        )),
    }
}

/// Handle TDG explain mode with function-level complexity breakdown (Issue #78)
async fn handle_explain_mode(analyzer: &TdgAnalyzer, config: &TdgCommandConfig) -> Result<()> {
    use crate::tdg::explain::ExplainedTDGScore;
    use crate::tdg::function_analyzer::FunctionAnalyzer;
    use crate::tdg::recommendation_engine::generate_recommendations;

    // First get the base TDG score
    let score = execute_tdg_analysis(analyzer, config).await?;

    // Create explained score
    let mut explained = ExplainedTDGScore::new(score.clone());

    // Analyze function-level complexity (only for single files)
    if config.path.is_file() && config.path.extension().is_some_and(|e| e == "rs") {
        let mut func_analyzer = FunctionAnalyzer::new()?;
        let functions = func_analyzer.analyze_file(&config.path)?;

        for func in functions {
            explained.add_function(func);
        }

        // Apply threshold filter
        explained.filter_functions_by_threshold(config.threshold);
        explained.sort_functions_by_impact();

        // Generate recommendations
        let recommendations = generate_recommendations(&explained);
        for rec in recommendations {
            explained.add_recommendation(rec);
        }
        explained.sort_recommendations();
    }

    // Format and output
    let output_str = format_explain_output(&explained, config)?;
    write_tdg_output(&output_str, config)?;

    Ok(())
}

/// Format explain mode output (Issue #78)
fn format_explain_output(
    explained: &crate::tdg::explain::ExplainedTDGScore,
    config: &TdgCommandConfig,
) -> Result<String> {
    match config.format {
        TdgOutputFormat::Json => Ok(serde_json::to_string_pretty(explained)?),
        TdgOutputFormat::Markdown => {
            // Markdown format uses same structure as table but in markdown
            let json = serde_json::to_string_pretty(explained)?;
            Ok(format!("```json\n{}\n```", json))
        }
        _ => {
            // Table/text format
            let mut output = String::new();

            // Header
            output.push_str("╭───────────────────────────────────────────────────────────────╮\n");
            output.push_str("│  TDG Explain Report                                           │\n");
            output.push_str("├───────────────────────────────────────────────────────────────┤\n");

            // Overall score
            output.push_str(&format!(
                "│  Score: {:.1}/100 ({})                                         │\n",
                explained.score.total,
                format_grade(explained.score.grade)
            ));
            output.push_str("│                                                               │\n");

            // Function breakdown
            if !explained.functions.is_empty() {
                output.push_str(&format!(
                    "│  📊 Functions by Complexity (threshold: {:2})                  │\n",
                    config.threshold
                ));
                output.push_str(
                    "├───────────────────────────────────────────────────────────────┤\n",
                );

                for func in explained.functions.iter().take(10) {
                    let severity_icon = match func.severity {
                        crate::tdg::explain::ComplexitySeverity::Low => "🟢",
                        crate::tdg::explain::ComplexitySeverity::Medium => "🟡",
                        crate::tdg::explain::ComplexitySeverity::High => "🟠",
                        crate::tdg::explain::ComplexitySeverity::Critical => "🔴",
                    };
                    output.push_str(&format!(
                        "│  {} {:30} [line {:4}] CC={:2} TDG={:.1} │\n",
                        severity_icon,
                        truncate_string(&func.name, 30),
                        func.line_number,
                        func.cyclomatic,
                        func.tdg_impact
                    ));
                }

                if explained.functions.len() > 10 {
                    output.push_str(&format!(
                        "│  ... and {} more functions                                    │\n",
                        explained.functions.len() - 10
                    ));
                }
            } else {
                output.push_str(
                    "│  ✅ No functions above complexity threshold                   │\n",
                );
            }

            // Recommendations
            if !explained.recommendations.is_empty() {
                output.push_str(
                    "│                                                               │\n",
                );
                output.push_str(
                    "│  💡 Recommendations                                           │\n",
                );
                output.push_str(
                    "├───────────────────────────────────────────────────────────────┤\n",
                );

                for (i, rec) in explained.recommendations.iter().take(5).enumerate() {
                    output.push_str(&format!(
                        "│  {}. [+{:.1} pts] {}                     │\n",
                        i + 1,
                        rec.expected_impact,
                        truncate_string(&rec.action, 40)
                    ));
                }
            }

            output.push_str("╰───────────────────────────────────────────────────────────────╯\n");
            Ok(output)
        }
    }
}

/// Truncate string to max length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}...", s.get(..max_len - 3).unwrap_or(s))
    }
}

// trueno-viz: Terminal Graph Visualization

/// Handle --viz mode: render TDG dependency graph in terminal
///
/// Collect all Rust files from path, excluding target directory
#[cfg(feature = "viz")]
fn collect_rust_files(path: &Path) -> Vec<PathBuf> {
    use walkdir::WalkDir;
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("/target/")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Parse visualization theme from string
#[cfg(feature = "viz")]
fn parse_viz_theme(theme_str: &str) -> crate::viz::terminal::TerminalTheme {
    use crate::viz::terminal::TerminalTheme;
    match theme_str.to_lowercase().as_str() {
        "high-contrast" | "highcontrast" => TerminalTheme::HighContrast,
        "light" => TerminalTheme::Light,
        "colorblind-safe" | "colorblind" | "cb" => TerminalTheme::ColorblindSafe,
        _ => TerminalTheme::Default,
    }
}

/// Print visualization legend and critical functions
#[cfg(feature = "viz")]
fn print_viz_legend(
    tdg_graph: &crate::tdg::tdg_graph::TdgGraph,
    theme: crate::viz::terminal::TerminalTheme,
) {
    println!("\n--- TDG Dependency Graph ---");
    println!("Theme: {:?}", theme);
    println!("Nodes: {} functions", tdg_graph.num_nodes());
    println!("Edges: {} dependencies", tdg_graph.num_edges());

    let critical = tdg_graph.critical_functions();
    if !critical.is_empty() {
        println!("\nTop Critical Functions (by PageRank):");
        for (i, (name, score)) in critical.iter().take(10).enumerate() {
            println!("  {}. {} (score: {:.4})", i + 1, name, score);
        }
    }
}

/// Uses trueno-viz force-directed layout with PageRank-based criticality scoring.
/// Supports multiple themes including colorblind-safe (Okabe-Ito palette).
#[cfg(feature = "viz")]
async fn handle_viz_mode(_analyzer: &TdgAnalyzer, config: &TdgCommandConfig) -> Result<()> {
    use crate::tdg::function_analyzer::FunctionAnalyzer;
    use crate::tdg::tdg_graph::TdgGraph;
    use crate::viz::terminal::{RenderConfig, Visualizable};

    let mut tdg_graph = TdgGraph::new();
    let mut func_analyzer = FunctionAnalyzer::new()?;
    let rust_files = collect_rust_files(&config.path);

    let mut all_functions = Vec::new();
    for file_path in &rust_files {
        if let Ok(functions) = func_analyzer.analyze_file(file_path) {
            for func in functions {
                let func_name = format!("{}::{}", file_path.display(), func.name);
                let _ = tdg_graph.add_function(func_name.clone());
                all_functions.push((file_path.clone(), func_name));
            }
        }
    }

    // Add edges: functions in same file are likely connected
    for (i, (file1, name1)) in all_functions.iter().enumerate() {
        for (file2, name2) in all_functions.iter().skip(i + 1) {
            if file1 == file2 {
                let _ = tdg_graph.add_edge(name1, name2);
            }
        }
    }

    tdg_graph.update_criticality()?;

    let theme = parse_viz_theme(&config.viz_theme);
    let render_config = RenderConfig {
        width: 120,
        height: 40,
        theme,
        mode: trueno_viz::output::TerminalMode::AnsiTrueColor,
        iterations: 100,
        critical_threshold: 0.5,
        max_nodes: 50,
        show_labels: true,
    };

    println!("{}", tdg_graph.render_terminal(&render_config)?);
    print_viz_legend(&tdg_graph, theme);

    Ok(())
}

// Tests extracted to tdg_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "tdg_handlers_tests.rs"]
mod tests;
