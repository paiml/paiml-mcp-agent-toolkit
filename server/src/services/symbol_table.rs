//! Symbol table for location resolution and qualified name mapping
//!
//! This module provides efficient symbol resolution for proof annotation location mapping.

use crate::models::unified_ast::{BytePos, Location, QualifiedName, RelativeLocation, Span};
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Symbol table for mapping qualified names to locations
#[derive(Debug)]
pub struct SymbolTable {
    /// Maps qualified names to canonical locations
    symbols: DashMap<QualifiedName, Location>,
    /// Reverse index for span-to-symbol lookup (simplified approach)
    /// Maps file path to sorted list of (start_pos, qualified_name) for binary search
    span_index: DashMap<std::path::PathBuf, Vec<(BytePos, QualifiedName)>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: DashMap::new(),
            span_index: DashMap::new(),
        }
    }

    /// Insert a symbol with its location
    pub fn insert(&self, qualified_name: QualifiedName, location: Location) {
        debug!("Inserting symbol: {} at {:?}", qualified_name, location);

        // Insert into main symbol table
        self.symbols
            .insert(qualified_name.clone(), location.clone());

        // Update span index for reverse lookup
        let mut entry = self
            .span_index
            .entry(location.file_path.clone())
            .or_default();
        entry.push((location.span.start, qualified_name));
        // Keep sorted by start position for binary search
        entry.sort_by_key(|(pos, _)| *pos);
    }

    /// Resolve a relative location to a canonical location
    pub fn resolve_relative(&self, rel: &RelativeLocation, file: &Path) -> Option<Location> {
        match rel {
            RelativeLocation::Function { name, module } => {
                let qname = self.build_qualified_name(file, module.as_deref(), name)?;
                self.symbols.get(&qname).map(|entry| entry.clone())
            }
            RelativeLocation::Span { start, end } => Some(Location {
                file_path: file.to_owned(),
                span: Span {
                    start: BytePos(*start),
                    end: BytePos(*end),
                },
            }),
            RelativeLocation::Symbol { qualified_name } => {
                let qname: QualifiedName = qualified_name.parse().ok()?;
                self.symbols.get(&qname).map(|entry| entry.clone())
            }
        }
    }

    /// Get symbol at a specific location
    pub fn symbol_at_location(&self, location: &Location) -> Option<QualifiedName> {
        if let Some(spans) = self.span_index.get(&location.file_path) {
            // Binary search for the position
            let pos = location.span.start;
            match spans.binary_search_by_key(&pos, |(start_pos, _)| *start_pos) {
                Ok(index) => Some(spans[index].1.clone()),
                Err(index) => {
                    // Find the closest symbol that contains this position
                    if index > 0 {
                        Some(spans[index - 1].1.clone())
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    /// Find all symbols within a span
    pub fn symbols_in_span(&self, location: &Location) -> Vec<QualifiedName> {
        if let Some(spans) = self.span_index.get(&location.file_path) {
            spans
                .iter()
                .filter(|(pos, _)| location.span.contains(*pos))
                .map(|(_, qname)| qname.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get the location of a qualified name
    pub fn get_location(&self, qualified_name: &QualifiedName) -> Option<Location> {
        self.symbols.get(qualified_name).map(|entry| entry.clone())
    }

    /// Get all symbols in the table
    pub fn all_symbols(&self) -> Vec<(QualifiedName, Location)> {
        self.symbols
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Clear all symbols
    pub fn clear(&self) {
        self.symbols.clear();
        self.span_index.clear();
    }

    /// Get symbol count
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Build a qualified name from file path, module, and name
    fn build_qualified_name(
        &self,
        file: &Path,
        module: Option<&str>,
        name: &str,
    ) -> Option<QualifiedName> {
        let module_path = match module {
            Some(explicit_module) => self.parse_explicit_module(explicit_module),
            None => self.infer_module_from_file_path(file),
        };

        Some(QualifiedName::new(module_path, name.to_string()))
    }

    /// Parse explicitly provided module path
    fn parse_explicit_module(&self, module: &str) -> Vec<String> {
        module.split("::").map(|s| s.to_string()).collect()
    }

    /// Infer module path from file system structure
    fn infer_module_from_file_path(&self, file: &Path) -> Vec<String> {
        let mut module_path = Vec::new();

        // Add file stem if it's a significant module file
        if let Some(stem_str) = self.extract_significant_file_stem(file) {
            module_path.push(stem_str);
        }

        // Add parent directories as module path
        self.add_parent_directories_to_module_path(file, &mut module_path);

        module_path
    }

    /// Extract significant file stem (excludes common non-module files)
    fn extract_significant_file_stem(&self, file: &Path) -> Option<String> {
        file.file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|&stem_str| !matches!(stem_str, "mod" | "lib" | "main"))
            .map(|stem_str| stem_str.to_string())
    }

    /// Add parent directories to module path, stopping at src directory
    fn add_parent_directories_to_module_path(&self, file: &Path, module_path: &mut Vec<String>) {
        let mut current = file.parent();
        while let Some(parent) = current {
            if let Some(dir_name) = self.extract_directory_name(parent) {
                if dir_name == "src" {
                    break;
                }
                module_path.insert(0, dir_name);
            }
            current = parent.parent();
        }
    }

    /// Extract directory name as string
    fn extract_directory_name(&self, path: &Path) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name_str| name_str.to_string())
    }
}

/// Builder for constructing symbol tables from AST analysis
pub struct SymbolTableBuilder {
    table: Arc<SymbolTable>,
}

impl SymbolTableBuilder {
    pub fn new() -> Self {
        Self {
            table: Arc::new(SymbolTable::new()),
        }
    }

    pub fn add_symbol(&self, qualified_name: QualifiedName, location: Location) {
        self.table.insert(qualified_name, location);
    }

    pub fn build(self) -> Arc<SymbolTable> {
        self.table
    }
}

impl Default for SymbolTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_symbol_table_insertion_and_lookup() {
        let table = SymbolTable::new();

        let qname = QualifiedName::new(
            vec!["std".to_string(), "collections".to_string()],
            "HashMap".to_string(),
        );
        let location = Location::new(PathBuf::from("src/lib.rs"), 100, 200);

        table.insert(qname.clone(), location.clone());

        assert_eq!(table.get_location(&qname), Some(location));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_relative_location_resolution() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("src/lib.rs");

        // Test span resolution
        let rel_span = RelativeLocation::Span {
            start: 100,
            end: 200,
        };
        let resolved = table.resolve_relative(&rel_span, &file_path).unwrap();

        assert_eq!(resolved.file_path, file_path);
        assert_eq!(resolved.span.start.0, 100);
        assert_eq!(resolved.span.end.0, 200);
    }

    #[test]
    fn test_qualified_name_parsing() {
        let qname = QualifiedName::from_string("std::collections::HashMap").unwrap();
        assert_eq!(qname.module_path, vec!["std", "collections"]);
        assert_eq!(qname.name, "HashMap");
        assert_eq!(qname.to_string(), "std::collections::HashMap");
    }

    #[test]
    fn test_symbol_table_builder() {
        let builder = SymbolTableBuilder::new();
        let qname = QualifiedName::new(vec!["test".to_string()], "function".to_string());
        let location = Location::new(PathBuf::from("test.rs"), 0, 10);

        builder.add_symbol(qname.clone(), location.clone());
        let table = builder.build();

        assert_eq!(table.get_location(&qname), Some(location));
    }
}
