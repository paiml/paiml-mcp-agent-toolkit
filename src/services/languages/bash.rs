#![cfg_attr(coverage_nightly, coverage(off))]
//! Bash/Shell Script Analysis Support for PMAT
//!
//! This module provides Bash-specific analysis capabilities using lexical analysis
//! and partial AST extraction for shell scripts within static analysis constraints.

#[cfg(feature = "shell-ast")]
use crate::services::context::AstItem;
use std::path::{Path, PathBuf};

/// Bash script analyzer that extracts shell-specific information
pub struct BashScriptAnalyzer {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    script_name: String,
    function_count: usize,
    variable_count: usize,
    command_count: usize,
}

/// Bash complexity analyzer for shell-specific metrics (complexity ≤10)
pub struct BashComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    _nesting_depth: u32,
}

impl Default for BashComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Shell script safety and best practices analyzer (complexity ≤10)
pub struct ShellSafetyAnalyzer {
    safety_violations: Vec<String>,
    best_practice_warnings: Vec<String>,
}

impl Default for ShellSafetyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Shell command parser for lexical analysis (complexity ≤10)
pub struct ShellCommandParser {
    commands: Vec<String>,
    variables: Vec<String>,
}

impl Default for ShellCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

// BashScriptAnalyzer methods: function extraction, variable analysis, command parsing, control flow
include!("bash_analysis.rs");

// BashComplexityAnalyzer methods: cyclomatic/cognitive complexity, pipeline and conditional analysis
include!("bash_complexity.rs");

// ShellSafetyAnalyzer and ShellCommandParser methods: safety checks, security, best practices, parsing
include!("bash_safety.rs");

// Unit tests and property tests for all bash analyzers
include!("bash_tests.rs");
