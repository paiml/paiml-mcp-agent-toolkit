#[cfg(feature = "scala-ast")]
impl Default for ScalaComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "scala-ast")]
impl ScalaComplexityAnalyzer {
    /// Creates a new Scala complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of Scala source code (complexity ≤10)
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
        // Control flow increases cyclomatic complexity
        if line.contains("if ") || line.contains(" if ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }

        // Match expressions
        if line.contains("match ") || line.contains(" match ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }

        // Case patterns
        if line.contains("case ") && !line.contains("case class ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }

        // Logical operators increase complexity
        if line.contains(" && ") || line.contains(" || ") {
            self.cyclomatic_complexity += 1;
        }

        // Loops
        if line.contains(" for ") || line.contains("while ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }

        // Try/catch blocks
        if line.contains("try ") || line.contains("catch ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
    }
}
