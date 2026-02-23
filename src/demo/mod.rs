#![cfg_attr(coverage_nightly, coverage(off))]
//! Demo and reporting system for PMAT.
//!
//! This module provides interactive demonstrations and visual reports of PMAT's
//! analysis capabilities. It supports multiple output formats and protocols,
//! allowing users to explore analysis results through web interfaces, CLI reports,
//! or programmatic APIs.
//!
//! # Architecture
//!
//! - **runner**: Orchestrates demo execution and analysis pipelines
//! - **server**: Local web server for interactive HTML reports
//! - **templates**: Report generation templates (HTML, Markdown, JSON)
//! - **adapters**: Protocol-specific output adapters
//! - **assets**: Static assets for web interface
//! - **orchestration**: Demo coordination and workflow logic
//! - **protocols**: Protocol types and argument structures
//! - **processing**: Data processing and analysis extraction
//! - **web**: Web server and TUI demo functions
//!
//! # Example
//!
//! ```ignore
//! use pmat::demo::runner::DemoRunner;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Run demo on a local repository
//! let repo_path = PathBuf::from(".");
//!
//! // Create a runner and analyze
//! let runner = DemoRunner::new();
//! let report = runner.analyze(&repo_path).await?;
//!
//! // Generate HTML report
//! runner.export_html(&report, "report.html")?;
//! # Ok(())
//! # }
//! ```

pub mod adapters;
pub mod assets;
pub mod config;
pub mod export;
pub mod orchestration;
pub mod processing;
pub mod protocol_harness;
pub mod protocols;
pub mod router;
pub mod runner;
pub mod server;
pub mod showcase;
pub mod templates;
pub mod web;

// Re-export from runner (preserving original public API)
pub use runner::{detect_repository, resolve_repository, DemoReport, DemoRunner, DemoStep};

// Re-export from server (preserving original public API)
pub use server::{DemoContent, Hotspot, LocalDemoServer};

// Re-export from protocols (preserving original public API)
pub use protocols::{DemoArgs, Protocol};

// Re-export from orchestration (preserving original public API)
pub use orchestration::run_demo;
