//! The enforcement graph: required-check contexts on one side, invocations of
//! the rule on the other, and every edge a failure would have to cross.
//!
//! Three layers:
//!
//! ```text
//! root (a required status check CONTEXT STRING)
//!   -> the job that reports it
//!     -> the jobs it `needs`
//!       -> the invocations inside them
//! ```
//!
//! Roots are **unioned**: a rule is enforced if *some* required context reaches
//! it, and the answer is computed once over the whole graph rather than context
//! by context, so a repository with four required checks cannot be reported
//! compliant four separate times on the strength of the one that happens to be
//! healthy.
//!
//! The reachability decision itself is not taken here — it is
//! [`super::kernel::reachable`], which is proved by `KANI-2100-1`.

use super::invocation::Invocation;
use super::kernel::{self, Edge};
use super::reach;
use super::resolve::Resolution;
use super::workflow::{Job, WorkflowSet};
use std::collections::HashMap;
use std::path::PathBuf;

/// The graph, plus everything needed to explain a verdict drawn from it.
#[derive(Debug, Clone, Default)]
pub struct Enforcement {
    pub node_count: usize,
    pub edges: Vec<Edge>,
    pub roots: Vec<usize>,
    /// Human label per node, parallel to node indices.
    pub labels: Vec<String>,
    /// `(node index, index into the invocation list)`.
    pub invocation_nodes: Vec<(usize, usize)>,
}

impl Enforcement {
    /// Is any invocation reachable from the union of the required contexts?
    pub fn any_invocation_reachable(&self) -> bool {
        self.invocation_nodes
            .iter()
            .any(|(node, _)| kernel::reachable(self.node_count, &self.edges, &self.roots, *node))
    }

    /// The invocations one specific required context reaches. Used by the
    /// ledger to say *which* gate carries a rule, and to name the required
    /// contexts that carry nothing at all.
    pub fn invocations_from_root(&self, root: usize) -> Vec<usize> {
        self.invocation_nodes
            .iter()
            .filter(|(node, _)| kernel::reachable(self.node_count, &self.edges, &[root], *node))
            .map(|(_, inv)| *inv)
            .collect()
    }
}

/// Node keys for jobs, so the same job reached from two contexts is one node.
type JobKey = (PathBuf, String);

struct Builder<'a> {
    set: &'a WorkflowSet,
    graph: Enforcement,
    jobs: HashMap<JobKey, usize>,
}

/// Build the graph for one set of required contexts.
pub fn build(
    set: &WorkflowSet,
    resolutions: &[(String, Resolution)],
    invocations: &[Invocation],
) -> Enforcement {
    let mut b = Builder {
        set,
        graph: Enforcement::default(),
        jobs: HashMap::new(),
    };
    for (context, _) in resolutions {
        let root = b.add_node(format!("required check `{context}`"));
        b.graph.roots.push(root);
    }
    for (i, (_, resolution)) in resolutions.iter().enumerate() {
        b.add_resolution(b.graph.roots[i], resolution);
    }
    b.add_invocations(invocations);
    b.graph
}

impl Builder<'_> {
    fn add_node(&mut self, label: String) -> usize {
        self.graph.labels.push(label);
        self.graph.node_count = self.graph.labels.len();
        self.graph.node_count - 1
    }

    fn job_node(&mut self, job: &Job) -> usize {
        let key: JobKey = (job.workflow.clone(), job.id.clone());
        if let Some(n) = self.jobs.get(&key) {
            return *n;
        }
        let n = self.add_node(format!("job `{}` in {}", job.id, job.workflow.display()));
        self.jobs.insert(key, n);
        n
    }

    /// Wire one required context into the graph, then walk its `needs` closure.
    fn add_resolution(&mut self, root: usize, resolution: &Resolution) {
        let Resolution::Job { workflow, job_id } = resolution else {
            return; // Phantom and Opaque contribute no edges — and are holes.
        };
        let Some(job) = self.set.job(workflow, job_id) else {
            return;
        };
        let node = self.job_node(job);
        self.graph.edges.push(edge(root, node, alive(job)));
        self.add_needs(job);
    }

    fn add_needs(&mut self, root_job: &Job) {
        let closure = reach::reachable_jobs(self.set, &root_job.workflow, &root_job.id);
        for r in &closure {
            let from = self.job_node(r.job);
            for need in &r.job.needs {
                let Some(next) = self.set.job(&r.job.workflow, need) else {
                    continue;
                };
                let to = self.job_node(next);
                let live = alive(next) && reach::broken_edge(r.job, need).is_none();
                self.graph.edges.push(edge(from, to, live));
            }
        }
    }

    /// An invocation hangs off its own job, and the edge is live exactly when
    /// nothing suppressed it. `Invocation::suppressions` already carries the
    /// job-level and edge-level reasons folded in by the closure walk, so a
    /// neutered invocation is a dead leaf however it was neutered.
    fn add_invocations(&mut self, invocations: &[Invocation]) {
        for (i, inv) in invocations.iter().enumerate() {
            let key: JobKey = (inv.workflow.clone(), inv.job_id.clone());
            let Some(&from) = self.jobs.get(&key) else {
                continue; // an invocation in no reachable job is not evidence
            };
            let node = self.add_node(format!(
                "invocation in {}:{} step `{}`",
                inv.workflow.display(),
                inv.job_id,
                inv.step
            ));
            self.graph
                .edges
                .push(edge(from, node, inv.suppressions.is_empty()));
            self.graph.invocation_nodes.push((node, i));
        }
    }
}

/// A job whose own `continue-on-error` is set cannot fail the run, so no
/// failure crosses the edge into it.
fn alive(job: &Job) -> bool {
    !job.continue_on_error.suppresses()
}

fn edge(from: usize, to: usize, live: bool) -> Edge {
    if live {
        Edge::live(from, to)
    } else {
        Edge::dead(from, to)
    }
}
