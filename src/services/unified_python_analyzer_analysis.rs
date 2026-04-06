impl UnifiedPythonAnalyzer {
    /// Analyze file with single parse
    ///
    /// This is the core GREEN phase implementation: minimal but correct.
    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError> {
        // Track parse count for testing
        #[cfg(test)]
        {
            self.parse_count.fetch_add(1, Ordering::SeqCst);
        }

        // 1. Read file content (single I/O operation)
        let content = tokio::fs::read_to_string(&self.file_path)
            .await
            .map_err(AnalysisError::Io)?;

        // 2. Parse ONCE with tree-sitter-python
        #[cfg(feature = "python-ast")]
        let tree = self.parse_python(&content)?;

        // 3. Extract AST items using tree-sitter
        #[cfg(feature = "python-ast")]
        let ast_items = self.extract_ast_items(&tree, &content);
        #[cfg(not(feature = "python-ast"))]
        let ast_items = Vec::new();

        // 4. Extract complexity metrics (minimal implementation for GREEN phase)
        let file_metrics = self.extract_complexity_metrics(&content);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    /// Extract complexity metrics from Python content
    ///
    /// GREEN PHASE: Minimal implementation using simple pattern counting.
    /// This will be enhanced in REFACTOR phase with proper complexity calculation.
    fn extract_complexity_metrics(&self, content: &str) -> FileComplexityMetrics {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut functions = Vec::new();

        // Count lines for rough estimation
        let lines = content.lines().count();

        // Simple function detection (GREEN phase - basic regex)
        let function_pattern =
            regex::Regex::new(r"(?m)^(?:async\s+)?def\s+(\w+)\s*\(").expect("static regex");

        for cap in function_pattern.captures_iter(content) {
            let name = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "anonymous".to_string());

            // Simple complexity: count control flow keywords
            let cyclomatic = self.estimate_complexity(content);

            functions.push(FunctionComplexity {
                name,
                line_start: 0, // Will be improved in REFACTOR
                line_end: 0,
                metrics: ComplexityMetrics {
                    cyclomatic: cyclomatic as u16,
                    cognitive: cyclomatic as u16, // Simplified for GREEN phase
                    nesting_max: 0,
                    lines: 10, // Rough estimate
                    halstead: None,
                },
            });
        }

        // Calculate file-level metrics
        let total_cyclomatic: u32 = functions.iter().map(|f| f.metrics.cyclomatic as u32).sum();

        let avg_cyclomatic = if functions.is_empty() {
            1
        } else {
            total_cyclomatic / functions.len() as u32
        };

        FileComplexityMetrics {
            path: self.file_path.display().to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: avg_cyclomatic as u16,
                cognitive: avg_cyclomatic as u16,
                nesting_max: 0,
                lines: lines as u16,
                halstead: None,
            },
            functions,
            classes: Vec::new(), // Will be extracted in REFACTOR phase
        }
    }

    /// Estimate complexity by counting control flow keywords
    /// GREEN PHASE: Simple pattern matching
    fn estimate_complexity(&self, content: &str) -> u32 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut complexity = 1; // Base complexity

        // Count control flow keywords
        let keywords = [
            "if ", "elif ", "for ", "while ", "try:", "except", "and ",
            "or ", // Logical operators
        ];

        for keyword in &keywords {
            complexity += content.matches(keyword).count() as u32;
        }

        complexity
    }
}
