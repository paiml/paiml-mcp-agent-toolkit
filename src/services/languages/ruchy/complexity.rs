#![cfg_attr(coverage_nightly, coverage(off))]
//! Ruchy complexity analyzer: AST-based cyclomatic, cognitive, and Halstead metrics.

use std::collections::{HashMap, HashSet};

use crate::services::complexity::{
    ComplexityMetrics, FileComplexityMetrics, FunctionComplexity, HalsteadMetrics,
};

use super::types::{
    ActorInfo, DeadlockWarning, MessageFlow, RuchyActorAnalysis, RuchyAst, RuchyDeadCode,
    RuchyImport, RuchyToken, RuchyType,
};

/// Ruchy complexity analyzer
pub struct RuchyComplexityAnalyzer {
    pub(super) current_complexity: ComplexityMetrics,
    pub(super) nesting_level: u8,
    pub(super) functions: Vec<FunctionComplexity>,
    pub(super) classes: Vec<crate::services::complexity::ClassComplexity>,
    // Halstead metrics tracking
    pub(super) operators: HashSet<String>,
    pub(super) operands: HashSet<String>,
    pub(super) operator_count: u32,
    pub(super) operand_count: u32,
    // Dead code tracking
    pub(super) defined_functions: HashSet<String>,
    pub(super) called_functions: HashSet<String>,
    pub(super) defined_variables: HashSet<String>,
    pub(super) used_variables: HashSet<String>,
    // Type inference tracking
    pub(super) type_environment: HashMap<String, RuchyType>,
    // Import/dependency tracking
    pub(super) imports: Vec<RuchyImport>,
    pub(super) exports: HashSet<String>,
    // Actor analysis tracking
    pub(super) actors: Vec<ActorInfo>,
    pub(super) current_actor: Option<String>,
    pub(super) message_flows: Vec<MessageFlow>,
    pub(super) _spawn_calls: Vec<(String, String, u32)>,
}

impl Default for RuchyComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RuchyComplexityAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            current_complexity: ComplexityMetrics::default(),
            nesting_level: 0,
            functions: Vec::new(),
            classes: Vec::new(),
            operators: HashSet::new(),
            operands: HashSet::new(),
            operator_count: 0,
            operand_count: 0,
            defined_functions: HashSet::new(),
            called_functions: HashSet::new(),
            defined_variables: HashSet::<String>::new(),
            used_variables: HashSet::new(),
            type_environment: HashMap::new(),
            imports: Vec::new(),
            exports: HashSet::new(),
            actors: Vec::new(),
            current_actor: None,
            message_flows: Vec::new(),
            _spawn_calls: Vec::new(),
        }
    }
}

// Halstead metrics: reset, track, calculate
include!("complexity_halstead.rs");

// Accessors: dead code, type inference, imports/exports, actor analysis, pattern analysis
include!("complexity_accessors.rs");

// AST analysis: analyze_node, analyze_function, analyze_if, analyze_while,
// analyze_for, analyze_match, analyze_binary_op, analyze_block,
// analyze_import, analyze_export, analyze_actor, analyze_program
include!("complexity_analysis.rs");
