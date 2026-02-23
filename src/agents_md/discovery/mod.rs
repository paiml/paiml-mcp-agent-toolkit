#![cfg_attr(coverage_nightly, coverage(off))]
//! AGENTS.md Discovery System
//!
//! Discovers and monitors AGENTS.md files in project hierarchies with caching.

mod core;
mod tests;
mod tests_part2;
mod types;

pub use types::{
    AgentsMdDiscovery, AgentsMdFile, AgentsMdHierarchy, DiscoveryConfig, FileChange,
    FileChangeType, HierarchyNode,
};
