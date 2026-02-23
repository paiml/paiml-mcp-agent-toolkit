#![cfg_attr(coverage_nightly, coverage(off))]
//! Execution mode, refactor mode, demo protocol, and WASM-related enums

use clap::ValueEnum;
use std::fmt;

/// Execution mode for the CLI
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Cli,
    Mcp,
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMode::Cli => write!(f, "cli"),
            ExecutionMode::Mcp => write!(f, "mcp"),
        }
    }
}

/// Refactor mode
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum RefactorMode {
    Batch,
    Interactive,
}

impl fmt::Display for RefactorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorMode::Batch => write!(f, "batch"),
            RefactorMode::Interactive => write!(f, "interactive"),
        }
    }
}

/// Demo protocol selection
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DemoProtocol {
    Cli,
    Http,
    Mcp,
    #[cfg(feature = "tui")]
    Tui,
    All,
}

impl fmt::Display for DemoProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DemoProtocol::Cli => write!(f, "cli"),
            DemoProtocol::Http => write!(f, "http"),
            DemoProtocol::Mcp => write!(f, "mcp"),
            #[cfg(feature = "tui")]
            DemoProtocol::Tui => write!(f, "tui"),
            DemoProtocol::All => write!(f, "all"),
        }
    }
}

/// Output format for WASM analysis
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum WasmOutputFormat {
    /// Summary output with key metrics
    Summary,
    /// Detailed analysis with all components
    Detailed,
    /// JSON format for programmatic use
    Json,
    /// SARIF format for security results
    Sarif,
}

impl fmt::Display for WasmOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmOutputFormat::Summary => write!(f, "summary"),
            WasmOutputFormat::Detailed => write!(f, "detailed"),
            WasmOutputFormat::Json => write!(f, "json"),
            WasmOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Deep WASM source language
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepWasmLanguage {
    /// Rust language
    Rust,
    /// Ruchy language
    Ruchy,
}

impl fmt::Display for DeepWasmLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepWasmLanguage::Rust => write!(f, "rust"),
            DeepWasmLanguage::Ruchy => write!(f, "ruchy"),
        }
    }
}

/// Deep WASM analysis focus
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepWasmFocus {
    /// Full pipeline analysis
    Full,
    /// Source code only
    Source,
    /// Compilation pipeline
    Compilation,
    /// Runtime behavior
    Runtime,
    /// JavaScript interop
    Interop,
}

impl fmt::Display for DeepWasmFocus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepWasmFocus::Full => write!(f, "full"),
            DeepWasmFocus::Source => write!(f, "source"),
            DeepWasmFocus::Compilation => write!(f, "compilation"),
            DeepWasmFocus::Runtime => write!(f, "runtime"),
            DeepWasmFocus::Interop => write!(f, "interop"),
        }
    }
}

/// Deep WASM output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepWasmOutputFormat {
    /// Markdown report
    Markdown,
    /// JSON data
    Json,
    /// HTML report
    Html,
}

impl fmt::Display for DeepWasmOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepWasmOutputFormat::Markdown => write!(f, "markdown"),
            DeepWasmOutputFormat::Json => write!(f, "json"),
            DeepWasmOutputFormat::Html => write!(f, "html"),
        }
    }
}
