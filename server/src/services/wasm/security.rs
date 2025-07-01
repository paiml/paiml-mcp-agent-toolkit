//! WebAssembly security validation
//!
//! This module provides security validation for WebAssembly modules to prevent
//! malicious patterns and resource exhaustion attacks.
use super::error::{WasmError, WasmResult};
use super::types::Severity;
use serde::{Deserialize, Serialize};

/// Configuration for security validation
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum allowed file size in bytes
    pub max_file_size: usize,

    /// Maximum number of functions allowed
    pub max_function_count: usize,

    /// Maximum memory pages (64KB each)
    pub max_memory_pages: u32,

    /// Maximum table size
    pub max_table_size: u32,

    /// Maximum number of imports
    pub max_import_count: usize,

    /// Maximum recursion depth in call graph
    pub max_recursion_depth: usize,

    /// Enable strict mode (fail on any suspicious pattern)
    pub strict_mode: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_file_size: 50 * 1_024 * 1_024, // 50MB
            max_function_count: 10_000,
            max_memory_pages: 65_536, // 4GB max
            max_table_size: 10_000,
            max_import_count: 1_000,
            max_recursion_depth: 100,
            strict_mode: false,
        }
    }
}

/// Security issue found during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Issue severity
    pub severity: Severity,

    /// Issue category
    pub category: SecurityCategory,

    /// Human-readable description
    pub description: String,

    /// `Location` in the module (if applicable)
    pub location: Option<String>,

    /// Suggested mitigation
    pub mitigation: Option<String>,
}

/// Categories of security issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityCategory {
    /// Resource exhaustion risks
    ResourceExhaustion,

    /// Suspicious instruction patterns
    SuspiciousPattern,

    /// Memory safety issues
    MemorySafety,

    /// Import/export violations
    InterfaceViolation,

    /// Malformed module structure
    MalformedModule,
}

impl std::fmt::Display for SecurityCategory {
///
/// Returns an error if the operation fails
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityCategory::ResourceExhaustion => write!(f, "Resource Exhaustion"),
            SecurityCategory::SuspiciousPattern => write!(f, "Suspicious Pattern"),
            SecurityCategory::MemorySafety => write!(f, "Memory Safety"),
            SecurityCategory::InterfaceViolation => write!(f, "Interface Violation"),
            SecurityCategory::MalformedModule => write!(f, "Malformed Module"),
        }
    }
}

/// WebAssembly security validator
pub struct WasmSecurityValidator {
    config: SecurityConfig,
}

impl WasmSecurityValidator {
    ///
    ///
    /// # Panics
    ///
    /// May panic on out-of-bounds array/slice access
    /// Create a new validator with default config
//! WebAssembly security validation
//!
//! This module provides security validation for WebAssembly modules to prevent
//! malicious patterns and resource exhaustion attacks.
use super::error::{WasmError, WasmResult};
use super::types::Severity;
use serde::{Deserialize, Serialize};

/// Configuration for security validation
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum allowed file size in bytes
    pub max_file_size: usize,

    /// Maximum number of functions allowed
    pub max_function_count: usize,

    /// Maximum memory pages (64KB each)
    pub max_memory_pages: u32,

    /// Maximum table size
    pub max_table_size: u32,

    /// Maximum number of imports
    pub max_import_count: usize,

    /// Maximum recursion depth in call graph
    pub max_recursion_depth: usize,

    /// Enable strict mode (fail on any suspicious pattern)
    pub strict_mode: bool,
}
