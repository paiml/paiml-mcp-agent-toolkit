//! Resolve a required branch-protection **context string** to the job that
//! produces it.
//!
//! INV-1400-3 lives here: resolution is driven by the context string, never by
//! a job's display name. `gate` and `ci / gate` are different contexts, and a
//! repo can have both — one required, one not.

use super::workflow::{Job, WorkflowSet};
use std::path::PathBuf;

/// What a required context string points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// A job in this repository's own workflows, readable in full.
    Job { workflow: PathBuf, job_id: String },
    /// A job inside a reusable workflow hosted in another repository. The
    /// caller is known; the callee's steps are not readable from here.
    Opaque {
        caller_workflow: PathBuf,
        caller_job: String,
        reference: String,
    },
    /// Nothing in `.github/workflows` can ever report this context. A required
    /// check that no workflow produces never turns green on its own — it is a
    /// phantom, and it is reported as a hole, not as an absence of evidence.
    Phantom,
}

/// A reusable-workflow reference that lives in this repository
/// (`./.github/workflows/x.yml`), as opposed to `owner/repo/...@ref`.
fn local_reusable_path(uses: &str) -> Option<PathBuf> {
    let trimmed = uses.trim();
    let rest = trimmed.strip_prefix("./")?;
    Some(PathBuf::from(rest))
}

/// Every context string this repository's workflows can produce, paired with
/// the resolution it maps to.
///
/// A caller job whose callee is external contributes no concrete contexts (it
/// is not enumerable), so it is matched by prefix in [`resolve_context`].
pub fn enumerate_contexts(set: &WorkflowSet) -> Vec<(String, Resolution)> {
    let mut out = Vec::new();
    for job in set.jobs() {
        match job.uses.as_deref() {
            None => out.push((
                job.context().to_string(),
                Resolution::Job {
                    workflow: job.workflow.clone(),
                    job_id: job.id.clone(),
                },
            )),
            Some(uses) => push_reusable_contexts(set, job, uses, &mut out),
        }
    }
    out
}

fn push_reusable_contexts(
    set: &WorkflowSet,
    caller: &Job,
    uses: &str,
    out: &mut Vec<(String, Resolution)>,
) {
    let Some(callee_path) = local_reusable_path(uses) else {
        return; // external: not enumerable, handled by prefix match
    };
    let Some(callee) = set.workflows.iter().find(|w| w.path == callee_path) else {
        return; // unreadable local callee: treated as opaque by prefix match
    };
    for job in &callee.jobs {
        out.push((
            format!("{} / {}", caller.context(), job.context()),
            Resolution::Job {
                workflow: job.workflow.clone(),
                job_id: job.id.clone(),
            },
        ));
    }
}

/// Resolve one required context string.
pub fn resolve_context(set: &WorkflowSet, context: &str) -> Resolution {
    if let Some((_, r)) = enumerate_contexts(set)
        .into_iter()
        .find(|(c, _)| c == context)
    {
        return r;
    }
    opaque_prefix_match(set, context).unwrap_or(Resolution::Phantom)
}

/// `ci / gate` where job `ci` calls an unreadable reusable workflow: the caller
/// is identified, the callee is not. Deliberately does not become a Pass —
/// [`super::analyze`] treats `Opaque` as "cannot be shown to invoke anything".
fn opaque_prefix_match(set: &WorkflowSet, context: &str) -> Option<Resolution> {
    for job in set.jobs() {
        let Some(uses) = job.uses.as_deref() else {
            continue;
        };
        let prefix = format!("{} / ", job.context());
        if context.starts_with(&prefix) {
            return Some(Resolution::Opaque {
                caller_workflow: job.workflow.clone(),
                caller_job: job.id.clone(),
                reference: uses.to_string(),
            });
        }
    }
    None
}
