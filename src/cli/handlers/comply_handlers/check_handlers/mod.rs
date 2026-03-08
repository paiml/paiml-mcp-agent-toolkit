#![cfg_attr(coverage_nightly, coverage(off))]
// check_handlers module - split from monolithic check_handlers.rs
//
// Submodules:
// - types: Core types (ComplianceCheck, CheckStatus, Severity, etc.)
// - check: Core compliance checks and handle_check
// - check_best_practices: Language-specific best practices (CB-400+, CB-500+, CB-600+, etc.)
// - check_dead_code: Dead code analysis (CB-304)
// - check_tdg_grade: TDG grade gate (CB-200)
// - check_extended: Extended checks (CB-300+, CB-081, file health)
// - check_sovereign: Sovereign stack patterns and PAIML deps
// - check_review_audit: Review and audit handlers (COMPLY-045)

pub(crate) mod check;
pub(crate) mod check_best_practices;
pub(crate) mod check_dead_code;
pub(crate) mod check_extended;
pub(crate) mod check_provable_contracts;
pub(crate) mod check_review_audit;
pub(crate) mod check_sovereign;
pub(crate) mod check_tdg_grade;
pub(crate) mod types;

// Re-export items needed by the parent comply_handlers module scope
// (for migrate_handlers.rs and command_dispatch.rs which are include!()'d there)
pub(crate) use check::*;
pub(crate) use check_review_audit::{handle_audit, handle_review};
pub(crate) use check_sovereign::{check_file_health_multi, generate_file_health_baseline};
pub(crate) use types::*;

// Additional re-exports needed by test files that reference functions
// via crate::cli::handlers::comply_handlers::<function_name>.
// These are only consumed by #[cfg(test)] test modules.
#[cfg(test)]
pub(crate) use check_dead_code::*;
#[cfg(test)]
pub(crate) use check_extended::{
    check_dead_code_percentage, check_edd_compliance, check_file_health, check_golden_trace_drift,
    check_muda_waste_score, check_reproducibility_level,
};
#[cfg(test)]
pub(crate) use check_sovereign::{check_paiml_deps_workspace, check_sovereign_stack_patterns};
