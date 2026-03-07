#![cfg_attr(coverage_nightly, coverage(off))]
// Work item resolution helpers (GitHub issues and YAML tickets)

use crate::models::roadmap::{ItemStatus, Priority, RoadmapItem};
use crate::services::roadmap_service::RoadmapService;
use anyhow::Result;

use super::github::{create_github_issue_from_item, fetch_github_issue};
use super::helpers::parse_acceptance_criteria;
use crate::cli::handlers::work_falsification::{ClaimResult, FalsificationReport};

/// Resolve a GitHub issue into a RoadmapItem (helper for handle_work_start)
pub(super) async fn resolve_github_issue(
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
pub(super) async fn resolve_yaml_ticket(
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
pub(super) async fn try_create_github_issue(
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

/// Print warning failures (non-blocking)
pub(super) fn print_warning_failures(report: &FalsificationReport) {
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
pub(super) fn print_blocked_result(
    report: &FalsificationReport,
    unoverrideable: &[&ClaimResult],
    id: &str,
) {
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
    println!(
        "  2. pmat work complete {} --override-claims coverage,complexity --ticket DEBT-XXX",
        id
    );
}
