//! Judging of `git log` lines against the roadmap (CB-2113).

use super::*;

/// GitHub app accounts author commits as `<id>+<app>[bot]@users.noreply.github.com`.
/// A commit by one of them cannot carry a trailer (dependabot writes its own
/// message), so a missing trailer is exempted for it, and reported as such.
pub(super) fn is_bot_author(email: &str) -> bool {
    email
        .trim()
        .to_ascii_lowercase()
        .ends_with("[bot]@users.noreply.github.com")
}

/// One `git log` line (`%H \x1f %s \x1f trailers \x1f %ae`) →
/// (hash, subject, ticket ids, author email).
/// `None` for a line with fewer than two fields, as before the extraction.
pub(super) fn parse_log_line(line: &str) -> Option<(String, String, Vec<&str>, String)> {
    let parts: Vec<&str> = line.split('\x1f').collect();
    if parts.len() < 2 {
        return None;
    }
    let tr = if parts.len() >= 3 {
        parts[2].trim()
    } else {
        ""
    };
    let email = parts
        .get(3)
        .map(|e| e.trim().to_string())
        .unwrap_or_default();
    let ids = tr
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    Some((parts[0].to_string(), parts[1].to_string(), ids, email))
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
/// count the untrailered bot-authored ones (exempt, not findings), and collect
/// the findings.
pub(super) fn judge_log(
    log: &str,
    roadmap: &std::collections::HashMap<String, crate::models::roadmap::ItemStatus>,
    open_at_base: &std::collections::HashSet<String>,
) -> (usize, usize, usize, Vec<Finding>) {
    let mut commits = 0;
    let mut trailered = 0;
    let mut bot_exempt = 0;
    let mut findings = vec![];
    for line in log.split('\n').filter(|l| !l.trim().is_empty()) {
        commits += 1;
        let Some((hash, subject, ids, email)) = parse_log_line(line) else {
            continue;
        };
        if ids.is_empty() && is_bot_author(&email) {
            bot_exempt += 1;
        } else if ids.is_empty() {
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
    (commits, trailered, bot_exempt, findings)
}
