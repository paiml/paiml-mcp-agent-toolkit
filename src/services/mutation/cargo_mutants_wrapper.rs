//! REFACTOR Phase Implementation for PMAT-070-001: CargoMutantsWrapper
//!
//! Wrapper for cargo-mutants mutation testing tool.
//!
//! # Functionality
//!
//! - Detect cargo-mutants in PATH (cargo subcommand)
//! - Execute `cargo mutants --version`
//! - Parse and validate version (require v24.7.0+)
//! - Graceful error handling when not installed
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::mutation::cargo_mutants_wrapper::CargoMutantsWrapper;
//!
//! let wrapper = CargoMutantsWrapper::new()?;
//! if wrapper.is_installed() {
//!     let version = wrapper.version()?;
//!     println!("cargo-mutants version: {}", version);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::path::PathBuf;
use std::process::Command;

/// Type alias for Result with boxed error
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Wrapper for cargo-mutants subprocess execution
///
/// cargo-mutants is a cargo subcommand, so we store "cargo" as the path
/// and execute via `cargo mutants <args>`.
pub struct CargoMutantsWrapper {
    cargo_mutants_path: Option<PathBuf>,
}

impl CargoMutantsWrapper {
    /// Initialize wrapper and detect cargo-mutants in PATH
    ///
    /// Returns Ok even if cargo-mutants is not installed (cargo_mutants_path will be None).
    /// This allows graceful degradation with helpful error messages.
    ///
    /// # Errors
    ///
    /// This method should not fail under normal circumstances. It returns Ok even when
    /// cargo-mutants is not installed.
    pub fn new() -> Result<Self> {
        // cargo-mutants is a cargo subcommand, so check if `cargo` exists
        let cargo_path = which::which("cargo").ok();

        // Verify cargo-mutants subcommand is actually installed
        let path = if cargo_path.is_some() {
            // Try to run cargo mutants --version to verify it's installed
            let output = Command::new("cargo")
                .arg("mutants")
                .arg("--version")
                .output();

            match output {
                Ok(result) if result.status.success() => Some(PathBuf::from("cargo")),
                _ => None,
            }
        } else {
            None
        };

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

    /// Get the detected cargo path (if cargo-mutants is installed)
    ///
    /// Returns the path to the cargo binary used for executing cargo-mutants subcommand.
    pub fn cargo_mutants_path(&self) -> Option<&PathBuf> {
        self.cargo_mutants_path.as_ref()
    }

    /// Check if cargo-mutants is installed
    ///
    /// Returns `true` if cargo-mutants subcommand is available via `cargo mutants`.
    pub fn is_installed(&self) -> bool {
        self.cargo_mutants_path.is_some()
    }

    /// Get cargo-mutants version
    ///
    /// Executes `cargo mutants --version` and returns output.
    ///
    /// # Errors
    ///
    /// Returns error if cargo-mutants is not installed or execution fails.
    pub fn version(&self) -> Result<String> {
        self.cargo_mutants_path
            .as_ref()
            .ok_or("cargo-mutants not found in PATH")?;

        let output = Command::new("cargo")
            .arg("mutants")
            .arg("--version")
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "cargo mutants --version failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let version_str = String::from_utf8(output.stdout)?.trim().to_string();

        Ok(version_str)
    }

    /// Validate version meets minimum requirement (v24.7.0+)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - cargo-mutants is not installed
    /// - Version cannot be retrieved
    /// - Version format is invalid
    /// - Version is below minimum (v24.7.0)
    pub fn validate_version(&self) -> Result<()> {
        let version_str = self.version()?;
        let (major, minor) = Self::parse_version(&version_str)?;

        // Enforce minimum v24.7.0
        if major < 24 || (major == 24 && minor < 7) {
            return Err(format!(
                "cargo-mutants version {}.{} is too old. Minimum required: v24.7.0",
                major, minor
            )
            .into());
        }

        Ok(())
    }

    /// Parse cargo-mutants version string
    ///
    /// Expects format: "cargo-mutants X.Y.Z" where X is major, Y is minor, Z is patch.
    ///
    /// # Errors
    ///
    /// Returns error if version string format is invalid or version numbers cannot be parsed.
    fn parse_version(version_str: &str) -> Result<(u32, u32)> {
        // Parse version (example: "cargo-mutants 24.7.1")
        let parts: Vec<&str> = version_str.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(format!(
                "Unexpected version format: '{}' (expected 'cargo-mutants X.Y.Z')",
                version_str
            )
            .into());
        }

        let version_number = parts[1];
        let version_parts: Vec<&str> = version_number.split('.').collect();

        if version_parts.len() < 2 {
            return Err(format!(
                "Invalid version number: '{}' (expected X.Y.Z format)",
                version_number
            )
            .into());
        }

        let major: u32 = version_parts[0].parse().map_err(|_| {
            format!(
                "Invalid major version: '{}' (not a number)",
                version_parts[0]
            )
        })?;

        let minor: u32 = version_parts[1].parse().map_err(|_| {
            format!(
                "Invalid minor version: '{}' (not a number)",
                version_parts[1]
            )
        })?;

        Ok((major, minor))
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
            assert!(
                !version_str.is_empty(),
                "Version string should not be empty"
            );
            assert!(
                version_str.contains("cargo-mutants"),
                "Version should mention cargo-mutants"
            );
        }
    }
}
