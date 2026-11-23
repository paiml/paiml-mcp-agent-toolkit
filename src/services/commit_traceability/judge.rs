//! Judging of `git log` lines against the roadmap (CB-2113).

use super::*;

/// One `git log` line (`%H \x1f %s \x1f trailers`) → (hash, subject, ticket ids).
/// `None` for a line with fewer than two fields, as before the extraction.
pub(super) fn parse_log_line(line: &str) -> Option<(String, String, Vec<&str>)> {
    let parts: Vec<&str> = line.split('\x1f').collect();
    if parts.len() < 2 {
        return None;
    }
    let tr = if parts.len() >= 3 {
        parts[2].trim()
    } else {
        ""
    };
    let ids = tr
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    Some((parts[0].to_string(), parts[1].to_string(), ids))
}

/// The violation, if any, that one trailer id commits against the roadmap.
///
/// A row that was open at the merge-base and is terminal at HEAD was closed
/// inside this range, and is not a finding (GH-1824).
pub(super) fn judge_id(
    roadmap: &std::collections::HashMap<String, crate::models::roadmap::ItemStatus>,
    open_at_base: &std::collections::HashSet<String>,
    id: &str,
) -> Option<Violation> {
    use crate::models::roadmap::ItemStatus;
    let Some(status) = roadmap.get(id) else {
        return Some(Violation::UnknownTicket(id.to_string()));
    };
    // Terminal states, spelled as the roadmap serialises them (`ItemStatus` is
    // `rename_all = "lowercase"`).
    let terminal = match status {
        ItemStatus::Completed => "completed",
        ItemStatus::Cancelled => "cancelled",
        _ => return None,
    };
    if open_at_base.contains(id) {
        return None;
    }
    Some(Violation::TerminalTicket {
        id: id.to_string(),
        status: terminal.to_string(),
    })
}

/// Judge every trailer id of one commit, appending a finding per violation.
pub(super) fn judge_ids(
    roadmap: &std::collections::HashMap<String, crate::models::roadmap::ItemStatus>,
    open_at_base: &std::collections::HashSet<String>,
    hash: &str,
    subject: &str,
    ids: &[&str],
    findings: &mut Vec<Finding>,
) {
    findings.extend(ids.iter().filter_map(|id| {
        judge_id(roadmap, open_at_base, id).map(|violation| Finding {
            hash: hash.to_string(),
            subject: subject.to_string(),
            violation,
        })
    }));
}

/// Judge every commit line of `git log`: count them, count the trailered ones,
/// and collect the findings.
pub(super) fn judge_log(
    log: &str,
    roadmap: &std::collections::HashMap<String, crate::models::roadmap::ItemStatus>,
    open_at_base: &std::collections::HashSet<String>,
) -> (usize, usize, Vec<Finding>) {
    let mut commits = 0;
    let mut trailered = 0;
    let mut findings = vec![];
    for line in log.split('\n').filter(|l| !l.trim().is_empty()) {
        commits += 1;
        let Some((hash, subject, ids)) = parse_log_line(line) else {
            continue;
        };
        if ids.is_empty() {
            findings.push(Finding {
                hash,
                subject,
                violation: Violation::NoTrailer,
            });
        } else {
            judge_ids(roadmap, open_at_base, &hash, &subject, &ids, &mut findings);
            trailered += 1;
        }
    }
    (commits, trailered, findings)
}
