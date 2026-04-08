#![allow(unused)]
//! Automated refactoring engine with state machine workflow.
//!
//! This module implements PMAT's intelligent refactoring system that follows
//! the Toyota Way principles of continuous improvement (Kaizen). The engine
//! uses a state machine to orchestrate the refactoring process through
//! analysis, planning, execution, and validation phases.
//!
//! # Architecture
//!
//! The refactoring engine supports three operation modes:
//! - **Server**: Low-latency mode for MCP/HTTP protocols
//! - **Interactive**: CLI mode with user confirmation steps
//! - **Batch**: High-throughput mode for CI/CD pipelines
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::refactor_engine::{UnifiedEngine, EngineMode};
//! use pmat::models::refactor::RefactorConfig;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create refactoring engine in interactive mode
//! let engine = UnifiedEngine::new_interactive(
//!     PathBuf::from("checkpoint.json"),
//!     Default::default()
//! )?;
//!
//! // Start refactoring session
//! let targets = vec![PathBuf::from("src/complex_module.rs")];
//! let config = RefactorConfig::default();
//!
//! engine.start_session(targets, config).await?;
//!
//! // Run refactoring workflow
//! while !engine.is_complete().await {
//!     engine.advance().await?;
//! }
//! # Ok(())
//! # }
//! ```ignore

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::models::refactor::{
    DefectPayload, RefactorConfig, RefactorStateMachine, RefactorType, State, Summary,
};
use crate::services::cache::unified_manager::UnifiedCacheManager;
use crate::services::unified_ast_engine::UnifiedAstEngine;
use crate::services::unified_refactor_analyzer::AnalyzerPool;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::RwLock;

// --- Type definitions ---
include!("refactor_engine_types.rs");

// --- RingBuffer implementation ---
include!("refactor_engine_ring_buffer.rs");

// --- UnifiedEngine core: constructor, run(), checkpoint, analysis ---
include!("refactor_engine_core.rs");

// --- UnifiedEngine mode runners: server, interactive, batch ---
include!("refactor_engine_modes.rs");

// --- Error types ---
include!("refactor_engine_errors.rs");

// Tests extracted to refactor_engine_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "refactor_engine_tests.rs"]
mod tests;
