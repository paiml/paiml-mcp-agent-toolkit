//! Coverage boost tests for error module - Part 2
//! Comprehensive tests for error behavior, recovery, severity, and edge cases.
//!
//! This module extends coverage_boost_error.rs with additional tests for:
//! - is_recoverable() method on all relevant variants
//! - should_retry() method on all relevant variants
//! - severity() method returning correct ErrorSeverity for all variants
//! - ErrorSeverity Display, Clone, Copy, Debug, PartialEq, Eq implementations
//! - From implementations (IO errors, JSON errors, anyhow errors)
//! - Error source chain traversal via thiserror #[source]
//! - Edge cases: empty strings, unicode, special characters, extreme values

use crate::models::error::{AnalysisError, ErrorSeverity, PmatError, TemplateError};
use std::error::Error;
use std::path::PathBuf;

// ============ ErrorSeverity Tests ============

#[test]
fn test_error_severity_display_warning() {
    assert_eq!(ErrorSeverity::Warning.to_string(), "WARNING");
}

#[test]
fn test_error_severity_display_error() {
    assert_eq!(ErrorSeverity::Error.to_string(), "ERROR");
}

#[test]
fn test_error_severity_display_critical() {
    assert_eq!(ErrorSeverity::Critical.to_string(), "CRITICAL");
}

#[test]
fn test_error_severity_clone() {
    let severity = ErrorSeverity::Warning;
    let cloned = severity.clone();
    assert_eq!(severity, cloned);
}

#[test]
fn test_error_severity_copy() {
    let severity = ErrorSeverity::Error;
    let copied = severity;
    assert_eq!(severity, copied);
}

#[test]
fn test_error_severity_debug() {
    let debug_warning = format!("{:?}", ErrorSeverity::Warning);
    let debug_error = format!("{:?}", ErrorSeverity::Error);
    let debug_critical = format!("{:?}", ErrorSeverity::Critical);

    assert!(debug_warning.contains("Warning"));
    assert!(debug_error.contains("Error"));
    assert!(debug_critical.contains("Critical"));
}

#[test]
fn test_error_severity_partial_eq() {
    assert!(ErrorSeverity::Warning == ErrorSeverity::Warning);
    assert!(ErrorSeverity::Error == ErrorSeverity::Error);
    assert!(ErrorSeverity::Critical == ErrorSeverity::Critical);
    assert!(ErrorSeverity::Warning != ErrorSeverity::Error);
    assert!(ErrorSeverity::Error != ErrorSeverity::Critical);
    assert!(ErrorSeverity::Warning != ErrorSeverity::Critical);
}

// ============ PmatError is_recoverable Tests ============

#[test]
fn test_is_recoverable_timeout_error() {
    let err = PmatError::TimeoutError {
        operation: "fetch".to_string(),
        timeout_ms: 5000,
    };
    assert!(err.is_recoverable());
}

#[test]
fn test_is_recoverable_network_error() {
    let err = PmatError::NetworkError {
        operation: "connect".to_string(),
    };
    assert!(err.is_recoverable());
}

#[test]
fn test_is_recoverable_cache_error() {
    let err = PmatError::CacheError {
        operation: "read".to_string(),
    };
    assert!(err.is_recoverable());
}

#[test]
fn test_is_recoverable_resource_exhausted() {
    let err = PmatError::ResourceExhausted {
        resource: "memory".to_string(),
    };
    assert!(err.is_recoverable());
}

#[test]
fn test_not_recoverable_file_not_found() {
    let err = PmatError::FileNotFound {
        path: PathBuf::from("/missing"),
    };
    assert!(!err.is_recoverable());
}

#[test]
fn test_not_recoverable_directory_not_found() {
    let err = PmatError::DirectoryNotFound {
        path: PathBuf::from("/missing/dir"),
    };
    assert!(!err.is_recoverable());
}

#[test]
fn test_not_recoverable_permission_denied() {
    let err = PmatError::PermissionDenied {
        path: PathBuf::from("/protected"),
    };
    assert!(!err.is_recoverable());
}

#[test]
fn test_not_recoverable_parse_error() {
    let err = PmatError::ParseError {
        file: PathBuf::from("test.rs"),
        line: Some(10),
        message: "syntax error".to_string(),
    };
    assert!(!err.is_recoverable());
}

#[test]
fn test_not_recoverable_syntax_error() {
    let err = PmatError::SyntaxError {
        language: "Rust".to_string(),
        message: "invalid".to_string(),
    };
    assert!(!err.is_recoverable());
}

#[test]
fn test_not_recoverable_validation_error() {
    let err = PmatError::ValidationError {
        field: "email".to_string(),
        reason: "invalid format".to_string(),
    };
    assert!(!err.is_recoverable());
}

#[test]
fn test_not_recoverable_config_error() {
    let err = PmatError::ConfigError {
        key: "max_threads".to_string(),
        value: "-1".to_string(),
        reason: "must be positive".to_string(),
    };
    assert!(!err.is_recoverable());
}

#[test]
fn test_not_recoverable_quality_gate_error() {
    let err = PmatError::QualityGateError {
        gate_name: "coverage".to_string(),
        reason: "below threshold".to_string(),
    };
    assert!(!err.is_recoverable());
}

// ============ PmatError should_retry Tests ============

#[test]
fn test_should_retry_timeout_error() {
    let err = PmatError::TimeoutError {
        operation: "download".to_string(),
        timeout_ms: 10000,
    };
    assert!(err.should_retry());
}

#[test]
fn test_should_retry_network_error() {
    let err = PmatError::NetworkError {
        operation: "upload".to_string(),
    };
    assert!(err.should_retry());
}

#[test]
fn test_should_retry_resource_exhausted() {
    let err = PmatError::ResourceExhausted {
        resource: "connections".to_string(),
    };
    assert!(err.should_retry());
}

#[test]
fn test_should_not_retry_cache_error() {
    // Cache errors are recoverable but not retryable
    let err = PmatError::CacheError {
        operation: "invalidate".to_string(),
    };
    assert!(!err.should_retry());
}

#[test]
fn test_should_not_retry_file_not_found() {
    let err = PmatError::FileNotFound {
        path: PathBuf::from("/nonexistent"),
    };
    assert!(!err.should_retry());
}

#[test]
fn test_should_not_retry_parse_error() {
    let err = PmatError::ParseError {
        file: PathBuf::from("bad.rs"),
        line: None,
        message: "invalid".to_string(),
    };
    assert!(!err.should_retry());
}

#[test]
fn test_should_not_retry_validation_error() {
    let err = PmatError::ValidationError {
        field: "name".to_string(),
        reason: "required".to_string(),
    };
    assert!(!err.should_retry());
}

#[test]
fn test_should_not_retry_allocation_error() {
    let err = PmatError::AllocationError { size: 1_000_000_000 };
    assert!(!err.should_retry());
}

// ============ PmatError severity Tests - Warning ============

#[test]
fn test_severity_file_not_found_is_warning() {
    let err = PmatError::FileNotFound {
        path: PathBuf::from("/test"),
    };
    assert_eq!(err.severity(), ErrorSeverity::Warning);
}

#[test]
fn test_severity_directory_not_found_is_warning() {
    let err = PmatError::DirectoryNotFound {
        path: PathBuf::from("/test"),
    };
    assert_eq!(err.severity(), ErrorSeverity::Warning);
}

#[test]
fn test_severity_validation_error_is_warning() {
    let err = PmatError::ValidationError {
        field: "name".to_string(),
        reason: "empty".to_string(),
    };
    assert_eq!(err.severity(), ErrorSeverity::Warning);
}

// ============ PmatError severity Tests - Error ============

#[test]
fn test_severity_permission_denied_is_error() {
    let err = PmatError::PermissionDenied {
        path: PathBuf::from("/protected"),
    };
    assert_eq!(err.severity(), ErrorSeverity::Error);
}

#[test]
fn test_severity_parse_error_is_error() {
    let err = PmatError::ParseError {
        file: PathBuf::from("test.rs"),
        line: Some(1),
        message: "error".to_string(),
    };
    assert_eq!(err.severity(), ErrorSeverity::Error);
}

#[test]
fn test_severity_syntax_error_is_error() {
    let err = PmatError::SyntaxError {
        language: "Rust".to_string(),
        message: "error".to_string(),
    };
    assert_eq!(err.severity(), ErrorSeverity::Error);
}

#[test]
fn test_severity_config_error_is_error() {
    let err = PmatError::ConfigError {
        key: "key".to_string(),
        value: "value".to_string(),
        reason: "invalid".to_string(),
    };
    assert_eq!(err.severity(), ErrorSeverity::Error);
}

#[test]
fn test_severity_default_is_error() {
    // Network error defaults to Error severity
    let err = PmatError::NetworkError {
        operation: "test".to_string(),
    };
    assert_eq!(err.severity(), ErrorSeverity::Error);
}

#[test]
fn test_severity_protocol_error_is_error() {
    let err = PmatError::ProtocolError {
        protocol: "HTTP".to_string(),
        message: "error".to_string(),
    };
    assert_eq!(err.severity(), ErrorSeverity::Error);
}

// ============ PmatError severity Tests - Critical ============

#[test]
fn test_severity_allocation_error_is_critical() {
    let err = PmatError::AllocationError { size: 1000 };
    assert_eq!(err.severity(), ErrorSeverity::Critical);
}

#[test]
fn test_severity_storage_full_error_is_critical() {
    let err = PmatError::StorageFullError {
        available: 0,
        required: 1000,
    };
    assert_eq!(err.severity(), ErrorSeverity::Critical);
}

#[test]
fn test_severity_simd_error_is_critical() {
    let err = PmatError::SimdError {
        operation: "vectorize".to_string(),
    };
    assert_eq!(err.severity(), ErrorSeverity::Critical);
}

#[test]
fn test_severity_model_error_is_critical() {
    let err = PmatError::ModelError {
        model_name: "predictor".to_string(),
    };
    assert_eq!(err.severity(), ErrorSeverity::Critical);
}

// ============ From Implementation Tests ============

#[test]
fn test_pmat_error_from_io_error_not_found() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: PmatError = io_err.into();
    assert!(err.to_string().contains("IO error"));
    assert_eq!(err.to_mcp_code(), -32003);
}

#[test]
fn test_pmat_error_from_io_error_permission_denied() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let err: PmatError = io_err.into();
    assert!(err.to_string().contains("IO error"));
    assert_eq!(err.to_mcp_code(), -32003);
}

#[test]
fn test_pmat_error_from_io_error_connection_refused() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err: PmatError = io_err.into();
    assert!(err.to_string().contains("IO error"));
}

#[test]
fn test_pmat_error_from_io_error_interrupted() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Interrupted, "interrupted");
    let err: PmatError = io_err.into();
    assert!(err.to_string().contains("IO error"));
}

#[test]
fn test_pmat_error_from_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("not valid json")
        .err()
        .unwrap();
    let err: PmatError = json_err.into();
    assert!(err.to_string().contains("JSON error"));
    assert_eq!(err.to_mcp_code(), -32000);
}

#[test]
fn test_pmat_error_from_anyhow_error() {
    let anyhow_err = anyhow::anyhow!("something went wrong");
    let err: PmatError = anyhow_err.into();
    assert!(err.to_string().contains("Generic error"));
    assert_eq!(err.to_mcp_code(), -32000);
}

#[test]
fn test_template_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err: TemplateError = io_err.into();
    assert!(err.to_string().contains("IO operation failed"));
}

#[test]
fn test_template_error_from_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("{invalid}")
        .err()
        .unwrap();
    let err: TemplateError = json_err.into();
    assert!(err.to_string().contains("JSON serialization error"));
}

#[test]
fn test_template_error_from_anyhow() {
    let anyhow_err = anyhow::anyhow!("cache failure");
    let err: TemplateError = anyhow_err.into();
    assert!(err.to_string().contains("Cache operation failed"));
}

#[test]
fn test_analysis_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io problem");
    let err: AnalysisError = io_err.into();
    assert!(err.to_string().contains("IO error"));
}

#[test]
fn test_analysis_error_from_template_error() {
    let template_err = TemplateError::InvalidUri {
        uri: "bad://uri".to_string(),
    };
    let err: AnalysisError = template_err.into();
    assert!(err.to_string().contains("Template error"));
}

// ============ Error Source Chain Tests ============

#[test]
fn test_template_error_s3_error_source() {
    let err = TemplateError::S3Error {
        operation: "GetObject".to_string(),
        source: anyhow::anyhow!("network timeout"),
    };
    // The #[source] attribute provides error chain
    assert!(err.source().is_some());
}

#[test]
fn test_template_error_cache_error_source() {
    let err = TemplateError::CacheError(anyhow::anyhow!("cache miss"));
    // #[from] implies #[source]
    assert!(err.source().is_some());
}

#[test]
fn test_template_error_json_error_source() {
    let json_err = serde_json::from_str::<serde_json::Value>("bad").err().unwrap();
    let err = TemplateError::JsonError(json_err);
    assert!(err.source().is_some());
}

#[test]
fn test_template_error_io_error_source() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err = TemplateError::Io(io_err);
    assert!(err.source().is_some());
}

#[test]
fn test_pmat_error_io_source() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err = PmatError::Io(io_err);
    assert!(err.source().is_some());
}

#[test]
fn test_pmat_error_json_source() {
    let json_err = serde_json::from_str::<serde_json::Value>("x").err().unwrap();
    let err = PmatError::Json(json_err);
    assert!(err.source().is_some());
}

#[test]
fn test_pmat_error_template_source() {
    let template_err = TemplateError::NotFound("test".to_string());
    let err = PmatError::Template(template_err);
    assert!(err.source().is_some());
}

#[test]
fn test_pmat_error_analysis_source() {
    let analysis_err = AnalysisError::ParseError("test".to_string());
    let err = PmatError::Analysis(analysis_err);
    assert!(err.source().is_some());
}

#[test]
fn test_pmat_error_other_source() {
    let anyhow_err = anyhow::anyhow!("other");
    let err = PmatError::Other(anyhow_err);
    assert!(err.source().is_some());
}

// ============ MCP Code Category Tests ============

#[test]
fn test_mcp_code_parsing_errors() {
    assert_eq!(
        PmatError::ParseError {
            file: PathBuf::new(),
            line: None,
            message: String::new()
        }
        .to_mcp_code(),
        -32004
    );
    assert_eq!(
        PmatError::SyntaxError {
            language: String::new(),
            message: String::new()
        }
        .to_mcp_code(),
        -32005
    );
    assert_eq!(
        PmatError::AnalysisError {
            file: PathBuf::new(),
            reason: String::new()
        }
        .to_mcp_code(),
        -32006
    );
    assert_eq!(
        PmatError::AstError {
            details: String::new()
        }
        .to_mcp_code(),
        -32007
    );
}

#[test]
fn test_mcp_code_simd_errors() {
    assert_eq!(
        PmatError::SimdError {
            operation: String::new()
        }
        .to_mcp_code(),
        -32008
    );
    assert_eq!(
        PmatError::VectorizedError {
            details: String::new()
        }
        .to_mcp_code(),
        -32009
    );
    assert_eq!(
        PmatError::AlignmentError {
            expected: 0,
            actual: 0
        }
        .to_mcp_code(),
        -32010
    );
}

#[test]
fn test_mcp_code_ml_errors() {
    assert_eq!(
        PmatError::ModelError {
            model_name: String::new()
        }
        .to_mcp_code(),
        -32011
    );
    assert_eq!(
        PmatError::FeatureExtractionError {
            feature_type: String::new()
        }
        .to_mcp_code(),
        -32012
    );
    assert_eq!(
        PmatError::TrainingDataError {
            reason: String::new()
        }
        .to_mcp_code(),
        -32013
    );
}

#[test]
fn test_mcp_code_config_errors() {
    assert_eq!(
        PmatError::ConfigError {
            key: String::new(),
            value: String::new(),
            reason: String::new()
        }
        .to_mcp_code(),
        -32014
    );
    assert_eq!(
        PmatError::ValidationError {
            field: String::new(),
            reason: String::new()
        }
        .to_mcp_code(),
        -32015
    );
    assert_eq!(
        PmatError::FormatError {
            expected: String::new(),
            actual: String::new()
        }
        .to_mcp_code(),
        -32016
    );
}

#[test]
fn test_mcp_code_network_errors() {
    assert_eq!(
        PmatError::RenderError {
            template: String::new(),
            line: 0,
            message: String::new()
        }
        .to_mcp_code(),
        -32018
    );
    assert_eq!(
        PmatError::NetworkError {
            operation: String::new()
        }
        .to_mcp_code(),
        -32019
    );
    assert_eq!(
        PmatError::ProtocolError {
            protocol: String::new(),
            message: String::new()
        }
        .to_mcp_code(),
        -32020
    );
    assert_eq!(
        PmatError::SerializationError {
            format: String::new()
        }
        .to_mcp_code(),
        -32021
    );
}

#[test]
fn test_mcp_code_storage_errors() {
    assert_eq!(
        PmatError::CacheError {
            operation: String::new()
        }
        .to_mcp_code(),
        -32022
    );
    assert_eq!(
        PmatError::DatabaseError {
            operation: String::new()
        }
        .to_mcp_code(),
        -32023
    );
    assert_eq!(
        PmatError::StorageFullError {
            available: 0,
            required: 0
        }
        .to_mcp_code(),
        -32024
    );
    assert_eq!(
        PmatError::ResourceExhausted {
            resource: String::new()
        }
        .to_mcp_code(),
        -32025
    );
    assert_eq!(
        PmatError::TimeoutError {
            operation: String::new(),
            timeout_ms: 0
        }
        .to_mcp_code(),
        -32026
    );
    assert_eq!(PmatError::AllocationError { size: 0 }.to_mcp_code(), -32027);
}

#[test]
fn test_mcp_code_vcs_errors() {
    assert_eq!(
        PmatError::GitError {
            operation: String::new()
        }
        .to_mcp_code(),
        -32028
    );
    assert_eq!(
        PmatError::RepositoryError {
            repo_path: PathBuf::new()
        }
        .to_mcp_code(),
        -32029
    );
    assert_eq!(
        PmatError::QualityGateError {
            gate_name: String::new(),
            reason: String::new()
        }
        .to_mcp_code(),
        -32030
    );
    assert_eq!(
        PmatError::VerificationError {
            property: String::new()
        }
        .to_mcp_code(),
        -32031
    );
    assert_eq!(
        PmatError::ProofError {
            method: String::new()
        }
        .to_mcp_code(),
        -32032
    );
}

// ============ Edge Case Tests ============

#[test]
fn test_error_with_empty_strings() {
    let err = PmatError::ParseError {
        file: PathBuf::from(""),
        line: None,
        message: "".to_string(),
    };
    // Should not panic
    let _ = err.to_string();
    let _ = format!("{:?}", err);
    let _ = err.to_mcp_code();
    let _ = err.is_recoverable();
    let _ = err.should_retry();
    let _ = err.severity();
}

#[test]
fn test_error_with_unicode_characters() {
    let err = PmatError::ParseError {
        file: PathBuf::from("/path/with/\u{1F600}/emoji"),
        line: Some(42),
        message: "\u{4E2D}\u{6587} error with \u{1F4A5}".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("\u{4E2D}\u{6587}"));
    assert!(msg.contains("\u{1F4A5}"));
}

#[test]
fn test_error_with_special_characters() {
    let err = PmatError::ConfigError {
        key: "key\"with'quotes".to_string(),
        value: "value\nwith\ttabs".to_string(),
        reason: "reason\\with\\backslashes".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("key\"with'quotes"));
}

#[test]
fn test_error_with_very_long_string() {
    let long_str = "x".repeat(100_000);
    let err = PmatError::AnalysisError {
        file: PathBuf::from(&long_str),
        reason: long_str.clone(),
    };
    let msg = err.to_string();
    assert!(msg.len() > 100_000);
}

#[test]
fn test_error_with_extreme_numeric_values() {
    let err = PmatError::StorageFullError {
        available: u64::MAX,
        required: u64::MAX,
    };
    let msg = err.to_string();
    assert!(msg.contains(&u64::MAX.to_string()));

    let err2 = PmatError::AllocationError { size: usize::MAX };
    let msg2 = err2.to_string();
    assert!(msg2.contains(&usize::MAX.to_string()));

    let err3 = PmatError::TimeoutError {
        operation: "test".to_string(),
        timeout_ms: u64::MAX,
    };
    let msg3 = err3.to_string();
    assert!(msg3.contains(&u64::MAX.to_string()));
}

#[test]
fn test_error_with_zero_values() {
    let err = PmatError::StorageFullError {
        available: 0,
        required: 0,
    };
    let msg = err.to_string();
    assert!(msg.contains("0"));

    let err2 = PmatError::AlignmentError {
        expected: 0,
        actual: 0,
    };
    let msg2 = err2.to_string();
    assert!(msg2.contains("0"));
}

#[test]
fn test_error_with_path_containing_spaces() {
    let err = PmatError::FileNotFound {
        path: PathBuf::from("/path with spaces/file name.rs"),
    };
    let msg = err.to_string();
    assert!(msg.contains("path with spaces"));
}

#[test]
fn test_error_with_path_containing_dots() {
    let err = PmatError::DirectoryNotFound {
        path: PathBuf::from("../relative/./path"),
    };
    let msg = err.to_string();
    assert!(msg.contains("../relative/./path"));
}

// ============ TemplateError Edge Cases ============

#[test]
fn test_template_error_render_with_high_line_number() {
    let err = TemplateError::RenderError {
        line: u32::MAX,
        message: "error at end of file".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains(&u32::MAX.to_string()));
}

#[test]
fn test_template_error_all_mcp_codes() {
    // Test that all special MCP codes are assigned correctly
    let codes = [
        (
            TemplateError::TemplateNotFound {
                uri: "t".to_string(),
            },
            -32001,
        ),
        (
            TemplateError::InvalidUri {
                uri: "u".to_string(),
            },
            -32002,
        ),
        (
            TemplateError::ValidationError {
                parameter: "p".to_string(),
                reason: "r".to_string(),
            },
            -32003,
        ),
        (
            TemplateError::RenderError {
                line: 1,
                message: "m".to_string(),
            },
            -32004,
        ),
    ];

    for (err, expected) in codes {
        assert_eq!(err.to_mcp_code(), expected);
    }
}

#[test]
fn test_template_error_generic_mcp_codes() {
    // These should all return -32000 (generic)
    let generic_errors = [
        TemplateError::NotFound("x".to_string()),
        TemplateError::InvalidUtf8("x".to_string()),
        TemplateError::CacheError(anyhow::anyhow!("x")),
    ];

    for err in generic_errors {
        assert_eq!(err.to_mcp_code(), -32000);
    }
}

// ============ Debug Implementation Tests ============

#[test]
fn test_all_pmat_variants_have_debug() {
    // Ensure all variants can be debug-printed without panic
    let variants: Vec<PmatError> = vec![
        PmatError::FileNotFound {
            path: PathBuf::from("test"),
        },
        PmatError::DirectoryNotFound {
            path: PathBuf::from("test"),
        },
        PmatError::PermissionDenied {
            path: PathBuf::from("test"),
        },
        PmatError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
        PmatError::ParseError {
            file: PathBuf::from("test"),
            line: Some(1),
            message: "test".to_string(),
        },
        PmatError::SyntaxError {
            language: "test".to_string(),
            message: "test".to_string(),
        },
        PmatError::AnalysisError {
            file: PathBuf::from("test"),
            reason: "test".to_string(),
        },
        PmatError::AstError {
            details: "test".to_string(),
        },
        PmatError::SimdError {
            operation: "test".to_string(),
        },
        PmatError::VectorizedError {
            details: "test".to_string(),
        },
        PmatError::AlignmentError {
            expected: 32,
            actual: 16,
        },
        PmatError::ModelError {
            model_name: "test".to_string(),
        },
        PmatError::FeatureExtractionError {
            feature_type: "test".to_string(),
        },
        PmatError::TrainingDataError {
            reason: "test".to_string(),
        },
        PmatError::ConfigError {
            key: "test".to_string(),
            value: "test".to_string(),
            reason: "test".to_string(),
        },
        PmatError::ValidationError {
            field: "test".to_string(),
            reason: "test".to_string(),
        },
        PmatError::FormatError {
            expected: "test".to_string(),
            actual: "test".to_string(),
        },
        PmatError::Template(TemplateError::NotFound("test".to_string())),
        PmatError::RenderError {
            template: "test".to_string(),
            line: 1,
            message: "test".to_string(),
        },
        PmatError::NetworkError {
            operation: "test".to_string(),
        },
        PmatError::ProtocolError {
            protocol: "test".to_string(),
            message: "test".to_string(),
        },
        PmatError::SerializationError {
            format: "test".to_string(),
        },
        PmatError::CacheError {
            operation: "test".to_string(),
        },
        PmatError::DatabaseError {
            operation: "test".to_string(),
        },
        PmatError::StorageFullError {
            available: 100,
            required: 200,
        },
        PmatError::ResourceExhausted {
            resource: "test".to_string(),
        },
        PmatError::TimeoutError {
            operation: "test".to_string(),
            timeout_ms: 1000,
        },
        PmatError::AllocationError { size: 1000 },
        PmatError::GitError {
            operation: "test".to_string(),
        },
        PmatError::RepositoryError {
            repo_path: PathBuf::from("test"),
        },
        PmatError::QualityGateError {
            gate_name: "test".to_string(),
            reason: "test".to_string(),
        },
        PmatError::VerificationError {
            property: "test".to_string(),
        },
        PmatError::ProofError {
            method: "test".to_string(),
        },
        PmatError::Analysis(AnalysisError::ParseError("test".to_string())),
        PmatError::Other(anyhow::anyhow!("test")),
    ];

    for err in variants {
        let debug = format!("{:?}", err);
        let display = format!("{}", err);
        assert!(!debug.is_empty());
        assert!(!display.is_empty());
    }
}

#[test]
fn test_all_template_variants_have_debug() {
    let variants: Vec<TemplateError> = vec![
        TemplateError::S3Error {
            operation: "test".to_string(),
            source: anyhow::anyhow!("test"),
        },
        TemplateError::TemplateNotFound {
            uri: "test".to_string(),
        },
        TemplateError::NotFound("test".to_string()),
        TemplateError::InvalidUri {
            uri: "test".to_string(),
        },
        TemplateError::RenderError {
            line: 1,
            message: "test".to_string(),
        },
        TemplateError::ValidationError {
            parameter: "test".to_string(),
            reason: "test".to_string(),
        },
        TemplateError::InvalidUtf8("test".to_string()),
        TemplateError::CacheError(anyhow::anyhow!("test")),
    ];

    for err in variants {
        let debug = format!("{:?}", err);
        let display = format!("{}", err);
        assert!(!debug.is_empty());
        assert!(!display.is_empty());
    }
}

#[test]
fn test_all_analysis_variants_have_debug() {
    let variants: Vec<AnalysisError> = vec![
        AnalysisError::ParseError("test".to_string()),
        AnalysisError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
        AnalysisError::InvalidPath("test".to_string()),
        AnalysisError::AnalysisFailed("test".to_string()),
        AnalysisError::Template(TemplateError::NotFound("test".to_string())),
    ];

    for err in variants {
        let debug = format!("{:?}", err);
        let display = format!("{}", err);
        assert!(!debug.is_empty());
        assert!(!display.is_empty());
    }
}

// ============ Consistency Tests ============

#[test]
fn test_retry_implies_recoverable() {
    // Any error that should_retry must also be is_recoverable
    let errors: Vec<PmatError> = vec![
        PmatError::TimeoutError {
            operation: "x".to_string(),
            timeout_ms: 1000,
        },
        PmatError::NetworkError {
            operation: "x".to_string(),
        },
        PmatError::ResourceExhausted {
            resource: "x".to_string(),
        },
    ];

    for err in errors {
        if err.should_retry() {
            assert!(
                err.is_recoverable(),
                "should_retry implies is_recoverable for {:?}",
                err
            );
        }
    }
}

#[test]
fn test_mcp_codes_are_negative() {
    // All MCP codes should be negative (JSON-RPC error convention)
    let errors: Vec<PmatError> = vec![
        PmatError::FileNotFound {
            path: PathBuf::new(),
        },
        PmatError::ParseError {
            file: PathBuf::new(),
            line: None,
            message: String::new(),
        },
        PmatError::SimdError {
            operation: String::new(),
        },
        PmatError::ModelError {
            model_name: String::new(),
        },
        PmatError::ConfigError {
            key: String::new(),
            value: String::new(),
            reason: String::new(),
        },
        PmatError::NetworkError {
            operation: String::new(),
        },
        PmatError::CacheError {
            operation: String::new(),
        },
        PmatError::GitError {
            operation: String::new(),
        },
    ];

    for err in errors {
        assert!(err.to_mcp_code() < 0, "MCP code should be negative for {:?}", err);
    }
}

#[test]
fn test_severity_never_panics() {
    // severity() should never panic regardless of input
    let errors: Vec<PmatError> = vec![
        PmatError::FileNotFound {
            path: PathBuf::from(""),
        },
        PmatError::TimeoutError {
            operation: "".to_string(),
            timeout_ms: 0,
        },
        PmatError::Other(anyhow::anyhow!("")),
    ];

    for err in errors {
        let _ = err.severity(); // Should not panic
    }
}
