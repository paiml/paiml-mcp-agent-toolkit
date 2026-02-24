#![cfg_attr(coverage_nightly, coverage(off))]

// AST-Aware Code Chunker
// PMAT-SEARCH-001: Extract semantic units (functions, classes, modules) from code
//
// GREEN Phase: Full implementation using tree-sitter AST parsers

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tree_sitter::{Node, Parser, Tree};

// --- Types: Language, ChunkType, CodeChunk, chunk_code ---
include!("chunker_types.rs");

// --- Shared helpers: doc comments, node mapping, push_chunk, checksum ---
include!("chunker_helpers.rs");

// --- Rust chunking ---
include!("chunker_rust.rs");

// --- TypeScript chunking ---
include!("chunker_typescript.rs");

// --- Python chunking ---
include!("chunker_python.rs");

// --- C and C++ chunking ---
include!("chunker_c_cpp.rs");

// --- Go and Lua chunking ---
include!("chunker_go_lua.rs");

// --- Text chunking with trueno-rag ---
include!("chunker_text.rs");

// --- File extract types and functions ---
include!("chunker_extract.rs");

// Tests extracted to chunker_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "chunker_tests.rs"]
mod tests;
