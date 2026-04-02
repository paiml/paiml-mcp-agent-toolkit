//! Unified contract definitions for ALL interfaces (CLI, MCP, HTTP)
//!
//! CRITICAL: This is the SINGLE SOURCE OF TRUTH for all command parameters.
//! Every interface MUST use these exact contracts with no variations.

pub mod adapter;
pub mod cli_impl;
pub mod cli_mapping;
#[cfg(feature = "http-server")]
pub mod http_impl;
// pub mod mcp_impl; // Disabled due to pmcp dependency issues
pub mod mcp_mapping;
pub mod mcp_simple;
pub mod real_service;
pub mod service;
pub mod simple_service;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests;
pub mod uniform_cli_commands;
pub mod versioning;

use crate::utils::path_validator::PathValidator;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Missing required parameter: {0}")]
    MissingParam(&'static str),

    #[error("Invalid timeout value")]
    InvalidTimeout,

    #[error("Too many files requested: {0} (max: 1000)")]
    TooManyFiles(usize),

    #[error("Invalid parameter value: {0}")]
    InvalidValue(String),
}

/// Canonical output format — ONE type for the entire codebase (CB-1300).
/// Modules use the subset of variants they need and match with `_ =>`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Yaml,
    Markdown,
    Csv,
    Summary,
    Text,
    Plain,
    Junit,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Csv => write!(f, "csv"),
            OutputFormat::Summary => write!(f, "summary"),
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Plain => write!(f, "plain"),
            OutputFormat::Junit => write!(f, "junit"),
        }
    }
}

/// SATD severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum SatdSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// --- Contract struct definitions ---
include!("contract_definitions.rs");

// --- ContractValidation trait and implementations ---
include!("contract_validation.rs");

// --- Unit tests and property tests ---
include!("contract_tests.rs");
