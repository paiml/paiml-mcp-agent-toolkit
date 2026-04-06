impl SymbolTable {
    #[must_use]
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
    #[must_use]
    pub fn resolve_relative(&self, rel: &RelativeLocation, file: &Path) -> Option<Location> {
        debug_assert!(file.exists(), "file must exist: {}", file.display());
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
    #[must_use]
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

    /// Find all symbols within a span using binary search for efficiency
    #[must_use]
    pub fn symbols_in_span(&self, location: &Location) -> Vec<QualifiedName> {
        if let Some(spans) = self.span_index.get(&location.file_path) {
            // Binary search to find the first symbol that could be in our span
            let start_idx = match spans.binary_search_by_key(&location.span.start, |(pos, _)| *pos)
            {
                Ok(idx) => idx,
                Err(idx) => idx.saturating_sub(1), // Check one before insertion point
            };

            // Collect symbols starting from the binary search result
            let mut result = Vec::new();
            for i in start_idx..spans.len() {
                let (pos, qname) = &spans[i];
                if *pos > location.span.end {
                    break; // No more symbols can be in the span
                }
                if location.span.contains(*pos) {
                    result.push(qname.clone());
                }
            }
            result
        } else {
            Vec::new()
        }
    }

    /// Get the location of a qualified name
    #[must_use]
    pub fn get_location(&self, qualified_name: &QualifiedName) -> Option<Location> {
        self.symbols.get(qualified_name).map(|entry| entry.clone())
    }

    /// Get all symbols in the table
    #[must_use]
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
    #[must_use]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the symbol table is empty
    #[must_use]
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
        debug_assert!(file.exists(), "file must exist: {}", file.display());
        debug_assert!(!name.is_empty(), "name must not be empty");
        let module_path = match module {
            Some(explicit_module) => self.parse_explicit_module(explicit_module),
            None => self.infer_module_from_file_path(file),
        };

        Some(QualifiedName::new(module_path, name.to_string()))
    }

    /// Parse explicitly provided module path
    fn parse_explicit_module(&self, module: &str) -> Vec<String> {
        module
            .split("::")
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Infer module path from file system structure
    fn infer_module_from_file_path(&self, file: &Path) -> Vec<String> {
        debug_assert!(file.exists(), "file must exist: {}", file.display());
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
        debug_assert!(file.exists(), "file must exist: {}", file.display());
        file.file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|&stem_str| !matches!(stem_str, "mod" | "lib" | "main"))
            .map(std::string::ToString::to_string)
    }

    /// Add parent directories to module path, stopping at src directory
    fn add_parent_directories_to_module_path(&self, file: &Path, module_path: &mut Vec<String>) {
        debug_assert!(file.exists(), "file must exist: {}", file.display());
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        path.file_name()
            .and_then(|name| name.to_str())
            .map(std::string::ToString::to_string)
    }
}
