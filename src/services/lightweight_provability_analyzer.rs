#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Lightweight provability analysis using abstract interpretation
//!
//! This module provides fast, incremental provability analysis using abstract
//! interpretation techniques instead of heavyweight SMT solvers. It analyzes
//! code properties like null safety, bounds checking, aliasing, and purity
//! to compute a provability score that feeds into the TDG calculation.
//!
//! # Approach
//!
//! The analyzer uses abstract domains to track program properties:
//! - **Nullability Analysis**: Tracks potential null/undefined values
//! - **Interval Analysis**: Bounds checking and numeric ranges
//! - **Alias Analysis**: Detects potential aliasing issues
//! - **Purity Analysis**: Identifies side-effect free functions
//!
//! # Abstract Interpretation
//!
//! Instead of precise symbolic execution, we use:
//! - **Lattice-based Domains**: Fast join/meet operations
//! - **Widening**: Ensures termination for loops
//! - **Incremental Analysis**: Only re-analyze changed functions
//! - **Caching**: Persistent results across analysis runs
//!
//! # Integration with TDG
//!
//! The provability score (0.0-1.0) is converted to a factor (0.0-5.0) for TDG:
//! - Higher provability → Lower TDG score (more maintainable code)
//! - Lower provability → Higher TDG score (harder to reason about)
//!
//! # Example
//!
//! ```rust
//! use pmat::services::lightweight_provability_analyzer::{
//!     LightweightProvabilityAnalyzer, FunctionId
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let analyzer = LightweightProvabilityAnalyzer::new();
//!
//! // Analyze specific functions incrementally
//! let changed_functions = vec![
//!     FunctionId {
//!         file_path: "src/main.rs".to_string(),
//!         function_name: "process_data".to_string(),
//!         line_number: 42,
//!     }
//! ];
//!
//! let summaries = analyzer.analyze_incrementally(&changed_functions).await;
//!
//! for summary in summaries {
//!     println!("Provability score: {:.2}", summary.provability_score);
//!     println!("Verified properties:");
//!     for prop in &summary.verified_properties {
//!         println!("  - {:?} (confidence: {:.0}%)",
//!                  prop.property_type, prop.confidence * 100.0);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Lightweight Provability Analyzer using Abstract Interpretation
/// Replaces heavyweight SMT solver approach with dataflow analysis
pub struct LightweightProvabilityAnalyzer {
    abstract_interpreter: AbstractInterpreter,
    proof_cache: Arc<DashMap<FunctionId, ProofSummary>>,
    current_version: u64,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct FunctionId {
    pub file_path: String,
    pub function_name: String,
    pub line_number: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofSummary {
    pub provability_score: f64,
    pub verified_properties: Vec<VerifiedProperty>,
    pub analysis_time_us: u128,
    pub version: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedProperty {
    pub property_type: PropertyType,
    pub confidence: f64,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PropertyType {
    NullSafety,
    BoundsCheck,
    NoAliasing,
    PureFunction,
    MemorySafety,
    ThreadSafety,
}

/// Abstract domain for property analysis
#[derive(Clone, Debug)]
pub struct PropertyDomain {
    pub nullability: NullabilityLattice,
    pub bounds: IntervalLattice,
    pub aliasing: AliasLattice,
    pub purity: PurityLattice,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NullabilityLattice {
    Top,       // Unknown
    NotNull,   // Definitely not null
    MaybeNull, // Possibly null
    Null,      // Definitely null
    Bottom,    // Unreachable
}

#[derive(Clone, Debug)]
pub struct IntervalLattice {
    pub lower: Option<i64>,
    pub upper: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AliasLattice {
    Top,       // Unknown aliasing
    NoAlias,   // No aliasing
    MayAlias,  // May have aliases
    MustAlias, // Definitely aliases
    Bottom,    // Unreachable
}

#[derive(Clone, Debug, PartialEq)]
pub enum PurityLattice {
    Top,         // Unknown purity
    Pure,        // Pure function
    ReadOnly,    // Only reads state
    WriteLocal,  // Only writes local state
    WriteGlobal, // Writes global state
    Bottom,      // Unreachable
}

pub struct AbstractInterpreter {
    analysis_depth: usize,
}

// Lattice join/meet/widen operations for PropertyDomain, NullabilityLattice,
// IntervalLattice, AliasLattice, and PurityLattice
include!("lightweight_provability_analyzer_lattice.rs");

// Core analysis: SourceFlags, LightweightProvabilityAnalyzer impl,
// AbstractInterpreter impl, Default impl
include!("lightweight_provability_analyzer_analysis.rs");

// Unit tests and property-based tests
include!("lightweight_provability_analyzer_tests.rs");
