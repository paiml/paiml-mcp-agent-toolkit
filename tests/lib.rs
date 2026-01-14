//! Integration and Acceptance Tests for PMAT Server
//!
//! This module provides comprehensive testing for all PMAT interfaces:
//! - CLI acceptance tests (100% command coverage)
//! - MCP acceptance tests (protocol compliance)
//! - HTTP API acceptance tests (REST compliance)

#[cfg(not(feature = "skip-slow-tests"))]
pub mod cli_acceptance;
#[cfg(not(feature = "skip-slow-tests"))]
pub mod http_acceptance;
#[cfg(not(feature = "skip-slow-tests"))]
pub mod mcp_acceptance;
