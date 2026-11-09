// #1502 — `pmat work start` / `pmat work delegate` refuse at the open-PR cap.
//
// Supply that is not refused at the door piles up behind the cap (Little's
// Law: merges/h <= cap / W). Drafts count. The cap lives in ONE place:
// `<project>/.pmat/pr-cap` (a single integer; absent = no cap). The open-PR
// list is one GraphQL call via `gh`. A failed read is `not_measured` and does
// not refuse (L23): it is printed, never silently treated as "below cap".

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// One open PR (draft or not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPr {
    pub number: u64,
    pub created_at: String,
}

/// Outcome of the cap check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapVerdict {
    /// Below cap (or no cap configured).
    Pass,
    /// At or over cap: names the count, the cap and the oldest open PR.
    Refuse {
        count: usize,
        cap: usize,
        oldest: u64,
    },
    /// The read failed; no decision is made on it.
    NotMeasured(String),
}

/// Pure decision: `open` PRs (drafts included) against `cap`.
/// 9 open with cap 10 passes; 10 open is refused.
pub fn decide(open: &[OpenPr], cap: usize) -> CapVerdict {
    if open.len() < cap {
        return CapVerdict::Pass;
    }
    // ISO-8601 UTC timestamps sort lexically.
    let oldest = open
        .iter()
        .min_by(|a, b| a.created_at.cmp(&b.created_at))
        .map_or(0, |p| p.number);
    CapVerdict::Refuse {
        count: open.len(),
        cap,
        oldest,
    }
}

/// Parse the cap file contents: one positive integer. Anything else is an error.
pub fn parse_cap(text: &str) -> Result<usize> {
    let t = text.trim();
    match t.parse::<usize>() {
        Ok(n) if n > 0 => Ok(n),
        _ => bail!("pr-cap must be one positive integer, got {t:?}"),
    }
}

/// Parse `gh api graphql` output for `repository.pullRequests.nodes`.
pub fn parse_open_prs(json: &str) -> Result<Vec<OpenPr>> {
    let v: serde_json::Value = serde_json::from_str(json)?;
    let nodes = v
        .pointer("/data/repository/pullRequests/nodes")
        .and_then(|n| n.as_array());
    let Some(nodes) = nodes else {
        bail!("graphql response has no data.repository.pullRequests.nodes");
    };
    nodes
        .iter()
        .map(|n| {
            let number = n.get("number").and_then(|x| x.as_u64());
            let created = n.get("createdAt").and_then(|x| x.as_str());
            match (number, created) {
                (Some(number), Some(c)) => Ok(OpenPr {
                    number,
                    created_at: c.to_string(),
                }),
                _ => bail!("pull request node missing number/createdAt"),
            }
        })
        .collect()
}

/// Check the cap for the repo at `project`; `fetch` returns the graphql JSON.
pub fn check_with(project: &Path, fetch: impl FnOnce() -> Result<String>) -> CapVerdict {
    let Ok(text) = std::fs::read_to_string(project.join(".pmat/pr-cap")) else {
        return CapVerdict::Pass;
    };
    let cap = match parse_cap(&text) {
        Ok(c) => c,
        Err(e) => return CapVerdict::NotMeasured(e.to_string()),
    };
    match fetch().and_then(|j| parse_open_prs(&j)) {
        Ok(open) => decide(&open, cap),
        Err(e) => CapVerdict::NotMeasured(e.to_string()),
    }
}

fn gh_fetch(project: &Path) -> Result<String> {
    // Drafts are open PRs: `states: OPEN` returns them. 100 covers any sane cap.
    let q = "query($owner:String!,$repo:String!) { repository(owner:$owner, name:$repo) { pullRequests(states:OPEN, first:100) { nodes { number createdAt } } } }";
    let out = Command::new("gh")
        .args([
            "api",
            "graphql",
            "-F",
            "owner={owner}",
            "-F",
            "repo={repo}",
            "-f",
            &format!("query={q}"),
        ])
        .current_dir(project)
        .output()?;
    if !out.status.success() {
        bail!(
            "gh api graphql failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8(out.stdout)?)
}

/// Entry-point gate for `work start` and `work delegate`. Errors when refused.
pub fn enforce(project: &Path) -> Result<()> {
    match check_with(project, || gh_fetch(project)) {
        CapVerdict::Pass => Ok(()),
        CapVerdict::NotMeasured(why) => {
            eprintln!("pr-cap: not_measured ({why}); not refusing on an unmeasured read");
            Ok(())
        }
        CapVerdict::Refuse { count, cap, oldest } => bail!(
            "open-PR cap reached: {count} open (drafts included), cap {cap}; oldest is #{oldest}. \
             Merge or finish an open PR before starting new work"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prs(n: usize) -> Vec<OpenPr> {
        (0..n)
            .map(|i| OpenPr {
                number: 100 + i as u64,
                created_at: format!("2026-11-{:02}T00:00:00Z", 20 - i),
            })
            .collect()
    }

    #[test]
    fn nine_open_with_cap_ten_passes() {
        assert_eq!(decide(&prs(9), 10), CapVerdict::Pass);
    }

    #[test]
    fn ten_open_with_cap_ten_is_refused_naming_oldest() {
        // the last one built has the earliest createdAt
        assert_eq!(
            decide(&prs(10), 10),
            CapVerdict::Refuse {
                count: 10,
                cap: 10,
                oldest: 109
            }
        );
    }

    #[test]
    fn drafts_count_because_every_open_node_counts() {
        let j = r#"{"data":{"repository":{"pullRequests":{"nodes":[
            {"number":1,"createdAt":"2026-11-01T00:00:00Z","isDraft":true},
            {"number":2,"createdAt":"2026-11-02T00:00:00Z","isDraft":false}]}}}}"#;
        let open = parse_open_prs(j).unwrap();
        assert_eq!(open.len(), 2);
        assert!(matches!(
            decide(&open, 2),
            CapVerdict::Refuse { oldest: 1, .. }
        ));
    }

    #[test]
    fn read_failure_is_not_measured_never_pass_or_refuse() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join(".pmat")).unwrap();
        std::fs::write(d.path().join(".pmat/pr-cap"), "10\n").unwrap();
        let v = check_with(d.path(), || bail!("HTTP 403 rate limited"));
        assert!(matches!(v, CapVerdict::NotMeasured(ref m) if m.contains("403")));
    }

    #[test]
    fn no_cap_file_passes_without_a_read() {
        let d = tempfile::tempdir().unwrap();
        let v = check_with(d.path(), || panic!("must not fetch without a cap"));
        assert_eq!(v, CapVerdict::Pass);
    }

    #[test]
    fn cap_file_must_be_one_positive_integer() {
        assert_eq!(parse_cap(" 10\n").unwrap(), 10);
        assert!(parse_cap("0").is_err());
        assert!(parse_cap("ten").is_err());
    }

    #[test]
    fn malformed_response_is_an_error_not_zero_prs() {
        assert!(parse_open_prs("{}").is_err());
    }
}
