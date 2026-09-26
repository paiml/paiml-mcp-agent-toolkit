//! FLOW-03 (#1440): `pmat work add` files a ticket under an epic, with a
//! priority and a kind, or does not file it at all.
//!
//! Before this, `pmat work add "x" --github-issue N` succeeded with no epic
//! and no priority (measured in paiml/infra 2026-09-25, PMAT-1115), so an
//! untriaged ticket was indistinguishable from a triaged one. The gate here
//! runs before anything is written, and the GitHub sub-issue link is made
//! before the row, so a row that says `epic: E` always has the link behind it.
//!
//! No milestone is asked for: defects and P0 are never refused for lack of one
//! (FLOW-001 §0.4).

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::commands::{WorkKind, WorkPriority};

/// What the operator passed for triage. Every field is required; `kind` may
/// instead arrive as a `kind:<x>` tag.
#[derive(Debug, Clone, Default)]
pub struct WorkAddTriage {
    pub epic: Option<u64>,
    pub priority: Option<WorkPriority>,
    pub kind: Option<WorkKind>,
}

/// A triage that passed the gate.
#[derive(Debug, Clone, PartialEq)]
pub struct Triaged {
    pub epic: u64,
    pub child: u64,
    pub priority: WorkPriority,
    pub kind: WorkKind,
}

/// What GitHub says about one issue.
#[derive(Debug, Clone, PartialEq)]
pub struct IssueFacts {
    /// The REST id (not the number) — what the sub-issue API takes.
    pub id: u64,
    pub open: bool,
    pub is_pull_request: bool,
    pub labels: Vec<String>,
}

/// The three GitHub calls the gate and the link need, so tests can stand in
/// for GitHub.
pub trait EpicLinks {
    /// Facts about issue `number` in this repository.
    fn issue(&self, number: u64) -> Result<IssueFacts>;
    /// Numbers of the issues already linked under `epic`.
    fn sub_issues(&self, epic: u64) -> Result<Vec<u64>>;
    /// Link the issue whose REST id is `child_id` under `epic`.
    fn link(&self, epic: u64, child_id: u64) -> Result<()>;
}

/// The `kind:` a ticket carries: `--kind`, else a `kind:<x>` tag. Both given
/// and disagreeing is refused rather than guessed at.
pub fn resolve_kind(flag: Option<WorkKind>, tags: Option<&str>) -> Result<WorkKind> {
    let tagged = kind_tag(tags)?;
    match (flag, tagged) {
        (Some(f), Some(t)) if f != t => bail!(
            "refused: --kind {} and the tag kind:{} disagree — pass one kind",
            f.as_str(),
            t.as_str()
        ),
        (Some(k), _) | (None, Some(k)) => Ok(k),
        (None, None) => bail!(
            "refused: a ticket needs a kind — pass --kind code|triage|docs|measurement|lifecycle (FLOW-03, #1440)"
        ),
    }
}

fn kind_tag(tags: Option<&str>) -> Result<Option<WorkKind>> {
    let mut found = None;
    for tag in tags.unwrap_or("").split(',').map(str::trim) {
        let Some(value) = tag.strip_prefix("kind:") else {
            continue;
        };
        let kind = WorkKind::parse(value)
            .with_context(|| format!("refused: tag kind:{value} is not a kind"))?;
        if found.is_some_and(|k| k != kind) {
            bail!("refused: two different kind: tags — pass one kind");
        }
        found = Some(kind);
    }
    Ok(found)
}

/// Refuse an untriaged add. Reads GitHub, writes nothing.
pub fn gate(
    triage: &WorkAddTriage,
    tags: Option<&str>,
    github_issue: Option<u64>,
    links: &dyn EpicLinks,
) -> Result<Triaged> {
    let Some(priority) = triage.priority else {
        bail!("refused: a ticket needs a priority — pass --priority P0|P1|P2|P3 (FLOW-03, #1440)");
    };
    let kind = resolve_kind(triage.kind, tags)?;
    let Some(epic) = triage.epic else {
        bail!("refused: a ticket is filed under an epic — pass --epic <issue#>, an open issue labelled `epic` (FLOW-03, #1440)");
    };
    let Some(child) = github_issue else {
        bail!("refused: --epic #{epic} links the ticket's GitHub issue under the epic, and this ticket has none — pass --github-issue N");
    };
    if epic == child {
        bail!(
            "refused: --epic #{epic} is the ticket's own issue — an issue cannot be its own epic"
        );
    }
    check_epic(epic, &links.issue(epic)?)?;
    Ok(Triaged {
        epic,
        child,
        priority,
        kind,
    })
}

fn check_epic(epic: u64, facts: &IssueFacts) -> Result<()> {
    if facts.is_pull_request {
        bail!("refused: --epic #{epic} is a pull request, not an issue");
    }
    if !facts.open {
        bail!("refused: --epic #{epic} is closed — a ticket is filed under an open epic");
    }
    if !facts.labels.iter().any(|l| l.eq_ignore_ascii_case("epic")) {
        bail!(
            "refused: --epic #{epic} is not labelled `epic` (labels: {:?})",
            facts.labels
        );
    }
    Ok(())
}

/// Make GitHub hold the link the row is about to record. Idempotent: a child
/// already under this epic is not linked twice. Returns whether a link was
/// created.
pub fn ensure_linked(triaged: &Triaged, links: &dyn EpicLinks) -> Result<bool> {
    if links.sub_issues(triaged.epic)?.contains(&triaged.child) {
        return Ok(false);
    }
    let child = links.issue(triaged.child)?;
    links.link(triaged.epic, child.id).with_context(|| {
        format!(
            "refused: could not link #{} under epic #{} — nothing was written",
            triaged.child, triaged.epic
        )
    })?;
    Ok(true)
}

/// Tags with the ticket's `kind:` label added when no tag carries it yet.
pub fn tags_with_kind(tags: Option<&str>, kind: WorkKind) -> String {
    let label = format!("kind:{}", kind.as_str());
    let mut parts: Vec<&str> = tags
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
    if !parts.contains(&label.as_str()) {
        parts.push(&label);
    }
    parts.join(",")
}

/// `gh`, run in the project so `{owner}/{repo}` is the project's own repository.
pub struct GhCli {
    dir: PathBuf,
}

impl GhCli {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }

    fn api(&self, args: &[&str]) -> Result<String> {
        let out = Command::new("gh")
            .arg("api")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .context("could not run `gh` — FLOW-03 checks the epic on GitHub")?;
        if !out.status.success() {
            bail!(
                "gh api {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

impl EpicLinks for GhCli {
    fn issue(&self, number: u64) -> Result<IssueFacts> {
        let body = self.api(&[&format!("repos/{{owner}}/{{repo}}/issues/{number}")])?;
        parse_issue(&body).with_context(|| format!("issue #{number}: unexpected reply from gh"))
    }

    fn sub_issues(&self, epic: u64) -> Result<Vec<u64>> {
        let body = self.api(&[
            "--paginate",
            &format!("repos/{{owner}}/{{repo}}/issues/{epic}/sub_issues"),
            "--jq",
            ".[].number",
        ])?;
        body.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().parse::<u64>().context("sub-issue number"))
            .collect()
    }

    fn link(&self, epic: u64, child_id: u64) -> Result<()> {
        self.api(&[
            "-X",
            "POST",
            &format!("repos/{{owner}}/{{repo}}/issues/{epic}/sub_issues"),
            "-F",
            &format!("sub_issue_id={child_id}"),
        ])
        .map(|_| ())
    }
}

/// Read the fields the gate needs from a REST issue object.
pub fn parse_issue(body: &str) -> Result<IssueFacts> {
    let v: serde_json::Value = serde_json::from_str(body)?;
    let id = v["id"].as_u64().context("no numeric `id`")?;
    let state = v["state"].as_str().context("no `state`")?;
    let labels = v["labels"]
        .as_array()
        .map(|ls| {
            ls.iter()
                .filter_map(|l| l["name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    Ok(IssueFacts {
        id,
        open: state.eq_ignore_ascii_case("open"),
        is_pull_request: v.get("pull_request").is_some_and(|p| !p.is_null()),
        labels,
    })
}
