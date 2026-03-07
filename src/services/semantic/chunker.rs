#![cfg_attr(coverage_nightly, coverage(off))]

// AST-Aware Code Chunker
// PMAT-SEARCH-001: Extract semantic units (functions, classes, modules) from code
//
// GREEN Phase: Full implementation using tree-sitter AST parsers

#[cfg(feature = "tree-sitter")]
use sha2::{Digest, Sha256};
#[cfg(feature = "tree-sitter")]
use std::collections::HashMap;
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Node, Parser, Tree};

// --- Types: Language, ChunkType, CodeChunk, chunk_code ---
include!("chunker_types.rs");

// --- Shared helpers: doc comments, node mapping, push_chunk, checksum ---
#[cfg(feature = "tree-sitter")]
include!("chunker_helpers.rs");

// --- Rust chunking ---
#[cfg(feature = "rust-ast")]
include!("chunker_rust.rs");

// --- TypeScript chunking ---
#[cfg(feature = "typescript-ast")]
include!("chunker_typescript.rs");

// --- Python chunking ---
#[cfg(feature = "python-ast")]
include!("chunker_python.rs");

// --- C and C++ chunking ---
#[cfg(feature = "c-ast")]
include!("chunker_c_cpp.rs");

// --- Go and Lua chunking ---
#[cfg(feature = "tree-sitter")]
include!("chunker_go_lua.rs");

// --- PTX assembly chunking (regex-based, no tree-sitter needed) ---
include!("chunker_ptx.rs");

// --- Text chunking with trueno-rag ---
include!("chunker_text.rs");

// --- File extract types and functions ---
#[cfg(feature = "tree-sitter")]
include!("chunker_extract.rs");

// --- Stub types when tree-sitter is disabled ---
#[cfg(not(feature = "tree-sitter"))]
include!("chunker_extract_stub.rs");

// Tests extracted to chunker_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "chunker_tests.rs"]
mod tests;
