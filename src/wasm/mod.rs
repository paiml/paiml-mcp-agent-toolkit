#![cfg_attr(coverage_nightly, coverage(off))]
//! WebAssembly analysis and quality assurance module
//!
//! Implements the WebAssembly Integration Specification v3 for
//! pragmatic zero-defect validation through incremental verification.

pub mod analyzer;
pub mod baseline;
pub mod hardware;
pub mod profiler;
pub mod security;
pub mod verifier;

pub use analyzer::{Analysis, WasmAnalyzer};
pub use baseline::{QualityAssessment, QualityBaseline, Violation};
pub use hardware::{CacheClass, CoreClass, HardwareClass};
pub use profiler::{AsyncProfiler, ShadowStack};
pub use security::{PatternDetector, VulnerabilityMatch, VulnerabilityPattern};
pub use verifier::{IncrementalVerifier, VerificationResult};

use anyhow::Result;
use serde::{Deserialize, Serialize};

// --- Type definitions (ProfilingReport, InstructionMix, HotFunction, etc.) ---
include!("types.rs");

// --- Public API functions (analyze_wasm_module, verify_wasm_safety, etc.) ---
include!("api.rs");

// --- Unit tests and property tests ---
include!("tests_unit.rs");

// --- Coverage tests: WASM binary fixtures and module-level API tests ---
include!("tests_coverage_api.rs");

// --- Coverage tests: ProfilingReport and InstructionMix struct tests ---
include!("tests_coverage_report.rs");

// --- Coverage tests: HotFunction, MemoryProfile, GrowthEvent, re-exports, edge cases ---
include!("tests_coverage_types.rs");
