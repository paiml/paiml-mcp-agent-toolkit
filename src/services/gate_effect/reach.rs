//! Which jobs a required check's failure can actually come from.
//!
//! A required job fails when one of its own steps fails, and — by default —
//! when any job in its `needs` closure fails, because GitHub skips a dependent
//! job whose dependency failed and a skipped required check is not a success.
//!
//! `if: always()` breaks that. A job that runs unconditionally reports success
//! even when everything it needed failed, *unless* it inspects
//! `needs.<id>.result` itself. This repo's own `gate` job does exactly that
//! inspection — which is why the edge has to be judged, not assumed.

use super::workflow::{Job, WorkflowSet};
use std::collections::HashSet;
use std::path::Path;

/// A job whose failure can (or provably cannot) reach the required context.
#[derive(Debug, Clone)]
pub struct ReachableJob<'a> {
    pub job: &'a Job,
    /// Reasons a failure here cannot reach the required check. Empty ⇒ it can.
    pub suppressions: Vec<String>,
}

/// Breadth-first walk of the required job and its `needs` closure.
pub fn reachable_jobs<'a>(
    set: &'a WorkflowSet,
    workflow: &Path,
    job_id: &str,
) -> Vec<ReachableJob<'a>> {
    let mut out: Vec<ReachableJob<'a>> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let Some(root) = set.job(workflow, job_id) else {
        return out;
    };
    let mut queue: Vec<(&Job, Vec<String>)> = vec![(root, Vec::new())];

    while let Some((job, inherited)) = queue.pop() {
        if !seen.insert(job.id.clone()) {
            continue;
        }
        let mut suppressions = inherited.clone();
        if job.continue_on_error.suppresses() {
            suppressions.push(format!(
                "job `{}` in {} carries continue-on-error, so its failure never fails the run",
                job.id,
                job.workflow.display()
            ));
        }
        for need in &job.needs {
            let Some(next) = set.job(workflow, need) else {
                continue;
            };
            let mut edge = suppressions.clone();
            if let Some(reason) = broken_edge(job, need) {
                edge.push(reason);
            }
            queue.push((next, edge));
        }
        out.push(ReachableJob { job, suppressions });
    }
    out
}

/// `Some(reason)` when a failure of `need` cannot fail `job`.
fn broken_edge(job: &Job, need: &str) -> Option<String> {
    let cond = job.if_expr.as_deref()?;
    if !cond.contains("always()") {
        return None;
    }
    let probe = format!("needs.{need}.result");
    if job.run_scripts().any(|s| s.contains(&probe)) {
        return None;
    }
    Some(format!(
        "job `{}` runs under `if: {}` and never inspects {probe}, so a failure of `{need}` \
         leaves it green",
        job.id,
        cond.trim()
    ))
}
