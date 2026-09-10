pub(crate) fn check_ticket_linkage(
    project_path: &std::path::Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&std::path::Path>,
) -> crate::cli::handlers::comply_handlers::check_handlers::types::ComplianceCheck {
    let name = "CB-2112: Ticket Linkage".to_string();

    let inputs = match roadmap_inputs(project_path, comply_config, "cb-2112", github_snapshot) {
        Ok(i) => i,
        Err(mut e) => {
            e.name = name;
            return e;
        }
    };

    let report = crate::services::work_sync::linkage::ticket_linkage(&inputs.roadmap, &inputs.snapshot);

    let mut check = crate::cli::handlers::comply_handlers::check_handlers::types::ComplianceCheck {
        name,
        status: crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Skip,
        severity: crate::models::comply_config::CheckSeverity::Error.into(),
        message: String::new(),
    };

    if report.findings.is_empty() {
        check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Pass;
        check.message = format!(
            "{} open item(s) (status not completed or cancelled, goal-mode.md §4.2) each name an open issue numbered as the item's numeric tail: linked {}; snapshot: {} taken {}",
            report.open_items, report.linked, inputs.source, inputs.taken_at
        );
    } else {
        check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail;
        let n = report.findings.len();
        let mut renderings = Vec::new();
        let mut count_no_issue = 0;
        let mut count_issue_closed = 0;
        let mut count_issue_absent = 0;
        let mut count_issue_excluded = 0;
        let mut count_tail_mismatch = 0;

        for f in &report.findings {
            match f {
                crate::services::work_sync::linkage::LinkFinding::NoIssue{..} => count_no_issue += 1,
                crate::services::work_sync::linkage::LinkFinding::IssueClosed{..} => count_issue_closed += 1,
                crate::services::work_sync::linkage::LinkFinding::IssueAbsent{..} => count_issue_absent += 1,
                crate::services::work_sync::linkage::LinkFinding::IssueExcluded{..} => count_issue_excluded += 1,
                crate::services::work_sync::linkage::LinkFinding::TailMismatch{..} => count_tail_mismatch += 1,
            }
            if renderings.len() < 8 {
                renderings.push(f.render());
            }
        }
        
        let more = if n > 8 {
            format!(" (+{} more)", n - 8)
        } else {
            String::new()
        };
        
        check.message = format!(
            "{} finding(s) — NO-ISSUE {}, ISSUE-CLOSED {}, ISSUE-ABSENT {}, ISSUE-EXCLUDED {}, TAIL-MISMATCH {}: {}{}; an item is minted from its issue by `pmat work add --github-issue N` (#1240), which is why the number must be the tail — an item minted before that whose issue is not its tail needs re-minting under its issue (no sync renames an id); `pmat work sync --direction yaml-to-github` opens an issue for an unlinked item but cannot make the number match; snapshot: {} taken {}",
            n, count_no_issue, count_issue_closed, count_issue_absent, count_issue_excluded, count_tail_mismatch,
            renderings.join("; "), more, inputs.source, inputs.taken_at
        );
    }

    check
}
