#![cfg_attr(coverage_nightly, coverage(off))]
//! Handler for `pmat falsify <spec-file>` — RAG-powered spec falsification
//!
//! Detects whether the target is a file path (spec falsification) or a work item ID
//! (contract falsification) and routes accordingly.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Handle the top-level `pmat falsify` command
///
/// Routes to spec falsification if target is a file path,
/// otherwise falls back to work item contract falsification.
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
                "Spec: {} — {} claims extracted (dry run)",
                spec_file.display(),
                claims.len()
            );
            for claim in &claims {
                println!(
                    "  [{}] {} {} (line {}): {}",
                    claim.id,
                    claim.priority,
                    claim.category,
                    claim.source_line,
                    truncate(&claim.original_text, 80),
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
        println!("=== Multi-Spec Summary ===");
        println!("  Specs analyzed: {}", spec_files.len());
        println!("  Total claims:   {}", total_claims);
        println!("  Falsified:      {}", total_falsified);
        let health = if total_claims > 0 {
            (total_claims - total_falsified) as f64 / total_claims as f64
        } else {
            1.0
        };
        println!("  Health:         {:.2}", health);
    }

    if dry_run {
        println!();
        println!("Dry run complete: {} claims extracted across {} specs", total_claims, spec_files.len());
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
            "{}: All {} claims survived",
            report.target_file.display(),
            report.summary.total_claims
        );
        return;
    }

    println!(
        "{}: {} falsified / {} total",
        report.target_file.display(),
        falsified.len(),
        report.summary.total_claims,
    );
    for verdict in &falsified {
        println!(
            "  line {}: {}",
            verdict.claim.source_line,
            truncate(&verdict.claim.original_text, 80),
        );
        for ev in &verdict.evidence {
            if ev.contradiction_score >= 0.8 {
                println!("    \x1b[31m✗\x1b[0m {} → {}", ev.check, ev.finding);
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
