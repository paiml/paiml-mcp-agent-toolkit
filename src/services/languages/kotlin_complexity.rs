#[cfg(feature = "kotlin-ast")]
impl Default for KotlinComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "kotlin-ast")]
impl KotlinComplexityAnalyzer {
    /// Creates a new Kotlin complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
            coroutine_complexity: 0,
        }
    }

    /// Analyzes complexity of Kotlin source code (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            self.analyze_complexity_for_line(trimmed);
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }

    /// Helper to analyze complexity for a single line (complexity ≤10)
    fn analyze_complexity_for_line(&mut self, line: &str) {
        debug_assert!(!line.is_empty(), "line must not be empty");
        if line.contains("if ") || line.contains("while ") || line.contains("for ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        if line.contains("&&") || line.contains("||") {
            self.cyclomatic_complexity += 1;
        }
        if line.contains("when ") || line.contains("catch ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
    }

    /// Analyzes coroutine complexity (complexity ≤10)
    pub fn analyze_coroutine_complexity(&mut self, source: &str) -> Result<u32, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.coroutine_complexity = 0;

        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains("suspend ")
                || trimmed.contains("async ")
                || trimmed.contains("launch ")
            {
                self.coroutine_complexity += 1;
            }
        }

        // Ensure at least complexity 1 if any coroutines detected
        if self.coroutine_complexity > 0 {
            self.coroutine_complexity = self.coroutine_complexity.max(1);
        }

        Ok(self.coroutine_complexity)
    }
}
