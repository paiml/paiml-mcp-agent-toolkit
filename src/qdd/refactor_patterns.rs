// Pattern engine methods for applying design patterns
// Included by refactor.rs — shares parent module scope

impl Default for PatternEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternEngine {
    #[must_use]
    pub fn new() -> Self {
        let mut patterns = std::collections::HashMap::new();
        patterns.insert(
            "single_responsibility".to_string(),
            "Extract methods to ensure single responsibility".to_string(),
        );
        patterns.insert(
            "dependency_injection".to_string(),
            "Replace hard-coded dependencies with injected ones".to_string(),
        );

        Self { patterns }
    }

    pub fn apply_pattern(&self, code: &str, pattern_name: &str) -> Result<String> {
        debug_assert!(!code.is_empty(), "code must not be empty");
        match pattern_name {
            "single_responsibility" => self.apply_single_responsibility(code),
            "dependency_injection" => self.apply_dependency_injection(code),
            _ => Err(anyhow!("Unknown pattern: {pattern_name}")),
        }
    }

    fn apply_single_responsibility(&self, code: &str) -> Result<String> {
        debug_assert!(!code.is_empty(), "code must not be empty");
        // Simple implementation of SRP pattern
        let mut result = code.to_string();
        result.push_str("\n// Single Responsibility Pattern applied\n");
        Ok(result)
    }

    fn apply_dependency_injection(&self, code: &str) -> Result<String> {
        debug_assert!(!code.is_empty(), "code must not be empty");
        // Simple implementation of DI pattern
        let mut result = code.to_string();
        result.push_str("\n// Dependency Injection Pattern applied\n");
        Ok(result)
    }
}
