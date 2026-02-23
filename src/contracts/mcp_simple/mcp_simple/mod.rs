//! Simplified MCP implementation using uniform contracts
//! This version doesn't depend on pmcp crate

mod handler;
mod schemas;
#[cfg(test)]
mod tests;

pub use handler::SimpleMcpHandler;
