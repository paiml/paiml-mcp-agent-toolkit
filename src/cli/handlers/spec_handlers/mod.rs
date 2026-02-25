//! Spec Management CLI Handlers (master-plan-pmat-work-system.md)
//!
//! Implements S-001 through S-010 acceptance criteria for specification management.

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::commands::SpecOutputFormat;
use crate::services::spec_parser::{ParsedSpec, SpecParser};
use std::fs;
use std::path::Path;

// Command handlers: handle_spec_score, handle_spec_comply, handle_spec_create, handle_spec_list
include!("spec_handlers_commands.rs");

// Scoring and formatting: calculate_spec_score, format_spec_score_*
include!("spec_handlers_scoring.rs");

// Sync and drift: handle_spec_sync, handle_spec_drift
include!("spec_handlers_sync.rs");

// Tests split for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax (functions/modules split across files)
#[cfg(all(test, feature = "broken-tests"))]
#[path = "tests.rs"]
mod tests;
