#![cfg_attr(coverage_nightly, coverage(off))]
//! Mutation testing handler (Sprint 61)
//!
//! Exposes PMAT's AST-based mutation testing infrastructure via CLI command.
//!
//! ## Module structure
//!
//! - `mutate_handler.rs`       — Main `handle()` entry point and execution helpers
//! - `mutate_output.rs`        — Output formatting (JSON, Markdown, text) and types
//! - `mutate_cargo_backend.rs` — cargo-mutants backend integration (Sprint 70)
//! - `mutate_tests.rs`         — Unit tests

use crate::cli::commands::MutateArgs;
use crate::services::mutation::engine::{MutationConfig, MutationEngine, MutationStrategy};
use crate::services::mutation::types::{MutationResult, MutationScore, SourceLocation};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

include!("mutate_handler.rs");
include!("mutate_output.rs");
include!("mutate_cargo_backend.rs");
include!("mutate_tests.rs");
