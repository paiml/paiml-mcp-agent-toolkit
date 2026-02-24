#![cfg_attr(coverage_nightly, coverage(off))]
//! Polyglot AST module for cross-language analysis
//!
//! This module provides a unified representation of AST nodes across different
//! programming languages, enabling cross-language analysis and dependency tracking.
//! The polyglot AST framework is designed to map language-specific AST nodes into
//! a common representation that preserves semantic meaning while abstracting away
//! language-specific details.
//!
//! # Architecture
//!
//! The polyglot AST system consists of three main components:
//!
//! 1. **UnifiedNode**: A language-agnostic representation of code elements
//! 2. **LanguageMapper**: Translates language-specific ASTs to UnifiedNodes
//! 3. **CrossLanguageDependencies**: Tracks relationships between nodes in different languages
//!
//! # Example
//!
//! ```rust,no_run
//! use crate::ast::polyglot::{UnifiedNode, LanguageMapper, JavaMapper, TypeScriptMapper};
//! use std::path::Path;
//!
//! // Create language mappers
//! let java_mapper = JavaMapper::new();
//! let ts_mapper = TypeScriptMapper::new();
//!
//! // Analyze files from different languages
//! let java_file = Path::new("src/main/java/com/example/Model.java");
//! let ts_file = Path::new("src/frontend/models/Model.ts");
//!
//! // Map to unified representation
//! let java_nodes = java_mapper.map_file(java_file).await?;
//! let ts_nodes = ts_mapper.map_file(ts_file).await?;
//!
//! // Find relationships between nodes
//! let dependencies = CrossLanguageDependencies::detect(&java_nodes, &ts_nodes);
//! ```

use crate::services::context::AstItem;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod cross_language_dependencies;
pub mod language_mapper;
pub mod language_mapper_factory;
pub mod unified_node;
pub mod utils;

pub use cross_language_dependencies::CrossLanguageDependencies;
pub use language_mapper::{
    BaseLanguageMapper, CSharpMapper, JavaMapper, JavaScriptMapper, KotlinMapper, LanguageMapper,
    LanguageMapperFactory as LMFactory, RubyMapper, ScalaMapper, TypeScriptMapper,
};
pub use language_mapper_factory::{LanguageMapperFactory, StubMapper};
pub use unified_node::UnifiedNode;
pub use utils::PolyglotPathValidator;

// --- Language enum and impl ---
include!("language_enum.rs");

// --- NodeKind enum and impl ---
include!("node_kind.rs");

// --- PolyglotConfig struct ---
include!("polyglot_config.rs");

// --- Unit tests ---
include!("mod_tests.rs");

// --- Coverage tests (part 1: language enum + node_kind from_ast_item) ---
include!("mod_coverage_tests.rs");

// --- Coverage tests (part 2: node_kind conversions, config, serialization, edge cases) ---
include!("mod_coverage_tests_part2.rs");
