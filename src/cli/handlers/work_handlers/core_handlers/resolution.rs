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
        // DBC §work_lifecycle: Validate state transition before starting
        if !existing.status.can_transition_to(ItemStatus::InProgress) {
            anyhow::bail!(
                "Invalid transition: {} → InProgress. \
                 {} is a terminal state and cannot be restarted.",
                existing.status.display_name(),
                existing.status.display_name(),
            );
        }
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

#[cfg(test)]
mod resolution_pure_tests {
    //! Covers print helpers in resolution.rs (196 lines, 0 prior tests).
    //! Skips async resolve_* fns (require GitHub API + RoadmapService
    //! fixtures).
    use super::*;
    use crate::cli::handlers::work_contract::{
        EvidenceType, FalsificationMethod, FalsificationResult,
    };

    fn fail_claim(idx: usize, hypothesis: &str, blocking: bool) -> ClaimResult {
        ClaimResult {
            index: idx,
            hypothesis: hypothesis.to_string(),
            method: FalsificationMethod::AbsoluteCoverage,
            result: FalsificationResult::failed(
                format!("{hypothesis} explanation"),
                EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 1.0,
                },
            ),
            is_blocking: blocking,
        }
    }

    fn pass_claim(idx: usize, hypothesis: &str) -> ClaimResult {
        ClaimResult {
            index: idx,
            hypothesis: hypothesis.to_string(),
            method: FalsificationMethod::AbsoluteCoverage,
            result: FalsificationResult::passed("passed"),
            is_blocking: true,
        }
    }

    fn report(claims: Vec<ClaimResult>) -> FalsificationReport {
        let total = claims.len();
        let passed = claims.iter().filter(|c| !c.result.falsified).count();
        let failed = claims
            .iter()
            .filter(|c| c.result.falsified && c.is_blocking)
            .count();
        let warnings = claims
            .iter()
            .filter(|c| c.result.falsified && !c.is_blocking)
            .count();
        FalsificationReport {
            total_claims: total,
            passed,
            failed,
            warnings,
            unmeasured: 0,
            claim_results: claims,
            all_passed: failed == 0,
        }
    }

    // ── print_warning_failures ──

    #[test]
    fn test_print_warning_failures_no_warnings_writes_nothing() {
        let r = report(vec![pass_claim(1, "good")]);
        // No panic on no warnings — output empty section.
        print_warning_failures(&r);
    }

    #[test]
    fn test_print_warning_failures_with_warnings_emits_section() {
        let r = report(vec![
            fail_claim(1, "wcov", false),
            fail_claim(2, "wcomplexity", false),
        ]);
        // Smoke: no panic, all branches reached.
        print_warning_failures(&r);
    }

    // ── print_blocked_result ──

    #[test]
    fn test_print_blocked_result_with_failures_only() {
        let r = report(vec![fail_claim(1, "coverage", true)]);
        let unoverrideable: Vec<&ClaimResult> = r
            .claim_results
            .iter()
            .filter(|c| c.result.falsified && c.is_blocking)
            .collect();
        // Smoke — verifies no panic and full branch coverage.
        print_blocked_result(&r, &unoverrideable, "PMAT-100");
    }

    #[test]
    fn test_print_blocked_result_with_failures_and_warnings() {
        let r = report(vec![
            fail_claim(1, "coverage", true),
            fail_claim(2, "complexity", false),
        ]);
        let unoverrideable: Vec<&ClaimResult> = r
            .claim_results
            .iter()
            .filter(|c| c.result.falsified && c.is_blocking)
            .collect();
        // Both blocking + warning branches.
        print_blocked_result(&r, &unoverrideable, "PMAT-200");
    }

    #[test]
    fn test_print_blocked_result_empty_unoverrideable() {
        // Edge case: no blocking failures but called anyway.
        let r = report(vec![fail_claim(1, "warn-only", false)]);
        let unoverrideable: Vec<&ClaimResult> = vec![];
        print_blocked_result(&r, &unoverrideable, "PMAT-300");
    }
}
