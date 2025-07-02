//! WebAssembly language support implementation
//!
//! This module provides comprehensive WebAssembly analysis capabilities including:
//! - AssemblyScript (.as, .ts with AS context)
//! - WebAssembly Text Format (.wat)
//! - WebAssembly Binary Format (.wasm)
//!
//! Follows Toyota Way principles with extreme quality standards.
pub mod assemblyscript;
#[cfg(test)]
pub mod assemblyscript_property_tests;
pub mod binary;
#[cfg(test)]
pub mod binary_property_tests;
pub mod complexity;
#[cfg(test)]
pub mod complexity_property_tests;
pub mod error;
pub mod language_detection;
pub mod memory_pool;
pub mod parallel;
pub mod security;
#[cfg(test)]
pub mod security_property_tests;
pub mod traits;
pub mod types;
pub mod wat;
#[cfg(test)]
pub mod wat_property_tests;
#[cfg(test)]
pub mod coverage_tests;
#[cfg(test)]
pub mod integration_tests;
#[cfg(test)]
pub mod tests;

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
