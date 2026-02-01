//! Coverage boost tests for error module
//! Tests for TemplateError, AnalysisError, and PmatError types

use crate::models::error::{AnalysisError, PmatError, TemplateError};
use std::path::PathBuf;

// ============ TemplateError Tests ============

#[test]
fn test_template_error_s3_error() {
    let err = TemplateError::S3Error {
        operation: "get".to_string(),
        source: anyhow::anyhow!("network failure"),
    };
    let msg = err.to_string();
    assert!(msg.contains("S3 operation failed"));
    assert!(msg.contains("get"));
}

#[test]
fn test_template_error_template_not_found() {
    let err = TemplateError::TemplateNotFound {
        uri: "template://test".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Template not found"));
    assert!(msg.contains("template://test"));
    assert_eq!(err.to_mcp_code(), -32001);
}

#[test]
fn test_template_error_not_found() {
    let err = TemplateError::NotFound("resource".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Not found"));
}

#[test]
fn test_template_error_invalid_uri() {
    let err = TemplateError::InvalidUri {
        uri: "invalid://uri".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Invalid template URI"));
    assert_eq!(err.to_mcp_code(), -32002);
}

#[test]
fn test_template_error_render_error() {
    let err = TemplateError::RenderError {
        line: 42,
        message: "syntax error".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Template rendering failed"));
    assert!(msg.contains("42"));
    assert_eq!(err.to_mcp_code(), -32004);
}

#[test]
fn test_template_error_validation_error() {
    let err = TemplateError::ValidationError {
        parameter: "name".to_string(),
        reason: "cannot be empty".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Parameter validation failed"));
    assert!(msg.contains("name"));
    assert_eq!(err.to_mcp_code(), -32003);
}

#[test]
fn test_template_error_invalid_utf8() {
    let err = TemplateError::InvalidUtf8("bad bytes".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Invalid UTF-8"));
}

#[test]
fn test_template_error_cache_error() {
    let err: TemplateError = anyhow::anyhow!("cache miss").into();
    let msg = err.to_string();
    assert!(msg.contains("Cache operation failed"));
    assert_eq!(err.to_mcp_code(), -32000); // Generic error
}

#[test]
fn test_template_error_debug() {
    let err = TemplateError::NotFound("test".to_string());
    let _ = format!("{:?}", err);
}

// ============ AnalysisError Tests ============

#[test]
fn test_analysis_error_parse_error() {
    let err = AnalysisError::ParseError("unexpected token".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Failed to parse file"));
}

#[test]
fn test_analysis_error_invalid_path() {
    let err = AnalysisError::InvalidPath("/bad/path".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Invalid path"));
}

#[test]
fn test_analysis_error_analysis_failed() {
    let err = AnalysisError::AnalysisFailed("timeout".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Analysis failed"));
}

#[test]
fn test_analysis_error_from_template() {
    let template_err = TemplateError::NotFound("test".to_string());
    let err: AnalysisError = template_err.into();
    let msg = err.to_string();
    assert!(msg.contains("Template error"));
}

#[test]
fn test_analysis_error_debug() {
    let err = AnalysisError::ParseError("test".to_string());
    let _ = format!("{:?}", err);
}

// ============ PmatError IO Tests ============

#[test]
fn test_pmat_error_file_not_found() {
    let err = PmatError::FileNotFound {
        path: PathBuf::from("/missing/file.rs"),
    };
    let msg = err.to_string();
    assert!(msg.contains("File not found"));
    assert!(msg.contains("/missing/file.rs"));
    assert_eq!(err.to_mcp_code(), -32001);
}

#[test]
fn test_pmat_error_directory_not_found() {
    let err = PmatError::DirectoryNotFound {
        path: PathBuf::from("/missing/dir"),
    };
    let msg = err.to_string();
    assert!(msg.contains("Directory not found"));
    assert_eq!(err.to_mcp_code(), -32002);
}

#[test]
fn test_pmat_error_permission_denied() {
    let err = PmatError::PermissionDenied {
        path: PathBuf::from("/protected/file"),
    };
    let msg = err.to_string();
    assert!(msg.contains("Permission denied"));
    assert_eq!(err.to_mcp_code(), -32003);
}

// ============ PmatError Parsing Tests ============

#[test]
fn test_pmat_error_parse_error() {
    let err = PmatError::ParseError {
        file: PathBuf::from("test.rs"),
        line: Some(10),
        message: "unexpected EOF".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Parse error"));
    assert!(msg.contains("test.rs"));
}

#[test]
fn test_pmat_error_syntax_error() {
    let err = PmatError::SyntaxError {
        language: "Rust".to_string(),
        message: "missing semicolon".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Syntax error"));
    assert!(msg.contains("Rust"));
}

#[test]
fn test_pmat_error_analysis_error() {
    let err = PmatError::AnalysisError {
        file: PathBuf::from("src/main.rs"),
        reason: "complexity too high".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Analysis failed"));
}

#[test]
fn test_pmat_error_ast_error() {
    let err = PmatError::AstError {
        details: "invalid node".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("AST processing failed"));
}

// ============ PmatError SIMD Tests ============

#[test]
fn test_pmat_error_simd_error() {
    let err = PmatError::SimdError {
        operation: "vector multiply".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("SIMD operation failed"));
}

#[test]
fn test_pmat_error_vectorized_error() {
    let err = PmatError::VectorizedError {
        details: "dimension mismatch".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Vectorized computation error"));
}

#[test]
fn test_pmat_error_alignment_error() {
    let err = PmatError::AlignmentError {
        expected: 16,
        actual: 8,
    };
    let msg = err.to_string();
    assert!(msg.contains("Cache alignment error"));
    assert!(msg.contains("16"));
    assert!(msg.contains("8"));
}

// ============ PmatError ML Tests ============

#[test]
fn test_pmat_error_model_error() {
    let err = PmatError::ModelError {
        model_name: "quality_predictor".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Model inference failed"));
}

#[test]
fn test_pmat_error_feature_extraction_error() {
    let err = PmatError::FeatureExtractionError {
        feature_type: "complexity".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Feature extraction failed"));
}

#[test]
fn test_pmat_error_training_data_error() {
    let err = PmatError::TrainingDataError {
        reason: "insufficient samples".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Training data invalid"));
}

// ============ PmatError Config Tests ============

#[test]
fn test_pmat_error_config_error() {
    let err = PmatError::ConfigError {
        key: "max_threads".to_string(),
        value: "-1".to_string(),
        reason: "must be positive".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Configuration error"));
    assert!(msg.contains("max_threads"));
}

#[test]
fn test_pmat_error_validation_error() {
    let err = PmatError::ValidationError {
        field: "email".to_string(),
        reason: "invalid format".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Validation failed"));
}

#[test]
fn test_pmat_error_format_error() {
    let err = PmatError::FormatError {
        expected: "JSON".to_string(),
        actual: "YAML".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Invalid format"));
}

// ============ PmatError Network Tests ============

#[test]
fn test_pmat_error_render_error() {
    let err = PmatError::RenderError {
        template: "report.html".to_string(),
        line: 25,
        message: "undefined variable".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Rendering failed"));
}

#[test]
fn test_pmat_error_network_error() {
    let err = PmatError::NetworkError {
        operation: "fetch".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Network error"));
}

#[test]
fn test_pmat_error_protocol_error() {
    let err = PmatError::ProtocolError {
        protocol: "MCP".to_string(),
        message: "invalid handshake".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Protocol error"));
}

#[test]
fn test_pmat_error_serialization_error() {
    let err = PmatError::SerializationError {
        format: "CBOR".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Serialization error"));
}

// ============ PmatError Storage Tests ============

#[test]
fn test_pmat_error_cache_error() {
    let err = PmatError::CacheError {
        operation: "get".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Cache error"));
}

#[test]
fn test_pmat_error_database_error() {
    let err = PmatError::DatabaseError {
        operation: "query".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Database error"));
}

#[test]
fn test_pmat_error_storage_full_error() {
    let err = PmatError::StorageFullError {
        available: 100,
        required: 500,
    };
    let msg = err.to_string();
    assert!(msg.contains("Storage full"));
    assert!(msg.contains("100"));
    assert!(msg.contains("500"));
}

#[test]
fn test_pmat_error_resource_exhausted() {
    let err = PmatError::ResourceExhausted {
        resource: "file descriptors".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Resource exhausted"));
}

#[test]
fn test_pmat_error_timeout_error() {
    let err = PmatError::TimeoutError {
        operation: "analysis".to_string(),
        timeout_ms: 5000,
    };
    let msg = err.to_string();
    assert!(msg.contains("Timeout occurred"));
    assert!(msg.contains("5000"));
}

#[test]
fn test_pmat_error_allocation_error() {
    let err = PmatError::AllocationError { size: 1_000_000 };
    let msg = err.to_string();
    assert!(msg.contains("Memory allocation failed"));
}

// ============ PmatError Git Tests ============

#[test]
fn test_pmat_error_git_error() {
    let err = PmatError::GitError {
        operation: "commit".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Git error"));
}

#[test]
fn test_pmat_error_repository_error() {
    let err = PmatError::RepositoryError {
        repo_path: PathBuf::from("/repo"),
    };
    let msg = err.to_string();
    assert!(msg.contains("Repository error"));
}

// ============ PmatError Quality Tests ============

#[test]
fn test_pmat_error_quality_gate_error() {
    let err = PmatError::QualityGateError {
        gate_name: "coverage".to_string(),
        reason: "below threshold".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Quality gate failed"));
}

#[test]
fn test_pmat_error_verification_error() {
    let err = PmatError::VerificationError {
        property: "type safety".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Verification failed"));
}

#[test]
fn test_pmat_error_proof_error() {
    let err = PmatError::ProofError {
        method: "induction".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Proof generation failed"));
}

// ============ PmatError From Conversion Tests ============

#[test]
fn test_pmat_error_from_analysis_error() {
    let analysis_err = AnalysisError::ParseError("test".to_string());
    let err: PmatError = analysis_err.into();
    let msg = err.to_string();
    assert!(msg.contains("Analysis error"));
}

#[test]
fn test_pmat_error_from_template_error() {
    let template_err = TemplateError::NotFound("test".to_string());
    let err: PmatError = template_err.into();
    let msg = err.to_string();
    assert!(msg.contains("Template error"));
}

#[test]
fn test_pmat_error_from_anyhow() {
    let anyhow_err = anyhow::anyhow!("generic error");
    let err: PmatError = anyhow_err.into();
    let msg = err.to_string();
    assert!(msg.contains("Generic error"));
    assert_eq!(err.to_mcp_code(), -32000);
}

// ============ PmatError Debug Tests ============

#[test]
fn test_pmat_error_debug_format() {
    let err = PmatError::FileNotFound {
        path: PathBuf::from("/test"),
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("FileNotFound"));
}

// ============ MCP Code Mapping Tests ============

#[test]
fn test_mcp_code_io_errors() {
    assert_eq!(
        PmatError::FileNotFound {
            path: PathBuf::from("test")
        }
        .to_mcp_code(),
        -32001
    );
    assert_eq!(
        PmatError::DirectoryNotFound {
            path: PathBuf::from("test")
        }
        .to_mcp_code(),
        -32002
    );
    assert_eq!(
        PmatError::PermissionDenied {
            path: PathBuf::from("test")
        }
        .to_mcp_code(),
        -32003
    );
}

#[test]
fn test_mcp_code_template_errors() {
    let err = PmatError::Template(TemplateError::TemplateNotFound {
        uri: "test".to_string(),
    });
    assert_eq!(err.to_mcp_code(), -32001);
}

#[test]
fn test_mcp_code_generic_errors() {
    let err: PmatError = anyhow::anyhow!("test").into();
    assert_eq!(err.to_mcp_code(), -32000);
}
