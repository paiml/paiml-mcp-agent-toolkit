//! #1502: `pmat work start` / `pmat work delegate` refuse new work when the repo is at its
//! open-PR cap, drafts included. The cap is read from ONE place: `[work] max_open_prs` in
//! pmat.toml. No key means no gate. A failed read is `not_measured` and never refuses (L23).

use anyhow::{bail, Result};
use std::path::Path;

/// Outcome of the cap check.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CapVerdict {
    Pass,
    Refuse(String),
}

/// `open` PRs (drafts counted) against `cap`: the 10th passes, the 11th is refused at cap 10,
/// so a start at `open >= cap` is refused. `oldest` is `(number, title)`.
pub(crate) fn decide(open: usize, cap: usize, oldest: Option<(u64, String)>) -> CapVerdict {
    if open < cap {
        return CapVerdict::Pass;
    }
    let oldest = oldest
        .map(|(n, t)| format!("oldest open PR: #{n} {t}"))
        .unwrap_or_else(|| "oldest open PR: unknown".to_string());
    CapVerdict::Refuse(format!(
        "open-PR cap reached: {open} open (drafts included), cap {cap}; {oldest}. \
         Merge or close one before starting new work."
    ))
}

/// Read `max_open_prs` from the `[work]` table of pmat.toml text. Pure.
pub(crate) fn parse_cap(toml_text: &str) -> Option<usize> {
    let mut in_work = false;
    for line in toml_text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_work = t == "[work]";
        } else if in_work {
            if let Some(v) = t.strip_prefix("max_open_prs") {
                return v
                    .trim_start()
                    .strip_prefix('=')?
                    .split('#')
                    .next()?
                    .trim()
                    .parse()
                    .ok();
            }
        }
    }
    None
}

/// Parse the `gh api graphql` answer: `(open count, oldest (number, title))`. Pure.
pub(crate) fn parse_open_prs(json: &str) -> Option<(usize, Option<(u64, String)>)> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let prs = v.pointer("/data/repository/pullRequests")?;
    let open = prs.get("totalCount")?.as_u64()? as usize;
    let oldest = prs.pointer("/nodes/0").and_then(|n| {
        Some((
            n.get("number")?.as_u64()?,
            n.get("title")?.as_str()?.to_string(),
        ))
    });
    Some((open, oldest))
}

const QUERY: &str = "query($o:String!,$r:String!){repository(owner:$o,name:$r){pullRequests(states:OPEN,first:1,orderBy:{field:CREATED_AT,direction:ASC}){totalCount nodes{number title}}}}";

fn gh_open_prs(project_path: &Path) -> Option<(usize, Option<(u64, String)>)> {
    let nwo = std::process::Command::new("gh")
        .args([
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "--jq",
            ".nameWithOwner",
        ])
        .current_dir(project_path)
        .output()
        .ok()?;
    let nwo = String::from_utf8(nwo.stdout).ok()?;
    let (owner, repo) = nwo.trim().split_once('/')?;
    let out = std::process::Command::new("gh")
        .args([
            "api",
            "graphql",
            "-f",
            &format!("query={QUERY}"),
            "-F",
            &format!("o={owner}"),
            "-F",
            &format!("r={repo}"),
        ])
        .current_dir(project_path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_open_prs(&String::from_utf8_lossy(&out.stdout))
}

/// Entry-point guard for `work start` and `work delegate`.
pub(crate) fn enforce_open_pr_cap(project_path: &Path) -> Result<()> {
    let Some(cap) = std::fs::read_to_string(project_path.join("pmat.toml"))
        .ok()
        .as_deref()
        .and_then(parse_cap)
    else {
        return Ok(());
    };
    let Some((open, oldest)) = gh_open_prs(project_path) else {
        eprintln!("open-PR cap: not_measured (GitHub read failed); not refusing on it");
        return Ok(());
    };
    match decide(open, cap, oldest) {
        CapVerdict::Pass => Ok(()),
        CapVerdict::Refuse(msg) => bail!(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tenth_open_pr_passes_and_the_eleventh_is_refused_at_cap_ten() {
        assert_eq!(decide(9, 10, None), CapVerdict::Pass);
        assert!(matches!(decide(10, 10, None), CapVerdict::Refuse(_)));
        assert!(matches!(decide(11, 10, None), CapVerdict::Refuse(_)));
    }

    #[test]
    fn the_refusal_names_count_cap_and_oldest_pr() {
        let CapVerdict::Refuse(m) = decide(12, 10, Some((77, "old".into()))) else {
            panic!()
        };
        assert!(
            m.contains("12 open") && m.contains("cap 10") && m.contains("#77 old"),
            "{m}"
        );
    }

    #[test]
    fn the_cap_is_read_from_the_work_table_only() {
        assert_eq!(parse_cap("[work]\nmax_open_prs = 10 # cap\n"), Some(10));
        assert_eq!(parse_cap("[quality]\nmax_open_prs = 3\n"), None);
        assert_eq!(parse_cap("[work]\nother = 1\n"), None);
        assert_eq!(parse_cap(""), None);
    }

    #[test]
    fn the_graphql_answer_parses_and_a_bad_one_is_not_measured() {
        let j = r#"{"data":{"repository":{"pullRequests":{"totalCount":4,"nodes":[{"number":9,"title":"t"}]}}}}"#;
        assert_eq!(parse_open_prs(j), Some((4, Some((9, "t".to_string())))));
        assert_eq!(parse_open_prs("{\"errors\":[]}"), None);
        assert_eq!(parse_open_prs("not json"), None);
    }

    #[test]
    fn no_cap_key_means_no_gate() {
        let t = tempfile::tempdir().unwrap();
        assert!(enforce_open_pr_cap(t.path()).is_ok());
    }
}
