//! Integration and Acceptance Tests for PMAT Server
//!
//! This module provides comprehensive testing for all PMAT interfaces:
//! - CLI acceptance tests (100% command coverage)
//! - MCP acceptance tests (protocol compliance)
//! - HTTP API acceptance tests (REST compliance)

pub mod cli_acceptance;
pub mod http_acceptance;
pub mod mcp_acceptance;
