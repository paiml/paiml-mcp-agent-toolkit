#![cfg_attr(coverage_nightly, coverage(off))]
//! MCP Server Core Implementation for Claude Code Agent Mode
//!
//! PMAT-7001: Basic MCP server with stdio transport, core tool implementations,
//! configuration system for Claude Code integration, and basic file system watching.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info};

use crate::services::analysis_service::{
    AnalysisInput, AnalysisOperation, AnalysisOptions, AnalysisService,
};
use crate::services::quality_gate_service::{
    QualityCheck, QualityGateInput, QualityGateOutput, QualityGateService,
};
use crate::services::service_base::Service;

/// Claude Code Agent MCP Server
///
/// Implements the MCP (Model Context Protocol) server interface for seamless
/// integration with Claude Code as a background agent service.
#[derive(Clone)]
pub struct ClaudeCodeAgentMcpServer {
    /// Server configuration
    config: AgentConfig,

    /// Currently monitored projects
    monitored_projects: HashMap<String, MonitoredProject>,

    /// Quality monitoring state
    quality_monitor: Option<mpsc::Sender<QualityMonitorCommand>>,

    /// Services for analysis
    quality_gate_service: Arc<QualityGateService>,

    /// Analysis service for complexity and other metrics
    analysis_service: Arc<AnalysisService>,
}

/// Configuration for the Claude Code agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name for MCP identification
    pub name: String,

    /// Agent version
    pub version: String,

    /// Default complexity threshold for monitoring
    pub complexity_threshold: u32,

    /// File patterns to watch
    pub watch_patterns: Vec<String>,

    /// Update interval in seconds for monitoring
    pub update_interval: u64,

    /// Maximum number of concurrent projects to monitor
    pub max_projects: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "pmat-agent".to_string(),
            version: "1.0.0".to_string(),
            complexity_threshold: 20, // Toyota Way standard
            watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
                "**/*.java".to_string(),
                "**/*.go".to_string(),
                "**/*.cpp".to_string(),
                "**/*.c".to_string(),
                "**/*.hpp".to_string(),
                "**/*.h".to_string(),
            ],
            update_interval: 5, // 5 seconds
            max_projects: 10,
        }
    }
}

/// Information about a monitored project
#[derive(Debug, Clone)]
pub struct MonitoredProject {
    /// Project root path
    pub path: PathBuf,

    /// Project name
    pub name: String,

    /// Watch patterns for this project
    pub watch_patterns: Vec<String>,

    /// Complexity threshold
    pub complexity_threshold: u32,

    /// Last analysis results
    pub last_analysis: Option<ProjectAnalysisResult>,

    /// Monitoring start time
    pub started_at: std::time::SystemTime,
}

/// Result of project analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysisResult {
    /// Timestamp of analysis
    pub timestamp: String,

    /// Overall quality score (0.0 - 1.0)
    pub quality_score: f64,

    /// Files analyzed
    pub files_analyzed: usize,

    /// Functions analyzed
    pub functions_analyzed: usize,

    /// Average complexity
    pub avg_complexity: f64,

    /// Number of hotspot functions
    pub hotspot_functions: usize,

    /// SATD issues found
    pub satd_issues: usize,

    /// Quality gate status
    pub quality_gate_status: String,

    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Commands for quality monitoring
#[derive(Debug)]
pub enum QualityMonitorCommand {
    StartMonitoring {
        project_path: PathBuf,
        config: Box<MonitoredProject>,
    },
    StopMonitoring {
        project_id: String,
    },
    GetStatus {
        project_id: String,
        response_tx: Box<oneshot::Sender<Option<ProjectAnalysisResult>>>,
    },
    Shutdown,
}

// --- Protocol: constructor, stdio transport, MCP protocol, request dispatch, tool handlers ---
include!("mcp_server_protocol.rs");

// --- Tool definitions: MCP tool schema ---
include!("mcp_server_tool_defs.rs");

// --- Capabilities: resource/prompt capabilities ---
include!("mcp_server_capabilities.rs");

// --- Monitoring: start/stop/status monitoring handlers ---
include!("mcp_server_monitoring.rs");

// --- Quality: quality gates handler, quality monitor background task ---
include!("mcp_server_quality.rs");

#[cfg(test)]
mod tests;
