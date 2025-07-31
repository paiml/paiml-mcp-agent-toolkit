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
//! ```
//! use pmat::utils::helpers::normalize_path;
//! use std::path::Path;
//!
//! let path = Path::new("./src/../src/main.rs");
//! let normalized = normalize_path(path);
//! assert_eq!(normalized.to_str().unwrap(), "src/main.rs");
//! ```

pub mod helpers;

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
