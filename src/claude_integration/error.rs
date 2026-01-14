// Error handling types for Claude integration bridge
// Zero-cost error propagation using discriminated unions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Error codes are stable across versions for backward compatibility
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    // Transport errors (1000-1999)
    PipeBrokenPipe = 1001,
    FramingError = 1002,
    MessageTooLarge = 1003,

    // Bridge errors (2000-2999)
    InitializationTimeout = 2001,
    WorkerCrashed = 2002,
    PoolExhausted = 2003,

    // Claude API errors (3000-3999)
    RateLimited = 3001,
    QuotaExceeded = 3002,
    InvalidApiKey = 3003,

    // Application errors (4000-4999)
    ComplexityExceeded = 4001,
    SatdDetected = 4002,
    QualityGateFailed = 4003,
}

/// Source language for error tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SourceLang {
    #[default]
    Rust,
    TypeScript,
}

/// Type-safe error propagation across language boundary
/// No exceptions, no unwinding, zero-cost success path
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload", rename_all = "snake_case")]
pub enum BridgeResult<T> {
    Success(T),
    Error {
        code: u32,
        message: String,
        backtrace: Option<String>,
        source_lang: SourceLang,
    },
    Timeout {
        elapsed_ms: u64,
    },
    CircuitOpen {
        retry_after_ms: u64,
    },
}

impl<T> BridgeResult<T> {
    /// Zero-cost conversion for success path
    #[inline(always)]
    pub fn unwrap_or_propagate(self) -> Result<T, BridgeError> {
        match self {
            Self::Success(val) => Ok(val),
            Self::Error {
                code,
                message,
                backtrace,
                source_lang,
            } => Err(BridgeError {
                code: ErrorCode::from_u32(code).unwrap_or(ErrorCode::FramingError),
                message,
                source: None,
                backtrace: backtrace.map(|s| s.into()),
                context: Box::new(ErrorContext {
                    source_lang,
                    ..Default::default()
                }),
            }),
            Self::Timeout { elapsed_ms } => Err(BridgeError::timeout(elapsed_ms)),
            Self::CircuitOpen { retry_after_ms } => Err(BridgeError::circuit_open(retry_after_ms)),
        }
    }
}

/// Zero-cost error propagation using Result<T, E>
#[derive(Debug)]
pub struct BridgeError {
    pub code: ErrorCode,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub backtrace: Option<Box<str>>,
    pub context: Box<ErrorContext>,
}

/// Contextual information for debugging
#[derive(Debug, Default)]
pub struct ErrorContext {
    pub request_id: Option<String>,
    pub bridge_version: &'static str,
    pub source_lang: SourceLang,
    pub metrics: HashMap<String, f64>,
}

impl ErrorCode {
    pub fn from_u32(code: u32) -> Option<Self> {
        match code {
            1001 => Some(Self::PipeBrokenPipe),
            1002 => Some(Self::FramingError),
            1003 => Some(Self::MessageTooLarge),
            2001 => Some(Self::InitializationTimeout),
            2002 => Some(Self::WorkerCrashed),
            2003 => Some(Self::PoolExhausted),
            3001 => Some(Self::RateLimited),
            3002 => Some(Self::QuotaExceeded),
            3003 => Some(Self::InvalidApiKey),
            4001 => Some(Self::ComplexityExceeded),
            4002 => Some(Self::SatdDetected),
            4003 => Some(Self::QualityGateFailed),
            _ => None,
        }
    }
}

impl BridgeError {
    /// Construct error with full context capture
    #[track_caller]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
            backtrace: Some(
                std::backtrace::Backtrace::force_capture()
                    .to_string()
                    .into(),
            ),
            context: Box::new(ErrorContext {
                bridge_version: env!("CARGO_PKG_VERSION"),
                ..Default::default()
            }),
        }
    }

    /// Construct timeout error
    pub fn timeout(elapsed_ms: u64) -> Self {
        Self::new(
            ErrorCode::InitializationTimeout,
            format!("Operation timed out after {}ms", elapsed_ms),
        )
    }

    /// Construct circuit open error
    pub fn circuit_open(retry_after_ms: u64) -> Self {
        Self::new(
            ErrorCode::PoolExhausted,
            format!("Circuit open, retry after {}ms", retry_after_ms),
        )
    }

    /// Chain errors while preserving original context
    pub fn with_source(
        mut self,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Serialize for cross-language boundary
    pub fn to_bridge_result<T>(&self) -> BridgeResult<T> {
        BridgeResult::Error {
            code: self.code as u32,
            message: self.message.clone(),
            backtrace: self.backtrace.as_ref().map(|b| b.to_string()),
            source_lang: self.context.source_lang,
        }
    }
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}

impl std::error::Error for BridgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| &**e as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_conversion() {
        assert_eq!(ErrorCode::from_u32(1001), Some(ErrorCode::PipeBrokenPipe));
        assert_eq!(
            ErrorCode::from_u32(2001),
            Some(ErrorCode::InitializationTimeout)
        );
        assert_eq!(ErrorCode::from_u32(9999), None);
    }

    #[test]
    fn test_bridge_result_success() {
        let result: BridgeResult<i32> = BridgeResult::Success(42);
        assert!(matches!(result, BridgeResult::Success(42)));
    }

    #[test]
    fn test_bridge_error_creation() {
        let error = BridgeError::new(ErrorCode::FramingError, "test error");
        assert_eq!(error.code, ErrorCode::FramingError);
        assert_eq!(error.message, "test error");
    }
}
