//! Which `#[test]` functions does any CI leg actually EXECUTE?
//!
//! # The defect this exists to find
//!
//! Four regression tests were added in the 3.32.0 cycle behind features no job
//! ran: three under `mcp-integration` (`mcp_integration::tdg_tools::tests::
//! test_analyze_technical_debt_refuses_ungradable_file` and two in
//! `mcp_integration::tools::context_adapters::tests`), one under
//! `analytics-gpu`. They compiled in no invocation, so they failed in none.
//! A regression test behind a feature nothing executes is a comment.
//!
//! Two of the three were invisible to a name search, and that is the
//! interesting part. `mcp_pmcp::agent_context_handlers::tests` contains tests
//! with *identical names* which DO run, so
//! `cargo test --lib -- --list | grep <name>` printed the name and the hidden
//! copy still never executed. This module keys on the FULL MODULE PATH.
//!
//! # Why the existing gates could not see it
//!
//! `orphan-ledger` (`.github/workflows/feature-matrix.yml`) requires every
//! orphan FEATURE to be tested or explained. It says nothing about a TEST that
//! lives behind a feature no gate runs. `cargo check --features X` — which the
//! `individual` and `bundles` jobs run — compiles nothing under `#[cfg(test)]`
//! and executes no body.
//!
//! # Scope, stated rather than implied
//!
//! The enforced universe is the LIB test target reached from `src/lib.rs`, the
//! same universe as the ~20,000 tests every leg already runs. Two residuals are
//! reported as numbers rather than hidden:
//!
//! * registered `[[test]]` targets under `tests/` — no CI leg runs any of them
//!   (every invocation is `cargo test --lib`), so enforcing them here would
//!   flag 100% of that population and say nothing new;
//! * `#[ignore]`d tests, which are compiled and *declared* unrun by `cargo
//!   test`'s own "N ignored" line. Invisibility is the defect being gated; an
//!   `#[ignore]` is visible.
//!
//! Files no compilation unit reaches at all are `pmat analyze reachability`.

pub mod cfg;
pub mod features;
pub mod ledger;
pub mod legs;
pub mod reasons;
pub mod walk;

#[cfg(test)]
mod tests;

use cfg::{Env, Tri};
use std::collections::BTreeSet;
use std::path::Path;

/// One test and why nothing runs it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// Full module path — the key. Never the bare name.
    pub path: String,
    pub file: String,
    /// Sorted, comma-joined features it positively requires that no leg
    /// enables; `<environment>` when no feature is at fault.
    pub bucket: String,
    /// The accumulated predicate, rendered.
    pub cfg: String,
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    /// `file:job[matrix]` for every resolved test-running invocation.
    pub legs: Vec<String>,
    pub total_tests: usize,
    pub executed: usize,
    /// Compiled by no leg.
    pub unrun: Vec<Finding>,
    /// The predicate could not be decided. A finding, never a pass.
    pub undeterminable: Vec<Finding>,
    /// Compiled and listed, but `#[ignore]`d.
    pub ignored: usize,
    /// `mod`/`include!` targets not found; every count above is a floor.
    pub unresolved: Vec<String>,
    /// Files `syn` refused; their tests are missing from every count.
    pub unparsed: Vec<String>,
    pub files: usize,
}

impl Report {
    /// Buckets in ledger order, with their members.
    #[must_use]
    pub fn buckets(&self) -> Vec<(String, Vec<&Finding>)> {
        let mut keys: Vec<&str> = self.unrun.iter().map(|f| f.bucket.as_str()).collect();
        keys.sort_unstable();
        keys.dedup();
        keys.into_iter()
            .map(|k| {
                (
                    k.to_string(),
                    self.unrun.iter().filter(|f| f.bucket == k).collect(),
                )
            })
            .collect()
    }

    /// A one-line verdict that always states the scope it was measured over.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut s = format!(
            "{} of {} lib tests are executed by at least one of {} CI test leg(s); \
             {} are compiled by none",
            self.executed,
            self.total_tests,
            self.legs.len(),
            self.unrun.len()
        );
        if !self.undeterminable.is_empty() {
            s.push_str(&format!(
                "; {} have a cfg predicate this analysis cannot decide (a finding, not a pass)",
                self.undeterminable.len()
            ));
        }
        if !self.unresolved.is_empty() || !self.unparsed.is_empty() {
            s.push_str(&format!(
                " — {} unresolved mod/include and {} unparsable file(s), so these are FLOORS",
                self.unresolved.len(),
                self.unparsed.len()
            ));
        }
        s
    }
}

/// Resolve one leg's feature closure.
fn env_for(graph: &features::FeatureGraph, leg: &legs::Leg) -> Env {
    if leg.all_features {
        return Env {
            features: graph.keys().cloned().collect(),
        };
    }
    let mut roots: Vec<String> = leg.features.clone();
    if leg.default_features {
        roots.push("default".to_string());
    }
    Env {
        features: features::closure(graph, roots),
    }
}

/// Run the analysis.
///
/// `extra_legs` are test-running invocations defined OUTSIDE this repository —
/// `ci / test` lives in the reusable `paiml/.github` workflow and cannot be
/// read from here. Each is a comma-separated feature spec; the empty string
/// means default features.
///
/// # Errors
///
/// Fails when no test-running leg could be resolved at all. Zero legs would
/// make every test in the tree a finding, which is a broken scanner reporting
/// a catastrophe rather than a catastrophe.
pub fn analyze(project_root: &Path, extra_legs: &[String]) -> Result<Report, String> {
    let manifest = std::fs::read_to_string(project_root.join("Cargo.toml"))
        .map_err(|e| format!("Cargo.toml: {e}"))?;
    let graph = features::parse(&manifest);
    if graph.len() < 40 {
        return Err(format!(
            "only {} features parsed from [features] — the parser is broken, not the crate",
            graph.len()
        ));
    }

    let mut legs: Vec<legs::Leg> = legs::from_workflows(&project_root.join(".github/workflows"))
        .into_iter()
        .filter(|l| l.runs_lib)
        .collect();
    // An empty `--executed ''` is an absent argument, not a leg. Pushing one
    // fabricates a CI leg that does not exist, and a fabricated leg makes tests
    // count as executed when nothing executes them — the exact claim this module
    // exists to refute.
    //
    // It also made the gate unsatisfiable by its own remediation. `ledger::check`
    // compares the committed file against a render of the report it is given;
    // `the_committed_ledger_matches_the_tree` built that report with
    // `&[String::new()]` while `--write-ledger` passes the CLI's empty vec. The
    // two disagreed by one leg (`--executed ''`) and therefore by the whole
    // rendered text, so the file the command wrote could never satisfy the test
    // that told you to run it. Filtering here makes both call shapes agree.
    for spec in extra_legs.iter().filter(|s| !s.trim().is_empty()) {
        legs.push(legs::Leg {
            origin: format!("--executed '{spec}'"),
            features: spec
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect(),
            default_features: true,
            all_features: false,
            runs_lib: true,
        });
    }
    if legs.is_empty() {
        return Err(
            "no `cargo test` invocation was resolved from .github/workflows and none was \
             supplied with --executed; with zero legs every test would be a finding"
                .to_string(),
        );
    }

    let collected = walk::collect(project_root, &project_root.join("src/lib.rs"));
    let envs: Vec<(String, Env)> = legs
        .iter()
        .map(|l| (l.origin.clone(), env_for(&graph, l)))
        .collect();
    let default_closure = features::closure(&graph, ["default"]);
    let every_feature: BTreeSet<String> = envs
        .iter()
        .map(|(_, e)| e.features.clone())
        .reduce(|a, b| a.intersection(&b).cloned().collect())
        .unwrap_or_default();

    let mut report = Report {
        legs: envs.iter().map(|(o, _)| o.clone()).collect(),
        total_tests: collected.tests.len(),
        unresolved: collected.unresolved,
        unparsed: collected.unparsed,
        files: collected.files,
        ..Report::default()
    };

    for t in &collected.tests {
        let verdict = envs
            .iter()
            .map(|(_, e)| t.cfg.eval(e))
            .fold(Tri::False, |acc, v| match (acc, v) {
                (Tri::True, _) | (_, Tri::True) => Tri::True,
                (Tri::Unknown, _) | (_, Tri::Unknown) => Tri::Unknown,
                _ => Tri::False,
            });
        match verdict {
            Tri::True => {
                report.executed += 1;
                if t.ignored {
                    report.ignored += 1;
                }
            }
            Tri::Unknown => {
                report
                    .undeterminable
                    .push(finding(t, &default_closure, &every_feature))
            }
            Tri::False => report
                .unrun
                .push(finding(t, &default_closure, &every_feature)),
        }
    }
    report.unrun.sort();
    report.undeterminable.sort();
    Ok(report)
}

/// The bucket names the REASON, in the vocabulary a fix would use.
///
/// Three distinct answers, which a single label would blur:
/// a feature nothing enables (`agents-md`), a feature everything enables where
/// the test needs it OFF (`not(viz)`), and a predicate no feature can change
/// (`<environment>` — a runner, not a flag).
fn finding(
    t: &walk::TestFn,
    default_closure: &BTreeSet<String>,
    enabled_everywhere: &BTreeSet<String>,
) -> Finding {
    if !cfg::satisfiable(&t.cfg) {
        return Finding {
            path: t.path.clone(),
            file: t.file.clone(),
            bucket: "<unsatisfiable>".to_string(),
            cfg: t.cfg.render(),
        };
    }
    let (mut pos, mut neg) = (BTreeSet::new(), BTreeSet::new());
    t.cfg.positive_features(false, &mut pos);
    t.cfg.negated_features(false, &mut neg);
    // The delta from a DEFAULT build: the flags a fix would have to add. Naming
    // it relative to `default` rather than to "some leg somewhere" is what
    // makes `mcp-integration,mutation-testing` legible as a COMBINATION no
    // single leg provides, rather than two features that each look covered.
    let mut parts: Vec<String> = pos
        .into_iter()
        .filter(|f| !default_closure.contains(f))
        .collect();
    parts.extend(
        neg.into_iter()
            .filter(|f| enabled_everywhere.contains(f))
            .map(|f| format!("not({f})")),
    );
    parts.sort();
    Finding {
        path: t.path.clone(),
        file: t.file.clone(),
        bucket: if parts.is_empty() {
            "<environment>".to_string()
        } else {
            parts.join(",")
        },
        cfg: t.cfg.render(),
    }
}
