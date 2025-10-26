// Symbol table for cross-module resolution
// Complexity: All functions ≤ 10
// SATD: Zero tolerance

use super::*;
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};

/// Global symbol table for O(1) lookups
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Map from symbol name to all definitions
    symbols: FxHashMap<String, Vec<SymbolEntry>>,
    /// Map from file path to symbols defined in that file
    file_symbols: FxHashMap<PathBuf, Vec<String>>,
    /// Map from module path to file path
    module_map: FxHashMap<String, PathBuf>,
}

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub symbol: Symbol,
    pub file_path: PathBuf,
    pub module_path: String,
    pub usage_count: usize,
    pub is_exported: bool,
}

impl SymbolTable {
    /// Create new empty symbol table
    /// Complexity: 1
    pub fn new() -> Self {
        SymbolTable {
            symbols: FxHashMap::default(),
            file_symbols: FxHashMap::default(),
            module_map: FxHashMap::default(),
        }
    }

    /// Insert a symbol into the table
    /// Complexity: 3 (hashmap operations)
    pub fn insert(&mut self, name: String, entry: SymbolEntry) {
        // Add to main symbol map
        self.symbols
            .entry(name.clone())
            .or_default()
            .push(entry.clone());

        // Add to file symbol map
        self.file_symbols
            .entry(entry.file_path.clone())
            .or_default()
            .push(name);

        // Update module map
        self.module_map
            .insert(entry.module_path.clone(), entry.file_path);
    }

    /// Resolve a symbol by name
    /// Complexity: 4 (lookup + visibility check)
    pub fn resolve(&self, name: &str, from_module: &str) -> Option<&SymbolEntry> {
        self.symbols.get(name).and_then(|entries| {
            entries.iter().find(|entry| {
                // Check visibility rules
                self.is_visible(entry, from_module)
            })
        })
    }

    /// Check if symbol is visible from a module
    /// Complexity: 5 (path comparison)
    fn is_visible(&self, entry: &SymbolEntry, from_module: &str) -> bool {
        match entry.symbol.visibility {
            Visibility::Public => true,
            Visibility::Private => {
                // Only visible in same module
                entry.module_path == from_module
            }
            Visibility::Protected => {
                // Visible in same module or submodules
                from_module.starts_with(&entry.module_path)
            }
        }
    }

    /// Get all symbols for a file
    /// Complexity: 2
    pub fn get_file_symbols(&self, path: &Path) -> Vec<&SymbolEntry> {
        self.file_symbols
            .get(path)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| {
                        self.symbols
                            .get(name)
                            .and_then(|entries| entries.iter().find(|e| e.file_path == path))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Increment usage count for a symbol
    /// Complexity: 3
    pub fn increment_usage(&mut self, name: &str, from_module: &str) {
        if let Some(entries) = self.symbols.get_mut(name) {
            for entry in entries.iter_mut() {
                // Check visibility without borrowing self
                let is_visible = match entry.symbol.visibility {
                    Visibility::Public => true,
                    Visibility::Private => entry.module_path == from_module,
                    Visibility::Protected => from_module.starts_with(&entry.module_path),
                };

                if is_visible {
                    entry.usage_count += 1;
                    break;
                }
            }
        }
    }

    /// Get total symbol count
    /// Complexity: 1
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty
    /// Complexity: 1
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_insertion() {
        let mut table = SymbolTable::new();

        let entry = SymbolEntry {
            symbol: Symbol {
                name: "test_func".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Public,
                line: 10,
            },
            file_path: PathBuf::from("src/lib.rs"),
            module_path: "lib".to_string(),
            usage_count: 0,
            is_exported: true,
        };

        table.insert("test_func".to_string(), entry);

        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());
    }

    #[test]
    fn test_symbol_resolution_cross_module() {
        let mut table = SymbolTable::new();

        // Public symbol in module A
        let public_entry = SymbolEntry {
            symbol: Symbol {
                name: "public_func".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Public,
                line: 10,
            },
            file_path: PathBuf::from("src/mod_a.rs"),
            module_path: "mod_a".to_string(),
            usage_count: 0,
            is_exported: true,
        };

        // Private symbol in module A
        let private_entry = SymbolEntry {
            symbol: Symbol {
                name: "private_func".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Private,
                line: 20,
            },
            file_path: PathBuf::from("src/mod_a.rs"),
            module_path: "mod_a".to_string(),
            usage_count: 0,
            is_exported: false,
        };

        table.insert("public_func".to_string(), public_entry);
        table.insert("private_func".to_string(), private_entry);

        // Public symbol visible from module B
        let resolved = table.resolve("public_func", "mod_b");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().symbol.name, "public_func");

        // Private symbol NOT visible from module B
        let resolved = table.resolve("private_func", "mod_b");
        assert!(resolved.is_none());

        // Private symbol IS visible from same module
        let resolved = table.resolve("private_func", "mod_a");
        assert!(resolved.is_some());
    }

    #[test]
    fn test_visibility_rules() {
        let mut table = SymbolTable::new();

        // Protected symbol
        let protected_entry = SymbolEntry {
            symbol: Symbol {
                name: "protected_func".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Protected,
                line: 30,
            },
            file_path: PathBuf::from("src/parent.rs"),
            module_path: "parent".to_string(),
            usage_count: 0,
            is_exported: true,
        };

        table.insert("protected_func".to_string(), protected_entry);

        // Visible from submodule
        let resolved = table.resolve("protected_func", "parent::child");
        assert!(resolved.is_some());

        // Not visible from sibling module
        let resolved = table.resolve("protected_func", "sibling");
        assert!(resolved.is_none());
    }

    #[test]
    fn test_usage_count_tracking() {
        let mut table = SymbolTable::new();

        let entry = SymbolEntry {
            symbol: Symbol {
                name: "tracked_func".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Public,
                line: 10,
            },
            file_path: PathBuf::from("src/lib.rs"),
            module_path: "lib".to_string(),
            usage_count: 0,
            is_exported: true,
        };

        table.insert("tracked_func".to_string(), entry);

        // Increment usage
        table.increment_usage("tracked_func", "other_mod");
        table.increment_usage("tracked_func", "another_mod");

        let resolved = table.resolve("tracked_func", "lib").unwrap();
        assert_eq!(resolved.usage_count, 2);
    }

    #[test]
    fn test_file_symbols_retrieval() {
        let mut table = SymbolTable::new();
        let path = PathBuf::from("src/test.rs");

        // Add multiple symbols from same file
        for i in 0..3 {
            let entry = SymbolEntry {
                symbol: Symbol {
                    name: format!("func_{}", i),
                    kind: SymbolKind::Function,
                    visibility: Visibility::Public,
                    line: i * 10,
                },
                file_path: path.clone(),
                module_path: "test".to_string(),
                usage_count: 0,
                is_exported: true,
            };
            table.insert(format!("func_{}", i), entry);
        }

        let file_symbols = table.get_file_symbols(&path);
        assert_eq!(file_symbols.len(), 3);
    }
}
