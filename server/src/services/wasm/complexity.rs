//! WebAssembly complexity analysis
//!
//! This module provides complexity analysis specifically tuned for WebAssembly,
//! including gas estimation and memory pressure calculation.
use super::types::{WasmComplexity, WasmOpcode};
use crate::models::unified_ast::{AstDag, AstKind, ExprKind, StmtKind};
use std::collections::HashMap;

/// Analyzes complexity of WebAssembly code with gas estimation
pub struct WasmComplexityAnalyzer {
    /// Opcode weights for gas estimation
    opcode_weights: HashMap<WasmOpcode, f32>,

    /// Memory operation cost model  
    memory_cost_model: MemoryCostModel,
}

/// Cost model for memory operations
#[derive(Debug, Clone)]
pub struct MemoryCostModel {
    pub load_cost: f32,
    pub store_cost: f32,
    pub grow_cost: f32,
    pub atomic_cost: f32,
    pub simd_cost: f32,
}

impl Default for MemoryCostModel {
    fn default() -> Self {
        Self {
            load_cost: 3.0,
            store_cost: 5.0,
            grow_cost: 100.0,
            atomic_cost: 10.0,
            simd_cost: 8.0,
        }
    }
}

impl WasmComplexityAnalyzer {
    ///
    ///
    /// # Panics
    ///
    /// May panic on out-of-bounds array/slice access
    /// Create a new analyzer with default weights
//! WebAssembly complexity analysis
//!
//! This module provides complexity analysis specifically tuned for WebAssembly,
//! including gas estimation and memory pressure calculation.
use super::types::{WasmComplexity, WasmOpcode};
use crate::models::unified_ast::{AstDag, AstKind, ExprKind, StmtKind};
use std::collections::HashMap;

/// Analyzes complexity of WebAssembly code with gas estimation
pub struct WasmComplexityAnalyzer {
    /// Opcode weights for gas estimation
    opcode_weights: HashMap<WasmOpcode, f32>,

    /// Memory operation cost model  
    memory_cost_model: MemoryCostModel,
}

/// Cost model for memory operations
#[derive(Debug, Clone)]
pub struct MemoryCostModel {
    pub load_cost: f32,
    pub store_cost: f32,
    pub grow_cost: f32,
    pub atomic_cost: f32,
    pub simd_cost: f32,
}
