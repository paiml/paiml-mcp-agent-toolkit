// Java complexity analyzer implementation
// Included from java.rs - NO use imports or #! inner attributes

impl Default for JavaComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaComplexityAnalyzer {
    /// Creates a new Java complexity analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of Java source code (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
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
        if line.contains("if ") || line.contains("while ") || line.contains("for ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        if line.contains("&&") || line.contains("||") {
            self.cyclomatic_complexity += 1;
        }
        if line.contains("case ") || line.contains("catch ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
    }
}
