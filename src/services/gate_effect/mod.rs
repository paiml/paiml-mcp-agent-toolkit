//! Gate-effect verification: does a required status check actually reach the
//! rules it is supposed to enforce?
//!
//! Every "this is enforced in CI" claim in this repository ultimately rests on
//! one chain:
//!
//! ```text
//! required status check CONTEXT STRING
//!   -> the job that reports that context
//!     -> a step (or a Makefile/script one hop away) that invokes the rule
//!       -> whose non-zero exit can still fail the job
//! ```
//!
//! Break any link and the rule is decoration. The seven invariants are:
//!
//! * **INV-2100-1** every severity=error rule is reachable from *some* required
//!   context — the roots are unioned, never taken one at a time;
//! * **INV-2100-2** a reachable invocation whose failure cannot propagate is
//!   NOT reachable (`¬continue_on_error ∧ ¬suppressed ∧ exit_code_compared`);
//! * **INV-2100-3** reachability is computed against the required-check CONTEXT
//!   STRING, not the job display name;
//! * **INV-2100-4** a command that prints a failure verdict and exits 0 does
//!   not gate;
//! * **INV-2100-5** an invocation that can never succeed does not gate;
//! * **INV-2100-6** a job that compiles tests without executing them does not
//!   establish reachability for those tests;
//! * **INV-2100-7** no gate name is hardcoded anywhere in the rule. The roots
//!   come from branch protection, so a repository that renames a job does not
//!   silently stop being checked.
//!
//! INV-2100-3 is the one that bites. A reusable-workflow job namespaces as
//! `<caller> / <callee>`, so a repo can have a required `ci / gate` **and** an
//! unrequired top-level job whose display name is `gate`. Matching display
//! names finds the wrong job and calls the repo compliant.
//!
//! Fails closed throughout: an unresolvable context list, an unparsable
//! workflow, a workflow with no jobs, or a required context that resolves into
//! an unreadable external workflow are all *failures*. An empty result set is
//! never a pass.
//!
//! Contract: `contracts/comply-gate-effect-v1.yaml`.

pub mod effect;
pub mod graph;
pub mod invocation;
pub mod kernel;
pub mod ledger;
pub mod reach;
pub mod required;
pub mod resolve;
pub mod roster;
pub mod workflow;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cb2100;

use crate::models::comply_config::{CheckSeverity, ComplyConfig};
use invocation::Invocation;
use resolve::Resolution;
use std::path::Path;

/// Command spellings that run the CB rule roster. `comply status` is a
/// documented alias of `comply check`.
pub const COMPLY_NEEDLES: &[&str] = &["comply check", "comply status"];

/// What one required status check actually does to the CB roster.
///
/// Only one of these four is a measurement. The roots table used to print a
/// single phrase — "this required check gates nothing in the CB roster" — for
/// every context that did not carry a rule, which said the same thing about a
/// check that was read in full and genuinely reaches nothing as about a check
/// whose steps this repository cannot read at all. The second is a HOLE: it
/// cannot be shown to run a rule, and it cannot be shown not to. Rendering a
/// hole as a zero is the same mistake as rendering an empty roster as a pass,
/// and this rule exists to reject that mistake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootEffect {
    /// Reaches at least one invocation of the roster whose failure propagates.
    Carries,
    /// Resolved to a job in this repository, read in full, and reaches no
    /// invocation. A measured zero.
    ReachesNothing,
    /// Resolves into a reusable workflow hosted elsewhere. Unmeasured.
    Opaque { reference: String },
    /// No job in `.github/workflows` produces this context at all, so it can
    /// never turn green on its own. Unmeasured.
    Phantom,
    /// No resolution was recorded for this context — the analysis never got as
    /// far as asking. Fail closed: unmeasured, never a zero.
    Unresolved,
}

impl RootEffect {
    /// Did this root reach the roster? `false` for every unmeasured case too,
    /// because "we could not tell" must never read as "enforced".
    pub fn carries(&self) -> bool {
        matches!(self, RootEffect::Carries)
    }

    /// Was this root actually *measured*? A hole is not a zero.
    pub fn measured(&self) -> bool {
        matches!(self, RootEffect::Carries | RootEffect::ReachesNothing)
    }

    /// The cell the ledger prints, and the sentence the check message uses.
    pub fn explain(&self) -> String {
        match self {
            RootEffect::Carries => "yes".to_string(),
            RootEffect::ReachesNothing => "**no** — this required check was read in full and \
                                          gates nothing in the CB roster"
                .to_string(),
            RootEffect::Opaque { reference } => format!(
                "**unknown** — resolves into `{reference}`, whose steps this repository cannot \
                 read. A HOLE, not a measured zero: no rule can be shown to run through it, and \
                 none can be shown not to"
            ),
            RootEffect::Phantom => "**unknown** — no job in `.github/workflows` produces this \
                                    context at all, so it is a phantom gate: it cannot turn \
                                    green on its own, and what it would reach is unmeasured"
                .to_string(),
            RootEffect::Unresolved => "**unknown** — no resolution was recorded for this \
                                       context, so nothing about it was measured"
                .to_string(),
        }
    }
}

/// The verdict, with everything needed to explain it.
#[derive(Debug, Clone, Default)]
pub struct GateEffectReport {
    /// Required contexts, and where the list came from.
    pub required_contexts: Vec<String>,
    pub context_source: Option<String>,
    /// One entry per required context.
    pub resolutions: Vec<(String, Resolution)>,
    /// Every invocation found, enforcing or not.
    pub invocations: Vec<Invocation>,
    /// Error-severity rule ids that must be reachable.
    pub rules: Vec<String>,
    /// Error-severity rule ids that are not.
    pub unreachable_rules: Vec<String>,
    /// Fail-closed reasons: things that could not be measured, or holes in the
    /// reachability graph. Any entry fails the check.
    pub holes: Vec<String>,
    /// The graph the verdict was drawn from: roots, jobs, invocations, and
    /// which edges a failure can actually cross.
    pub graph: graph::Enforcement,
}

impl GateEffectReport {
    pub fn passed(&self) -> bool {
        self.holes.is_empty() && self.unreachable_rules.is_empty() && !self.rules.is_empty()
    }

    /// One `(context, what it does to the roster)` pair per required context.
    ///
    /// This is where a required check with no CB mapping shows up: a context
    /// that reaches nothing is a gate on something other than the rule roster,
    /// and the ledger says so instead of quietly counting it as coverage.
    pub fn context_effects(&self) -> Vec<(String, RootEffect)> {
        self.required_contexts
            .iter()
            .enumerate()
            .map(|(i, c)| (c.clone(), self.root_effect(i)))
            .collect()
    }

    fn root_effect(&self, i: usize) -> RootEffect {
        let carries = self
            .graph
            .roots
            .get(i)
            .is_some_and(|r| !self.graph.invocations_from_root(*r).is_empty());
        if carries {
            return RootEffect::Carries;
        }
        // Not carrying is three different findings, and only one of them is a
        // measurement. `resolutions` is pushed once per context, in order, by
        // `finish`, so index `i` is this context's own resolution.
        match self.resolutions.get(i).map(|(_, r)| r) {
            Some(Resolution::Job { .. }) => RootEffect::ReachesNothing,
            Some(Resolution::Opaque { reference, .. }) => RootEffect::Opaque {
                reference: reference.clone(),
            },
            Some(Resolution::Phantom) => RootEffect::Phantom,
            None => RootEffect::Unresolved,
        }
    }

    pub fn enforcing(&self) -> impl Iterator<Item = &Invocation> {
        self.invocations.iter().filter(|i| i.is_enforcing())
    }

    pub fn neutered(&self) -> impl Iterator<Item = &Invocation> {
        self.invocations.iter().filter(|i| !i.is_enforcing())
    }
}

/// The rules whose failure is declared to be an error. Sorted, deduplicated,
/// upper-cased to the `CB-NNNN` spelling used in reports.
pub fn error_severity_rules(config: &ComplyConfig) -> Vec<String> {
    let mut ids: Vec<String> = config
        .checks
        .iter()
        .filter(|(_, c)| {
            c.enabled && matches!(c.severity, CheckSeverity::Error | CheckSeverity::Critical)
        })
        .map(|(id, _)| id.to_uppercase())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// Run the analysis against a project directory.
pub fn analyze(project_path: &Path, config: &ComplyConfig) -> GateEffectReport {
    let mut report = new_report(config);
    let required = match required::resolve(project_path) {
        Ok(r) => r,
        Err(e) => {
            report.holes.push(e);
            report.unreachable_rules = report.rules.clone();
            return report;
        }
    };
    finish(project_path, &required, report)
}

/// The half of [`analyze`] that runs once the required contexts are known.
/// Split out so fixtures can supply a context list directly instead of going
/// through the environment, which would make the falsification tests racy.
pub fn analyze_with_contexts(
    project_path: &Path,
    config: &ComplyConfig,
    required: &required::RequiredContexts,
) -> GateEffectReport {
    finish(project_path, required, new_report(config))
}

/// A fresh report carrying the rule roster, and the fail-closed hole for an
/// empty one: "every error rule is enforced" is vacuously true over no rules,
/// which is exactly the shape of assertion this check exists to reject.
fn new_report(config: &ComplyConfig) -> GateEffectReport {
    let mut report = GateEffectReport {
        rules: error_severity_rules(config),
        ..Default::default()
    };
    if report.rules.is_empty() {
        report.holes.push(empty_roster_hole(config));
    }
    report
}

/// Why the roster came out empty, in the terms that make it actionable.
///
/// The two cases are genuinely different. No checks at all is a missing
/// configuration. Checks that exist and are all sub-error is a configuration
/// that *runs* rules while declaring that none of them may fail — and because
/// `ComplyConfig::get_severity` answers `Warning` for an id it does not know,
/// a `checks:` map that lists a handful of rules silently demotes every rule it
/// omits. Naming that is the difference between a hole and a diagnosis.
fn empty_roster_hole(config: &ComplyConfig) -> String {
    if config.checks.is_empty() {
        return "no comply check is declared at all, so 'every severity=error rule is enforced' \
                is vacuous — an empty roster is a failure, not a pass"
            .into();
    }
    format!(
        "{} comply check(s) are declared and not one of them is severity=error, so \
         'every severity=error rule is enforced' is vacuous — an empty roster is a failure, not \
         a pass. Note that an id absent from `checks:` resolves to Warning, not to its default \
         severity, so a partial map demotes every rule it omits",
        config.checks.len()
    )
}

fn finish(
    project_path: &Path,
    required: &required::RequiredContexts,
    mut report: GateEffectReport,
) -> GateEffectReport {
    report.context_source = Some(required.source.label().to_string());
    report.required_contexts = required.contexts.clone();

    let set = workflow::load_workflows(project_path);
    record_workflow_holes(&set, &mut report);

    for context in &required.contexts {
        let resolution = resolve::resolve_context(&set, context);
        collect_for_context(project_path, &set, context, &resolution, &mut report);
        report.resolutions.push((context.clone(), resolution));
    }

    // Two required contexts resolving to the same job would otherwise report
    // the same invocation twice, which inflates the evidence for a rule that
    // is carried exactly once.
    dedup_invocations(&mut report.invocations);

    // INV-2100-1: the roots are unioned and the question is asked once, of the
    // graph, by the kernel `KANI-2100-1` proves. Asking it context-by-context
    // would let one healthy check answer for four.
    report.graph = graph::build(&set, &report.resolutions, &report.invocations);
    if !report.graph.any_invocation_reachable() {
        report.unreachable_rules = report.rules.clone();
    }
    report
}

/// Keep the first occurrence of each invocation site, in discovery order.
fn dedup_invocations(invocations: &mut Vec<invocation::Invocation>) {
    let mut seen: Vec<invocation::Invocation> = Vec::new();
    invocations.retain(|i| {
        if seen.contains(i) {
            return false;
        }
        seen.push(i.clone());
        true
    });
}

fn record_workflow_holes(set: &workflow::WorkflowSet, report: &mut GateEffectReport) {
    for (path, err) in &set.unparsable {
        report.holes.push(format!(
            "{} did not parse ({err}); a workflow that cannot be read is a hole in the \
             reachability graph",
            path.display()
        ));
    }
    if set.job_count() == 0 {
        report.holes.push(
            ".github/workflows declares zero jobs, so no required context can be produced by \
             this repository"
                .into(),
        );
    }
}

fn collect_for_context(
    project_path: &Path,
    set: &workflow::WorkflowSet,
    context: &str,
    resolution: &Resolution,
    report: &mut GateEffectReport,
) {
    match resolution {
        Resolution::Phantom => report.holes.push(format!(
            "required context `{context}` is produced by no job in .github/workflows — a \
             required check nothing reports is a phantom gate"
        )),
        Resolution::Opaque {
            caller_workflow,
            caller_job,
            reference,
        } => report.holes.push(format!(
            "required context `{context}` resolves into `{reference}` (called by job \
             `{caller_job}` in {}), whose steps are not readable from this repository, so no \
             rule can be *shown* to run through it",
            caller_workflow.display()
        )),
        Resolution::Job { workflow, job_id } => {
            for r in reach::reachable_jobs(set, workflow, job_id) {
                report.invocations.extend(invocation::find_in_job(
                    project_path,
                    r.job,
                    COMPLY_NEEDLES,
                    &r.suppressions,
                ));
            }
        }
    }
}
