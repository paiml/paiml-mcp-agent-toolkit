// GHC-001: `pmat comply github` — include!()'d into comply_handlers/mod.rs scope.

use crate::services::github_comply::{self as ghc, source as ghc_source};

/// Which rules a `comply github` invocation runs: `--pr` is the PR check, `--rules`
/// names rules, and with neither it is the reconciler set (R1, R2, R4, R4d, R6b).
fn github_rules(pr: Option<u64>, rules: &[String]) -> Result<Vec<ghc::Rule>> {
    if pr.is_some() {
        return Ok(vec![ghc::Rule::R3, ghc::Rule::R6]);
    }
    if rules.is_empty() {
        return Ok(ghc::Rule::RECONCILER.to_vec());
    }
    rules.iter().map(|r| ghc::Rule::parse(r)).collect()
}

/// Judge `snap` and label the report with where the snapshot came from.
pub(crate) fn github_report(
    policy: &ghc::Policy,
    snap: &ghc::RepoSnapshot,
    pr: Option<u64>,
    rules: &[String],
    source: &str,
) -> Result<ghc::Report> {
    let selected = github_rules(pr, rules)?;
    let mut report = match pr {
        Some(n) => ghc::evaluate_pr(snap, policy, n),
        None => ghc::evaluate(snap, policy, &selected),
    };
    report.source = source.to_string();
    Ok(report)
}

/// The snapshot to judge and the label for its source.
fn github_snapshot(
    path: &Path,
    repo: Option<&str>,
    file: Option<&Path>,
) -> Result<(ghc::RepoSnapshot, &'static str)> {
    if let Some(file) = file {
        let snap = ghc_source::load_snapshot_file(file)?;
        if let Some(r) = repo.filter(|r| !r.eq_ignore_ascii_case(&snap.repo)) {
            anyhow::bail!("--repo {r} but the snapshot is of {}", snap.repo);
        }
        return Ok((snap, "snapshot-file"));
    }
    let repo = match repo {
        Some(r) => r.to_string(),
        None => ghc_source::current_repo(path)?,
    };
    let snap = ghc_source::fetch_snapshot(&ghc_source::GhGraphql, &repo, chrono::Utc::now())?;
    Ok((snap, "github-graphql"))
}

/// `pmat comply github`: print the report, exit 1 unless it is clean.
pub(crate) fn handle_github(
    path: &Path,
    repo: Option<&str>,
    pr: Option<u64>,
    rules: &[String],
    format: crate::cli::commands::GithubComplyFormat,
    snapshot: Option<&Path>,
) -> Result<()> {
    github_rules(pr, rules)?; // refuse a bad rule id before any read
    let (snap, source) = github_snapshot(path, repo, snapshot)?;
    let policy = ghc::Policy::load(path, &snap.repo)?;
    let report = github_report(&policy, &snap, pr, rules, source)?;
    match format {
        crate::cli::commands::GithubComplyFormat::Text => print!("{}", report.render_text()),
        crate::cli::commands::GithubComplyFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?)
        }
    }
    if !report.ok() {
        std::process::exit(1);
    }
    Ok(())
}

/// `pmat comply github`: the rules report, or with `--backfill` the backfill table.
pub(crate) fn handle_github_command(
    path: &Path,
    repo: Option<&str>,
    pr: Option<u64>,
    rules: &[String],
    format: crate::cli::commands::GithubComplyFormat,
    snapshot: Option<&Path>,
    backfill: bool,
) -> Result<()> {
    if backfill {
        handle_github_backfill(path, repo, format)
    } else {
        handle_github(path, repo, pr, rules, format, snapshot)
    }
}

/// `pmat comply github --backfill`: read open issues and merged PRs over GraphQL and
/// print the DONE / PARTIAL / WRONG-LINK table. Classifies only; writes nothing.
pub(crate) fn handle_github_backfill(
    path: &Path,
    repo: Option<&str>,
    format: crate::cli::commands::GithubComplyFormat,
) -> Result<()> {
    let repo = match repo {
        Some(r) => r.to_string(),
        None => ghc_source::current_repo(path)?,
    };
    let g = ghc_source::GhGraphql;
    let snap = ghc_source::fetch_snapshot(&g, &repo, chrono::Utc::now())?;
    let merged = ghc_source::fetch_merged_prs(&g, &repo)?;
    let report = ghc::backfill::backfill(&snap, &merged);
    match format {
        crate::cli::commands::GithubComplyFormat::Text => print!("{}", report.render_text()),
        crate::cli::commands::GithubComplyFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?)
        }
    }
    Ok(())
}

#[cfg(test)]
mod github_handler_tests {
    use super::*;

    fn snap(issues: usize) -> ghc::RepoSnapshot {
        ghc::RepoSnapshot {
            repo: "paiml/x".into(),
            taken_at: chrono::Utc::now(),
            open_issue_total: issues,
            open_issues: vec![],
            open_prs: vec![],
        }
    }

    #[test]
    fn default_rules_are_the_reconciler_set_and_pr_is_the_pr_check() {
        assert_eq!(github_rules(None, &[]).expect("fixture"), ghc::Rule::RECONCILER);
        assert_eq!(
            github_rules(Some(3), &[]).expect("fixture"),
            [ghc::Rule::R3, ghc::Rule::R6]
        );
        assert_eq!(
            github_rules(None, &["r1".into(), "R6".into()]).expect("fixture"),
            [ghc::Rule::R1, ghc::Rule::R6]
        );
        assert!(github_rules(None, &["R5".into()]).is_err());
    }

    #[test]
    fn report_is_labelled_with_its_source_and_fails_on_a_violation() {
        let p = ghc::Policy::default();
        let r = github_report(&p, &snap(101), None, &[], "snapshot-file").expect("fixture");
        assert_eq!(r.source, "snapshot-file");
        assert!(!r.ok());
        assert!(github_report(&p, &snap(100), None, &[], "x").expect("fixture").ok());
    }

    #[test]
    fn snapshot_file_for_another_repo_is_refused() {
        let dir = tempfile::tempdir().expect("fixture");
        let file = dir.path().join("s.json");
        std::fs::write(&file, serde_json::to_string(&snap(1)).expect("fixture")).expect("fixture");
        let ok = github_snapshot(dir.path(), Some("PAIML/x"), Some(&file)).expect("fixture");
        assert_eq!(ok.1, "snapshot-file");
        assert!(github_snapshot(dir.path(), Some("paiml/y"), Some(&file)).is_err());
    }
}
