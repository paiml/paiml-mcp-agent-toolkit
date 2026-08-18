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
    poison_closure(&mut out);
    out
}

/// INV-2100-5, applied to the whole closure.
///
/// A job that can never succeed does not merely neuter itself: the required
/// check that depends on it can never go green either, so *nothing* in the
/// closure is producing a verdict anybody acts on. The reason is therefore
/// folded into every job, not just the guilty one.
fn poison_closure(jobs: &mut [ReachableJob<'_>]) {
    let poison: Vec<String> = jobs.iter().filter_map(|r| never_succeeds(r.job)).collect();
    if poison.is_empty() {
        return;
    }
    for r in jobs.iter_mut() {
        r.suppressions.extend(poison.iter().cloned());
    }
}

/// INV-2100-5: a job that can never succeed never produces a verdict.
///
/// The msrv/`--all-features` shape: a step whose script fails unconditionally
/// makes the job permanently red. A permanently red required check is not a
/// gate — it blocks everything, gets bypassed, and the rule it was supposed to
/// carry is never actually consulted.
pub fn never_succeeds(job: &Job) -> Option<String> {
    for script in job.run_scripts() {
        if let Some((_, line)) = super::effect::first_unconditional_failure(script) {
            return Some(format!(
                "job `{}` in {} can never succeed (`{line}` always fails), so it never produces a \
                 verdict",
                job.id,
                job.workflow.display()
            ));
        }
    }
    None
}

/// `Some(reason)` when a failure of `need` cannot fail `job`.
pub fn broken_edge(job: &Job, need: &str) -> Option<String> {
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
