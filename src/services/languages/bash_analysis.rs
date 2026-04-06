// BashScriptAnalyzer implementation methods
// Provides shell function extraction, variable analysis, command parsing, and control flow detection.

impl BashScriptAnalyzer {
    /// Creates a new Bash script analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            script_name: file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            function_count: 0,
            variable_count: 0,
            command_count: 0,
        }
    }

    /// Analyzes Bash script and extracts AST items (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_bash_script(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_shell_functions(source)?;
        self.extract_variables(source)?;
        self.analyze_commands(source)?;
        self.extract_control_flow(source)?;

        Ok(self.items)
    }

    /// Extracts function definitions from shell script (complexity ≤10)
    fn extract_shell_functions(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.ends_with("() {") || trimmed.contains("function ") {
                let func_name = self.extract_function_name(trimmed)?;
                let qualified_name = self.get_qualified_name(&func_name);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility: "public".to_string(),
                    is_async: false,
                    line: line_num + 1,
                });
                self.function_count += 1;
            }
        }
        Ok(())
    }

    /// Extracts variable declarations and usage (complexity ≤10)
    fn extract_variables(&mut self, source: &str) -> Result<(), String> {
        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.contains('=') && !trimmed.starts_with('#') {
                let parts: Vec<&str> = trimmed.split('=').collect();
                if parts.len() >= 2 {
                    self.variable_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Analyzes command invocations and pipelines (complexity ≤10)
    fn analyze_commands(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("#!/") {
                // Treat basic commands as functions for AST structure
                if let Some(cmd) = trimmed.split_whitespace().next() {
                    if !cmd.contains('=') && !cmd.ends_with('{') {
                        // Not assignments or function defs
                        let qualified_name = self.get_qualified_name(cmd);
                        self.items.push(AstItem::Function {
                            name: qualified_name,
                            visibility: "public".to_string(),
                            is_async: false,
                            line: line_num + 1,
                        });
                    }
                }

                if trimmed.contains('|') {
                    self.command_count += 2; // Count pipeline as multiple commands
                } else {
                    self.command_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts control flow structures (complexity ≤10)
    fn extract_control_flow(&mut self, source: &str) -> Result<(), String> {
        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("if ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("case ")
            {
                // Control flow statements found - could extract more details
                // For now, just noting their presence
            }
        }
        Ok(())
    }

    /// Extracts function name from shell line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Result<String, String> {
        if let Some(pos) = line.find("() {") {
            let name_part = &line[..pos];
            Ok(name_part.trim().to_string())
        } else if line.contains("function ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Ok(parts[1].to_string())
            } else {
                Err("Invalid function declaration".to_string())
            }
        } else {
            Err("Invalid function format".to_string())
        }
    }

    /// Gets qualified name for shell symbol (complexity ≤10)
    fn get_qualified_name(&self, symbol_name: &str) -> String {
        if self.script_name.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{}::{}", self.script_name, symbol_name)
        }
    }
}
