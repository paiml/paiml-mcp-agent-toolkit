#![cfg_attr(coverage_nightly, coverage(off))]
//! Enforce command handlers for extreme quality enforcement
//!
//! This module implements the state machine-based quality enforcement system
//! that iteratively improves code quality until extreme standards are met.

pub mod analysis;
pub mod assessment;
pub mod config;
pub mod enforcement;
pub mod handler;
pub mod output;
pub mod states;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use analysis::{
    run_complexity_analysis, run_coverage_analysis, run_dead_code_analysis,
    run_duplication_analysis, run_satd_analysis, run_tdg_analysis, AnalysisScope,
};
pub use assessment::{assess_project, QualityAssessment};
pub use config::{
    clear_enforcement_cache, initialize_enforcement_environment, load_quality_profile,
    EnforcementConfig,
};
pub use enforcement::{
    check_improvement_targets, execute_enforcement_iteration, execute_main_loop,
    finalize_enforcement_run, handle_enforcement_iteration, handle_special_modes,
    run_enforcement_step, should_continue_enforcement, should_stop_for_target_improvement,
};
pub use handler::route_enforce_command;
pub use output::{
    format_violations_output, output_result, print_enforcement_header, print_enforcement_summary,
    print_progress_bar,
};
pub use states::{
    handle_analyzing_enforcement_state, handle_analyzing_state, handle_refactoring_pass,
    handle_validating_enforcement_state, handle_violating_enforcement_state_proxy,
    handle_violating_state,
};

// The handlers that answer without measuring no longer exist outside the test
// build: nothing the binary can run reaches them. See
// `states::handle_refactoring_pass`, and `run_enforcement_step`'s Complete arm.
#[cfg(test)]
pub use states::{
    handle_complete_enforcement_state, handle_complete_state, handle_refactoring_enforcement_state,
    handle_refactoring_state,
};
pub use types::{
    EnforcementIterationResult, EnforcementLoopResult, EnforcementProgress, EnforcementResult,
    EnforcementState, QualityProfile, QualityViolation,
};

#[cfg(test)]
#[path = "satd_enforcement_tests.rs"]
mod satd_enforcement_tests;

#[cfg(test)]
#[path = "unmeasured_cannot_pass_tests.rs"]
mod unmeasured_cannot_pass_tests;

#[cfg(test)]
#[path = "flag_fidelity_tests.rs"]
mod flag_fidelity_tests;

#[cfg(test)]
#[path = "surface_agreement_tests.rs"]
mod surface_agreement_tests;
