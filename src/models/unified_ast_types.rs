// Unified AST representation for cross-language code analysis
//
// This module provides a language-agnostic AST representation that enables
// consistent analysis across Rust, TypeScript/JavaScript, and Python codebases.
// Enhanced with formal verification metadata for proof-enriched ASTs.
//
// Split into submodules for file health (CB-040):
// - unified_ast_node_kinds.rs: Language enum, NodeFlags, AstKind enums
// - unified_ast_proof.rs: ProofAnnotation system
// - unified_ast_location.rs: Location, Span, BytePos, QualifiedName
// - unified_ast_node.rs: RelativeLocation, ProofMap, NodeMetadata, UnifiedAstNode
// - unified_ast_dag.rs: ColumnStore, AstDag, LanguageParsers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use uuid::Uuid;

include!("unified_ast_node_kinds.rs");
include!("unified_ast_proof.rs");
include!("unified_ast_location.rs");
include!("unified_ast_node.rs");
include!("unified_ast_dag.rs");
