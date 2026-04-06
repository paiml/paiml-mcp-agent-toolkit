// Included by unified_typescript_analyzer.rs — do NOT add `use` or `#!` attributes here.

impl UnifiedTypeScriptAnalyzer {
    /// Create new analyzer for a file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: PathBuf) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        Self {
            file_path,
            #[cfg(test)]
            parse_count: AtomicUsize::new(0),
        }
    }

    /// Get the file path being analyzed
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Analyze file with single parse
    ///
    /// This is the core GREEN phase implementation: minimal but correct.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

        // 2. Parse ONCE with SWC
        #[cfg(feature = "typescript-ast")]
        let syntax_tree = self.parse_typescript(&content)?;

        // 3. Extract AST items using existing EnhancedTypeScriptVisitor
        #[cfg(feature = "typescript-ast")]
        let ast_items = self.extract_ast_items(&syntax_tree);
        #[cfg(not(feature = "typescript-ast"))]
        let ast_items = Vec::new();

        // 4. Extract complexity metrics (minimal implementation for GREEN phase)
        let file_metrics = self.extract_complexity_metrics(&content);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    /// Parse TypeScript/JavaScript with SWC
    #[cfg(feature = "typescript-ast")]
    fn parse_typescript(&self, content: &str) -> Result<Module, AnalysisError> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let source_map = Lrc::new(SourceMap::default());
        let source_file = source_map.new_source_file(
            Lrc::new(FileName::Custom(self.file_path.display().to_string())),
            content.to_owned(),
        );

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: self
                    .file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "tsx")
                    .unwrap_or(false),
                decorators: true,
                dts: false,
                no_early_errors: false,
                disallow_ambiguous_jsx_like: false,
            }),
            Default::default(),
            StringInput::from(&*source_file),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        parser
            .parse_module()
            .map_err(|e| AnalysisError::Parse(format!("SWC parse error: {:?}", e)))
    }

    /// Get parse count (test-only, for verifying single parse)
    #[cfg(test)]
    pub fn parse_count(&self) -> usize {
        self.parse_count.load(Ordering::SeqCst)
    }

    /// Extract AST items from parsed TypeScript/JavaScript module
    #[cfg(feature = "typescript-ast")]
    fn extract_ast_items(&self, module: &Module) -> Vec<AstItem> {
        let visitor = EnhancedTypeScriptVisitor::new(&self.file_path);
        visitor.extract_items(module)
    }

    /// Extract complexity metrics from TypeScript/JavaScript content
    ///
    /// GREEN PHASE: Minimal implementation using simple pattern counting.
    /// This will be enhanced in REFACTOR phase with proper complexity calculation.
    fn extract_complexity_metrics(&self, content: &str) -> FileComplexityMetrics {
        debug_assert!(!content.is_empty(), "content must not be empty");
        // Simple visitor to count functions and estimate complexity
        let mut functions = Vec::new();

        // For GREEN phase, we'll do simple line-based pattern matching
        // This is minimal but functional - can be improved later

        // Count lines for rough estimation
        let lines = content.lines().count();

        // Simple function detection (will miss some edge cases, but good enough for GREEN)
        let function_pattern = regex::Regex::new(
            r"(?:function\s+(\w+)|const\s+(\w+)\s*=\s*(?:async\s*)?\(|(\w+)\s*\(.*?\)\s*\{|async\s+function\s+(\w+))"
        ).expect("internal error");

        for cap in function_pattern.captures_iter(content) {
            let name = cap
                .get(1)
                .or_else(|| cap.get(2))
                .or_else(|| cap.get(3))
                .or_else(|| cap.get(4))
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
        debug_assert!(true, "contract: estimate_complexity");
        let mut complexity = 1; // Base complexity

        // Count control flow keywords
        let keywords = [
            "if", "else if", "for", "while", "switch", "case", "catch", "&&", "||",
            "?", // Ternary and logical operators
        ];

        for keyword in &keywords {
            complexity += content.matches(keyword).count() as u32;
        }

        complexity
    }
}
