//! `pmat comply github --org`: judge every repository of an organisation in one run.
//! A repository that cannot be read is a NOT MEASURED row, never 0 findings.

use super::source::{fetch_snapshot, list_org_repos, Graphql};
use super::{evaluate, Policy, Report, Rule};
use anyhow::Result;
use chrono::{DateTime, Utc};

/// One [`Report`] per judged repository.
#[derive(Debug, Clone, PartialEq)]
pub struct OrgReport {
    pub org: String,
    pub repos: Vec<Report>,
}

impl OrgReport {
    /// (compliant, violating, not_measured) repository counts.
    pub fn counts(&self) -> (usize, usize, usize) {
        let n = |f: fn(&Report) -> bool| self.repos.iter().filter(|r| f(r)).count();
        (
            n(|r| r.ok()),
            n(|r| !r.violations.is_empty()),
            n(|r| !r.not_measured.is_empty()),
        )
    }

    /// Clean only when something was judged, nothing violated and nothing went unmeasured.
    pub fn ok(&self) -> bool {
        let (_, v, nm) = self.counts();
        !self.repos.is_empty() && v + nm == 0
    }

    pub fn render_text(&self) -> String {
        let mut out = format!("GHC-001 organisation {}\n", self.org);
        for r in &self.repos {
            let n = r.violations.len();
            let row = match r.not_measured.is_empty() {
                true => format!("{n} findings"),
                false => format!("not_measured: {}", r.not_measured.join("; ")),
            };
            out += &format!("  {:<40} {row}\n", r.repo);
        }
        let (c, v, nm) = self.counts();
        out + &format!("  totals: {c} compliant, {v} violating, {nm} not_measured\n")
    }
}

/// Judge every non-archived repository of `org`.
pub fn evaluate_org(
    g: &dyn Graphql,
    org: &str,
    policy: &Policy,
    rules: &[Rule],
    now: DateTime<Utc>,
) -> Result<OrgReport> {
    let mut repos = Vec::new();
    for name in list_org_repos(g, org)? {
        repos.push(match fetch_snapshot(g, &name, now) {
            Ok(snap) => evaluate(&snap, policy, rules),
            Err(e) => Report {
                repo: name,
                source: "github-graphql".to_string(),
                rules_run: rules.to_vec(),
                violations: vec![],
                not_measured: vec![format!("{e:#}")],
            },
        });
    }
    Ok(OrgReport {
        org: org.to_string(),
        repos,
    })
}
