#![cfg_attr(coverage_nightly, coverage(off))]
//! Protocol types, argument structures, and protocol-related utilities.

use anyhow::Result;

/// Arguments for running a demo.
#[derive(Debug, Clone)]
pub struct DemoArgs {
    pub path: Option<std::path::PathBuf>,
    pub url: Option<String>,
    pub repo: Option<String>,
    pub format: crate::cli::OutputFormat,
    pub no_browser: bool,
    pub port: Option<u16>,
    pub web: bool,
    pub target_nodes: usize,
    pub centrality_threshold: f64,
    pub merge_threshold: usize,
    pub protocol: Protocol,
    pub show_api: bool,
    pub debug: bool,
    pub debug_output: Option<std::path::PathBuf>,
    pub skip_vendor: bool,
    pub max_line_length: Option<usize>,
}

/// Supported demo protocols.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol {
    Cli,
    Http,
    Mcp,
    #[cfg(feature = "tui")]
    Tui,
    All,
}

// Helper functions: conversion, formatting, request building, output printing.
include!("protocols_helpers.rs");

// Unit tests for protocols module.
include!("protocols_tests.rs");

// Property-based tests for protocols module.
include!("protocols_property_tests.rs");
