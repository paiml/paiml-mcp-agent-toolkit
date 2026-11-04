// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1667: a Rust project pins its toolchain (KZ-001 a).
///
/// STUB (test-first commit): always passes, so the tests below are RED.
pub(crate) fn check_toolchain_pinned(_project_path: &Path) -> ComplianceCheck {
    ComplianceCheck {
        name: "CB-1667: Pinned Rust Toolchain".to_string(),
        status: CheckStatus::Pass,
        message: "stub".to_string(),
        severity: Severity::Error,
    }
}
