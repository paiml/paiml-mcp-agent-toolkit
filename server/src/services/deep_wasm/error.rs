//! Error types for deep WASM inspection

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeepWasmError {
    #[error("WASM parsing error: {0}")]
    WasmParse(String),

    #[error("DWARF parsing error: {0}")]
    DwarfParse(String),

    #[error("Source map error: {0}")]
    SourceMap(String),

    #[error("Correlation error: {0}")]
    Correlation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid WASM module: {0}")]
    InvalidModule(String),

    #[error("Missing debug information")]
    MissingDebugInfo,

    #[error("Unsupported WASM version: {0}")]
    UnsupportedVersion(u32),

    #[error("Quality gate violation: {0}")]
    QualityGate(String),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

pub type DeepWasmResult<T> = Result<T, DeepWasmError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DeepWasmError::WasmParse("test error".to_string());
        assert!(err.to_string().contains("WASM parsing error"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: DeepWasmError = io_err.into();
        assert!(matches!(err, DeepWasmError::Io(_)));
    }
}
