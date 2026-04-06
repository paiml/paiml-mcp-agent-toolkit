#![cfg_attr(coverage_nightly, coverage(off))]
//! Correlation Engine
//!
//! Creates bidirectional mappings between source locations and WASM offsets.
//! Uses DWARF as primary source, source maps as fallback.
//!
//! Implements DWASM-010: Source-to-WASM correlation with:
//! - Bidirectional mapping (source <-> WASM)
//! - DWARF as primary source
//! - Source maps as fallback
//! - Confidence scoring for each mapping

use crate::services::deep_wasm::{
    DeepWasmResult, DwarfDebugEntry, Location, SourceMapEntry, SourceToWasmMapping,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Estimate WASM function index from DWARF offset
/// This is a heuristic until we have full WASM module analysis
fn estimate_function_index(die_offset: u64) -> u32 {
    // Simple heuristic: assume functions are roughly 100 bytes apart in DWARF
    // This will be replaced with accurate mapping from WASM module analysis
    (die_offset / 100) as u32
}

/// Correlation engine for multi-layer mapping
pub struct CorrelationEngine {
    /// Confidence threshold for mappings (0.0-1.0)
    confidence_threshold: f64,
}

impl CorrelationEngine {
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7, // 70% confidence minimum
        }
    }

    /// Create correlation engine with custom confidence threshold
    pub fn with_confidence_threshold(threshold: f64) -> Self {
        debug_assert!(threshold >= 0.0, "threshold must be non-negative");
        Self {
            confidence_threshold: threshold.clamp(0.0, 1.0),
        }
    }
}

impl Default for CorrelationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// --- Submodule includes (semantic split) ---
include!("correlation_engine_correlate.rs");
include!("correlation_engine_line_programs.rs");
include!("correlation_engine_lookups.rs");
include!("correlation_engine_unit_tests.rs");
