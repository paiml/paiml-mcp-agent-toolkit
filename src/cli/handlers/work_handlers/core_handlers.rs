// Work command handlers for unified GitHub/YAML workflow (Issue #75)
//
// Implements the hybrid write-through architecture for GitHub and YAML tracking.

use crate::cli::commands::SyncDirection;
use crate::models::roadmap::{ItemStatus, Priority, RoadmapItem};
use crate::services::changelog_manager::{ChangeCategory, ChangelogEntry};
#[cfg(feature = "github-api")]
use crate::services::github_client::GitHubClient;
use crate::services::hook_manager;
use crate::services::roadmap_service::RoadmapService;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Quality handlers extracted to work_quality_handlers.rs for file health compliance (CB-040)
pub use super::work_quality_handlers::{run_quality_gates, FalsificationResult};

// Work Contract and Falsification (PMAT Work Contract specification)
use super::work_contract::{FileManifest, WorkContract};
use super::work_falsification::{capture_baseline, run_falsification_tests};

/// Handle work init command
pub async fn handle_work_init(
    github_repo: Option<String>,
    no_github: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("🚀 Initializing unified GitHub/YAML workflow...");
    println!();

    // Create roadmap service
    let service = RoadmapService::new(&roadmap_path);

    // Check if already initialized
    if service.exists() {
        println!("⚠️  Roadmap already exists at: {}", roadmap_path.display());
        println!("   Use `pmat work status` to view current items");
        return Ok(());
    }

    // Determine GitHub configuration
    let github_enabled = !no_github;
    let repo = if github_enabled {
        match github_repo {
            Some(r) => Some(r),
            None => {
                // Try to detect from git remote
                detect_github_repo(&project_path)?
            }
        }
    } else {
        None
    };

    // Initialize roadmap
    service.initialize(repo.clone())?;

    println!("✅ Created roadmap: {}", roadmap_path.display());

    // Install commit-msg hook
    match hook_manager::install_commit_msg_hook(&project_path) {
        Ok(()) => {
            println!("✅ Installed commit-msg hook");
        }
        Err(e) => {
            println!("⚠️  Failed to install commit-msg hook: {}", e);
            println!("   Workflow will work, but commit messages won't be validated");
        }
    }

    println!();

    // Display configuration
    println!("📋 Configuration:");
    println!(
        "   GitHub integration: {}",
        if github_enabled {
            "✅ enabled"
        } else {
            "❌ disabled"
        }
    );
    if let Some(r) = &repo {
        println!("   GitHub repository: {}", r);
    }
    println!();

    // Next steps
    println!("🎯 Next steps:");
    println!("   1. Create GitHub issue or edit roadmap.yaml");
    println!("   2. Start work: pmat work start <issue-number-or-ticket-id>");
    println!("   3. Continue: pmat work continue <id>");
    println!("   4. Complete: pmat work complete <id>");
    println!();

    if github_enabled && repo.is_none() {
        println!("💡 Tip: Set GitHub repo with:");
        println!("   pmat config set github.repo owner/repo");
        println!();
    }

    Ok(())
}

/// Resolve a GitHub issue into a RoadmapItem (helper for handle_work_start)
async fn resolve_github_issue(
    roadmap: &crate::models::roadmap::Roadmap,
    issue_num: u64,
) -> RoadmapItem {
    println!("📋 Type: GitHub issue #{}", issue_num);

    let mut item = if let Some(ref repo) = roadmap.github_repo {
        match fetch_github_issue(repo, issue_num).await {
            Ok(gh_issue) => {
                println!("   ✅ Fetched from GitHub: {}", gh_issue.title);
                let mut item = RoadmapItem::from_github_issue(issue_num, gh_issue.title.clone());
                item.labels = gh_issue.labels.clone();
                if let Some(body) = &gh_issue.body {
                    item.acceptance_criteria = parse_acceptance_criteria(body);
                }
                item
            }
            Err(e) => {
                println!("   ⚠️  Failed to fetch from GitHub: {}", e);
                println!("   Creating placeholder (will sync later)");
                RoadmapItem::from_github_issue(issue_num, format!("Issue #{}", issue_num))
            }
        }
    } else {
        println!("   ℹ️  GitHub not configured, creating placeholder");
        RoadmapItem::from_github_issue(issue_num, format!("Issue #{}", issue_num))
    };

    item.status = ItemStatus::InProgress;
    item
}

/// Resolve or create a YAML ticket (helper for handle_work_start)
async fn resolve_yaml_ticket(
    service: &RoadmapService,
    id: &str,
    roadmap: &crate::models::roadmap::Roadmap,
    create_github: bool,
) -> Result<RoadmapItem> {
    println!("📋 Type: YAML ticket {}", id);

    if let Some(existing) = service.find_item(id)? {
        println!("   Found existing ticket");
        let mut item = existing;
        item.status = ItemStatus::InProgress;
        item.updated = chrono::Utc::now().to_rfc3339();
        return Ok(item);
    }

    let mut item = RoadmapItem::new(id.to_string(), format!("New task: {}", id));
    item.status = ItemStatus::InProgress;
    item.priority = Priority::Medium;

    if create_github {
        try_create_github_issue(roadmap, &mut item).await;
    }

    Ok(item)
}

/// Try to create a GitHub issue for a new YAML ticket (helper for resolve_yaml_ticket)
async fn try_create_github_issue(
    roadmap: &crate::models::roadmap::Roadmap,
    item: &mut RoadmapItem,
) {
    if let Some(ref repo) = roadmap.github_repo {
        println!("   🔄 Creating GitHub issue...");
        match create_github_issue_from_item(repo, item).await {
            Ok(gh_issue) => {
                println!("   ✅ Created GitHub issue #{}", gh_issue.number);
                item.github_issue = Some(gh_issue.number);
                item.id = format!("GH-{}", gh_issue.number);
            }
            Err(e) => {
                println!("   ⚠️  Failed to create GitHub issue: {}", e);
                println!("   Continuing with YAML-only ticket");
            }
        }
    } else {
        println!("   ⚠️  GitHub not configured, skipping issue creation");
    }
}

/// Create a work contract with baseline metrics (helper for handle_work_start)
async fn create_work_contract(project_path: &Path, item_id: &str) {
    println!();
    println!("📋 Creating Work Contract (Popperian Falsification)...");

    let baseline_commit = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let mut contract = WorkContract::new(item_id.to_string(), baseline_commit);

    match capture_baseline(project_path).await {
        Ok((tdg, coverage, rust_score)) => {
            contract.baseline_tdg = tdg;
            contract.baseline_coverage = coverage;
            contract.baseline_rust_score = rust_score;
        }
        Err(e) => {
            println!("   ⚠️  Could not capture baseline metrics: {}", e);
        }
    }

    build_and_attach_manifest(project_path, &mut contract);
    save_contract(project_path, &contract);
}

/// Build file manifest and attach to contract (helper for create_work_contract)
fn build_and_attach_manifest(project_path: &Path, contract: &mut WorkContract) {
    println!("   📂 Building file manifest...");
    match FileManifest::build(project_path) {
        Ok(manifest) => {
            let file_count = manifest.files.len();
            let protected_count = manifest.coverage_required.len();
            contract.baseline_file_manifest = manifest;
            println!(
                "      ✅ {} files tracked, {} protected from exclusion",
                file_count, protected_count
            );
        }
        Err(e) => {
            println!("   ⚠️  Could not build file manifest: {}", e);
        }
    }
}

/// Save contract to disk (helper for create_work_contract)
fn save_contract(project_path: &Path, contract: &WorkContract) {
    match contract.save(project_path) {
        Ok(contract_path) => {
            println!("   ✅ Contract saved: {}", contract_path.display());
        }
        Err(e) => {
            println!("   ⚠️  Could not save contract: {}", e);
        }
    }
}

/// Handle work start command
pub async fn handle_work_start(
    id: String,
    with_spec: bool,
    epic: bool,
    path: Option<PathBuf>,
    create_github: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    println!("🚀 Starting work on: {}", id);
    println!();

    let mut roadmap = service.load()?;
    let is_github_issue = id.parse::<u64>().is_ok();

    let mut item = if is_github_issue {
        let issue_num: u64 = id.parse()?;
        resolve_github_issue(&roadmap, issue_num).await
    } else {
        resolve_yaml_ticket(&service, &id, &roadmap, create_github).await?
    };

    if epic {
        item.item_type = crate::models::roadmap::ItemType::Epic;
        println!("📦 Created as epic: {}", item.title);
        println!("   Add subtasks manually to roadmap.yaml or use future commands");
    }

    roadmap.upsert_item(item.clone());
    service.save(&roadmap)?;
    println!("✅ Updated roadmap: {}", roadmap_path.display());

    create_work_contract(&project_path, &item.id).await;

    if with_spec {
        create_spec_if_needed(&project_path, &item, &id, is_github_issue)?;
    }

    print_work_start_next_steps(&id);
    Ok(())
}

/// Create specification file if it does not exist (helper for handle_work_start)
fn create_spec_if_needed(
    project_path: &Path,
    item: &RoadmapItem,
    id: &str,
    is_github_issue: bool,
) -> Result<()> {
    let spec_path = if is_github_issue {
        project_path.join(format!(
            "docs/specifications/{:03}-spec.md",
            item.github_issue.expect("internal error")
        ))
    } else {
        project_path.join(format!("docs/specifications/{}-spec.md", id.to_lowercase()))
    };

    if !spec_path.exists() {
        create_specification_template(&spec_path.to_path_buf(), item)?;
        println!("✅ Created specification: {}", spec_path.display());
    } else {
        println!("   Specification exists: {}", spec_path.display());
    }
    Ok(())
}

/// Print next steps after work start (helper for handle_work_start)
fn print_work_start_next_steps(id: &str) {
    println!();
    println!("🎯 Next steps:");
    println!("   1. Review specification (if created)");
    println!("   2. Write failing tests (RED phase)");
    println!("   3. Implement feature (GREEN phase)");
    println!("   4. Refactor (REFACTOR phase)");
    println!("   5. Continue: pmat work continue {}", id);
    println!("   6. Complete: pmat work complete {}", id);
    println!();
}

/// Handle work continue command
pub async fn handle_work_continue(id: String, path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    println!("🔄 Continuing work on: {}", id);
    println!();

    // Find item
    let item = service
        .find_item(&id)?
        .with_context(|| format!("Item not found: {}", id))?;

    // Display progress
    let completion = item.completion_percentage();
    println!("📊 Progress: {}% complete", completion);
    println!("   Status: {:?}", item.status);
    println!("   Title: {}", item.title);
    if let Some(spec) = &item.spec {
        println!("   Spec: {}", spec.display());
    }
    println!();

    // Show acceptance criteria
    if !item.acceptance_criteria.is_empty() {
        println!("📋 Acceptance Criteria:");
        for (i, criterion) in item.acceptance_criteria.iter().enumerate() {
            println!("   {}. {}", i + 1, criterion);
        }
        println!();
    }

    // Show phases
    if !item.phases.is_empty() {
        println!("📌 Phases:");
        for phase in &item.phases {
            let emoji = match phase.status {
                ItemStatus::Completed => "✅",
                ItemStatus::InProgress => "⏳",
                _ => "⬜",
            };
            println!("   {} {} ({}%)", emoji, phase.name, phase.completion);
        }
        println!();
    }

    // Show subtasks (for epics)
    if !item.subtasks.is_empty() {
        println!("📦 Subtasks:");
        for subtask in &item.subtasks {
            let emoji = match subtask.status {
                ItemStatus::Completed => "✅",
                ItemStatus::InProgress => "⏳",
                _ => "⬜",
            };
            println!("   {} {} ({}%)", emoji, subtask.title, subtask.completion);
        }
        println!();
    }

    // Next steps
    println!("🎯 Next steps:");
    println!("   Continue working on: {}", item.title);
    println!("   When done: pmat work complete {}", id);
    println!();

    Ok(())
}

/// Commit metadata structure for linking commits to work items and quality scores
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitMetadata {
    commit_sha: Option<String>,
    work_item_id: String,
    prompt: String,
    tdg_score: f64,
    repo_score: f64,
    rust_project_score: Option<f64>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Capture commit metadata (O(1) from .pmat-metrics/ cache)
async fn capture_commit_metadata(
    project_path: &PathBuf,
    item: &RoadmapItem,
) -> Result<CommitMetadata> {
    use std::process::Command;

    let short_sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(project_path)
        .output()?;
    let short_sha = String::from_utf8_lossy(&short_sha.stdout)
        .trim()
        .to_string();

    // Capture scores (O(1) from cache)
    let tdg_score = capture_tdg_score(project_path).await.unwrap_or(0.0);
    let repo_score = capture_repo_score(project_path).await.unwrap_or(0.0);
    let rust_score = if project_path.join("Cargo.toml").exists() {
        Some(
            capture_rust_project_score(project_path)
                .await
                .unwrap_or(0.0),
        )
    } else {
        None
    };

    let metadata = CommitMetadata {
        commit_sha: None, // Will be filled after commit
        work_item_id: item.id.clone(),
        prompt: item.title.clone(),
        tdg_score,
        repo_score,
        rust_project_score: rust_score,
        timestamp: chrono::Utc::now(),
    };

    // Write to .pmat-metrics/
    let metrics_dir = project_path.join(".pmat-metrics");
    std::fs::create_dir_all(&metrics_dir)?;

    let meta_file = metrics_dir.join(format!("commit-{}-meta.json", short_sha));
    let json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(meta_file, json)?;

    Ok(metadata)
}

/// Capture TDG score (O(1) from cache)
async fn capture_tdg_score(project_path: &PathBuf) -> Result<f64> {
    let metrics_dir = project_path.join(".pmat-metrics");
    let tdg_file = metrics_dir.join("tdg-score.json");

    if tdg_file.exists() {
        let content = std::fs::read_to_string(&tdg_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(score) = json.get("score").and_then(|v| v.as_f64()) {
            return Ok(score);
        }
    }

    // Fallback: compute score if cache doesn't exist
    Ok(0.0)
}

/// Capture repo score (O(1) from cache)
async fn capture_repo_score(project_path: &PathBuf) -> Result<f64> {
    let metrics_dir = project_path.join(".pmat-metrics");
    let repo_file = metrics_dir.join("repo-score.json");

    if repo_file.exists() {
        let content = std::fs::read_to_string(&repo_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(score) = json.get("score").and_then(|v| v.as_f64()) {
            return Ok(score);
        }
    }

    // Fallback: compute score if cache doesn't exist
    Ok(0.0)
}

/// Capture rust project score (O(1) from cache)
async fn capture_rust_project_score(project_path: &PathBuf) -> Result<f64> {
    let metrics_dir = project_path.join(".pmat-metrics");
    let rust_file = metrics_dir.join("rust-project-score.json");

    if rust_file.exists() {
        let content = std::fs::read_to_string(&rust_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(score) = json.get("total_earned").and_then(|v| v.as_f64()) {
            return Ok(score);
        }
    }

    // Fallback: compute score if cache doesn't exist
    Ok(0.0)
}

// Falsification report types from work_falsification module
use super::work_falsification::{ClaimResult, FalsificationReport};
// Falsification ledger: append-only receipt tracking
use super::work_ledger::{FalsificationLedger, FalsificationReceipt, FalsificationTrigger};

/// Filter failures not covered by overrides
fn filter_unoverriden_failures<'a>(
    failures: &[&'a ClaimResult],
    override_claims: Option<&Vec<String>>,
) -> Vec<&'a ClaimResult> {
    failures
        .iter()
        .filter(|failure| {
            let claim_name = claim_to_override_name(&failure.hypothesis);
            if let Some(overrides) = override_claims {
                !overrides.iter().any(|o| o.to_lowercase() == claim_name.to_lowercase())
            } else {
                true
            }
        })
        .copied()
        .collect()
}

/// Print warning failures (non-blocking)
fn print_warning_failures(report: &FalsificationReport) {
    let warnings = report.warning_failures();
    if !warnings.is_empty() {
        println!();
        println!("Warnings (non-blocking):");
        for warning in warnings {
            println!(
                "  - [{}] {}: {}",
                warning.index, warning.hypothesis, warning.result.explanation
            );
        }
    }
}

/// Print result when work is blocked
fn print_blocked_result(report: &FalsificationReport, unoverrideable: &[&ClaimResult], id: &str) {
    println!(
        "❌ FALSIFICATION RESULT: BLOCKED ({} failure(s), {} warning(s))",
        report.failed, report.warnings
    );
    println!();
    println!("Failures (must fix):");
    for failure in unoverrideable {
        println!(
            "  - [{}] {}: {}",
            failure.index, failure.hypothesis, failure.result.explanation
        );
    }

    print_warning_failures(report);

    println!();
    println!("Fix issues and retry: pmat work complete {}", id);
    println!();
    println!("Or override with accountability (Popperian Protocol):");
    println!("  1. Create debt ticket: pmat comply upgrade --target popperian");
    println!("  2. pmat work complete {} --override-claims coverage,complexity --ticket DEBT-XXX", id);
}

/// Validate that override claims have an associated ticket (Popperian accountability)
fn validate_override_accountability(
    override_claims: &Option<Vec<String>>,
    ticket: &Option<String>,
    id: &str,
) -> Result<()> {
    if override_claims.is_some() && ticket.is_none() {
        anyhow::bail!(
            "Error: --ticket is mandatory for overrides.\n\n\
             Popperian Principle: Every override must be accountable.\n\
             Create a debt ticket first:\n\
             1. pmat comply upgrade --target popperian\n\
             2. Or manually create .pmat-tickets/DEBT-XXX.yaml\n\n\
             Then retry with: pmat work complete {} --override-claims <claims> --ticket <TICKET-ID>",
            id
        );
    }
    Ok(())
}

/// Run quality gates and report results (helper for handle_work_complete)
async fn run_quality_check(project_path: &PathBuf, skip_quality: bool) -> Result<()> {
    if skip_quality {
        println!("⚠️  Quality gates SKIPPED (--skip-quality)");
        println!();
        return Ok(());
    }

    println!("🔍 Running quality gates...");
    println!();

    match run_quality_gates(project_path).await {
        Ok(passed) => {
            if passed {
                println!("✅ All quality gates passed");
                println!();
            } else {
                anyhow::bail!(
                    "Quality gates failed. Fix issues or use --skip-quality to bypass."
                );
            }
        }
        Err(e) => {
            println!("⚠️  Quality gates error: {}", e);
            println!("   Continuing (use strict mode to block on errors)");
            println!();
        }
    }
    Ok(())
}

/// Run contract-based falsification (helper for handle_work_complete)
///
/// Legacy mode (no contract) is now a hard error — users must run `pmat work start` first.
async fn run_contract_falsification(
    project_path: &Path,
    item_id: &str,
    override_claims: &Option<Vec<String>>,
    ticket: &Option<String>,
    id: &str,
) -> Result<()> {
    if !WorkContract::exists(project_path, item_id) {
        anyhow::bail!(
            "No work contract found for '{}'. Run 'pmat work start {}' to create one.\n\
             Contracts are required for falsification-gated completion.",
            item_id, id
        );
    }

    println!("📜 Loading Work Contract...");
    match WorkContract::load(project_path, item_id) {
        Ok(contract) => {
            println!(
                "   Baseline: {} (TDG: {:.1}, Coverage: {:.1}%)",
                contract.baseline_commit.get(..8.min(contract.baseline_commit.len())).unwrap_or(&contract.baseline_commit),
                contract.baseline_tdg,
                contract.baseline_coverage
            );
            run_contract_tests(project_path, &contract, override_claims, ticket, id).await
        }
        Err(e) => {
            anyhow::bail!(
                "Could not load contract for '{}': {}. Re-run 'pmat work start {}' to recreate.",
                item_id, e, id
            );
        }
    }
}

/// Run falsification tests against a loaded contract, produce receipt, gate on result
async fn run_contract_tests(
    project_path: &Path,
    contract: &WorkContract,
    override_claims: &Option<Vec<String>>,
    ticket: &Option<String>,
    id: &str,
) -> Result<()> {
    let report = run_falsification_tests(project_path, contract).await?;

    // Build immutable receipt
    let git_sha = super::work_ledger::get_current_git_sha(project_path);
    let receipt = FalsificationReceipt::from_report(
        &report,
        git_sha,
        contract.work_item_id.clone(),
        FalsificationTrigger::WorkComplete,
        override_claims.as_ref(),
        ticket.as_ref(),
    );

    // Persist receipt and append to global ledger
    let ledger = FalsificationLedger::new(project_path);
    let receipt_path = ledger.persist_receipt(&receipt)?;
    ledger.append_to_ledger(&receipt)?;

    // Gate on receipt summary
    if receipt.summary.allows_completion {
        println!(
            "✅ FALSIFICATION RESULT: PASSED ({}/{} claims validated)",
            receipt.summary.passed, receipt.summary.total
        );
        if receipt.summary.overridden > 0 {
            println!(
                "   ⚠️  {} claim(s) overridden (ticket: {})",
                receipt.summary.overridden,
                ticket.as_deref().unwrap_or("unknown")
            );
        }
        print_warning_failures(&report);
        println!("   📋 Receipt: {}", receipt_path.display());
        println!();
        Ok(())
    } else {
        let failures = report.blocking_failures();
        let unoverrideable = filter_unoverriden_failures(&failures, override_claims.as_ref());
        print_blocked_result(&report, &unoverrideable, id);
        println!("   📋 Receipt: {}", receipt_path.display());
        anyhow::bail!(
            "Work blocked: {} falsification(s) found. Fix issues or use --override-claims with --ticket.",
            unoverrideable.len()
        )
    }
}

/// Update changelog from item labels (helper for handle_work_complete)
fn update_changelog(project_path: &PathBuf, item: &RoadmapItem) {
    if item.labels.is_empty() {
        return;
    }

    if let Some(category) = ChangeCategory::from_labels(&item.labels) {
        let entry = ChangelogEntry::new(category, item.title.clone(), item.github_issue);
        match crate::services::changelog_manager::add_to_changelog(project_path, entry) {
            Ok(()) => println!("✅ Updated CHANGELOG.md"),
            Err(e) => {
                println!("⚠️  Failed to update CHANGELOG.md: {}", e);
                println!("   You may need to update it manually");
            }
        }
    } else {
        println!("ℹ️  No changelog category inferred from labels");
    }
}

/// Print completion next steps with commit metadata (helper for handle_work_complete)
fn print_complete_next_steps(item: &RoadmapItem, id: &str, metadata: &CommitMetadata) {
    println!("🎯 Next steps:");
    let rust_score_line = metadata
        .rust_project_score
        .map(|s| format!("Rust-Score: {:.1}/134\n", s))
        .unwrap_or_default();
    let commit_msg = format!(
        "feat: {} (Refs {})\n\nWork-Item: {}\nTDG-Score: {:.1}/100\nRepo-Score: {:.1}/100\n{}Metrics: .pmat-metrics/commit-*-meta.json",
        item.title, id, item.id, metadata.tdg_score, metadata.repo_score, rust_score_line
    );

    println!("   1. git commit -m \"$(cat <<'EOF'");
    println!("{}", commit_msg);
    println!("EOF");
    println!(")\"");

    if item.is_github_synced() {
        println!(
            "   2. Close GitHub issue: gh issue close {}",
            item.github_issue.expect("internal error")
        );
    }
    println!();
}

/// Auto-commit tracked files modified by `pmat work complete`.
///
/// Prevents the circular dependency where `pmat work complete` creates
/// dirty files that the user must manually commit before pushing.
/// Files committed: docs/roadmaps/roadmap.yaml, CHANGELOG.md (if modified).
fn auto_commit_work_files(
    project_path: &Path,
    item: &RoadmapItem,
    id: &str,
    metadata: &CommitMetadata,
) {
    use std::process::Command;

    // Stage files that pmat work complete may have modified
    let roadmap_path = "docs/roadmaps/roadmap.yaml";
    let changelog_path = "CHANGELOG.md";

    let mut files_to_add = vec![roadmap_path];
    if project_path.join(changelog_path).exists() {
        // Only stage CHANGELOG.md if it has changes
        let status = Command::new("git")
            .args(["diff", "--quiet", "--", changelog_path])
            .current_dir(project_path)
            .status();
        if matches!(status, Ok(s) if !s.success()) {
            files_to_add.push(changelog_path);
        }
    }

    // git add the modified files
    let add_status = Command::new("git")
        .arg("add")
        .args(&files_to_add)
        .current_dir(project_path)
        .status();

    if !matches!(add_status, Ok(s) if s.success()) {
        println!("⚠️  Auto-commit: failed to stage files");
        println!();
        print_complete_next_steps(item, id, metadata);
        return;
    }

    // Build commit message
    let rust_score_line = metadata
        .rust_project_score
        .map(|s| format!("Rust-Score: {:.1}/134\n", s))
        .unwrap_or_default();
    let commit_msg = format!(
        "feat: {} (Refs {})\n\nWork-Item: {}\nTDG-Score: {:.1}/100\nRepo-Score: {:.1}/100\n{}Metrics: .pmat-metrics/commit-*-meta.json",
        item.title, id, item.id, metadata.tdg_score, metadata.repo_score, rust_score_line
    );

    let commit_status = Command::new("git")
        .args(["commit", "-m", &commit_msg, "--no-verify"])
        .current_dir(project_path)
        .status();

    match commit_status {
        Ok(s) if s.success() => {
            println!();
            println!("✅ Auto-committed work completion files");
            if item.is_github_synced() {
                println!(
                    "🎯 Next: gh issue close {}",
                    item.github_issue.expect("internal error")
                );
            }
            println!("🎯 Next: git push origin master");
        }
        _ => {
            println!("⚠️  Auto-commit failed (nothing to commit or hook error)");
            println!();
            print_complete_next_steps(item, id, metadata);
        }
    }
}

/// Handle standalone falsification (does NOT complete the work item)
///
/// Runs the full Popperian falsification protocol and produces a receipt,
/// but does not mark the item as completed or update the roadmap.
/// Use `pmat work complete` to both falsify AND close the item.
pub async fn handle_work_falsify(
    id: String,
    override_claims: Option<Vec<String>>,
    ticket: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    validate_override_accountability(&override_claims, &ticket, &id)?;

    println!("🔬 Running falsification for: {}", id);
    println!();

    run_contract_falsification(&project_path, &id, &override_claims, &ticket, &id).await
}

/// Handle work complete command
///
/// Popperian Falsification Protocol:
/// - Quality gates can be skipped (--skip-quality)
/// - Falsification ALWAYS runs (cannot be skipped)
/// - Overrides require accountability (--override-claims + --ticket)
pub async fn handle_work_complete(
    id: String,
    skip_quality: bool,
    override_claims: Option<Vec<String>>,
    ticket: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    validate_override_accountability(&override_claims, &ticket, &id)?;

    println!("✅ Completing work on: {}", id);
    println!();

    let mut item = service
        .find_item(&id)?
        .with_context(|| format!("Item not found: {}", id))?;

    run_quality_check(&project_path, skip_quality).await?;

    // O(1) freshness check: skip re-running falsification if a fresh receipt exists
    let ledger = FalsificationLedger::new(&project_path);
    let current_sha = super::work_ledger::get_current_git_sha(&project_path);
    if ledger.has_fresh_receipt(&item.id, &current_sha)? {
        println!("✅ Fresh falsification receipt found (matches HEAD {})", &current_sha[..8.min(current_sha.len())]);
        println!("   Skipping re-run (receipt still valid)");
        println!();
    } else {
        run_contract_falsification(&project_path, &item.id, &override_claims, &ticket, &id).await?;
    }

    // Mark as completed
    item.status = ItemStatus::Completed;
    item.updated = chrono::Utc::now().to_rfc3339();

    let mut roadmap = service.load()?;
    roadmap.upsert_item(item.clone());
    service.save(&roadmap)?;

    println!("✅ Marked as complete: {}", item.title);
    println!("✅ Updated roadmap: {}", roadmap_path.display());

    // Capture commit metadata
    println!();
    println!("   📊 Capturing commit metadata...");
    let metadata = capture_commit_metadata(&project_path, &item).await?;
    println!("      ✅ TDG Score: {:.1}/100", metadata.tdg_score);
    println!("      ✅ Repo Score: {:.1}/100", metadata.repo_score);
    if let Some(rust_score) = metadata.rust_project_score {
        println!("      ✅ Rust Project Score: {:.1}/134", rust_score);
    }
    let meta_file = project_path.join(".pmat-metrics").join("commit-*-meta.json");
    println!("✅ Commit metadata: {}", meta_file.display());

    update_changelog(&project_path, &item);

    // Auto-commit modified tracked files to prevent circular dependency (#223)
    // pmat work complete modifies roadmap.yaml and optionally CHANGELOG.md,
    // which leaves the working tree dirty. Auto-committing prevents the user
    // from having to manually commit these pmat-generated changes.
    auto_commit_work_files(&project_path, &item, &id, &metadata);

    Ok(())
}

/// Handle work status command
pub async fn handle_work_status(
    id: Option<String>,
    path: Option<PathBuf>,
    active: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    let roadmap = service.load()?;

    if let Some(item_id) = id {
        // Show specific item
        let item = roadmap
            .find_item(&item_id)
            .with_context(|| format!("Item not found: {}", item_id))?;

        println!("📊 Status for: {}", item.id);
        println!();
        println!("   Title: {}", item.title);
        println!("   Status: {:?}", item.status);
        println!("   Priority: {:?}", item.priority);
        println!("   Progress: {}%", item.completion_percentage());
        if let Some(gh) = item.github_issue {
            println!("   GitHub: #{}", gh);
        }
        println!();
    } else {
        // Show all items
        let items: Vec<_> = if active {
            roadmap
                .roadmap
                .iter()
                .filter(|item| {
                    matches!(
                        item.status,
                        ItemStatus::InProgress | ItemStatus::Planned | ItemStatus::Blocked
                    )
                })
                .collect()
        } else {
            roadmap.roadmap.iter().collect()
        };

        if items.is_empty() {
            println!("📋 No items found");
            println!();
            println!("   Start work with: pmat work start <id>");
            return Ok(());
        }

        println!("📋 Roadmap items: {} total", items.len());
        println!();

        for item in items {
            let emoji = match item.status {
                ItemStatus::Completed => "✅",
                ItemStatus::InProgress => "⏳",
                ItemStatus::Planned => "📋",
                ItemStatus::Blocked => "🚫",
                ItemStatus::Review => "👀",
                ItemStatus::Cancelled => "❌",
            };

            let progress = item.completion_percentage();

            // Truncate long IDs for display (show first 30 chars + "...")
            // Use chars() to avoid Unicode boundary panics (issue #128)
            let display_id = if item.id.chars().count() > 30 {
                format!("{}...", item.id.chars().take(30).collect::<String>())
            } else {
                item.id.clone()
            };

            println!(
                "   {} [{}] {} ({}%)",
                emoji, display_id, item.title, progress
            );
            if item.is_github_synced() {
                println!(
                    "      GitHub: #{}",
                    item.github_issue.expect("internal error")
                );
            }
        }
        println!();
    }

    Ok(())
}

/// Handle work sync command
pub async fn handle_work_sync(
    direction: SyncDirection,
    path: Option<PathBuf>,
    dry_run: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    let action = if dry_run { "Dry run" } else { "Syncing" };
    println!("🔄 {} roadmap...", action);
    println!();

    let roadmap = service.load()?;

    match direction {
        SyncDirection::YamlToGithub => {
            println!("📤 Direction: YAML → GitHub");
            let yaml_only = roadmap.yaml_only_items();
            println!("   Found {} YAML-only items", yaml_only.len());
            for item in yaml_only {
                println!("      - {} ({})", item.id, item.title);
            }
            println!();
            println!("   ⚠️  GitHub sync not yet implemented");
        }
        SyncDirection::GithubToYaml => {
            println!("📥 Direction: GitHub → YAML");
            println!("   ⚠️  GitHub sync not yet implemented");
        }
        SyncDirection::Full => {
            println!("🔄 Direction: Full bidirectional sync");
            println!("   ⚠️  GitHub sync not yet implemented");
        }
    }

    println!();
    Ok(())
}

/// Minimal issue info for API-agnostic GitHub operations
/// Works with either octocrab (github-api feature) or gh CLI fallback
#[derive(Debug, Clone)]
pub struct GitHubIssueInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub labels: Vec<String>,
}

/// Fetch GitHub issue details using octocrab (requires github-api feature)
#[cfg(feature = "github-api")]
async fn fetch_github_issue(repo: &str, issue_num: u64) -> Result<GitHubIssueInfo> {
    // Try authenticated client first, fall back to unauthenticated
    let client = match GitHubClient::new(repo) {
        Ok(c) => c,
        Err(_) => {
            // GITHUB_TOKEN not set, try unauthenticated
            GitHubClient::new_unauthenticated(repo)?
        }
    };

    let issue = client.fetch_issue(issue_num).await?;
    Ok(GitHubIssueInfo {
        number: issue.number,
        title: issue.title,
        body: issue.body,
        labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
    })
}

/// Fetch GitHub issue details using gh CLI (no octocrab dependency)
#[cfg(not(feature = "github-api"))]
async fn fetch_github_issue(repo: &str, issue_num: u64) -> Result<GitHubIssueInfo> {
    use std::process::Command;

    let output = Command::new("gh")
        .args([
            "issue",
            "view",
            &issue_num.to_string(),
            "--repo",
            repo,
            "--json",
            "number,title,body,labels",
        ])
        .output()
        .context("Failed to run gh CLI. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue view failed: {}", stderr);
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse gh output")?;

    Ok(GitHubIssueInfo {
        number: json["number"].as_u64().unwrap_or(issue_num),
        title: json["title"].as_str().unwrap_or("").to_string(),
        body: json["body"].as_str().map(|s| s.to_string()),
        labels: json["labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// Create GitHub issue from roadmap item using octocrab (requires github-api feature)
#[cfg(feature = "github-api")]
async fn create_github_issue_from_item(repo: &str, item: &RoadmapItem) -> Result<GitHubIssueInfo> {
    // Requires authentication
    let client = GitHubClient::new(repo)?;

    // Build issue body from acceptance criteria
    let body = if !item.acceptance_criteria.is_empty() {
        let criteria_md: Vec<String> = item
            .acceptance_criteria
            .iter()
            .map(|c| format!("- [ ] {}", c))
            .collect();

        format!(
            "## Acceptance Criteria\n\n{}\n\n---\n\n*Created via `pmat work start --create-github`*",
            criteria_md.join("\n")
        )
    } else {
        "*Created via `pmat work start --create-github`*".to_string()
    };

    let labels = if item.labels.is_empty() {
        None
    } else {
        Some(item.labels.clone())
    };

    let issue = client.create_issue(&item.title, &body, labels).await?;
    Ok(GitHubIssueInfo {
        number: issue.number,
        title: issue.title,
        body: issue.body,
        labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
    })
}

/// Create GitHub issue from roadmap item using gh CLI (no octocrab dependency)
#[cfg(not(feature = "github-api"))]
async fn create_github_issue_from_item(repo: &str, item: &RoadmapItem) -> Result<GitHubIssueInfo> {
    use std::process::Command;

    // Build issue body from acceptance criteria
    let body = if !item.acceptance_criteria.is_empty() {
        let criteria_md: Vec<String> = item
            .acceptance_criteria
            .iter()
            .map(|c| format!("- [ ] {}", c))
            .collect();

        format!(
            "## Acceptance Criteria\n\n{}\n\n---\n\n*Created via `pmat work start --create-github`*",
            criteria_md.join("\n")
        )
    } else {
        "*Created via `pmat work start --create-github`*".to_string()
    };

    let mut args = vec![
        "issue".to_string(),
        "create".to_string(),
        "--repo".to_string(),
        repo.to_string(),
        "--title".to_string(),
        item.title.clone(),
        "--body".to_string(),
        body.clone(),
    ];

    // Add labels if present
    for label in &item.labels {
        args.push("--label".to_string());
        args.push(label.clone());
    }

    let output = Command::new("gh")
        .args(&args)
        .output()
        .context("Failed to run gh CLI. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue create failed: {}", stderr);
    }

    // gh issue create outputs the URL, parse the issue number from it
    let stdout = String::from_utf8_lossy(&output.stdout);
    let issue_num: u64 = stdout
        .trim()
        .rsplit('/')
        .next()
        .and_then(|s| s.parse().ok())
        .context("Failed to parse issue number from gh output")?;

    Ok(GitHubIssueInfo {
        number: issue_num,
        title: item.title.clone(),
        body: Some(body),
        labels: item.labels.clone(),
    })
}

/// Parse acceptance criteria from GitHub issue body
///
/// Looks for markdown checklists in the body and extracts them as criteria.
fn parse_acceptance_criteria(body: &str) -> Vec<String> {
    let mut criteria = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        // Match markdown checkboxes: - [ ] or - [x]
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
            let criterion = trimmed
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim()
                .to_string();
            if !criterion.is_empty() {
                criteria.push(criterion);
            }
        }
    }

    criteria
}

/// Detect GitHub repository from git remote
fn detect_github_repo(project_path: &PathBuf) -> Result<Option<String>> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(project_path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout);
            let url = url.trim();

            // Parse GitHub URL
            // https://github.com/owner/repo.git or git@github.com:owner/repo.git
            if let Some(repo) = parse_github_url(url) {
                return Ok(Some(repo));
            }
        }
    }

    Ok(None)
}

/// Parse GitHub URL to extract owner/repo
fn parse_github_url(url: &str) -> Option<String> {
    // HTTPS: https://github.com/owner/repo.git
    if let Some(start) = url.find("github.com/") {
        let rest = url.get(start + 11..).unwrap_or_default();
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    // SSH: git@github.com:owner/repo.git
    if let Some(start) = url.find("github.com:") {
        let rest = url.get(start + 11..).unwrap_or_default();
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    None
}

/// Create specification template
fn create_specification_template(spec_path: &PathBuf, item: &RoadmapItem) -> Result<()> {
    use std::fs;

    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let github_link = if let Some(issue) = item.github_issue {
        format!(
            "**GitHub Issue**: [#{}](https://github.com/YOUR_ORG/YOUR_REPO/issues/{})",
            issue, issue
        )
    } else {
        format!("**Ticket ID**: {}", item.id)
    };

    let template = format!(
        r#"---
title: {}
issue: {}
status: In Progress
created: {}
updated: {}
---

# {} Specification

{}
**Status**: In Progress

## Summary

[Brief 2-3 sentence overview of what this work accomplishes]

## Requirements

### Functional Requirements
- [ ] Requirement 1
- [ ] Requirement 2

### Non-Functional Requirements
- [ ] Performance: [specific target]
- [ ] Test coverage: ≥85%

## Architecture

### Design Overview

[Describe the high-level design approach]

### API Design

```rust
// Example API design
pub struct Example {{
    // ...
}}
```

## Implementation Plan

### Phase 1: Foundation
- [ ] Task 1
- [ ] Task 2

### Phase 2: Core Implementation
- [ ] Task 3
- [ ] Task 4

## Testing Strategy

### Unit Tests
- [ ] Test case 1
- [ ] Test case 2

### Integration Tests
- [ ] Integration test 1

## Success Criteria

- ✅ All acceptance criteria met
- ✅ Test coverage ≥85%
- ✅ Zero clippy warnings
- ✅ Documentation complete

## References

- [Related documentation]
"#,
        item.title, item.id, item.created, item.updated, item.title, github_link
    );

    fs::write(spec_path, template)?;
    Ok(())
}

/// Convert hypothesis text to CLI-friendly override name
///
/// Maps the verbose hypothesis strings from FalsifiableClaim to short,
/// CLI-friendly names that users can specify with --override-claims.
/// Hypothesis pattern to CLI override name mapping (CB-040 complexity refactor)
const CLAIM_PATTERNS: &[(&[&str], &str)] = &[
    (&["manifest", "files deleted"], "manifest"),
    (&["meta-falsification", "falsification system"], "meta-falsification"),
    (&["coverage gaming", "coverage exclusion", "cfg(not(coverage))"], "coverage-gaming"),
    (&["differential coverage", "new code", "changed lines"], "differential-coverage"),
    (&["total coverage", "absolute coverage", "coverage does not decrease", "coverage >= 95"], "coverage"),
    (&["tdg", "test-driven grade"], "tdg"),
    (&["complexity", "cyclomatic"], "complexity"),
    (&["supply chain", "dependencies", "vulnerable dependencies"], "supply-chain"),
    (&["file size", "500 lines"], "file-size"),
    (&["spec", "specification"], "spec-quality"),
    (&["github", "sync", "changes pushed", "uncommitted"], "github-sync"),
    (&["examples", "compile"], "examples"),
    (&["book", "pmat-book"], "book"),
    (&["satd", "todo/fixme/hack"], "satd"),
    (&["dead code introduced", "dead code detected"], "dead-code"),
    (&["per-file coverage", "files have >= 95%", "all files have"], "per-file-coverage"),
    (&["lint passes", "make lint"], "lint"),
    // v3.1 defect churn prevention
    (&["match arm", "variant"], "variant-coverage"),
    (&["fix-after-fix", "fix chain"], "fix-chain"),
    (&["cross-crate", "sibling project", "integration tests pass"], "cross-crate"),
    (&["regression", "performance"], "regression-gate"),
];

fn claim_to_override_name(hypothesis: &str) -> String {
    let hypothesis_lower = hypothesis.to_lowercase();

    // Pattern-based lookup (reduces cyclomatic complexity vs if-else chain)
    for (patterns, name) in CLAIM_PATTERNS {
        if patterns.iter().any(|p| hypothesis_lower.contains(p)) {
            return name.to_string();
        }
    }

    // Unknown claim - use a sanitized version of the hypothesis
    hypothesis_lower
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(30)
        .collect()
}

