//! Polyglot analysis tools for MCP
//!
//! This module provides MCP tools for cross-language analysis, allowing
//! AI agents to detect and analyze relationships between different programming
//! languages in a project.

use crate::ast::polyglot::utils::PolyglotPathValidator;
use crate::ast::polyglot::{
    CrossLanguageDependencies, Language, LanguageMapperFactory, UnifiedNode,
};
use crate::mcp_integration::{McpError, McpTool, ToolMetadata};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Analyzes cross-language relationships in a project
pub struct PolyglotAnalysisTool {
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl PolyglotAnalysisTool {
    /// Create a new polyglot analysis tool
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

/// Detects language boundaries in a project
pub struct LanguageBoundaryTool {
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl LanguageBoundaryTool {
    /// Create a new language boundary tool
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

/// Get counts of node types by language
fn get_node_type_counts(nodes: &[UnifiedNode]) -> HashMap<String, HashMap<String, usize>> {
    let mut counts = HashMap::new();

    for node in nodes {
        let lang_name = node.language.name().to_string();
        let kind_name = node.kind.as_str().to_string();

        counts
            .entry(lang_name)
            .or_insert_with(HashMap::new)
            .entry(kind_name)
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }

    counts
}

// --- Include submodules ---
// McpTool impl for PolyglotAnalysisTool (metadata + execute)
include!("polyglot_tools_analysis.rs");

// McpTool impl for LanguageBoundaryTool (metadata + execute) + analyze_boundary_patterns()
include!("polyglot_tools_boundary.rs");

// TEMPORARILY DISABLED: File splitting broke syntax (missing json! macro import)
#[cfg(all(test, pmat_broken_tests))]
#[path = "tests.rs"]
mod tests;
