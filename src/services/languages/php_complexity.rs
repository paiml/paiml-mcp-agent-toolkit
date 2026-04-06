// PHP complexity analyzer
// Included from php.rs - shares parent module scope

impl Default for PhpComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpComplexityAnalyzer {
    /// Creates a new PHP complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of PHP script (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("if ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("foreach ")
                || trimmed.starts_with("switch ")
                || trimmed.starts_with("case ")
                || trimmed.starts_with("elseif ")
                || trimmed.contains("} elseif ")
            {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1;
            }

            // Count ternary operators
            if trimmed.contains('?') && trimmed.contains(':') {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1;
            }
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }
}
