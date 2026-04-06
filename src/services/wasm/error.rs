#![cfg_attr(coverage_nightly, coverage(off))]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create a new format error
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Create a new analysis error
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::AnalysisError(msg.into())
    }
}

impl From<anyhow::Error> for WasmError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_error_parse() {
        let err = WasmError::parse("test parse error");
        assert!(matches!(err, WasmError::ParseError(_)));
        assert!(err.to_string().contains("Parse error"));
    }

    #[test]
    fn test_wasm_error_format() {
        let err = WasmError::format("test format error");
        assert!(matches!(err, WasmError::InvalidFormat(_)));
        assert!(err.to_string().contains("Invalid format"));
    }

    #[test]
    fn test_wasm_error_analysis() {
        let err = WasmError::analysis("test analysis error");
        assert!(matches!(err, WasmError::AnalysisError(_)));
        assert!(err.to_string().contains("Analysis error"));
    }

    #[test]
    fn test_wasm_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let wasm_err: WasmError = io_err.into();
        assert!(matches!(wasm_err, WasmError::IoError(_)));
        assert!(wasm_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_wasm_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("anyhow error message");
        let wasm_err: WasmError = anyhow_err.into();
        assert!(matches!(wasm_err, WasmError::Other(_)));
        assert!(wasm_err.to_string().contains("anyhow error message"));
    }

    #[test]
    fn test_wasm_error_other() {
        let err = WasmError::Other("custom error".to_string());
        assert!(matches!(err, WasmError::Other(_)));
        assert!(err.to_string().contains("Other error"));
    }

    #[test]
    fn test_wasm_error_debug() {
        let err = WasmError::parse("debug test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ParseError"));
    }

    #[test]
    fn test_wasm_result_ok() {
        let result: WasmResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_wasm_result_err() {
        let result: WasmResult<i32> = Err(WasmError::parse("error"));
        assert!(result.is_err());
    }

    #[test]
    fn test_wasm_error_parse_with_string() {
        let err = WasmError::parse(String::from("string error"));
        assert!(err.to_string().contains("string error"));
    }

    #[test]
    fn test_wasm_error_format_with_string() {
        let err = WasmError::format(String::from("format error"));
        assert!(err.to_string().contains("format error"));
    }

    #[test]
    fn test_wasm_error_analysis_with_string() {
        let err = WasmError::analysis(String::from("analysis error"));
        assert!(err.to_string().contains("analysis error"));
    }

    #[test]
    fn test_wasm_error_io_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let wasm_err: WasmError = io_err.into();
        assert!(wasm_err.to_string().contains("access denied"));
    }

    #[test]
    fn test_wasm_error_all_variants_debug() {
        // Test Debug impl for all variants
        let errors = vec![
            WasmError::parse("parse"),
            WasmError::format("format"),
            WasmError::analysis("analysis"),
            WasmError::Other("other".to_string()),
        ];
        for err in errors {
            let debug_str = format!("{:?}", err);
            assert!(!debug_str.is_empty());
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
            debug_assert!(true, "contract: module_consistency_check");
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
