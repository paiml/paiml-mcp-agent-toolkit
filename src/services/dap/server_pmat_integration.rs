// DAP Server - PMAT Integration Methods
// Split from server.rs for file health compliance
//
// Contains: TRACE-004 DAP-PMAT integration (language detection, AST parsing, variable inspection)
// NOTE: This file is included via include!() - no `use` imports allowed

impl DapServer {
    // TRACE-004: DAP-PMAT Integration Methods

    /// Get the current detected language
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_language(&self) -> Option<Language> {
        let lang = self
            .current_language
            .lock()
            .expect("Mutex should not be poisoned");
        *lang
    }

    /// Check if AST is cached for a given file path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_ast_for(&self, path: &str) -> bool {
        debug_assert!(!path.is_empty(), "path must not be empty");
        let cache = self.ast_cache.lock().expect("Mutex should not be poisoned");
        cache.contains_key(Path::new(path))
    }

    /// Get variables at a specific line in a file using VariableInspector
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_variables_at_line(&self, path: &str, line: usize) -> Result<Vec<Variable>, String> {
        debug_assert!(!path.is_empty(), "path must not be empty");
        // Read file contents
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

        // Detect language from path
        let language = self
            .detect_language_from_path(Path::new(path))
            .ok_or_else(|| format!("Could not detect language for {}", path))?;

        // Use VariableInspector to extract variables
        match language {
            Language::Rust => self.variable_inspector.inspect_rust(&source, line),
            Language::TypeScript | Language::JavaScript => {
                self.variable_inspector.inspect_typescript(&source, line)
            }
            Language::Python => self.variable_inspector.inspect_python(&source, line),
            _ => Err(format!(
                "Language {:?} not supported for variable inspection",
                language
            )),
        }
    }

    /// Simulate stopping at a specific line (for testing)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn simulate_stop_at_line(&mut self, path: &str, line: usize) {
        debug_assert!(!path.is_empty(), "path must not be empty");
        let mut stopped_file = self
            .current_stopped_file
            .lock()
            .expect("Mutex should not be poisoned");
        *stopped_file = Some(path.to_string());
        drop(stopped_file);

        let mut stopped_line = self
            .current_stopped_line
            .lock()
            .expect("Mutex should not be poisoned");
        *stopped_line = Some(line);
    }

    /// Get current stopped file (TRACE-005)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_stopped_file(&self) -> Option<String> {
        self.current_stopped_file
            .lock()
            .expect("Mutex should not be poisoned")
            .clone()
    }

    /// Get current stopped line (TRACE-005)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_stopped_line(&self) -> Option<usize> {
        *self
            .current_stopped_line
            .lock()
            .expect("Mutex should not be poisoned")
    }

    /// Detect language from file path
    fn detect_language_from_path(&self, path: &Path) -> Option<Language> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let extension = path.extension()?.to_str()?;

        match extension {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" => Some(Language::JavaScript),
            _ => None,
        }
    }

    /// Parse and cache AST for a file
    fn parse_and_cache_ast(&self, path: &Path) -> Result<(), String> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Read source
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Detect language
        let language = self
            .detect_language_from_path(path)
            .ok_or_else(|| format!("Could not detect language for {:?}", path))?;

        // Parse using tree-sitter
        let tree = match language {
            Language::Rust => {
                let mut parser = tree_sitter::Parser::new();
                parser
                    .set_language(&tree_sitter_rust::LANGUAGE.into())
                    .map_err(|e| format!("Failed to set Rust language: {}", e))?;
                parser
                    .parse(&source, None)
                    .ok_or_else(|| "Failed to parse Rust source".to_string())?
            }
            Language::Python => {
                #[cfg(feature = "python-ast")]
                {
                    let mut parser = tree_sitter::Parser::new();
                    parser
                        .set_language(&tree_sitter_python::LANGUAGE.into())
                        .map_err(|e| format!("Failed to set Python language: {}", e))?;
                    parser
                        .parse(&source, None)
                        .ok_or_else(|| "Failed to parse Python source".to_string())?
                }
                #[cfg(not(feature = "python-ast"))]
                {
                    return Err("python-ast feature is disabled".to_string());
                }
            }
            Language::TypeScript | Language::JavaScript => {
                let mut parser = tree_sitter::Parser::new();
                parser
                    .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                    .map_err(|e| format!("Failed to set TypeScript language: {}", e))?;
                parser
                    .parse(&source, None)
                    .ok_or_else(|| "Failed to parse TypeScript source".to_string())?
            }
            _ => return Err(format!("Language {:?} not supported for parsing", language)),
        };

        // Cache the tree
        let mut cache = self.ast_cache.lock().expect("Mutex should not be poisoned");
        cache.insert(path.to_path_buf(), tree);

        Ok(())
    }
}
