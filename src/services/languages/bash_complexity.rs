// BashComplexityAnalyzer implementation methods
// Provides cyclomatic/cognitive complexity analysis, pipeline complexity, and conditional complexity.

impl BashComplexityAnalyzer {
    /// Creates a new Bash complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
            _nesting_depth: 0,
        }
    }

    /// Analyzes complexity of Bash script (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("if ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("case ")
                || trimmed.starts_with("elif ")
            {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1;
            }
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }

    /// Analyzes pipeline complexity (complexity ≤10)
    pub fn analyze_pipeline_complexity(&mut self, pipeline: &str) -> Result<u32, String> {
        let pipe_count = pipeline.matches('|').count();
        Ok(pipe_count as u32 + 1) // Base complexity of 1 plus number of pipes
    }

    /// Analyzes conditional complexity (complexity ≤10)
    pub fn analyze_conditional_complexity(&mut self, conditions: &str) -> Result<u32, String> {
        let mut complexity = 1;

        // Count logical operators
        complexity += conditions.matches(" && ").count() as u32;
        complexity += conditions.matches(" || ").count() as u32;
        complexity += conditions.matches(" -a ").count() as u32;
        complexity += conditions.matches(" -o ").count() as u32;

        Ok(complexity)
    }
}
