//! WebAssembly parsing and analysis error types
//!
//! This module defines error types specific to WebAssembly processing,
//! providing detailed error information for debugging and recovery.
use std::fmt;
use thiserror::Error;

/// `Result` type alias for WebAssembly operations
pub type WasmResult<T> = Result<T, WasmError>;

/// WebAssembly processing errors with detailed context
#[derive(Error, Debug)]
pub enum WasmError {
    /// Parser initialization or configuration error
    #[error("Parser error: {0}")]
    ParserError(String),

    /// Invalid WebAssembly module structure
    #[error("Invalid WASM module: {0}")]
    InvalidModule(String),

    /// File size exceeds configured limits
    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    FileTooLarge { size: usize, limit: usize },

    /// Parsing timeout exceeded
    #[error("Parsing timeout exceeded after {elapsed_secs} seconds")]
    Timeout { elapsed_secs: u64 },

    /// Memory limit exceeded during parsing
    #[error("Memory limit exceeded: {used} bytes exceeds limit of {limit} bytes")]
    MemoryExceeded { used: usize, limit: usize },

    /// Recursion depth limit exceeded
    #[error("Maximum recursion depth {depth} exceeded at {location}")]
    RecursionLimit { depth: usize, location: String },

    /// Node count limit exceeded
    #[error("Maximum node count {count} exceeded")]
    NodeLimitExceeded { count: usize },

    /// Invalid magic number for WASM binary
    #[error("Invalid WASM magic number: expected 0x0061_736D, got 0x{actual:08X}")]
    InvalidMagicNumber { actual: u32 },

    /// Unsupported WASM version
    #[error("Unsupported WASM version: {version}")]
    UnsupportedVersion { version: u32 },

    /// Security validation failure
    #[error("Security validation failed: {reason}")]
    SecurityViolation { reason: String },

    /// Language detection failure
    #[error("Failed to detect WebAssembly variant for file: {path}")]
    DetectionFailed { path: String },

    /// Feature not supported by parser
    #[error("Feature not supported: {feature}")]
    UnsupportedFeature { feature: String },

    /// IO error wrapper
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic parsing error with context
    #[error("Parse error at {location}: {message}")]
    ParseError { location: String, message: String },

    /// `UTF-8` decoding error
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Other errors wrapped from anyhow
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl WasmError {
    ///
    ///
    /// # Panics
    ///
    /// May panic on out-of-bounds array/slice access
    /// Create a parser error with a message
//! WebAssembly parsing and analysis error types
//!
//! This module defines error types specific to WebAssembly processing,
//! providing detailed error information for debugging and recovery.
use std::fmt;
use thiserror::Error;

/// `Result` type alias for WebAssembly operations
pub type WasmResult<T> = Result<T, WasmError>;

/// WebAssembly processing errors with detailed context
#[derive(Error, Debug)]
pub enum WasmError {
    /// Parser initialization or configuration error
    #[error("Parser error: {0}")]
    ParserError(String),

    /// Invalid WebAssembly module structure
    #[error("Invalid WASM module: {0}")]
    InvalidModule(String),

    /// File size exceeds configured limits
    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    FileTooLarge { size: usize, limit: usize },

    /// Parsing timeout exceeded
    #[error("Parsing timeout exceeded after {elapsed_secs} seconds")]
    Timeout { elapsed_secs: u64 },

    /// Memory limit exceeded during parsing
    #[error("Memory limit exceeded: {used} bytes exceeds limit of {limit} bytes")]
    MemoryExceeded { used: usize, limit: usize },

    /// Recursion depth limit exceeded
    #[error("Maximum recursion depth {depth} exceeded at {location}")]
    RecursionLimit { depth: usize, location: String },

    /// Node count limit exceeded
    #[error("Maximum node count {count} exceeded")]
    NodeLimitExceeded { count: usize },

    /// Invalid magic number for WASM binary
    #[error("Invalid WASM magic number: expected 0x0061_736D, got 0x{actual:08X}")]
    InvalidMagicNumber { actual: u32 },

    /// Unsupported WASM version
    #[error("Unsupported WASM version: {version}")]
    UnsupportedVersion { version: u32 },

    /// Security validation failure
    #[error("Security validation failed: {reason}")]
    SecurityViolation { reason: String },

    /// Language detection failure
    #[error("Failed to detect WebAssembly variant for file: {path}")]
    DetectionFailed { path: String },

    /// Feature not supported by parser
    #[error("Feature not supported: {feature}")]
    UnsupportedFeature { feature: String },

    /// IO error wrapper
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic parsing error with context
    #[error("Parse error at {location}: {message}")]
    ParseError { location: String, message: String },

    /// `UTF-8` decoding error
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Other errors wrapped from anyhow
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
