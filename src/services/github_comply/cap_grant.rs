//! CAP-VALVE-001 (paiml/infra#5489): the PR CAP-GRANT rule, a port of `cap_grant` in the
//! fleet's `gh pr create` hook. A fresh PR grant for a repo whose count covers the open
//! count admits new work at the open-PR cap. Everything else grants nothing: no grant, an
//! unreadable file, or one malformed line anywhere in the file.
//!
//! A PR grant is one line:
//! `<UTC> CAP-GRANT repo=<r> count=<n/cap> cop=<c> reason=PR...: ...`
//! `<r>` is `owner/name`, or a bare name that stands for `paiml/<name>` only. An issue
//! grant (a reason not starting with `PR`/`PRs`) never opens the PR cap.

use regex::Regex;
use std::path::Path;

/// Where the grants are written on a host that has the valve. There is deliberately no
/// environment or config override: whoever is capped cannot point the cap at its own file.
pub const GRANT_FILE: &str = "/mnt/nvme-raid0/cop-inbox/handoff/arbiter-cop/CAP-GRANTS.md";
/// Where a capped session asks for a grant.
pub const INBOX: &str = "/mnt/nvme-raid0/cop-inbox/inbox.md";
/// A grant is fresh for this many seconds.
pub const TTL_SECS: i64 = 7200;
/// How far ahead of this clock a grant may be stamped.
const FUTURE_SKEW_SECS: i64 = 120;

/// The verdict of a grant file on one (repo, open count, time).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Grant {
    /// A fresh PR grant covers the count; the text is the grant line without its request.
    Covers(String),
    /// The file is well formed and holds no such grant.
    NoGrant,
    /// The file is unreadable, or holds a malformed line: no grant counts.
    Unusable,
}
/// `<UTC> CAP-GRANT repo=<r> count=<n/cap> cop=<c> reason=<reason>`; groups: repo, n, reason.
const GRANT_LINE: &str = r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}(?::[0-9]{2})?Z CAP-GRANT repo=([A-Za-z0-9_./-]+) count=([0-9]{1,6})/[0-9]{1,6}[,:;.]? cop=[A-Za-z0-9_-]+ reason=(.*)$";
/// The one other entry: a refused request. It is checked for its date and grants nothing.
const NOT_GRANTED_LINE: &str =
    r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}(?::[0-9]{2})?Z NOT-GRANTED ";
/// A line's trailing copy of the request; nothing after it is judged.
const REQUEST: &str = " | req: ";

/// The Unix time of a stamp the line patterns matched; `None` for a time that does not exist.
fn unix_time(stamp: &str) -> Option<i64> {
    let bare = stamp.strip_suffix('Z')?;
    // chrono reads a leap second; the hook does not, so neither does this.
    if bare.len() > 16 && bare.ends_with(":60") {
        return None;
    }
    let format = if bare.len() == 16 {
        "%Y-%m-%dT%H:%M"
    } else {
        "%Y-%m-%dT%H:%M:%S"
    };
    chrono::NaiveDateTime::parse_from_str(bare, format)
        .ok()
        .map(|t| t.and_utc().timestamp())
}

/// The two entry patterns, compiled.
struct Patterns {
    grant: Regex,
    not_granted: Regex,
}

/// One entry line: `Err` = malformed, `Ok(None)` = well formed but grants nothing here.
fn judge_line<'a>(
    p: &Patterns,
    line: &'a str,
    want: &str,
    open: usize,
    now: i64,
) -> Result<Option<&'a str>, ()> {
    let fields = if p.not_granted.is_match(line) {
        None
    } else {
        Some(p.grant.captures(line).ok_or(())?)
    };
    // The date is checked on every entry line, before any filter can skip it.
    let at = line.split(' ').next().and_then(unix_time).ok_or(())?;
    let Some(fields) = fields else {
        return Ok(None);
    };
    let [repo, count, reason] = [1, 2, 3].map(|i| fields.get(i).map_or("", |m| m.as_str()));
    let mut repo = repo.to_ascii_lowercase();
    if !repo.contains('/') {
        repo = format!("paiml/{repo}");
    }
    let reason = before_request(reason).to_ascii_lowercase();
    let pr_grant = ["pr ", "pr:", "prs ", "prs:"]
        .iter()
        .any(|k| reason.starts_with(k));
    let covered = count.parse::<usize>().is_ok_and(|n| n >= open);
    let fresh = (-FUTURE_SKEW_SECS..=TTL_SECS).contains(&now.saturating_sub(at));
    Ok((repo == want && pr_grant && covered && fresh).then(|| before_request(line)))
}

fn before_request(text: &str) -> &str {
    text.split_once(REQUEST).map_or(text, |(head, _)| head)
}

/// Judge the text of a grant file. `now` is a Unix time.
pub fn judge(text: &str, repo: &str, open: usize, now: i64) -> Grant {
    let (Ok(grant), Ok(not_granted)) = (Regex::new(GRANT_LINE), Regex::new(NOT_GRANTED_LINE))
    else {
        return Grant::Unusable;
    };
    let patterns = Patterns { grant, not_granted };
    let want = repo.to_ascii_lowercase();
    let mut hit = None;
    for line in text.split('\n') {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match judge_line(&patterns, line, &want, open, now) {
            Ok(found) => hit = found.or(hit),
            Err(()) => return Grant::Unusable,
        }
    }
    hit.map_or(Grant::NoGrant, |l| Grant::Covers(l.to_string()))
}

/// Judge the grant file at `path`: only a readable regular file can grant.
pub fn read(path: &Path, repo: &str, open: usize, now: i64) -> Grant {
    if !path.is_file() {
        return Grant::Unusable;
    }
    std::fs::read_to_string(path).map_or(Grant::Unusable, |text| judge(&text, repo, open, now))
}

/// The way on, appended to a cap refusal. `None` on a host without the valve (the grant
/// file's directory does not exist), where the cap refuses as it always did.
pub fn escalation(
    path: &Path,
    grant: &Grant,
    repo: &str,
    open: usize,
    cap: usize,
) -> Option<String> {
    if !path.parent().is_some_and(Path::is_dir) {
        return None;
    }
    let state = if *grant == Grant::Unusable {
        "grant file unreadable or malformed, so no grant counts:"
    } else {
        "no fresh PR grant covers it in"
    };
    Some(format!(
        "CAP-ESCALATE ({state} {}): append one line to {INBOX}: '<HH:MM>Z <your-cop> | CAP-ESCALATE {repo} {open}/{cap} PR: <what is blocked>'. arbiter-cop's grant lands within 10 min; then run the same command again. Do other work meanwhile. Never bypass or edit this check.",
        path.display()
    ))
}

#[cfg(test)]
#[path = "cap_grant_tests.rs"]
mod tests;
