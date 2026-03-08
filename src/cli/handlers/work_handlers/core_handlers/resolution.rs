#![cfg_attr(coverage_nightly, coverage(off))]
// Work item resolution helpers (GitHub issues and YAML tickets)

use crate::cli::colors as c;
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
    println!(
        "{} Type: GitHub issue #{}",
        c::label("📋"),
        c::number(&issue_num.to_string())
    );

    let mut item = if let Some(ref repo) = roadmap.github_repo {
        match fetch_github_issue(repo, issue_num).await {
            Ok(gh_issue) => {
                println!(
                    "   {}",
                    c::pass(&format!("Fetched from GitHub: {}", gh_issue.title))
                );
                let mut item = RoadmapItem::from_github_issue(issue_num, gh_issue.title.clone());
                item.labels = gh_issue.labels.clone();
                if let Some(body) = &gh_issue.body {
                    item.acceptance_criteria = parse_acceptance_criteria(body);
                }
                item
            }
            Err(e) => {
                println!(
                    "   {}",
                    c::warn(&format!("Failed to fetch from GitHub: {}", e))
                );
                println!("   {}", c::dim("Creating placeholder (will sync later)"));
                RoadmapItem::from_github_issue(issue_num, format!("Issue #{}", issue_num))
            }
        }
    } else {
        println!(
            "   ℹ️  {}",
            c::dim("GitHub not configured, creating placeholder")
        );
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
    println!("{} Type: YAML ticket {}", c::label("📋"), c::path(id));

    if let Some(existing) = service.find_item(id)? {
        println!("   {}", c::pass("Found existing ticket"));
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
        println!("   {} Creating GitHub issue...", c::label("🔄"));
        match create_github_issue_from_item(repo, item).await {
            Ok(gh_issue) => {
                println!(
                    "   {}",
                    c::pass(&format!(
                        "Created GitHub issue #{}",
                        c::number(&gh_issue.number.to_string())
                    ))
                );
                item.github_issue = Some(gh_issue.number);
                item.id = format!("GH-{}", gh_issue.number);
            }
            Err(e) => {
                println!(
                    "   {}",
                    c::warn(&format!("Failed to create GitHub issue: {}", e))
                );
                println!("   {}", c::dim("Continuing with YAML-only ticket"));
            }
        }
    } else {
        println!(
            "   {}",
            c::warn("GitHub not configured, skipping issue creation")
        );
    }
}

/// Print warning failures (non-blocking)
pub(super) fn print_warning_failures(report: &FalsificationReport) {
    let warnings = report.warning_failures();
    if !warnings.is_empty() {
        println!();
        println!("{}", c::subheader("Warnings (non-blocking):"));
        for warning in warnings {
            println!(
                "  - [{}] {}{}{}: {}",
                c::number(&warning.index.to_string()),
                c::YELLOW,
                warning.hypothesis,
                c::RESET,
                warning.result.explanation
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
        "{}❌ FALSIFICATION RESULT: BLOCKED{} ({} failure(s), {} warning(s))",
        c::BOLD_RED,
        c::RESET,
        c::number(&report.failed.to_string()),
        c::number(&report.warnings.to_string())
    );
    println!();
    println!("{}", c::subheader("Failures (must fix):"));
    for failure in unoverrideable {
        println!(
            "  - [{}] {}{}{}: {}",
            c::number(&failure.index.to_string()),
            c::RED,
            failure.hypothesis,
            c::RESET,
            failure.result.explanation
        );
    }

    print_warning_failures(report);

    println!();
    println!("Fix issues and retry: pmat work complete {}", id);
    println!();
    println!(
        "{}",
        c::dim("Or override with accountability (Popperian Protocol):")
    );
    println!(
        "  {}",
        c::dim("1. Create debt ticket: pmat comply upgrade --target popperian")
    );
    println!(
        "  {}",
        c::dim(&format!(
            "2. pmat work complete {} --override-claims coverage,complexity --ticket DEBT-XXX",
            id
        ))
    );
}
