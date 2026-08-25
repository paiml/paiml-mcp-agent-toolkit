#![cfg_attr(coverage_nightly, coverage(off))]
//! Utility functions and helpers for PMAT.
//!
//! This module contains shared utilities, helper functions, and common patterns
//! used throughout the PMAT codebase. These utilities follow the DRY (Don't Repeat
//! Yourself) principle and provide consistent implementations for common tasks.
//!
//! # Utilities
//!
//! - **helpers**: General-purpose helper functions for file operations, string
//!   manipulation, and common patterns
//!
//! # Example
//!
//! ```ignore
//! use pmat::utils::helpers;
//! use std::path::Path;
//!
//! // Example of using utility functions
//! let path = Path::new("./src/main.rs");
//! println!("Processing: {}", path.display());
//! ```

pub mod command_suggestions;
pub mod file_filter;
pub mod helpers;
pub mod path_validator;
pub mod pattern_helpers;
pub mod pmat_cache_dir;
pub mod scratch;
pub mod sovereign_compression;
pub mod string_truncate;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod pattern_helpers_integration_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
