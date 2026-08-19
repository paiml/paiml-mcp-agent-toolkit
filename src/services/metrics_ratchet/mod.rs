//! Ratchet baselines (CB-2102) and threshold coherence (CB-2101).
//!
//! Two gates over the numbers a repo writes down about itself:
//!
//! * **CB-2102 ratchet** — `.pmat-ratchet.toml` records a baseline per metric,
//!   MEASURED at a named commit. A PR may not exceed it; a nightly job lowers
//!   it when the measurement drops; raising it needs a written justification.
//! * **CB-2101 threshold coherence** — every threshold in `.pmat-metrics.toml`
//!   is classified FIRING, VIOLATED or VACUOUS against the same measurements.
//!   A VIOLATED threshold on a green build is a fail: the config asserts a
//!   bound the tree does not meet and nothing noticed.
//!
//! The defect that motivated both is in this repo's own history:
//! `.pmat-metrics.toml` carried `max_unwrap_calls = 100` with the comment
//! "Current: 570", while the tree measured 11,056 — a limit exceeded 110x, a
//! comment stale by 19x, and a green build throughout, because nothing read
//! the key at all.
//!
//! Layering: [`config`] and [`kernel`] are PURE — they take measurements as an
//! argument and never touch a file, a clock or a subprocess, which is what
//! lets the falsification tests drive every branch without a source tree.
//! [`measure`], [`history`] and [`rewrite`] are the impure half: running each
//! metric's own pinned command, asking git what the baseline file used to say,
//! and writing a lowered baseline back without destroying the file's comments.
//! The comply check that drives them is
//! `cli/handlers/comply_handlers/check_handlers/check_metrics_ratchet.rs`.

pub mod config;
pub mod history;
pub mod kernel;
pub mod measure;
pub mod rewrite;

#[cfg(test)]
mod kernel_tests;

#[cfg(test)]
mod config_tests;

#[cfg(test)]
mod drive_tests;

#[cfg(test)]
mod coherence_drive_tests;

use std::path::Path;

/// How a project relates to a ratchet file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatchetStatus {
    /// A `.pmat-ratchet.toml` is on disk.
    Present,
    /// There is none, and there never was: the project declares no baselines.
    Absent,
    /// There was one and it is gone. Deleting a gate's input is not a way of
    /// passing it.
    Deleted,
}

/// Is there a ratchet to check at all?
pub fn status(project_path: &Path) -> RatchetStatus {
    if project_path.join(config::RATCHET_FILE).is_file() {
        return RatchetStatus::Present;
    }
    match history::was_ever_committed(project_path, config::RATCHET_FILE) {
        Ok(true) => RatchetStatus::Deleted,
        // No git and no file: absent is the honest reading.
        Ok(false) | Err(_) => RatchetStatus::Absent,
    }
}

/// Read the config, run every metric's own command, ask git what the file used
/// to say, and judge the result.
///
/// Fails closed at every step. In particular, when git cannot produce the
/// previous version of the file, the run records a HOLE rather than passing
/// `None` through to [`config::evaluate_ratchet`] — `None` legitimately means
/// "this is the initial capture, nothing was raised", and letting an
/// unreadable history borrow that meaning would make an unjustified raise
/// invisible on exactly the machines where history is hardest to read.
pub fn run(project_path: &Path) -> Result<config::RatchetReport, config::ConfigError> {
    let cfg = config::RatchetConfig::load(project_path)?;
    let measurements = measure::measure_all(project_path, &cfg.metric);
    let current = std::fs::read_to_string(project_path.join(config::RATCHET_FILE)).ok();
    let (previous, hole) =
        match history::prior_version(project_path, config::RATCHET_FILE, current.as_deref()) {
            history::Prior::NoHistory => (None, None),
            history::Prior::Unavailable(e) => (
                None,
                Some(format!(
                    "cannot read the previous {} from git ({e}), so a raised baseline could \
                     not be detected",
                    config::RATCHET_FILE
                )),
            ),
            history::Prior::Content(text) => match config::RatchetConfig::parse(&text) {
                Ok(prev) => (Some(prev), None),
                Err(e) => (
                    None,
                    Some(format!(
                        "the previous committed {} does not parse ({e}), so a raised baseline \
                         could not be detected",
                        config::RATCHET_FILE
                    )),
                ),
            },
        };

    let mut report = config::evaluate_ratchet(
        &cfg.metric,
        &measurements,
        previous.as_ref().map(|p| &p.metric),
    );
    if let Some(h) = hole {
        report.holes.push(h);
        report.outcome = config::Outcome::Fail;
    }
    Ok(report)
}

/// Audit every threshold in `.pmat-metrics.toml` against a live measurement
/// (CB-2101).
///
/// Reads the SAME `.pmat-ratchet.toml` the ratchet uses, because the two gates
/// must not be able to disagree about what a metric means: the coherence audit
/// classifies a declared limit against the number a metric's own `command`
/// prints, never against the baseline written beside it. A limit judged against
/// a remembered number is the defect this rule is named for, one indirection
/// further out.
///
/// Fails closed like [`run`]: a missing or unparsable config on either side is
/// an `Err`, and the caller must render that as a failure rather than a skip.
pub fn run_coherence(project_path: &Path) -> Result<config::CoherenceReport, config::ConfigError> {
    let cfg = config::RatchetConfig::load(project_path)?;
    let roster = config::MetricsRoster::load(project_path)?;
    let measurements = measure::measure_all(project_path, &cfg.metric);
    let enforcers = config::EnforcerIndex::resolve(project_path, &cfg.coherence);
    Ok(config::evaluate_coherence(
        &roster,
        &cfg.coherence,
        &measurements,
        &cfg.metric,
        &enforcers,
    ))
}

/// The scheduled lowering pass: rewrite every baseline the tree has beaten.
///
/// Returns the changes it made, in `metric: old -> new` form. Writes nothing
/// when there is nothing to lower, and never raises anything (`INV-2102-2` is
/// enforced in [`kernel::next_baseline`], which is the only source of the new
/// numbers).
pub fn lower(project_path: &Path) -> Result<Vec<String>, String> {
    let cfg = config::RatchetConfig::load(project_path).map_err(|e| e.to_string())?;
    let measurements = measure::measure_all(project_path, &cfg.metric);
    let lowered = rewrite::lowered_baselines(&cfg.metric, &measurements);
    if lowered.is_empty() {
        return Ok(Vec::new());
    }
    let path = project_path.join(config::RATCHET_FILE);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let rewritten = rewrite::apply(&text, &lowered)?;
    std::fs::write(&path, rewritten).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(lowered
        .iter()
        .map(|(id, next)| format!("{id}: {} -> {next}", cfg.metric[id].baseline))
        .collect())
}
