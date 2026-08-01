//! Symbol table analysis - extracts and analyzes symbols from code

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// How many resolved use sites `--show-references` prints per symbol before
/// summarising the rest as "(+N more)".
const REFERENCE_SITES_SHOWN: usize = 5;

/// How many symbols of each kind `--format summary` lists before "... and N more".
/// `--format detailed` lists all of them.
const SYMBOLS_PER_GROUP_IN_SUMMARY: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Symbol.
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub visibility: Visibility,
    pub references: Vec<Reference>,
}

// `Ord` is derived (declaration order) so symbol groups can be held in a
// `BTreeMap` and rendered in a fixed order; a `HashMap` reshuffled the sections
// between runs of the same command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Symbol kind.
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
/// Visibility.
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Reference.
pub struct Reference {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub kind: ReferenceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Reference kind.
pub enum ReferenceKind {
    Definition,
    Usage,
    Import,
    Export,
}

#[derive(Debug, Serialize)]
/// Symbol table.
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub total_symbols: usize,
    pub unreferenced_symbols: Vec<String>,
    pub most_referenced: Vec<(String, usize)>,
}

include!("symbol_table_references.rs");
include!("symbol_table_extraction.rs");
include!("symbol_table_formatting.rs");
include!("symbol_table_tests.rs");
