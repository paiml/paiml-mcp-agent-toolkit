//! WebAssembly language support implementation
//!
//! This module provides comprehensive WebAssembly analysis capabilities including:
//! - AssemblyScript (.as, .ts with AS context)
//! - WebAssembly Text Format (.wat)
//! - WebAssembly Binary Format (.wasm)
//!
//! Follows Toyota Way principles with extreme quality standards.
pub mod assemblyscript;
pub mod binary;
pub mod complexity;
pub mod error;
pub mod language_detection;
pub mod memory_pool;
pub mod parallel;
pub mod security;
pub mod traits;
pub mod types;
pub mod wat;

pub use self::assemblyscript::AssemblyScriptParser;
pub use self::binary::WasmBinaryAnalyzer;
pub use self::error::{WasmError, WasmResult};
pub use self::language_detection::WasmLanguageDetector;
pub use self::traits::{WasmAnalysisCapabilities, WasmAwareParser};
pub use self::types::{MemoryOpStats, WasmComplexity, WasmMetrics, WebAssemblyVariant};
pub use self::wat::WatParser;

// Re-export commonly used items for convenience
pub use self::complexity::WasmComplexityAnalyzer;
pub use self::security::WasmSecurityValidator;
