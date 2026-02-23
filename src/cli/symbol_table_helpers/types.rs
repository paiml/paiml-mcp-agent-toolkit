#![cfg_attr(coverage_nightly, coverage(off))]
//! Core types for symbol table analysis

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub file: PathBuf,
    pub line: usize,
    pub visibility: String,
    pub is_async: bool,
}
