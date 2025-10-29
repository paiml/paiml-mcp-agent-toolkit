//! GREEN Phase Implementation for PMAT-070-001: CargoMutantsWrapper
//!
//! Minimal implementation to pass RED phase tests.
//! This is intentionally simple - REFACTOR phase will clean it up.
//!
//! Functionality:
//! - Detect cargo-mutants in PATH
//! - Execute cargo-mutants --version
//! - Parse and validate version (require v24.7.0+)
//! - Graceful error handling when not installed

use std::path::PathBuf;
use std::process::Command;

/// Wrapper for cargo-mutants subprocess execution
///
/// GREEN Phase: Minimal implementation to pass tests
pub struct CargoMutantsWrapper {
    pub cargo_mutants_path: Option<PathBuf>,
}

impl CargoMutantsWrapper {
    /// Initialize wrapper and detect cargo-mutants in PATH
    ///
    /// Returns Ok even if cargo-mutants is not installed (cargo_mutants_path will be None).
    /// This allows graceful degradation with helpful error messages.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Use which crate to find cargo-mutants in PATH
        let path = which::which("cargo-mutants").ok();

        if path.is_none() {
            // Not installed, but don't error - allow graceful handling
            eprintln!("⚠️  cargo-mutants not found in PATH");
            eprintln!("   Install: cargo install cargo-mutants");
            eprintln!();
        }

        Ok(Self {
            cargo_mutants_path: path,
        })
    }

    /// Check if cargo-mutants is installed
    pub fn is_installed(&self) -> bool {
        self.cargo_mutants_path.is_some()
    }

    /// Get cargo-mutants version
    ///
    /// Executes `cargo-mutants --version` and returns output.
    /// Returns error if not installed or execution fails.
    pub fn version(&self) -> Result<String, Box<dyn std::error::Error>> {
        let path = self.cargo_mutants_path.as_ref()
            .ok_or("cargo-mutants not found in PATH")?;

        let output = Command::new(path)
            .arg("--version")
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "cargo-mutants --version failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }

        let version_str = String::from_utf8(output.stdout)?
            .trim()
            .to_string();

        Ok(version_str)
    }

    /// Validate version meets minimum requirement (v24.7.0+)
    ///
    /// GREEN Phase: Basic validation
    /// REFACTOR Phase: Could extract this to separate function
    #[allow(dead_code)]
    pub fn validate_version(&self) -> Result<(), Box<dyn std::error::Error>> {
        let version_str = self.version()?;

        // Parse version (example: "cargo-mutants 24.7.1")
        let parts: Vec<&str> = version_str.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(format!(
                "Unexpected version format: '{}'",
                version_str
            ).into());
        }

        let version_number = parts[1];
        let version_parts: Vec<&str> = version_number.split('.').collect();

        if version_parts.len() < 2 {
            return Err(format!(
                "Invalid version number: '{}'",
                version_number
            ).into());
        }

        let major: u32 = version_parts[0].parse()
            .map_err(|_| format!("Invalid major version: '{}'", version_parts[0]))?;

        let minor: u32 = version_parts[1].parse()
            .map_err(|_| format!("Invalid minor version: '{}'", version_parts[1]))?;

        // Enforce minimum v24.7.0
        if major < 24 || (major == 24 && minor < 7) {
            return Err(format!(
                "cargo-mutants version {} is too old. Minimum required: v24.7.0",
                version_number
            ).into());
        }

        Ok(())
    }
}

// GREEN Phase: Minimal tests to verify implementation
// More comprehensive tests are in tests/cargo_mutants_wrapper_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapper_initialization_succeeds() {
        // Should not panic even if cargo-mutants is not installed
        let result = CargoMutantsWrapper::new();
        assert!(result.is_ok(), "Wrapper initialization should never fail");
    }

    #[test]
    fn test_is_installed_returns_bool() {
        let wrapper = CargoMutantsWrapper::new().unwrap();
        // Should return true or false, not panic
        let _installed = wrapper.is_installed();
    }

    #[test]
    #[ignore] // Only run if cargo-mutants is actually installed
    fn test_version_returns_string_when_installed() {
        let wrapper = CargoMutantsWrapper::new().unwrap();

        if wrapper.is_installed() {
            let version = wrapper.version();
            assert!(version.is_ok(), "version() should succeed when installed");

            let version_str = version.unwrap();
            assert!(!version_str.is_empty(), "Version string should not be empty");
            assert!(version_str.contains("cargo-mutants"), "Version should mention cargo-mutants");
        }
    }
}
