//! Symbol table analysis - extracts and analyzes symbols from code

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub visibility: Visibility,
    pub references: Vec<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Variable,
    Constant,
    Type,
    Interface,
    Enum,
    Module,
    Property,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub kind: ReferenceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceKind {
    Definition,
    Usage,
    Import,
    Export,
}

#[derive(Debug, Serialize)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub total_symbols: usize,
    pub unreferenced_symbols: Vec<String>,
    pub most_referenced: Vec<(String, usize)>,
}

include!("symbol_table_extraction.rs");
include!("symbol_table_formatting.rs");
include!("symbol_table_tests.rs");
