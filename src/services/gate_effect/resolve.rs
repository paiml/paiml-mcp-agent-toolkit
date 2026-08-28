//! Resolve a required branch-protection **context string** to the job that
//! produces it.
//!
//! INV-2100-3 lives here: resolution is driven by the context string, never by
//! a job's display name. `gate` and `ci / gate` are different contexts, and a
//! repo can have both — one required, one not.
//!
//! The match itself is [`super::kernel::select_by_context`], which takes the
//! display name as an argument it never reads — so `KANI-2100-2` can prove the
//! display name cannot influence the answer, instead of the module asserting it
//! in a comment.

use super::kernel;
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

/// One thing a required context string could name.
///
/// `display` is carried alongside `context` on purpose. For a top-level job the
/// two coincide; for a job inside a reusable workflow they do not, and the gap
/// between them is the defect INV-2100-3 exists to catch. Keeping both means
/// the matcher can be *shown* to use one and not the other.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub context: String,
    pub display: String,
    pub resolution: Resolution,
}

/// Every context string this repository's workflows can produce, paired with
/// the resolution it maps to.
///
/// A caller job whose callee is external contributes no concrete contexts (it
/// is not enumerable), so it is matched by prefix in [`resolve_context`].
pub fn enumerate_contexts(set: &WorkflowSet) -> Vec<Candidate> {
    let mut out = Vec::new();
    for job in set.jobs() {
        match job.uses.as_deref() {
            None => out.push(Candidate {
                context: job.context().to_string(),
                display: job.display_name.clone().unwrap_or_else(|| job.id.clone()),
                resolution: Resolution::Job {
                    workflow: job.workflow.clone(),
                    job_id: job.id.clone(),
                },
            }),
            Some(uses) => push_reusable_contexts(set, job, uses, &mut out),
        }
    }
    out
}

fn push_reusable_contexts(set: &WorkflowSet, caller: &Job, uses: &str, out: &mut Vec<Candidate>) {
    let Some(callee_path) = local_reusable_path(uses) else {
        return; // external: not enumerable, handled by prefix match
    };
    let Some(callee) = set.workflows.iter().find(|w| w.path == callee_path) else {
        return; // unreadable local callee: treated as opaque by prefix match
    };
    for job in &callee.jobs {
        out.push(Candidate {
            context: format!("{} / {}", caller.context(), job.context()),
            display: job.display_name.clone().unwrap_or_else(|| job.id.clone()),
            resolution: Resolution::Job {
                workflow: job.workflow.clone(),
                job_id: job.id.clone(),
            },
        });
    }
}

/// Resolve one required context string.
#[provable_contracts_macros::contract(
    "comply-gate-effect-v1.yaml",
    equation = "context_string_resolution"
)]
pub fn resolve_context(set: &WorkflowSet, context: &str) -> Resolution {
    let candidates = enumerate_contexts(set);
    let pairs: Vec<(String, String)> = candidates
        .iter()
        .map(|c| (c.context.clone(), c.display.clone()))
        .collect();
    if let Some(i) = kernel::select_by_context(&pairs, &context.to_string()) {
        return candidates[i].resolution.clone();
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
