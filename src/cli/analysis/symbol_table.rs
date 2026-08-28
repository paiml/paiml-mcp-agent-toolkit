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
    /// The top `--top-files` names by resolved use sites.
    ///
    /// This was hard-capped at 10 with nothing saying so and `--top-files`
    /// ignored, so its length was 10 whether the project had 11 referenced
    /// names or 11 000 — a cap wearing the shape of a total. It now honours the
    /// flag, and [`Self::referenced_symbol_count`] names the population it is
    /// drawn from.
    pub most_referenced: Vec<(String, usize)>,
    /// How many distinct names have at least one resolved use site — the whole
    /// that `most_referenced` is the top slice of.
    pub referenced_symbol_count: usize,
    /// How many source files this table was extracted from.
    ///
    /// The denominator `total_symbols` had none of (#1015): "Total symbols: 0"
    /// was printed, and `{"total_symbols": 0}` serialized, both for a project
    /// whose files genuinely declare nothing and for a directory holding no
    /// source file at all. A consumer could not tell the two apart, and the
    /// handler refuses the second case using this count.
    pub files_scanned: usize,
}

include!("symbol_table_references.rs");
include!("symbol_table_extraction.rs");
include!("symbol_table_formatting.rs");
include!("symbol_table_tests.rs");
