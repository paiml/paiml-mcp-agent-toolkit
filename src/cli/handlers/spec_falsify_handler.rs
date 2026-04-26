#![cfg_attr(coverage_nightly, coverage(off))]
//! Handler for `pmat falsify <spec-file>` — RAG-powered spec falsification
//!
//! Detects whether the target is a file path (spec falsification) or a work item ID
//! (contract falsification) and routes accordingly.

use crate::cli::colors as c;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Handle the top-level `pmat falsify` command
///
/// Routes to spec falsification if target is a file path,
/// otherwise falls back to work item contract falsification.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_falsify(
    target: String,
    override_claims: Option<Vec<String>>,
    ticket: Option<String>,
    path: Option<PathBuf>,
    format: Option<String>,
    failures_only: bool,
    dry_run: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Detect if target is a file path or a directory
    let target_path = project_path.join(&target);
    if target_path.exists() && (target_path.is_file() || target_path.is_dir()) {
        return handle_spec_falsification(
            &target_path,
            &project_path,
            format.as_deref(),
            failures_only,
            dry_run,
        )
        .await;
    }

    // Also check if it's an absolute or relative path that exists directly
    let direct_path = Path::new(&target);
    if direct_path.exists() && (direct_path.is_file() || direct_path.is_dir()) {
        return handle_spec_falsification(
            direct_path,
            &project_path,
            format.as_deref(),
            failures_only,
            dry_run,
        )
        .await;
    }

    // Not a file path — treat as work item ID (delegate to existing handler)
    super::work_handlers::handle_work_falsify(target, override_claims, ticket, Some(project_path))
        .await
}

/// Run spec falsification on a file or directory
async fn handle_spec_falsification(
    target: &Path,
    project_path: &Path,
    format: Option<&str>,
    failures_only: bool,
    dry_run: bool,
) -> Result<()> {
    let engine = crate::services::spec_falsification::FalsificationEngine::new(project_path);

    // Collect spec files
    let spec_files = if target.is_dir() {
        collect_spec_files(target)?
    } else {
        vec![target.to_path_buf()]
    };

    if spec_files.is_empty() {
        anyhow::bail!("No specification files found at: {}", target.display());
    }

    let mut total_claims = 0usize;
    let mut total_falsified = 0usize;
    let mut all_reports = Vec::new();

    for spec_file in &spec_files {
        if dry_run {
            // Dry run: extract claims only
            let extractor = crate::services::spec_falsification::SpecClaimExtractor::new();
            let content = std::fs::read_to_string(spec_file)
                .with_context(|| format!("Failed to read: {}", spec_file.display()))?;
            let claims = extractor.extract(&content, spec_file);
            println!(
                "{} {} -- {} claims extracted {}",
                c::label("Spec:"),
                c::path(&spec_file.display().to_string()),
                c::number(&claims.len().to_string()),
                c::dim("(dry run)")
            );
            for claim in &claims {
                println!(
                    "  [{}] {} {} (line {}): {}",
                    c::label(&claim.id),
                    claim.priority,
                    claim.category,
                    c::number(&claim.source_line.to_string()),
                    c::dim(&truncate(&claim.original_text, 80)),
                );
            }
            total_claims += claims.len();
            continue;
        }

        let report = engine.falsify_spec(spec_file)?;
        total_claims += report.summary.total_claims;
        total_falsified += report.summary.falsified;

        match format {
            Some("json") => {
                let json = report.to_json()?;
                println!("{}", json);
            }
            _ => {
                if failures_only {
                    print_failures_only(&report);
                } else {
                    report.display();
                }
            }
        }

        all_reports.push(report);
    }

    // Multi-spec summary
    if spec_files.len() > 1 && !dry_run {
        println!();
        println!("{}", c::header("Multi-Spec Summary"));
        println!(
            "  {} {}",
            c::label("Specs analyzed:"),
            c::number(&spec_files.len().to_string())
        );
        println!(
            "  {} {}",
            c::label("Total claims:  "),
            c::number(&total_claims.to_string())
        );
        println!(
            "  {} {}",
            c::label("Falsified:     "),
            c::number(&total_falsified.to_string())
        );
        let health = if total_claims > 0 {
            (total_claims - total_falsified) as f64 / total_claims as f64
        } else {
            1.0
        };
        println!(
            "  {} {}",
            c::label("Health:        "),
            c::pct(health * 100.0, 90.0, 70.0)
        );
    }

    if dry_run {
        println!();
        println!(
            "{} {} claims extracted across {} specs",
            c::dim("Dry run complete:"),
            c::number(&total_claims.to_string()),
            c::number(&spec_files.len().to_string())
        );
        println!("Run without --dry-run to falsify claims against the codebase.");
    }

    // Exit with non-zero if any claims were falsified
    if total_falsified > 0 && !dry_run {
        anyhow::bail!(
            "Falsification failed: {} claims falsified across {} specs",
            total_falsified,
            spec_files.len()
        );
    }

    Ok(())
}

/// Collect markdown spec files from a directory
fn collect_spec_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "md" || ext == "yaml" || ext == "yml" {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Print only falsified claims
fn print_failures_only(report: &crate::services::spec_falsification::SpecFalsificationReport) {
    use crate::services::spec_falsification::VerdictStatus;

    let falsified: Vec<_> = report
        .verdicts
        .iter()
        .filter(|v| v.status == VerdictStatus::Falsified)
        .collect();

    if falsified.is_empty() {
        println!(
            "{}: {}",
            c::path(&report.target_file.display().to_string()),
            c::pass(&format!(
                "All {} claims survived",
                report.summary.total_claims
            ))
        );
        return;
    }

    println!(
        "{}: {} falsified / {} total",
        c::path(&report.target_file.display().to_string()),
        c::number(&falsified.len().to_string()),
        c::number(&report.summary.total_claims.to_string()),
    );
    for verdict in &falsified {
        println!(
            "  line {}: {}",
            c::number(&verdict.claim.source_line.to_string()),
            truncate(&verdict.claim.original_text, 80),
        );
        for ev in &verdict.evidence {
            if ev.contradiction_score >= 0.8 {
                println!("    {} {} -> {}", c::fail(&ev.check), c::DIM, ev.finding);
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod spec_falsify_helper_tests {
    //! Wave 39 PR19 — pure-helper coverage for spec_falsify_handler.rs.
    //! The async handle_* functions invoke RAG indexes + spec parsing
    //! pipelines (disqualified). The pure helpers `truncate` and
    //! `collect_spec_files` are testable.
    use super::*;

    // ── truncate ────────────────────────────────────────────────────────────

    #[test]
    fn test_truncate_under_limit_returns_original() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_at_limit_returns_original() {
        // PIN: `s.len() <= max` is INCLUSIVE — exact-length strings pass through.
        assert_eq!(truncate("abcde", 5), "abcde");
    }

    #[test]
    fn test_truncate_over_limit_appends_ellipsis() {
        // PIN: truncates to (max - 3) chars then appends "..." (total length = max).
        let result = truncate("abcdefghij", 8);
        assert_eq!(result.len(), 8);
        assert!(result.ends_with("..."));
        assert_eq!(result, "abcde...");
    }

    #[test]
    fn test_truncate_max_three_or_less_uses_saturating_sub() {
        // PIN: max < 3 uses `saturating_sub` to avoid underflow → slice 0 chars + "..."
        // Result is just "..." (length 3) regardless of input.
        assert_eq!(truncate("abcdefgh", 2), "...");
        assert_eq!(truncate("abcdefgh", 1), "...");
        assert_eq!(truncate("abcdefgh", 0), "...");
    }

    #[test]
    fn test_truncate_empty_input() {
        assert_eq!(truncate("", 10), "");
    }

    // ── collect_spec_files ──────────────────────────────────────────────────

    #[test]
    fn test_collect_spec_files_md_and_yaml_collected() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("spec1.md"), "# spec").unwrap();
        std::fs::write(tmp.path().join("spec2.yaml"), "version: 1").unwrap();
        std::fs::write(tmp.path().join("spec3.yml"), "version: 1").unwrap();
        let files = collect_spec_files(tmp.path()).unwrap();
        // PIN: .md, .yaml, .yml are ALL collected.
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_collect_spec_files_filters_other_extensions() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("spec.md"), "# spec").unwrap();
        std::fs::write(tmp.path().join("notes.txt"), "notes").unwrap();
        std::fs::write(tmp.path().join("config.json"), "{}").unwrap();
        let files = collect_spec_files(tmp.path()).unwrap();
        // PIN: only .md/.yaml/.yml are collected; .txt/.json are skipped.
        assert_eq!(files.len(), 1);
        assert!(files[0].extension().unwrap() == "md");
    }

    #[test]
    fn test_collect_spec_files_skips_directories() {
        // PIN: only files (path.is_file()) are collected; subdirectories are ignored.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("subdir.md")).unwrap();
        std::fs::write(tmp.path().join("real.md"), "# spec").unwrap();
        let files = collect_spec_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name().unwrap(), "real.md");
    }

    #[test]
    fn test_collect_spec_files_results_sorted() {
        // PIN: results are sorted alphabetically.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("z.md"), "").unwrap();
        std::fs::write(tmp.path().join("a.md"), "").unwrap();
        std::fs::write(tmp.path().join("m.md"), "").unwrap();
        let files = collect_spec_files(tmp.path()).unwrap();
        assert_eq!(files[0].file_name().unwrap(), "a.md");
        assert_eq!(files[1].file_name().unwrap(), "m.md");
        assert_eq!(files[2].file_name().unwrap(), "z.md");
    }

    #[test]
    fn test_collect_spec_files_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let files = collect_spec_files(tmp.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_spec_files_nonexistent_dir_returns_err() {
        let nonexistent = std::path::PathBuf::from("/nonexistent/path/that/should/not/exist");
        let result = collect_spec_files(&nonexistent);
        assert!(result.is_err());
    }
}
