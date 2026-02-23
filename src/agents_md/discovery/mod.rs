#![cfg_attr(coverage_nightly, coverage(off))]
//! AGENTS.md Discovery System
//!
//! Discovers and monitors AGENTS.md files in project hierarchies with caching.

mod core;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types and the discovery struct
pub use core::AgentsMdDiscovery;
pub use types::{
    AgentsMdFile, AgentsMdHierarchy, DiscoveryConfig, FileChange, FileChangeType, HierarchyNode,
};
