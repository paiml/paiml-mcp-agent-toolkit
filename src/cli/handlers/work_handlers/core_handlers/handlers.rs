#![cfg_attr(coverage_nightly, coverage(off))]
// Public handler functions for work commands

use crate::cli::colors as c;
use crate::cli::commands::SyncDirection;
use crate::models::roadmap::ItemStatus;
use crate::services::hook_manager;
use crate::services::roadmap_service::RoadmapService;
use anyhow::{Context, Result};
use std::path::PathBuf;

use super::commit::{auto_commit_work_files, capture_commit_metadata, update_changelog};
use super::contract::{create_work_contract, run_contract_falsification, run_quality_check};
use super::github::detect_github_repo;
use super::helpers::{create_specification_template, validate_override_accountability};
use super::resolution::{resolve_github_issue, resolve_yaml_ticket};
use crate::cli::handlers::work_ledger::FalsificationLedger;

/// Handle work init command
pub async fn handle_work_init(
    github_repo: Option<String>,
    no_github: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!(
        "{}",
        c::label("🚀 Initializing unified GitHub/YAML workflow...")
    );
    println!();

    // Create roadmap service
    let service = RoadmapService::new(&roadmap_path);

    // Check if already initialized
    if service.exists() {
        println!(
            "{}",
            c::warn(&format!(
                "Roadmap already exists at: {}",
                c::path(&roadmap_path.display().to_string())
            ))
        );
        println!(
            "   {}",
            c::dim("Use `pmat work status` to view current items")
        );
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

    println!(
        "{}",
        c::pass(&format!(
            "Created roadmap: {}",
            c::path(&roadmap_path.display().to_string())
        ))
    );

    // Install commit-msg hook
    match hook_manager::install_commit_msg_hook(&project_path) {
        Ok(()) => {
            println!("{}", c::pass("Installed commit-msg hook"));
        }
        Err(e) => {
            println!(
                "{}",
                c::warn(&format!("Failed to install commit-msg hook: {}", e))
            );
            println!(
                "   {}",
                c::dim("Workflow will work, but commit messages won't be validated")
            );
        }
    }

    println!();

    // Display configuration
    println!("{}", c::subheader("📋 Configuration:"));
    println!(
        "   GitHub integration: {}",
        if github_enabled {
            format!("{}✅ enabled{}", c::GREEN, c::RESET)
        } else {
            format!("{}❌ disabled{}", c::RED, c::RESET)
        }
    );
    if let Some(r) = &repo {
        println!("   GitHub repository: {}", c::path(r));
    }
    println!();

    // Next steps
    println!("{}", c::subheader("🎯 Next steps:"));
    println!("   1. Create GitHub issue or edit roadmap.yaml");
    println!("   2. Start work: pmat work start <issue-number-or-ticket-id>");
    println!("   3. Continue: pmat work continue <id>");
    println!("   4. Complete: pmat work complete <id>");
    println!();

    if github_enabled && repo.is_none() {
        println!("{}", c::dim("💡 Tip: Set GitHub repo with:"));
        println!("   {}", c::dim("pmat config set github.repo owner/repo"));
        println!();
    }

    Ok(())
}

/// Handle work start command
#[allow(clippy::too_many_arguments)]
pub async fn handle_work_start(
    id: String,
    with_spec: bool,
    epic: bool,
    path: Option<PathBuf>,
    create_github: bool,
    profile_override: Option<String>,
    without: Vec<String>,
    iteration: u32,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    println!(
        "{}",
        c::label(&format!("🚀 Starting work on: {}", c::path(&id)))
    );
    println!();

    let mut roadmap = service.load()?;

    // §11.4: Warn if another work item is already in-progress
    let active_items: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| matches!(item.status, ItemStatus::InProgress) && item.id != id)
        .collect();
    if !active_items.is_empty() {
        println!(
            "{}",
            c::warn(&format!(
                "{} other item(s) already in-progress:",
                active_items.len()
            ))
        );
        for item in &active_items {
            println!("   - {} ({})", c::path(&item.id), item.title);
        }
        println!(
            "   {}",
            c::dim("Consider completing them first with `pmat work complete`.")
        );
        println!();
    }

    let is_github_issue = id.parse::<u64>().is_ok();

    let mut item = if is_github_issue {
        let issue_num: u64 = id.parse()?;
        resolve_github_issue(&roadmap, issue_num).await
    } else {
        resolve_yaml_ticket(&service, &id, &roadmap, create_github).await?
    };

    if epic {
        item.item_type = crate::models::roadmap::ItemType::Epic;
        println!("{} Created as epic: {}", c::label("📦"), item.title);
        println!(
            "   {}",
            c::dim("Add subtasks manually to roadmap.yaml or use future commands")
        );
    }

    roadmap.upsert_item(item.clone());
    service.save(&roadmap)?;
    println!(
        "{}",
        c::pass(&format!(
            "Updated roadmap: {}",
            c::path(&roadmap_path.display().to_string())
        ))
    );

    create_work_contract(
        &project_path,
        &item.id,
        profile_override.as_deref(),
        &without,
        iteration,
    )
    .await;

    if with_spec {
        create_spec_if_needed(&project_path, &item, &id, is_github_issue)?;
    }

    print_work_start_next_steps(&id);
    Ok(())
}

/// Create specification file if it does not exist (helper for handle_work_start)
fn create_spec_if_needed(
    project_path: &std::path::Path,
    item: &crate::models::roadmap::RoadmapItem,
    id: &str,
    is_github_issue: bool,
) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
        println!(
            "{}",
            c::pass(&format!(
                "Created specification: {}",
                c::path(&spec_path.display().to_string())
            ))
        );
    } else {
        println!(
            "   Specification exists: {}",
            c::path(&spec_path.display().to_string())
        );
    }
    Ok(())
}

/// Print next steps after work start (helper for handle_work_start)
fn print_work_start_next_steps(id: &str) {
    println!();
    println!("{}", c::subheader("🎯 Next steps:"));
    println!("   1. Review specification (if created)");
    println!(
        "   2. Write failing tests ({}RED{} phase)",
        c::RED,
        c::RESET
    );
    println!(
        "   3. Implement feature ({}GREEN{} phase)",
        c::GREEN,
        c::RESET
    );
    println!("   4. Refactor ({}REFACTOR{} phase)", c::YELLOW, c::RESET);
    println!("   5. Continue: pmat work continue {}", id);
    println!("   6. Complete: pmat work complete {}", id);
    println!();
}

/// Handle work continue command
pub async fn handle_work_continue(id: String, path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    println!(
        "{}",
        c::label(&format!("🔄 Continuing work on: {}", c::path(&id)))
    );
    println!();

    // Find item
    let item = service
        .find_item(&id)?
        .with_context(|| format!("Item not found: {}", id))?;

    // Display progress
    let completion = item.completion_percentage();
    println!(
        "{} Progress: {}% complete",
        c::subheader("📊"),
        c::number(&completion.to_string())
    );
    println!("   {} {:?}", c::label("Status:"), item.status);
    println!("   {} {}", c::label("Title:"), item.title);
    if let Some(spec) = &item.spec {
        println!(
            "   {} {}",
            c::label("Spec:"),
            c::path(&spec.display().to_string())
        );
    }
    println!();

    // Show acceptance criteria
    if !item.acceptance_criteria.is_empty() {
        println!("{}", c::subheader("📋 Acceptance Criteria:"));
        for (i, criterion) in item.acceptance_criteria.iter().enumerate() {
            println!("   {}. {}", c::number(&(i + 1).to_string()), criterion);
        }
        println!();
    }

    // Show phases
    if !item.phases.is_empty() {
        println!("{}", c::subheader("📌 Phases:"));
        for phase in &item.phases {
            let emoji = match phase.status {
                ItemStatus::Completed => format!("{}✅{}", c::GREEN, c::RESET),
                ItemStatus::InProgress => "⏳".to_string(),
                _ => format!("{}⬜{}", c::DIM, c::RESET),
            };
            println!(
                "   {} {} ({}%)",
                emoji,
                phase.name,
                c::number(&phase.completion.to_string())
            );
        }
        println!();
    }

    // Show subtasks (for epics)
    if !item.subtasks.is_empty() {
        println!("{}", c::subheader("📦 Subtasks:"));
        for subtask in &item.subtasks {
            let emoji = match subtask.status {
                ItemStatus::Completed => format!("{}✅{}", c::GREEN, c::RESET),
                ItemStatus::InProgress => "⏳".to_string(),
                _ => format!("{}⬜{}", c::DIM, c::RESET),
            };
            println!(
                "   {} {} ({}%)",
                emoji,
                subtask.title,
                c::number(&subtask.completion.to_string())
            );
        }
        println!();
    }

    // Next steps
    println!("{}", c::subheader("🎯 Next steps:"));
    println!("   Continue working on: {}", item.title);
    println!("   When done: pmat work complete {}", id);
    println!();

    Ok(())
}

/// Handle work checkpoint command (DbC §4.2 — invariant evaluation)
///
/// Evaluates all invariant clauses from the contract at the current point in time.
/// Results are persisted to `.pmat-work/{id}/checkpoints/` for audit trail.
/// Invariant failures are reported but do not halt work — they accumulate
/// and block completion at `work complete`.
pub async fn handle_work_checkpoint(id: String, path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    println!(
        "{}",
        c::label(&format!(
            "🔍 Running invariant checkpoint for: {}",
            c::path(&id)
        ))
    );
    println!();

    let record = super::checkpoint::run_checkpoint(&project_path, &id)?;

    // Display results
    if record.invariant_results.is_empty() {
        println!(
            "   ℹ️  {}",
            c::dim("No invariant clauses in contract (v4.0 or no invariants defined)")
        );
        println!(
            "   {}",
            c::dim("Use --profile rust or --profile pmat for invariant checking")
        );
        println!();
        return Ok(());
    }

    for result in &record.invariant_results {
        let emoji = if result.passed {
            format!("{}✓{}", c::GREEN, c::RESET)
        } else {
            format!("{}✗{}", c::RED, c::RESET)
        };
        println!(
            "  [{}] {} {}",
            c::label(&result.clause_id),
            emoji,
            result.explanation
        );
    }
    println!();

    let checkpoint_path = record.save(&project_path)?;

    if record.all_invariants_hold {
        println!(
            "{}",
            c::pass(&format!(
                "All invariants hold. Checkpoint recorded. ({}/{})",
                record.invariant_results.len(),
                record.invariant_results.len()
            ))
        );
    } else {
        let failed_count = record
            .invariant_results
            .iter()
            .filter(|r| !r.passed)
            .count();
        println!(
            "{}",
            c::warn(&format!(
                "{} invariant(s) violated. Fix before completion.",
                failed_count
            ))
        );
    }

    // Display drift bound if available (DBC spec §13.5)
    if let Some(drift_bound) = record.drift_bound {
        println!(
            "   {} {}  |  {} {}  |  {} {}",
            c::label("Iteration:"),
            c::number(&record.iteration.to_string()),
            c::label("Git SHA:"),
            c::dim(&record.git_sha[..8.min(record.git_sha.len())]),
            c::label("Drift:"),
            c::number(&format!("{:.2}", drift_bound))
        );
    } else {
        println!(
            "   {} {}  |  {} {}",
            c::label("Iteration:"),
            c::number(&record.iteration.to_string()),
            c::label("Git SHA:"),
            c::dim(&record.git_sha[..8.min(record.git_sha.len())])
        );
    }
    println!(
        "   {} {}",
        c::label("Checkpoint:"),
        c::path(&checkpoint_path.display().to_string())
    );
    println!();

    Ok(())
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

    println!(
        "{}",
        c::label(&format!("🔬 Running falsification for: {}", c::path(&id)))
    );
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

    println!(
        "{}",
        c::pass(&format!("Completing work on: {}", c::path(&id)))
    );
    println!();

    let mut item = service
        .find_item(&id)?
        .with_context(|| format!("Item not found: {}", id))?;

    // DBC §work_lifecycle: Validate state transition (Planned→Completed is INVALID)
    if !item.status.can_transition_to(ItemStatus::Completed) {
        anyhow::bail!(
            "Invalid transition: {} → Completed. \
             Item must be InProgress or Review to complete. \
             Current status: {}. Run 'pmat work start {}' first.",
            item.status.display_name(),
            item.status.display_name(),
            id
        );
    }

    run_quality_check(&project_path, skip_quality).await?;

    // DbC §4.3: Final invariant check before postcondition evaluation
    run_final_invariant_check(&project_path, &item.id)?;

    // O(1) freshness check: skip re-running falsification if a fresh receipt exists
    let ledger = FalsificationLedger::new(&project_path);
    let current_sha = crate::cli::handlers::work_ledger::get_current_git_sha(&project_path);
    if ledger.has_fresh_receipt(&item.id, &current_sha)? {
        println!(
            "{}",
            c::pass(&format!(
                "Fresh falsification receipt found (matches HEAD {})",
                c::dim(&current_sha[..8.min(current_sha.len())])
            ))
        );
        println!("   {}", c::dim("Skipping re-run (receipt still valid)"));
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

    println!(
        "{}",
        c::pass(&format!("Marked as complete: {}", item.title))
    );
    println!(
        "{}",
        c::pass(&format!(
            "Updated roadmap: {}",
            c::path(&roadmap_path.display().to_string())
        ))
    );

    // Capture commit metadata
    println!();
    println!("   {} Capturing commit metadata...", c::subheader("📊"));
    let metadata = capture_commit_metadata(&project_path, &item).await?;
    println!(
        "      {} TDG Score: {}",
        c::pass(""),
        c::number(&format!("{:.1}/100", metadata.tdg_score))
    );
    println!(
        "      {} Repo Score: {}",
        c::pass(""),
        c::number(&format!("{:.1}/100", metadata.repo_score))
    );
    if let Some(rust_score) = metadata.rust_project_score {
        println!(
            "      {} Rust Project Score: {}",
            c::pass(""),
            c::number(&format!("{:.1}/134", rust_score))
        );
    }
    let meta_file = project_path
        .join(".pmat-metrics")
        .join("commit-*-meta.json");
    println!(
        "{}",
        c::pass(&format!(
            "Commit metadata: {}",
            c::path(&meta_file.display().to_string())
        ))
    );

    // DBC spec §13.4: Final contract scoring
    if let Ok(contract) =
        crate::cli::handlers::work_contract::WorkContract::load(&project_path, &item.id)
    {
        let score = crate::cli::handlers::work_contract::score_contract(&contract, &project_path);
        println!(
            "   {} {:.2} ({})",
            c::label("Contract Score:"),
            c::number(&format!("{:.2}", score.total)),
            c::grade(&score.grade.to_string())
        );
    }

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

        println!("{} Status for: {}", c::subheader("📊"), c::path(&item.id));
        println!();
        println!("   {} {}", c::label("Title:"), item.title);
        println!("   {} {:?}", c::label("Status:"), item.status);
        println!("   {} {:?}", c::label("Priority:"), item.priority);
        println!(
            "   {} {}%",
            c::label("Progress:"),
            c::number(&item.completion_percentage().to_string())
        );
        if let Some(gh) = item.github_issue {
            println!("   {} #{}", c::label("GitHub:"), c::number(&gh.to_string()));
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
            println!("{} No items found", c::subheader("📋"));
            println!();
            println!("   {}", c::dim("Start work with: pmat work start <id>"));
            return Ok(());
        }

        println!(
            "{} Roadmap items: {} total",
            c::subheader("📋"),
            c::number(&items.len().to_string())
        );
        println!();

        for item in items {
            let emoji = match item.status {
                ItemStatus::Completed => format!("{}✅{}", c::GREEN, c::RESET),
                ItemStatus::InProgress => "⏳".to_string(),
                ItemStatus::Planned => "📋".to_string(),
                ItemStatus::Blocked => format!("{}🚫{}", c::RED, c::RESET),
                ItemStatus::Review => "👀".to_string(),
                ItemStatus::Cancelled => format!("{}❌{}", c::RED, c::RESET),
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
                emoji,
                c::path(&display_id),
                item.title,
                c::number(&progress.to_string())
            );
            if item.is_github_synced() {
                println!(
                    "      {} #{}",
                    c::label("GitHub:"),
                    c::number(&item.github_issue.expect("internal error").to_string())
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
    println!("{}", c::label(&format!("🔄 {} roadmap...", action)));
    println!();

    let roadmap = service.load()?;

    match direction {
        SyncDirection::YamlToGithub => {
            println!("{}", c::subheader("📤 Direction: YAML → GitHub"));
            let yaml_only = roadmap.yaml_only_items();
            println!(
                "   Found {} YAML-only items",
                c::number(&yaml_only.len().to_string())
            );
            for item in yaml_only {
                println!("      - {} ({})", c::path(&item.id), item.title);
            }
            println!();
            println!("   {}", c::warn("GitHub sync not yet implemented"));
        }
        SyncDirection::GithubToYaml => {
            println!("{}", c::subheader("📥 Direction: GitHub → YAML"));
            println!("   {}", c::warn("GitHub sync not yet implemented"));
        }
        SyncDirection::Full => {
            println!("{}", c::subheader("🔄 Direction: Full bidirectional sync"));
            println!("   {}", c::warn("GitHub sync not yet implemented"));
        }
    }

    println!();
    Ok(())
}

/// Run final invariant check before completion (DbC §4.3).
///
/// Evaluates all invariant clauses and persists a checkpoint record.
/// If any invariant fails, completion is blocked.
fn run_final_invariant_check(project_path: &std::path::Path, item_id: &str) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use crate::cli::handlers::work_contract::WorkContract;

    // Only run invariant check for v5.0 contracts with invariant clauses
    if let Ok(contract) = WorkContract::load(project_path, item_id) {
        // §12.1: Contract quality check — warn if too many claims excluded
        if let Some(quality) = &contract.contract_quality {
            let pct = quality.score * 100.0;
            if pct < 50.0 {
                println!(
                    "{}",
                    c::warn(&format!(
                        "Contract quality LOW: {} ({}) — {}/{} claims active",
                        c::number(&format!("{:.0}%", pct)),
                        quality.rating,
                        quality.active_claims,
                        quality.applicable_claims
                    ))
                );
                println!(
                    "   {}",
                    c::dim("Consider removing --without exclusions for stronger guarantees.")
                );
                println!();
            }
        }

        if contract.is_dbc() && !contract.invariant.is_empty() {
            println!("{}", c::label("🔍 Evaluating invariants (final check)..."));

            let (results, all_pass) =
                super::checkpoint::evaluate_final_invariants(project_path, &contract);

            for result in &results {
                let emoji = if result.passed {
                    format!("{}✓{}", c::GREEN, c::RESET)
                } else {
                    format!("{}✗{}", c::RED, c::RESET)
                };
                println!(
                    "  [{}] {} {}",
                    c::label(&result.clause_id),
                    emoji,
                    result.explanation
                );
            }

            if all_pass {
                println!(
                    "   {}",
                    c::pass(&format!(
                        "All invariants hold ({}/{})",
                        results.len(),
                        results.len()
                    ))
                );
                println!();
            } else {
                let failed: Vec<_> = results.iter().filter(|r| !r.passed).collect();
                println!();
                anyhow::bail!(
                    "Invariant violation: {} invariant(s) failed. Fix before completing.\n  {}",
                    failed.len(),
                    failed
                        .iter()
                        .map(|r| format!("{}: {}", r.clause_id, r.explanation))
                        .collect::<Vec<_>>()
                        .join("\n  ")
                );
            }
        }
    }
    Ok(())
}
