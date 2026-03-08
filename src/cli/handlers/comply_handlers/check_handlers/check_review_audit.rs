// Three-Layer CLI: Review and Audit handlers (COMPLY-045)
//
// Layer 2 (Genchi Genbutsu): Evidence-based review
// Layer 3 (Governance): Audit artifact generation

use crate::cli::colors as c;
use crate::cli::commands::ComplyOutputFormat;
use crate::models::comply_config::PmatYamlConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::check::*;
use super::check_extended::*;
use super::types::*;

/// Handle `pmat comply review` - generate evidence-based review checklist.
pub(crate) async fn handle_review(
    project_path: &Path,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    use crate::cli::handlers::comply_handlers::muda_handlers;
    use crate::cli::handlers::comply_handlers::reproducibility_handlers;

    println!(
        "{}",
        c::header("PMAT Comply Review (Layer 2: Genchi Genbutsu)")
    );
    println!("{}\n", c::rule());

    let repro = reproducibility_handlers::check_reproducibility(project_path);
    let golden = reproducibility_handlers::check_golden_trace_drift(project_path);
    let muda = muda_handlers::calculate_muda_score(project_path);
    let git_clean = check_git_clean(project_path);
    let checklist = build_review_checklist(&repro, golden, &muda, git_clean);

    let content = match format {
        ComplyOutputFormat::Json => serde_json::to_string_pretty(&checklist)?,
        ComplyOutputFormat::Markdown | ComplyOutputFormat::Text => {
            format_review_markdown(&checklist)
        }
    };

    if let Some(out_path) = output {
        fs::write(out_path, &content)?;
        println!(
            "{}",
            c::pass(&format!(
                "Review checklist written to {}",
                c::path(&out_path.display().to_string())
            ))
        );
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// A single review checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReviewItem {
    category: String,
    question: String,
    evidence: String,
    status: String,
}

/// Build the review checklist from gathered evidence.
fn build_review_checklist(
    repro: &crate::cli::handlers::comply_handlers::reproducibility_handlers::ReproducibilityReport,
    golden: Option<bool>,
    muda: &crate::cli::handlers::comply_handlers::muda_handlers::MudaReport,
    git_clean: bool,
) -> Vec<ReviewItem> {
    let mut items = Vec::new();

    items.push(ReviewItem {
        category: "Reproducibility".into(),
        question: "Can a reviewer reproduce the test suite?".into(),
        evidence: format!("Level: {} | Lockfile: {} | CI: {}",
            repro.level, repro.has_lockfile, repro.has_ci_config),
        status: if repro.level >= crate::cli::handlers::comply_handlers::reproducibility_handlers::ReproducibilityLevel::Bronze {
            "PASS".into()
        } else {
            "FAIL".into()
        },
    });

    items.push(ReviewItem {
        category: "Golden Traces".into(),
        question: "Are golden traces configured and passing?".into(),
        evidence: match golden {
            None => "Not configured (renacer.toml missing)".into(),
            Some(true) => "Traces valid - no drift detected".into(),
            Some(false) => "DRIFT DETECTED - traces have diverged".into(),
        },
        status: match golden {
            None => "N/A".into(),
            Some(true) => "PASS".into(),
            Some(false) => "FAIL".into(),
        },
    });

    items.push(ReviewItem {
        category: "Waste Score".into(),
        question: "Is the Muda waste score within acceptable limits?".into(),
        evidence: format!("Score: {:.1}/100 ({})", muda.total_score, muda.grade),
        status: if muda.total_score <= 60.0 {
            "PASS".into()
        } else {
            "WARN".into()
        },
    });

    items.push(ReviewItem {
        category: "Git State".into(),
        question: "Is the working tree clean?".into(),
        evidence: if git_clean {
            "Clean working tree".into()
        } else {
            "Uncommitted changes present".into()
        },
        status: if git_clean {
            "PASS".into()
        } else {
            "WARN".into()
        },
    });

    items.push(ReviewItem {
        category: "Environment".into(),
        question: "Is the build environment documented?".into(),
        evidence: format!(
            "Dockerfile: {} | CI: {}",
            repro.has_dockerfile, repro.has_ci_config
        ),
        status: if repro.has_dockerfile || repro.has_ci_config {
            "PASS".into()
        } else {
            "WARN".into()
        },
    });

    items
}

/// Format review checklist as markdown.
fn format_review_markdown(items: &[ReviewItem]) -> String {
    let mut out = String::new();
    out.push_str("# PMAT Comply Review Checklist\n\n");
    out.push_str("**Layer 2 (Genchi Genbutsu)**: Reviewer must verify evidence.\n\n");
    for item in items {
        let icon = match item.status.as_str() {
            "PASS" => "[x]",
            "FAIL" => "[ ] **FAIL**",
            "WARN" => "[-]",
            _ => "[ ]",
        };
        out.push_str(&format!(
            "- {} **{}**: {}\n",
            icon, item.category, item.question
        ));
        out.push_str(&format!("  - Evidence: {}\n\n", item.evidence));
    }
    out
}

/// Check if git working tree is clean.
fn check_git_clean(project_path: &Path) -> bool {
    std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_path)
        .output()
        .map(|o| o.stdout.is_empty())
        .unwrap_or(false)
}

// ============================================================
// Layer 3 (Governance): Audit artifact generation (COMPLY-045)
// ============================================================

/// Audit artifact containing compliance evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditArtifact {
    version: String,
    timestamp: DateTime<Utc>,
    git_sha: String,
    git_clean: bool,
    layer1_checks: Vec<ComplianceCheck>,
    layer2_review: Vec<ReviewItem>,
    reproducibility_level: String,
    muda_score: f64,
    golden_traces: String,
}

/// Handle `pmat comply audit` - generate governance audit artifact.
pub(crate) async fn handle_audit(
    project_path: &Path,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    println!("{}", c::header("PMAT Comply Audit (Layer 3: Governance)"));
    println!("{}\n", c::rule());

    let git_clean = check_git_clean(project_path);
    if !git_clean {
        println!("{}", c::fail("ERROR: Audit requires clean git state."));
        println!("Commit or stash all changes before generating an audit artifact.");
        println!(
            "\n{}",
            c::dim("Rationale: Audit artifacts must be reproducible from a specific commit.")
        );
        std::process::exit(1);
    }

    let git_sha = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let yaml_config = PmatYamlConfig::load(project_path).unwrap_or_default();
    let comply_config = &yaml_config.comply;

    let layer1_checks = collect_layer1_checks(project_path, comply_config);

    let repro =
        crate::cli::handlers::comply_handlers::reproducibility_handlers::check_reproducibility(
            project_path,
        );
    let golden =
        crate::cli::handlers::comply_handlers::reproducibility_handlers::check_golden_trace_drift(
            project_path,
        );
    let muda =
        crate::cli::handlers::comply_handlers::muda_handlers::calculate_muda_score(project_path);
    let layer2_review = build_review_checklist(&repro, golden, &muda, git_clean);

    let artifact = AuditArtifact {
        version: PMAT_VERSION.to_string(),
        timestamp: Utc::now(),
        git_sha,
        git_clean,
        layer1_checks,
        layer2_review,
        reproducibility_level: format!("{}", repro.level),
        muda_score: muda.total_score,
        golden_traces: match golden {
            None => "not_configured".to_string(),
            Some(true) => "passing".to_string(),
            Some(false) => "drift_detected".to_string(),
        },
    };

    let content = match format {
        ComplyOutputFormat::Json => serde_json::to_string_pretty(&artifact)?,
        ComplyOutputFormat::Markdown | ComplyOutputFormat::Text => format_audit_markdown(&artifact),
    };

    if let Some(out_path) = output {
        fs::write(out_path, &content)?;
        println!(
            "{}",
            c::pass(&format!(
                "Audit artifact written to {}",
                c::path(&out_path.display().to_string())
            ))
        );
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Collect Layer 1 checks for the audit artifact.
fn collect_layer1_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![
        filter_check_by_config(check_compute_brick(project_path), "cb-060", comply_config),
        filter_check_by_config(
            check_oip_tarantula_patterns(project_path),
            "cb-120",
            comply_config,
        ),
        filter_check_by_config(
            check_coverage_quality_patterns(project_path),
            "cb-125",
            comply_config,
        ),
        filter_check_by_config(
            check_muda_waste_score(project_path),
            "cb-300",
            comply_config,
        ),
        filter_check_by_config(
            check_reproducibility_level(project_path),
            "cb-301",
            comply_config,
        ),
        filter_check_by_config(
            check_golden_trace_drift(project_path),
            "cb-302",
            comply_config,
        ),
        filter_check_by_config(check_edd_compliance(project_path), "cb-303", comply_config),
    ]
}

/// Format audit artifact as markdown.
fn format_audit_markdown(artifact: &AuditArtifact) -> String {
    let mut out = String::new();
    out.push_str("# PMAT Compliance Audit\n\n");
    out.push_str(&format!("**PMAT Version**: {}\n", artifact.version));
    out.push_str(&format!("**Timestamp**: {}\n", artifact.timestamp));
    out.push_str(&format!("**Git SHA**: {}\n", artifact.git_sha));
    out.push_str(&format!("**Git Clean**: {}\n\n", artifact.git_clean));
    out.push_str("## Layer 1: Automated Checks (Jidoka)\n\n");
    for check in &artifact.layer1_checks {
        let icon = match check.status {
            CheckStatus::Pass => "PASS",
            CheckStatus::Warn => "WARN",
            CheckStatus::Fail => "FAIL",
            CheckStatus::Skip => "SKIP",
        };
        out.push_str(&format!(
            "- [{}] **{}**: {}\n",
            icon, check.name, check.message
        ));
    }
    out.push_str("\n## Layer 2: Review Evidence (Genchi Genbutsu)\n\n");
    for item in &artifact.layer2_review {
        out.push_str(&format!(
            "- [{}] **{}**: {}\n",
            item.status, item.category, item.evidence
        ));
    }
    out.push_str("\n## Summary\n\n");
    out.push_str(&format!(
        "- Reproducibility: {}\n",
        artifact.reproducibility_level
    ));
    out.push_str(&format!("- Muda Score: {:.1}/100\n", artifact.muda_score));
    out.push_str(&format!("- Golden Traces: {}\n", artifact.golden_traces));
    out
}
