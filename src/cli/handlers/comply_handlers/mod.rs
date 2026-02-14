#![cfg_attr(coverage_nightly, coverage(off))]
// Comply handlers - split for file health (CB-040)
include!("check_handlers.rs");
include!("check_handlers_part2.rs");
include!("migrate_handlers.rs");

// CB-050/CB-060 detection logic
pub mod comply_cb_detect;

// CB-300: Muda Waste Score (COMPLY-040)
pub mod muda_handlers;

// CB-301/CB-302: Reproducibility & Golden Trace (COMPLY-041/042)
pub mod reproducibility_handlers;

// CB-303: Equation-Driven Development (COMPLY-043)
pub mod edd_handlers;

#[cfg(test)]
#[path = "comply_handlers_tests.rs"]
mod tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod falsification_tests;
