//! Documentation Enforcement
//!
//! TICKET: PMAT-7001
//!
//! This module enforces documentation quality standards for both CLI and MCP
//! interfaces. It ensures that all commands, flags, and parameters have
//! complete, accurate, and non-generic documentation.
//!
//! ## Modules
//!
//! - `generic_detector` - Detects generic/placeholder documentation
//! - `cli_checker` - Validates CLI help text and flag documentation
//! - `mcp_checker` - Validates MCP tool descriptions and schemas

pub mod generic_detector;
pub mod cli_checker;
pub mod mcp_checker;

// Re-export main functions
pub use generic_detector::{is_generic_description, suggest_improvements};
pub use cli_checker::validate_cli_documentation;
pub use mcp_checker::validate_mcp_documentation;
