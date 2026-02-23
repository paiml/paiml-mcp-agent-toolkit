#![cfg_attr(coverage_nightly, coverage(off))]
//! Background Daemon for Claude Code Agent Mode
//!
//! Manages the lifecycle of the PMAT background agent service with graceful
//! startup, shutdown, and continuous operation capabilities.

mod event_loop;
mod lifecycle;
mod manager;
mod types;

// Re-export all public types for backward compatibility
pub use lifecycle::AgentDaemon;
pub use manager::DaemonManager;
pub use types::{
    DaemonCommand, DaemonConfig, DaemonSettings, DaemonState, DaemonStatus, QualityGateResult,
};

#[cfg(test)]
mod tests;
