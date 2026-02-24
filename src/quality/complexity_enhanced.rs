use std::collections::HashMap;
use syn::{self, visit::Visit, File, Item};

use super::complexity::ComplexityAnalyzer;

// Simple directed graph for CFG analysis (no petgraph dependency)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CfgNodeIndex(usize);

struct SimpleDiGraph<N, E> {
    nodes: Vec<N>,
    edges: Vec<(CfgNodeIndex, CfgNodeIndex, E)>,
}

pub struct ControlFlowGraph {
    graph: SimpleDiGraph<CfgNode, CfgEdge>,
    _entry: CfgNodeIndex,
    _exit: CfgNodeIndex,
}

#[derive(Debug, Clone)]
pub enum CfgNode {
    Entry,
    Exit,
    Statement(String),
    Condition(String),
    Branch(String),
}

#[derive(Debug, Clone)]
pub enum CfgEdge {
    Sequential,
    True,
    False,
    Jump,
}

struct CfgBuilder {
    graph: SimpleDiGraph<CfgNode, CfgEdge>,
    current: CfgNodeIndex,
    _exit: CfgNodeIndex,
    break_targets: Vec<CfgNodeIndex>,
    continue_targets: Vec<CfgNodeIndex>,
}

#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub max_nesting: u32,
    pub essential: u32,
}

#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    pub name: String,
    pub complexity: u32,
    pub lines: usize,
    pub parameters: usize,
}

#[derive(Debug, Clone)]
pub struct HalsteadMetrics {
    pub vocabulary: usize, // n = n1 + n2
    pub length: usize,     // N = N1 + N2
    pub volume: f64,       // V = N * log2(n)
    pub difficulty: f64,   // D = (n1/2) * (N2/n2)
    pub effort: f64,       // E = D * V
}

// SimpleDiGraph impl, ControlFlowGraph impl, CfgBuilder Visit impl
include!("complexity_enhanced_graph.rs");

// ComplexityAnalyzer impl, free functions (tokenize, is_operator, is_operand, maintainability index)
include!("complexity_enhanced_analysis.rs");

// Tests
include!("complexity_enhanced_tests.rs");
