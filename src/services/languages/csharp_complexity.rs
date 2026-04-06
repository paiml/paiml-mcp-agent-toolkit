// CSharpComplexityAnalyzer - included from csharp.rs

/// C# complexity analyzer for extracting C#-specific metrics (complexity ≤10)
#[cfg(feature = "csharp-ast")]
pub struct CSharpComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "csharp-ast")]
impl Default for CSharpComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "csharp-ast")]
impl CSharpComplexityAnalyzer {
    /// Creates a new C# complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of C# source code (complexity ≤10)
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
        if line.contains("if ")
            || line.contains("while ")
            || line.contains("for ")
            || line.contains("foreach ")
        {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        if line.contains("&&") || line.contains("||") {
            self.cyclomatic_complexity += 1;
        }
        if line.contains("case ") || line.contains("catch ") || line.contains("switch ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
    }
}
