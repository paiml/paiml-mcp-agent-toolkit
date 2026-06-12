#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG baseline management: create, compare, list, update baselines
//!
//! Sprint 66 Phase 1: Baseline CRUD operations for quality tracking.

use super::display::{display_baseline_table, display_grade_distribution};
use super::is_analyzable_file;
use crate::cli::colors as c;
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
            name,
        } => create_baseline(_analyzer, &path, &output, with_git_context, name).await,

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
            println!(
                "   {} {} on {}",
                c::label("Git:"),
                c::number(&ctx.commit_sha_short),
                c::path(&ctx.branch)
            );
            Some(ctx)
        }
        None => {
            println!(
                "   {}",
                c::warn("Not in a git repository, git context unavailable")
            );
            None
        }
    }
}

/// Build a process-unique scratch path for an ephemeral baseline file.
///
/// Fixed names like `/tmp/pmat-current-baseline.json` are machine-global:
/// two concurrent pmat invocations would overwrite each other's scratch
/// baseline mid-comparison. PID plus a per-process counter makes the path
/// unique across processes and across sequential calls within one process.
pub(super) fn ephemeral_temp_json(prefix: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}.json", std::process::id()))
}

/// Create a new TDG baseline for the project (Sprint 66 Phase 1)
pub(super) async fn create_baseline(
    analyzer: &TdgAnalyzer,
    path: &Path,
    output: &Path,
    with_git_context: bool,
    name: Option<String>,
) -> Result<()> {
    use crate::tdg::TdgBaseline;

    println!("{}", c::header("Creating TDG baseline..."));
    println!(
        "   {} {}",
        c::label("Path:"),
        c::path(&path.display().to_string())
    );
    println!(
        "   {} {}",
        c::label("Output:"),
        c::path(&output.display().to_string())
    );
    println!(
        "   {} {}",
        c::label("Git context:"),
        if with_git_context { "yes" } else { "no" }
    );

    let git_context = extract_git_context(path, with_git_context);
    let mut baseline = TdgBaseline::new(git_context);
    if let Some(label) = name {
        println!("   {} {}", c::label("Name:"), c::path(&label));
        baseline.name = Some(label);
    }
    let (files_analyzed, files_skipped) =
        analyze_baseline_files(analyzer, path, &mut baseline).await?;

    println!();
    println!("\n{}", c::pass("Analysis complete:"));
    println!(
        "   {} {}",
        c::label("Files analyzed:"),
        c::number(&format!("{}", files_analyzed))
    );
    println!(
        "   {} {}",
        c::label("Files skipped:"),
        c::number(&format!("{}", files_skipped))
    );
    println!(
        "   {} {}",
        c::label("Average score:"),
        c::number(&format!("{:.1}", baseline.summary.avg_score))
    );

    display_grade_distribution(&baseline);

    baseline.save(output)?;
    println!(
        "\n{}",
        c::pass(&format!(
            "Baseline saved to: {}",
            c::path(&output.display().to_string())
        ))
    );

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

    println!("\n{}", c::dim("Analyzing files..."));

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
        // Skip include!() fragment files — they aren't standalone Rust modules
        // and tree-sitter can't parse them, resulting in false F-grade scores
        if crate::cli::language_analyzer::is_include_fragment(file_path) {
            files_skipped += 1;
            continue;
        }
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
                    println!(
                        "   {}",
                        c::warn(&format!(
                            "Skipped {}: {}",
                            c::path(&file_path.display().to_string()),
                            e
                        ))
                    );
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

    println!("{}", c::header("Comparing against baseline..."));
    println!(
        "   {} {}",
        c::label("Baseline:"),
        c::path(&baseline_path.display().to_string())
    );
    println!(
        "   {} {}",
        c::label("Current path:"),
        c::path(&current_path.display().to_string())
    );

    // Load baseline
    let old_baseline = TdgBaseline::load(baseline_path)?;
    println!(
        "   {} {} files, avg score {}",
        c::label("Loaded baseline:"),
        c::number(&format!("{}", old_baseline.summary.total_files)),
        c::number(&format!("{:.1}", old_baseline.summary.avg_score))
    );

    // Create new baseline for current state
    println!("\n{}", c::dim("Analyzing current state..."));
    let temp_output = ephemeral_temp_json("pmat-current-baseline");
    create_baseline(analyzer, current_path, &temp_output, false, None).await?;
    let new_baseline = TdgBaseline::load(&temp_output)?;

    // Clean up ephemeral baseline file
    std::fs::remove_file(&temp_output).ok();

    // Compare
    println!("\n{}", c::dim("Computing comparison..."));
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

/// Read the name label of an existing baseline file, if any
fn existing_baseline_name(path: &Path) -> Option<String> {
    crate::tdg::TdgBaseline::load(path)
        .ok()
        .and_then(|b| b.name)
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
    println!(
        "{} {}",
        c::header("Listing baselines in:"),
        c::path(&path.display().to_string())
    );

    let baselines = find_baseline_files(path);

    if baselines.is_empty() {
        println!("   {}", c::dim("No baselines found"));
        return Ok(());
    }

    println!(
        "\n{} {}:\n",
        c::label("Found"),
        c::number(&format!("{} baseline(s)", baselines.len()))
    );

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
                        "name": baseline.name,
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
                println!("{}", c::path(&path.display().to_string()));
                println!(
                    "   {} {} | {} {}",
                    c::label("Files:"),
                    c::number(&format!("{}", baseline.summary.total_files)),
                    c::label("Avg:"),
                    c::number(&format!("{:.1}", baseline.summary.avg_score))
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
    println!("{}", c::header("Updating baseline..."));
    println!(
        "   {} {}",
        c::label("Baseline:"),
        c::path(&baseline_path.display().to_string())
    );
    println!(
        "   {} {}",
        c::label("Path:"),
        c::path(&project_path.display().to_string())
    );

    // Simply re-create the baseline (overwrites the file), preserving any
    // existing name label
    create_baseline(
        analyzer,
        project_path,
        baseline_path,
        with_git_context,
        existing_baseline_name(baseline_path),
    )
    .await?;

    println!("\n{}", c::pass("Baseline updated successfully"));

    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod baseline_tests {
    use super::*;
    use crate::tdg::TdgBaseline;

    /// Ephemeral scratch paths must be unique per call and per process so
    /// concurrent pmat invocations can't clobber each other's scratch files
    /// (regression for fixed /tmp/pmat-*-baseline.json names).
    #[test]
    fn test_ephemeral_temp_json_unique_per_call() {
        let a = ephemeral_temp_json("pmat-test-scratch");
        let b = ephemeral_temp_json("pmat-test-scratch");
        assert_ne!(a, b, "two calls must yield distinct paths");

        let pid = std::process::id().to_string();
        let name = a.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.contains(&pid), "path must embed the PID: {name}");
        assert_eq!(a.extension().unwrap(), "json");
    }

    /// existing_baseline_name reads the name label back from a saved
    /// baseline, returns None for unnamed baselines and missing files —
    /// this is what lets `tdg baseline update` preserve the label.
    #[test]
    fn test_existing_baseline_name() {
        let tmp = tempfile::tempdir().unwrap();

        let mut named = TdgBaseline::new(None);
        named.name = Some("sprint-66".to_string());
        let named_path = tmp.path().join("named-baseline.json");
        named.save(&named_path).unwrap();
        assert_eq!(
            existing_baseline_name(&named_path).as_deref(),
            Some("sprint-66")
        );

        let unnamed_path = tmp.path().join("unnamed-baseline.json");
        TdgBaseline::new(None).save(&unnamed_path).unwrap();
        assert!(existing_baseline_name(&unnamed_path).is_none());

        assert!(existing_baseline_name(&tmp.path().join("missing.json")).is_none());
    }

    /// extract_git_context returns None immediately when with_git_context=false,
    /// skipping the GitContext::try_from_current_dir path.
    #[test]
    fn test_extract_git_context_disabled_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let result = extract_git_context(tmp.path(), false);
        assert!(result.is_none());
    }

    /// extract_git_context on a non-git directory hits the None arm
    /// of try_from_current_dir (logs "Not in a git repository").
    #[test]
    fn test_extract_git_context_non_git_dir_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        // Tempdir is not a git repo → try_from_current_dir returns None →
        // the warning-log arm fires and returns None.
        let result = extract_git_context(tmp.path(), true);
        assert!(result.is_none());
    }

    /// find_baseline_files returns empty for a dir with no baseline files.
    #[test]
    fn test_find_baseline_files_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let out = find_baseline_files(tmp.path());
        assert!(out.is_empty());
    }

    /// find_baseline_files picks up a real baseline file written with the
    /// canonical `-baseline.json` suffix at the project root.
    #[test]
    fn test_find_baseline_files_matches_suffix() {
        let tmp = tempfile::tempdir().unwrap();
        let baseline = TdgBaseline::new(None);
        let out_path = tmp.path().join("my-baseline.json");
        baseline.save(&out_path).unwrap();

        let found = find_baseline_files(tmp.path());
        assert_eq!(found.len(), 1, "must find the -baseline.json file");
        assert_eq!(found[0].0, out_path);
    }

    /// find_baseline_files picks up the canonical `.pmat-baseline.json` dotfile
    /// name (second name arm of the `||` match).
    #[test]
    fn test_find_baseline_files_matches_pmat_dotfile_name() {
        let tmp = tempfile::tempdir().unwrap();
        let baseline = TdgBaseline::new(None);
        let out_path = tmp.path().join(".pmat-baseline.json");
        baseline.save(&out_path).unwrap();

        let found = find_baseline_files(tmp.path());
        assert_eq!(found.len(), 1, "must find .pmat-baseline.json");
        assert_eq!(found[0].0, out_path);
    }

    /// find_baseline_files ignores files that don't match either naming rule.
    #[test]
    fn test_find_baseline_files_ignores_unrelated_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("unrelated.json"), "{}").unwrap();
        std::fs::write(tmp.path().join("baseline.json"), "{}").unwrap(); // no `-` prefix
        std::fs::write(tmp.path().join("notes.txt"), "hi").unwrap();

        let found = find_baseline_files(tmp.path());
        assert!(found.is_empty(), "must not pick up non-matching names");
    }

    /// find_baseline_files gracefully skips baseline files that fail to load
    /// (invalid JSON content) — the `TdgBaseline::load(...).ok().map(...)`
    /// returns None and filter_map drops the entry.
    #[test]
    fn test_find_baseline_files_skips_unreadable_baseline() {
        let tmp = tempfile::tempdir().unwrap();
        // Matching name but invalid JSON → load returns Err, filter_map drops.
        std::fs::write(tmp.path().join("bad-baseline.json"), "not json").unwrap();

        let found = find_baseline_files(tmp.path());
        assert!(found.is_empty(), "invalid content must be dropped");
    }
}
