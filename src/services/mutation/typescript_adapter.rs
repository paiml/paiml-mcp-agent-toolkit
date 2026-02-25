#![cfg_attr(coverage_nightly, coverage(off))]
//! TypeScript/JavaScript language adapter for mutation testing
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass tests

use super::language::{LanguageAdapter, TestRunResult};
use super::operators::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "typescript-ast")]
use swc_common::{sync::Lrc, FileName, SourceMap};
#[cfg(feature = "typescript-ast")]
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

/// TypeScript/JavaScript language adapter
pub struct TypeScriptAdapter;

impl TypeScriptAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeScriptAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// LanguageAdapter trait implementation
include!("typescript_adapter_language_impl.rs");

// Helper functions: find_package_json_root, parse_test_failures,
// extract_test_name, detect_test_command
include!("typescript_adapter_parsing.rs");

// Unit tests
include!("typescript_adapter_unit_tests.rs");
