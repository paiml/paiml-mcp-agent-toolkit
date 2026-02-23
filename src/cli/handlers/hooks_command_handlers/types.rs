//! Result and status types for hooks command handlers

#![cfg_attr(coverage_nightly, coverage(off))]

/// Hook installation result
#[derive(Debug, PartialEq)]
pub struct HookInstallResult {
    pub success: bool,
    pub hook_created: bool,
    pub backup_created: bool,
    pub message: String,
}

/// Hook uninstall result
#[derive(Debug, PartialEq)]
pub struct HookUninstallResult {
    pub success: bool,
    pub hook_removed: bool,
    pub backup_restored: bool,
    pub message: String,
}

/// Hook status information
#[derive(Debug, PartialEq)]
pub struct HookStatus {
    pub installed: bool,
    pub is_pmat_managed: bool,
    pub config_up_to_date: bool,
    pub last_updated: Option<String>,
    pub hook_content_preview: Option<String>,
}

/// Hook verification result
#[derive(Debug, PartialEq)]
pub struct HookVerificationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub fixes_applied: Vec<String>,
}

/// Hook refresh result
#[derive(Debug, PartialEq)]
pub struct HookRefreshResult {
    pub success: bool,
    pub hook_updated: bool,
    pub config_changes_detected: bool,
    pub message: String,
}

/// Hook run result (for CI/CD)
#[derive(Debug, PartialEq)]
pub struct HookRunResult {
    pub success: bool,
    pub checks_passed: usize,
    pub checks_failed: usize,
    pub output: String,
}
