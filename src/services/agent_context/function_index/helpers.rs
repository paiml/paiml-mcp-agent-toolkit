#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::*;
use crate::services::semantic::{CodeChunk, Language};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

// --- Identifier & keyword utilities ---
include!("helpers_identifiers.rs");

// --- Index building: workspace parsing, corpus, indices, SHA256 ---
include!("helpers_index_building.rs");

// --- Annotations: churn, clones, diversity, faults, name frequency ---
include!("helpers_annotations.rs");

// --- Call graph construction, PageRank, graph metrics ---
include!("helpers_call_graph.rs");

// --- Quality metrics: complexity, SATD, Big-O, TDG, doc comments, filesystem helpers ---
include!("helpers_quality_metrics.rs");

// --- compile_commands.json integration for C/C++ include paths ---
include!("helpers_compile_commands.rs");
