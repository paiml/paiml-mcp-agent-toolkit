#![cfg_attr(coverage_nightly, coverage(off))]
//! Dead code prover using reachability analysis and formal verification.
//!
//! This module implements sophisticated dead code detection that goes beyond
//! simple unused symbol detection. It performs whole-program reachability
//! analysis to prove that code is truly unreachable, considering various
//! entry points including FFI exports, dynamic dispatch, and reflection.
//!
//! # Analysis Approach
//!
//! 1. **Entry Point Discovery**: Identifies all program entry points
//!    - Main functions, tests, benchmarks
//!    - FFI exports (`#[no_mangle]`, `extern "C"`)
//!    - Dynamic dispatch targets (trait objects, function pointers)
//!    - Reflection and macro-generated code
//!
//! 2. **Reachability Propagation**: Traces execution paths from entry points
//!    - Direct function calls
//!    - Method invocations
//!    - Closure captures
//!    - Const evaluation paths
//!
//! 3. **Proof Generation**: Provides evidence for dead code claims
//!    - Call graph showing unreachability
//!    - Entry point analysis results
//!    - Confidence scores based on analysis completeness
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::dead_code_prover::ReachabilityAnalyzer;
//! use pmat::models::unified_ast::UnifiedAstNode;
//!
//! # fn example(ast: &UnifiedAstNode) {
//! let mut analyzer = ReachabilityAnalyzer::new();
//!
//! // Analyze AST to find entry points
//! analyzer.find_entry_points(ast);
//!
//! // Perform reachability analysis
//! let dead_code = analyzer.analyze_reachability(ast);
//!
//! // Generate proof for dead code
//! for item in &dead_code {
//!     println!("Dead code: {} (confidence: {}%)",
//!              item.name, item.confidence);
//!     println!("Reason: {}", item.reason);
//! }
//! # }
//! ```ignore

use crate::models::unified_ast::{AstKind, FunctionKind, UnifiedAstNode};
use crate::services::dead_code_analyzer::{DeadCodeItem, DeadCodeReport, DeadCodeType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Symbol identifier for cross-reference tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolId {
    pub file_path: String,
    pub function_name: String,
    pub line_number: usize,
}

/// Entry point types for reachability analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryPointType {
    Main,
    Test,
    Benchmark,
    Binary,
    FFIExport,
    DynamicDispatch,
    Reflection,
}

/// Reachability analyzer with FFI awareness
pub struct ReachabilityAnalyzer {
    /// Known entry points
    entry_points: HashSet<SymbolId>,
    /// Reachable symbols discovered during analysis
    #[allow(dead_code)]
    reachable: HashSet<SymbolId>,
    /// FFI exports that should never be marked as dead
    #[allow(dead_code)]
    ffi_exports: HashSet<SymbolId>,
    /// Dynamic dispatch targets
    #[allow(dead_code)]
    dynamic_targets: HashSet<SymbolId>,
}

/// FFI reference tracker for detecting externally visible symbols
pub struct FFIReferenceTracker {
    /// Symbols marked with #[`no_mangle`]
    no_mangle_symbols: HashSet<SymbolId>,
    /// Symbols with custom export names
    export_name_symbols: HashMap<SymbolId, String>,
    /// extern "C" functions
    extern_c_functions: HashSet<SymbolId>,
    /// WASM bindgen exports
    wasm_exports: HashSet<SymbolId>,
    /// `PyO3` exports
    python_exports: HashSet<SymbolId>,
}

/// Dynamic dispatch analyzer for trait objects and function pointers
pub struct DynamicDispatchAnalyzer {
    /// Trait implementations
    trait_impls: HashMap<String, Vec<SymbolId>>,
    /// Function pointer usage
    function_pointers: HashSet<SymbolId>,
    /// Trait object usage
    trait_objects: HashMap<String, Vec<SymbolId>>,
}

#[derive(Debug, Clone)]
pub enum Usage {
    TraitObject(String),
    FunctionPointer,
    VTable,
}

/// Dead code proof with confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeProof {
    pub item: SymbolId,
    pub proof_type: DeadCodeProofType,
    pub confidence: f64,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeadCodeProofType {
    ProvenDead,      // Definitely unreachable
    ProvenLive,      // Definitely reachable
    UnknownLiveness, // Cannot determine
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub description: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    NoReferences,
    FFIExport,
    DynamicDispatch,
    TestFunction,
    MainFunction,
    UnreachableCode,
}

/// Main dead code prover with FFI awareness
pub struct DeadCodeProver {
    #[allow(dead_code)]
    reachability: ReachabilityAnalyzer,
    ffi_tracker: FFIReferenceTracker,
    dynamic_analyzer: DynamicDispatchAnalyzer,
}

// Default impls
impl Default for ReachabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FFIReferenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DynamicDispatchAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DeadCodeProver {
    fn default() -> Self {
        Self::new()
    }
}

// Implementation split into include files
include!("dead_code_prover_reachability.rs");
include!("dead_code_prover_ffi.rs");
include!("dead_code_prover_analysis.rs");
include!("dead_code_prover_tests.rs");
