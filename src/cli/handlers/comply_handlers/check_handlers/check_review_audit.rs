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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn handle_review(
    project_path: &Path,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    use crate::cli::handlers::comply_handlers::muda_handlers;
    use crate::cli::handlers::comply_handlers::reproducibility_handlers;

    crate::status_println!(
        "{}",
        c::header("PMAT Comply Review (Layer 2: Genchi Genbutsu)")
    );
    crate::status_println!("{}\n", c::rule());

    let repro = reproducibility_handlers::check_reproducibility(project_path);
    let golden = reproducibility_handlers::check_golden_trace_drift(project_path);
    let muda = muda_handlers::calculate_muda_score(project_path);
    let git_clean = check_git_clean(project_path);
    let checklist = build_review_checklist(&repro, golden, &muda, git_clean);

    let content = match format {
        ComplyOutputFormat::Json | ComplyOutputFormat::Sarif => {
            serde_json::to_string_pretty(&checklist)?
        }
        ComplyOutputFormat::Markdown | ComplyOutputFormat::Text => {
            format_review_markdown(&checklist)
        }
    };

    if let Some(out_path) = output {
        fs::write(out_path, &content)?;
        crate::status_println!(
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

/// Whether `comply audit`'s decoration may share stdout with the artifact.
///
/// `-f json` is this subcommand's DEFAULT, and it used to print the
/// "PMAT Comply Audit (Layer 3: Governance)" banner and a rule with `println!`
/// ahead of the JSON document — with an empty stderr, so `2>/dev/null` did not
/// help and `pmat comply audit | python3 -m json.tool` failed on the
/// documented default invocation. Machine formats get the artifact alone.
fn audit_banner_belongs_on_stdout(format: &ComplyOutputFormat) -> bool {
    matches!(
        format,
        ComplyOutputFormat::Text | ComplyOutputFormat::Markdown
    )
}

/// Handle `pmat comply audit` - generate governance audit artifact.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn handle_audit(
    project_path: &Path,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    if audit_banner_belongs_on_stdout(&format) {
        crate::status_println!("{}", c::header("PMAT Comply Audit (Layer 3: Governance)"));
        crate::status_println!("{}\n", c::rule());
    }

    let git_clean = check_git_clean(project_path);
    if !git_clean {
        // Errors go to stderr: this used to be printed to stdout too, so
        // `comply audit -f json 2>/dev/null` still produced an unparseable
        // document on the failure path.
        eprintln!("{}", c::fail("ERROR: Audit requires clean git state."));
        eprintln!("Commit or stash all changes before generating an audit artifact.");
        eprintln!(
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
        ComplyOutputFormat::Json | ComplyOutputFormat::Sarif => {
            serde_json::to_string_pretty(&artifact)?
        }
        ComplyOutputFormat::Markdown | ComplyOutputFormat::Text => format_audit_markdown(&artifact),
    };

    if let Some(out_path) = output {
        fs::write(out_path, &content)?;
        crate::status_println!(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_formats_get_stdout_to_themselves() {
        // `comply audit` defaults to -f json; a banner printed with println!
        // ahead of the document made that default invocation unparseable.
        assert!(!audit_banner_belongs_on_stdout(&ComplyOutputFormat::Json));
        assert!(!audit_banner_belongs_on_stdout(&ComplyOutputFormat::Sarif));
        assert!(audit_banner_belongs_on_stdout(&ComplyOutputFormat::Text));
        assert!(audit_banner_belongs_on_stdout(
            &ComplyOutputFormat::Markdown
        ));
    }

    fn item(category: &str, status: &str) -> ReviewItem {
        ReviewItem {
            category: category.into(),
            question: format!("Q for {category}"),
            evidence: format!("E for {category}"),
            status: status.into(),
        }
    }

    fn check(name: &str, status: CheckStatus) -> ComplianceCheck {
        ComplianceCheck {
            name: name.into(),
            message: format!("msg for {name}"),
            status,
            severity: Severity::Info,
        }
    }

    fn artifact_with(checks: Vec<ComplianceCheck>, items: Vec<ReviewItem>) -> AuditArtifact {
        AuditArtifact {
            version: "test-1.0".into(),
            timestamp: Utc::now(),
            git_sha: "abc1234".into(),
            git_clean: true,
            layer1_checks: checks,
            layer2_review: items,
            reproducibility_level: "Bronze".into(),
            muda_score: 42.5,
            golden_traces: "passing".into(),
        }
    }

    // ── format_review_markdown ──────────────────────────────────────────────

    #[test]
    fn test_format_review_markdown_pass_uses_check_icon() {
        let s = format_review_markdown(&[item("Cat", "PASS")]);
        assert!(s.contains("[x] **Cat**"));
        assert!(s.contains("Q for Cat"));
        assert!(s.contains("E for Cat"));
    }

    #[test]
    fn test_format_review_markdown_fail_emphasized() {
        let s = format_review_markdown(&[item("Cat", "FAIL")]);
        assert!(s.contains("[ ] **FAIL**"));
        assert!(s.contains("**Cat**"));
    }

    #[test]
    fn test_format_review_markdown_warn_uses_dash_icon() {
        let s = format_review_markdown(&[item("Cat", "WARN")]);
        assert!(s.contains("[-] **Cat**"));
    }

    #[test]
    fn test_format_review_markdown_other_status_default_box() {
        let s = format_review_markdown(&[item("Cat", "N/A")]);
        // Non-PASS/FAIL/WARN status falls through to "[ ]"
        assert!(s.contains("[ ] **Cat**"));
    }

    #[test]
    fn test_format_review_markdown_empty_items_emits_only_header() {
        let s = format_review_markdown(&[]);
        assert!(s.contains("# PMAT Comply Review Checklist"));
        assert!(s.contains("Layer 2"));
        // No items → no `[x]` / `[ ]` markers
        assert!(!s.contains("[x]"));
        assert!(!s.contains("[-]"));
    }

    #[test]
    fn test_format_review_markdown_multiple_items_all_present() {
        let items = vec![item("R1", "PASS"), item("R2", "FAIL"), item("R3", "WARN")];
        let s = format_review_markdown(&items);
        assert!(s.contains("**R1**"));
        assert!(s.contains("**R2**"));
        assert!(s.contains("**R3**"));
    }

    // ── format_audit_markdown ───────────────────────────────────────────────

    #[test]
    fn test_format_audit_markdown_includes_metadata() {
        let a = artifact_with(vec![], vec![]);
        let s = format_audit_markdown(&a);
        assert!(s.starts_with("# PMAT Compliance Audit"));
        assert!(s.contains("**PMAT Version**: test-1.0"));
        assert!(s.contains("**Git SHA**: abc1234"));
        assert!(s.contains("**Git Clean**: true"));
    }

    #[test]
    fn test_format_audit_markdown_check_status_arms() {
        let a = artifact_with(
            vec![
                check("c_pass", CheckStatus::Pass),
                check("c_warn", CheckStatus::Warn),
                check("c_fail", CheckStatus::Fail),
                check("c_skip", CheckStatus::Skip),
            ],
            vec![],
        );
        let s = format_audit_markdown(&a);
        assert!(s.contains("[PASS] **c_pass**"));
        assert!(s.contains("[WARN] **c_warn**"));
        assert!(s.contains("[FAIL] **c_fail**"));
        assert!(s.contains("[SKIP] **c_skip**"));
    }

    #[test]
    fn test_format_audit_markdown_includes_layer2_items() {
        let a = artifact_with(vec![], vec![item("R1", "PASS")]);
        let s = format_audit_markdown(&a);
        assert!(s.contains("Layer 2: Review Evidence"));
        assert!(s.contains("**R1**"));
        assert!(s.contains("E for R1"));
    }

    #[test]
    fn test_format_audit_markdown_summary_lines() {
        let a = artifact_with(vec![], vec![]);
        let s = format_audit_markdown(&a);
        assert!(s.contains("## Summary"));
        assert!(s.contains("Reproducibility: Bronze"));
        assert!(s.contains("Muda Score: 42.5/100"));
        assert!(s.contains("Golden Traces: passing"));
    }

    // ── check_git_clean ─────────────────────────────────────────────────────

    // Note: a "non-git dir" test (expecting check_git_clean → false) is
    // environment-dependent. `git status` walks up the filesystem and may
    // find a parent .git on developer machines. Removed in favor of the
    // initialized-repo + untracked-files tests below, which are deterministic.

    #[test]
    fn test_check_git_clean_initialized_empty_repo() {
        // git init in a tempdir → no untracked files → status --porcelain is empty → true
        let tmp = tempfile::TempDir::new().unwrap();
        let init_ok = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(tmp.path())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !init_ok {
            // git not on PATH — skip rather than fail
            return;
        }
        // Empty repo with no untracked files → clean
        assert!(check_git_clean(tmp.path()));
    }

    #[test]
    fn test_check_git_clean_with_untracked_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let init_ok = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(tmp.path())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !init_ok {
            return;
        }
        // Add an untracked file → status --porcelain emits `?? newfile`
        fs::write(tmp.path().join("newfile.txt"), "x").unwrap();
        assert!(!check_git_clean(tmp.path()));
    }

    // ── the help must not promise evidence the artifact does not carry ──────

    /// `comply audit --help` promised "Produces signed compliance evidence"
    /// while `AuditArtifact` has no signature, digest or attestation field —
    /// the serialized document is plain JSON that anyone can edit undetectably.
    #[test]
    fn audit_help_does_not_claim_a_signature_the_artifact_lacks() {
        use clap::Subcommand;
        let help = crate::cli::commands::on_big_stack(|| {
            let cmd = crate::cli::commands::ComplyCommands::augment_subcommands(
                clap::Command::new("comply"),
            );
            let audit = cmd
                .get_subcommands()
                .find(|s| s.get_name() == "audit")
                .expect("comply audit subcommand must exist");
            audit
                .get_long_about()
                .or_else(|| audit.get_about())
                .map(std::string::ToString::to_string)
                .unwrap_or_default()
        });

        assert!(
            !help.contains("signed compliance evidence"),
            "audit help must not promise a signature: {help}"
        );

        // And the artifact still carries no such field, which is what makes the
        // claim false — if one is ever added, this assertion is the reminder to
        // restore the promise in the help text.
        let artifact = AuditArtifact {
            version: "0.0.0".to_string(),
            timestamp: Utc::now(),
            git_sha: "deadbeef".to_string(),
            git_clean: true,
            layer1_checks: vec![],
            layer2_review: vec![],
            reproducibility_level: "L0".to_string(),
            muda_score: 0.0,
            golden_traces: "not_configured".to_string(),
        };
        let doc: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&artifact).unwrap()).unwrap();
        let keys: Vec<&String> = doc.as_object().unwrap().keys().collect();
        assert!(
            !keys
                .iter()
                .any(|k| k.contains("signature") || k.contains("digest") || k.contains("attest")),
            "artifact keys changed: {keys:?}"
        );
    }
}
