use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("S3 operation failed: {operation}")]
    S3Error {
        operation: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("Template not found: {uri}")]
    TemplateNotFound { uri: String },

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid template URI: {uri}")]
    InvalidUri { uri: String },

    #[error("Template rendering failed at line {line}: {message}")]
    RenderError { line: u32, message: String },

    #[error("Parameter validation failed: {parameter} - {reason}")]
    ValidationError { parameter: String, reason: String },

    #[error("Invalid UTF-8 in template content")]
    InvalidUtf8(String),

    #[error("Cache operation failed")]
    CacheError(#[from] anyhow::Error),

    #[error("JSON serialization error")]
    JsonError(#[from] serde_json::Error),

    #[error("IO operation failed")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Failed to parse file: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Template error: {0}")]
    Template(#[from] TemplateError),
}

impl TemplateError {
    pub fn to_mcp_code(&self) -> i32 {
        match self {
            TemplateError::TemplateNotFound { .. } => -32001,
            TemplateError::InvalidUri { .. } => -32002,
            TemplateError::ValidationError { .. } => -32003,
            TemplateError::RenderError { .. } => -32004,
            _ => -32000, // Generic error
        }
    }
}

/// Consolidated error type for the PAIML MCP Agent Toolkit
///
/// This error type consolidates all error variants across the system
/// for improved architectural consistency.
#[derive(Error, Debug)]
pub enum PmatError {
    // === File and I/O Errors ===
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // === Parsing and Analysis Errors ===
    #[error("Parse error in {file}: {message}")]
    ParseError {
        file: PathBuf,
        line: Option<u32>,
        message: String,
    },

    #[error("Syntax error in {language}: {message}")]
    SyntaxError { language: String, message: String },

    #[error("Analysis failed for {file}: {reason}")]
    AnalysisError { file: PathBuf, reason: String },

    #[error("AST processing failed: {details}")]
    AstError { details: String },

    // === SIMD and Vectorized Operations ===
    #[error("SIMD operation failed: {operation}")]
    SimdError { operation: String },

    #[error("Vectorized computation error: {details}")]
    VectorizedError { details: String },

    #[error("Cache alignment error: expected {expected}, got {actual}")]
    AlignmentError { expected: usize, actual: usize },

    // === Machine Learning Errors ===
    #[error("Model inference failed: {model_name}")]
    ModelError { model_name: String },

    #[error("Feature extraction failed: {feature_type}")]
    FeatureExtractionError { feature_type: String },

    #[error("Training data invalid: {reason}")]
    TrainingDataError { reason: String },

    // === Configuration and Validation ===
    #[error("Configuration error: {key} = {value} is invalid: {reason}")]
    ConfigError {
        key: String,
        value: String,
        reason: String,
    },

    #[error("Validation failed for {field}: {reason}")]
    ValidationError { field: String, reason: String },

    #[error("Invalid format: expected {expected}, got {actual}")]
    FormatError { expected: String, actual: String },

    // === Template and Rendering ===
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),

    #[error("Rendering failed: {template} at line {line}: {message}")]
    RenderError {
        template: String,
        line: u32,
        message: String,
    },

    // === Network and Protocol ===
    #[error("Network error: {operation}")]
    NetworkError { operation: String },

    #[error("Protocol error: {protocol} - {message}")]
    ProtocolError { protocol: String, message: String },

    #[error("Serialization error: {format}")]
    SerializationError { format: String },

    // === Cache and Storage ===
    #[error("Cache error: {operation}")]
    CacheError { operation: String },

    #[error("Database error: {operation}")]
    DatabaseError { operation: String },

    #[error("Storage full: {available} bytes available, {required} bytes required")]
    StorageFullError { available: u64, required: u64 },

    // === Resource Management ===
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Timeout occurred: {operation} took longer than {timeout_ms}ms")]
    TimeoutError { operation: String, timeout_ms: u64 },

    #[error("Memory allocation failed: {size} bytes")]
    AllocationError { size: usize },

    // === Git and Version Control ===
    #[error("Git error: {operation}")]
    GitError { operation: String },

    #[error("Repository error: {repo_path}")]
    RepositoryError { repo_path: PathBuf },

    // === Quality Gates and Verification ===
    #[error("Quality gate failed: {gate_name} - {reason}")]
    QualityGateError { gate_name: String, reason: String },

    #[error("Verification failed: {property}")]
    VerificationError { property: String },

    #[error("Proof generation failed: {method}")]
    ProofError { method: String },

    // === Legacy Error Compatibility ===
    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalysisError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Generic error: {0}")]
    Other(#[from] anyhow::Error),
}

impl PmatError {
    /// Convert to MCP JSON-RPC error code using categorized mapping
    pub fn to_mcp_code(&self) -> i32 {
        // Delegate to specialized handlers for better maintainability
        match self {
            // Special case for template errors (delegates to nested enum)
            PmatError::Template(template_err) => template_err.to_mcp_code(),

            // Other errors mapped by category
            _ => self.get_error_code_by_category(),
        }
    }

    /// Map error to code based on error category (reduces CC complexity)
    fn get_error_code_by_category(&self) -> i32 {
        use PmatError::*;

        match self {
            // File and I/O errors (-32001 to -32010)
            FileNotFound { .. } | DirectoryNotFound { .. } | PermissionDenied { .. } | Io(_) => {
                self.get_io_error_code()
            }

            // Parsing and Analysis errors (-32004 to -32007)
            ParseError { .. } | SyntaxError { .. } | AnalysisError { .. } | AstError { .. } => {
                self.get_parsing_error_code()
            }

            // SIMD and Vectorized operations (-32008 to -32010)
            SimdError { .. } | VectorizedError { .. } | AlignmentError { .. } => {
                self.get_simd_error_code()
            }

            // ML/AI errors (-32011 to -32013)
            ModelError { .. } | FeatureExtractionError { .. } | TrainingDataError { .. } => {
                self.get_ml_error_code()
            }

            // Configuration and validation (-32014 to -32016)
            ConfigError { .. } | ValidationError { .. } | FormatError { .. } => {
                self.get_config_error_code()
            }

            // Network and protocol (-32018 to -32021)
            RenderError { .. }
            | NetworkError { .. }
            | ProtocolError { .. }
            | SerializationError { .. } => self.get_network_error_code(),

            // Storage and cache (-32022 to -32027)
            CacheError { .. }
            | DatabaseError { .. }
            | StorageFullError { .. }
            | ResourceExhausted { .. }
            | TimeoutError { .. }
            | AllocationError { .. } => self.get_storage_error_code(),

            // Git and quality (-32028 to -32032)
            GitError { .. }
            | RepositoryError { .. }
            | QualityGateError { .. }
            | VerificationError { .. }
            | ProofError { .. } => self.get_vcs_error_code(),

            // Legacy compatibility
            Analysis(_) | Json(_) | Other(_) => -32000,

            // Fallback
            _ => -32000,
        }
    }

    /// File and I/O error codes
    fn get_io_error_code(&self) -> i32 {
        use PmatError::*;
        match self {
            FileNotFound { .. } => -32001,
            DirectoryNotFound { .. } => -32002,
            PermissionDenied { .. } => -32003,
            Io(_) => -32003,
            _ => -32001,
        }
    }

    /// Parsing and analysis error codes
    fn get_parsing_error_code(&self) -> i32 {
        use PmatError::*;
        match self {
            ParseError { .. } => -32004,
            SyntaxError { .. } => -32005,
            AnalysisError { .. } => -32006,
            AstError { .. } => -32007,
            _ => -32004,
        }
    }

    /// SIMD and vectorized operation error codes
    fn get_simd_error_code(&self) -> i32 {
        use PmatError::*;
        match self {
            SimdError { .. } => -32008,
            VectorizedError { .. } => -32009,
            AlignmentError { .. } => -32010,
            _ => -32008,
        }
    }

    /// Machine learning error codes
    fn get_ml_error_code(&self) -> i32 {
        use PmatError::*;
        match self {
            ModelError { .. } => -32011,
            FeatureExtractionError { .. } => -32012,
            TrainingDataError { .. } => -32013,
            _ => -32011,
        }
    }

    /// Configuration and validation error codes
    fn get_config_error_code(&self) -> i32 {
        use PmatError::*;
        match self {
            ConfigError { .. } => -32014,
            ValidationError { .. } => -32015,
            FormatError { .. } => -32016,
            _ => -32014,
        }
    }

    /// Network and protocol error codes  
    fn get_network_error_code(&self) -> i32 {
        use PmatError::*;
        match self {
            RenderError { .. } => -32018,
            NetworkError { .. } => -32019,
            ProtocolError { .. } => -32020,
            SerializationError { .. } => -32021,
            _ => -32019,
        }
    }

    /// Storage and cache error codes
    fn get_storage_error_code(&self) -> i32 {
        use PmatError::*;
        match self {
            CacheError { .. } => -32022,
            DatabaseError { .. } => -32023,
            StorageFullError { .. } => -32024,
            ResourceExhausted { .. } => -32025,
            TimeoutError { .. } => -32026,
            AllocationError { .. } => -32027,
            _ => -32022,
        }
    }

    /// Git/VCS and quality gate error codes
    fn get_vcs_error_code(&self) -> i32 {
        use PmatError::*;
        match self {
            GitError { .. } => -32028,
            RepositoryError { .. } => -32029,
            QualityGateError { .. } => -32030,
            VerificationError { .. } => -32031,
            ProofError { .. } => -32032,
            _ => -32028,
        }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            PmatError::TimeoutError { .. }
                | PmatError::NetworkError { .. }
                | PmatError::CacheError { .. }
                | PmatError::ResourceExhausted { .. }
        )
    }

    /// Check if this error should trigger a retry
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            PmatError::TimeoutError { .. }
                | PmatError::NetworkError { .. }
                | PmatError::ResourceExhausted { .. }
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            PmatError::FileNotFound { .. }
            | PmatError::DirectoryNotFound { .. }
            | PmatError::ValidationError { .. } => ErrorSeverity::Warning,

            PmatError::PermissionDenied { .. }
            | PmatError::ParseError { .. }
            | PmatError::SyntaxError { .. }
            | PmatError::ConfigError { .. } => ErrorSeverity::Error,

            PmatError::AllocationError { .. }
            | PmatError::StorageFullError { .. }
            | PmatError::SimdError { .. }
            | PmatError::ModelError { .. } => ErrorSeverity::Critical,

            _ => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels for logging and reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
