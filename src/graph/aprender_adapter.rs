#![cfg_attr(coverage_nightly, coverage(off))]
// Adapter module: PMAT DependencyGraph → aprender::graph::Graph
// Complexity: All functions ≤ 10
// SATD: Zero tolerance
//
// Split into submodules:
//   aprender_adapter_conversion.rs  - Graph conversion + edge weight extraction
//   aprender_adapter_algorithms.rs  - Graph algorithm wrappers (SCC, cycles, paths, centrality)
//   aprender_adapter_tests.rs       - Unit tests

use super::types::{DependencyGraph, EdgeData, UndirectedGraph};
use aprender::graph::Graph as AprenderGraph;
use aprender::graph::GraphCentrality;

include!("aprender_adapter_conversion.rs");
include!("aprender_adapter_algorithms.rs");
include!("aprender_adapter_tests.rs");
