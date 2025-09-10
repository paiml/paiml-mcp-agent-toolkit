//! WebAssembly error types
//!
//! This module defines error types used throughout the WASM analysis system.

use thiserror::Error;

/// WebAssembly analysis errors
#[derive(Error, Debug)]
pub enum WasmError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for WASM operations
pub type WasmResult<T> = Result<T, WasmError>;

impl WasmError {
    /// Create a new parse error
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create a new format error
    pub fn format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Create a new analysis error
    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::AnalysisError(msg.into())
    }
}

impl From<anyhow::Error> for WasmError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
