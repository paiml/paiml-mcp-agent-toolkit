// ShellSafetyAnalyzer and ShellCommandParser implementation methods
// Provides safety violation detection, security vulnerability checks, best practice
// validation, command line parsing, and variable assignment extraction.

impl ShellSafetyAnalyzer {
    /// Creates a new shell safety analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            safety_violations: Vec::new(),
            best_practice_warnings: Vec::new(),
        }
    }

    /// Analyzes shell script for safety issues (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_safety(&mut self, source: &str) -> Result<Vec<String>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut violations = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.contains("rm -rf $") {
                violations.push("Dangerous rm -rf with variable".to_string());
            }
            if trimmed.contains("eval \"$") {
                violations.push("Dangerous eval with user input".to_string());
            }
            if trimmed.contains("$@") && !trimmed.contains("\"$@\"") {
                violations.push("Unquoted $@ parameter expansion".to_string());
            }
        }

        self.safety_violations = violations.clone();
        Ok(violations)
    }

    /// Checks for common security vulnerabilities (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn check_security_vulnerabilities(&mut self, source: &str) -> Result<Vec<String>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut vulnerabilities = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.contains("curl") && !trimmed.contains("--fail") {
                vulnerabilities.push("curl without --fail may ignore errors".to_string());
            }
            if trimmed.contains("wget") && !trimmed.contains("-O") {
                vulnerabilities.push("wget without explicit output may overwrite".to_string());
            }
        }

        Ok(vulnerabilities)
    }

    /// Validates best practices compliance (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate_best_practices(&mut self, source: &str) -> Result<Vec<String>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut warnings = Vec::new();

        let has_shebang = source.lines().next().unwrap_or("").starts_with("#!");
        if !has_shebang {
            warnings.push("Missing shebang line".to_string());
        }

        let has_set_flags = source.contains("set -e") || source.contains("set -u");
        if !has_set_flags {
            warnings.push("Consider using 'set -e' or 'set -u' for error handling".to_string());
        }

        self.best_practice_warnings = warnings.clone();
        Ok(warnings)
    }

    /// Gets safety violations
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_safety_violations(&self) -> &[String] {
        &self.safety_violations
    }

    /// Gets best practice warnings
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_best_practice_warnings(&self) -> &[String] {
        &self.best_practice_warnings
    }
}

impl ShellCommandParser {
    /// Creates a new shell command parser
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            variables: Vec::new(),
        }
    }

    /// Parses shell command line into tokens (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parse_command_line(&mut self, line: &str) -> Result<Vec<String>, String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let tokens: Vec<String> = line
            .split_whitespace()
            .map(std::string::ToString::to_string)
            .collect();

        self.commands.extend(tokens.clone());
        Ok(tokens)
    }

    /// Extracts variable assignments (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn extract_variable_assignments(
        &mut self,
        line: &str,
    ) -> Result<Vec<(String, String)>, String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let mut assignments = Vec::new();

        if line.contains('=') && !line.trim().starts_with('#') {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() >= 2 {
                let mut var_part = parts[0].trim();

                // Handle export keyword: "export VAR=value" -> "VAR"
                if var_part.starts_with("export ") {
                    var_part = &var_part[7..]; // Remove "export "
                }

                let var_name = var_part.trim().to_string();
                let var_value = parts[1].trim().to_string();
                assignments.push((var_name.clone(), var_value));
                self.variables.push(var_name);
            }
        }

        Ok(assignments)
    }

    /// Gets parsed commands
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_commands(&self) -> &[String] {
        &self.commands
    }

    /// Gets extracted variables
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_variables(&self) -> &[String] {
        &self.variables
    }
}
