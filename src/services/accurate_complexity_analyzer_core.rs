// Included from accurate_complexity_analyzer.rs — NO use imports or #! attributes

impl AccurateComplexityAnalyzer {
    /// Analyze a single Rust file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_file(&self, path: &Path) -> Result<FileComplexityResult> {
        let content = tokio::fs::read_to_string(path).await?;
        let ast = syn::parse_file(&content)?;

        // Measured spans (start AND end) for every function in the source text.
        let mut line_map = build_function_line_map(&content);
        let total_lines = line_map.total_lines();

        let mut functions = Vec::new();

        // Every function, not just the free ones: methods in `impl` blocks,
        // trait default methods and functions inside inline `mod` blocks all
        // used to be invisible here, so idiomatic Rust measured as 0 functions.
        for func in collect_functions(&ast.items) {
            // Spans are consumed in textual order so duplicate names in
            // different scopes don't all collapse onto the first one.
            let span = line_map.take(&func.name).unwrap_or(LineSpan::UNKNOWN);
            functions.push(self.measure_discovered(&func, span));
        }

        Ok(FileComplexityResult {
            functions,
            file_path: path.display().to_string(),
            total_lines,
        })
    }

    /// Analyze an entire project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_project(&self, path: &Path) -> Result<ProjectComplexityResult> {
        let mut file_metrics = Vec::new();
        let mut files_analyzed = 0;

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let file_path = entry.path();

            // Skip test files if requested
            if self.exclude_tests && self.is_test_file(file_path) {
                continue;
            }

            if let Ok(result) = self.analyze_file(file_path).await {
                files_analyzed += 1;
                file_metrics.push(result);
            }
        }

        Ok(ProjectComplexityResult {
            files_analyzed,
            file_metrics,
        })
    }

    /// Analyze a single free function (kept for callers holding an `ItemFn`).
    fn analyze_function(&self, func: &ItemFn, span: LineSpan) -> FunctionMetrics {
        self.measure_discovered(
            &DiscoveredFn {
                name: func.sig.ident.to_string(),
                attrs: &func.attrs,
                block: &func.block,
            },
            span,
        )
    }

    /// Measure one discovered function body, wherever it was declared.
    fn measure_discovered(&self, func: &DiscoveredFn<'_>, span: LineSpan) -> FunctionMetrics {
        let suppressed = self.respect_annotations && self.has_suppress_annotation(func.attrs);

        let complexity = measure_block(&func.name, func.block);

        FunctionMetrics {
            name: func.name.clone(),
            cyclomatic_complexity: complexity.cyclomatic,
            cognitive_complexity: complexity.cognitive,
            max_nesting: complexity.max_nesting,
            suppressed,
            line_start: span.start,
            line_end: span.end,
        }
    }

    /// Check if file is a test file
    fn is_test_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/tests/")
            || path_str.contains("/test/")
            || path_str.ends_with("_test.rs")
            || path_str.ends_with("_tests.rs")
            || path_str.contains("test_")
            || path_str.contains("tests.rs")
    }

    /// Check if function has suppression annotation
    fn has_suppress_annotation(&self, attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            // Check if it's an allow attribute
            if attr.path().is_ident("allow") {
                // Check if it contains complex_function
                // In syn 2.0, we need to parse the token stream differently
                let tokens_str = attr
                    .meta
                    .require_list()
                    .map(|list| list.tokens.to_string())
                    .unwrap_or_default();
                tokens_str.contains("complex_function")
            } else {
                false
            }
        })
    }
}
