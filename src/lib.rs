//! # PMAT (Professional Project Quantitative Analysis Toolkit)
//!
//! A comprehensive toolkit for project analysis, quality assurance, and technical debt management.
//
// Coverage: Enable #[coverage(off)] attribute when cargo-llvm-cov instruments.
// Do not gate on `coverage_attr_stable` — cargo-llvm-cov 0.8.x sets both
// `coverage_nightly` and `coverage_attr_stable` even though the attribute is
// still gated on rustc nightly (E0658). See rust-lang/rust#84605.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![deny(unused_imports)]
#![deny(unused_variables)]
//
#![allow(clippy::needless_range_loop)]
#![allow(clippy::single_match)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::manual_clamp)]
// docs.rs: allow doc warnings that rustdoc treats as errors
#![allow(rustdoc::invalid_rust_codeblocks)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]
//! PMAT provides multiple interfaces (CLI, MCP, HTTP API) for analyzing code quality, complexity,
//! and generating actionable insights for software development teams.
//!
//! ## Core Modules
//!
//! ### Analysis Engines
//! - [`complexity`](services/complexity.rs) - Code complexity analysis using AST parsing
//! - [`entropy`] - Actionable entropy analysis for pattern detection
//! - [`tdg`] - Technical Debt Grading system with persistent scoring
//! - [`wasm`] - WebAssembly quality assurance and verification
//!
//! ### Quality Assurance  
//! - [`qdd`] - Quality-Driven Development with automated code generation
//! - [`services`] - Core analysis services and quality gates
//! - [`models`] - Data models for analysis results and reports
//!
//! ### Interface Modules
//! - [`cli`] - Command-line interface with 25+ commands
//! - [`mcp_server`] - Model Context Protocol server implementation  
//! - [`handlers`] - HTTP API handlers for REST endpoints
//! - [`demo`] - Demo server and showcase functionality
//!
//! ### Infrastructure
//! - [`agent`] - Claude Code Agent Mode implementation
//! - [`ast`] - Unified AST module for multi-language parsing
//! - [`contracts`] - Uniform contracts across ALL interfaces (CLI, MCP, HTTP)
//! - [`protocol`] - Unified protocol design per SPECIFICATION.md Section 3
//! - [`utils`] - Common utilities and helpers
//!
//! ## Quick Start Examples
//!
//! ### Basic Usage
//!
//! ```ignore
//! use pmat::{MetadataCache, ContentCache};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//! use lru::LruCache;
//! use std::num::NonZeroUsize;
//!
//! // Create caches for template management
//! let metadata_cache: MetadataCache = Arc::new(RwLock::new(
//!     LruCache::new(NonZeroUsize::new(100).expect("internal error"))
//! ));
//! let content_cache: ContentCache = Arc::new(RwLock::new(
//!     LruCache::new(NonZeroUsize::new(50).expect("internal error"))  
//! ));
//!
//! // Caches are ready for use
//! assert!(metadata_cache.read().await.len() == 0);
//! assert!(content_cache.read().await.len() == 0);
//! ```
//!
//! ### Template Server Implementation
//!
//! ```ignore
//! use pmat::{TemplateServerTrait, S3Client};
//! use anyhow::Result;
//! use std::sync::Arc;
//!
//! // Example template server implementation
//! struct MyTemplateServer {
//!     client: S3Client,
//! }
//!
//! #[async_trait::async_trait]
//! impl TemplateServerTrait for MyTemplateServer {
//!     async fn get_template_metadata(&self, uri: &str) -> Result<Arc<pmat::PublicTemplateResource>> {
//!         // Implementation would fetch from storage
//!         # use pmat::PublicTemplateResource;
//!         # let resource = PublicTemplateResource::default();
//!         # Ok(Arc::new(resource))
//!         Ok(Arc::new(PublicTemplateResource::default()))
//!     }
//!
//!     async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>> {
//!         // Implementation would fetch content from S3
//!         Ok(Arc::from("template content"))
//!     }
//!
//!     async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<pmat::PublicTemplateResource>>> {
//!         // Implementation would list templates with prefix filter
//!         Ok(vec![])
//!     }
//!
//!     fn get_metadata_cache(&self) -> Option<&pmat::MetadataCache> {
//!         None // Optional caching
//!     }
//! }
//!
//! let server = MyTemplateServer { client: S3Client };
//! // Server is ready to handle template requests
//! ```

// Feature-gated: Experimental module (0% coverage, CLI-only usage via agent_handlers)
#[cfg(all(feature = "standard-deps", feature = "agent-daemon"))]
pub mod agent; // Claude Code Agent Mode implementation
               // Feature-gated: Actor system only used by mcp_integration (0% coverage, ~6,905 lines)
#[cfg(all(feature = "standard-deps", feature = "mcp-integration"))]
pub mod agents; // Agent system with Actix actors
                // Feature-gated: Experimental module (0% coverage, not production-ready)
#[cfg(all(feature = "standard-deps", feature = "agents-md"))]
pub mod agents_md; // AGENTS.md integration for AI agent guidance
#[cfg(feature = "standard-deps")]
pub mod ast; // Unified AST module for all language parsing
#[cfg(feature = "standard-deps")]
pub mod cli;
pub mod cli_exit;
#[cfg(feature = "standard-deps")]
pub mod contracts; // Uniform contracts across ALL interfaces (CLI, MCP, HTTP)
                   // Feature-gated: Demo/showcase functionality (opt-in, ~13,400 lines)
#[cfg(all(feature = "standard-deps", feature = "demo"))]
pub mod demo;
#[cfg(feature = "standard-deps")]
pub mod docs_enforcement; // Documentation quality enforcement (TICKET-PMAT-7001)
#[cfg(feature = "standard-deps")]
pub mod entropy; // Actionable entropy analysis
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(feature = "standard-deps")]
pub mod explain; // Check/metric explanation registry (--explain)
#[cfg(feature = "standard-deps")]
pub mod graph; // Graph-theoretic analysis for dependency networks
#[cfg(feature = "standard-deps")]
pub mod handlers;
#[cfg(feature = "standard-deps")]
pub mod maintenance; // Roadmap and ticket maintenance system (Sprint 17)
#[cfg(feature = "standard-deps")]
pub mod mcp; // MCP tools and handlers (Sprint 30: Semantic search tools)
             // Feature-gated: Experimental module (0% coverage, not production-ready)
#[cfg(all(feature = "standard-deps", feature = "mcp-integration"))]
pub mod mcp_integration; // MCP protocol integration
#[cfg(feature = "standard-deps")]
pub mod mcp_pmcp; // Now always available with pmcp 1.0
#[cfg(feature = "standard-deps")]
pub mod mcp_server;
#[cfg(feature = "standard-deps")]
pub mod models;
// Feature-gated: Only used by agents and mcp_integration modules (~921 lines)
#[cfg(all(feature = "standard-deps", feature = "mcp-integration"))]
pub mod modules; // Modular monolith architecture
#[cfg(feature = "standard-deps")]
pub mod prompts; // AI prompt generation from organizational intelligence (Phase 4)
#[cfg(feature = "standard-deps")]
pub mod protocol; // Unified protocol design per SPECIFICATION.md Section 3
#[cfg(feature = "standard-deps")]
pub mod qdd; // Quality-Driven Development tool
#[cfg(feature = "standard-deps")]
pub mod quality; // Quality gates and enforcement (Sprint 18: Gate executor)
#[cfg(feature = "standard-deps")]
pub mod red_team; // Automated hallucination detection (EXTREME TDD - Sprint 47)
                  // Feature-gated: Only used by mcp_integration module (~2,572 lines, 0% coverage)
#[cfg(all(feature = "standard-deps", feature = "mcp-integration"))]
pub mod resources; // Resource control and limits
#[cfg(feature = "standard-deps")]
pub mod roadmap; // Roadmap-driven development with quality gates
#[cfg(feature = "standard-deps")]
pub mod scaffold;
#[cfg(feature = "standard-deps")]
pub mod services;
#[cfg(feature = "standard-deps")]
pub mod state; // State management with event sourcing
#[cfg(feature = "standard-deps")]
pub mod stateless_server;
#[cfg(feature = "standard-deps")]
pub mod tdg; // Technical Debt Grading system
#[cfg(feature = "standard-deps")]
pub mod test_performance;
// Feature-gated: Workflow orchestration only used by mcp_integration (0% coverage, ~5,608 lines)
#[cfg(all(feature = "standard-deps", feature = "mcp-integration"))]
pub mod workflow; // Workflow orchestration engine
                  // #[cfg(test)]
                  // pub mod testing;
                  // Feature-gated: Experimental module (0% coverage, not production-ready)
#[cfg(all(feature = "standard-deps", feature = "unified-protocol"))]
pub mod unified_protocol;
#[cfg(feature = "standard-deps")]
pub mod unified_quality; // Unified Quality Enforcement System
#[cfg(feature = "standard-deps")]
pub mod utils;
#[cfg(feature = "standard-deps")]
pub mod viz; // Terminal graph visualization (trueno-viz integration)
#[cfg(all(feature = "standard-deps", feature = "wasm-ast"))]
pub mod wasm; // WebAssembly quality assurance module

#[cfg(feature = "standard-deps")]
use anyhow::Result;
#[cfg(feature = "standard-deps")]
use lru::LruCache;
#[cfg(feature = "standard-deps")]
use std::num::NonZeroUsize;
#[cfg(feature = "standard-deps")]
use std::sync::Arc;
#[cfg(feature = "standard-deps")]
use tokio::sync::RwLock;
#[cfg(feature = "standard-deps")]
use tracing::info;

#[cfg(feature = "standard-deps")]
use crate::models::template::TemplateResource;
#[cfg(feature = "standard-deps")]
use crate::services::renderer::TemplateRenderer;

/// Shared cache for template metadata with LRU eviction policy.
///
/// This cache stores parsed template metadata to avoid repeated parsing operations.
/// Uses Arc<`RwLock`<>> for thread-safe access across async contexts.
///
/// # Examples
///
/// ```ignore
/// use pmat::MetadataCache;
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use lru::LruCache;
/// use std::num::NonZeroUsize;
///
/// // Create a metadata cache with 100 entry capacity
/// let cache: MetadataCache = Arc::new(RwLock::new(
///     LruCache::new(NonZeroUsize::new(100).expect("internal error"))
/// ));
///
/// // Cache starts empty
/// assert!(cache.read().await.len() == 0);
/// ```
#[cfg(feature = "standard-deps")]
pub type MetadataCache = Arc<RwLock<LruCache<String, Arc<TemplateResource>>>>;

/// Shared cache for template content with LRU eviction policy.
///
/// This cache stores rendered template content to improve performance
/// for frequently accessed templates.
///
/// # Examples
///
/// ```ignore
/// use pmat::ContentCache;
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use lru::LruCache;
/// use std::num::NonZeroUsize;
///
/// // Create a content cache with 50 entry capacity
/// let cache: ContentCache = Arc::new(RwLock::new(
///     LruCache::new(NonZeroUsize::new(50).expect("internal error"))
/// ));
///
/// // Insert content into cache
/// {
///     let mut cache_guard = cache.write().await;
///     cache_guard.put("template_key".to_string(), Arc::from("template content"));
/// }
///
/// // Retrieve content from cache
/// let content = cache.read().await.peek("template_key").cloned();
/// assert_eq!(content.as_deref(), Some("template content"));
/// ```
#[cfg(feature = "standard-deps")]
pub type ContentCache = Arc<RwLock<LruCache<String, Arc<str>>>>;

// Re-exports for test compatibility
#[cfg(feature = "standard-deps")]
pub use crate::models::template::TemplateResource as PublicTemplateResource;
#[cfg(feature = "standard-deps")]
pub use crate::services::renderer::TemplateRenderer as PublicTemplateRenderer;

/// Placeholder S3 client for template storage operations.
///
/// This is a lightweight implementation that satisfies trait requirements
/// without requiring the full AWS SDK dependency.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::S3Client;
///
/// let client = S3Client;
/// // Client can be used in template server implementations
/// ```
#[cfg(feature = "standard-deps")]
pub struct S3Client;

/// Template server trait defining the interface for template management operations.
///
/// This trait provides methods for retrieving template metadata and content,
/// as well as access to caching and rendering capabilities. Implementations
/// can use different storage backends (S3, local filesystem, etc.).
///
/// # Examples
///
/// ```ignore
/// use pmat::{TemplateServerTrait, TemplateRenderer, S3Client, MetadataCache, ContentCache};
/// use anyhow::Result;
/// use std::sync::Arc;
///
/// struct ExampleTemplateServer {
///     renderer: TemplateRenderer,
/// }
///
/// #[async_trait::async_trait]
/// impl TemplateServerTrait for ExampleTemplateServer {
///     async fn get_template_metadata(&self, uri: &str) -> Result<Arc<pmat::PublicTemplateResource>> {
///         // Fetch metadata from storage based on URI
///         Ok(Arc::new(pmat::PublicTemplateResource::default()))
///     }
///
///     async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>> {
///         // Fetch template content from storage
///         Ok(Arc::from("template content"))
///     }
///
///     async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<pmat::PublicTemplateResource>>> {
///         // List templates with optional prefix filter
///         Ok(vec![])
///     }
///
///     fn get_renderer(&self) -> &TemplateRenderer {
///         &self.renderer
///     }
///
///     fn get_metadata_cache(&self) -> Option<&MetadataCache> {
///         None // No caching in this example
///     }
///
///     fn get_content_cache(&self) -> Option<&ContentCache> {
///         None
///     }
///
///     fn get_s3_client(&self) -> Option<&S3Client> {
///         None
///     }
///
///     fn get_bucket_name(&self) -> Option<&str> {
///         None
///     }
/// }
/// ```
#[cfg(feature = "standard-deps")]
#[async_trait::async_trait]
pub trait TemplateServerTrait: Send + Sync {
    /// Retrieves template metadata for the given URI.
    ///
    /// # Arguments
    /// * `uri` - The URI identifying the template
    ///
    /// # Returns
    /// Template resource containing metadata information
    async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>>;

    /// Retrieves template content from storage.
    ///
    /// # Arguments  
    /// * `s3_key` - The storage key for the template content
    ///
    /// # Returns
    /// Template content as a shared string reference
    async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>>;

    /// Lists templates with optional prefix filtering.
    ///
    /// # Arguments
    /// * `prefix` - Optional prefix to filter template names
    ///
    /// # Returns
    /// Vector of template resources matching the prefix
    async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<TemplateResource>>>;

    /// Gets the template renderer instance.
    fn get_renderer(&self) -> &TemplateRenderer;

    /// Gets the metadata cache if available.
    fn get_metadata_cache(&self) -> Option<&MetadataCache>;

    /// Gets the content cache if available.  
    fn get_content_cache(&self) -> Option<&ContentCache>;

    /// Gets the S3 client if available.
    fn get_s3_client(&self) -> Option<&S3Client>;

    /// Gets the S3 bucket name if available.
    fn get_bucket_name(&self) -> Option<&str>;
}

/// Template server.
#[cfg(feature = "standard-deps")]
pub struct TemplateServer {
    pub s3_client: S3Client,
    pub bucket_name: String,
    pub metadata_cache: MetadataCache,
    pub content_cache: ContentCache,
    pub renderer: TemplateRenderer,
}

#[cfg(feature = "standard-deps")]
#[cfg_attr(coverage_nightly, coverage(off))]
impl TemplateServer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn new() -> Result<Self> {
        // Dummy implementation for Lambda compatibility
        // The stateless server should be used instead
        let cache_size = 1024;

        Ok(Self {
            s3_client: S3Client,
            bucket_name: "dummy".to_string(),
            metadata_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(cache_size / 2).expect("internal error"),
            ))),
            content_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(cache_size).expect("internal error"),
            ))),
            renderer: TemplateRenderer::new()?,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn warm_cache(&self) -> Result<()> {
        let common_templates = vec![
            "template://makefile/rust/cli",
            "template://makefile/deno/cli",
            "template://makefile/python-uv/cli",
            "template://readme/rust/cli",
            "template://gitignore/rust/cli",
        ];

        info!(
            "Warming cache with {} common templates",
            common_templates.len()
        );

        for template_uri in common_templates {
            match self.get_template_metadata(template_uri).await {
                Ok(resource) => {
                    let _ = self.get_template_content(&resource.s3_object_key).await;
                }
                Err(e) => {
                    info!("Failed to warm cache for {}: {}", template_uri, e);
                }
            }
        }

        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_template_metadata(&self, _uri: &str) -> Result<Arc<TemplateResource>> {
        // Dummy implementation - use StatelessTemplateServer instead
        Err(anyhow::anyhow!(
            "TemplateServer with S3 is deprecated. Use StatelessTemplateServer instead."
        ))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_template_content(&self, _s3_key: &str) -> Result<Arc<str>> {
        // Dummy implementation - use StatelessTemplateServer instead
        Err(anyhow::anyhow!(
            "TemplateServer with S3 is deprecated. Use StatelessTemplateServer instead."
        ))
    }
}

#[cfg(feature = "standard-deps")]
#[async_trait::async_trait]
#[cfg_attr(coverage_nightly, coverage(off))]
impl TemplateServerTrait for TemplateServer {
    async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>> {
        self.get_template_metadata(uri).await
    }

    async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>> {
        self.get_template_content(s3_key).await
    }

    async fn list_templates(&self, _prefix: &str) -> Result<Vec<Arc<TemplateResource>>> {
        // Dummy implementation - use StatelessTemplateServer instead
        Err(anyhow::anyhow!(
            "TemplateServer with S3 is deprecated. Use StatelessTemplateServer instead."
        ))
    }

    fn get_renderer(&self) -> &TemplateRenderer {
        &self.renderer
    }

    fn get_metadata_cache(&self) -> Option<&MetadataCache> {
        Some(&self.metadata_cache)
    }

    fn get_content_cache(&self) -> Option<&ContentCache> {
        Some(&self.content_cache)
    }

    fn get_s3_client(&self) -> Option<&S3Client> {
        Some(&self.s3_client)
    }

    fn get_bucket_name(&self) -> Option<&str> {
        Some(&self.bucket_name)
    }
}

// Public exports for CLI consumption
#[cfg(feature = "standard-deps")]
pub use models::error::TemplateError;
#[cfg(feature = "standard-deps")]
pub use models::template::{ParameterSpec, ParameterType};
#[cfg(feature = "standard-deps")]
pub use services::template_service::{
    generate_template, list_templates, scaffold_project, search_templates, validate_template,
};

// #696: `pub async fn run_mcp_server<T: TemplateServerTrait>(…)` used to live
// here, together with its private helper chain (`should_skip_line`,
// `process_mcp_line`, `parse_mcp_request`, `handle_valid_request`,
// `handle_parse_error`, `write_response_to_stdout`). It was a SECOND MCP stdio
// server, reachable from the library API and kept alive by its own tests, and
// it did not serve what `pmat` serves: 21 tools against the live server's 20,
// only 7 names in common, those 7 taking different arguments
// (`project_path` string vs `paths` array), and 7 of the 21 describing
// themselves as "(unimplemented stub — KAIZEN-0200)" and answering -32001.
// An advertised path that reaches a different server is the defect, so it is
// deleted rather than re-pointed.
//
// The one MCP stdio entry point is `crate::mcp_pmcp::run_stdio_server`, which
// `src/bin/pmat.rs` and `cli::run` both call. `mcp_pmcp`'s
// `lib_rs_exposes_no_second_mcp_stdio_server` test fails if this file grows
// another one.

// Tests previously included via #[path = "../tests/..."] are now compiled
// only through tests/all.rs to avoid "file loaded as module multiple times"
// errors in Rust 1.94+. See paiml-mcp-agent-toolkit#282.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(all(test, feature = "standard-deps"))]
mod lib_unit_tests {
    use super::*;

    // #696: the unit tests for the legacy MCP stdio server
    // (`test_should_skip_line`, `test_parse_mcp_request_valid`,
    // `test_parse_mcp_request_invalid`, `test_write_response_to_stdout`,
    // `test_handle_parse_error`) went with the code they tested. The one MCP
    // stdio server is covered in `mcp_pmcp`.

    #[tokio::test]
    async fn test_template_server_new() {
        let server = TemplateServer::new().await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_template_server_deprecated_methods() {
        let server = TemplateServer::new().await.unwrap();

        // get_template_metadata should return deprecated error
        let metadata_result = server.get_template_metadata("test://uri").await;
        assert!(metadata_result.is_err());
        assert!(metadata_result
            .unwrap_err()
            .to_string()
            .contains("deprecated"));

        // get_template_content should return deprecated error
        let content_result = server.get_template_content("test_key").await;
        assert!(content_result.is_err());
        assert!(content_result
            .unwrap_err()
            .to_string()
            .contains("deprecated"));
    }

    #[tokio::test]
    async fn test_template_server_trait_methods() {
        let server = TemplateServer::new().await.unwrap();

        // Test trait method implementations - just verify they don't panic
        let _ = server.get_renderer();
        assert!(server.get_metadata_cache().is_some());
        assert!(server.get_content_cache().is_some());
        assert!(server.get_s3_client().is_some());
        assert!(server.get_bucket_name().is_some());

        // list_templates should also return deprecated error
        let list_result = TemplateServerTrait::list_templates(&server, "").await;
        assert!(list_result.is_err());
    }

    #[test]
    fn test_s3_client_struct() {
        let _client = S3Client; // Just verify it can be constructed
    }

    // #696: `test_process_mcp_line_valid` / `test_process_mcp_line_invalid_json`
    // were deleted with `process_mcp_line`. The first also asserted
    // `result.is_ok() || result.is_err()`, which is true of every `Result`.

    #[tokio::test]
    async fn test_template_server_warm_cache() {
        let server = TemplateServer::new().await.unwrap();
        // warm_cache calls get_template_metadata which returns error
        let result = server.warm_cache().await;
        // Should complete without panic (errors are logged)
        assert!(result.is_ok());
    }
}

// #976: `cargo test` never compiles `build.rs`, so the build script's logic is
// unreachable from every test target. Its testable core lives in
// `build_support.rs` at the crate root, which `build.rs` pulls in with
// `include!`; compiling the same file here under `cfg(test)` is what lets
// `cargo test --lib` run its regression tests. Test-only: this adds nothing to
// the shipped library.
#[cfg(test)]
#[allow(dead_code, clippy::unwrap_used, clippy::expect_used)]
#[path = "../build_support.rs"]
mod build_support;

// PMAT-630 (#1034 EV-4): the mutation-on-diff gate is a shell script, for the
// same reason `build_support` is compiled here — the thing CI runs has to be the
// thing that is tested. Registering the module in `lib.rs` rather than adding a
// file under `tests/` is not a style choice: `autotests = false`
// (`Cargo.toml:30`) means an unregistered test file is silently never compiled,
// and a mutation gate whose own tests do not run would be the joke it exists to
// prevent. `cargo test --lib -- mutation_diff_gate` lists them.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod mutation_diff_gate_tests;
