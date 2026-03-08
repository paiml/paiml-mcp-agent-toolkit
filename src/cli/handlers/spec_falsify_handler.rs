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
