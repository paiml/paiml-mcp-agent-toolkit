// Dependency graph builder with language-specific parsers
// Complexity: All functions ≤ 10
// SATD: Zero tolerance

use super::*;
use super::symbol_table::{SymbolTable, SymbolEntry};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use rustc_hash::FxHashMap;
use petgraph::graph::NodeIndex;

pub struct DependencyGraphBuilder {
    graph: DependencyGraph,
    symbol_table: SymbolTable,
    node_map: FxHashMap<PathBuf, NodeIndex>,
    /// Track processed files for incremental updates
    processed_hashes: FxHashMap<PathBuf, u64>,
}

impl DependencyGraphBuilder {
    /// Create new builder
    /// Complexity: 1
    pub fn new() -> Self {
        DependencyGraphBuilder {
            graph: DependencyGraph::new(),
            symbol_table: SymbolTable::new(),
            node_map: FxHashMap::default(),
            processed_hashes: FxHashMap::default(),
        }
    }

    /// Build from workspace path
    /// Complexity: 8 (file collection + analysis loop)
    pub fn from_workspace(path: &Path) -> Result<Self> {
        let mut builder = Self::new();

        // Collect source files
        let files = builder.collect_source_files(path)?;

        // Phase 1: Build symbol table from all files
        for file_path in &files {
            builder.build_file_symbols(file_path)?;
        }

        // Phase 2: Analyze dependencies
        for file_path in &files {
            let node_id = builder.analyze_file(file_path)?;
            builder.resolve_file_dependencies(node_id, file_path)?;
        }

        Ok(builder)
    }

    /// Collect all source files from workspace
    /// Complexity: 6 (directory traversal)
    fn collect_source_files(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.collect_files_recursive(root, &mut files)?;
        Ok(files)
    }

    /// Recursively collect source files
    /// Complexity: 8 (recursive with early exit)
    fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip common non-source directories
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !dir_name.starts_with('.') && dir_name != "target" && dir_name != "node_modules" {
                    self.collect_files_recursive(&path, files)?;
                }
            } else if Self::is_source_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Check if file is a source file we can analyze
    /// Complexity: 3
    fn is_source_file(path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") | Some("py") | Some("js") | Some("jsx") |
            Some("ts") | Some("tsx") | Some("go") | Some("java") |
            Some("c") | Some("cpp") | Some("cc") | Some("h") | Some("hpp") => true,
            _ => false,
        }
    }

    /// Build symbol table for a file
    /// Complexity: 9 (file read + parsing)
    fn build_file_symbols(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let module_name = self.path_to_module(path);

        // Parse based on file extension
        let symbols = match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => self.parse_rust_symbols(&content)?,
            Some("py") => self.parse_python_symbols(&content)?,
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") => {
                self.parse_typescript_symbols(&content)?
            }
            _ => vec![], // Skip unsupported files
        };

        // Add symbols to table
        for symbol in symbols {
            let entry = SymbolEntry {
                symbol: symbol.clone(),
                file_path: path.to_path_buf(),
                module_path: module_name.clone(),
                usage_count: 0,
                is_exported: matches!(symbol.visibility, Visibility::Public),
            };
            self.symbol_table.insert(symbol.name.clone(), entry);
        }

        Ok(())
    }

    /// Analyze single file and create node
    /// Complexity: 10 (parsing + node creation)
    fn analyze_file(&mut self, path: &Path) -> Result<NodeIndex> {
        let content = fs::read_to_string(path)?;

        // Calculate hash for incremental updates
        let hash = self.calculate_hash(&content);

        // Check if already processed with same hash
        if let Some(&existing_hash) = self.processed_hashes.get(path) {
            if existing_hash == hash {
                // Skip if unchanged
                return Ok(*self.node_map.get(path).unwrap());
            }
        }

        // Create node data
        let node_data = NodeData {
            path: path.to_path_buf(),
            module: self.path_to_module(path),
            symbols: self.symbol_table.get_file_symbols(path)
                .iter()
                .map(|e| e.symbol.clone())
                .collect(),
            loc: content.lines().count(),
            complexity: self.estimate_complexity(&content),
            ast_hash: hash,
        };

        // Add or update node
        let node_id = if let Some(&existing_id) = self.node_map.get(path) {
            *self.graph.node_weight_mut(existing_id).unwrap() = node_data;
            existing_id
        } else {
            let id = self.graph.add_node(node_data);
            self.node_map.insert(path.to_path_buf(), id);
            id
        };

        self.processed_hashes.insert(path.to_path_buf(), hash);
        Ok(node_id)
    }

    /// Resolve dependencies for a file
    /// Complexity: 9 (import parsing + edge creation)
    fn resolve_file_dependencies(&mut self, node_id: NodeIndex, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;

        let imports = match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => self.parse_rust_imports(&content)?,
            Some("py") => self.parse_python_imports(&content)?,
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") => {
                self.parse_typescript_imports(&content)?
            }
            _ => vec![],
        };

        // Create edges for each import
        for import in imports {
            if let Some(target_node) = self.resolve_import_to_node(&import) {
                let edge = EdgeData::Import {
                    weight: 1.0,
                    visibility: Visibility::Public,
                };
                self.graph.add_edge(node_id, target_node, edge);
            }
        }

        Ok(())
    }

    /// Parse Rust symbols
    /// Complexity: 8
    fn parse_rust_symbols(&self, content: &str) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();

        // Simple regex-based parsing for MVP
        // Will enhance with syn in production
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("pub fn ") {
                if let Some(name) = Self::extract_function_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        visibility: Visibility::Public,
                        line: line_num,
                    });
                }
            } else if trimmed.starts_with("fn ") {
                if let Some(name) = Self::extract_function_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        visibility: Visibility::Private,
                        line: line_num,
                    });
                }
            } else if trimmed.starts_with("pub struct ") {
                if let Some(name) = Self::extract_type_name(trimmed, "struct") {
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Struct,
                        visibility: Visibility::Public,
                        line: line_num,
                    });
                }
            }
        }

        Ok(symbols)
    }

    /// Parse Python symbols
    /// Complexity: 7
    fn parse_python_symbols(&self, content: &str) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("def ") {
                if let Some(name) = Self::extract_python_function_name(trimmed) {
                    let visibility = if name.starts_with('_') {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    };

                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        visibility,
                        line: line_num,
                    });
                }
            } else if trimmed.starts_with("class ") {
                if let Some(name) = Self::extract_python_class_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Struct,
                        visibility: Visibility::Public,
                        line: line_num,
                    });
                }
            }
        }

        Ok(symbols)
    }

    /// Parse TypeScript/JavaScript symbols
    /// Complexity: 8
    fn parse_typescript_symbols(&self, content: &str) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("export function ") || trimmed.starts_with("export const ") {
                if let Some(name) = Self::extract_ts_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        visibility: Visibility::Public,
                        line: line_num,
                    });
                }
            } else if trimmed.starts_with("function ") || trimmed.starts_with("const ") {
                if let Some(name) = Self::extract_ts_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        visibility: Visibility::Private,
                        line: line_num,
                    });
                }
            } else if trimmed.starts_with("export class ") {
                if let Some(name) = Self::extract_ts_class_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Struct,
                        visibility: Visibility::Public,
                        line: line_num,
                    });
                }
            }
        }

        Ok(symbols)
    }

    /// Parse Rust imports
    /// Complexity: 6
    fn parse_rust_imports(&self, content: &str) -> Result<Vec<String>> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") && trimmed.ends_with(';') {
                let import = trimmed
                    .strip_prefix("use ")
                    .and_then(|s| s.strip_suffix(';'))
                    .unwrap_or("")
                    .trim();
                imports.push(import.to_string());
            }
        }

        Ok(imports)
    }

    /// Parse Python imports
    /// Complexity: 5
    fn parse_python_imports(&self, content: &str) -> Result<Vec<String>> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                imports.push(trimmed.to_string());
            }
        }

        Ok(imports)
    }

    /// Parse TypeScript/JavaScript imports
    /// Complexity: 6
    fn parse_typescript_imports(&self, content: &str) -> Result<Vec<String>> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                if let Some(end) = trimmed.rfind(" from ") {
                    let module = trimmed[end + 6..].trim_matches(|c| c == '\'' || c == '"' || c == ';');
                    imports.push(module.to_string());
                }
            } else if trimmed.starts_with("const ") && trimmed.contains(" = require(") {
                if let Some(start) = trimmed.find("require('") {
                    if let Some(end) = trimmed[start + 9..].find('\'') {
                        let module = &trimmed[start + 9..start + 9 + end];
                        imports.push(module.to_string());
                    }
                }
            }
        }

        Ok(imports)
    }

    /// Helper functions for parsing
    /// Complexity: 2 each
    fn extract_function_name(line: &str) -> Option<&str> {
        line.split_whitespace()
            .skip_while(|&w| w == "pub" || w == "fn")
            .next()
            .and_then(|s| s.split('(').next())
    }

    fn extract_type_name<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
        line.split_whitespace()
            .skip_while(|&w| w == "pub" || w == keyword)
            .next()
            .and_then(|s| s.split('{').next())
            .and_then(|s| s.split('<').next())
    }

    fn extract_python_function_name(line: &str) -> Option<&str> {
        line.strip_prefix("def ")
            .and_then(|s| s.split('(').next())
            .map(|s| s.trim())
    }

    fn extract_python_class_name(line: &str) -> Option<&str> {
        line.strip_prefix("class ")
            .and_then(|s| s.split('(').next())
            .and_then(|s| s.split(':').next())
            .map(|s| s.trim())
    }

    fn extract_ts_name(line: &str) -> Option<&str> {
        line.split_whitespace()
            .skip_while(|&w| w == "export" || w == "const" || w == "function")
            .next()
            .and_then(|s| s.split('(').next())
            .and_then(|s| s.split('=').next())
            .map(|s| s.trim())
    }

    fn extract_ts_class_name(line: &str) -> Option<&str> {
        line.split_whitespace()
            .skip_while(|&w| w == "export" || w == "class")
            .next()
            .and_then(|s| s.split('{').next())
            .map(|s| s.trim())
    }

    /// Convert path to module name
    /// Complexity: 4
    fn path_to_module(&self, path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Calculate content hash
    /// Complexity: 2
    fn calculate_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Estimate complexity from content
    /// Complexity: 5
    fn estimate_complexity(&self, content: &str) -> f64 {
        let mut complexity = 1.0;

        for line in content.lines() {
            let trimmed = line.trim();
            // Count control flow keywords
            if trimmed.starts_with("if ") || trimmed.starts_with("else") {
                complexity += 1.0;
            } else if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
                complexity += 2.0;
            } else if trimmed.starts_with("match ") || trimmed.starts_with("switch ") {
                complexity += 1.5;
            }
        }

        complexity
    }

    /// Resolve import string to node
    /// Complexity: 4
    fn resolve_import_to_node(&self, import: &str) -> Option<NodeIndex> {
        // Try to find matching module in node_map
        for (path, &node_id) in &self.node_map {
            let module_name = self.path_to_module(path);
            if import.contains(&module_name) {
                return Some(node_id);
            }
        }
        None
    }

    /// Build the final graph
    /// Complexity: 1
    pub fn build(self) -> Result<DependencyGraph> {
        Ok(self.graph)
    }

    /// Get symbol table
    /// Complexity: 1
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_source_file() {
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.rs")));
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.py")));
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.ts")));
        assert!(!DependencyGraphBuilder::is_source_file(Path::new("test.txt")));
        assert!(!DependencyGraphBuilder::is_source_file(Path::new("README.md")));
    }

    #[test]
    fn test_extract_function_names() {
        assert_eq!(
            DependencyGraphBuilder::extract_function_name("pub fn test_func() {"),
            Some("test_func")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_function_name("fn private_func(arg: i32) {"),
            Some("private_func")
        );
    }

    #[test]
    fn test_extract_python_names() {
        assert_eq!(
            DependencyGraphBuilder::extract_python_function_name("def test_func():"),
            Some("test_func")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_python_class_name("class TestClass(BaseClass):"),
            Some("TestClass")
        );
    }

    #[test]
    fn test_builder_creation() {
        let builder = DependencyGraphBuilder::new();
        assert!(builder.symbol_table.is_empty());
        assert_eq!(builder.graph.node_count(), 0);
        assert_eq!(builder.graph.edge_count(), 0);
    }
}