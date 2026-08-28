#![cfg_attr(coverage_nightly, coverage(off))]
// Comply handlers - split for file health (CB-040)

use crate::cli::commands::{ComplyCommands, ComplyOutputFormat, NumericClaimsFormat};
use anyhow::Result;
// `handle_report` no longer stamps its own timestamp — it renders the report
// `compute_compliance_report` built — so `Utc` is reached only by the tests
// that construct `ComplianceReport` literals.
#[cfg(test)]
use chrono::Utc;
use std::fs;
use std::path::Path;

// Check handlers split into submodules
pub(crate) mod check_handlers;
pub(crate) use check_handlers::*;

// Migration, enforce, report, init, upgrade handlers
include!("migrate_handlers.rs");

// CB-2100: the enforcement ledger generator (`pmat comply ledger`)
include!("ledger_handler.rs");
// CB-2102: `pmat comply ratchet` — the baseline gate and its lowering pass
include!("ratchet_handler.rs");
// CB-2101: `pmat comply coherence` — classify every threshold, with reasons
include!("coherence_handler.rs");
// CB-2104: `pmat comply numeric-claims` — replicated and self-contradicting numbers
include!("numeric_claims_handler.rs");

// Command dispatch (needs access to both check_handlers and migrate_handlers items)
include!("command_dispatch.rs");

// CB-050/CB-060 detection logic
pub mod comply_cb_detect;

// CB-300: Muda Waste Score (COMPLY-040)
pub mod muda_handlers;

// CB-301/CB-302: Reproducibility & Golden Trace (COMPLY-041/042)
pub mod reproducibility_handlers;

// CB-303: Equation-Driven Development (COMPLY-043)
pub mod edd_handlers;

// CC-001 through CC-005: Cross-Crate Duplication Detection (#232)
pub mod cross_crate_handlers;

#[cfg(test)]
#[path = "comply_handlers_tests.rs"]
mod tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod falsification_tests;
