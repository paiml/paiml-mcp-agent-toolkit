#![cfg_attr(coverage_nightly, coverage(off))]

// Cross-crate duplication detection (CC-001 through CC-005)
// Split from monolithic cross_crate_handlers.rs for file health.

mod types;
mod handler;
mod discovery;
mod helpers;
mod baseline;
mod detection_cc001_cc002;
mod detection_cc003_cc004;
mod detection_cc005;
mod output;

// Re-export public API
pub use discovery::{discover_workspace_crates, read_cargo_deps};
pub use handler::handle_cross_crate;
pub use types::{
    CcSeverity, CrateInfo, CrossCrateFinding, CrossCrateReport, CrossCrateSummary,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_part2.rs"]
mod tests_part2;
