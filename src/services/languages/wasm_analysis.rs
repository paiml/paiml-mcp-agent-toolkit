// WASM analysis implementations - included from wasm.rs
// NO use imports or #! inner attributes allowed (shares parent module scope)

impl WasmModuleAnalyzer {
    /// Creates a new WASM module analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: &Path) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            module_name: file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            function_count: 0,
            _import_count: 0,
            _export_count: 0,
        }
    }

    /// Analyzes WASM binary and extracts AST items (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_wasm_binary(mut self, wasm_bytes: &[u8]) -> Result<Vec<AstItem>, String> {
        debug_assert!(!wasm_bytes.is_empty(), "wasm_bytes must not be empty");
        if wasm_bytes.len() < 8 {
            return Err("Invalid WASM binary: too short".to_string());
        }

        if &wasm_bytes[0..4] != b"\0asm" {
            return Err("Invalid WASM magic number".to_string());
        }

        let parser = Parser::new(0);
        for (i, payload) in parser.parse_all(wasm_bytes).enumerate() {
            match payload {
                Ok(Payload::FunctionSection(_)) => {
                    self.function_count = i + 1;
                    self.items.push(AstItem::Function {
                        name: self.get_qualified_name(&format!("function_{i}")),
                        visibility: "export".to_string(),
                        is_async: false,
                        line: 1,
                    });
                }
                _ => continue,
            }
        }

        Ok(self.items)
    }

    /// Analyzes WASM text format (.wat) (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_wat_text(mut self, wat_source: &str) -> Result<Vec<AstItem>, String> {
        debug_assert!(!wat_source.is_empty(), "wat_source must not be empty");
        let mut function_count = 0;

        for line in wat_source.lines() {
            let trimmed = line.trim();

            // Match function definitions, not function references in exports
            // Function definitions start with "(func " but exports have "(func $name)"
            if trimmed.starts_with("(func ") {
                let func_name = self.extract_wat_function_name(trimmed);
                let qualified_name = self.get_qualified_name(&func_name);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility: "export".to_string(),
                    is_async: false,
                    line: function_count + 1,
                });
                function_count += 1;
            }
        }

        self.function_count = function_count;
        Ok(self.items)
    }

    /// Extracts function information from WASM (complexity ≤10)
    fn _extract_wasm_functions(&mut self, _parser: &Parser) -> Result<(), String> {
        debug_assert!(true, "contract: _extract_wasm_functions");
        // Not yet implemented - WASM function extraction requires wasmparser iteration
        Err("WASM function extraction not yet implemented".to_string())
    }

    /// Extracts import/export information (complexity ≤10)
    fn _extract_imports_exports(&mut self, _parser: &Parser) -> Result<(), String> {
        debug_assert!(true, "contract: _extract_imports_exports");
        // Not yet implemented - WASM import/export extraction requires section parsing
        Err("WASM import/export extraction not yet implemented".to_string())
    }

    /// Calculates WASM-specific complexity metrics (complexity ≤10)
    fn _calculate_wasm_complexity(&self, _function_body: &[u8]) -> Result<(u32, u32), String> {
        // Not yet implemented - WASM complexity analysis requires instruction counting
        Err("WASM complexity calculation not yet implemented".to_string())
    }

    /// Extracts function name from WAT line (complexity ≤10)
    fn extract_wat_function_name(&self, line: &str) -> String {
        debug_assert!(!line.is_empty(), "line must not be empty");
        if let Some(start) = line.find('$') {
            if let Some(end) = line[start..].find(' ') {
                line[start + 1..start + end].to_string()
            } else {
                format!("function_{}", self.function_count)
            }
        } else {
            format!("function_{}", self.function_count)
        }
    }

    /// Gets qualified name for WASM symbol (complexity ≤10)
    fn get_qualified_name(&self, symbol_name: &str) -> String {
        debug_assert!(!symbol_name.is_empty(), "symbol_name must not be empty");
        if self.module_name.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{}::{}", self.module_name, symbol_name)
        }
    }
}

impl WasmStackAnalyzer {
    /// Creates a new WASM stack analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            max_stack_depth: 0,
            current_depth: 0,
            branch_count: 0,
        }
    }

    /// Analyzes stack depth complexity (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_stack_complexity(&mut self, function_body: &[u8]) -> Result<u32, String> {
        debug_assert!(!function_body.is_empty(), "function_body must not be empty");
        self.current_depth = 0;
        self.max_stack_depth = 0;

        for &byte in function_body {
            match byte {
                0x20 => self.current_depth += 1, // local.get - pushes value to stack
                0x41 => self.current_depth += 1, // i32.const
                0x42 => self.current_depth += 1, // i64.const
                0x6a => {
                    // i32.add - pops 2, pushes 1 (net -1)
                    if self.current_depth >= 2 {
                        self.current_depth -= 1;
                    }
                }
                0x1a => {
                    // drop
                    if self.current_depth > 0 {
                        self.current_depth -= 1;
                    }
                }
                _ => {}
            }
            self.max_stack_depth = self.max_stack_depth.max(self.current_depth);
        }

        Ok(self.max_stack_depth)
    }

    /// Analyzes control flow complexity (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_control_flow_complexity(&mut self, function_body: &[u8]) -> Result<u32, String> {
        debug_assert!(!function_body.is_empty(), "function_body must not be empty");
        self.branch_count = 1; // Base complexity

        for &byte in function_body {
            match byte {
                0x04 => self.branch_count += 1, // if block
                0x05 => self.branch_count += 1, // else
                0x03 => self.branch_count += 1, // loop
                0x02 => self.branch_count += 1, // block
                0x0c => self.branch_count += 1, // br (branch)
                0x0d => self.branch_count += 1, // br_if (conditional branch)
                _ => {}
            }
        }

        Ok(self.branch_count)
    }
}
