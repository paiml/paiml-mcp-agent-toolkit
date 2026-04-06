// FFIReferenceTracker implementation methods
// Included from dead_code_prover.rs - do NOT add `use` imports or `#!` attributes here.

impl FFIReferenceTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            no_mangle_symbols: HashSet::new(),
            export_name_symbols: HashMap::new(),
            extern_c_functions: HashSet::new(),
            wasm_exports: HashSet::new(),
            python_exports: HashSet::new(),
        }
    }

    fn lookahead_function(&self, lines: &[&str], line_num: usize, max_offset: usize) -> Option<(String, usize)> {
        for offset in 1..=max_offset {
            if let Some(next_line) = lines.get(line_num + offset) {
                if let Some(func_name) = self.extract_function_name_from_line(next_line) {
                    return Some((func_name, line_num + offset + 1));
                }
            }
        }
        None
    }

    /// Scan AST for FFI exports
    pub fn scan_for_ffi_exports(&mut self, content: &str, file_path: &str) {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed == "#[no_mangle]" {
                if let Some((func_name, ln)) = self.lookahead_function(&lines, line_num, 3) {
                    self.no_mangle_symbols.insert(SymbolId {
                        file_path: file_path.to_string(), function_name: func_name, line_number: ln,
                    });
                }
            }

            if let Some(export_name) = self.extract_export_name(trimmed) {
                if let Some((func_name, ln)) = self.lookahead_function(&lines, line_num, 3) {
                    self.export_name_symbols.insert(SymbolId {
                        file_path: file_path.to_string(), function_name: func_name, line_number: ln,
                    }, export_name);
                }
            }

            if trimmed.contains("extern \"C\"") && trimmed.contains("fn ") {
                if let Some(func_name) = self.extract_function_name_from_line(trimmed) {
                    self.extern_c_functions.insert(SymbolId {
                        file_path: file_path.to_string(), function_name: func_name, line_number: line_num + 1,
                    });
                }
            }

            if trimmed == "#[wasm_bindgen]" {
                if let Some((func_name, ln)) = self.lookahead_function(&lines, line_num, 1) {
                    self.wasm_exports.insert(SymbolId {
                        file_path: file_path.to_string(), function_name: func_name, line_number: ln,
                    });
                }
            }

            if trimmed.starts_with("#[pyfunction") {
                if let Some((func_name, ln)) = self.lookahead_function(&lines, line_num, 1) {
                    self.python_exports.insert(SymbolId {
                        file_path: file_path.to_string(), function_name: func_name, line_number: ln,
                    });
                }
            }
        }
    }

    pub(crate) fn extract_function_name_from_line(&self, line: &str) -> Option<String> {
        // Extract function name from "pub fn name(" or "fn name("
        if let Some(fn_pos) = line.find("fn ") {
            let after_fn = line.get(fn_pos + 3..).unwrap_or_default();
            if let Some(paren_pos) = after_fn.find('(') {
                let name = after_fn.get(..paren_pos).unwrap_or_default().trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    fn extract_export_name(&self, line: &str) -> Option<String> {
        // Extract name from #[export_name = "custom_name"]
        if line.starts_with("#[export_name") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.get(start + 1..).unwrap_or_default().find('"') {
                    return Some(
                        line.get(start + 1..start + 1 + end)
                            .unwrap_or_default()
                            .to_string(),
                    );
                }
            }
        }
        None
    }

    /// Check if a symbol is externally visible
    #[must_use]
    pub fn is_externally_visible(&self, symbol: &SymbolId) -> bool {
        self.no_mangle_symbols.contains(symbol)
            || self.export_name_symbols.contains_key(symbol)
            || self.extern_c_functions.contains(symbol)
            || self.wasm_exports.contains(symbol)
            || self.python_exports.contains(symbol)
    }

    /// Get count of detected FFI exports for testing
    #[must_use]
    pub fn ffi_export_count(&self) -> usize {
        self.no_mangle_symbols.len()
            + self.export_name_symbols.len()
            + self.extern_c_functions.len()
            + self.wasm_exports.len()
            + self.python_exports.len()
    }
}
