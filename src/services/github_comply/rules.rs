//! The GHC-001 rules themselves: pure functions over a [`RepoSnapshot`].

use super::{CheckState, IssueRec, Policy, PrRec, RepoSnapshot, Report, Rule, Violation};
use anyhow::{bail, Result};
use chrono::Duration;
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::LazyLock;

/// Violations printed per rule before the text report summarises the rest.
pub const SHOWN_PER_RULE: usize = 20;

impl Report {
    /// True when no rule was violated and nothing went unmeasured.
    pub fn ok(&self) -> bool {
        self.violations.is_empty() && self.not_measured.is_empty()
    }

    /// The human report. At most [`SHOWN_PER_RULE`] violations print per rule.
    pub fn render_text(&self) -> String {
        let mut out = format!("GitHub comply (GHC-001) — {}", self.repo);
        if !self.source.is_empty() {
            out.push_str(&format!(" [source: {}]", self.source));
        }
        out.push('\n');
        let ids: Vec<&str> = self.rules_run.iter().map(|r| r.id()).collect();
        out.push_str(&format!("  rules run: {}\n", ids.join(", ")));
        for rule in &self.rules_run {
            let mine: Vec<&Violation> =
                self.violations.iter().filter(|v| v.rule == *rule).collect();
            for v in mine.iter().take(SHOWN_PER_RULE) {
                let at = v.subject.map(|n| format!(" #{n}")).unwrap_or_default();
                out.push_str(&format!("  VIOLATION {}{at}: {}\n", v.rule.id(), v.message));
            }
            if mine.len() > SHOWN_PER_RULE {
                out.push_str(&format!(
                    "  ... and {} more {} violations\n",
                    mine.len() - SHOWN_PER_RULE,
                    rule.id()
                ));
            }
        }
        for m in &self.not_measured {
            out.push_str(&format!("  NOT MEASURED: {m}\n"));
        }
        out.push_str(&format!("  verdict: {}\n", self.verdict()));
        out
    }

    fn verdict(&self) -> String {
        if !self.violations.is_empty() {
            format!("VIOLATION ({} found)", self.violations.len())
        } else if !self.not_measured.is_empty() {
            "NOT MEASURED".to_string()
        } else {
            "COMPLIANT".to_string()
        }
    }
}

fn new_report(snap: &RepoSnapshot, rules: &[Rule]) -> Report {
    let rules_run: BTreeSet<Rule> = rules.iter().copied().collect();
    Report {
        repo: snap.repo.clone(),
        source: String::new(),
        rules_run: rules_run.into_iter().collect(),
        violations: vec![],
        not_measured: vec![],
    }
}

fn violation(rule: Rule, subject: Option<u64>, message: String) -> Violation {
    Violation {
        rule,
        subject,
        message,
    }
}

/// Judge `rules` over the whole snapshot.
pub fn evaluate(snap: &RepoSnapshot, policy: &Policy, rules: &[Rule]) -> Report {
    let mut report = new_report(snap, rules);
    for rule in report.rules_run.clone() {
        let found = match rule {
            Rule::R1 => check_r1(snap, policy),
            Rule::R2 => check_r2(snap, policy),
            Rule::R3 => snap.open_issues.iter().filter_map(check_r3_issue).collect(),
            Rule::R4 => check_r4(snap, policy),
            Rule::R5 => vec![], // creation-time only: see `guard_new_ticket`
            Rule::R6 => snap.open_prs.iter().filter_map(check_r6_pr).collect(),
        };
        report.violations.extend(found);
    }
    report
}

fn check_r1(snap: &RepoSnapshot, policy: &Policy) -> Vec<Violation> {
    let open = snap.open_issue_total.max(snap.open_issues.len());
    if open <= policy.max_open_issues {
        return vec![];
    }
    vec![violation(
        Rule::R1,
        None,
        format!(
            "{open} open issues exceed the cap of {} (target {})",
            policy.max_open_issues, policy.target_open_issues
        ),
    )]
}

/// `*` matches any run of characters; everything else is literal; anchored at both ends.
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let Some((first, rest)) = parts.split_first() else {
        return false;
    };
    let Some(mut tail) = text.strip_prefix(first) else {
        return false;
    };
    let Some((last, middle)) = rest.split_last() else {
        return tail.is_empty();
    };
    for mid in middle {
        match tail.find(mid) {
            Some(i) => tail = &tail[i + mid.len()..],
            None => return false,
        }
    }
    tail.ends_with(last)
}

fn is_exempt(pr: &PrRec, policy: &Policy) -> bool {
    policy
        .pr_exempt
        .iter()
        .any(|g| glob_match(g, &pr.head_ref) || pr.labels.iter().any(|l| glob_match(g, l)))
}

fn check_r2(snap: &RepoSnapshot, policy: &Policy) -> Vec<Violation> {
    let counted: Vec<u64> = snap
        .open_prs
        .iter()
        .filter(|p| !p.is_draft && !is_exempt(p, policy))
        .map(|p| p.number)
        .collect();
    if counted.len() <= policy.max_open_prs {
        return vec![];
    }
    let listed: Vec<String> = counted.iter().map(|n| format!("#{n}")).collect();
    vec![violation(
        Rule::R2,
        None,
        format!(
            "{} open non-draft PRs exceed the cap of {} (counted: {})",
            counted.len(),
            policy.max_open_prs,
            listed.join(" ")
        ),
    )]
}

fn check_r3_issue(issue: &IssueRec) -> Option<Violation> {
    if issue.milestone.is_some() || issue.parent.is_some() {
        return None;
    }
    Some(violation(
        Rule::R3,
        Some(issue.number),
        format!(
            "\"{}\" has no milestone and is not a sub-issue of an epic",
            issue.title
        ),
    ))
}

fn check_r4(snap: &RepoSnapshot, policy: &Policy) -> Vec<Violation> {
    let limit = Duration::hours(policy.pr_max_age_hours);
    snap.open_prs
        .iter()
        .filter(|p| !p.is_draft && p.checks != CheckState::Pass)
        .filter(|p| snap.taken_at - p.created_at > limit)
        .map(|p| {
            let hours = (snap.taken_at - p.created_at).num_hours();
            violation(
                Rule::R4,
                Some(p.number),
                format!(
                    "open {hours} h (max {}) with checks {:?}, no PASS",
                    policy.pr_max_age_hours, p.checks
                ),
            )
        })
        .collect()
}

fn check_r6_pr(pr: &PrRec) -> Option<Violation> {
    if has_issue_reference(&pr.body) {
        return None;
    }
    Some(violation(
        Rule::R6,
        Some(pr.number),
        "body has no `Closes|Fixes|Resolves owner/repo#N` or `Part of owner/repo#N`".into(),
    ))
}

/// `Closes|Fixes|Resolves|Part of owner/repo#N` — the keyword (any tense, any case, optional
/// colon) must be followed by whitespace and a FULL `owner/repo#N`: a bare `#N` names no repo.
static ISSUE_REF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:^|[^A-Za-z0-9_])(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?|part\s+of):?\s+([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)#([0-9]+)",
    )
    .expect("ISSUE_REF is a valid regex")
});

/// R6 predicate: does `body` carry a closing or "Part of" reference to `owner/repo#N`?
pub fn has_issue_reference(body: &str) -> bool {
    ISSUE_REF.is_match(body)
}

/// Every `(owner/repo, N)` the body links to, in order, without repeats.
pub fn issue_references(body: &str) -> Vec<(String, u64)> {
    let mut seen = Vec::new();
    for c in ISSUE_REF.captures_iter(body) {
        let Ok(n) = c[2].parse::<u64>() else { continue };
        let r = (c[1].to_string(), n);
        if !seen.contains(&r) {
            seen.push(r);
        }
    }
    seen
}

/// The PR check: R3 (every ticket the PR references) and R6 (the PR body), for one PR.
pub fn evaluate_pr(snap: &RepoSnapshot, _policy: &Policy, pr: u64) -> Report {
    let mut report = new_report(snap, &[Rule::R3, Rule::R6]);
    let Some(p) = snap.open_prs.iter().find(|p| p.number == pr) else {
        report
            .not_measured
            .push(format!("PR #{pr} is not among {}'s open PRs", snap.repo));
        return report;
    };
    report.violations.extend(check_r6_pr(p));
    for (repo, n) in issue_references(&p.body) {
        if !repo.eq_ignore_ascii_case(&snap.repo) {
            report
                .not_measured
                .push(format!("R3: {repo}#{n} is in another repository"));
        } else if let Some(issue) = snap.open_issues.iter().find(|i| i.number == n) {
            report.violations.extend(check_r3_issue(issue));
        } else {
            report
                .not_measured
                .push(format!("R3: {repo}#{n} is not an open issue"));
        }
    }
    report
}

const STOPWORDS: [&str; 14] = [
    "a", "an", "the", "of", "to", "in", "on", "for", "and", "or", "with", "is", "are", "be",
];

fn tokens(title: &str) -> BTreeSet<String> {
    title
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty() && !STOPWORDS.contains(t))
        .map(str::to_string)
        .collect()
}

fn jaccard(a: &BTreeSet<String>, b: &BTreeSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    a.intersection(b).count() as f64 / a.union(b).count() as f64
}

/// Do the two titles share at least one non-stopword token?
pub fn titles_share_a_word(a: &str, b: &str) -> bool {
    jaccard(&tokens(a), &tokens(b)) > 0.0
}

/// R5: the open ticket in `issues` that `title` duplicates, if any: the best title
/// match at or above `threshold` (token Jaccard). `exclude` is the ticket being filed
/// (already open on GitHub, so it must not match itself).
pub fn find_fitting_issue<'a>(
    title: &str,
    issues: &'a [IssueRec],
    exclude: Option<u64>,
    threshold: f64,
) -> Option<(&'a IssueRec, f64)> {
    let mine = tokens(title);
    issues
        .iter()
        .filter(|i| Some(i.number) != exclude)
        .map(|i| (i, jaccard(&mine, &tokens(&i.title))))
        .filter(|(_, s)| *s >= threshold && *s > 0.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// `pmat work add`'s guards R1, R3 and R5 — Ok when the ticket may be filed.
///
/// `child` is the ticket's own GitHub issue (already open, so it is excluded from the
/// R1 count and the R5 search); `epic` is the epic it will be linked under.
pub fn guard_new_ticket(
    policy: &Policy,
    snap: &RepoSnapshot,
    title: &str,
    child: Option<u64>,
    epic: Option<u64>,
) -> Result<()> {
    let child_listed = child.is_some_and(|c| snap.open_issues.iter().any(|i| i.number == c));
    let others = snap
        .open_issue_total
        .max(snap.open_issues.len())
        .saturating_sub(usize::from(child_listed));
    if others >= policy.max_open_issues {
        bail!(
            "refused (R1): {} already has {others} open issues — the cap is {} (target {}); close or merge tickets before filing another",
            snap.repo,
            policy.max_open_issues,
            policy.target_open_issues
        );
    }
    if let Some((fit, score)) =
        find_fitting_issue(title, &snap.open_issues, child, policy.fit_similarity)
    {
        bail!(
            "refused (R5): open ticket #{} \"{}\" already fits this title (similarity {score:.2}) — extend it instead of filing a new one",
            fit.number,
            fit.title
        );
    }
    let has_home = epic.is_some()
        || snap
            .open_issues
            .iter()
            .any(|i| Some(i.number) == child && (i.milestone.is_some() || i.parent.is_some()));
    if !has_home {
        bail!("refused (R3): the ticket has no milestone and no epic to be a sub-issue of");
    }
    Ok(())
}
