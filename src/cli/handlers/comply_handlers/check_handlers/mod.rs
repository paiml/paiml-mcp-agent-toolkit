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
pub(crate) mod check_binding_scope;
pub(crate) mod check_codegen;
pub(crate) mod check_cot_proof;
pub(crate) mod check_dead_code;
pub(crate) mod check_evidence_gates;
pub(crate) mod check_extended;
pub(crate) mod check_falsification_unification;
pub(crate) mod check_macs;
pub(crate) mod check_mono_spec;
pub(crate) mod check_provable_contracts;
pub(crate) mod check_review_audit;
pub(crate) mod check_sovereign;
pub(crate) mod check_tdg_grade;
pub(crate) mod check_work_ladder;
pub(crate) mod types;

// Re-export items needed by the parent comply_handlers module scope
// (for migrate_handlers.rs and command_dispatch.rs which are include!()'d there)
pub(crate) use check::*;
pub(crate) use check_review_audit::{handle_audit, handle_review};
pub(crate) use check_sovereign::{check_file_health_multi, generate_file_health_baseline};
pub(crate) use types::*;

// Test re-exports removed — test files now include!() into check.rs scope directly.
