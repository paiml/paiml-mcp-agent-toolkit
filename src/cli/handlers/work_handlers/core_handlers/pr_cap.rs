//! Open-PR cap at the door of `pmat work start` / `pmat work delegate` (GH-1502).
//!
//! The cap is read from ONE place: `.github/open-pr-cap` (a single integer, or the word
//! `external` for a repo that is excluded). No file means no cap is configured and the
//! start is allowed. The open count includes drafts (GraphQL `totalCount`). A failed
//! read is `not_measured` and never refuses (L23); it is printed, not silent.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// What the GitHub read returned: the open-PR count (drafts included) and the oldest one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPrs {
    pub count: usize,
    pub oldest: Option<(u64, String)>,
}

/// The cap as configured in `.github/open-pr-cap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapConfig {
    /// No file: nothing to enforce.
    Unset,
    /// Repo marked `external`: excluded.
    External,
    Cap(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapVerdict {
    Allow,
    NotMeasured(String),
    Refuse {
        open: usize,
        cap: usize,
        oldest: Option<(u64, String)>,
    },
}

/// Parse the cap file body. Garbage is an error, never a silent "no cap".
pub fn parse_cap(body: &str) -> Result<CapConfig, String> {
    let t = body.trim();
    if t.eq_ignore_ascii_case("external") {
        return Ok(CapConfig::External);
    }
    match t.parse::<usize>() {
        Ok(n) if n > 0 => Ok(CapConfig::Cap(n)),
        _ => Err(format!(
            ".github/open-pr-cap must hold one positive integer or `external`, found {t:?}"
        )),
    }
}

/// Pure decision. With cap = 10: 9 open -> Allow, 10 open -> Refuse (the 11th PR is refused).
pub fn decide(cap: usize, read: Result<OpenPrs, String>) -> CapVerdict {
    match read {
        Err(why) => CapVerdict::NotMeasured(why),
        Ok(p) if p.count >= cap => CapVerdict::Refuse {
            open: p.count,
            cap,
            oldest: p.oldest,
        },
        Ok(_) => CapVerdict::Allow,
    }
}

/// Parse `gh api graphql` output for the open-PR query.
pub fn parse_open_prs(json: &str) -> Result<OpenPrs, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("bad json: {e}"))?;
    let prs = &v["data"]["repository"]["pullRequests"];
    let count = prs["totalCount"]
        .as_u64()
        .ok_or_else(|| "no totalCount in the GraphQL response".to_string())?
        as usize;
    let oldest = prs["nodes"]
        .as_array()
        .and_then(|n| n.first())
        .and_then(|n| Some((n["number"].as_u64()?, n["createdAt"].as_str()?.to_string())));
    Ok(OpenPrs { count, oldest })
}

fn read_open_prs(repo: &str) -> Result<OpenPrs, String> {
    let (owner, name) = repo
        .split_once('/')
        .ok_or_else(|| format!("repo {repo:?} is not owner/name"))?;
    let query = "query($o:String!,$n:String!){repository(owner:$o,name:$n){pullRequests(states:OPEN,first:1,orderBy:{field:CREATED_AT,direction:ASC}){totalCount nodes{number createdAt}}}}";
    let out = Command::new("gh")
        .args(["api", "graphql", "-f"])
        .arg(format!("query={query}"))
        .args(["-F"])
        .arg(format!("o={owner}"))
        .args(["-F"])
        .arg(format!("n={name}"))
        .output()
        .map_err(|e| format!("gh not runnable: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "gh api graphql failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    parse_open_prs(&String::from_utf8_lossy(&out.stdout))
}

/// Turn a verdict into the command's result. Pure, so the exit contract is testable.
pub fn apply(verdict: CapVerdict) -> Result<()> {
    match verdict {
        CapVerdict::Allow => Ok(()),
        CapVerdict::NotMeasured(why) => {
            eprintln!("not_measured: open-PR cap not checked ({why}); starting anyway");
            Ok(())
        }
        CapVerdict::Refuse { open, cap, oldest } => {
            let oldest = oldest
                .map(|(n, at)| format!("oldest open PR is #{n} (opened {at})"))
                .unwrap_or_else(|| "oldest open PR unknown".to_string());
            bail!("refused: {open} open PRs (drafts included) at the cap of {cap}; {oldest}. Merge or close one first.")
        }
    }
}

/// The door check. Called first by `work start` and `work delegate`.
pub fn enforce(project_path: &Path, repo: Option<String>) -> Result<()> {
    let body = match std::fs::read_to_string(project_path.join(".github/open-pr-cap")) {
        Ok(b) => b,
        Err(_) => return Ok(()),
    };
    let cap = match parse_cap(&body) {
        Ok(CapConfig::Cap(n)) => n,
        Ok(_) => return Ok(()),
        Err(e) => bail!("{e}"),
    };
    let read = match repo {
        Some(r) => read_open_prs(&r),
        None => Err("no GitHub repo resolves".to_string()),
    };
    apply(decide(cap, read))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prs(count: usize) -> Result<OpenPrs, String> {
        Ok(OpenPrs {
            count,
            oldest: Some((7, "2026-11-01T00:00:00Z".into())),
        })
    }

    #[test]
    fn nine_open_at_cap_ten_is_allowed() {
        assert_eq!(decide(10, prs(9)), CapVerdict::Allow);
    }

    #[test]
    fn ten_open_at_cap_ten_is_refused_and_names_count_cap_oldest() {
        let v = decide(10, prs(10));
        assert!(matches!(
            v,
            CapVerdict::Refuse {
                open: 10,
                cap: 10,
                ..
            }
        ));
        let msg = apply(v).unwrap_err().to_string();
        assert!(
            msg.contains("10 open") && msg.contains("cap of 10") && msg.contains("#7"),
            "{msg}"
        );
    }

    #[test]
    fn a_failed_read_is_not_measured_and_never_refuses() {
        let v = decide(10, Err("rate limited".into()));
        assert_eq!(v, CapVerdict::NotMeasured("rate limited".into()));
        assert!(apply(v).is_ok());
    }

    #[test]
    fn drafts_count_because_the_graphql_total_is_taken_as_is() {
        let json = r#"{"data":{"repository":{"pullRequests":{"totalCount":10,"nodes":[{"number":3,"createdAt":"2026-10-30T00:00:00Z"}]}}}}"#;
        let p = parse_open_prs(json).unwrap();
        assert_eq!(p.count, 10);
        assert_eq!(p.oldest, Some((3, "2026-10-30T00:00:00Z".into())));
        assert!(matches!(decide(10, Ok(p)), CapVerdict::Refuse { .. }));
    }

    #[test]
    fn a_response_without_a_total_is_not_measured() {
        assert!(parse_open_prs(r#"{"data":null}"#).is_err());
        assert!(parse_open_prs("not json").is_err());
    }

    #[test]
    fn cap_file_parses_integer_and_external_and_rejects_garbage() {
        assert_eq!(parse_cap("10\n"), Ok(CapConfig::Cap(10)));
        assert_eq!(parse_cap(" External "), Ok(CapConfig::External));
        assert!(parse_cap("0").is_err());
        assert!(parse_cap("ten").is_err());
    }

    #[test]
    fn no_cap_file_means_start_is_allowed() {
        let dir = std::env::temp_dir().join(format!("pmat-prcap-none-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(enforce(&dir, None).is_ok());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_malformed_cap_file_refuses_rather_than_silently_allowing() {
        let dir = std::env::temp_dir().join(format!("pmat-prcap-bad-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".github")).unwrap();
        std::fs::write(dir.join(".github/open-pr-cap"), "ten").unwrap();
        assert!(enforce(&dir, Some("o/r".into())).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn external_repo_is_excluded() {
        let dir = std::env::temp_dir().join(format!("pmat-prcap-ext-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".github")).unwrap();
        std::fs::write(dir.join(".github/open-pr-cap"), "external").unwrap();
        assert!(enforce(&dir, Some("o/r".into())).is_ok());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
