// Symbol parsing methods for DependencyGraphBuilder
// Extracted from builder.rs

impl DependencyGraphBuilder {
    /// Build symbol table for a file
    /// Complexity: 9 (file read + parsing)
    fn build_file_symbols(&mut self, path: &Path) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
}
