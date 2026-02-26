#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-500 Series: Rust Best Practices Detection
//!
//! Generic Rust defect detection for `pmat comply check`.
//! These checks apply to ANY Rust project, not just PAIML-specific patterns.

mod performance;
mod publish;
mod runtime;
mod type_safety;
mod utilities;

// Re-export all public detection functions

// Publishing and project configuration (CB-500, CB-503, CB-504, CB-505, CB-509, CB-529)
pub use publish::{
    detect_cb500_publish_hygiene, detect_cb503_clippy_config, detect_cb504_deny_config,
    detect_cb505_workspace_lint_hygiene, detect_cb509_feature_gate_coverage,
    detect_cb529_pmat_tracked_in_git,
};

// Type safety and value correctness (CB-501, CB-502, CB-506, CB-508, CB-515, CB-516)
pub use type_safety::{
    detect_cb501_unwrap_density, detect_cb502_expect_quality, detect_cb506_string_byte_indexing,
    detect_cb508_lossy_numeric_casts, detect_cb515_catch_all_match_default,
    detect_cb516_hardcoded_magic_numbers,
};

// Runtime behavior checks (CB-507, CB-510, CB-511, CB-512, CB-513, CB-514)
pub use runtime::{
    detect_cb507_panic_macros, detect_cb510_include_macro_hygiene, detect_cb511_flaky_timing_tests,
    detect_cb512_error_propagation_gap, detect_cb513_silent_error_swallowing,
    detect_cb514_debug_eprintln_leaks,
};

// Performance and data integrity (CB-517, CB-518, CB-519, CB-520, CB-521)
pub use performance::{
    detect_cb517_stale_debug_artifacts, detect_cb518_expensive_clone_in_loop,
    detect_cb519_lossy_data_pipeline, detect_cb520_expensive_init_in_loop,
    detect_cb521_format_without_magic_bytes,
};

// NOTE: CB-532 (assert in library API) was removed -- false positive rate too high.
// Dimension assertions in `pub fn` are a deliberate design choice in ML libs.
