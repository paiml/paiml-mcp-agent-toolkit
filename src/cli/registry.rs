#![cfg_attr(coverage_nightly, coverage(off))]
//! Command Registry - Single Source of Truth for CLI/MCP/Help
//!
//! This module provides unified command metadata that is used to generate:
//! - `--help` text (dynamic, always accurate)
//! - MCP tool schemas (JSON Schema)
//! - Documentation (README examples)
//! - Semantic help search (RAG-powered)
//!
//! # Architecture (Toyota Way - Jidoka)
//!
//! All command metadata flows from a single source:
//! ```text
//! CommandRegistry (source of truth)
//!        │
//!        ├─▶ HelpGenerator (--help text)
//!        ├─▶ McpSchemaGenerator (MCP tools/list)
//!        └─▶ DocsGenerator (README.md)
//! ```
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118
//! - Toyota Way: Jidoka (built-in quality), Poka-yoke (error-proofing)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Type definitions ────────────────────────────────────────────────────────
include!("registry_types.rs");

// ─── CommandRegistry + CommandMetadata impl blocks ───────────────────────────
include!("registry_impl.rs");

// ─── CommandMetadataBuilder ──────────────────────────────────────────────────
include!("registry_builder.rs");

// ─── RegistryError enum + Display/Error impls ────────────────────────────────
include!("registry_error.rs");

// ─── Tests ───────────────────────────────────────────────────────────────────
include!("registry_tests.rs");
