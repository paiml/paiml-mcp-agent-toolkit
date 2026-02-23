#![cfg_attr(coverage_nightly, coverage(off))]
//! HTTP server implementation using uniform contracts
//! This ensures HTTP endpoints use exactly the same contracts as CLI and MCP

mod handlers;
mod openapi;
mod router;
mod tests;
mod tests_endpoints;
mod tests_openapi;
mod tests_property;
mod types;

pub use router::create_router;
