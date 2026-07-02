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
    super::work_handlers::handle_work_falsify(
        target,
        override_claims,
        ticket,
        Some(project_path),
        super::work_ledger::DeclaredAgent::default(),
    )
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
    let json_output = matches!(format, Some("json"));
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
    let mut dry_run_claims = Vec::new();

    for spec_file in &spec_files {
        if dry_run {
            total_claims += process_dry_run_spec(spec_file, json_output, &mut dry_run_claims)?;
            continue;
        }

        let report = engine.falsify_spec(spec_file)?;
        total_claims += report.summary.total_claims;
        total_falsified += report.summary.falsified;

        if !json_output {
            if failures_only {
                print_failures_only(&report);
            } else {
                report.display();
            }
        }

        all_reports.push(report);
    }

    // JSON mode: stdout carries exactly one jq-parseable document, no decoration
    if json_output {
        let doc = if dry_run {
            dry_run_claims_to_json(&dry_run_claims)?
        } else {
            reports_to_json(&all_reports, failures_only)?
        };
        println!("{doc}");
    } else {
        if spec_files.len() > 1 && !dry_run {
            print_multi_spec_summary(spec_files.len(), total_claims, total_falsified);
        }
        if dry_run {
            print_dry_run_footer(total_claims, spec_files.len());
        }
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

/// Extract claims from one spec in dry-run mode; print them in human mode,
/// collect them for the single JSON document otherwise. Returns claim count.
fn process_dry_run_spec(
    spec_file: &Path,
    json_output: bool,
    dry_run_claims: &mut Vec<(PathBuf, Vec<crate::services::spec_falsification::SpecClaim>)>,
) -> Result<usize> {
    let extractor = crate::services::spec_falsification::SpecClaimExtractor::new();
    let content = std::fs::read_to_string(spec_file)
        .with_context(|| format!("Failed to read: {}", spec_file.display()))?;
    let claims = extractor.extract(&content, spec_file);
    let count = claims.len();

    if json_output {
        dry_run_claims.push((spec_file.to_path_buf(), claims));
        return Ok(count);
    }

    println!(
        "{} {} -- {} claims extracted {}",
        c::label("Spec:"),
        c::path(&spec_file.display().to_string()),
        c::number(&count.to_string()),
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
    Ok(count)
}

/// Human-format summary block for multi-spec full runs
fn print_multi_spec_summary(spec_count: usize, total_claims: usize, total_falsified: usize) {
    println!();
    println!("{}", c::header("Multi-Spec Summary"));
    println!(
        "  {} {}",
        c::label("Specs analyzed:"),
        c::number(&spec_count.to_string())
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

/// Human-format trailer for dry runs
fn print_dry_run_footer(total_claims: usize, spec_count: usize) {
    println!();
    println!(
        "{} {} claims extracted across {} specs",
        c::dim("Dry run complete:"),
        c::number(&total_claims.to_string()),
        c::number(&spec_count.to_string())
    );
    println!("Run without --dry-run to falsify claims against the codebase.");
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

/// Serialize dry-run claim extraction to a single pretty-JSON document
fn dry_run_claims_to_json(
    specs: &[(PathBuf, Vec<crate::services::spec_falsification::SpecClaim>)],
) -> Result<String> {
    #[derive(serde::Serialize)]
    struct SpecClaims<'a> {
        spec: &'a Path,
        claims: &'a [crate::services::spec_falsification::SpecClaim],
    }
    #[derive(serde::Serialize)]
    struct DryRunOutput<'a> {
        dry_run: bool,
        total_claims: usize,
        specs: Vec<SpecClaims<'a>>,
    }
    let output = DryRunOutput {
        dry_run: true,
        total_claims: specs.iter().map(|(_, claims)| claims.len()).sum(),
        specs: specs
            .iter()
            .map(|(spec, claims)| SpecClaims { spec, claims })
            .collect(),
    };
    serde_json::to_string_pretty(&output).context("Failed to serialize dry-run claims")
}

/// Serialize falsification reports to a single pretty-JSON document.
///
/// Single spec emits one report object (back-compat with prior JSON output);
/// multiple specs emit an array. With `failures_only`, only Falsified verdicts
/// are retained — summary counts still reflect the full run.
fn reports_to_json(
    reports: &[crate::services::spec_falsification::SpecFalsificationReport],
    failures_only: bool,
) -> Result<String> {
    use crate::services::spec_falsification::VerdictStatus;

    let filtered: Vec<_> = reports
        .iter()
        .map(|report| {
            let mut report = report.clone();
            if failures_only {
                report
                    .verdicts
                    .retain(|v| v.status == VerdictStatus::Falsified);
            }
            report
        })
        .collect();

    match filtered.as_slice() {
        // Delegate so the single-report shape has exactly one serialization
        // path (SpecFalsificationReport::to_json)
        [single] => single.to_json(),
        many => {
            serde_json::to_string_pretty(many).context("Failed to serialize falsification reports")
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

    // ── JSON output (--format json) ─────────────────────────────────────────

    use crate::services::spec_falsification::{
        ClaimPriority, SpecClaim, SpecClaimCategory, SpecFalsificationReport,
        SpecFalsificationSummary, SpecVerdict, VerdictStatus,
    };

    fn sample_claim(id: &str) -> SpecClaim {
        SpecClaim {
            id: id.to_string(),
            original_text: "The parser MUST handle UTF-8".to_string(),
            source_line: 10,
            category: SpecClaimCategory::CodeEntity,
            priority: ClaimPriority::P0Critical,
            is_absolute: false,
            path_refs: vec![],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        }
    }

    fn sample_report() -> SpecFalsificationReport {
        SpecFalsificationReport {
            target_file: PathBuf::from("spec.md"),
            timestamp: "2026-06-12T00:00:00Z".to_string(),
            verdicts: vec![
                SpecVerdict {
                    claim: sample_claim("SPEC-001"),
                    status: VerdictStatus::Survived,
                    evidence: vec![],
                    contradiction_score: 0.0,
                },
                SpecVerdict {
                    claim: sample_claim("SPEC-002"),
                    status: VerdictStatus::Falsified,
                    evidence: vec![],
                    contradiction_score: 1.0,
                },
            ],
            summary: SpecFalsificationSummary {
                total_claims: 2,
                survived: 1,
                falsified: 1,
                unfalsifiable: 0,
                inconclusive: 0,
                health_score: 0.5,
            },
        }
    }

    #[test]
    fn test_dry_run_claims_to_json_is_parseable_with_totals() {
        let specs = vec![
            (PathBuf::from("a.md"), vec![sample_claim("SPEC-001")]),
            (
                PathBuf::from("b.md"),
                vec![sample_claim("SPEC-002"), sample_claim("SPEC-003")],
            ),
        ];
        let json = dry_run_claims_to_json(&specs).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["dry_run"], true);
        assert_eq!(parsed["total_claims"], 3);
        assert_eq!(parsed["specs"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["specs"][1]["claims"][0]["id"], "SPEC-002");
        assert_eq!(parsed["specs"][1]["claims"][0]["source_line"], 10);
    }

    #[test]
    fn test_dry_run_claims_to_json_empty_specs() {
        let json = dry_run_claims_to_json(&[]).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["total_claims"], 0);
        assert!(parsed["specs"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_reports_to_json_single_report_emits_object() {
        // PIN: single spec emits one report object, not a 1-element array (back-compat).
        let json = reports_to_json(&[sample_report()], false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
        assert_eq!(parsed["summary"]["total_claims"], 2);
        assert_eq!(parsed["verdicts"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_reports_to_json_multiple_reports_emit_array() {
        let json = reports_to_json(&[sample_report(), sample_report()], false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_reports_to_json_failures_only_filters_verdicts() {
        let json = reports_to_json(&[sample_report()], true).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let verdicts = parsed["verdicts"].as_array().unwrap();
        assert_eq!(verdicts.len(), 1);
        assert_eq!(verdicts[0]["status"], "Falsified");
        // PIN: summary still reflects the full run, not the filtered view.
        assert_eq!(parsed["summary"]["total_claims"], 2);
    }
}
