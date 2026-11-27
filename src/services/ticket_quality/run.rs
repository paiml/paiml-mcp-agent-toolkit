//! The two verbs, end to end over a [`Graphql`] seam: `pmat work score --issue N` and
//! `pmat work findings`. Each returns what to print and the process exit code.

use super::findings::{self, Evidence, Settings};
use super::{rubric, source};
use crate::services::github_comply::source::Graphql;
use chrono::{DateTime, Utc};

/// Exit code of a run that could not read GitHub: not measured, never a pass.
pub const EXIT_NOT_MEASURED: i32 = 2;

fn not_measured(what: &str, err: &anyhow::Error) -> (String, i32) {
    (
        format!("not_measured: {what}: {err:#}\n"),
        EXIT_NOT_MEASURED,
    )
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_default();
    text.push('\n');
    text
}

/// `pmat work score --issue N`: 0 at the pass mark or over, 1 under it, 2 on a read fault.
pub fn run_score(
    g: &dyn Graphql,
    repo: &str,
    issue: u64,
    pass_mark: u32,
    json: bool,
) -> (String, i32) {
    let ticket = match source::fetch_ticket(g, repo, issue) {
        Ok(t) => t,
        Err(e) => return not_measured(&format!("{repo}#{issue}"), &e),
    };
    let score = rubric::score(&ticket, pass_mark);
    let text = if json {
        to_json(&score)
    } else {
        format!("{repo}#{issue} {score}")
    };
    (text, i32::from(!score.passed))
}

/// `pmat work findings`: 0 when the repository was read, 2 when it was not.
pub fn run_findings(
    g: &dyn Graphql,
    repo: &str,
    settings: &Settings,
    json: bool,
    now: DateTime<Utc>,
) -> (String, i32) {
    let read = source::fetch_tickets(g, repo).and_then(|(total, tickets)| {
        let (gone, notes) = source::gone_paths(g, repo, &tickets)?;
        Ok((total, tickets, gone, notes))
    });
    let (open_total, tickets, gone, notes) = match read {
        Ok(r) => r,
        Err(e) => return not_measured(repo, &e),
    };
    let report = findings::evaluate(
        Evidence {
            repo,
            now,
            open_total,
            tickets: &tickets,
            gone_paths: &gone,
            not_measured: notes,
        },
        settings,
    );
    let text = if json {
        to_json(&report)
    } else {
        report.render_text()
    };
    (text, 0)
}
