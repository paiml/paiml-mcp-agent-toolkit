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
//! Layering: everything in this module is PURE. Measurement, the filesystem
//! and git live in the comply check
//! (`cli/handlers/comply_handlers/check_handlers/check_metrics_ratchet.rs`),
//! which passes measurements in. That split is what lets the falsification
//! tests drive every branch without a source tree.

pub mod config;
pub mod kernel;

#[cfg(test)]
mod kernel_tests;

#[cfg(test)]
mod config_tests;
