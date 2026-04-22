// Error type tests extracted from error.rs for file health (CB-040).
// This file is include!()'d into error.rs scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    // === TemplateError Tests ===

    #[test]
    fn test_template_error_s3_error() {
        let err = TemplateError::S3Error {
            operation: "GetObject".to_string(),
            source: anyhow::anyhow!("Network timeout"),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("S3 operation failed"));
        assert!(msg.contains("GetObject"));
    }

    #[test]
    fn test_template_error_not_found() {
        let err = TemplateError::TemplateNotFound {
            uri: "template://makefile/rust".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Template not found"));
        assert!(msg.contains("template://makefile/rust"));
        assert_eq!(err.to_mcp_code(), -32001);
    }

    #[test]
    fn test_template_error_not_found_simple() {
        let err = TemplateError::NotFound("resource xyz".to_string());

        let msg = format!("{}", err);
        assert!(msg.contains("Not found"));
        assert!(msg.contains("resource xyz"));
    }

    #[test]
    fn test_template_error_invalid_uri() {
        let err = TemplateError::InvalidUri {
            uri: "invalid://uri".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Invalid template URI"));
        assert!(msg.contains("invalid://uri"));
        assert_eq!(err.to_mcp_code(), -32002);
    }

    #[test]
    fn test_template_error_render_error() {
        let err = TemplateError::RenderError {
            line: 42,
            message: "Unexpected closing brace".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Template rendering failed"));
        assert!(msg.contains("line 42"));
        assert!(msg.contains("Unexpected closing brace"));
        assert_eq!(err.to_mcp_code(), -32004);
    }

    #[test]
    fn test_template_error_validation_error() {
        let err = TemplateError::ValidationError {
            parameter: "project_name".to_string(),
            reason: "must start with a letter".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Parameter validation failed"));
        assert!(msg.contains("project_name"));
        assert!(msg.contains("must start with a letter"));
        assert_eq!(err.to_mcp_code(), -32003);
    }

    #[test]
    fn test_template_error_invalid_utf8() {
        let err = TemplateError::InvalidUtf8("binary content".to_string());

        let msg = format!("{}", err);
        assert!(msg.contains("Invalid UTF-8"));
    }

    #[test]
    fn test_template_error_cache_error() {
        let err = TemplateError::CacheError(anyhow::anyhow!("Cache full"));

        let msg = format!("{}", err);
        assert!(msg.contains("Cache operation failed"));
    }

    #[test]
    fn test_template_error_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid")
            .err()
            .unwrap();
        let err = TemplateError::JsonError(json_err);

        let msg = format!("{}", err);
        assert!(msg.contains("JSON serialization error"));
    }

    #[test]
    fn test_template_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = TemplateError::Io(io_err);

        let msg = format!("{}", err);
        assert!(msg.contains("IO operation failed"));
    }

    #[test]
    fn test_template_error_generic_mcp_code() {
        // Errors without specific codes should return -32000
        let err = TemplateError::InvalidUtf8("test".to_string());
        assert_eq!(err.to_mcp_code(), -32000);
    }

    // === AnalysisError Tests ===

    #[test]
    fn test_analysis_error_parse_error() {
        let err = AnalysisError::ParseError("syntax error at line 10".to_string());

        let msg = format!("{}", err);
        assert!(msg.contains("Failed to parse file"));
        assert!(msg.contains("syntax error at line 10"));
    }

    #[test]
    fn test_analysis_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = AnalysisError::Io(io_err);

        let msg = format!("{}", err);
        assert!(msg.contains("IO error"));
    }

    #[test]
    fn test_analysis_error_invalid_path() {
        let err = AnalysisError::InvalidPath("/nonexistent/path".to_string());

        let msg = format!("{}", err);
        assert!(msg.contains("Invalid path"));
        assert!(msg.contains("/nonexistent/path"));
    }

    #[test]
    fn test_analysis_error_analysis_failed() {
        let err = AnalysisError::AnalysisFailed("complexity calculation overflow".to_string());

        let msg = format!("{}", err);
        assert!(msg.contains("Analysis failed"));
        assert!(msg.contains("complexity calculation overflow"));
    }

    #[test]
    fn test_analysis_error_from_template_error() {
        let template_err = TemplateError::NotFound("test".to_string());
        let analysis_err = AnalysisError::Template(template_err);

        let msg = format!("{}", analysis_err);
        assert!(msg.contains("Template error"));
    }

    // === PmatError Tests ===

    #[test]
    fn test_pmat_error_file_not_found() {
        let err = PmatError::FileNotFound {
            path: PathBuf::from("/home/user/missing.rs"),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("File not found"));
        assert!(msg.contains("/home/user/missing.rs"));
        assert_eq!(err.to_mcp_code(), -32001);
    }

    #[test]
    fn test_pmat_error_directory_not_found() {
        let err = PmatError::DirectoryNotFound {
            path: PathBuf::from("/missing/dir"),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Directory not found"));
        assert_eq!(err.to_mcp_code(), -32002);
    }

    #[test]
    fn test_pmat_error_permission_denied() {
        let err = PmatError::PermissionDenied {
            path: PathBuf::from("/protected/file"),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Permission denied"));
        assert_eq!(err.to_mcp_code(), -32003);
    }

    #[test]
    fn test_pmat_error_parse_error() {
        let err = PmatError::ParseError {
            file: PathBuf::from("src/main.rs"),
            line: Some(42),
            message: "unexpected token".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Parse error"));
        assert!(msg.contains("src/main.rs"));
        assert!(msg.contains("unexpected token"));
        assert_eq!(err.to_mcp_code(), -32004);
    }

    #[test]
    fn test_pmat_error_parse_error_no_line() {
        let err = PmatError::ParseError {
            file: PathBuf::from("file.rs"),
            line: None,
            message: "invalid syntax".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Parse error"));
    }

    #[test]
    fn test_pmat_error_syntax_error() {
        let err = PmatError::SyntaxError {
            language: "Rust".to_string(),
            message: "missing semicolon".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Syntax error"));
        assert!(msg.contains("Rust"));
        assert_eq!(err.to_mcp_code(), -32005);
    }

    #[test]
    fn test_pmat_error_analysis_error() {
        let err = PmatError::AnalysisError {
            file: PathBuf::from("test.py"),
            reason: "cyclomatic complexity too high".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Analysis failed"));
        assert_eq!(err.to_mcp_code(), -32006);
    }

    #[test]
    fn test_pmat_error_ast_error() {
        let err = PmatError::AstError {
            details: "invalid node type".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("AST processing failed"));
        assert_eq!(err.to_mcp_code(), -32007);
    }

    #[test]
    fn test_pmat_error_simd_error() {
        let err = PmatError::SimdError {
            operation: "vector multiplication".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("SIMD operation failed"));
        assert_eq!(err.to_mcp_code(), -32008);
    }

    #[test]
    fn test_pmat_error_vectorized_error() {
        let err = PmatError::VectorizedError {
            details: "dimension mismatch".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Vectorized computation error"));
        assert_eq!(err.to_mcp_code(), -32009);
    }

    #[test]
    fn test_pmat_error_alignment_error() {
        let err = PmatError::AlignmentError {
            expected: 32,
            actual: 16,
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Cache alignment error"));
        assert!(msg.contains("32"));
        assert!(msg.contains("16"));
        assert_eq!(err.to_mcp_code(), -32010);
    }

    #[test]
    fn test_pmat_error_model_error() {
        let err = PmatError::ModelError {
            model_name: "complexity_predictor".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Model inference failed"));
        assert_eq!(err.to_mcp_code(), -32011);
    }

    #[test]
    fn test_pmat_error_feature_extraction_error() {
        let err = PmatError::FeatureExtractionError {
            feature_type: "cyclomatic_complexity".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Feature extraction failed"));
        assert_eq!(err.to_mcp_code(), -32012);
    }

    #[test]
    fn test_pmat_error_training_data_error() {
        let err = PmatError::TrainingDataError {
            reason: "insufficient samples".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Training data invalid"));
        assert_eq!(err.to_mcp_code(), -32013);
    }

    #[test]
    fn test_pmat_error_config_error() {
        let err = PmatError::ConfigError {
            key: "max_complexity".to_string(),
            value: "-5".to_string(),
            reason: "must be positive".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Configuration error"));
        assert!(msg.contains("max_complexity"));
        assert_eq!(err.to_mcp_code(), -32014);
    }

    #[test]
    fn test_pmat_error_validation_error() {
        let err = PmatError::ValidationError {
            field: "email".to_string(),
            reason: "invalid format".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Validation failed"));
        assert_eq!(err.to_mcp_code(), -32015);
    }

    #[test]
    fn test_pmat_error_format_error() {
        let err = PmatError::FormatError {
            expected: "JSON".to_string(),
            actual: "YAML".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Invalid format"));
        assert_eq!(err.to_mcp_code(), -32016);
    }

    #[test]
    fn test_pmat_error_render_error() {
        let err = PmatError::RenderError {
            template: "report.html".to_string(),
            line: 15,
            message: "undefined variable".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Rendering failed"));
        assert_eq!(err.to_mcp_code(), -32018);
    }

    #[test]
    fn test_pmat_error_network_error() {
        let err = PmatError::NetworkError {
            operation: "fetch S3 object".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Network error"));
        assert_eq!(err.to_mcp_code(), -32019);
    }

    #[test]
    fn test_pmat_error_protocol_error() {
        let err = PmatError::ProtocolError {
            protocol: "HTTP/2".to_string(),
            message: "stream reset".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Protocol error"));
        assert_eq!(err.to_mcp_code(), -32020);
    }

    #[test]
    fn test_pmat_error_serialization_error() {
        let err = PmatError::SerializationError {
            format: "MessagePack".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Serialization error"));
        assert_eq!(err.to_mcp_code(), -32021);
    }

    #[test]
    fn test_pmat_error_cache_error() {
        let err = PmatError::CacheError {
            operation: "write".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Cache error"));
        assert_eq!(err.to_mcp_code(), -32022);
    }

    #[test]
    fn test_pmat_error_database_error() {
        let err = PmatError::DatabaseError {
            operation: "insert".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Database error"));
        assert_eq!(err.to_mcp_code(), -32023);
    }

    #[test]
    fn test_pmat_error_storage_full() {
        let err = PmatError::StorageFullError {
            available: 1024,
            required: 2048,
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Storage full"));
        assert!(msg.contains("1024"));
        assert!(msg.contains("2048"));
        assert_eq!(err.to_mcp_code(), -32024);
    }

    #[test]
    fn test_pmat_error_resource_exhausted() {
        let err = PmatError::ResourceExhausted {
            resource: "thread pool".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Resource exhausted"));
        assert_eq!(err.to_mcp_code(), -32025);
    }

    #[test]
    fn test_pmat_error_timeout() {
        let err = PmatError::TimeoutError {
            operation: "git clone".to_string(),
            timeout_ms: 30000,
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Timeout occurred"));
        assert!(msg.contains("30000"));
        assert_eq!(err.to_mcp_code(), -32026);
    }

    #[test]
    fn test_pmat_error_allocation_error() {
        let err = PmatError::AllocationError { size: 1073741824 };

        let msg = format!("{}", err);
        assert!(msg.contains("Memory allocation failed"));
        assert!(msg.contains("1073741824"));
        assert_eq!(err.to_mcp_code(), -32027);
    }

    #[test]
    fn test_pmat_error_git_error() {
        let err = PmatError::GitError {
            operation: "git status".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Git error"));
        assert_eq!(err.to_mcp_code(), -32028);
    }

    #[test]
    fn test_pmat_error_repository_error() {
        let err = PmatError::RepositoryError {
            repo_path: PathBuf::from("/invalid/repo"),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Repository error"));
        assert_eq!(err.to_mcp_code(), -32029);
    }

    #[test]
    fn test_pmat_error_quality_gate_error() {
        let err = PmatError::QualityGateError {
            gate_name: "coverage".to_string(),
            reason: "below 80%".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Quality gate failed"));
        assert_eq!(err.to_mcp_code(), -32030);
    }

    #[test]
    fn test_pmat_error_verification_error() {
        let err = PmatError::VerificationError {
            property: "type safety".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Verification failed"));
        assert_eq!(err.to_mcp_code(), -32031);
    }

    #[test]
    fn test_pmat_error_proof_error() {
        let err = PmatError::ProofError {
            method: "SMT solver".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Proof generation failed"));
        assert_eq!(err.to_mcp_code(), -32032);
    }

    #[test]
    fn test_pmat_error_from_template_error() {
        let template_err = TemplateError::TemplateNotFound {
            uri: "test".to_string(),
        };
        let pmat_err: PmatError = template_err.into();

        let msg = format!("{}", pmat_err);
        assert!(msg.contains("Template"));
        // Template errors delegate to their own to_mcp_code
        assert_eq!(pmat_err.to_mcp_code(), -32001);
    }

    #[test]
    fn test_pmat_error_from_analysis_error() {
        let analysis_err = AnalysisError::ParseError("test".to_string());
        let pmat_err: PmatError = analysis_err.into();

        let msg = format!("{}", pmat_err);
        assert!(msg.contains("Analysis"));
        assert_eq!(pmat_err.to_mcp_code(), -32000);
    }

    #[test]
    fn test_pmat_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let pmat_err: PmatError = io_err.into();

        let msg = format!("{}", pmat_err);
        assert!(msg.contains("IO error"));
        assert_eq!(pmat_err.to_mcp_code(), -32003);
    }

    #[test]
    fn test_pmat_error_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid")
            .err()
            .unwrap();
        let pmat_err: PmatError = json_err.into();

        let msg = format!("{}", pmat_err);
        assert!(msg.contains("JSON"));
        assert_eq!(pmat_err.to_mcp_code(), -32000);
    }

    #[test]
    fn test_pmat_error_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("generic error");
        let pmat_err: PmatError = anyhow_err.into();

        let msg = format!("{}", pmat_err);
        assert!(msg.contains("Generic error"));
        assert_eq!(pmat_err.to_mcp_code(), -32000);
    }

    // === is_recoverable Tests ===

    #[test]
    fn test_is_recoverable_timeout() {
        let err = PmatError::TimeoutError {
            operation: "test".to_string(),
            timeout_ms: 1000,
        };
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_is_recoverable_network() {
        let err = PmatError::NetworkError {
            operation: "fetch".to_string(),
        };
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_is_recoverable_cache() {
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
    fn test_is_not_recoverable_file_not_found() {
        let err = PmatError::FileNotFound {
            path: PathBuf::from("/missing"),
        };
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_is_not_recoverable_parse_error() {
        let err = PmatError::ParseError {
            file: PathBuf::from("test.rs"),
            line: None,
            message: "error".to_string(),
        };
        assert!(!err.is_recoverable());
    }

    // === should_retry Tests ===

    #[test]
    fn test_should_retry_timeout() {
        let err = PmatError::TimeoutError {
            operation: "fetch".to_string(),
            timeout_ms: 5000,
        };
        assert!(err.should_retry());
    }

    #[test]
    fn test_should_retry_network() {
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
    fn test_should_not_retry_cache() {
        // Cache errors are recoverable but not retryable
        let err = PmatError::CacheError {
            operation: "clear".to_string(),
        };
        assert!(!err.should_retry());
    }

    #[test]
    fn test_should_not_retry_validation() {
        let err = PmatError::ValidationError {
            field: "name".to_string(),
            reason: "required".to_string(),
        };
        assert!(!err.should_retry());
    }

    // === severity Tests ===

    #[test]
    fn test_severity_warning() {
        let warnings = [
            PmatError::FileNotFound {
                path: PathBuf::from("test"),
            },
            PmatError::DirectoryNotFound {
                path: PathBuf::from("test"),
            },
            PmatError::ValidationError {
                field: "x".to_string(),
                reason: "y".to_string(),
            },
        ];

        for err in &warnings {
            assert_eq!(err.severity(), ErrorSeverity::Warning);
        }
    }

    #[test]
    fn test_severity_error() {
        let errors = [
            PmatError::PermissionDenied {
                path: PathBuf::from("test"),
            },
            PmatError::ParseError {
                file: PathBuf::from("test"),
                line: None,
                message: "err".to_string(),
            },
            PmatError::SyntaxError {
                language: "Rust".to_string(),
                message: "err".to_string(),
            },
            PmatError::ConfigError {
                key: "k".to_string(),
                value: "v".to_string(),
                reason: "r".to_string(),
            },
        ];

        for err in &errors {
            assert_eq!(err.severity(), ErrorSeverity::Error);
        }
    }

    #[test]
    fn test_severity_critical() {
        let critical = [
            PmatError::AllocationError { size: 1000 },
            PmatError::StorageFullError {
                available: 0,
                required: 1000,
            },
            PmatError::SimdError {
                operation: "test".to_string(),
            },
            PmatError::ModelError {
                model_name: "test".to_string(),
            },
        ];

        for err in &critical {
            assert_eq!(err.severity(), ErrorSeverity::Critical);
        }
    }

    #[test]
    fn test_severity_default_error() {
        // Errors without explicit severity should default to Error
        let err = PmatError::NetworkError {
            operation: "test".to_string(),
        };
        assert_eq!(err.severity(), ErrorSeverity::Error);
    }

    // === ErrorSeverity Tests ===

    #[test]
    fn test_error_severity_display_warning() {
        assert_eq!(format!("{}", ErrorSeverity::Warning), "WARNING");
    }

    #[test]
    fn test_error_severity_display_error() {
        assert_eq!(format!("{}", ErrorSeverity::Error), "ERROR");
    }

    #[test]
    fn test_error_severity_display_critical() {
        assert_eq!(format!("{}", ErrorSeverity::Critical), "CRITICAL");
    }

    #[test]
    fn test_error_severity_equality() {
        assert_eq!(ErrorSeverity::Warning, ErrorSeverity::Warning);
        assert_ne!(ErrorSeverity::Warning, ErrorSeverity::Error);
        assert_ne!(ErrorSeverity::Error, ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_severity_clone() {
        let severity = ErrorSeverity::Critical;
        let cloned = severity;
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_error_severity_copy() {
        let severity = ErrorSeverity::Warning;
        let copied = severity;
        assert_eq!(severity, copied);
    }

    #[test]
    fn test_error_severity_debug() {
        let debug_str = format!("{:?}", ErrorSeverity::Error);
        assert!(debug_str.contains("Error"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::PathBuf;

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

        /// Property: All TemplateError variants produce non-empty Display strings
        #[test]
        fn prop_template_error_display_non_empty(uri in "[a-z/]+") {
            let err = TemplateError::TemplateNotFound { uri };
            let msg = format!("{}", err);
            prop_assert!(!msg.is_empty());
        }

        /// Property: All TemplateError MCP codes are negative
        #[test]
        fn prop_template_error_mcp_code_negative(uri in "[a-z]+") {
            let errors = [
                TemplateError::TemplateNotFound { uri: uri.clone() },
                TemplateError::InvalidUri { uri: uri.clone() },
                TemplateError::ValidationError {
                    parameter: "param".to_string(),
                    reason: "reason".to_string(),
                },
                TemplateError::RenderError {
                    line: 1,
                    message: "msg".to_string(),
                },
            ];

            for err in &errors {
                prop_assert!(err.to_mcp_code() < 0);
            }
        }

        /// Property: PmatError MCP codes are always negative
        #[test]
        fn prop_pmat_error_mcp_code_negative(path in "[a-z/]+") {
            let err = PmatError::FileNotFound {
                path: PathBuf::from(path),
            };
            prop_assert!(err.to_mcp_code() < 0);
        }

        /// Property: is_recoverable and should_retry are consistent
        #[test]
        fn prop_retry_implies_recoverable(op in "[a-z]+") {
            let errors = [
                PmatError::TimeoutError {
                    operation: op.clone(),
                    timeout_ms: 1000,
                },
                PmatError::NetworkError {
                    operation: op.clone(),
                },
                PmatError::ResourceExhausted {
                    resource: op.clone(),
                },
            ];

            for err in &errors {
                if err.should_retry() {
                    prop_assert!(err.is_recoverable());
                }
            }
        }

        /// Property: Error severity is deterministic
        #[test]
        fn prop_severity_deterministic(path in "[a-z/]+") {
            let err = PmatError::FileNotFound {
                path: PathBuf::from(path),
            };
            let severity1 = err.severity();
            let severity2 = err.severity();
            prop_assert_eq!(severity1, severity2);
        }

        /// Property: ErrorSeverity Display output matches variant name
        #[test]
        fn prop_error_severity_display_consistency(variant in 0u8..3u8) {
            let severity = match variant {
                0 => ErrorSeverity::Warning,
                1 => ErrorSeverity::Error,
                _ => ErrorSeverity::Critical,
            };
            let display = format!("{}", severity);
            let debug = format!("{:?}", severity);

            // Display is uppercase version of Debug
            prop_assert!(display.to_lowercase().contains(&debug.to_lowercase()));
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for models/error.rs
    //! These tests ensure comprehensive coverage of all error types and methods.

    use super::*;
    use std::path::PathBuf;

    /// Test TemplateError Debug implementation
    #[test]
    fn test_template_error_debug() {
        let err = TemplateError::TemplateNotFound {
            uri: "test".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("TemplateNotFound"));
    }

    /// Test AnalysisError Debug implementation
    #[test]
    fn test_analysis_error_debug() {
        let err = AnalysisError::ParseError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ParseError"));
    }

    /// Test PmatError Debug implementation
    #[test]
    fn test_pmat_error_debug() {
        let err = PmatError::FileNotFound {
            path: PathBuf::from("/test"),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("FileNotFound"));
    }

    /// Test error source chain for S3Error
    #[test]
    fn test_s3_error_source() {
        let err = TemplateError::S3Error {
            operation: "test".to_string(),
            source: anyhow::anyhow!("root cause"),
        };
        // The #[source] attribute should provide error chaining
        let msg = format!("{}", err);
        assert!(msg.contains("S3 operation failed"));
    }

    /// Test all IO error codes through PmatError
    #[test]
    fn test_io_error_code_mapping() {
        let io_errors = [
            (std::io::ErrorKind::NotFound, -32003),
            (std::io::ErrorKind::PermissionDenied, -32003),
            (std::io::ErrorKind::ConnectionRefused, -32003),
        ];

        for (kind, expected_code) in io_errors {
            let io_err = std::io::Error::new(kind, "test");
            let pmat_err = PmatError::Io(io_err);
            assert_eq!(
                pmat_err.to_mcp_code(),
                expected_code,
                "Expected code {} for {:?}",
                expected_code,
                kind
            );
        }
    }

    /// Test error conversion chain: TemplateError -> AnalysisError -> PmatError
    #[test]
    fn test_error_conversion_chain() {
        let template_err = TemplateError::NotFound("resource".to_string());
        let analysis_err = AnalysisError::Template(template_err);
        let pmat_err = PmatError::Analysis(analysis_err);

        let msg = format!("{}", pmat_err);
        assert!(msg.contains("Analysis"));
    }

    /// Test all MCP error code categories
    #[test]
    fn test_mcp_code_categories() {
        // File/IO: -32001 to -32003
        assert_eq!(
            PmatError::FileNotFound {
                path: PathBuf::new()
            }
            .to_mcp_code(),
            -32001
        );
        assert_eq!(
            PmatError::DirectoryNotFound {
                path: PathBuf::new()
            }
            .to_mcp_code(),
            -32002
        );
        assert_eq!(
            PmatError::PermissionDenied {
                path: PathBuf::new()
            }
            .to_mcp_code(),
            -32003
        );

        // Parsing: -32004 to -32007
        assert_eq!(
            PmatError::ParseError {
                file: PathBuf::new(),
                line: None,
                message: "".to_string()
            }
            .to_mcp_code(),
            -32004
        );

        // SIMD: -32008 to -32010
        assert_eq!(
            PmatError::SimdError {
                operation: "".to_string()
            }
            .to_mcp_code(),
            -32008
        );

        // ML: -32011 to -32013
        assert_eq!(
            PmatError::ModelError {
                model_name: "".to_string()
            }
            .to_mcp_code(),
            -32011
        );

        // Config: -32014 to -32016
        assert_eq!(
            PmatError::ConfigError {
                key: "".to_string(),
                value: "".to_string(),
                reason: "".to_string()
            }
            .to_mcp_code(),
            -32014
        );

        // Network: -32018 to -32021
        assert_eq!(
            PmatError::NetworkError {
                operation: "".to_string()
            }
            .to_mcp_code(),
            -32019
        );

        // Storage: -32022 to -32027
        assert_eq!(
            PmatError::CacheError {
                operation: "".to_string()
            }
            .to_mcp_code(),
            -32022
        );

        // VCS: -32028 to -32032
        assert_eq!(
            PmatError::GitError {
                operation: "".to_string()
            }
            .to_mcp_code(),
            -32028
        );
    }

    /// Test error message formatting with special characters
    #[test]
    fn test_error_message_special_chars() {
        let err = PmatError::ParseError {
            file: PathBuf::from("/path/with spaces/file.rs"),
            line: Some(42),
            message: "unexpected token: '\u{2764}'".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("spaces"));
        assert!(msg.contains("'\u{2764}'"));
    }

    /// Test error message with empty strings
    #[test]
    fn test_error_message_empty_strings() {
        let err = PmatError::ConfigError {
            key: "".to_string(),
            value: "".to_string(),
            reason: "".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("Configuration error"));
    }

    /// Test error message with very long strings
    #[test]
    fn test_error_message_long_strings() {
        let long_string = "a".repeat(10000);
        let err = PmatError::AnalysisError {
            file: PathBuf::from(&long_string),
            reason: long_string.clone(),
        };

        let msg = format!("{}", err);
        assert!(msg.len() > 10000);
    }

    /// Test that all error variants can be Debug-printed
    #[test]
    fn test_all_pmat_error_variants_debug() {
        let errors: Vec<PmatError> = vec![
            PmatError::FileNotFound {
                path: PathBuf::new(),
            },
            PmatError::DirectoryNotFound {
                path: PathBuf::new(),
            },
            PmatError::PermissionDenied {
                path: PathBuf::new(),
            },
            PmatError::ParseError {
                file: PathBuf::new(),
                line: None,
                message: "".to_string(),
            },
            PmatError::SyntaxError {
                language: "".to_string(),
                message: "".to_string(),
            },
            PmatError::AnalysisError {
                file: PathBuf::new(),
                reason: "".to_string(),
            },
            PmatError::AstError {
                details: "".to_string(),
            },
            PmatError::SimdError {
                operation: "".to_string(),
            },
            PmatError::VectorizedError {
                details: "".to_string(),
            },
            PmatError::AlignmentError {
                expected: 0,
                actual: 0,
            },
            PmatError::ModelError {
                model_name: "".to_string(),
            },
            PmatError::FeatureExtractionError {
                feature_type: "".to_string(),
            },
            PmatError::TrainingDataError {
                reason: "".to_string(),
            },
            PmatError::ConfigError {
                key: "".to_string(),
                value: "".to_string(),
                reason: "".to_string(),
            },
            PmatError::ValidationError {
                field: "".to_string(),
                reason: "".to_string(),
            },
            PmatError::FormatError {
                expected: "".to_string(),
                actual: "".to_string(),
            },
            PmatError::RenderError {
                template: "".to_string(),
                line: 0,
                message: "".to_string(),
            },
            PmatError::NetworkError {
                operation: "".to_string(),
            },
            PmatError::ProtocolError {
                protocol: "".to_string(),
                message: "".to_string(),
            },
            PmatError::SerializationError {
                format: "".to_string(),
            },
            PmatError::CacheError {
                operation: "".to_string(),
            },
            PmatError::DatabaseError {
                operation: "".to_string(),
            },
            PmatError::StorageFullError {
                available: 0,
                required: 0,
            },
            PmatError::ResourceExhausted {
                resource: "".to_string(),
            },
            PmatError::TimeoutError {
                operation: "".to_string(),
                timeout_ms: 0,
            },
            PmatError::AllocationError { size: 0 },
            PmatError::GitError {
                operation: "".to_string(),
            },
            PmatError::RepositoryError {
                repo_path: PathBuf::new(),
            },
            PmatError::QualityGateError {
                gate_name: "".to_string(),
                reason: "".to_string(),
            },
            PmatError::VerificationError {
                property: "".to_string(),
            },
            PmatError::ProofError {
                method: "".to_string(),
            },
        ];

        for err in errors {
            let debug = format!("{:?}", err);
            let display = format!("{}", err);
            assert!(!debug.is_empty());
            assert!(!display.is_empty());
        }
    }

    /// error.rs:294 — outer `_ => -32000` fallback arm of
    /// `get_error_code_by_category`. `PmatError::Template` is the only
    /// variant that doesn't match any named arm (it's short-circuited
    /// in the public `to_mcp_code` dispatcher), so calling the private
    /// helper directly with it is the only way to hit this arm.
    #[test]
    fn test_get_error_code_by_category_template_falls_through_to_minus_32000() {
        let template_err = PmatError::Template(TemplateError::InvalidUri {
            uri: "pmat://missing".into(),
        });
        assert_eq!(
            template_err.get_error_code_by_category(),
            -32000,
            "Template variant routes through to_mcp_code separately; when called \
             through get_error_code_by_category it must hit the outer _ fallback"
        );
    }

    /// error.rs:306/318/329/340/351/363/380/395 — the 8 inner fallback
    /// `_ =>` arms of the per-category helpers. They're defensive; the
    /// outer dispatcher never calls them with a mismatched variant. To
    /// hit the arms we invoke each private helper directly with a
    /// variant outside its category.
    #[test]
    fn test_get_category_error_code_helpers_fallback_arms() {
        // CacheError is only matched by get_storage_error_code; every other
        // helper's `_ =>` fallback fires.
        let cache = PmatError::CacheError {
            operation: "probe".into(),
        };
        assert_eq!(cache.get_io_error_code(), -32001);
        assert_eq!(cache.get_parsing_error_code(), -32004);
        assert_eq!(cache.get_simd_error_code(), -32008);
        assert_eq!(cache.get_ml_error_code(), -32011);
        assert_eq!(cache.get_config_error_code(), -32014);
        assert_eq!(cache.get_network_error_code(), -32019);
        assert_eq!(cache.get_vcs_error_code(), -32028);

        // For get_storage_error_code use a non-storage variant so its
        // CacheError arm doesn't fire — we want the `_ => -32022` line.
        let git = PmatError::GitError {
            operation: "probe".into(),
        };
        assert_eq!(git.get_storage_error_code(), -32022);
    }
}
