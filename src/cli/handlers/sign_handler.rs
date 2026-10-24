use crate::cli::commands::SignArgs;
use crate::cli_exit::{with_code, ExitCode};
use crate::services::sign::{self, Inputs, SignError};
use anyhow::{anyhow, Result};

/// Real head lookup via `gh` (not used by tests, which inject a fixture).
fn gh_head(repo: &str, pr: u64) -> Result<String, String> {
    let o = std::process::Command::new("gh")
        .args([
            "pr",
            "view",
            &pr.to_string(),
            "-R",
            repo,
            "--json",
            "headRefOid",
            "-q",
            ".headRefOid",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).into_owned())
    }
}

fn git(dir: &std::path::Path, args: &[&str]) -> Result<String, SignError> {
    let o = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output();
    match o {
        Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).into_owned()),
        _ => Err(SignError::NotMeasured(format!("git {args:?} failed"))),
    }
}

pub fn handle_sign_command(a: &SignArgs) -> Result<()> {
    let pr = a
        .rearm
        .or(a.pr)
        .ok_or_else(|| anyhow!("--pr or --rearm <pr> is required"))?;
    if a.rearm.is_some() && !sign::rearm_authorized(&a.rulings, pr) {
        return Err(anyhow!(
            "refused: no `ALFREDO <UTC> — …` rearm line naming PR {pr} in {}",
            a.rulings.display()
        ));
    }
    let run = || -> Result<sign::SignFile, SignError> {
        let range = format!("{}..{}", a.base, a.head);
        let ids: Vec<String> = git(&a.repo_dir, &["rev-list", "--reverse", &range])?
            .lines()
            .map(|c| sign::patch_id_u0(&a.repo_dir, c))
            .collect::<Result<_, _>>()?;
        let msgs = git(&a.repo_dir, &["log", "--format=%B", &range])?;
        let mut authors = a.author_models.clone();
        authors.extend(sign::co_authored_models(&[msgs]));
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        sign::sign(&Inputs {
            repo: &a.repo,
            pr,
            root: &a.root,
            patch_ids: &ids,
            head_lookup: &gh_head,
            lane_roster: &a.lanes,
            required_seats: &a.seats,
            author_models: &authors,
            now: &now,
            rearm: a.rearm.is_some(),
        })
    };
    match run() {
        Ok(sf) => {
            println!("{}", serde_json::to_string_pretty(&sf)?);
            Ok(())
        }
        // Exit 3 = not measured (cop's spec); exit 1 = refused.
        Err(SignError::NotMeasured(m)) => Err(with_code(
            ExitCode::QualityGateFailure,
            anyhow!("not measured: {m}"),
        )),
        Err(SignError::Refused(r)) => Err(anyhow!("refused: {r:?}")),
    }
}
