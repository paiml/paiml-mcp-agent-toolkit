//! Claude Code Agent Mode Implementation
//!
//! This module implements PMAT as a Claude Code background agent, providing:
//! - Continuous quality monitoring with file system watching
//! - AI-driven refactoring suggestions following Toyota Way principles
//! - Quality gate automation and proactive analysis
//! - MCP server for seamless Claude Code integration
//!
//! # Architecture
//!
//! The agent operates in multiple modes:
//! - **Background Daemon**: Continuous monitoring and proactive quality management
//! - **Interactive Assistant**: On-demand analysis and guided refactoring
//! - **CI/CD Integration**: Headless execution with structured output
//!
//! # Implementation Roadmap
//!
//! - **PMAT-7001**: MCP Server Core Implementation
//! - **PMAT-7002**: Quality Monitoring Engine
//! - **PMAT-7003**: Claude Code Integration Testing

pub mod mcp_server;
pub mod quality_monitor;
pub mod daemon;

pub use mcp_server::*;
pub use quality_monitor::*;
pub use daemon::*;