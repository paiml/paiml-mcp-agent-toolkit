#![cfg_attr(coverage_nightly, coverage(off))]
//! HTTP server implementation using uniform contracts
//! This ensures HTTP endpoints use exactly the same contracts as CLI and MCP

mod error;
mod handlers;
mod openapi;
mod router;

#[cfg(test)]
mod tests;

pub use router::create_router;
