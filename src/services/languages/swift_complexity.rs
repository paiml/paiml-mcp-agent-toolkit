// SwiftComplexityAnalyzer implementation methods
// Provides Swift-specific cyclomatic and cognitive complexity analysis.

impl SwiftComplexityAnalyzer {
    /// Creates a new Swift complexity analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of Swift source (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("if ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("switch ")
                || trimmed.starts_with("case ")
                || trimmed.starts_with("guard ")
                || trimmed.contains("} else if ")
                || trimmed.contains("else if ")
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
